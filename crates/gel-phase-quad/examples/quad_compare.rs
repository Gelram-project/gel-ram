//! Synthetic comparison, not a text encoder, accuracy test or optimal-single baseline.
#![forbid(unsafe_code)]
use gel_phase_quad::{effective_workers, grid, Policy, Reader, Record, SharedScore};
#[path = "../src/reference.rs"]
mod reference;
use grid::{Grid, DIM};
use reference::{frames, indices, mean4, score, Frame};
use std::hint::black_box;
use std::time::Instant;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
struct Options {
    orbs: usize,
    rounds: usize,
    workers: usize,
    sparse: bool,
    masked: bool,
}
fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        orbs: 512,
        rounds: 9,
        workers: 1,
        sparse: false,
        masked: true,
    };
    if args.len() % 2 != 0 {
        return Err("expected option/value pairs".into());
    }
    for pair in args.chunks_exact(2) {
        let n = || {
            pair[1]
                .parse::<usize>()
                .map_err(|_| "invalid unsigned integer".to_string())
        };
        match pair[0].as_str() {
            "--orbs" => o.orbs = n()?,
            "--rounds" => o.rounds = n()?,
            "--workers" => o.workers = n()?,
            "--sparse" => {
                o.sparse = match pair[1].as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("sparse must be 0 or 1".into()),
                }
            }
            "--policy" => {
                o.masked = match pair[1].as_str() {
                    "active" => true,
                    "archive" => false,
                    _ => return Err("policy must be active or archive".into()),
                }
            }
            _ => return Err("unknown option".into()),
        }
    }
    if !(1..=32768).contains(&o.orbs) || !(1..=99).contains(&o.rounds) || o.workers == 0 {
        return Err("limits: orbs 1..32768, rounds 1..99, workers >=1".into());
    }
    Ok(o)
}

