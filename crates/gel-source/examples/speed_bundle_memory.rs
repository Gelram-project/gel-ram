//! Repeatable timing of `load_bundle` of one 64 MiB text, with a setup that
//! needs less memory than one load, so that the peak resident memory of the
//! whole process is the peak of a load. Run it under `/usr/bin/time -v` and
//! read "Maximum resident set size" (VmHWM) to compare the peak memory of a
//! bundle load between two builds.
//!
//! Usage: `speed_bundle_memory [RUNS] [DIR]` (RUNS 1..=999, default 9; DIR
//! defaults to the system temporary directory, where a fresh subdirectory is
//! created and removed at the end). Every input is generated from fixed seeds
//! and only public API is used, so the same file builds on older trees and two
//! builds can be compared by their `SPEED_CHECK` line.
//!
//! Setup: the corpus is built with `CorpusBuilder` from 2,048 ASCII passages of
//! 32 KiB, made one at a time, as `import_text` splits one 64 MiB ASCII text
//! (node 1, entry 1, roles 1..=2048, title `Speed`, sections `Lead (<role>)`).
//! The bundle file is written piece by piece from the encoded corpus: the
//! magic, the two lengths, the catalog and the text. A small corpus written the
//! same way is first checked to be byte for byte the file `write_bundle_new`
//! writes. The pin is `digest_reader` of the file, and the corpus is dropped
//! before the first load.
//!
//! Cases (one timed call each, file already written, per run):
//! - `bundle_load_64mib`: `load_bundle` of the 64 MiB bundle.
//!
//! The SHA-256 of every result (root, text hash, record count, text length and
//! the digest of the source text) is taken outside the timed region.
//!
//! Output: `SPEED_SETUP example=speed_bundle_memory runs=<R> ...`, then
//! `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of every run (case name and
//! result, not the run number). On a failed step: `SPEED_FAIL reason=<slug>`
//! on stderr, no `SPEED_CHECK`, exit 1. A bad argument: exit 2. Output
//! grammar: `docs/SPEED.md`.
use gel_source::{
    digest, digest_reader, hex, load_bundle, write_bundle_new, Address, CorpusBuilder,
    EncodedCorpus, MAX_PASSAGE,
};
use std::{fs, io::Write, path::Path, path::PathBuf, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const PASSAGES: usize = 2048;
const TEXT_BYTES: usize = PASSAGES * MAX_PASSAGE;

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
    let args = speed::args("speed_bundle_memory", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    speed::self_check();
    let dir = base.join(format!("gel-speed-bundle-memory-{}", std::process::id()));
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

/// `passages` passages of at most `MAX_PASSAGE` bytes, each made just before
/// it is pushed, so the builder holds the only copy of the text.
fn corpus(seed: u64, passages: usize, len: usize) -> Result<EncodedCorpus, String> {
    let mut builder = CorpusBuilder::new();
    for role in 1..=passages as u64 {
        let passage = text(seed + role, len);
        builder
            .push(
                Address { node: 1, role },
                1,
                "Speed",
                &format!("Lead ({role})"),
                &passage,
            )
            .map_err(|e| fail("push", e))?;
    }
    builder.finish().map_err(|e| fail("finish", e))
}

/// The bundle bytes of `corpus`, written piece by piece to a new file.
fn write_pieces(path: &Path, corpus: &EncodedCorpus) -> Result<(), String> {
    let (catalog, text) = (corpus.catalog_bytes(), corpus.text_bytes());
    let mut file = fs::File::create_new(path).map_err(|e| fail("bundle_create", e))?;
    for piece in [
        &b"GELSRC01"[..],
        &(catalog.len() as u64).to_le_bytes(),
        &(text.len() as u64).to_le_bytes(),
        catalog,
        text,
    ] {
        file.write_all(piece).map_err(|e| fail("bundle_write", e))?;
    }
    file.sync_all().map_err(|e| fail("bundle_sync", e))
}

fn run(dir: &Path, runs: usize) -> Result<speed::Sha256, String> {
    // The piecewise writer against the library's publisher, on a small corpus.
    let small = corpus(7, 3, 1000)?;
    let (published, pieces) = (dir.join("published.gelsrc"), dir.join("pieces.gelsrc"));
    write_bundle_new(&published, &small).map_err(|e| fail("small_publish", e))?;
    write_pieces(&pieces, &small)?;
    let same = fs::read(&published).map_err(|e| fail("small_read", e))?
        == fs::read(&pieces).map_err(|e| fail("small_read", e))?;
    if !same {
        return Err(fail("writer_differs", "piecewise bundle differs"));
    }
    drop(small);

    let bundle = dir.join("source.gelsrc");
    let big = corpus(1000, PASSAGES, MAX_PASSAGE)?;
    if big.text_bytes().len() != TEXT_BYTES {
        return Err(fail("text_size", big.text_bytes().len()));
    }
    let catalog_bytes = big.catalog_bytes().len();
    write_pieces(&bundle, &big)?;
    drop(big);
    let file = fs::File::open(&bundle).map_err(|e| fail("bundle_open", e))?;
    let bundle_pin = digest_reader(file).map_err(|e| fail("bundle_digest", e))?;
    let bundle_bytes = fs::metadata(&bundle)
        .map_err(|e| fail("bundle_metadata", e))?
        .len();

    speed::setup(
        "speed_bundle_memory",
        runs,
        &[
            ("text_bytes", TEXT_BYTES.to_string()),
            ("passages", PASSAGES.to_string()),
            ("catalog_bytes", catalog_bytes.to_string()),
            ("bundle_bytes", bundle_bytes.to_string()),
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
        let loaded = load_bundle(&bundle, bundle_pin);
        let ns = start.elapsed().as_nanos();
        speed::line("bundle_load_64mib", run, ns);
        let loaded = loaded.map_err(|e| fail("bundle_load", e))?;
        let value = format!(
            "{} {} {} {} {}",
            hex(&loaded.root()),
            hex(&loaded.text_hash()),
            loaded.records().count(),
            loaded.source_text().len(),
            hex(&digest(loaded.source_text().as_bytes()))
        );
        drop(loaded);
        check.update(format!("bundle_load_64mib {value}\n").as_bytes());
    }
    Ok(check)
}
