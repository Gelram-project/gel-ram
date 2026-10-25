//! Repeatable timing of `literal::read`, the file step of `--literal FILE`.
//!
//! Usage: `speed_literal [RUNS] [WORK_DIR]` (RUNS 1..=999, default 9;
//! WORK_DIR defaults to the system temporary directory). Every input file is
//! generated from a fixed seed, so two builds can be compared by their
//! `SPEED_CHECK` line. Only the public API is used.
//!
//! Cases, each a fixed number of reads per run:
//! - `literal_64mib`: one file of exactly `MAX_TEXT` (64 MiB) bytes, 4 reads.
//! - `literal_16mib`: one 16 MiB file, 8 reads.
//! - `literal_1mib`: one 1 MiB file, 64 reads.
//! - `literal_64kib`: one 64 KiB file, 1024 reads.
//! - `literal_small`: files of 1, 1023, 1024 and 1025 bytes, 256 reads each.
//! - `literal_refused`: a sparse file of `MAX_TEXT` + 1 bytes, an empty file,
//!   a directory and a missing path, 256 reads each; every read is refused.
//!
//! Output: `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over the pin, placed and total length of every
//! accepted read and the text of every refusal, in every run. Output grammar:
//! `docs/SPEED.md`.
use gel_live_lab::literal;
use gel_source::{digest, hex, MAX_TEXT};
use std::{
    fs::{self, File},
    hint::black_box,
    path::{Path, PathBuf},
    time::Instant,
};

#[path = "../../../tools/speed_support.rs"]
mod speed;

/// Deterministic bytes of exactly `len` bytes.
fn bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = speed::SplitMix64(seed);
    let mut out = Vec::with_capacity(len + 8);
    while out.len() < len {
        out.extend_from_slice(&rng.next().to_le_bytes());
    }
    out.truncate(len);
    out
}

fn main() -> Result<(), String> {
    let args = speed::args("speed_literal", "[RUNS] [WORK_DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(Into::into)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("gel-speed-literal-{}", std::process::id()));
    fs::create_dir(&dir).map_err(|e| format!("WORK_DIR {e}"))?;
    let result = run(&dir, runs);
    let cleanup = fs::remove_dir_all(&dir);
    result?;
    cleanup.map_err(|e| format!("CLEANUP {e}"))
}

fn file(dir: &Path, name: &str, seed: u64, len: usize) -> Result<PathBuf, String> {
    let path = dir.join(name);
    fs::write(&path, bytes(seed, len)).map_err(|e| format!("WRITE {name} {e}"))?;
    Ok(path)
}

fn run(dir: &Path, runs: usize) -> Result<(), String> {
    let small: Vec<PathBuf> = [1, 1023, 1024, 1025]
        .iter()
        .enumerate()
        .map(|(i, &len)| file(dir, &format!("small{len}"), 10 + i as u64, len))
        .collect::<Result<_, _>>()?;
    let over = dir.join("over");
    File::create(&over)
        .and_then(|f| f.set_len(MAX_TEXT as u64 + 1))
        .map_err(|e| format!("SPARSE {e}"))?;
    let empty = file(dir, "empty", 0, 0)?;
    let sub = dir.join("directory");
    fs::create_dir(&sub).map_err(|e| format!("DIR {e}"))?;
    let refused = vec![over, empty, sub, dir.join("missing")];
    let cases: Vec<(&str, Vec<PathBuf>, usize)> = vec![
        ("literal_64mib", vec![file(dir, "f64mib", 1, MAX_TEXT)?], 4),
        ("literal_16mib", vec![file(dir, "f16mib", 2, 16 << 20)?], 8),
        ("literal_1mib", vec![file(dir, "f1mib", 3, 1 << 20)?], 64),
        (
            "literal_64kib",
            vec![file(dir, "f64kib", 4, 64 << 10)?],
            1024,
        ),
        ("literal_small", small, 256),
        ("literal_refused", refused, 256),
    ];
    speed::setup(
        "speed_literal",
        runs,
        &[
            ("cases", cases.len().to_string()),
            ("max_text", MAX_TEXT.to_string()),
        ],
    );

    let mut out = Vec::new();
    for run in 0..runs {
        for (name, paths, reads) in &cases {
            let mut results = Vec::with_capacity(paths.len() * reads);
            let start = Instant::now();
            for path in paths {
                for _ in 0..*reads {
                    results.push(black_box(literal::read(black_box(path))));
                }
            }
            speed::line(name, run, start.elapsed().as_nanos());
            for (i, result) in results.iter().enumerate() {
                let value = match result {
                    Ok(lit) => format!(
                        "{} {} {:?}",
                        hex(&digest(&literal::encode(lit.record())?)),
                        lit.placed(),
                        lit.total()
                    ),
                    Err(e) => format!("ERR {e}"),
                };
                out.extend_from_slice(format!("{name} {run} {i} {value}\n").as_bytes());
            }
        }
    }
    speed::check_digest(&speed::sha256(&out));
    Ok(())
}
