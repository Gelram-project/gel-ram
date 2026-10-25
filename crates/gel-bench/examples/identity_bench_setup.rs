//! Identity check for the setup work of `gel-bench` and `gel-physics`: the
//! threaded bank fill, the threaded bank CRC, the selection-based Top-K
//! reference and the chain
//! order built once per size, each against an exact copy of the earlier
//! serial code, on seeded inputs including empty, tail and full sizes.
//!
//! Usage: cargo run --release -p gel-bench --example identity_bench_setup
//!
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL when any output
//! differs or when no case ran); exit status 0 on PASS, 1 on FAIL. Output
//! grammar: `docs/SPEED.md`.

#![forbid(unsafe_code)]

#[path = "../src/setup.rs"]
mod setup;
// `Tally::all_equal` is not needed here: the verdict comes from the counts.
#[allow(dead_code)]
#[path = "../src/setup_reference.rs"]
mod setup_reference;

#[path = "../../gel-physics/src/chain.rs"]
mod chain;
#[path = "../../gel-physics/src/chain_reference.rs"]
mod chain_reference;

#[path = "../../../tools/identity_support.rs"]
mod identity;
use identity::{Scale, Tally};
use std::process::ExitCode;

const GENERATOR_SEED: u64 = 0x4745_4C00_4245_4E43;
const DEFAULT_ORBS: usize = 1_310_720;

/// Runs every case. There is no smaller scale: `scale` is accepted for the
/// common interface only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    let mut results: Vec<Tally> = Vec::new();
    let mut push = |name, tally: setup_reference::Tally| {
        results.push(Tally::counted(name, tally.cases, tally.equal));
    };

    let auto_and_fixed = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 24];
    push(
        "bank_generate_small",
        setup_reference::compare_bank(
            &[
                0, 1, 2, 3, 7, 8, 63, 64, 1000, 8191, 8192, 16383, 16384, 16385, 32769, 100_003,
            ],
            &auto_and_fixed,
            GENERATOR_SEED,
        ),
    );
    push(
        "bank_generate_default",
        setup_reference::compare_bank(&[DEFAULT_ORBS], &[0, 1, 4, 24], GENERATOR_SEED),
    );
    push(
        "bank_generate_other_seed",
        setup_reference::compare_bank(&[0, 5, 9999, 65_536], &[0, 3, 7], 91),
    );
    push(
        "bank_crc_threaded",
        setup_reference::compare_bank_crc(
            &[
                0, 1, 2, 3, 7, 8, 63, 64, 1000, 4097, 16383, 16384, 16385, 32769, 100_003,
            ],
            &[0, 1, 2, 3, 4, 5, 7, 8, 9, 24, 100_004],
            GENERATOR_SEED,
        ),
    );
    push(
        "bank_crc_pin",
        setup_reference::compare_crc_pins(
            GENERATOR_SEED,
            &[
                (8192, 0x6f03_8d6d_b5b8_839f),
                (DEFAULT_ORBS, 0x7244_2fd2_64cb_251f),
            ],
        ),
    );
    push(
        "reference_top_k_small",
        setup_reference::compare_top_k_small(0..=200, &[1, 2, 3, 4, 5]),
    );
    let bank_8192 = setup_reference::new_bank(8192, GENERATOR_SEED, 4);
    push(
        "reference_top_k_8192",
        setup_reference::compare_top_k_bank(
            &bank_8192,
            &[0, 1, 8192 / 3, 8192 / 2, 8191],
            &[0, 1, 8, 1024, 8191, 8192, 8193],
            &[1, 2, 7, 24],
        ),
    );
    drop(bank_8192);
    let bank = setup_reference::new_bank(
        DEFAULT_ORBS,
        GENERATOR_SEED,
        setup::workers_for(DEFAULT_ORBS, setup::BANK_ORBS_PER_WORKER),
    );
    push(
        "reference_top_k_default",
        setup_reference::compare_top_k_bank(&bank, &[DEFAULT_ORBS / 3, 0], &[1, 8], &[]),
    );
    drop(bank);

    let sizes = [
        0,
        1,
        8,
        15,
        16,
        17,
        24,
        4104,
        48 << 10,
        1 << 20,
        1_000_003,
        6 << 20,
        12 << 20,
        24 << 20,
        64 << 20,
        256 << 20,
    ];
    let (cases, equal) = chain_reference::compare_chain(&sizes);
    results.push(Tally::counted("chain", cases, equal));
    results
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
