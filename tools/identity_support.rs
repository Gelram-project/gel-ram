#![allow(dead_code)]
//! Shared helpers for the identity programs, `crates/*/examples/identity_*.rs`.
//!
//! Included from an example with
//! `#[path = "../../../tools/identity_support.rs"] mod identity;`
//! so no crate gains a dependency. Pure `std`.
//!
//! Output grammar (stdout, see `docs/SPEED.md`):
//! - `IDENTITY name=<case> cases=<n> equal=<n>` per case;
//! - `IDENTITY_VERDICT=PASS|FAIL` once, last.
//!
//! PASS when at least one case ran, every case made at least one comparison
//! and every comparison was equal. Exit 0 on PASS, 1 on FAIL.

use std::process::ExitCode;

/// How much of the input space a program covers. `main` always runs `Full`;
/// a test wrapper may run `Small`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Full,
    Small,
}

/// Comparisons of one named case: how many ran and how many were equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tally {
    pub name: String,
    pub cases: usize,
    pub equal: usize,
}

impl Tally {
    /// An empty tally named `name` (`[a-z0-9_]+`).
    pub fn new(name: &str) -> Self {
        Self::counted(name, 0, 0)
    }

    /// A tally with counts that were made elsewhere.
    pub fn counted(name: &str, cases: usize, equal: usize) -> Self {
        assert!(
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
            "identity case name {name:?}"
        );
        Self {
            name: name.to_string(),
            cases,
            equal,
        }
    }

    /// Counts one comparison; `same` is its result.
    pub fn check(&mut self, same: bool) {
        self.cases += 1;
        self.equal += usize::from(same);
    }

    /// Counts one comparison of `got` with `expected`.
    pub fn check_eq<T: PartialEq>(&mut self, got: T, expected: T) {
        self.check(got == expected);
    }

    /// Counts one comparison per item of `results`.
    pub fn check_all(&mut self, results: impl IntoIterator<Item = bool>) {
        for same in results {
            self.check(same);
        }
    }

    /// At least one comparison ran and every one was equal.
    pub fn passed(&self) -> bool {
        self.cases > 0 && self.equal == self.cases
    }
}

/// PASS: at least one case, and every case passed.
pub fn verdict(tallies: &[Tally]) -> bool {
    !tallies.is_empty() && tallies.iter().all(Tally::passed)
}

/// The output lines: one `IDENTITY` line per case, then `IDENTITY_VERDICT`.
pub fn lines(tallies: &[Tally]) -> Vec<String> {
    let mut out: Vec<String> = tallies
        .iter()
        .map(|t| {
            format!(
                "IDENTITY name={} cases={} equal={}",
                t.name, t.cases, t.equal
            )
        })
        .collect();
    out.push(format!(
        "IDENTITY_VERDICT={}",
        if verdict(tallies) { "PASS" } else { "FAIL" }
    ));
    out
}

/// Prints `lines(tallies)`; exit status 0 on PASS, 1 on FAIL.
pub fn report(tallies: &[Tally]) -> ExitCode {
    for line in lines(tallies) {
        println!("{line}");
    }
    if verdict(tallies) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
