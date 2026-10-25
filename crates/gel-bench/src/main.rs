#![forbid(unsafe_code)]

mod setup;
#[cfg(test)]
mod setup_reference;

use gel_core::ORB_BYTES;
use gel_kernel::{backend, KernelBackend};
use gel_orb::Orb1024;
use gel_reader::{
    top1_many, top1_many_threads, top1_threads_report, top_k_progressive, Top1Execution,
    Top1Fallback,
};
use setup::{orb, reference_top_k};
use std::hint::black_box;
use std::time::Instant;

fn positive_arg(index: usize, default: usize, name: &str) -> Result<usize, String> {
    let Some(raw) = std::env::args_os().nth(index) else {
        return Ok(default);
    };
    let raw = raw
        .into_string()
        .map_err(|bad| format!("{name} is not valid UTF-8: {}", bad.to_string_lossy()))?;
    let value = raw
        .parse::<usize>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    if value == 0 {
        Err(format!("{name} must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn percentile(sorted: &[u128], q: f64) -> u128 {
    let index = ((sorted.len() - 1) as f64 * q).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

/// Comma-joined values (no spaces) so a row stays one key=value token.
fn joined(values: &[u128]) -> String {
    values
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Field 39 (`processor`) of a `/proc/<pid>/stat` line: the CPU the task
/// last ran on. The `comm` field (2) may contain spaces and parentheses, so
/// the line is split after its last `)`; field 39 is then the 37th token.
fn processor_field(stat: &str) -> Option<usize> {
    let (_, rest) = stat.rsplit_once(')')?;
    rest.split_ascii_whitespace().nth(36)?.parse().ok()
}

/// `cpu_model=` value: the first Linux /proc/cpuinfo "model name" line, or
/// `unavailable`. It identifies hardware; it does not prove clock or power state.
fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|l| l.strip_prefix("model name"))
                .and_then(|rest| rest.split_once(':'))
                .map(|(_, name)| name.trim().to_owned())
        })
        .unwrap_or_else(|| "unavailable".to_owned())
}

/// `observed_cpu_*=` value: the CPU this thread was last scheduled on, or
/// `unavailable` when `/proc/self/stat` cannot be read or parsed. It records
/// where the process ran at that instant; it does not prove pinning.
fn observed_cpu() -> String {
    std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|stat| processor_field(&stat))
        .map_or_else(|| "unavailable".to_owned(), |cpu| cpu.to_string())
}

/// Timed full scan with actual execution metadata. Requested parallel work
/// may fall back; correctness and execution mode are reported separately.
fn scan<'a>(
    bank: &'a [Orb1024],
    query: &Orb1024,
    threads: usize,
) -> (Option<(usize, u16, &'a Orb1024)>, Top1Execution) {
    top1_threads_report(bank, query, threads)
}

fn require_exact_rounds(exact: usize, rounds: usize) -> Result<(), String> {
    if exact != rounds {
        return Err(format!("timed Top-1 mismatch: {exact}/{rounds}"));
    }
    Ok(())
}

// `reference_top_k` (in `setup`) is the independent per-bit scoring in
// full-sort order, outside the timer. It does not call the ranking
// insertion, progressive bound or score kernel under test.

fn require_top_k(actual: &[(usize, u16)], expected: &[(usize, u16)]) -> Result<(), String> {
    if actual != expected {
        return Err("progressive Top-K differs from independent full-sort reference".into());
    }
    Ok(())
}

fn execution_mode(reports: &[Top1Execution]) -> &'static str {
    if reports.iter().any(|r| r.fallback.is_some()) {
        "mixed_or_serial_fallback"
    } else if reports.iter().any(|r| r.effective_workers > 1) {
        "exact_merge_lowest_index_tie"
    } else {
        "single"
    }
}

/// True when two Top-1 hits name the same bank slot with the same score.
fn same_hit(a: (usize, u16, &Orb1024), b: (usize, u16, &Orb1024)) -> bool {
    a.0 == b.0 && a.1 == b.1 && std::ptr::eq(a.2, b.2)
}

