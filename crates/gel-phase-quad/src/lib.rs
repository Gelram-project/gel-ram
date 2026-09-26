//! One canonical Q8 record, four reversible views, one shared similarity.
//! Scores apply only to matching poles of this Grid256 geometry.
#![forbid(unsafe_code)]
pub mod bound_view;
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
        for (j, &yes) in active.iter().enumerate() {
            if yes {
                mask[j / 64] |= 1u64 << (j % 64);
            }
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
            cos: std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos()),
        }
    }
    pub fn prepare(&self, record: Record) -> Query {
        let active = (0..DIM).filter(|&j| record.active(j).unwrap()).collect();
        Query { record, active }
    }
    pub fn view(&self, record: &Record, pole: u8) -> Result<View, String> {
        Ok(View {
            phase: self.grid.view(&record.phase, pole)?,
            active: Grid::support_view(&record.active_mask(), pole)?,
        })
    }
    pub fn read(&self, q: &Query, body: &Record, policy: Policy) -> Option<SharedScore> {
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
            .zip(self.cos)
            .map(|(&n, c)| n as f64 * c)
            .sum::<f64>()
            / q.active.len() as f64;
        Some(SharedScore(s))
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
            for (record, score) in records.iter().zip(scores) {
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
