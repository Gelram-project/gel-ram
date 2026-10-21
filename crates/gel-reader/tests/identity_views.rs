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
