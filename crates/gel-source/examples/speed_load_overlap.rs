//! Repeatable timing of loading a source bundle and a collection snapshot.
//!
//! Usage: `speed_load_overlap [RUNS] [DIR]` (RUNS 1..=999, default 9; DIR
//! defaults to the system temporary directory, where a fresh subdirectory is
//! created and removed at the end). Every input is generated from fixed seeds
//! and only public API is used, so the same file builds on older trees and two
//! builds can be compared by their `SPEED_CHECK` line.
//!
//! Cases (one timed call each, file already written, in this order per run):
//! - `bundle_load_64mib`: `load_bundle` of one imported 64 MiB ASCII text.
//! - `bundle_load_512kib`: `load_bundle` of one imported 512 KiB text.
//! - `encoded_load_64mib`: `EncodedCorpus::load` of the 64 MiB corpus.
//! - `collection_load_64mib`: `Collection::load` of 64 documents of 1 MiB.
//! - `collection_load_64mib_wrong_pin`: the same file under a wrong pin.
//! - `collection_load_512kib`: `Collection::load` of 8 documents of 64 KiB.
//! - `backup_inspect_64mib`: `backup::inspect` of a backup of the 64 MiB
//!   snapshot.
//! - `bundle_load_64mib_wrong_pin`: `load_bundle` of the 64 MiB bundle under
//!   a wrong pin (refused).
//!
//! Output: `SPEED_SETUP example=speed_load_overlap runs=<R> ...`, then
//! `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of every run (case name and
//! result, not the run number). On a failed step: `SPEED_FAIL reason=<slug>`
//! on stderr, no `SPEED_CHECK`, exit 1. A bad argument: exit 2. Output
//! grammar: `docs/SPEED.md`.
use gel_source::{backup, collection::Collection, collection::CollectionBuilder, hex};
use gel_source::{import_text, load_bundle, write_bundle_new};
use std::{fs, path::Path, path::PathBuf, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const BUNDLE_TEXT: usize = 64 << 20;
const SMALL_TEXT: usize = 512 << 10;
const SNAPSHOT_DOCS: usize = 64;
const SNAPSHOT_DOC_BYTES: usize = 1 << 20;
const SMALL_DOCS: usize = 8;
const SMALL_DOC_BYTES: usize = 64 << 10;

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

fn fail(reason: &str, detail: impl std::fmt::Debug) -> String {
    eprintln!("{detail:?}");
    reason.to_string()
}

fn main() -> ExitCode {
    let args = speed::args("speed_load_overlap", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    speed::self_check();
    let dir = base.join(format!("gel-speed-load-overlap-{}", std::process::id()));
    if let Err(e) = fs::create_dir(&dir) {
        eprintln!("{e}");
        speed::fail("work_dir");
    }
    let result = run(&dir, runs);
    let cleanup = fs::remove_dir_all(&dir);
    match (result, cleanup) {
        (Ok(check), Ok(())) => {
            speed::check(check);
            ExitCode::SUCCESS
        }
        (Err(reason), _) => speed::fail(&reason),
        (Ok(_), Err(e)) => {
            eprintln!("{e}");
            speed::fail("cleanup");
        }
    }
}

fn snapshot(
    dir: &Path,
    name: &str,
    docs: usize,
    bytes: usize,
    seed: u64,
) -> Result<[u8; 32], String> {
    let mut staged = CollectionBuilder::new();
    for i in 0..docs {
        staged
            .add(&format!("document {i}"), &text(seed + i as u64, bytes))
            .map_err(|e| fail("snapshot_add", e))?;
    }
    staged
        .build()
        .save_new(&dir.join(name))
        .map_err(|e| fail("snapshot_save", e))
}

fn run(dir: &Path, runs: usize) -> Result<speed::Sha256, String> {
    let corpus =
        import_text(text(1000, BUNDLE_TEXT).as_bytes(), "Speed").map_err(|e| fail("import", e))?;
    let bundle = dir.join("source.gelsrc");
    let bundle_pin = write_bundle_new(&bundle, &corpus).map_err(|e| fail("bundle_write", e))?;
    let small_corpus = import_text(text(1001, SMALL_TEXT).as_bytes(), "Small")
        .map_err(|e| fail("import_small", e))?;
    let small_bundle = dir.join("small.gelsrc");
    let small_bundle_pin =
        write_bundle_new(&small_bundle, &small_corpus).map_err(|e| fail("bundle_write", e))?;
    let snapshot_pin = snapshot(
        dir,
        "collection.gelset",
        SNAPSHOT_DOCS,
        SNAPSHOT_DOC_BYTES,
        2000,
    )?;
    let small_pin = snapshot(dir, "small.gelset", SMALL_DOCS, SMALL_DOC_BYTES, 3000)?;
    let backup_dir = dir.join("backup");
    backup::create(&dir.join("collection.gelset"), snapshot_pin, &backup_dir)
        .map_err(|e| fail("backup_create", e))?;
    let mut wrong_pin = snapshot_pin;
    wrong_pin[0] ^= 1;
    let mut bundle_wrong_pin = bundle_pin;
    bundle_wrong_pin[0] ^= 1;

    speed::setup(
        "speed_load_overlap",
        runs,
        &[
            ("bundle_bytes", BUNDLE_TEXT.to_string()),
            ("small_bundle_bytes", SMALL_TEXT.to_string()),
            ("snapshot_docs", SNAPSHOT_DOCS.to_string()),
            ("snapshot_doc_bytes", SNAPSHOT_DOC_BYTES.to_string()),
            ("small_docs", SMALL_DOCS.to_string()),
            ("small_doc_bytes", SMALL_DOC_BYTES.to_string()),
            (
                "available_parallelism",
                std::thread::available_parallelism()
                    .map_or(1, usize::from)
                    .to_string(),
            ),
        ],
    );

    let mut check = speed::Sha256::new();
    let mut record = |name: &str, value: String| {
        check.update(format!("{name} {value}\n").as_bytes());
    };
    let report = |name: &str, run: usize, start: Instant| {
        let ns = start.elapsed().as_nanos();
        speed::line(name, run, ns);
    };
    for run in 0..runs {
        for (name, path, pin) in [
            ("bundle_load_64mib", &bundle, bundle_pin),
            ("bundle_load_512kib", &small_bundle, small_bundle_pin),
        ] {
            let start = Instant::now();
            let loaded = load_bundle(path, pin);
            report(name, run, start);
            let loaded = loaded.map_err(|e| fail("bundle_load", e))?;
            record(
                name,
                format!(
                    "{} {} {}",
                    hex(&loaded.root()),
                    hex(&loaded.text_hash()),
                    loaded.records().count()
                ),
            );
        }

        let start = Instant::now();
        let loaded = corpus.load();
        report("encoded_load_64mib", run, start);
        let loaded = loaded.map_err(|e| fail("encoded_load", e))?;
        record(
            "encoded_load_64mib",
            format!("{} {}", hex(&loaded.root()), loaded.records().count()),
        );
        drop(loaded);

        for (name, file, pin) in [
            ("collection_load_64mib", "collection.gelset", snapshot_pin),
            (
                "collection_load_64mib_wrong_pin",
                "collection.gelset",
                wrong_pin,
            ),
            ("collection_load_512kib", "small.gelset", small_pin),
        ] {
            let path = dir.join(file);
            let start = Instant::now();
            let loaded = Collection::load(&path, pin);
            report(name, run, start);
            let value = match loaded {
                Ok(c) => format!(
                    "{} {} {} {}",
                    hex(&c.root()),
                    c.revision(),
                    c.documents().count(),
                    c.text_bytes()
                ),
                Err(e) => e,
            };
            record(name, value);
        }

        let start = Instant::now();
        let inspected = backup::inspect(&backup_dir, snapshot_pin);
        report("backup_inspect_64mib", run, start);
        record("backup_inspect_64mib", format!("{inspected:?}"));

        let start = Instant::now();
        let refused = load_bundle(&bundle, bundle_wrong_pin);
        report("bundle_load_64mib_wrong_pin", run, start);
        match refused {
            Ok(_) => return Err(fail("bundle_wrong_pin", "loaded under a wrong pin")),
            Err(e) => record("bundle_load_64mib_wrong_pin", format!("{e:?}")),
        }
    }
    Ok(check)
}
