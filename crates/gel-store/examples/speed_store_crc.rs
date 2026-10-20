#![forbid(unsafe_code)]

//! Timing harness for the store payload CRC: building a store, streaming
//! verification and verified load, with unchanged control cases.
//!
//! Usage: `speed_store_crc [RUNS] [DIR]`
//!
//! - `RUNS` (default 9, accepted 1..=999): repetitions of every case.
//! - `DIR` (default: the OS temporary directory): where the store files live.
//!
//! Cases (one line `SPEED name=<case> run=<i> ns=<n>` per case and run):
//!
//! - `build_256mib`: `RamStore::from_orbs` on 2,097,152 records (the vector
//!   is cloned outside the timed region);
//! - `verify_256mib`, `open_256mib`: `verify_file` and
//!   `open_verified_with_limits` on the 268,435,520-byte v2 file;
//! - `verify_v1_64mib`: `verify_file` on a legacy v1 file of 524,288 records;
//! - `build_1mib`, `verify_1mib`, `open_1mib`: the same calls on 8,192
//!   records;
//! - `crc_256mib`: `gel_core::crc64_ecma` on the 256 MiB payload bytes.
//!
//! The first line is `SPEED_SETUP example=speed_store_crc runs=<R> ...`, the
//! last `SPEED_CHECK sha256=<hex>`: the SHA-256 of every case name and result
//! of every run (headers, verification reports, load formats, CRC values)
//! and of both file contents, so two builds can be compared for identical
//! output. Uses only public API. Output grammar: `docs/SPEED.md`.

use gel_core::{checksum64, crc64_ecma, Generation, GEL_MAGIC_V1, ORB_BITS, ORB_BYTES};
use gel_orb::Orb1024;
use gel_store::{verify_file, OpenLimits, RamStore, DEFAULT_MAX_RECORDS, HEADER_BYTES};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const SEED: u64 = 0x5eed_2026_1020_0003;
const SMALL_RECORDS: usize = 8192;
const V1_RECORDS: usize = 524_288;

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn fail(reason: &str) -> ! {
    speed::fail(reason)
}

fn orbs(count: usize, state: &mut u64) -> Vec<Orb1024> {
    (0..count)
        .map(|_| {
            let mut words = [0u64; 16];
            for word in &mut words {
                *word = next(state);
            }
            Orb1024::from_words(words)
        })
        .collect()
}

fn v1_file_bytes(records: &[Orb1024], generation: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(records.len() * ORB_BYTES);
    for orb in records {
        payload.extend_from_slice(&orb.to_le_bytes());
    }
    let mut bytes = vec![0u8; HEADER_BYTES];
    bytes[0..8].copy_from_slice(&GEL_MAGIC_V1);
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&(ORB_BITS as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&(ORB_BYTES as u32).to_le_bytes());
    bytes[24..32].copy_from_slice(&(records.len() as u64).to_le_bytes());
    bytes[32..40].copy_from_slice(&generation.to_le_bytes());
    bytes[40..48].copy_from_slice(&checksum64(&payload).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

fn record(hash: &mut speed::Sha256, name: &str, result: &str) {
    hash.update(name.as_bytes());
    hash.update(b"=");
    hash.update(result.as_bytes());
    hash.update(b"\n");
}

fn main() {
    let args = speed::args("speed_store_crc", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let dir: PathBuf = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);

    speed::self_check();

    let big_records = DEFAULT_MAX_RECORDS as usize;
    let mut state = SEED;
    let big = orbs(big_records, &mut state);
    let small = orbs(SMALL_RECORDS, &mut state);
    let legacy = orbs(V1_RECORDS, &mut state);

    let id = std::process::id();
    let big_path = dir.join(format!("speed-store-crc-big-{id}.gel"));
    let small_path = dir.join(format!("speed-store-crc-small-{id}.gel"));
    let v1_path = dir.join(format!("speed-store-crc-v1-{id}.gel"));
    let cleanup = |paths: &[&Path]| {
        for path in paths {
            let _ = fs::remove_file(path);
        }
    };
    let paths = [big_path.as_path(), small_path.as_path(), v1_path.as_path()];

    let big_store = RamStore::from_orbs(big.clone(), Generation(11));
    if big_store.write_atomic(&big_path).is_err() {
        cleanup(&paths);
        fail("write_big");
    }
    drop(big_store);
    let small_store = RamStore::from_orbs(small.clone(), Generation(12));
    if small_store.write_atomic(&small_path).is_err() {
        cleanup(&paths);
        fail("write_small");
    }
    drop(small_store);
    let v1_bytes = v1_file_bytes(&legacy, 13);
    if fs::write(&v1_path, &v1_bytes).is_err() {
        cleanup(&paths);
        fail("write_v1");
    }
    drop(legacy);
    let big_bytes = match fs::read(&big_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            cleanup(&paths);
            fail("read_big");
        }
    };
    let limits = OpenLimits::UNLIMITED;

    speed::setup(
        "speed_store_crc",
        runs,
        &[
            ("records", big_records.to_string()),
            ("small_records", SMALL_RECORDS.to_string()),
            ("v1_records", V1_RECORDS.to_string()),
        ],
    );

    let mut hash = speed::Sha256::new();
    for run in 0..runs {
        // Building: the clone is outside the timed region.
        for (name, records, generation) in
            [("build_256mib", &big, 21u64), ("build_1mib", &small, 22u64)]
        {
            let input = records.clone();
            let start = Instant::now();
            let store = RamStore::from_orbs(input, Generation(generation));
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            record(&mut hash, name, &format!("{:?}", store.header()));
            drop(store);
        }

        for (name, path) in [
            ("verify_256mib", big_path.as_path()),
            ("verify_1mib", small_path.as_path()),
            ("verify_v1_64mib", v1_path.as_path()),
        ] {
            let start = Instant::now();
            let result = verify_file(path, limits);
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            if result.is_err() {
                cleanup(&paths);
                fail(name);
            }
            record(&mut hash, name, &format!("{result:?}"));
        }

        for (name, path, expected) in [
            ("open_256mib", big_path.as_path(), &big),
            ("open_1mib", small_path.as_path(), &small),
        ] {
            let start = Instant::now();
            let result = RamStore::open_verified_with_limits(path, limits);
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            let Ok(loaded) = result else {
                cleanup(&paths);
                fail(name);
            };
            if loaded.orbs() != expected.as_slice() {
                cleanup(&paths);
                fail(name);
            }
            record(
                &mut hash,
                name,
                &format!("{:?}|{:?}", loaded.header(), loaded.source_format()),
            );
            drop(loaded);
        }

        let start = Instant::now();
        let crc = crc64_ecma(&big_bytes[HEADER_BYTES..]);
        let ns = start.elapsed().as_nanos();
        speed::line("crc_256mib", run, ns);
        record(&mut hash, "crc_256mib", &format!("{crc:016x}"));
    }

    hash.update(&big_bytes);
    drop(big_bytes);
    match fs::read(&small_path) {
        Ok(bytes) => hash.update(&bytes),
        Err(_) => {
            cleanup(&paths);
            fail("read_small");
        }
    }
    hash.update(&v1_bytes);
    cleanup(&paths);
    speed::check(hash);
}
