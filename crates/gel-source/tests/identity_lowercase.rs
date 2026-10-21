//! Runs the comparison of `examples/identity_lowercase.rs` under `cargo test`.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_lowercase.rs"]
mod program;

#[test]
fn line_lowercasing_and_search_match_the_earlier_code() {
    let tallies = program::run(program::identity::Scale::Small);
    let lines = program::identity::lines(&tallies);
    assert!(program::identity::verdict(&tallies), "{lines:?}");
}
