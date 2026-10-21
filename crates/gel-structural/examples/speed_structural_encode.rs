//! Timing harness for the structural encoder. Prints per-run wall time in
//! nanoseconds and a SHA-256 over every output produced, so two builds can
//! be compared for both speed and output. Uses only public API.
//!
//! Usage: cargo run --release -p gel-structural --example speed_structural_encode -- [RUNS]
//! (RUNS 1..=999, default 9). Every run of a case must give the value of run
//! 0; a difference ends the run with `SPEED_FAIL reason=run_differs`. Output
//! grammar: `docs/SPEED.md`.
//!
//! Input (deterministic splitmix64):
//! - 131,072 (target, predictor) pairs per `from_exact_xor_*` case: random
//!   targets (`far`), 101..=200 changed bits (`mid`), 0..=100 changed bits
//!   (`near`);
//! - 4,096 targets against one pool of 64 prototypes (`encode_best_prototype_pool64`);
//! - `apply_near` and `popcount_near` run on the `near` residuals.

use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_orb::Orb1024;
use gel_structural::{encode_best_prototype, EncodedOrb, ParentRef, Residual};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const PAIRS: usize = 131_072;
const POOL: usize = 64;
const TARGETS: usize = 4_096;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix64(self.0)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn orb(&mut self) -> Orb1024 {
        let mut words = [0u64; ORB_WORDS];
        for word in &mut words {
            *word = self.next();
        }
        Orb1024::from_words(words)
    }

    /// `orb` with exactly `k` distinct bits flipped.
    fn flipped(&mut self, orb: &Orb1024, k: usize) -> Orb1024 {
        let mut out = *orb;
        let mut order: Vec<usize> = (0..ORB_BITS).collect();
        for i in 0..k {
            let j = i + self.below(ORB_BITS - i);
            order.swap(i, j);
            out.words_mut()[order[i] >> 6] ^= 1u64 << (order[i] & 63);
        }
        out
    }
}

fn pairs(seed: u64, changed: impl Fn(usize) -> Option<usize>) -> Vec<(Orb1024, Orb1024)> {
    let mut rng = Rng(seed);
    (0..PAIRS)
        .map(|i| {
            let predictor = rng.orb();
            let target = match changed(i) {
                Some(k) => rng.flipped(&predictor, k),
                None => rng.orb(),
            };
            (target, predictor)
        })
        .collect()
}

fn residual_bytes(residual: &Residual, out: &mut Vec<u8>) {
    match residual {
        Residual::Sparse {
            count,
            packed_positions,
        } => {
            out.push(0);
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(packed_positions);
        }
        Residual::Dense(delta) => {
            out.push(1);
            for word in delta.words() {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
    }
}

fn encoded_bytes(encoded: &EncodedOrb, index: Option<usize>, out: &mut Vec<u8>) {
    match encoded {
        EncodedOrb::Literal(orb) => {
            out.push(0);
            for word in orb.words() {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
        EncodedOrb::Delta {
            parent,
            parent_depth,
            residual,
        } => {
            out.push(1);
            match parent {
                ParentRef::Prototype(id) => {
                    out.push(0);
                    out.extend_from_slice(&id.to_le_bytes());
                }
                ParentRef::SegmentLocal(id) => {
                    out.push(1);
                    out.extend_from_slice(&id.to_le_bytes());
                }
            }
            out.push(*parent_depth);
            residual_bytes(residual, out);
        }
    }
    out.extend_from_slice(&index.map_or(u64::MAX, |i| i as u64).to_le_bytes());
}

/// Times `case` RUNS times. Every run must give the same value; run 0's
/// value is serialized into `outputs`.
fn timed<T: PartialEq>(
    name: &str,
    runs: usize,
    outputs: &mut Vec<u8>,
    mut case: impl FnMut() -> T,
    serialize: impl Fn(&T, &mut Vec<u8>),
) -> T {
    let mut first: Option<T> = None;
    for run in 0..runs {
        let start = Instant::now();
        let value = black_box(case());
        let ns = start.elapsed().as_nanos();
        speed::line(name, run, ns);
        match &first {
            None => first = Some(value),
            Some(expected) if *expected == value => {}
            Some(_) => {
                eprintln!("{name}: run {run} differs from run 0");
                speed::fail("run_differs");
            }
        }
    }
    let value = first.expect("RUNS is positive");
    outputs.extend_from_slice(name.as_bytes());
    serialize(&value, outputs);
    value
}

fn main() {
    let args = speed::args("speed_structural_encode", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::setup(
        "speed_structural_encode",
        runs,
        &[
            ("pairs", PAIRS.to_string()),
            ("pool", POOL.to_string()),
            ("targets", TARGETS.to_string()),
        ],
    );
    let far = pairs(0x4745_4C5F_4641_5231, |_| None);
    let mid = pairs(0x4745_4C5F_4D49_4431, |i| Some(101 + i % 100));
    let near = pairs(0x4745_4C5F_4E45_4131, |i| Some(i % 101));

    let mut rng = Rng(0x4745_4C5F_504F_4F4C);
    let pool: Vec<Orb1024> = (0..POOL).map(|_| rng.orb()).collect();
    let targets: Vec<Orb1024> = (0..TARGETS)
        .map(|i| {
            if i % 4 == 3 {
                rng.orb()
            } else {
                let parent = pool[rng.below(POOL)];
                let k = rng.below(121);
                rng.flipped(&parent, k)
            }
        })
        .collect();

    let mut outputs = Vec::new();
    let encode_all = |input: &[(Orb1024, Orb1024)]| -> Vec<Residual> {
        input
            .iter()
            .map(|(target, predictor)| Residual::from_exact_xor(black_box(target), predictor))
            .collect()
    };
    let serialize_residuals = |values: &Vec<Residual>, out: &mut Vec<u8>| {
        for residual in values {
            residual_bytes(residual, out);
        }
    };
    timed(
        "from_exact_xor_far",
        runs,
        &mut outputs,
        || encode_all(&far),
        serialize_residuals,
    );
    timed(
        "from_exact_xor_mid",
        runs,
        &mut outputs,
        || encode_all(&mid),
        serialize_residuals,
    );
    let near_residuals = timed(
        "from_exact_xor_near",
        runs,
        &mut outputs,
        || encode_all(&near),
        serialize_residuals,
    );
    timed(
        "encode_best_prototype_pool64",
        runs,
        &mut outputs,
        || {
            targets
                .iter()
                .map(|target| encode_best_prototype(black_box(target), &pool).expect("encode"))
                .collect::<Vec<_>>()
        },
        |values, out| {
            for (encoded, index) in values {
                encoded_bytes(encoded, *index, out);
            }
        },
    );
    timed(
        "apply_near",
        runs,
        &mut outputs,
        || {
            near_residuals
                .iter()
                .zip(near.iter())
                .map(|(residual, (_, predictor))| {
                    black_box(residual).apply(predictor).expect("apply")
                })
                .collect::<Vec<_>>()
        },
        |values, out| {
            for orb in values {
                for word in orb.words() {
                    out.extend_from_slice(&word.to_le_bytes());
                }
            }
        },
    );
    timed(
        "popcount_near",
        runs,
        &mut outputs,
        || {
            near_residuals
                .iter()
                .map(|residual| black_box(residual).popcount().expect("popcount"))
                .collect::<Vec<_>>()
        },
        |values, out| {
            for count in values {
                out.extend_from_slice(&count.to_le_bytes());
            }
        },
    );
    speed::check_digest(&speed::sha256(&outputs));
}
