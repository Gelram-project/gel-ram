// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Runs the comparison of `examples/identity_catalog_parse.rs` under `cargo test`,
//! at a smaller scale.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_catalog_parse.rs"]
mod program;

#[test]
fn catalog_reader_and_writer_match_the_earlier_code() {
    let tallies = program::run(program::identity::Scale::Small);
    assert!(program::identity::verdict(&tallies));
}
