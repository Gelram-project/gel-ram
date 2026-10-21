#![forbid(unsafe_code)]

//! Timing harness for store write, streaming verification and verified load.
//!
//! Usage: `speed_store_io [RUNS] [DIR] [RECORDS]`
//!
//! - `RUNS` (1..=999, default 9): repetitions of every case.
//! - `DIR` (default: the OS temporary directory): where the store file lives.
//! - `RECORDS` (default 2,097,152): records in the store. The default is the
//!   largest store the default open budget admits: 256 MiB of payload, a
//!   268,435,520-byte file.
//!
//! Every run writes the same deterministic store with `write_atomic`, checks
//! it with `verify_file` and loads it with `open_verified_with_limits`. Each
//! case prints `SPEED name=<case> run=<i> ns=<n>`. The last line,
//! `SPEED_CHECK sha256=<hex>`, is the SHA-256 of the file bytes, the
//! verification report, the loaded header and format, and the loaded records
//! of the last run, so two builds can be compared for identical output. A
//! header or record mismatch ends the run with `SPEED_FAIL`. Uses only public
//! API. Output grammar: `docs/SPEED.md`.

use gel_core::Generation;
use gel_orb::Orb1024;
use gel_store::{verify_file, OpenLimits, RamStore, DEFAULT_MAX_RECORDS};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const SEED: u64 = 0x5eed_2026_1019_0001;

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn main() {
    let args = speed::args("speed_store_io", "[RUNS] [DIR] [RECORDS]", 3);
    let runs = speed::runs(&args);
    let dir: PathBuf = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let records = speed::number(
        &args,
        2,
        "RECORDS",
        DEFAULT_MAX_RECORDS as usize,
        1..=usize::MAX,
    );

    let mut state = SEED;
    let orbs: Vec<Orb1024> = (0..records)
        .map(|_| {
            let mut words = [0u64; 16];
            for word in &mut words {
                *word = next(&mut state);
            }
            Orb1024::from_words(words)
        })
        .collect();
    let store = RamStore::from_orbs(orbs, Generation(7));
    let limits = OpenLimits {
        max_records: u64::MAX,
        max_file_bytes: u64::MAX,
    };
    let path = dir.join(format!("speed-store-io-{}.gel", std::process::id()));
    let mib = records * 128 / (1024 * 1024);
    speed::setup(
        "speed_store_io",
        runs,
        &[
            ("records", records.to_string()),
            ("payload_bytes", (records * 128).to_string()),
        ],
    );

    let mut report = String::new();
    let mut last: Option<RamStore> = None;
    for run in 0..runs {
        drop(last.take());
        let start = Instant::now();
        store.write_atomic(&path).expect("write_atomic");
        let ns = start.elapsed().as_nanos();
        speed::line(&format!("write_{mib}mib"), run, ns);

        let start = Instant::now();
        let verification = verify_file(&path, limits).expect("verify_file");
        let ns = start.elapsed().as_nanos();
        speed::line(&format!("verify_{mib}mib"), run, ns);

        let start = Instant::now();
        let loaded = RamStore::open_verified_with_limits(&path, limits).expect("open_verified");
        let ns = start.elapsed().as_nanos();
        speed::line(&format!("open_{mib}mib"), run, ns);

        if verification.header != store.header() || loaded.header() != store.header() {
            let _ = fs::remove_file(&path);
            speed::fail("header_differs");
        }
        if loaded.orbs() != store.orbs() {
            let _ = fs::remove_file(&path);
            speed::fail("records_differ");
        }
        report = format!(
            "{verification:?}|{:?}|{:?}",
            loaded.header(),
            loaded.source_format()
        );
        last = Some(loaded);
    }

    let mut hash = speed::Sha256::new();
    if let Some(loaded) = last {
        let bytes = fs::read(&path).expect("read store file");
        hash.update(&bytes);
        drop(bytes);
        hash.update(report.as_bytes());
        for orb in loaded.orbs() {
            hash.update(&orb.to_le_bytes());
        }
        let _ = fs::remove_file(&path);
    }
    speed::check(hash);
}
