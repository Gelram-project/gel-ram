#![forbid(unsafe_code)]

//! Same-binary timing of `top_k` against `top_k_threads` on deterministic
//! 524,288-ORB banks (64 MiB). Every threaded result must equal the serial one.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_topk_threads --
//! [RUNS] [THREADS]` (defaults 9 and 8). Prints
//! `SPEED name=<case> run=<i> ns=<n>` per run and, at the end,
//! `SPEED_CHECK crc64=<hex>` over every result of every run.

use gel_core::{splitmix64, Crc64Ecma, ORB_BITS};
use gel_orb::Orb1024;
use gel_reader::{top_k, top_k_threads};
use std::hint::black_box;
use std::time::Instant;

const ORBS: usize = 524_288;

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let runs = args
        .next()
        .map(|v| v.parse::<usize>().expect("RUNS must be a positive integer"))
        .unwrap_or(9);
    let threads = args
        .next()
        .map(|v| v.parse::<usize>().expect("THREADS must be an integer"))
        .unwrap_or(8);
    assert!(runs >= 1, "RUNS must be a positive integer");
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
    let mut crc = Crc64Ecma::new();
    for (name, bank, query) in [
        ("uniform", &uniform, uniform_query),
        ("clustered", &clustered, clustered_query),
    ] {
        for k in [8, 256] {
            for run in 1..=runs {
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
                assert_eq!(threaded, serial, "{name} k={k} threads={threads}");
                println!("SPEED name={name}_k{k}_serial run={run} ns={serial_ns}");
                println!("SPEED name={name}_k{k}_threads{threads} run={run} ns={threaded_ns}");
                for (index, score) in threaded {
                    crc.update(&(index as u64).to_le_bytes());
                    crc.update(&score.to_le_bytes());
                }
            }
        }
    }
    println!("SPEED_CHECK crc64={:016x}", crc.finish());
}
