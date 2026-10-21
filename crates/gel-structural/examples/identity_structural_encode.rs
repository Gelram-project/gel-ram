//! Compares the popcount-first `Residual::from_exact_xor`,
//! `EncodedOrb::encode_against` and `encode_best_prototype` with verbatim
//! copies of the earlier encoder on seeded and edge-case inputs.
//!
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per group and one final
//! line `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL`. The verdict is
//! FAIL when any output differs or when no comparison ran; exit status 0 on
//! PASS, 1 on FAIL. Output grammar: `docs/SPEED.md`.
//!
//! Usage: cargo run --release -p gel-structural --example identity_structural_encode

use gel_structural::{encode_best_prototype, EncodedOrb, ParentRef, Residual};
use std::process::ExitCode;

#[path = "../src/popcount_first_reference.rs"]
mod popcount_first_reference;

#[path = "../../../tools/identity_support.rs"]
mod identity;
use identity::{Scale, Tally};

/// Runs every comparison group, in the fixed order of the reference module.
/// There is no smaller scale: `scale` is accepted for the common interface
/// only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    popcount_first_reference::identity_cases()
        .into_iter()
        .map(|group| Tally::counted(group.name, group.cases, group.equal))
        .collect()
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
