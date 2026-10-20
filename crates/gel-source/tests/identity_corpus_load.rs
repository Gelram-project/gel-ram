//! Runs the catalog loader identity program at its small scale.
#[path = "../examples/identity_corpus_load.rs"]
#[allow(dead_code)]
mod program;

#[test]
fn catalog_loader_matches_the_one_thread_reference() {
    let tallies = program::run(program::identity::Scale::Small);
    let lines = program::identity::lines(&tallies);
    assert!(program::identity::verdict(&tallies), "{lines:#?}");
}
