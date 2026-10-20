//! Runs the comparison of `examples/identity_store_crc.rs` under `cargo test`
//! on a short subset of record counts and file variants.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_store_crc.rs"]
mod program;

#[test]
fn store_payload_crc_paths_match_the_earlier_code() {
    let tallies = program::run(program::identity::Scale::Small);
    for line in program::identity::lines(&tallies) {
        println!("{line}");
    }
    assert!(program::identity::verdict(&tallies));
}
