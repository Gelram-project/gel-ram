//! Identity check for `backup::restore`, which now publishes the snapshot
//! bytes that `inspect` already read and verified, against the earlier
//! restore, which read, hashed and parsed the snapshot a second time.
//!
//! Usage: `cargo run --release -p gel-source --example identity_backup_restore [DIR]`.
//! DIR (default: the system temporary directory) receives one fresh
//! subdirectory, removed at the end. Every trial builds one backup directory
//! from fixed seeds, damages it in one way or leaves it intact, and runs both
//! restores on it, each into its own empty output directory. A trial is equal
//! when both return the same value (including the error text) and leave the
//! same entries with the same bytes in their output directories, and neither
//! changes the backup directory.
//!
//! The earlier code is `src/backup_restore_reference.rs`, verbatim. Here its
//! two crate-internal calls are served by public equivalents defined in this
//! file: the no-replace publisher through `write_file_new` (the same
//! publisher; every snapshot here is far below its 64 MiB limit) and the
//! snapshot shape through `Collection::from_bytes`, which accepts and refuses
//! the same pinned bytes with the same error. The crate's unit tests compile
//! the same file against the crate-internal functions themselves.
//!
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when a case
//! ran no trial or no case ran). Exit code 0 on PASS, 1 on FAIL (output
//! grammar: `docs/SPEED.md`). stderr lists the
//! distinct results per case (`IDENTITY_RESULT`) and every difference
//! (`IDENTITY_DIFF`). The integration
//! test `tests/identity_backup_restore.rs` runs the same comparison at a
//! smaller scale.
#![forbid(unsafe_code)]
use gel_source::backup::{self, Inspection, RestoreError, State, MANIFEST, SNAPSHOT, WITHDRAWN};
use gel_source::collection::{
    Collection as SourceCollection, MAX_DOCUMENTS, MAX_TITLE, MAX_TOTAL_TEXT,
};
use gel_source::{digest, hex, read_regular, write_file_new, BundleError, Hash};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

// ---- What the reference file takes from its parent module ----

/// The bound `backup` uses for a snapshot file.
const MAX_SNAPSHOT: usize = MAX_TOTAL_TEXT + MAX_DOCUMENTS * (20 + MAX_TITLE) + 32;
/// The bound `backup` uses for a manifest or marker file.
const MAX_MARKER: usize = 4096;

/// The no-replace publisher for bytes whose pin the caller holds; `pin`
/// equals the digest of `bytes` wherever the reference calls it.
fn write_bytes_new_pinned(path: &Path, bytes: &[u8], pin: Hash) -> Result<Hash, BundleError> {
    let _ = pin;
    write_file_new(path, bytes)
}

/// Stands in for the crate-internal shape check of a pinned snapshot.
struct Collection;
impl Collection {
    fn verified_shape(bytes: &[u8]) -> Result<(usize, u64), String> {
        let parsed = SourceCollection::from_bytes(bytes, digest(bytes))?;
        Ok((parsed.documents().count(), parsed.revision()))
    }
}

#[allow(dead_code)]
#[path = "../src/backup_restore_reference.rs"]
mod reference;

// ---- Tallies ----

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// Comparisons of one case, with the distinct results it saw.
pub struct Case {
    pub name: &'static str,
    pub cases: u64,
    pub equal: u64,
    /// The distinct results the current restore returned in this case (every
    /// success as `Ok(..)`), for the diagnostics on stderr.
    pub results: std::collections::BTreeSet<String>,
}

impl Case {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            cases: 0,
            equal: 0,
            results: Default::default(),
        }
    }
    fn check(&mut self, same: bool) {
        self.cases += 1;
        self.equal += u64::from(same);
    }
}

// ---- Seeded inputs ----

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next() as u8).collect()
    }
}

const WORDS: [&str; 10] = [
    "snapshot",
    "pin",
    "zażółć",
    "gęślą",
    "jaźń",
    "RAM",
    "line",
    "quote",
    "żółw",
    "exact",
];

fn text(rng: &mut Rng) -> String {
    let len = 1 + rng.below(1500) as usize;
    let mut s = String::new();
    while s.len() < len {
        s.push_str(WORDS[rng.below(WORDS.len() as u64) as usize]);
        s.push(if rng.below(9) == 0 { '\n' } else { ' ' });
    }
    s
}

