#![forbid(unsafe_code)]

//! Repeatable timing of `top_k` on deterministic 524,288-ORB banks (64 MiB).
//! It uses only API that predates `top_k_threads`, so the same file builds on
//! older trees and the printed output digests can be compared between them.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_topk -- [RUNS]`
//! (1..=999, default 9). Prints `SPEED name=<case> run=<i> ns=<n>` per run
//! and, at the end, `SPEED_CHECK sha256=<hex>` over every result of every run.
//! Output grammar: `docs/SPEED.md`.

use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_orb::Orb1024;
use gel_reader::top_k;
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

/// An ORB whose score against the all-zero query is exactly `score`.
fn orb_with_score(score: usize, rotate: usize) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for j in 0..ORB_BITS - score {
        let bit = (rotate + j) % ORB_BITS;
        words[bit / 64] |= 1 << (bit % 64);
    }
    Orb1024::from_words(words)
}

/// Deterministic bank and query for one case.
fn case(name: &str) -> (Vec<Orb1024>, Orb1024) {
    let mut seed = 0x2026_1019_544f_504b;
    let zero = Orb1024::from_words([0; ORB_WORDS]);
    match name {
        "uniform" => {
            let bank = (0..ORBS).map(|_| random_orb(&mut seed)).collect();
            (bank, random_orb(&mut seed))
        }
        "clustered" => {
            let centers: Vec<_> = (0..16).map(|_| random_orb(&mut seed)).collect();
            let bank = (0..ORBS)
                .map(|i| {
                    let mut orb = centers[i % centers.len()];
                    for _ in 0..16 {
                        let bit = next(&mut seed) as usize % ORB_BITS;
                        orb.words_mut()[bit / 64] ^= 1 << (bit % 64);
                    }
                    orb
                })
                .collect();
            let mut query = centers[3];
            query.words_mut()[0] ^= 0xff;
            (bank, query)
        }
        "ties" => (vec![tie_orb(); ORBS], zero),
        // 255 perfect scores, then a rising saw: candidates keep entering the
        // full set at its last position.
        "tail_entry" => {
            let bank = (0..ORBS)
                .map(|i| {
                    if i < 255 {
                        orb_with_score(ORB_BITS, i)
                    } else {
                        orb_with_score((i - 255) % ORB_BITS, i)
                    }
                })
                .collect();
            (bank, zero)
        }
        // Scores rise in equal steps from 0 to 1024; each step admits a full
        // set of equal-score entries at every position of the kept list.
        "staircase" => {
            let bank = (0..ORBS)
                .map(|i| orb_with_score(i * (ORB_BITS + 1) / ORBS, i))
                .collect();
            (bank, zero)
        }
        _ => unreachable!("unknown case {name}"),
    }
}

/// One fixed ORB repeated across the whole bank: every score ties.
fn tie_orb() -> Orb1024 {
    let mut seed = 0x7469_6573;
    random_orb(&mut seed)
}

fn main() {
    let args = speed::args("speed_topk", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::setup(
        "speed_topk",
        runs,
        &[
            ("orbs", ORBS.to_string()),
            ("bytes", (ORBS * 128).to_string()),
        ],
    );
    let mut digest = speed::Sha256::new();
    for (name, k) in [
        ("uniform", 256),
        ("uniform", 8),
        ("clustered", 256),
        ("ties", 256),
        ("tail_entry", 256),
        ("staircase", 256),
    ] {
        let (bank, query) = case(name);
        let label = format!("{name}_k{k}");
        for run in 0..runs {
            let start = Instant::now();
            let out = top_k(black_box(&bank), black_box(&query), black_box(k));
            let ns = start.elapsed().as_nanos();
            black_box(&out);
            speed::line(&label, run, ns);
            digest.update(label.as_bytes());
            digest.update(&(out.len() as u64).to_le_bytes());
            for (index, score) in out {
                digest.update(&(index as u64).to_le_bytes());
                digest.update(&score.to_le_bytes());
            }
        }
    }
    speed::check(digest);
}
