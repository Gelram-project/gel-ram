//! Runs the comparison of `examples/identity_tree_gates.rs` under `cargo test`,
//! at the small scale: generated trees only, no clippy-driver.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_tree_gates.rs"]
mod program;

#[test]
fn tree_gates_match_the_sequential_reference() {
    let tallies = program::run(program::identity::Scale::Small);
    let lines = program::identity::lines(&tallies);
    assert!(program::identity::verdict(&tallies), "{}", lines.join("\n"));
}
