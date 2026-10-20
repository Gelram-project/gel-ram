#![forbid(unsafe_code)]

//! Same-binary timing of 16 single-query Top-1 scans against one multi-query
//! scan on a deterministic bank of 1,310,720 ORBs (160 MiB): 16 x `top1`
//! (`top1_q16_ref`) against `top1_many` (`top1_q16_many`), and 16 x
//! `top1_threads` (`top1_q16_t<T>_ref`) against `top1_many_threads`
//! (`top1_q16_t<T>_many`) with T threads. Every multi-query result must equal
//! the single-query one; a difference ends the run with
//! `SPEED_FAIL reason=<case>_differs`.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_top1_queries_paired --
//! [RUNS] [THREADS]` (RUNS 1..=999, default 9; THREADS 1..=999, default 8).
//! Prints `SPEED name=<case> run=<i> ns=<n>` per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over every result of every run. Output grammar:
//! `docs/SPEED.md`.

use gel_core::splitmix64;
use gel_orb::Orb1024;
use gel_reader::{top1, top1_many, top1_many_threads, top1_threads};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const ORBS: usize = 1_310_720;
const QUERIES: usize = 16;

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

type Hit<'a> = Option<(usize, u16, &'a Orb1024)>;

fn record(hash: &mut speed::Sha256, hits: &[Hit<'_>]) {
    for hit in hits {
        let (index, score) = hit.map_or((u64::MAX, u16::MAX), |(i, s, _)| (i as u64, s));
        hash.update(&index.to_le_bytes());
        hash.update(&score.to_le_bytes());
    }
}

fn main() {
    let args = speed::args("speed_top1_queries_paired", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let threads = speed::threads(&args, 1);
    let mut seed = 0x2026_1020_7031_0016;
    let bank: Vec<Orb1024> = (0..ORBS).map(|_| random_orb(&mut seed)).collect();
    let queries: Vec<Orb1024> = (0..QUERIES).map(|_| random_orb(&mut seed)).collect();
    speed::setup(
        "speed_top1_queries_paired",
        runs,
        &[
            ("orbs", ORBS.to_string()),
            ("bytes", (ORBS * 128).to_string()),
            ("queries", QUERIES.to_string()),
            ("threads", format!("1,{threads}")),
        ],
    );
    let mut hash = speed::Sha256::new();
    for run in 0..runs {
        let start = Instant::now();
        let single: Vec<Hit<'_>> = queries
            .iter()
            .map(|q| top1(black_box(&bank), black_box(q)))
            .collect();
        let single_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let many = top1_many(black_box(&bank), black_box(&queries));
        let many_ns = start.elapsed().as_nanos();
        if many != single {
            speed::fail("top1_many_differs");
        }
        let start = Instant::now();
        let single_threads: Vec<Hit<'_>> = queries
            .iter()
            .map(|q| top1_threads(black_box(&bank), black_box(q), black_box(threads)))
            .collect();
        let single_threads_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let many_threads =
            top1_many_threads(black_box(&bank), black_box(&queries), black_box(threads));
        let many_threads_ns = start.elapsed().as_nanos();
        if single_threads != single {
            speed::fail("top1_threads_differs");
        }
        if many_threads != single {
            speed::fail("top1_many_threads_differs");
        }
        speed::line("top1_q16_ref", run, single_ns);
        speed::line("top1_q16_many", run, many_ns);
        speed::line(&format!("top1_q16_t{threads}_ref"), run, single_threads_ns);
        speed::line(&format!("top1_q16_t{threads}_many"), run, many_threads_ns);
        record(&mut hash, &many);
        record(&mut hash, &many_threads);
    }
    speed::check(hash);
}
