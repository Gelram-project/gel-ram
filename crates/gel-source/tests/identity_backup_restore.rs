// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Runs the comparison of `examples/identity_backup_restore.rs` under `cargo test`.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../examples/identity_backup_restore.rs"]
mod program;

#[test]
fn restore_matches_the_earlier_restore_on_seeded_backups() {
    let tallies = program::run(program::identity::Scale::Small);
    for line in program::identity::lines(&tallies) {
        println!("{line}");
    }
    assert!(program::identity::verdict(&tallies));
}
