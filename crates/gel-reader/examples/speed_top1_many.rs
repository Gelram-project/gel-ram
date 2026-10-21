//! Timing of `top1_many` and `top1_many_threads` for 16 queries on two
//! deterministic banks (32,768 ORBs = 4 MiB and 1,310,720 ORBs = 160 MiB),
//! with 16 single `top1` and `top1_threads` calls as cases whose code this
//! comparison does not change. Uses only API that exists before the blocked
//! `top1_many`, so the same file runs on both trees for an A/B comparison.
//! Every multi-query result must equal the single-query results of its run.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_top1_many --
//! [RUNS] [THREADS]` (defaults 9 and 8; RUNS in 1..=999, THREADS in 1..=256).
//! Prints one `SPEED_SETUP` line, `SPEED name=<case> run=<i> ns=<n>` per case
//! and run, and one final `SPEED_CHECK sha256=<hex>` over every result of
//! every run. A mismatch prints `SPEED_FAIL reason=<slug>` on stderr and exits 1.
//! Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]

use gel_core::splitmix64;
use gel_orb::Orb1024;
use gel_reader::{top1, top1_many, top1_many_threads, top1_threads};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const BANKS: [usize; 2] = [32_768, 1_310_720];
const QUERIES: usize = 16;

type Hit<'a> = Option<(usize, u16, &'a Orb1024)>;

fn fail(reason: &str) -> ! {
    speed::fail(reason)
}

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

fn record(hash: &mut speed::Sha256, hits: &[Hit<'_>]) {
    for hit in hits {
        let (index, score) = hit.map_or((u64::MAX, u16::MAX), |(i, s, _)| (i as u64, s));
        hash.update(&index.to_le_bytes());
        hash.update(&score.to_le_bytes());
    }
}

fn main() {
    let args = speed::args("speed_top1_many", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let threads = speed::number(&args, 1, "THREADS", speed::THREADS_DEFAULT, 1..=256);
    speed::self_check();
    let mut seed = 0x2026_1020_7031_0016;
    let banks: Vec<Vec<Orb1024>> = BANKS
        .iter()
        .map(|&n| (0..n).map(|_| random_orb(&mut seed)).collect())
        .collect();
    let queries: Vec<Orb1024> = (0..QUERIES).map(|_| random_orb(&mut seed)).collect();
    speed::setup(
        "speed_top1_many",
        runs,
        &[
            ("orbs", format!("{},{}", BANKS[0], BANKS[1])),
            ("queries", QUERIES.to_string()),
            ("threads", format!("1,{threads}")),
        ],
    );
    let mut hash = speed::Sha256::new();
    for run in 0..runs {
        for bank in &banks {
            let n = bank.len();
            let start = Instant::now();
            let single: Vec<Hit<'_>> = queries
                .iter()
                .map(|q| top1(black_box(bank), black_box(q)))
                .collect();
            let single_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            let many = top1_many(black_box(bank), black_box(&queries));
            let many_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            let single_threads: Vec<Hit<'_>> = queries
                .iter()
                .map(|q| top1_threads(black_box(bank), black_box(q), black_box(threads)))
                .collect();
            let single_threads_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            let many_threads =
                top1_many_threads(black_box(bank), black_box(&queries), black_box(threads));
            let many_threads_ns = start.elapsed().as_nanos();
            if many != single {
                fail("top1_many_differs_from_top1");
            }
            if single_threads != single {
                fail("top1_threads_differs_from_top1");
            }
            if many_threads != single {
                fail("top1_many_threads_differs_from_top1");
            }
            speed::line(&format!("single16_{n}_t1"), run, single_ns);
            speed::line(&format!("many16_{n}_t1"), run, many_ns);
            speed::line(&format!("single16_{n}_t{threads}"), run, single_threads_ns);
            speed::line(&format!("many16_{n}_t{threads}"), run, many_threads_ns);
            record(&mut hash, &many);
            record(&mut hash, &many_threads);
        }
    }
    speed::check(hash);
}
