//! One canonical Q8 record, four reversible views, one shared similarity.
//! Scores apply only to matching poles of this Grid256 geometry.
#![forbid(unsafe_code)]
pub mod bound_view;
pub mod fixture;
pub mod grid;
use grid::{Carrier, Grid, DIM};
#[cfg(test)]
#[allow(dead_code)]
#[path = "../../../docs/evidence-q8-current/summarize.rs"]
mod evidence_summary_regressions;
#[cfg(test)]
mod reference;

#[derive(Clone)]
pub struct Record {
    phase: Carrier,
    mask: [u64; DIM / 64],
}
impl Record {
    pub fn new(phase: Carrier, active: &[bool; DIM]) -> Self {
        let mut mask = [0; DIM / 64];
        for (word, bits) in mask.iter_mut().zip(active.chunks_exact(64)) {
            *word = bits
                .iter()
                .enumerate()
                .fold(0u64, |w, (b, &yes)| w | (u64::from(yes) << b));
        }
        Self { phase, mask }
    }
    pub fn phase(&self) -> &Carrier {
        &self.phase
    }
    pub fn active(&self, j: usize) -> Option<bool> {
        (j < DIM).then(|| (self.mask[j / 64] >> (j % 64)) & 1 != 0)
    }
    pub fn active_mask(&self) -> [bool; DIM] {
        std::array::from_fn(|j| (self.mask[j / 64] >> (j % 64)) & 1 != 0)
    }
}
pub struct View {
    pub phase: Carrier,
    pub active: [bool; DIM],
}
pub struct Query {
    record: Record,
    active: Vec<usize>,
}
#[derive(Clone, Copy)]
pub enum Policy {
    /// Historical mode: body activity is ignored; inactive body phases may contribute.
    Archive,
    /// Inactive body dimensions contribute zero; denominator is query support.
    BodyActivity,
}

/// Per-call ceiling, including the calling thread. Not a global thread pool.
pub const MAX_WORKERS: usize = 64;

fn budget(requested: usize, records: usize, available: usize) -> Result<usize, &'static str> {
    if requested == 0 {
        return Err("worker count must be positive");
    }
    Ok(requested
        .min(available.max(1))
        .min(MAX_WORKERS)
        .min(records))
}

/// Actual per-call budget, capped by OS-reported parallelism and MAX_WORKERS.
/// Zero records need no workers; zero requested workers is an error.
pub fn effective_workers(requested: usize, records: usize) -> Result<usize, &'static str> {
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    budget(requested, records, available)
}
/// Bodies scored together inside one scan chunk.
const LANES: usize = 4;

/// Calls `f` with the phase difference of every position that counts: query
/// support, intersected with body activity unless the policy is Archive.
/// Positions are visited word by word over the set mask bits.
#[inline(always)]
fn for_each_difference(q: &Query, body: &Record, policy: Policy, mut f: impl FnMut(usize)) {
    let archive = matches!(policy, Policy::Archive);
    for (w, (&qm, &bm)) in q.record.mask.iter().zip(&body.mask).enumerate() {
        let mut m = if archive { qm } else { qm & bm };
        while m != 0 {
            let j = w * 64 + m.trailing_zeros() as usize;
            f(q.record.phase[j].wrapping_sub(body.phase[j]) as usize);
            m &= m - 1;
        }
    }
}

/// Execution facts of one scan, for timing reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanReport {
    /// `false` for an UNKNOWN (empty) query; output was cleared.
    pub scored: bool,
    /// A thread failed to start, so the complete scan ran serially.
    pub serial_fallback: bool,
}