/// A collection with 0..=6 documents and some replaced or removed ones, so
/// the revision and the next identifier vary too.
fn collection(rng: &mut Rng) -> SourceCollection {
    let mut c = SourceCollection::new();
    let mut ids = Vec::new();
    for d in 0..rng.below(7) {
        let title = format!("{}-{d}.txt", WORDS[rng.below(WORDS.len() as u64) as usize]);
        if let Ok(id) = c.add(&title, &text(rng)) {
            ids.push(id);
        }
    }
    for &id in &ids {
        match rng.below(6) {
            0 => {
                let _ = c.remove(id);
            }
            1 => {
                let _ = c.replace(id, &text(rng));
            }
            _ => {}
        }
    }
    c
}

/// Writes a seeded snapshot and a complete backup of it; returns the pin.
fn complete_backup(rng: &mut Rng, work: &Path, dir: &Path) -> Hash {
    let c = collection(rng);
    let snap = work.join(format!("{}.gelset", unique()));
    let pin = c.save_new(&snap).expect("save the seeded snapshot");
    backup::create(&snap, pin, dir).expect("create the seeded backup");
    fs::remove_file(&snap).expect("remove the seeded snapshot");
    pin
}

fn unique() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!("n{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Replaces a file by new bytes (remove first, so a hard link stays intact).
fn rewrite(path: &Path, bytes: &[u8]) {
    let _ = fs::remove_file(path);
    fs::write(path, bytes).expect("rewrite a backup file");
}

fn set_len(path: &Path, len: u64) {
    let _ = fs::remove_file(path);
    fs::File::create(path)
        .and_then(|f| f.set_len(len))
        .expect("size a sparse file");
}

#[cfg(unix)]
fn symlink(from: &Path, to: &Path) {
    std::os::unix::fs::symlink(from, to).expect("create a symbolic link");
}
#[cfg(not(unix))]
fn symlink(_from: &Path, to: &Path) {
    fs::write(to, b"not a link on this platform").expect("write a stand-in file");
}

// ---- One trial ----

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    File(usize, Hash),
    /// A file above `LISTED_BYTES` (only the oversized sparse snapshots):
    /// its length only.
    Large(u64),
    Dir,
    Link(PathBuf),
    Other,
}

const LISTED_BYTES: u64 = 1 << 20;

/// The entries of one directory (not recursive), sorted by name.
fn listing(dir: &Path) -> Option<Vec<(String, Entry)>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).ok()?;
        let kind = if meta.file_type().is_symlink() {
            Entry::Link(fs::read_link(&path).ok()?)
        } else if meta.is_dir() {
            Entry::Dir
        } else if meta.is_file() && meta.len() > LISTED_BYTES {
            Entry::Large(meta.len())
        } else if meta.is_file() {
            let bytes = fs::read(&path).ok()?;
            Entry::File(bytes.len(), digest(&bytes))
        } else {
            Entry::Other
        };
        out.push((entry.file_name().to_string_lossy().into_owned(), kind));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Some(out)
}

/// Where a trial restores to, relative to each side's own output directory.
#[derive(Clone, Copy)]
enum Target {
    New,
    File,
    Dir,
    DanglingLink,
    MissingParent,
    EmptyPath,
    ReadOnlyParent,
}

type Restore = fn(&Path, Hash, &Path) -> Result<usize, RestoreError>;

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    result: String,
    output: Option<Vec<(String, Entry)>>,
    backup_unchanged: bool,
}

fn side(dir: &Path, pin: Hash, out: &Path, target: Target, run: Restore) -> Outcome {
    fs::create_dir(out).expect("create an output directory");
    let name = out.join("restored.gelset");
    let path = match target {
        Target::New | Target::ReadOnlyParent => name,
        Target::File => {
            fs::write(&name, b"keep me").expect("occupy the target");
            name
        }
        Target::Dir => {
            fs::create_dir(&name).expect("occupy the target");
            name
        }
        Target::DanglingLink => {
            // Relative, so both sides hold the same link text.
            symlink(Path::new("nowhere"), &name);
            name
        }
        Target::MissingParent => out.join("absent").join("restored.gelset"),
        Target::EmptyPath => PathBuf::new(),
    };
    let read_only = matches!(target, Target::ReadOnlyParent);
    set_read_only(out, read_only);
    let before = listing(dir);
    let result = format!("{:?}", run(dir, pin, &path));
    let after = listing(dir);
    set_read_only(out, false);
    Outcome {
        result,
        output: listing(out),
        backup_unchanged: before == after,
    }
}

