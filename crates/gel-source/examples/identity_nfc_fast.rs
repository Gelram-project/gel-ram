//! Identity check for the NFC of gel-source: properties read from one
//! two-level table, primary composites from a hash table, buffers reused
//! across lines and the prefix that passed the quick check copied as it is,
//! against an exact copy of the earlier binary-search implementation
//! (`src/nfc_reference.rs`). Every scalar value is compared for each property
//! and for the NFC of one character; composition pairs, the Unicode
//! NormalizationTest 15.1, the seeded sets of the pinned unit tests, seeded
//! strings with a prefix that passes the quick check, and long lines are
//! compared for the whole normalization. The line end search of document
//! search, eight bytes at a time (`src/line_end.rs`), is compared with the
//! earlier byte-by-byte search on short buffers with marked bytes at every
//! position, seeded buffers and texts made of the long lines.
//!
//! Usage: `cargo run --release -p gel-source --example identity_nfc_fast`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when no case
//! ran); exit status 0 on PASS, 1 on FAIL. The unit test in
//! `src/nfc_fast_identity.rs` runs the same comparison at a smaller scale.
//! Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
// The unit tests inside these files run with the library; a test build of
// this example (`cargo test --all-targets`) leaves them out.
#[cfg(not(test))]
#[path = "../src/line_end.rs"]
mod line_end;
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "../src/nfc.rs"]
mod nfc;
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "../src/nfc_fast_identity.rs"]
mod nfc_fast_identity;
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "../src/nfc_reference.rs"]
mod nfc_reference;
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "../src/nfc_tables.rs"]
mod nfc_tables;

#[cfg(not(test))]
#[path = "../../../tools/identity_support.rs"]
pub mod identity;
#[cfg(not(test))]
use identity::{Scale, Tally};

/// The comparison of `src/nfc_fast_identity.rs`, one tally per case, in its order.
#[cfg(not(test))]
pub fn run(scale: Scale) -> Vec<Tally> {
    let scale = match scale {
        Scale::Full => nfc_fast_identity::Scale::Full,
        Scale::Small => nfc_fast_identity::Scale::Small,
    };
    let count = |n: u64| usize::try_from(n).expect("case count fits usize");
    nfc_fast_identity::run(scale)
        .iter()
        .map(|t| Tally::counted(t.name, count(t.cases), count(t.equal)))
        .collect()
}

#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    if std::env::args().len() > 1 {
        eprintln!("usage: identity_nfc_fast");
        return std::process::ExitCode::from(2);
    }
    identity::report(&run(Scale::Full))
}
