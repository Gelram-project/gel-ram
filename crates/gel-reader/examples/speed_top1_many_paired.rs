//! Same-binary pairs for 16 queries on two deterministic banks (32,768 ORBs =
//! 4 MiB and 1,310,720 ORBs = 160 MiB): 16 single `top1` calls (`_ref`)
//! against one `top1_many` call (`_many`), and 16 `top1_threads` calls against
//! one `top1_many_threads` call with THREADS threads. The two sides of a pair
//! alternate which one runs first from run to run, and their results must be
//! equal in every run.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_top1_many_paired --
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

/// Times `reference` and `variant` in the order given by `reference_first`.
fn pair<'a>(
    reference_first: bool,
    reference: impl Fn() -> Vec<Hit<'a>>,
    variant: impl Fn() -> Vec<Hit<'a>>,
) -> ((Vec<Hit<'a>>, u128), (Vec<Hit<'a>>, u128)) {
    let timed = |f: &dyn Fn() -> Vec<Hit<'a>>| {
        let start = Instant::now();
        let hits = f();
        (hits, start.elapsed().as_nanos())
    };
    if reference_first {
        let r = timed(&reference);
        let v = timed(&variant);
        (r, v)
    } else {
        let v = timed(&variant);
        let r = timed(&reference);
        (r, v)
    }
}

fn main() {
    let args = speed::args("speed_top1_many_paired", "[RUNS] [THREADS]", 2);
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
        "speed_top1_many_paired",
        runs,
        &[
            ("orbs", format!("{},{}", BANKS[0], BANKS[1])),
            ("queries", QUERIES.to_string()),
            ("threads", format!("1,{threads}")),
        ],
    );
    let mut hash = speed::Sha256::new();
    for run in 0..runs {
        let reference_first = run % 2 == 0;
        for bank in &banks {
            let n = bank.len();
            let ((single, single_ns), (many, many_ns)) = pair(
                reference_first,
                || {
                    queries
                        .iter()
                        .map(|q| top1(black_box(bank), black_box(q)))
                        .collect()
                },
                || top1_many(black_box(bank), black_box(&queries)),
            );
            if many != single {
                fail("top1_many_differs_from_top1");
            }
            let ((single_threads, single_threads_ns), (many_threads, many_threads_ns)) = pair(
                reference_first,
                || {
                    queries
                        .iter()
                        .map(|q| top1_threads(black_box(bank), black_box(q), black_box(threads)))
                        .collect()
                },
                || top1_many_threads(black_box(bank), black_box(&queries), black_box(threads)),
            );
            if single_threads != single {
                fail("top1_threads_differs_from_top1");
            }
            if many_threads != single {
                fail("top1_many_threads_differs_from_top1");
            }
            speed::line(&format!("queries16_{n}_t1_ref"), run, single_ns);
            speed::line(&format!("queries16_{n}_t1_many"), run, many_ns);
            speed::line(
                &format!("queries16_{n}_t{threads}_ref"),
                run,
                single_threads_ns,
            );
            speed::line(
                &format!("queries16_{n}_t{threads}_many"),
                run,
                many_threads_ns,
            );
            record(&mut hash, &single);
            record(&mut hash, &many);
            record(&mut hash, &single_threads);
            record(&mut hash, &many_threads);
        }
    }
    speed::check(hash);
}
