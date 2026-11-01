// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
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
