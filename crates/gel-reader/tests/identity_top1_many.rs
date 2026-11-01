// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Runs the comparison of `examples/identity_top1_many.rs` under `cargo test`.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_top1_many.rs"]
mod program;

#[test]
fn blocked_top1_many_matches_the_per_orb_reference() {
    let tallies = program::run(program::identity::Scale::Small);
    for t in &tallies {
        assert_eq!(t.equal, t.cases, "{}", t.name);
    }
    assert!(program::identity::verdict(&tallies));
}
