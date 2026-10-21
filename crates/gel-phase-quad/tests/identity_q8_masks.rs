//! Runs the comparison of `examples/identity_q8_masks.rs` under `cargo test`.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_q8_masks.rs"]
mod program;

#[test]
fn word_level_mask_paths_match_the_per_position_reference() {
    let tallies = program::run(program::identity::Scale::Full);
    assert!(program::identity::verdict(&tallies));
}
