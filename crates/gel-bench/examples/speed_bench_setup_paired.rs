//! Wall-time harness for the setup work alone, earlier serial code
//! (`<case>_ref`) against the current code (`<case>_new`) in one binary: bank
//! generation, the Top-8 reference and the pointer-chase chain for five
//! rounds. It needs the `setup` and `chain` modules, so it runs only on trees
//! that have them. Prints per-run wall time in nanoseconds and a SHA-256 over
//! every produced output; differing outputs end the run with
//! `SPEED_FAIL reason=outputs_differ`.
//!
//! Usage: cargo run --release -p gel-bench --example speed_bench_setup_paired -- [RUNS]
//! (RUNS 1..=999, default 9). Inputs: 1,310,720 orbs (160 MiB bank, the
//! gel-bench default), Top-8 for the query at index 1,310,720 / 3, and a
//! 256 MiB chain (the largest gel-physics size) built for 5 rounds. Output
//! grammar: `docs/SPEED.md`.

#![forbid(unsafe_code)]

#[path = "../src/setup.rs"]
mod setup;
// Only the earlier-code copies are used here; their comparisons belong to
// the identity program.
#[allow(dead_code)]
#[path = "../src/setup_reference.rs"]
mod setup_reference;

#[path = "../../gel-physics/src/chain.rs"]
mod chain;
#[allow(dead_code)]
#[path = "../../gel-physics/src/chain_reference.rs"]
mod chain_reference;

#[path = "../../../tools/speed_support.rs"]
mod speed;

use std::hint::black_box;
use std::time::Instant;

const GENERATOR_SEED: u64 = 0x4745_4C00_4245_4E43;
const ORBS: usize = 1_310_720;
const CHAIN_BYTES: usize = 256 << 20;
const CHAIN_ROUNDS: usize = 5;

fn fold(next: &[usize]) -> u64 {
    next.iter().enumerate().fold(0u64, |s, (i, &v)| {
        s.rotate_left(5) ^ (v as u64).wrapping_add(i as u64)
    })
}

/// One timed case: its output bytes.
type Case<'a> = &'a dyn Fn() -> Vec<u8>;

fn entries(top: &[(usize, u16)]) -> Vec<u8> {
    let mut out = Vec::new();
    for &(index, score) in top {
        out.extend_from_slice(&(index as u64).to_be_bytes());
        out.extend_from_slice(&score.to_be_bytes());
    }
    out
}

fn timed(name: &str, run: usize, case: &Case<'_>) -> Vec<u8> {
    let start = Instant::now();
    let out = black_box(case());
    let ns = start.elapsed().as_nanos();
    speed::line(name, run, ns);
    out
}

fn main() {
    let args = speed::args("speed_bench_setup_paired", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::setup(
        "speed_bench_setup_paired",
        runs,
        &[
            ("orbs", ORBS.to_string()),
            ("chain_bytes", CHAIN_BYTES.to_string()),
            ("chain_rounds", CHAIN_ROUNDS.to_string()),
        ],
    );
    let workers = setup::workers_for(ORBS, setup::BANK_ORBS_PER_WORKER);
    let bank = setup_reference::new_bank(ORBS, GENERATOR_SEED, workers);
    let query = bank[ORBS / 3];
    let mut outputs = Vec::new();
    let bank_old = || {
        let crc = setup_reference::bank_crc64(&setup_reference::old_bank(ORBS, GENERATOR_SEED));
        crc.to_be_bytes().to_vec()
    };
    let bank_new = || {
        let workers = setup::workers_for(ORBS, setup::BANK_ORBS_PER_WORKER);
        let crc =
            setup_reference::bank_crc64(&setup_reference::new_bank(ORBS, GENERATOR_SEED, workers));
        crc.to_be_bytes().to_vec()
    };
    let top_old = || entries(&setup_reference::old_reference_top_k(&bank, &query, 8));
    let top_new = || entries(&setup::reference_top_k(&bank, &query, 8));
    let chain_old = || {
        (0..CHAIN_ROUNDS)
            .flat_map(|_| fold(&chain_reference::old_chain(CHAIN_BYTES).1).to_be_bytes())
            .collect::<Vec<u8>>()
    };
    let chain_new = || {
        let order = chain::chain_order(CHAIN_BYTES);
        (0..CHAIN_ROUNDS)
            .flat_map(|_| fold(&chain::build_next(&order)).to_be_bytes())
            .collect::<Vec<u8>>()
    };
    let cases: [(&str, Case<'_>, Case<'_>); 3] = [
        ("bank_generate_1310720", &bank_old, &bank_new),
        ("reference_top8_1310720", &top_old, &top_new),
        ("chain_256mib_5rep", &chain_old, &chain_new),
    ];
    for run in 0..runs {
        for (name, old, new) in &cases {
            // Alternate which side goes first, so drift favours neither.
            let mut got = [Vec::new(), Vec::new()];
            let order = if run % 2 == 0 { [0, 1] } else { [1, 0] };
            for side in order {
                let (label, case) = if side == 0 {
                    ("ref", old)
                } else {
                    ("new", new)
                };
                got[side] = timed(&format!("{name}_{label}"), run, case);
            }
            if got[0] != got[1] {
                eprintln!("{name}: ref and new outputs differ in run {run}");
                speed::fail("outputs_differ");
            }
            outputs.extend_from_slice(name.as_bytes());
            outputs.extend_from_slice(&got[1]);
        }
    }
    speed::check_digest(&speed::sha256(&outputs));
}
