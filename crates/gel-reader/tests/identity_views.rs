// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Runs the comparison of `examples/identity_views.rs` under `cargo test`.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_views.rs"]
mod program;

#[test]
fn word_level_affine_views_match_the_per_bit_reference() {
    let tallies = program::run(program::identity::Scale::Full);
    assert!(program::identity::verdict(&tallies));
}
