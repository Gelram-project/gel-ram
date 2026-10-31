//! Repeatable timing of `Collection::load` of one 64 MiB snapshot, with a setup
//! that needs less memory than one load, so that the peak resident memory of
//! the whole process is the peak of a load. Run it under `/usr/bin/time -v` and
//! read "Maximum resident set size" (VmHWM) to compare the peak memory of a
//! collection load between two builds.
//!
//! Usage: `speed_collection_memory [RUNS] [DIR]` (RUNS 1..=999, default 9; DIR
//! defaults to the system temporary directory, where a fresh subdirectory is
//! created and removed at the end). Every input is generated from fixed seeds
//! and only public API is used, so the same file builds on older trees and two
//! builds can be compared by their `SPEED_CHECK` line.
//!
//! Setup: a snapshot of 64 ASCII documents of 1 MiB (ids 1..=64, titles
//! `Speed <id>`, revision 64), as 64 sequential `add` calls make it, written
//! piece by piece in the GELSET01 layout: the magic, the revision, the next id
//! and the count, then per document its id, title length, text length, title
//! and text, each text made just before it is written. A small snapshot written
//! the same way is first checked to be byte for byte the file
//! `Collection::save_new` writes. The pin is `digest_reader` of the file.
//!
//! Cases (one timed call each, file already written, per run):
//! - `collection_load_64mib`: `Collection::load` of the 64 MiB snapshot.
//!
//! The SHA-256 of every result (root, revision, document count, text bytes, and
//! per document its id, title, stated hash and the digest of its text) is taken
//! outside the timed region.
//!
//! Output: `SPEED_SETUP example=speed_collection_memory runs=<R> ...`, then
//! `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of every run (case name and
//! result, not the run number). On a failed step: `SPEED_FAIL reason=<slug>`
//! on stderr, no `SPEED_CHECK`, exit 1. A bad argument: exit 2. Output
//! grammar: `docs/SPEED.md`.
use gel_source::{
    collection::{Collection, CollectionBuilder},
    digest, digest_reader, hex,
};
use std::{fs, io::Write, path::Path, path::PathBuf, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const DOCUMENTS: usize = 64;
const DOCUMENT_BYTES: usize = 1 << 20;

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut generator = speed::SplitMix64(seed);
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        let z = generator.next();
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
    let args = speed::args("speed_collection_memory", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    speed::self_check();
    let dir = base.join(format!(
        "gel-speed-collection-memory-{}",
        std::process::id()
    ));
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

/// The snapshot of `documents` sequential adds of `len`-byte texts from
/// `seed`, written piece by piece to a new file.
fn write_pieces(path: &Path, seed: u64, documents: usize, len: usize) -> Result<(), String> {
    let mut file = fs::File::create_new(path).map_err(|e| fail("snapshot_create", e))?;
    let count = documents as u64;
    let mut put = |piece: &[u8]| file.write_all(piece).map_err(|e| fail("snapshot_write", e));
    put(b"GELSET01")?;
    for n in [count, count + 1, count] {
        put(&n.to_le_bytes())?;
    }
    for id in 1..=count {
        let title = format!("Speed {id}");
        let text = text(seed + id, len);
        put(&id.to_le_bytes())?;
        put(&(title.len() as u32).to_le_bytes())?;
        put(&(text.len() as u64).to_le_bytes())?;
        put(title.as_bytes())?;
        put(text.as_bytes())?;
    }
    file.sync_all().map_err(|e| fail("snapshot_sync", e))
}

fn run(dir: &Path, runs: usize) -> Result<speed::Sha256, String> {
    // The piecewise writer against the library's save, on a small snapshot.
    let mut small = CollectionBuilder::new();
    for id in 1..=3u64 {
        small
            .add(&format!("Speed {id}"), &text(7 + id, 1000))
            .map_err(|e| fail("small_add", e))?;
    }
    let small = small.build();
    let (saved, pieces) = (dir.join("saved.gelset"), dir.join("pieces.gelset"));
    small.save_new(&saved).map_err(|e| fail("small_save", e))?;
    write_pieces(&pieces, 7, 3, 1000)?;
    let same = fs::read(&saved).map_err(|e| fail("small_read", e))?
        == fs::read(&pieces).map_err(|e| fail("small_read", e))?;
    if !same {
        return Err(fail("writer_differs", "piecewise snapshot differs"));
    }
    drop(small);

    let snapshot = dir.join("snapshot.gelset");
    write_pieces(&snapshot, 1000, DOCUMENTS, DOCUMENT_BYTES)?;
    let file = fs::File::open(&snapshot).map_err(|e| fail("snapshot_open", e))?;
    let pin = digest_reader(file).map_err(|e| fail("snapshot_digest", e))?;
    let snapshot_bytes = fs::metadata(&snapshot)
        .map_err(|e| fail("snapshot_metadata", e))?
        .len();

    speed::setup(
        "speed_collection_memory",
        runs,
        &[
            ("documents", DOCUMENTS.to_string()),
            ("document_bytes", DOCUMENT_BYTES.to_string()),
            ("snapshot_bytes", snapshot_bytes.to_string()),
            (
                "available_parallelism",
                std::thread::available_parallelism()
                    .map_or(1, usize::from)
                    .to_string(),
            ),
        ],
    );

    let mut check = speed::Sha256::new();
    for run in 0..runs {
        let start = Instant::now();
        let loaded = Collection::load(&snapshot, pin);
        let ns = start.elapsed().as_nanos();
        speed::line("collection_load_64mib", run, ns);
        let loaded = loaded.map_err(|e| fail("collection_load", e))?;
        let mut value = format!(
            "{} {} {} {}",
            hex(&loaded.root()),
            loaded.revision(),
            loaded.documents().count(),
            loaded.text_bytes()
        );
        for d in loaded.documents() {
            value.push_str(&format!(
                " {} {:?} {} {}",
                d.id(),
                d.title(),
                hex(&d.hash()),
                hex(&digest(d.text().as_bytes()))
            ));
        }
        drop(loaded);
        check.update(format!("collection_load_64mib {value}\n").as_bytes());
    }
    Ok(check)
}
