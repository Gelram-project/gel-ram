//! Repeatable timing of `read_regular`, the bounded whole-file read used by
//! every bundle, snapshot, backup and document import path.
//!
//! Usage: `speed_read_regular [RUNS] [DIR]` (RUNS 1..=999, default 9; DIR
//! defaults to the system temporary directory, where a fresh subdirectory is
//! created and removed at the end). Every input is generated from fixed seeds
//! and only public API is used, so the same file builds on older trees and two
//! builds can be compared by their `SPEED_CHECK` line.
//!
//! Cases (one timed call each, files already written, in this order per run):
//! - `read_64mib`, `read_16mib`, `read_1mib`, `read_4kib`: `read_regular` of
//!   one ASCII file of that size under the limit `MAX_TEXT` (64 MiB, so the
//!   largest file is exactly at the limit).
//! - `bundle_load_64mib`: `load_bundle` of one imported 64 MiB text.
//! - `refused_64mib_plus1`: `read_regular` of a file one byte over
//!   `MAX_TEXT`, refused from its metadata before any read (control).
//! - `digest_64mib`: `digest` of 64 MiB already in memory (control).
//!
//! The SHA-256 of every result is taken outside the timed region.
//!
//! Output: `SPEED_SETUP example=speed_read_regular runs=<R> ...`, then
//! `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of every run (case name and
//! result, not the run number). On a failed step: `SPEED_FAIL reason=<slug>`
//! on stderr, no `SPEED_CHECK`, exit 1. A bad argument: exit 2. Output
//! grammar: `docs/SPEED.md`.
use gel_source::{digest, hex, import_text, load_bundle, read_regular, write_bundle_new, MAX_TEXT};
use std::{fs, path::Path, path::PathBuf, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const READ_CASES: [(&str, usize); 4] = [
    ("read_64mib", 64 << 20),
    ("read_16mib", 16 << 20),
    ("read_1mib", 1 << 20),
    ("read_4kib", 4 << 10),
];
const BUNDLE_TEXT: usize = 64 << 20;

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
    let args = speed::args("speed_read_regular", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    speed::self_check();
    let dir = base.join(format!("gel-speed-read-regular-{}", std::process::id()));
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

fn run(dir: &Path, runs: usize) -> Result<speed::Sha256, String> {
    let mut files = Vec::new();
    for (i, (name, len)) in READ_CASES.into_iter().enumerate() {
        let path = dir.join(format!("{name}.txt"));
        fs::write(&path, text(100 + i as u64, len)).map_err(|e| fail("write", e))?;
        files.push((name, path));
    }
    let over = dir.join("over.txt");
    fs::write(&over, text(200, MAX_TEXT + 1)).map_err(|e| fail("write", e))?;
    let corpus =
        import_text(text(1000, BUNDLE_TEXT).as_bytes(), "Speed").map_err(|e| fail("import", e))?;
    let bundle = dir.join("source.gelsrc");
    let bundle_pin = write_bundle_new(&bundle, &corpus).map_err(|e| fail("bundle_write", e))?;
    drop(corpus);
    let memory = text(300, 64 << 20).into_bytes();

    speed::setup(
        "speed_read_regular",
        runs,
        &[
            (
                "read_bytes",
                READ_CASES
                    .iter()
                    .map(|(_, len)| len.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            ("limit", MAX_TEXT.to_string()),
            ("bundle_text_bytes", BUNDLE_TEXT.to_string()),
        ],
    );

    let mut check = speed::Sha256::new();
    let mut record = |name: &str, value: String| {
        check.update(format!("{name} {value}\n").as_bytes());
    };
    let report = |name: &str, run: usize, start: Instant| {
        speed::line(name, run, start.elapsed().as_nanos());
    };
    for run in 0..runs {
        for (name, path) in &files {
            let start = Instant::now();
            let bytes = read_regular(path, MAX_TEXT);
            report(name, run, start);
            let bytes = bytes.map_err(|e| fail("read", e))?;
            record(
                name,
                format!("{} {}", bytes.len(), speed::hex(&speed::sha256(&bytes))),
            );
        }

        let start = Instant::now();
        let loaded = load_bundle(&bundle, bundle_pin);
        report("bundle_load_64mib", run, start);
        let loaded = loaded.map_err(|e| fail("bundle_load", e))?;
        record(
            "bundle_load_64mib",
            format!(
                "{} {} {}",
                hex(&loaded.root()),
                hex(&loaded.text_hash()),
                loaded.records().count()
            ),
        );
        drop(loaded);

        let start = Instant::now();
        let refused = read_regular(&over, MAX_TEXT);
        report("refused_64mib_plus1", run, start);
        match refused {
            Ok(bytes) => return Err(fail("not_refused", bytes.len())),
            Err(e) => record("refused_64mib_plus1", format!("{e:?}")),
        }

        let start = Instant::now();
        let pin = digest(&memory);
        report("digest_64mib", run, start);
        record("digest_64mib", hex(&pin));
    }
    Ok(check)
}
