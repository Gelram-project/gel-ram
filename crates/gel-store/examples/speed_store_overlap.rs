#![forbid(unsafe_code)]

//! Timing harness for streaming store reads whose payload spans several
//! read blocks (where reading a block can overlap the digest of the
//! previous one), with one-block and CRC-only control cases.
//!
//! Usage: `speed_store_overlap [RUNS] [DIR]`
//!
//! - `RUNS` (default 9, accepted 1..=999): repetitions of every case.
//! - `DIR` (default: the OS temporary directory): where the store files live.
//!
//! Cases (one line `SPEED name=<case> run=<i> ns=<n>` per case and run), in
//! this order within a run:
//!
//! - `verify_1mib`, `verify_8mib`, `open_8mib`: `verify_file` and
//!   `open_verified_with_limits` on v2 files of 8,192 and 65,536 records
//!   (one read block or less);
//! - `verify_64mib`: `verify_file` on a v2 file of 524,288 records;
//! - `verify_v1_64mib`: `verify_file` on a legacy v1 file of 524,288 records;
//! - `verify_256mib`, `open_256mib`: the same calls on the 268,435,520-byte v2
//!   file of 2,097,152 records;
//! - `verify_corrupt_256mib`: `verify_file` on a copy of that file whose last
//!   payload byte is changed (the result is `Err(CorruptStore)`);
//! - `crc_256mib`: `gel_core::crc64_ecma` on the 256 MiB payload bytes.
//!
//! The first line is `SPEED_SETUP example=speed_store_overlap runs=<R> ...`,
//! the last `SPEED_CHECK sha256=<hex>`: the SHA-256 of every case name and
//! result of every run (verification reports and errors, loaded headers and
//! formats, CRC values) and of every file's contents, so two builds can be
//! compared for identical output. A loaded store whose records differ from
//! the written ones, or a verification result other than the expected one,
//! ends the run with `SPEED_FAIL`. Uses only public API. Output grammar:
//! `docs/SPEED.md`.

use gel_core::{checksum64, crc64_ecma, GelError, Generation, GEL_MAGIC_V1, ORB_BITS, ORB_BYTES};
use gel_orb::Orb1024;
use gel_store::{verify_file, OpenLimits, RamStore, DEFAULT_MAX_RECORDS, HEADER_BYTES};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const SEED: u64 = 0x5eed_2026_1024_0002;
const TINY_RECORDS: usize = 8192;
const BLOCK_RECORDS: usize = 65_536;
const MID_RECORDS: usize = 524_288;

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
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

/// The store files of one session; removed again on every exit path.
struct Files {
    paths: Vec<PathBuf>,
}

impl Files {
    fn path(&mut self, dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(format!(
            "speed-store-overlap-{name}-{}.gel",
            std::process::id()
        ));
        self.paths.push(path.clone());
        path
    }

    fn remove(&self) {
        for path in &self.paths {
            let _ = fs::remove_file(path);
        }
    }

    fn fail(&self, reason: &str) -> ! {
        self.remove();
        speed::fail(reason)
    }
}

/// One run's timed calls: each prints its `SPEED` line and records its
/// result in the check digest.
struct Session<'a> {
    files: &'a Files,
    hash: &'a mut speed::Sha256,
    run: usize,
}

impl Session<'_> {
    /// `verify_file`; `corrupt` expects `Err(CorruptStore)`, otherwise `Ok`.
    fn verify(&mut self, name: &str, path: &Path, corrupt: bool) {
        let start = Instant::now();
        let result = verify_file(path, OpenLimits::UNLIMITED);
        let ns = start.elapsed().as_nanos();
        speed::line(name, self.run, ns);
        let expected = if corrupt {
            matches!(result, Err(GelError::CorruptStore))
        } else {
            result.is_ok()
        };
        if !expected {
            self.files.fail(name);
        }
        record(self.hash, name, &format!("{result:?}"));
    }

    /// `open_verified_with_limits`, whose records must equal `expected`.
    fn open(&mut self, name: &str, path: &Path, expected: &[Orb1024]) {
        let start = Instant::now();
        let result = RamStore::open_verified_with_limits(path, OpenLimits::UNLIMITED);
        let ns = start.elapsed().as_nanos();
        speed::line(name, self.run, ns);
        let Ok(loaded) = result else {
            self.files.fail(name);
        };
        if loaded.orbs() != expected {
            self.files.fail(name);
        }
        record(
            self.hash,
            name,
            &format!("{:?}|{:?}", loaded.header(), loaded.source_format()),
        );
    }
}