type Hit<'a> = (usize, u16, &'a Orb1024);

/// Bit flips of the quality probes, in output order.
const QUALITY_FLIPS: [usize; 7] = [1, 8, 32, 64, 128, 256, 384];

/// `orb` with its first `flips` probe positions inverted.
fn flipped(orb: &Orb1024, flips: usize) -> Orb1024 {
    let mut words = *orb.words();
    for bit in 0..flips {
        // Odd multiplier creates unique positions modulo 1024.
        let position = (405 * bit + 137) & 1023;
        words[position / 64] ^= 1u64 << (position % 64);
    }
    Orb1024::from_words(words)
}

/// Results of the untimed Top-1 checks.
#[derive(Debug)]
struct Checks<'a> {
    /// Single-thread Top-1 of the null query.
    null_hit: Hit<'a>,
    /// Single-thread Top-1 of `bank[warmup_index]`, when one was requested.
    single_warmup: Option<Hit<'a>>,
    /// Top-1 of each quality probe, in `QUALITY_FLIPS` order.
    flips: Vec<Hit<'a>>,
}

/// The untimed Top-1 checks after the timed loop. The single-thread
/// references (the null query and, when requested, the warm-up query) share
/// one `top1_many` pass; the quality probes share one `top1_many_threads`
/// pass on `workers` threads. Both are identical to one `top1` per query,
/// including the returned bank reference, and every thread is joined before
/// they return. A score of 1024 means all 1024 bits match, so it occurs
/// exactly when the null query equals a bank orb.
fn untimed_checks<'a>(
    bank: &'a [Orb1024],
    null: &Orb1024,
    warmup_index: Option<usize>,
    quality_index: usize,
    workers: usize,
) -> Result<Checks<'a>, String> {
    let mut references = vec![*null];
    references.extend(warmup_index.map(|index| bank[index]));
    let reference_hits = top1_many(bank, &references);
    let null_hit = reference_hits[0].ok_or("empty bank")?;
    if null_hit.1 == 1024 {
        return Err("null query unexpectedly exists in generated bank".into());
    }
    let single_warmup = match warmup_index {
        Some(_) => Some(reference_hits[1].ok_or("empty bank")?),
        None => None,
    };
    let probes: Vec<Orb1024> = QUALITY_FLIPS
        .iter()
        .map(|&flips| flipped(&bank[quality_index], flips))
        .collect();
    let flips = top1_many_threads(bank, &probes, workers)
        .into_iter()
        .map(|hit| hit.ok_or_else(|| "empty bank".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Checks {
        null_hit,
        single_warmup,
        flips,
    })
}