#[cfg(unix)]
fn set_read_only(dir: &Path, read_only: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if read_only { 0o555 } else { 0o755 };
    fs::set_permissions(dir, fs::Permissions::from_mode(mode)).expect("set directory mode");
}
#[cfg(not(unix))]
fn set_read_only(_dir: &Path, _read_only: bool) {}

/// Runs both restores on `dir` and compares what they did.
fn trial(tally: &mut Case, work: &Path, dir: &Path, pin: Hash, target: Target) {
    let k = unique();
    let new = side(
        dir,
        pin,
        &work.join(format!("out-new-{k}")),
        target,
        backup::restore,
    );
    let old = side(
        dir,
        pin,
        &work.join(format!("out-old-{k}")),
        target,
        reference::restore,
    );
    let same = new == old && new.backup_unchanged;
    if !same {
        eprintln!(
            "IDENTITY_DIFF case={} new={:?} old={:?}",
            tally.name, new, old
        );
    }
    let kind = if new.result.starts_with("Ok(") {
        "Ok(..)".to_string()
    } else {
        new.result
    };
    tally.results.insert(kind);
    tally.check(same);
}

// ---- Cases ----

fn fresh(work: &Path) -> PathBuf {
    work.join(format!("backup-{}", unique()))
}

fn complete_seeded(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("complete_seeded");
    for seed in 0..trials {
        let mut rng = Rng(0x1000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        for n in 0..rng.below(3) {
            fs::write(dir.join(format!(".gel-source-9-{n}.tmp")), rng.bytes(5)).unwrap();
        }
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn withdrawn(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("withdrawn");
    for seed in 0..trials {
        let mut rng = Rng(0x2000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        backup::withdraw(&dir, "superseded").unwrap();
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn incomplete(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("incomplete");
    for seed in 0..trials {
        let mut rng = Rng(0x3000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        fs::remove_file(dir.join(MANIFEST)).unwrap();
        match seed % 4 {
            0 => fs::remove_file(dir.join(SNAPSHOT)).unwrap(),
            1 => {
                fs::remove_file(dir.join(SNAPSHOT)).unwrap();
                fs::write(dir.join(".gel-source-1-0.tmp"), b"partial").unwrap();
            }
            2 => {}
            _ => fs::write(dir.join(".gel-source-1-1.tmp"), b"GEL_BACKUP=1\n").unwrap(),
        }
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn untrusted(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("untrusted");
    for seed in 0..trials {
        let mut rng = Rng(0x4000 + seed);
        let dir = fresh(work);
        let mut pin = complete_backup(&mut rng, work, &dir);
        let bit = rng.below(256) as usize;
        pin[bit / 8] ^= 1 << (bit % 8);
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn snapshot_mutation(work: &Path, trials: u64, scale: Scale) -> Case {
    let mut t = Case::new("snapshot_mutation");
    for seed in 0..trials {
        let mut rng = Rng(0x5000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        let path = dir.join(SNAPSHOT);
        let mut bytes = fs::read(&path).unwrap();
        match seed % 9 {
            0..=2 => {
                let at = rng.below(bytes.len() as u64) as usize;
                bytes[at] ^= 1 << rng.below(8);
                rewrite(&path, &bytes);
            }
            3 => {
                bytes.truncate(rng.below(bytes.len() as u64) as usize);
                rewrite(&path, &bytes);
            }
            4 => {
                let len = 1 + rng.below(40) as usize;
                let extra = rng.bytes(len);
                bytes.extend_from_slice(&extra);
                rewrite(&path, &bytes);
            }
            5 => fs::remove_file(&path).unwrap(),
            6 => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            7 => {
                let copy = work.join(format!("{}.gelset", unique()));
                fs::rename(&path, &copy).unwrap();
                symlink(&copy, &path);
            }
            _ => set_len(&path, MAX_SNAPSHOT as u64 + 1),
        }
        trial(&mut t, work, &dir, pin, Target::New);
    }
    if scale == Scale::Full {
        // Exactly at the size bound: read in full, then refused by the pin.
        let mut rng = Rng(0x5fff);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        set_len(&dir.join(SNAPSHOT), MAX_SNAPSHOT as u64);
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn manifest_mutation(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("manifest_mutation");
    for seed in 0..trials {
        let mut rng = Rng(0x6000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        let path = dir.join(MANIFEST);
        let mut bytes = fs::read(&path).unwrap();
        match seed % 10 {
            0..=2 => {
                let at = rng.below(bytes.len() as u64) as usize;
                bytes[at] ^= 1 << rng.below(8);
                rewrite(&path, &bytes);
            }
            3 => {
                bytes.truncate(rng.below(bytes.len() as u64) as usize);
                rewrite(&path, &bytes);
            }
            4 => {
                bytes.extend_from_slice(b"extra=1\n");
                rewrite(&path, &bytes);
            }
            5 => {
                bytes[0] = 0xff;
                rewrite(&path, &bytes);
            }
            6 => set_len(&path, MAX_MARKER as u64 + 1),
            7 => {
                bytes.resize(MAX_MARKER, b'\n');
                rewrite(&path, &bytes);
            }
            8 => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            _ => {
                let upper = String::from_utf8(bytes).unwrap().to_uppercase();
                rewrite(&path, upper.as_bytes());
            }
        }
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

/// A manifest that names the right pin but a different length, document
/// count or revision.
fn manifest_disagree(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("manifest_disagree");
    for seed in 0..trials {
        let mut rng = Rng(0x7000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        let path = dir.join(MANIFEST);
        let text = fs::read_to_string(&path).unwrap();
        let key = ["bytes=", "documents=", "revision="][(seed % 3) as usize];
        let changed: Vec<String> = text
            .lines()
            .map(|line| match line.strip_prefix(key) {
                Some(n) => {
                    let n: u64 = n.parse().unwrap();
                    let m = if seed % 2 == 0 || n == 0 {
                        n + 1
                    } else {
                        n.saturating_sub(1)
                    };
                    format!("{key}{m}")
                }
                None => line.to_string(),
            })
            .collect();
        rewrite(&path, format!("{}\n", changed.join("\n")).as_bytes());
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

/// Bytes that are not a valid snapshot, pinned by a well-formed manifest.
fn pinned_unparseable(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("pinned_unparseable");
    for seed in 0..trials {
        let mut rng = Rng(0x8000 + seed);
        let mut bytes = collection(&mut rng).to_bytes();
        match seed % 4 {
            0 => {
                let len = rng.below(200) as usize;
                bytes = rng.bytes(len);
            }
            1 => bytes[0] ^= 1,
            2 => bytes.truncate(bytes.len().saturating_sub(1 + rng.below(8) as usize)),
            _ => bytes[16..24].copy_from_slice(&u64::MAX.to_le_bytes()),
        }
        let pin = digest(&bytes);
        let dir = fresh(work);
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join(SNAPSHOT), &bytes).unwrap();
        let manifest = format!(
            "GEL_BACKUP=1\nsnapshot={SNAPSHOT}\nbytes={}\nsha256={}\ndocuments=0\nrevision=0\n",
            bytes.len(),
            hex(&pin)
        );
        fs::write(dir.join(MANIFEST), manifest).unwrap();
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn unexpected_entry(work: &Path, trials: u64) -> Case {
    let mut t = Case::new("unexpected_entry");
    for seed in 0..trials {
        let mut rng = Rng(0x9000 + seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        match seed % 3 {
            0 => fs::write(dir.join("extra.txt"), b"x").unwrap(),
            1 => fs::create_dir(dir.join("sub")).unwrap(),
            _ => {
                fs::write(dir.join("a.tmp"), b"x").unwrap();
                fs::write(dir.join(".gel-source-x"), b"y").unwrap();
            }
        }
        trial(&mut t, work, &dir, pin, Target::New);
    }
    t
}

fn inspect_error(work: &Path) -> Case {
    let mut t = Case::new("inspect_error");
    let missing = fresh(work);
    trial(&mut t, work, &missing, [1; 32], Target::New);
    let file = fresh(work);
    fs::write(&file, b"not a directory").unwrap();
    trial(&mut t, work, &file, [1; 32], Target::New);
    t
}

/// Every backup kind against every occupied target: the order of refusals.
fn target_exists(work: &Path) -> Case {
    let mut t = Case::new("target_exists");
    for (kind, seed) in (0..5).zip(0xa000u64..) {
        let mut rng = Rng(seed);
        let dir = fresh(work);
        let mut pin = complete_backup(&mut rng, work, &dir);
        match kind {
            0 => {}
            1 => backup::withdraw(&dir, "superseded").unwrap(),
            2 => {
                let path = dir.join(SNAPSHOT);
                let mut bytes = fs::read(&path).unwrap();
                bytes[30] ^= 4;
                rewrite(&path, &bytes);
            }
            3 => fs::remove_file(dir.join(MANIFEST)).unwrap(),
            _ => pin[0] ^= 1,
        }
        for target in [Target::File, Target::Dir, Target::DanglingLink] {
            trial(&mut t, work, &dir, pin, target);
        }
    }
    t
}

fn publication_failure(work: &Path) -> Case {
    let mut t = Case::new("publication_failure");
    for (kind, seed) in (0..2).zip(0xb000u64..) {
        let mut rng = Rng(seed);
        let dir = fresh(work);
        let pin = complete_backup(&mut rng, work, &dir);
        if kind == 1 {
            fs::remove_file(dir.join(MANIFEST)).unwrap();
        }
        for target in [
            Target::MissingParent,
            Target::EmptyPath,
            Target::ReadOnlyParent,
        ] {
            trial(&mut t, work, &dir, pin, target);
        }
    }
    t
}

/// Runs every case in a fresh subdirectory of the system temporary directory.
pub fn run(scale: Scale) -> Vec<Tally> {
    run_in(scale, &std::env::temp_dir())
}

/// Runs every case in a fresh subdirectory of `base`, one tally per case. At
/// full scale the distinct results of every case are listed on stderr
/// (`IDENTITY_RESULT` lines).
pub fn run_in(scale: Scale, base: &Path) -> Vec<Tally> {
    let cases = run_cases(scale, base);
    if scale == Scale::Full {
        for t in &cases {
            for result in &t.results {
                eprintln!("IDENTITY_RESULT case={} result={result}", t.name);
            }
        }
    }
    let count = |n: u64| usize::try_from(n).expect("case count fits usize");
    cases
        .iter()
        .map(|c| Tally::counted(c.name, count(c.cases), count(c.equal)))
        .collect()
}

/// Runs every case in a fresh subdirectory of `base`, removed afterwards.
pub fn run_cases(scale: Scale, base: &Path) -> Vec<Case> {
    let work = base.join(format!(
        "identity-backup-restore-{}-{}",
        std::process::id(),
        unique()
    ));
    fs::create_dir_all(&work).expect("create the work directory");
    let n = |full: u64, small: u64| if scale == Scale::Full { full } else { small };
    let tallies = vec![
        complete_seeded(&work, n(120, 12)),
        withdrawn(&work, n(20, 3)),
        incomplete(&work, n(20, 4)),
        untrusted(&work, n(20, 3)),
        snapshot_mutation(&work, n(126, 18), scale),
        manifest_mutation(&work, n(120, 20)),
        manifest_disagree(&work, n(30, 6)),
        pinned_unparseable(&work, n(20, 4)),
        unexpected_entry(&work, n(12, 3)),
        inspect_error(&work),
        target_exists(&work),
        publication_failure(&work),
    ];
    let _ = fs::remove_dir_all(&work);
    tallies
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let base: Option<PathBuf> = args.next().map(PathBuf::from);
    if args.next().is_some() {
        eprintln!("usage: identity_backup_restore [DIR]");
        return ExitCode::from(2);
    }
    match base {
        None => identity::report(&run(Scale::Full)),
        Some(base) => identity::report(&run_in(Scale::Full, &base)),
    }
}
