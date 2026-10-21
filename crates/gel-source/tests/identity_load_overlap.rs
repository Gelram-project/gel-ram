//! Runs the comparison of `examples/identity_load_overlap.rs` under `cargo test`,
//! at the small scale (the inputs still reach the threaded load path).
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_load_overlap.rs"]
mod program;

#[test]
fn threaded_loads_match_the_one_thread_reference() {
    let tallies = program::run(program::identity::Scale::Small);
    assert!(program::identity::verdict(&tallies));
}