fn run() -> Result<(), String> {
    let cpu_start = observed_cpu();
    let count = positive_arg(1, 1_310_720, "ORB_COUNT")?;
    let rounds = positive_arg(2, 16, "ROUNDS")?;
    let threads = positive_arg(3, 1, "THREADS")?;
    let threads_cap = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .saturating_mul(4);
    if threads > threads_cap {
        return Err(format!(
            "THREADS must not exceed 4 * available_parallelism ({threads_cap})"
        ));
    }
    if threads > 1 && threads > count / 2 {
        return Err("THREADS must satisfy THREADS <= ORB_COUNT / 2".into());
    }
    const GENERATOR_SEED: u64 = 0x4745_4C00_4245_4E43;
    let bank_bytes = count.checked_mul(ORB_BYTES).ok_or("bank size overflow")?;
    let mut bank = Vec::new();
    bank.try_reserve_exact(count)
        .map_err(|_| "benchmark bank does not fit in available memory")?;
    // Every page is first written on this thread, as with the serial fill;
    // then each slot gets its index-pure orb on worker threads.
    bank.resize(count, Orb1024::ZERO);
    setup::fill_bank(
        &mut bank,
        GENERATOR_SEED,
        setup::workers_for(count, setup::BANK_ORBS_PER_WORKER),
    );
    // Joins its worker threads before the warm-up scan below.
    let bank_crc64 = setup::bank_crc64(&bank);

    let warmup_index = count / 2;
    let mut executions = Vec::new();
    let (warmup, execution) = scan(black_box(&bank), black_box(&bank[warmup_index]), threads);
    let warmup = warmup.ok_or("empty bank")?;
    executions.push(execution);
    if warmup.0 != warmup_index || warmup.1 != 1024 {
        return Err("warm-up Top-1 mismatch".into());
    }

    let mut durations = Vec::with_capacity(rounds);
    let mut exact = 0usize;
    let started_all = Instant::now();
    for round in 0..rounds {
        let query_index = (warmup_index + round.wrapping_mul(65_537)) % count;
        let started = Instant::now();
        let (hit, execution) = black_box(scan(
            black_box(&bank),
            black_box(&bank[query_index]),
            threads,
        ));
        durations.push(started.elapsed().as_nanos());
        executions.push(execution);
        let hit = hit.ok_or("timed Top-1 returned no result for nonempty bank")?;
        exact += usize::from(hit.0 == query_index && hit.1 == 1024);
    }
    let elapsed = started_all.elapsed();
    require_exact_rounds(exact, rounds)?;
    // Keep every timed observation in execution order before sorting.
    let durations_in_order = durations.clone();
    durations.sort_unstable();
    let scans = count.checked_mul(rounds).ok_or("scan count overflow")?;
    let seconds = elapsed.as_secs_f64();
    let orbs_per_sec = scans as f64 / seconds;
    let gib_per_sec = scans as f64 * ORB_BYTES as f64 / seconds / (1024.0 * 1024.0 * 1024.0);

    let query_index = count / 3;
    let (progressive, progressive_stats) = top_k_progressive(&bank, &bank[query_index], 8);
    require_top_k(&progressive, &reference_top_k(&bank, &bank[query_index], 8))?;

    let null = orb(0, GENERATOR_SEED ^ 0xA5A5_5A5A_D3C1_B7E9);
    let quality_index = count / 2;
    let checks = untimed_checks(
        &bank,
        &null,
        (threads > 1).then_some(warmup_index),
        quality_index,
        setup::workers_for(count, setup::BANK_ORBS_PER_WORKER),
    )?;
    let null_hit = checks.null_hit;

    // With THREADS > 1 the timed `warmup` came from `top1_threads`; check it
    // against single-thread Top-1 on the same query, and check `top1_threads`
    // against the single-thread null hit, so both directions are covered.
    let thread_scan_exact = if let Some(single_warmup) = checks.single_warmup {
        let (threaded_null, execution) = top1_threads_report(&bank, &null, threads);
        executions.push(execution);
        let threaded_null = threaded_null.ok_or("empty bank")?;
        if !same_hit(warmup, single_warmup) {
            return Err(
                "multi-thread Top-1 differs from single-thread Top-1 on warm-up query".into(),
            );
        }
        if !same_hit(null_hit, threaded_null) {
            return Err("multi-thread Top-1 differs from single-thread Top-1 on null query".into());
        }
        "PASS"
    } else {
        "SINGLE"
    };
    let backend: KernelBackend = backend();

    println!("GEL_BENCH_V5");
    println!(
        "profile={}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    println!("cpu_model={}", cpu_model());
    println!("generator=splitmix64_with_unique_word0");
    println!("generator_seed={GENERATOR_SEED}");
    println!("bank_crc64_ecma={bank_crc64:016x}");
    println!("orbs={count}");
    println!("bank_bytes={bank_bytes}");
    println!("orb_bytes={ORB_BYTES}");
    println!("rounds={rounds}");
    println!("warmup_rounds=1");
    println!("query_schedule=varying-exact-self-stride-65537");
    println!("requested_workers={threads}");
    println!("worker_budget={}", executions[0].worker_budget);
    println!(
        "effective_workers_min={}",
        executions
            .iter()
            .map(|r| r.effective_workers)
            .min()
            .unwrap()
    );
    println!(
        "effective_workers_max={}",
        executions
            .iter()
            .map(|r| r.effective_workers)
            .max()
            .unwrap()
    );
    println!(
        "spawned_workers_total={}",
        executions.iter().map(|r| r.spawned_workers).sum::<usize>()
    );
    println!("execution_reported_scans={}", executions.len());
    println!("execution_scope=warmup+timed+threaded_null_check_if_requested");
    // One key=value token: the Debug text without spaces.
    println!(
        "execution_reports={}",
        format!("{executions:?}").replace(' ', "")
    );
    println!(
        "fallback_scans={}",
        executions.iter().filter(|r| r.fallback.is_some()).count()
    );
    for (label, reason) in [
        ("bank_too_small", Top1Fallback::BankTooSmall),
        ("budget_exceeded", Top1Fallback::BudgetExceeded),
        ("thread_start_failed", Top1Fallback::ThreadStartFailed),
        ("worker_panicked", Top1Fallback::WorkerPanicked),
    ] {
        println!(
            "fallback_{label}={}",
            executions
                .iter()
                .filter(|r| r.fallback == Some(reason))
                .count()
        );
    }
    if executions.iter().any(|r| r.fallback.is_some()) {
        println!("TIMING_COMPARISON=DEGRADED_SERIAL_FALLBACK");
    }
    println!("thread_scan={}", execution_mode(&executions));
    println!("thread_scan_exact={thread_scan_exact}");
    println!("backend={}", backend.describe());
    println!("target_arch={}", std::env::consts::ARCH);
    println!("observed_cpu_start={cpu_start}");
    println!("scanned_orbs={scans}");
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!("query_p50_ns={}", percentile(&durations, 0.50));
    println!("query_p95_ns={}", percentile(&durations, 0.95));
    println!("query_p99_ns={}", percentile(&durations, 0.99));
    println!(
        "query_samples={rounds} percentile_method=sorted[ceil((n-1)*q)] empirical_not_a_tail_bound"
    );
    println!(
        "query_p99_is_max={}",
        percentile(&durations, 0.99) == durations[durations.len() - 1]
    );
    println!("query_ns_execution_order={}", joined(&durations_in_order));
    println!("orbs_per_sec={orbs_per_sec:.3}");
    println!("effective_gib_per_sec={gib_per_sec:.6}");
    println!("top1_exact={exact}/{rounds}");
    println!("progressive_top8_exact=PASS");
    println!("progressive_candidates={}", progressive_stats.candidates);
    println!(
        "progressive_reject_32b={}",
        progressive_stats.rejected_after_32b
    );
    println!(
        "progressive_reject_64b={}",
        progressive_stats.rejected_after_64b
    );
    println!(
        "progressive_full_128b={}",
        progressive_stats.full_128b_scores
    );

    for (flips, hit) in QUALITY_FLIPS.iter().zip(&checks.flips) {
        println!(
            "quality_flips_{flips}=target:{quality_index},hit:{},score:{},survived:{}",
            hit.0,
            hit.1,
            hit.0 == quality_index
        );
    }
    println!("null_top1_index={}", null_hit.0);
    println!("null_top1_score={}", null_hit.1);
    println!("observed_cpu_end={}", observed_cpu());
    Ok(())
}

fn main() -> Result<(), String> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_gate_rejects_one_wrong_hit_and_missing_results() {
        let rounds = 3;
        let mut hits = [(0usize, 1024u16), (1, 1024), (2, 1024)];
        let count = |hits: &[(usize, u16)]| {
            hits.iter()
                .enumerate()
                .filter(|(i, hit)| hit.0 == *i && hit.1 == 1024)
                .count()
        };
        assert!(require_exact_rounds(count(&hits), rounds).is_ok());
        hits[1].1 = 1023;
        assert!(require_exact_rounds(count(&hits), rounds).is_err());
        hits[1] = (0, 1024);
        assert!(require_exact_rounds(count(&hits), rounds).is_err());
        assert!(require_exact_rounds(0, rounds).is_err());
    }

    #[test]
    fn top8_gate_checks_every_index_score_order_and_length() {
        let query = orb(0, 91);
        let bank = vec![query; 12];
        let expected = reference_top_k(&bank, &query, 8);
        assert_eq!(expected, (0..8).map(|i| (i, 1024)).collect::<Vec<_>>());
        assert!(require_top_k(&expected, &expected).is_ok());
        for i in 0..8 {
            let mut corrupt = expected.clone();
            corrupt[i].0 = 11;
            assert!(require_top_k(&corrupt, &expected).is_err());
            corrupt = expected.clone();
            corrupt[i].1 = 1023;
            assert!(require_top_k(&corrupt, &expected).is_err());
        }
        let mut reordered = expected.clone();
        reordered.swap(6, 7);
        assert!(require_top_k(&reordered, &expected).is_err());
        assert!(require_top_k(&expected[..7], &expected).is_err());
        for len in [0, 1, 7, 8, 9, 32] {
            let bank: Vec<_> = (0..len).map(|i| orb(i, 90)).collect();
            let (got, _) = top_k_progressive(&bank, &query, 8);
            assert!(require_top_k(&got, &reference_top_k(&bank, &query, 8)).is_ok());
        }
    }

    #[test]
    fn threaded_setup_matches_the_serial_reference() {
        const SEED: u64 = 0x4745_4C00_4245_4E43;
        let bank = setup_reference::compare_bank(
            &[0, 1, 2, 3, 7, 8, 63, 64, 1000, 4097],
            &[0, 1, 2, 3, 5, 9],
            SEED,
        );
        assert!(bank.all_equal(), "{bank:?}");
        let pins = setup_reference::compare_crc_pins(SEED, &[(8192, 0x6f03_8d6d_b5b8_839f)]);
        assert!(pins.all_equal(), "{pins:?}");
        let small = setup_reference::compare_top_k_small(0..=40, &[1, 2, 3, 5]);
        assert!(small.all_equal(), "{small:?}");
        let big = setup_reference::new_bank(3000, SEED, 4);
        let mid = setup_reference::compare_top_k_bank(&big, &[0, 1000, 2999], &[1, 8], &[1, 3]);
        assert!(mid.all_equal(), "{mid:?}");
    }

    /// The earlier untimed checks, one full pass each: `contains`, `top1`
    /// of the null query, `top1` of the warm-up query and `top1` per probe.
    fn old_untimed_checks<'a>(
        bank: &'a [Orb1024],
        null: &Orb1024,
        warmup_index: Option<usize>,
        quality_index: usize,
    ) -> Result<Checks<'a>, String> {
        use gel_reader::top1;
        if bank.contains(null) {
            return Err("null query unexpectedly exists in generated bank".into());
        }
        let null_hit = top1(bank, null).ok_or("empty bank")?;
        let single_warmup = match warmup_index {
            Some(index) => Some(top1(bank, &bank[index]).ok_or("empty bank")?),
            None => None,
        };
        let mut flips_hits = Vec::new();
        for flips in [1usize, 8, 32, 64, 128, 256, 384] {
            let mut words = *bank[quality_index].words();
            for bit in 0..flips {
                let position = (405 * bit + 137) & 1023;
                words[position / 64] ^= 1u64 << (position % 64);
            }
            let query = Orb1024::from_words(words);
            flips_hits.push(top1(bank, &query).ok_or("empty bank")?);
        }
        Ok(Checks {
            null_hit,
            single_warmup,
            flips: flips_hits,
        })
    }

    fn same_checks(a: &Result<Checks<'_>, String>, b: &Result<Checks<'_>, String>) -> bool {
        match (a, b) {
            (Ok(a), Ok(b)) => {
                same_hit(a.null_hit, b.null_hit)
                    && match (a.single_warmup, b.single_warmup) {
                        (Some(x), Some(y)) => same_hit(x, y),
                        (None, None) => true,
                        _ => false,
                    }
                    && a.flips.len() == b.flips.len()
                    && a.flips.iter().zip(&b.flips).all(|(x, y)| same_hit(*x, *y))
            }
            (Err(a), Err(b)) => a == b,
            _ => false,
        }
    }

    #[test]
    fn untimed_checks_match_the_earlier_one_pass_per_query_checks() {
        const SEED: u64 = 0x4745_4C00_4245_4E43;
        let null = orb(0, SEED ^ 0xA5A5_5A5A_D3C1_B7E9);
        let mut banks: Vec<Vec<Orb1024>> = [1, 2, 3, 127, 128, 129, 257, 1000, 4097]
            .iter()
            .map(|&len| setup_reference::new_bank(len, SEED, 3))
            .collect();
        // Banks with many equal orbs and scores, so the lowest-index rule
        // decides; the flipped probes then tie across blocks.
        for (len, pool) in [(300, 3), (513, 5), (1000, 1)] {
            banks.push(setup_reference::tie_bank(len, pool, 0x51ab ^ len as u64));
        }
        // The null query present once, and present at the end.
        let mut with_null = setup_reference::new_bank(700, SEED, 2);
        with_null[333] = null;
        banks.push(with_null);
        let mut null_last = setup_reference::new_bank(129, SEED, 1);
        null_last[128] = null;
        banks.push(null_last);
        let mut cases = 0;
        let mut errors = 0;
        for bank in &banks {
            let len = bank.len();
            for warmup in [None, Some(len / 2), Some(len - 1)] {
                for quality in [0, len / 2, len - 1] {
                    let old = old_untimed_checks(bank, &null, warmup, quality);
                    errors += usize::from(old.is_err());
                    for workers in [1, 2, 3, 8, len / 2 + 1, len + 5] {
                        let new = untimed_checks(bank, &null, warmup, quality, workers);
                        assert!(
                            same_checks(&old, &new),
                            "len {len} warmup {warmup:?} quality {quality} workers {workers}: {old:?} vs {new:?}"
                        );
                        cases += 1;
                    }
                }
            }
        }
        assert!(cases > 0);
        assert_eq!(errors, 2 * 9, "both null-holding banks refuse every case");
    }

    #[test]
    fn bank_crc_matches_the_earlier_per_orb_loop() {
        const SEED: u64 = 0x4745_4C00_4245_4E43;
        let crc = setup_reference::compare_bank_crc(
            &[0, 1, 2, 3, 7, 8, 1000, 4097, 16385],
            &[0, 1, 2, 3, 5, 9, 24, 16386],
            SEED,
        );
        assert!(crc.all_equal(), "{crc:?}");
        let other_seed = setup_reference::compare_bank_crc(&[0, 5, 9999], &[0, 1, 4, 7], 91);
        assert!(other_seed.all_equal(), "{other_seed:?}");
    }

    #[test]
    fn telemetry_never_labels_fallback_as_parallel_execution() {
        let mut report = Top1Execution {
            requested_workers: 2,
            worker_budget: 4,
            effective_workers: 2,
            spawned_workers: 1,
            fallback: None,
        };
        assert_eq!(execution_mode(&[report]), "exact_merge_lowest_index_tie");
        report.effective_workers = 1;
        report.spawned_workers = 0;
        report.fallback = Some(Top1Fallback::ThreadStartFailed);
        assert_eq!(execution_mode(&[report]), "mixed_or_serial_fallback");
    }

    #[test]
    fn execution_order_is_one_token_without_spaces() {
        assert_eq!(joined(&[222_479, 208_232, 207_991]), "222479,208232,207991");
        assert_eq!(joined(&[7]), "7");
        assert_eq!(joined(&[]), "");
    }

    #[test]
    fn processor_field_survives_spaces_and_parentheses_in_comm() {
        let mut fields = (3..=52).map(|i| i.to_string()).collect::<Vec<_>>();
        fields[39 - 3] = "17".to_owned();
        let stat = format!("4242 (gel (a) b) c) {}\n", fields.join(" "));
        assert_eq!(processor_field(&stat), Some(17));
        assert_eq!(processor_field("4242 (short) R 1 2 3"), None);
        assert_eq!(processor_field("no parenthesis at all"), None);
    }
}
