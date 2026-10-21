//! Timing harness for the record history and the sparse residual decoder.
//! Prints per-run wall time in nanoseconds and a SHA-256 over a fingerprint
//! of every output produced, so two builds can be compared for both speed
//! and output. Uses only public API.
//!
//! Usage: cargo run --release -p gel-history --example speed_history -- [RUNS]
//! (RUNS 1..=999, default 9). Inputs: deterministic splitmix64 walks of
//! 100,000 states changing 3 or 40 bits per step, 20,000 unrelated states,
//! and 1,000,000 applications of 3- and 92-bit sparse residuals. Output
//! grammar: `docs/SPEED.md`.

use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_history::RecordHistory;
use gel_orb::Orb1024;
use gel_structural::Residual;
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const WALK_STATES: usize = 100_000;
const UNRELATED_STATES: usize = 20_000;
const APPLY_ROUNDS: usize = 1_000_000;
const NO_LIMIT: u64 = u64::MAX;

fn sample(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

/// The next state: `k` distinct bits changed (an odd step is coprime with 1024).
fn step(mut state: Orb1024, k: usize, seed: u64) -> Orb1024 {
    let start = (splitmix64(seed) % ORB_BITS as u64) as usize;
    let stride = (splitmix64(seed ^ 0xA5A5) % 512 * 2 + 1) as usize;
    for j in 0..k {
        let bit = (start + stride * j) % ORB_BITS;
        state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
    }
    state
}

fn walk(n: usize, k: usize, seed: u64) -> Vec<Orb1024> {
    let mut states = Vec::with_capacity(n);
    let mut state = sample(seed);
    for i in 0..n {
        states.push(state);
        state = step(state, k, seed.wrapping_add(i as u64 * 0x9E37_79B9));
    }
    states
}

fn unrelated(n: usize, seed: u64) -> Vec<Orb1024> {
    (0..n)
        .map(|i| sample(seed.wrapping_add(1_000_003 * i as u64)))
        .collect()
}

fn history_of(states: &[Orb1024]) -> RecordHistory {
    let mut history = RecordHistory::new();
    for state in states {
        history.append(*state).expect("append");
    }
    history
}

fn fold_words(mut acc: u64, words: impl IntoIterator<Item = u64>) -> u64 {
    for word in words {
        acc = splitmix64(acc ^ word);
    }
    acc
}

fn fold_states(states: &[Orb1024]) -> u64 {
    fold_words(
        states.len() as u64,
        states.iter().flat_map(|s| s.words().iter().copied()),
    )
}

fn fold_bytes(bytes: &[u8]) -> u64 {
    fold_words(
        bytes.len() as u64,
        bytes.chunks(8).map(|chunk| {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            u64::from_le_bytes(word)
        }),
    )
}

/// Residuals of `k` bits from `count` different predictors.
fn residuals(k: usize, count: usize, seed: u64) -> Vec<(Orb1024, Residual)> {
    (0..count)
        .map(|i| {
            let predictor = sample(seed.wrapping_add(i as u64 * 977));
            let target = step(predictor, k, seed ^ i as u64);
            let residual = Residual::from_exact_xor(&target, &predictor);
            assert!(matches!(residual, Residual::Sparse { .. }));
            (predictor, residual)
        })
        .collect()
}

fn main() {
    let args = speed::args("speed_history", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let walk3 = walk(WALK_STATES, 3, 0x4849_0003);
    let walk40 = walk(WALK_STATES, 40, 0x4849_0040);
    let lone = unrelated(UNRELATED_STATES, 0x4849_0000);
    let history3 = history_of(&walk3);
    let bytes3 = history3.to_bytes().expect("to_bytes");
    let bytes40 = history_of(&walk40).to_bytes().expect("to_bytes");
    let bytes_lone = history_of(&lone).to_bytes().expect("to_bytes");
    let history40 = RecordHistory::from_bytes(&bytes40, NO_LIMIT, NO_LIMIT).expect("open");
    let sparse3 = residuals(3, 64, 0x5350_0003);
    let sparse92 = residuals(92, 64, 0x5350_0092);
    speed::setup(
        "speed_history",
        runs,
        &[
            ("walk_states", WALK_STATES.to_string()),
            ("walk_k3_bytes", bytes3.len().to_string()),
            ("walk_k40_bytes", bytes40.len().to_string()),
            ("unrelated_states", UNRELATED_STATES.to_string()),
            ("unrelated_bytes", bytes_lone.len().to_string()),
            ("apply_rounds", APPLY_ROUNDS.to_string()),
        ],
    );

    let mut outputs = Vec::new();
    let mut record = |name: &str, run: usize, ns: u128, fingerprint: u64| {
        speed::line(name, run, ns);
        outputs.extend_from_slice(name.as_bytes());
        outputs.extend_from_slice(&fingerprint.to_be_bytes());
    };

    for run in 0..runs {
        let start = Instant::now();
        let history = black_box(history_of(black_box(&walk3)));
        let ns = start.elapsed().as_nanos();
        record(
            "append_walk_k3_100k",
            run,
            ns,
            fold_bytes(&history.to_bytes().expect("to_bytes")),
        );
    }
    for run in 0..runs {
        let start = Instant::now();
        let bytes = black_box(black_box(&history3).to_bytes().expect("to_bytes"));
        let ns = start.elapsed().as_nanos();
        record("to_bytes_walk_k3_100k", run, ns, fold_bytes(&bytes));
    }
    let opens: [(&str, &[u8]); 3] = [
        ("from_bytes_walk_k3_100k", &bytes3),
        ("from_bytes_walk_k40_100k", &bytes40),
        ("from_bytes_unrelated_20k", &bytes_lone),
    ];
    for (name, bytes) in opens {
        for run in 0..runs {
            let start = Instant::now();
            let history = black_box(RecordHistory::from_bytes(
                black_box(bytes),
                NO_LIMIT,
                NO_LIMIT,
            ));
            let ns = start.elapsed().as_nanos();
            let history = history.expect("open");
            record(
                name,
                run,
                ns,
                fold_bytes(&history.to_bytes().expect("to_bytes")),
            );
        }
    }
    let exacts: [(&str, &RecordHistory); 2] = [
        ("exact_history_walk_k3_100k", &history3),
        ("exact_history_walk_k40_100k", &history40),
    ];
    for (name, history) in exacts {
        for run in 0..runs {
            let start = Instant::now();
            let states = black_box(black_box(history).exact_history());
            let ns = start.elapsed().as_nanos();
            record(name, run, ns, fold_states(&states.expect("exact_history")));
        }
    }
    let applies: [(&str, &[(Orb1024, Residual)]); 2] = [
        ("sparse_apply_k3_1m", &sparse3),
        ("sparse_apply_k92_1m", &sparse92),
    ];
    for (name, pool) in applies {
        for run in 0..runs {
            let start = Instant::now();
            let mut acc = 0u64;
            for i in 0..APPLY_ROUNDS {
                let (predictor, residual) = &pool[i % pool.len()];
                let out = black_box(residual)
                    .apply(black_box(predictor))
                    .expect("apply");
                acc ^= out.words()[i % ORB_WORDS].rotate_left((i % 64) as u32);
            }
            let ns = start.elapsed().as_nanos();
            record(name, run, ns, acc);
        }
    }
    for run in 0..runs {
        let start = Instant::now();
        let mut acc = 0u64;
        for i in 0..APPLY_ROUNDS {
            let (_, residual) = &sparse92[i % sparse92.len()];
            acc = acc.wrapping_add(black_box(residual).popcount().expect("popcount") as u64);
        }
        let ns = start.elapsed().as_nanos();
        record("sparse_popcount_k92_1m", run, ns, acc);
    }
    speed::check_digest(&speed::sha256(&outputs));
}
