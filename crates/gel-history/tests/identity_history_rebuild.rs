//! The identity comparison of examples/identity_history_rebuild.rs at a
//! smaller scale: the streaming sparse decoder and the rolled history rebuild
//! against exact copies of the code they replace, results and errors equal.

#[allow(dead_code)]
#[path = "../examples/identity_history_rebuild.rs"]
mod program;

#[test]
fn streaming_decoder_and_rolled_rebuild_equal_the_reference() {
    let tallies = program::run(program::Scale::Small);
    for t in &tallies {
        assert!(t.cases > 0, "{}: no case ran", t.name);
        assert_eq!(t.equal, t.cases, "{}", t.name);
    }
    assert!(program::verdict(&tallies));
}
