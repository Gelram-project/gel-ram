//! Repeatable timing of the load, backup and sequential-add paths.
//!
//! Usage: `speed_source_load [RUNS] [WORK_DIR]` (RUNS 1..=999, default 9;
//! WORK_DIR defaults to the system temporary directory). Every input is
//! generated from fixed seeds, so two builds can be compared by their
//! `SPEED_CHECK` line. Only the public API is used.
//!
//! Cases and sizes:
//! - `adds_then_root`: 128 sequential `Collection::add` calls of 64 KiB each
//!   (8 MiB in all), then one `root()`.
//! - `bundle_load`: `load_bundle` of one 32 MiB imported text (1024 parts).
//! - `encoded_load`: `EncodedCorpus::load` of the same corpus.
//! - `collection_load`: `Collection::load` of a 32 MiB snapshot (32 x 1 MiB).
//! - `backup_create`, `backup_inspect`, `backup_restore`: the backup module on
//!   that snapshot, into a new directory and a new target on every run.
//!
//! Output: `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every produced result. Output grammar:
//! `docs/SPEED.md`.
use gel_source::{
    backup,
    collection::{Collection, CollectionBuilder},
    digest, hex, import_text, load_bundle, write_bundle_new,
};
use std::{fs, path::Path, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const ADD_DOCS: usize = 128;
const ADD_DOC_BYTES: usize = 64 << 10;
const BUNDLE_TEXT: usize = 32 << 20;
const SNAPSHOT_DOCS: usize = 32;
const SNAPSHOT_DOC_BYTES: usize = 1 << 20;

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut state = seed;
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        s.push_str(WORDS[(z % 8) as usize]);
        s.push(if z % 13 == 0 { '\n' } else { ' ' });
    }
    s.truncate(len);
    s
}

fn report(name: &str, run: usize, start: Instant) {
    speed::line(name, run, start.elapsed().as_nanos());
}

fn main() -> Result<(), String> {
    let args = speed::args("speed_source_load", "[RUNS] [WORK_DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(Into::into)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("gel-speed-source-load-{}", std::process::id()));
    fs::create_dir(&dir).map_err(|e| format!("WORK_DIR {e}"))?;
    let result = run(&dir, runs);
    let cleanup = fs::remove_dir_all(&dir);
    result?;
    cleanup.map_err(|e| format!("CLEANUP {e}"))
}

fn run(dir: &Path, runs: usize) -> Result<(), String> {
    let docs: Vec<String> = (0..ADD_DOCS)
        .map(|i| text(i as u64 + 1, ADD_DOC_BYTES))
        .collect();
    let corpus = import_text(text(1000, BUNDLE_TEXT).as_bytes(), "Speed")
        .map_err(|e| format!("IMPORT {e:?}"))?;
    let bundle = dir.join("source.gelsrc");
    let bundle_pin = write_bundle_new(&bundle, &corpus).map_err(|e| format!("BUNDLE {e:?}"))?;
    let mut staged = CollectionBuilder::new();
    for i in 0..SNAPSHOT_DOCS {
        staged.add(
            &format!("snapshot {i}"),
            &text(2000 + i as u64, SNAPSHOT_DOC_BYTES),
        )?;
    }
    let snapshot = dir.join("collection.gelset");
    let snapshot_pin = staged.build().save_new(&snapshot)?;
    speed::setup(
        "speed_source_load",
        runs,
        &[
            ("add_docs", ADD_DOCS.to_string()),
            ("add_doc_bytes", ADD_DOC_BYTES.to_string()),
            ("bundle_text_bytes", BUNDLE_TEXT.to_string()),
            ("snapshot_docs", SNAPSHOT_DOCS.to_string()),
            ("snapshot_doc_bytes", SNAPSHOT_DOC_BYTES.to_string()),
        ],
    );

    let mut out = Vec::new();
    let mut record = |name: &str, run: usize, value: String| {
        out.extend_from_slice(format!("{name} {run} {value}\n").as_bytes());
    };
    for run in 0..runs {
        let start = Instant::now();
        let mut bank = Collection::new();
        for (i, d) in docs.iter().enumerate() {
            bank.add(&format!("doc {i}"), d)?;
        }
        let root = bank.root();
        report("adds_then_root", run, start);
        record("adds_then_root", run, hex(&root));

        let start = Instant::now();
        let loaded = load_bundle(&bundle, bundle_pin).map_err(|e| format!("LOAD {e:?}"))?;
        report("bundle_load", run, start);
        record(
            "bundle_load",
            run,
            format!(
                "{} {} {}",
                hex(&loaded.root()),
                hex(&loaded.text_hash()),
                loaded.records().count()
            ),
        );
        drop(loaded);

        let start = Instant::now();
        let loaded = corpus.load().map_err(|e| format!("ENCODED {e:?}"))?;
        report("encoded_load", run, start);
        record(
            "encoded_load",
            run,
            format!("{} {}", hex(&loaded.root()), loaded.records().count()),
        );
        drop(loaded);

        let start = Instant::now();
        let reopened = Collection::load(&snapshot, snapshot_pin)?;
        report("collection_load", run, start);
        record(
            "collection_load",
            run,
            format!(
                "{} {} {}",
                hex(&reopened.root()),
                reopened.revision(),
                reopened.documents().count()
            ),
        );
        drop(reopened);

        let backup_dir = dir.join(format!("backup-{run}"));
        let start = Instant::now();
        let created = backup::create(&snapshot, snapshot_pin, &backup_dir);
        report("backup_create", run, start);
        record("backup_create", run, format!("{created:?}"));

        let start = Instant::now();
        let inspected = backup::inspect(&backup_dir, snapshot_pin);
        report("backup_inspect", run, start);
        record("backup_inspect", run, format!("{inspected:?}"));

        let target = dir.join(format!("restored-{run}"));
        let start = Instant::now();
        let restored = backup::restore(&backup_dir, snapshot_pin, &target);
        report("backup_restore", run, start);
        let restored_pin = fs::read(&target).map(|b| hex(&digest(&b)));
        record(
            "backup_restore",
            run,
            format!("{restored:?} {restored_pin:?}"),
        );
        let _ = fs::remove_file(&target);
        let _ = fs::remove_dir_all(&backup_dir);
    }
    speed::check_digest(&speed::sha256(&out));
    Ok(())
}