/// One piece of evidence, not four independent confidence votes.
#[derive(Clone, Copy, Default)]
pub struct SharedScore(f64);
impl SharedScore {
    pub fn value(self) -> f64 {
        self.0
    }
    pub fn at_pole(self, pole: u8) -> Result<f64, &'static str> {
        if pole > 3 {
            Err("pole must be 0..3")
        } else {
            Ok(self.0)
        }
    }
}
pub struct Reader {
    grid: Grid,
    cos: [f64; 256],
}
impl Reader {
    pub fn new(seed: u64) -> Self {
        Self {
            grid: Grid::new(256, seed).expect("fixed valid grid"),
            cos: grid::cos256(),
        }
    }
    pub fn prepare(&self, record: Record) -> Query {
        let support = record.mask.iter().map(|w| w.count_ones() as usize).sum();
        let mut active = Vec::with_capacity(support);
        for (w, &word) in record.mask.iter().enumerate() {
            let mut m = word;
            while m != 0 {
                active.push(w * 64 + m.trailing_zeros() as usize);
                m &= m - 1;
            }
        }
        Query { record, active }
    }
    pub fn view(&self, record: &Record, pole: u8) -> Result<View, String> {
        let phase = self.grid.view(&record.phase, pole)?;
        // Same support as `Grid::support_view(&record.active_mask(), pole)`:
        // poles 2 and 3 read position DIM-1-i, which is bit 63-i%64 of word
        // 15-i/64, i.e. bit i%64 of that word reversed.
        let words: [u64; DIM / 64] = if pole & 2 != 0 {
            std::array::from_fn(|w| record.mask[DIM / 64 - 1 - w].reverse_bits())
        } else {
            record.mask
        };
        Ok(View {
            phase,
            active: std::array::from_fn(|i| (words[i / 64] >> (i % 64)) & 1 != 0),
        })
    }
    pub fn read(&self, q: &Query, body: &Record, policy: Policy) -> Option<SharedScore> {
        if q.active.is_empty() {
            return None;
        }
        let mut hist = [0u32; 256];
        for_each_difference(q, body, policy, |d| hist[d] += 1);
        let s = hist
            .iter()
            .zip(self.cos)
            .map(|(&n, c)| n as f64 * c)
            .sum::<f64>()
            / q.active.len() as f64;
        Some(SharedScore(s))
    }
    /// Scores `LANES` bodies of a nonempty query at once. Each lane runs the
    /// same multiply-then-add sequence over bins 0..255 as [`Self::read`],
    /// followed by the same division, so every score keeps its bits.
    fn read_lanes(&self, q: &Query, bodies: &[Record], policy: Policy, scores: &mut [SharedScore]) {
        let mut hist = [[0u32; LANES]; 256];
        for (lane, body) in bodies.iter().enumerate() {
            for_each_difference(q, body, policy, |d| hist[d][lane] += 1);
        }
        let mut acc = [-0.0f64; LANES];
        for (counts, &c) in hist.iter().zip(&self.cos) {
            for (a, &n) in acc.iter_mut().zip(counts) {
                *a += n as f64 * c;
            }
        }
        let support = q.active.len() as f64;
        for (score, a) in scores.iter_mut().zip(acc) {
            *score = SharedScore(a / support);
        }
    }
    /// `false` = UNKNOWN query; clears stale output. `true` may include an empty bank.
    /// Reuses the allocation; only canonical records are scanned.
    /// Caps concurrency including the caller. A failed thread start falls back to
    /// a complete serial scan after all started threads finish. Errors clear output.
    /// Timing reports should use [`Self::scan_into_report`], which exposes that fallback.
    pub fn scan_into(
        &self,
        q: &Query,
        bank: &[Record],
        policy: Policy,
        workers: usize,
        output: &mut Vec<SharedScore>,
    ) -> Result<bool, &'static str> {
        self.scan_into_report(q, bank, policy, workers, output)
            .map(|report| report.scored)
    }
    /// Same scan as [`Self::scan_into`], also reporting whether a requested
    /// parallel scan completed on the serial fallback path.
    pub fn scan_into_report(
        &self,
        q: &Query,
        bank: &[Record],
        policy: Policy,
        workers: usize,
        output: &mut Vec<SharedScore>,
    ) -> Result<ScanReport, &'static str> {
        let workers = effective_workers(workers, bank.len()).inspect_err(|_| output.clear())?;
        if q.active.is_empty() {
            output.clear();
            return Ok(ScanReport {
                scored: false,
                serial_fallback: false,
            });
        }
        output
            .try_reserve(bank.len().saturating_sub(output.len()))
            .map_err(|_| {
                output.clear();
                "score buffer allocation failed"
            })?;
        output.resize(bank.len(), SharedScore::default());
        if bank.is_empty() {
            return Ok(ScanReport {
                scored: true,
                serial_fallback: false,
            });
        }
        let fill = |records: &[Record], scores: &mut [SharedScore]| {
            let mut groups = records.chunks_exact(LANES);
            let mut outs = scores.chunks_exact_mut(LANES);
            for (group, out) in (&mut groups).zip(&mut outs) {
                self.read_lanes(q, group, policy, out);
            }
            for (record, score) in groups.remainder().iter().zip(outs.into_remainder()) {
                *score = self
                    .read(q, record, policy)
                    .expect("nonempty prepared query");
            }
        };
        if workers == 1 {
            fill(bank, output);
            return Ok(ScanReport {
                scored: true,
                serial_fallback: false,
            });
        }
        let chunk = bank.len().div_ceil(workers);
        let failed = std::thread::scope(|scope| {
            let (first_records, rest_records) = bank.split_at(chunk);
            let (first_scores, rest_scores) = output.split_at_mut(chunk);
            for (records, scores) in rest_records
                .chunks(chunk)
                .zip(rest_scores.chunks_mut(chunk))
            {
                if std::thread::Builder::new()
                    .spawn_scoped(scope, move || fill(records, scores))
                    .is_err()
                {
                    return true;
                }
            }
            fill(first_records, first_scores);
            false
        });
        if failed {
            fill(bank, output);
        }
        Ok(ScanReport {
            scored: true,
            serial_fallback: failed,
        })
    }
    /// Scores several queries against one bank in a single pass over it.
    ///
    /// `output` is row-major: `output[q * bank.len() + b]` is the score of
    /// query `q` for body `b`, and each row has the same bits as the output of
    /// [`Self::scan_into`] for that query alone. `scored[q]` is `false` for an
    /// UNKNOWN (empty) query, whose row holds `SharedScore::default()`.
    /// Each group of bodies is scored against every query before the next
    /// group is read. The bank is split into the same contiguous chunks as in
    /// [`Self::scan_into_report`], with the same worker budget and the same
    /// complete serial fallback after a failed thread start. Errors clear
    /// output; the allocation is reused.
    pub fn scan_many_into(
        &self,
        queries: &[Query],
        bank: &[Record],
        policy: Policy,
        workers: usize,
        output: &mut Vec<SharedScore>,
    ) -> Result<ScanManyReport, &'static str> {
        let workers = effective_workers(workers, bank.len()).inspect_err(|_| output.clear())?;
        let n = bank.len();
        let total = queries.len().checked_mul(n).ok_or_else(|| {
            output.clear();
            "score buffer allocation failed"
        })?;
        output
            .try_reserve(total.saturating_sub(output.len()))
            .map_err(|_| {
                output.clear();
                "score buffer allocation failed"
            })?;
        output.resize(total, SharedScore::default());
        let mut report = ScanManyReport {
            scored: queries.iter().map(|q| !q.active.is_empty()).collect(),
            serial_fallback: false,
        };
        if total == 0 {
            return Ok(report);
        }
        // `rows[q]` is the part of query `q`'s row that belongs to `records`.
        let fill = |records: &[Record], rows: &mut [&mut [SharedScore]]| {
            for (q, row) in queries.iter().zip(rows.iter_mut()) {
                if q.active.is_empty() {
                    row.fill(SharedScore::default());
                }
            }
            let mut groups = records.chunks_exact(LANES);
            let mut at = 0;
            for group in &mut groups {
                for (q, row) in queries.iter().zip(rows.iter_mut()) {
                    if !q.active.is_empty() {
                        self.read_lanes(q, group, policy, &mut row[at..at + LANES]);
                    }
                }
                at += LANES;
            }
            for record in groups.remainder() {
                for (q, row) in queries.iter().zip(rows.iter_mut()) {
                    if !q.active.is_empty() {
                        row[at] = self
                            .read(q, record, policy)
                            .expect("nonempty prepared query");
                    }
                }
                at += 1;
            }
        };
        if workers == 1 {
            let mut rows: Vec<&mut [SharedScore]> = output.chunks_exact_mut(n).collect();
            fill(bank, &mut rows);
            return Ok(report);
        }
        let chunk = n.div_ceil(workers);
        let failed = {
            let mut parts: Vec<Vec<&mut [SharedScore]>> = (0..n.div_ceil(chunk))
                .map(|_| Vec::with_capacity(queries.len()))
                .collect();
            for row in output.chunks_exact_mut(n) {
                for (piece, part) in row.chunks_mut(chunk).zip(parts.iter_mut()) {
                    part.push(piece);
                }
            }
            std::thread::scope(|scope| {
                let mut work = bank.chunks(chunk).zip(parts.iter_mut());
                let (first_records, first_rows) = work.next().expect("nonempty bank");
                for (records, rows) in work {
                    if std::thread::Builder::new()
                        .spawn_scoped(scope, move || fill(records, rows))
                        .is_err()
                    {
                        return true;
                    }
                }
                fill(first_records, first_rows);
                false
            })
        };
        if failed {
            let mut rows: Vec<&mut [SharedScore]> = output.chunks_exact_mut(n).collect();
            fill(bank, &mut rows);
        }
        report.serial_fallback = failed;
        Ok(report)
    }
}

