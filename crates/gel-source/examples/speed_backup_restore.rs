//! Repeatable wall-time of `backup::create`, `backup::inspect` and
//! `backup::restore` on fixed-seed collection snapshots. Only public API that
//! older trees also have is used, so the same file builds and runs on them.
//!
//! Usage: `speed_backup_restore [RUNS] [WORK_DIR]`. RUNS is 1..=999 (default
//! 9); WORK_DIR (default: the system temporary directory) receives one fresh
//! subdirectory, removed at the end. The paths are single-threaded, so there
//! is no thread argument.
//!
//! Inputs: a 64 MiB snapshot (64 documents of 1 MiB ASCII text) and a 1 MiB
//! snapshot (16 documents of 64 KiB). One complete backup of each, plus one
//! withdrawn backup of the 64 MiB snapshot, are made before timing.
//!
//! Cases, each timed once per run:
//! - `backup_create_64mib`: `create` into a new directory.
//! - `backup_inspect_64mib`: `inspect` of the complete backup.
//! - `backup_restore_64mib`, `backup_restore_1mib`: `restore` of the complete
//!   backup to a new path.
//! - `backup_restore_withdrawn_64mib`: `restore` of the withdrawn backup
//!   (refused as WITHDRAWN after the inspection).
//! - `backup_restore_target_exists_64mib`: `restore` onto an existing path
//!   (refused as TARGET_EXISTS after the inspection).
//!
//! Output (stdout): one `SPEED_SETUP example=speed_backup_restore runs=<R>
//! ...` line, one `SPEED name=<case> run=<i> ns=<n>` line per case and run
//! (`i` from 0), and one last `SPEED_CHECK sha256=<hex>` line: the SHA-256,
//! computed by `tools/speed_support.rs`, of every case name, every returned
//! value and the bytes of every restored file. An unexpected result prints
//! `SPEED_FAIL reason=<slug>` on stderr, no `SPEED_CHECK`, and exits 1; an
//! invalid argument exits 2. Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::{backup, collection::Collection, Hash};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

#[path = "../../../tools/speed_support.rs"]
mod speed;

// ---- Inputs ----

const BIG_DOCS: usize = 64;
const BIG_DOC_BYTES: usize = 1 << 20;
const SMALL_DOCS: usize = 16;
const SMALL_DOC_BYTES: usize = 64 << 10;

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "backup", "restore", "exact", "pin", "snapshot", "root", "line", "copy",
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

fn snapshot(path: &Path, seed: u64, docs: usize, doc_bytes: usize) -> Result<Hash, String> {
    let mut c = Collection::new();
    for d in 0..docs {
        c.add(&format!("doc-{d}.txt"), &text(seed + d as u64, doc_bytes))?;
    }
    c.save_new(path)
}

// ---- Run ----

fn sha_hex(bytes: &[u8]) -> String {
    speed::hex(&speed::sha256(bytes))
}

fn line(name: &str, run: usize, start: Instant) {
    speed::line(name, run, start.elapsed().as_nanos());
}

fn record(check: &mut speed::Sha256, name: &str, value: &str) {
    check.update(name.as_bytes());
    check.update(b"\n");
    check.update(value.as_bytes());
    check.update(b"\n");
}

fn main() -> ExitCode {
    let args = speed::args("speed_backup_restore", "[RUNS] [WORK_DIR]", 2);
    let runs = speed::runs(&args);
    let base = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    speed::self_check();
    let work = base.join(format!("speed-backup-restore-{}", std::process::id()));
    if fs::create_dir_all(&work).is_err() {
        speed::fail("work_dir");
    }
    let result = measure(runs, &work);
    let _ = fs::remove_dir_all(&work);
    match result {
        Ok(check) => {
            speed::check(check);
            ExitCode::SUCCESS
        }
        Err(reason) => speed::fail(reason),
    }
}

/// The digest of every result, or the case that ended unexpectedly.
fn measure(runs: usize, work: &Path) -> Result<speed::Sha256, &'static str> {
    let big = work.join("big.gelset");
    let small = work.join("small.gelset");
    let (Ok(big_pin), Ok(small_pin)) = (
        snapshot(&big, 0x5eed_0064, BIG_DOCS, BIG_DOC_BYTES),
        snapshot(&small, 0x5eed_0001, SMALL_DOCS, SMALL_DOC_BYTES),
    ) else {
        return Err("snapshot");
    };
    let (Ok(big_bytes), Ok(small_bytes)) = (fs::read(&big), fs::read(&small)) else {
        return Err("snapshot_read");
    };
    let big_sha = sha_hex(&big_bytes);
    let small_sha = sha_hex(&small_bytes);
    let big_dir = work.join("backup-big");
    let small_dir = work.join("backup-small");
    let withdrawn_dir = work.join("backup-withdrawn");
    let occupied = work.join("occupied.gelset");
    let ready = backup::create(&big, big_pin, &big_dir).is_ok()
        && backup::create(&small, small_pin, &small_dir).is_ok()
        && backup::create(&big, big_pin, &withdrawn_dir).is_ok()
        && backup::withdraw(&withdrawn_dir, "superseded").is_ok()
        && fs::write(&occupied, b"occupied").is_ok();
    if !ready {
        return Err("backup_setup");
    }
    speed::setup(
        "speed_backup_restore",
        runs,
        &[
            ("snapshot_bytes", big_bytes.len().to_string()),
            ("small_snapshot_bytes", small_bytes.len().to_string()),
        ],
    );
    drop((big_bytes, small_bytes));

    let mut check = speed::Sha256::new();
    for run in 0..runs {
        let name = "backup_create_64mib";
        let dir = work.join(format!("create-{run}"));
        let start = Instant::now();
        let created = backup::create(&big, big_pin, &dir);
        line(name, run, start);
        if created.is_err() {
            return Err(name);
        }
        record(&mut check, name, &format!("{created:?}"));
        let _ = fs::remove_dir_all(&dir);

        let name = "backup_inspect_64mib";
        let start = Instant::now();
        let inspected = backup::inspect(&big_dir, big_pin);
        line(name, run, start);
        record(&mut check, name, &format!("{inspected:?}"));

        for (name, dir, pin, sha) in [
            ("backup_restore_64mib", &big_dir, big_pin, &big_sha),
            ("backup_restore_1mib", &small_dir, small_pin, &small_sha),
        ] {
            let target = work.join(format!("restored-{name}-{run}.gelset"));
            let start = Instant::now();
            let restored = backup::restore(dir, pin, &target);
            line(name, run, start);
            let written = fs::read(&target).map(|b| sha_hex(&b)).unwrap_or_default();
            if restored.is_err() || written != *sha {
                return Err(name);
            }
            record(&mut check, name, &format!("{restored:?} {written}"));
            let _ = fs::remove_file(&target);
        }

        let name = "backup_restore_withdrawn_64mib";
        let target = work.join(format!("withdrawn-{run}.gelset"));
        let start = Instant::now();
        let refused = backup::restore(&withdrawn_dir, big_pin, &target);
        line(name, run, start);
        if refused.is_ok() || target.exists() {
            return Err(name);
        }
        record(&mut check, name, &format!("{refused:?}"));

        let name = "backup_restore_target_exists_64mib";
        let start = Instant::now();
        let refused = backup::restore(&big_dir, big_pin, &occupied);
        line(name, run, start);
        if refused.is_ok() {
            return Err(name);
        }
        record(&mut check, name, &format!("{refused:?}"));
    }
    Ok(check)
}