fn write_v2(files: &Files, path: &Path, records: &[Orb1024], generation: u64, reason: &str) {
    let store = RamStore::from_orbs(records.to_vec(), Generation(generation));
    if store.write_atomic(path).is_err() {
        files.fail(reason);
    }
}

fn main() {
    let args = speed::args("speed_store_overlap", "[RUNS] [DIR]", 2);
    let runs = speed::runs(&args);
    let dir: PathBuf = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);

    speed::self_check();

    let big_records = DEFAULT_MAX_RECORDS as usize;
    let mut state = SEED;
    let big = orbs(big_records, &mut state);
    let block = orbs(BLOCK_RECORDS, &mut state);
    let tiny = orbs(TINY_RECORDS, &mut state);
    let mid = orbs(MID_RECORDS, &mut state);
    let legacy = orbs(MID_RECORDS, &mut state);

    let mut files = Files { paths: Vec::new() };
    let big_path = files.path(&dir, "big");
    let corrupt_path = files.path(&dir, "corrupt");
    let block_path = files.path(&dir, "block");
    let tiny_path = files.path(&dir, "tiny");
    let mid_path = files.path(&dir, "mid");
    let v1_path = files.path(&dir, "v1");

    write_v2(&files, &big_path, &big, 31, "write_big");
    write_v2(&files, &block_path, &block, 32, "write_block");
    write_v2(&files, &tiny_path, &tiny, 33, "write_tiny");
    write_v2(&files, &mid_path, &mid, 34, "write_mid");
    drop(mid);
    let v1_bytes = v1_file_bytes(&legacy, 35);
    drop(legacy);
    if fs::write(&v1_path, &v1_bytes).is_err() {
        files.fail("write_v1");
    }
    drop(v1_bytes);
    let mut big_bytes = match fs::read(&big_path) {
        Ok(bytes) => bytes,
        Err(_) => files.fail("read_big"),
    };
    if let Some(last) = big_bytes.last_mut() {
        *last ^= 0x01;
    }
    if fs::write(&corrupt_path, &big_bytes).is_err() {
        files.fail("write_corrupt");
    }
    if let Some(last) = big_bytes.last_mut() {
        *last ^= 0x01;
    }

    speed::setup(
        "speed_store_overlap",
        runs,
        &[
            ("records", big_records.to_string()),
            ("block_records", BLOCK_RECORDS.to_string()),
            ("tiny_records", TINY_RECORDS.to_string()),
            ("mid_records", MID_RECORDS.to_string()),
        ],
    );

    let mut hash = speed::Sha256::new();
    for run in 0..runs {
        let mut session = Session {
            files: &files,
            hash: &mut hash,
            run,
        };
        session.verify("verify_1mib", &tiny_path, false);
        session.verify("verify_8mib", &block_path, false);
        session.open("open_8mib", &block_path, &block);
        session.verify("verify_64mib", &mid_path, false);
        session.verify("verify_v1_64mib", &v1_path, false);
        session.verify("verify_256mib", &big_path, false);
        session.open("open_256mib", &big_path, &big);
        session.verify("verify_corrupt_256mib", &corrupt_path, true);

        let start = Instant::now();
        let crc = crc64_ecma(&big_bytes[HEADER_BYTES..]);
        let ns = start.elapsed().as_nanos();
        speed::line("crc_256mib", run, ns);
        record(&mut hash, "crc_256mib", &format!("{crc:016x}"));
    }

    hash.update(&big_bytes);
    drop(big_bytes);
    for (name, path) in [
        ("read_corrupt", &corrupt_path),
        ("read_block", &block_path),
        ("read_tiny", &tiny_path),
        ("read_mid", &mid_path),
        ("read_v1", &v1_path),
    ] {
        match fs::read(path) {
            Ok(bytes) => hash.update(&bytes),
            Err(_) => files.fail(name),
        }
    }
    files.remove();
    speed::check(hash);
}