// The test-only injector rejects one chosen spawn in this local instance.
// Successful attempts still use real OS threads; production has no injection state.
#[derive(Default)]
struct ReferenceSpawner {
    #[cfg(test)]
    fail_at: Option<usize>,
    #[cfg(test)]
    attempts: usize,
    #[cfg(test)]
    started: usize,
}
impl ReferenceSpawner {
    fn spawn<'scope, 'env, F>(
        &mut self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope,
    {
        #[cfg(test)]
        {
            let attempt = self.attempts;
            self.attempts += 1;
            if self.fail_at == Some(attempt) {
                return Err(std::io::Error::other("injected reference worker refusal"));
            }
        }
        let handle = std::thread::Builder::new().spawn_scoped(scope, work)?;
        #[cfg(test)]
        {
            self.started += 1;
        }
        Ok(handle)
    }
}

// Returns true only if a failed spawn required a complete serial recomputation.
fn parallel_fill<T: Sync, U: Send>(
    bank: &[T],
    output: &mut [U],
    workers: usize,
    fill: impl Fn(&[T], &mut [U]) + Sync,
    spawner: &mut ReferenceSpawner,
) -> bool {
    assert_eq!(bank.len(), output.len());
    assert!(workers > 0);
    if workers == 1 || bank.is_empty() {
        fill(bank, output);
        return false;
    }
    let chunk = bank.len().div_ceil(workers);
    let fill = &fill;
    let failed = std::thread::scope(|scope| {
        let (first, rest) = bank.split_at(chunk);
        let (first_out, rest_out) = output.split_at_mut(chunk);
        for (records, scores) in rest.chunks(chunk).zip(rest_out.chunks_mut(chunk)) {
            if spawner.spawn(scope, move || fill(records, scores)).is_err() {
                return true;
            }
        }
        fill(first, first_out);
        false
    }); // All successfully started workers have joined before output is reused.
    if failed {
        fill(bank, output);
    }
    failed
}

fn reference_scan<const P: usize>(
    query: &[Frame; 4],
    active: &[Vec<usize>; 4],
    bank: &[[Frame; 4]],
    masked: bool,
    cos: &[f64; 256],
    workers: usize,
    output: &mut [[f64; P]],
) -> bool {
    let fill = |records: &[[Frame; 4]], values: &mut [[f64; P]]| {
        for (record, value) in records.iter().zip(values) {
            *value = std::array::from_fn(|pole| {
                score(&query[pole], &record[pole], &active[pole], masked, cos).unwrap()
            });
        }
    };
    parallel_fill(
        bank,
        output,
        workers,
        fill,
        &mut ReferenceSpawner::default(),
    )
}

fn run(o: Options) -> Result<(), String> {
    const SEED: u64 = 510051;
    let workers = effective_workers(o.workers, o.orbs)?;
    let grid = Grid::new(256, SEED)?;
    let reader = Reader::new(SEED);
    let mut rng = Rng(SEED);
    let bank: Vec<_> = (0..o.orbs)
        .map(|_| {
            let phase = std::array::from_fn(|_| rng.next() as u8);
            let mask = std::array::from_fn(|j| !o.sparse || j == 0 || rng.next() % 4 == 0);
            Record::new(phase, &mask)
        })
        .collect();
    let views: Vec<_> = bank
        .iter()
        .map(|r| frames(r.phase(), &r.active_mask(), &grid))
        .collect();
    // Confirm every stored coordinate and support view before taking timings.
    for (r, vs) in bank.iter().zip(&views) {
        for (p, v) in vs.iter().enumerate() {
            if grid.invert(&v.phase, p as u8)? != *r.phase()
                || Grid::support_view(&v.active, p as u8)? != r.active_mask()
            {
                return Err("view roundtrip mismatch".into());
            }
        }
    }
    let cos = std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos());
    let policy = if o.masked {
        Policy::BodyActivity
    } else {
        Policy::Archive
    };
    let mut one = vec![[0.0; 1]; o.orbs];
    let mut four = vec![[0.0; 4]; o.orbs];
    let mut shared = Vec::<SharedScore>::with_capacity(o.orbs);
    let mut sums = [0u128; 3];
    let mut checked = 0usize;
    let mut reference_fallbacks = [0usize; 2];
    let mut shared_fallbacks = 0usize;
    println!("Q8_QUAD_COMPARE_V3 seed={SEED} arch={} profile={} orbs={} rounds={} warmup_rounds=1 requested_workers={} effective_workers={} sparse={} policy={}",
        std::env::consts::ARCH, if cfg!(debug_assertions) {"debug"} else {"release"}, o.orbs, o.rounds, o.workers, workers, o.sparse, if o.masked {"active"} else {"archive"});
    println!("canonical_record_bytes={} materialized_four_frame_bytes={} packed_four_record_bytes={} timing=scan_only_reused_output_views_precomputed host_isolation=false",
        std::mem::size_of::<Record>(), std::mem::size_of::<[Frame; 4]>(), 4 * std::mem::size_of::<Record>());
    println!("round,reference_one_ns,four_views_ns,shared_ns");
    for round in 0..=o.rounds {
        let index = round * 65537 % o.orbs;
        let mut phase = *bank[index].phase();
        phase[round % DIM] = phase[round % DIM].wrapping_add(17);
        let q_record = Record::new(phase, &bank[index].active_mask());
        let q_views = frames(q_record.phase(), &q_record.active_mask(), &grid);
        let q_indices = std::array::from_fn(|p| indices(&q_views[p]));
        let q = reader.prepare(q_record);
        let mut elapsed = [0u128; 3];
        for offset in 0..3 {
            let method = (round + offset) % 3;
            let start = Instant::now();
            match method {
                0 => {
                    reference_fallbacks[0] += usize::from(reference_scan(
                        black_box(&q_views),
                        &q_indices,
                        black_box(&views),
                        o.masked,
                        &cos,
                        workers,
                        &mut one,
                    ));
                    black_box(&one);
                }
                1 => {
                    reference_fallbacks[1] += usize::from(reference_scan(
                        black_box(&q_views),
                        &q_indices,
                        black_box(&views),
                        o.masked,
                        &cos,
                        workers,
                        &mut four,
                    ));
                    black_box(&four);
                }
                _ => {
                    let report = reader.scan_into_report(
                        black_box(&q),
                        black_box(&bank),
                        policy,
                        o.workers,
                        &mut shared,
                    )?;
                    shared_fallbacks += usize::from(report.serial_fallback);
                    black_box(&shared);
                }
            }
            elapsed[method] = start.elapsed().as_nanos();
        }
        for i in 0..o.orbs {
            let expected = shared[i].value().to_bits();
            if one[i][0].to_bits() != expected
                || mean4(four[i]).to_bits() != expected
                || four[i].iter().any(|v| v.to_bits() != expected)
            {
                return Err(format!("score mismatch at round={round}, record={i}"));
            }
            checked += 4;
        }
        if round > 0 {
            println!("{round},{},{},{}", elapsed[0], elapsed[1], elapsed[2]);
            for i in 0..3 {
                sums[i] += elapsed[i];
            }
        }
    }
    println!(
        "reference_one_fallback_scans={} four_views_fallback_scans={} shared_fallback_scans={} warmup_included=true",
        reference_fallbacks[0], reference_fallbacks[1], shared_fallbacks
    );
    if shared_fallbacks > 0 {
        println!("TIMING_COMPARISON=DEGRADED_SHARED_SERIAL_FALLBACK");
    }
    if reference_fallbacks.iter().any(|&count| count > 0) {
        println!("TIMING_COMPARISON=DEGRADED_REFERENCE_SERIAL_FALLBACK");
    }
    println!("four_over_shared_total={:.6} reference_one_over_shared_total={:.6} score_view_comparisons={checked}", sums[1] as f64 / sums[2] as f64, sums[0] as f64 / sums[2] as f64);
    println!("Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED");
    Ok(())
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("quad_compare [--orbs 1..32768] [--rounds 1..99] [--workers N] [--sparse 0|1] [--policy active|archive]\nSynthetic, scan-only; four reversible views are not independent knowledge. ReferenceOne uses the same materialized four-view layout, not an optimal canonical single reader.");
        return std::process::ExitCode::SUCCESS;
    }
    match options(&args).and_then(run) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Q8_QUAD=ERROR {e}");
            std::process::ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    #[test]
    fn bounds_are_checked_before_allocating() {
        for args in [
            vec!["--orbs", "0"],
            vec!["--orbs", "32769"],
            vec!["--rounds", "100"],
            vec!["--workers", "0"],
            vec!["--sparse", "2"],
            vec!["--policy", "bad"],
            vec!["--orbs"],
            vec!["--wat", "1"],
        ] {
            assert!(options(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_err());
        }
        assert!(options(&["--workers".into(), usize::MAX.to_string()]).is_ok());
    }

    fn compare_reference<const P: usize>(fail_at: Option<usize>) {
        let grid = Grid::new(256, 510051).unwrap();
        let mut rng = Rng(510051);
        for sparse in [false, true] {
            let bank: Vec<_> = (0..11)
                .map(|_| {
                    let phase = std::array::from_fn(|_| rng.next() as u8);
                    let mask = std::array::from_fn(|j| !sparse || j == 0 || rng.next() % 4 == 0);
                    frames(&phase, &mask, &grid)
                })
                .collect();
            let query = &bank[0];
            let active = std::array::from_fn(|p| indices(&query[p]));
            let cos = std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos());
            for masked in [false, true] {
                let mut expected = vec![[f64::NAN; P]; bank.len()];
                assert!(!reference_scan(
                    query,
                    &active,
                    &bank,
                    masked,
                    &cos,
                    1,
                    &mut expected
                ));
                let mut output = vec![[f64::NAN; P]; bank.len()];
                let mut spawner = ReferenceSpawner {
                    fail_at,
                    ..Default::default()
                };
                let completed = std::sync::atomic::AtomicUsize::new(0);
                let fallback = parallel_fill(
                    &bank,
                    &mut output,
                    4,
                    |records, values| {
                        reference_scan(query, &active, records, masked, &cos, 1, values);
                        completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    },
                    &mut spawner,
                );
                assert_eq!(fallback, fail_at.is_some());
                assert_eq!(spawner.started, fail_at.unwrap_or(3));
                assert_eq!(spawner.attempts, fail_at.map_or(3, |at| at + 1));
                // Children and the caller's fill (or full fallback) must all be done.
                assert_eq!(
                    completed.load(std::sync::atomic::Ordering::SeqCst),
                    spawner.started + 1
                );
                for (actual, expected) in output.iter().zip(&expected) {
                    assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
                }
                // Reusing the output after fallback must not expose stale or racing writes.
                output.fill([f64::NAN; P]);
                reference_scan(query, &active, &bank, masked, &cos, 1, &mut output);
                for (actual, expected) in output.iter().zip(&expected) {
                    assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
                }
            }
        }
    }

    #[test]
    fn reference_one_recovers_from_first_spawn_failure() {
        compare_reference::<1>(Some(0));
    }
    #[test]
    fn reference_four_recovers_from_first_spawn_failure() {
        compare_reference::<4>(Some(0));
    }
    #[test]
    fn reference_one_recovers_after_partial_launch() {
        for failure in [1, 2] {
            compare_reference::<1>(Some(failure));
        }
    }
    #[test]
    fn reference_four_recovers_after_partial_launch() {
        for failure in [1, 2] {
            compare_reference::<4>(Some(failure));
        }
    }
    #[test]
    fn reference_parallel_matches_serial_without_failure() {
        compare_reference::<1>(None);
        compare_reference::<4>(None);
    }
    #[test]
    fn serial_and_empty_bank_do_not_spawn() {
        let mut spawner = ReferenceSpawner {
            fail_at: Some(0),
            ..Default::default()
        };
        let fill = |records: &[u64], output: &mut [u64]| output.copy_from_slice(records);
        let mut output = [0; 3];
        assert!(!parallel_fill(
            &[3, 5, 7],
            &mut output,
            1,
            fill,
            &mut spawner
        ));
        assert_eq!(output, [3, 5, 7]);
        assert!(!parallel_fill(&[], &mut [], 24, fill, &mut spawner));
        assert_eq!(spawner.attempts, 0);
    }
}