/// Execution facts of one multi-query scan, see [`Reader::scan_many_into`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanManyReport {
    /// One flag per query, in order; `false` = UNKNOWN (empty) query.
    pub scored: Vec<bool>,
    /// A thread failed to start, so the complete scan ran serially.
    pub serial_fallback: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::{frames, indices, score};
    #[test]
    fn worker_budget_extremes_are_bounded_without_spawning() {
        for available in [0, 1, 2, 24, 100, usize::MAX] {
            for records in [0, 1, 2, 129, usize::MAX] {
                for requested in [1, 2, 24, 100, usize::MAX] {
                    let n = budget(requested, records, available).unwrap();
                    assert!(n <= MAX_WORKERS && n <= records && n <= requested);
                    assert!(n <= available.max(1));
                    assert_eq!(n == 0, records == 0);
                }
            }
        }
        assert!(budget(0, 10, 24).is_err());
        assert!(budget(0, 0, 24).is_err());
    }
    #[test]
    fn large_worker_requests_partial_chunks_and_masks_match_scalar() {
        let reader = Reader::new(17);
        let query = reader.prepare(Record::new(
            std::array::from_fn(|i| (i * 13) as u8),
            &std::array::from_fn(|i| i % 3 != 0),
        ));
        let bank: Vec<_> = (0..129)
            .map(|n| {
                Record::new(
                    std::array::from_fn(|i| (n * 37 + i * 23) as u8),
                    &std::array::from_fn(|i| (i + n) % 5 != 0),
                )
            })
            .collect();
        let mut out = Vec::new();
        for len in [1, 2, 7, 129] {
            for policy in [Policy::Archive, Policy::BodyActivity] {
                let expected: Vec<_> = bank[..len]
                    .iter()
                    .map(|r| reader.read(&query, r, policy).unwrap().value().to_bits())
                    .collect();
                for workers in [1, 2, 7, 24, usize::MAX] {
                    assert!(reader
                        .scan_into(&query, &bank[..len], policy, workers, &mut out)
                        .unwrap());
                    assert_eq!(
                        out.iter().map(|x| x.value().to_bits()).collect::<Vec<_>>(),
                        expected
                    );
                }
            }
        }
    }
    /// The per-position scoring loop that `read` and `scan_into` replaced,
    /// kept verbatim as the differential reference.
    fn read_reference(
        reader: &Reader,
        q: &Query,
        body: &Record,
        policy: Policy,
    ) -> Option<SharedScore> {
        if q.active.is_empty() {
            return None;
        }
        let mut hist = [0u32; 256];
        for &j in &q.active {
            if matches!(policy, Policy::Archive) || (body.mask[j / 64] >> (j % 64)) & 1 != 0 {
                hist[q.record.phase[j].wrapping_sub(body.phase[j]) as usize] += 1;
            }
        }
        let s = hist
            .iter()
            .zip(reader.cos)
            .map(|(&n, c)| n as f64 * c)
            .sum::<f64>()
            / q.active.len() as f64;
        Some(SharedScore(s))
    }
    struct SplitMix(u64);
    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        }
        /// Activity with probability `per_256 / 256` at each position.
        fn mask(&mut self, per_256: u64) -> [bool; DIM] {
            std::array::from_fn(|_| self.next() % 256 < per_256)
        }
        fn phase(&mut self) -> Carrier {
            std::array::from_fn(|_| self.next() as u8)
        }
    }
    fn single(bits: &[usize]) -> [bool; DIM] {
        std::array::from_fn(|j| bits.contains(&j))
    }
    #[test]
    fn mask_walk_and_lanes_match_the_per_position_reference_bit_for_bit() {
        let mut checked = 0usize;
        for seed in 0..4u64 {
            let reader = Reader::new(seed * 7919 + 1);
            let mut rng = SplitMix(seed ^ 0x51ed_270b_d1c3_a4f1);
            let base = rng.phase();
            let mut queries: Vec<Record> = Vec::new();
            for per_256 in [0, 1, 4, 64, 128, 171, 252, 256] {
                queries.push(Record::new(rng.phase(), &rng.mask(per_256)));
            }
            for bits in [
                &[0][..],
                &[63],
                &[64],
                &[1023],
                &[0, 1023],
                &[63, 64, 127, 960],
            ] {
                queries.push(Record::new(rng.phase(), &single(bits)));
            }
            queries.push(Record::new(base, &std::array::from_fn(|j| j >= DIM - 64)));
            let bank: Vec<Record> = (0..1031)
                .map(|n| {
                    let phase = match n % 11 {
                        // Zero and opposite differences against the base query.
                        0 => base,
                        1 => base.map(|p| p.wrapping_add(128)),
                        _ => rng.phase(),
                    };
                    let active = match n % 7 {
                        0 => [false; DIM],
                        1 => [true; DIM],
                        2 => single(&[n % DIM]),
                        k => rng.mask([64, 171, 255, 128][k - 3]),
                    };
                    Record::new(phase, &active)
                })
                .collect();
            let mut out = vec![SharedScore(f64::NAN); 3];
            for record in &queries {
                let q = reader.prepare(record.clone());
                for policy in [Policy::Archive, Policy::BodyActivity] {
                    let expected: Vec<Option<u64>> = bank
                        .iter()
                        .map(|b| {
                            read_reference(&reader, &q, b, policy).map(|s| s.value().to_bits())
                        })
                        .collect();
                    let direct: Vec<Option<u64>> = bank
                        .iter()
                        .map(|b| reader.read(&q, b, policy).map(|s| s.value().to_bits()))
                        .collect();
                    assert_eq!(direct, expected, "read, seed {seed}");
                    for len in (0..=9).chain([129, 1031]) {
                        for workers in [1, 2, 3, 7, usize::MAX] {
                            let scored = reader
                                .scan_into(&q, &bank[..len], policy, workers, &mut out)
                                .unwrap();
                            if q.active.is_empty() {
                                assert!(!scored && out.is_empty());
                                continue;
                            }
                            assert!(scored);
                            let got: Vec<Option<u64>> =
                                out.iter().map(|s| Some(s.value().to_bits())).collect();
                            assert_eq!(got, expected[..len], "scan, seed {seed} len {len}");
                            checked += len;
                        }
                    }
                }
            }
        }
        assert!(checked > 100_000);
    }
    #[test]
    fn scan_many_rows_equal_single_scans_bit_for_bit() {
        let reader = Reader::new(41);
        let mut rng = SplitMix(0x5ca1_ab1e_0000_0001);
        let mut records: Vec<Record> = [0, 1, 64, 171, 256, 0, 128]
            .iter()
            .map(|&p| Record::new(rng.phase(), &rng.mask(p)))
            .collect();
        records.push(Record::new(rng.phase(), &single(&[0, 63, 64, 1023])));
        let queries: Vec<Query> = records.into_iter().map(|r| reader.prepare(r)).collect();
        let bank: Vec<Record> = (0..133)
            .map(|n| Record::new(rng.phase(), &rng.mask([0, 64, 171, 256][n % 4])))
            .collect();
        let (mut many, mut one) = (vec![SharedScore(f64::NAN); 5], Vec::new());
        let mut compared = 0usize;
        for count in [0, 1, 2, 3, queries.len()] {
            let qs = &queries[..count];
            for len in [0, 1, 3, 4, 5, 9, 133] {
                for policy in [Policy::Archive, Policy::BodyActivity] {
                    for workers in [1, 2, 3, 7, usize::MAX] {
                        let report = reader
                            .scan_many_into(qs, &bank[..len], policy, workers, &mut many)
                            .unwrap();
                        assert!(!report.serial_fallback);
                        assert_eq!(report.scored.len(), count);
                        assert_eq!(many.len(), count * len);
                        for (i, q) in qs.iter().enumerate() {
                            let scored = reader
                                .scan_into(q, &bank[..len], policy, workers, &mut one)
                                .unwrap();
                            assert_eq!(report.scored[i], scored);
                            let row = &many[i * len..(i + 1) * len];
                            let got: Vec<u64> = row.iter().map(|s| s.value().to_bits()).collect();
                            let want: Vec<u64> = if scored {
                                one.iter().map(|s| s.value().to_bits()).collect()
                            } else {
                                vec![0; len]
                            };
                            assert_eq!(got, want, "query {i} len {len} workers {workers}");
                            compared += len;
                        }
                    }
                }
            }
        }
        assert!(compared > 10_000);
        assert_eq!(
            reader.scan_many_into(&queries, &bank, Policy::Archive, 0, &mut many),
            Err("worker count must be positive")
        );
        assert!(many.is_empty());
    }
    #[test]
    fn invalid_budget_clears_stale_scores_and_recovers() {
        let reader = Reader::new(0);
        let r = Record::new([0; DIM], &[true; DIM]);
        let q = reader.prepare(r.clone());
        let mut out = vec![SharedScore(99.0)];
        assert!(reader
            .scan_into(&q, std::slice::from_ref(&r), Policy::Archive, 0, &mut out)
            .is_err());
        assert!(out.is_empty());
        assert!(reader
            .scan_into(&q, &[r], Policy::Archive, 1, &mut out)
            .unwrap());
        assert_eq!(out[0].value(), 1.0);
    }
    #[test]
    fn storage_is_1152_bytes_and_every_bit_survives() {
        assert_eq!(std::mem::size_of::<Record>(), 1152);
        for bit in 0..DIM {
            let mut a = [false; DIM];
            a[bit] = true;
            let p = std::array::from_fn(|j| j as u8);
            let r = Record::new(p, &a);
            assert_eq!(*r.phase(), p);
            assert_eq!(r.active_mask(), a);
            assert_eq!(r.active(DIM), None);
        }
    }
    #[test]
    fn prepare_lists_the_same_support_as_the_per_position_filter() {
        let reader = Reader::new(5);
        let mut rng = SplitMix(0x7072_6570_6172_6531);
        let mut masks = vec![[false; DIM], [true; DIM]];
        for bits in [&[0][..], &[63], &[64], &[1023], &[0, 63, 64, 1023]] {
            masks.push(single(bits));
        }
        for per_256 in [1, 64, 171, 255] {
            for _ in 0..16 {
                masks.push(rng.mask(per_256));
            }
        }
        for a in &masks {
            let record = Record::new(rng.phase(), a);
            let expected: Vec<usize> = (0..DIM).filter(|&j| record.active(j).unwrap()).collect();
            assert_eq!(reader.prepare(record).active, expected);
        }
    }
    #[test]
    fn all_views_and_masks_match_reference() {
        for seed in [0, 210021, 999] {
            let reader = Reader::new(seed);
            let grid = Grid::new(256, seed).unwrap();
            let p = std::array::from_fn(|j| (j * 31) as u8);
            let a = std::array::from_fn(|j| j % 3 == 0);
            let r = Record::new(p, &a);
            let reference = frames(&p, &a, &grid);
            for (pole, expected) in reference.iter().enumerate() {
                let v = reader.view(&r, pole as u8).unwrap();
                assert_eq!(v.phase, expected.phase);
                assert_eq!(v.active, expected.active);
                assert_eq!(grid.invert(&v.phase, pole as u8).unwrap(), p);
            }
            assert!(reader.view(&r, 4).is_err());
        }
    }
    #[test]
    fn score_matches_four_independent_reads() {
        let seed = 210021;
        let reader = Reader::new(seed);
        let grid = Grid::new(256, seed).unwrap();
        for k in 0..48 {
            let x = std::array::from_fn(|j| (j * 71 + k * 11) as u8);
            let y = std::array::from_fn(|j| (j * 23 + k * 17) as u8);
            let a = std::array::from_fn(|j| j % 3 != 0);
            let b = std::array::from_fn(|j| (j + k) % 5 != 0);
            let q = reader.prepare(Record::new(x, &a));
            let body = Record::new(y, &b);
            let qv = frames(&x, &a, &grid);
            let bv = frames(&y, &b, &grid);
            for policy in [Policy::Archive, Policy::BodyActivity] {
                let s = reader.read(&q, &body, policy).unwrap();
                for pole in 0..4 {
                    let expected = score(
                        &qv[pole],
                        &bv[pole],
                        &indices(&qv[pole]),
                        matches!(policy, Policy::BodyActivity),
                        &reader.cos,
                    )
                    .unwrap();
                    assert_eq!(s.at_pole(pole as u8).unwrap().to_bits(), expected.to_bits());
                }
                assert!(s.at_pole(4).is_err());
            }
        }
    }
    #[test]
    fn silence_not_active_zero_and_denominator_is_query_support() {
        let reader = Reader::new(0);
        let q = reader.prepare(Record::new([0; DIM], &[true; DIM]));
        let mut a = [false; DIM];
        a[5] = true;
        let b = Record::new([0; DIM], &a);
        assert_eq!(
            reader.read(&q, &b, Policy::BodyActivity).unwrap().value(),
            1.0 / DIM as f64
        );
        assert_eq!(reader.read(&q, &b, Policy::Archive).unwrap().value(), 1.0);
    }
    #[test]
    fn scan_report_matches_scan_and_names_no_fallback_on_normal_paths() {
        let reader = Reader::new(3);
        let r = Record::new([9; DIM], &[true; DIM]);
        let q = reader.prepare(r.clone());
        let bank = vec![r.clone(); 64];
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for workers in [1, 2, 8, 24] {
            let report = reader
                .scan_into_report(&q, &bank, Policy::Archive, workers, &mut a)
                .unwrap();
            let scored = reader
                .scan_into(&q, &bank, Policy::Archive, workers, &mut b)
                .unwrap();
            assert_eq!(
                report,
                ScanReport {
                    scored: true,
                    serial_fallback: false
                }
            );
            assert!(scored);
            assert!(a
                .iter()
                .zip(&b)
                .all(|(x, y)| x.value().to_bits() == y.value().to_bits()));
        }
        let silent = reader.prepare(Record::new([9; DIM], &[false; DIM]));
        assert_eq!(
            reader
                .scan_into_report(&silent, &bank, Policy::Archive, 8, &mut a)
                .unwrap(),
            ScanReport {
                scored: false,
                serial_fallback: false
            }
        );
        assert!(a.is_empty());
    }
    #[test]
    fn unknown_clears_previous_result() {
        let reader = Reader::new(0);
        let r = Record::new([7; DIM], &[false; DIM]);
        let q = reader.prepare(r.clone());
        let mut out = vec![SharedScore(1.0)];
        assert!(reader.read(&q, &r, Policy::Archive).is_none());
        assert!(!reader
            .scan_into(&q, &[r], Policy::Archive, 24, &mut out)
            .unwrap());
        assert!(out.is_empty());
    }
    #[test]
    fn threads_buffers_and_empty_bank_are_stable() {
        let reader = Reader::new(0);
        let r = Record::new([17; DIM], &[true; DIM]);
        let q = reader.prepare(r.clone());
        let bank = vec![r; 17];
        let mut out = Vec::new();
        for workers in [1, 2, 12, 24, 100] {
            assert!(reader
                .scan_into(&q, &bank, Policy::BodyActivity, workers, &mut out)
                .unwrap());
            assert_eq!(out.len(), 17);
            assert!(out.iter().all(|s| s.value() == 1.0));
        }
        let ptr = out.as_ptr();
        reader
            .scan_into(&q, &bank, Policy::BodyActivity, 24, &mut out)
            .unwrap();
        assert_eq!(ptr, out.as_ptr());
        assert!(reader
            .scan_into(&q, &bank, Policy::Archive, 0, &mut out)
            .is_err());
        assert!(reader
            .scan_into(&q, &[], Policy::Archive, 1, &mut out)
            .unwrap());
        assert!(out.is_empty());
    }
}
