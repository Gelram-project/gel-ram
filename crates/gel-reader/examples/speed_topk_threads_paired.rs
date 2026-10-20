#![forbid(unsafe_code)]

//! Same-binary timing of `top_k` (`<case>_ref`) against `top_k_threads`
//! (`<case>_t<THREADS>`) on deterministic 524,288-ORB banks (64 MiB). Every
//! threaded result must equal the serial one; a difference ends the run with
//! `SPEED_FAIL reason=threads_differ`.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_topk_threads_paired --
//! [RUNS] [THREADS]` (RUNS 1..=999, default 9; THREADS 1..=999, default 8).
//! Prints `SPEED name=<case> run=<i> ns=<n>` per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over every result of every run. Output grammar:
//! `docs/SPEED.md`.

use gel_core::{splitmix64, ORB_BITS};
use gel_orb::Orb1024;
use gel_reader::{top_k, top_k_threads};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const ORBS: usize = 524_288;

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

fn main() {
    let args = speed::args("speed_topk_threads_paired", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let threads = speed::threads(&args, 1);
    speed::setup(
        "speed_topk_threads_paired",
        runs,
        &[
            ("orbs", ORBS.to_string()),
            ("bytes", (ORBS * 128).to_string()),
            ("threads", format!("1,{threads}")),
        ],
    );
    let mut seed = 0x2026_1019_5448_5244;
    let uniform: Vec<_> = (0..ORBS).map(|_| random_orb(&mut seed)).collect();
    let centers: Vec<_> = (0..16).map(|_| random_orb(&mut seed)).collect();
    let clustered: Vec<_> = (0..ORBS)
        .map(|i| {
            let mut orb = centers[i % centers.len()];
            for _ in 0..16 {
                let bit = next(&mut seed) as usize % ORB_BITS;
                orb.words_mut()[bit / 64] ^= 1 << (bit % 64);
            }
            orb
        })
        .collect();
    let uniform_query = random_orb(&mut seed);
    let clustered_query = centers[5];
    let mut hash = speed::Sha256::new();
    for (name, bank, query) in [
        ("uniform", &uniform, uniform_query),
        ("clustered", &clustered, clustered_query),
    ] {
        for k in [8, 256] {
            for run in 0..runs {
                let start = Instant::now();
                let serial = top_k(black_box(bank), black_box(&query), black_box(k));
                let serial_ns = start.elapsed().as_nanos();
                let start = Instant::now();
                let threaded = top_k_threads(
                    black_box(bank),
                    black_box(&query),
                    black_box(k),
                    black_box(threads),
                );
                let threaded_ns = start.elapsed().as_nanos();
                if threaded != serial {
                    speed::fail("threads_differ");
                }
                speed::line(&format!("{name}_k{k}_ref"), run, serial_ns);
                speed::line(&format!("{name}_k{k}_t{threads}"), run, threaded_ns);
                for (index, score) in threaded {
                    hash.update(&(index as u64).to_le_bytes());
                    hash.update(&score.to_le_bytes());
                }
            }
        }
    }
    speed::check(hash);
}
