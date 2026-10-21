#![forbid(unsafe_code)]

//! Identity check for `top1_many` and `top1_many_threads`: entry `q` must equal
//! `top1(bank, &queries[q])` and a verbatim copy of the `top1` loop (index,
//! score and the same ORB reference). Seeded banks: random, all ties,
//! clustered, and ties with the winner placed at chunk edges; lengths 0..=9,
//! 64, 1000 and 131,072; query counts 0..=33; thread counts 0..=usize::MAX.
//!
//! Usage: `cargo run --release -p gel-reader --example identity_top1_queries`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (also FAIL if no case ran).
//! Exit status 0 on PASS, 1 on FAIL. Output grammar: `docs/SPEED.md`.

use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_orb::Orb1024;
use gel_reader::{score_xnor, top1, top1_many, top1_many_threads};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
mod identity;
use identity::{Scale, Tally};

type Hit<'a> = Option<(usize, u16, &'a Orb1024)>;

/// The `top1` loop as it stands before this change, kept as the reference.
fn top1_old<'a>(bank: &'a [Orb1024], query: &Orb1024) -> Hit<'a> {
    let mut best: Option<(usize, u16, &Orb1024)> = None;
    for (index, orb) in bank.iter().enumerate() {
        let score = score_xnor(orb, query);
        if best.as_ref().is_none_or(|x| score > x.1) {
            best = Some((index, score, orb));
        }
    }
    best
}

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

/// An ORB whose score against the all-zero query is exactly `score`.
fn orb_with_score(score: u16, seed: u64) -> Orb1024 {
    let ones = ORB_BITS - score as usize;
    let mut words = [0u64; ORB_WORDS];
    let start = (splitmix64(seed) as usize) % ORB_BITS;
    for j in 0..ones {
        let bit = (start + j) % ORB_BITS;
        words[bit / 64] |= 1 << (bit % 64);
    }
    Orb1024::from_words(words)
}

fn banks(len: usize, seed: u64) -> Vec<(&'static str, Vec<Orb1024>)> {
    let mut state = seed;
    let random: Vec<Orb1024> = (0..len).map(|_| random_orb(&mut state)).collect();
    let ties = vec![random_orb(&mut state); len];
    let centers: Vec<Orb1024> = (0..4).map(|_| random_orb(&mut state)).collect();
    let clustered = (0..len)
        .map(|_| {
            let mut orb = centers[(next(&mut state) % 4) as usize];
            for _ in 0..(next(&mut state) % 24) {
                let bit = next(&mut state) as usize % ORB_BITS;
                orb.words_mut()[bit / 64] ^= 1 << (bit % 64);
            }
            orb
        })
        .collect();
    // Narrow scores against the zero query; the best score recurs at the first
    // index of many chunk layouts, so ties straddle chunk edges.
    let edges = (0..len)
        .map(|i| {
            let score = if i % 3 == 0 || i == len / 2 || i + 1 == len {
                1000
            } else {
                990 + (next(&mut state) % 10) as u16
            };
            orb_with_score(score, seed ^ i as u64)
        })
        .collect();
    vec![
        ("random", random),
        ("ties", ties),
        ("clustered", clustered),
        ("edges", edges),
    ]
}

fn same(a: Hit<'_>, b: Hit<'_>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.0 == y.0 && x.1 == y.1 && std::ptr::eq(x.2, y.2),
        _ => false,
    }
}

/// Runs every case. There is no smaller scale: `scale` is accepted for the
/// common interface only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    let mut serial_vs_top1 = Tally::new("top1_many_equals_top1");
    let mut serial_vs_old = Tally::new("top1_equals_old_loop");
    let mut threads_vs_top1 = Tally::new("top1_many_threads_equals_top1");
    let mut lengths = Tally::new("result_lengths");
    for len in [0usize, 1, 2, 3, 7, 8, 9, 64, 1000, 131_072] {
        for (_, bank) in banks(len, 0x7091_0000 + len as u64) {
            let mut state = 0x51ed ^ len as u64;
            let mut queries = vec![Orb1024::from_words([0; ORB_WORDS])];
            queries.extend(bank.iter().step_by(len / 7 + 1).take(8).copied());
            while queries.len() < 33 {
                queries.push(random_orb(&mut state));
            }
            let counts: &[usize] = if len > 1000 {
                &[16, 33]
            } else {
                &[0, 1, 2, 5, 16, 33]
            };
            for &count in counts {
                let qs = &queries[..count];
                let expected: Vec<Hit<'_>> = qs.iter().map(|q| top1(&bank, q)).collect();
                for (q, hit) in qs.iter().zip(&expected) {
                    serial_vs_old.check(same(top1_old(&bank, q), *hit));
                }
                let serial = top1_many(&bank, qs);
                lengths.check(serial.len() == count);
                for (got, want) in serial.iter().zip(&expected) {
                    serial_vs_top1.check(same(*got, *want));
                }
                for threads in [0, 1, 2, 3, 8, 17, len / 2, len / 2 + 1, usize::MAX] {
                    let threaded = top1_many_threads(&bank, qs, threads);
                    lengths.check(threaded.len() == count);
                    for (got, want) in threaded.iter().zip(&expected) {
                        threads_vs_top1.check(same(*got, *want));
                    }
                }
            }
        }
    }
    vec![serial_vs_old, serial_vs_top1, threads_vs_top1, lengths]
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
