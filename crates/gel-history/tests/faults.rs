// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Faults of a saved history file, on disk and between processes: a cut file,
//! damage that keeps the length, two processes saving one path, a reader beside
//! a writer, a restart that meets leftover temporary files, and what opening a
//! file leaves behind. A child process is this test binary run again. Ending or
//! racing processes is not a power cut; nothing here shows power-loss durability.
#![forbid(unsafe_code)]

use gel_core::{splitmix64, GelError, ORB_BITS, ORB_WORDS};
use gel_history::RecordHistory;
use gel_orb::Orb1024;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const NO_LIMIT: u64 = u64::MAX;
const FILE: &str = "record.gelhis";
/// States in the first version a writer saves; each later version adds one.
const FIRST_STATES: usize = 24;
/// Versions one writer saves.
const VERSIONS: usize = 40;
const WAIT_LIMIT: Duration = Duration::from_secs(120);
/// `create_temp_file` tries this many names before it gives up.
const TEMP_TRIES: usize = 128;

fn sample(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

/// `n` states, each differing from the one before in three bits.
fn walk(n: usize, seed: u64) -> Vec<Orb1024> {
    let mut states = vec![sample(seed)];
    for i in 1..n {
        let mut next = states[i - 1];
        for j in 0..3 {
            let bit = (i * 101 + j * 37) % ORB_BITS;
            next.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
        }
        states.push(next);
    }
    states
}

fn history_of(states: &[Orb1024]) -> RecordHistory {
    let mut history = RecordHistory::new();
    for state in states {
        history.append(*state).unwrap();
    }
    history
}

/// The bytes of version `v` of the writer with this seed.
#[cfg(unix)]
fn version(seed: u64, v: usize) -> Vec<u8> {
    history_of(&walk(FIRST_STATES + v, seed))
        .to_bytes()
        .unwrap()
}

fn open(path: &Path) -> Result<RecordHistory, GelError> {
    RecordHistory::open_verified(path, NO_LIMIT, NO_LIMIT)
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "gel-history-faults-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
    }
    fn directory(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir(&path).unwrap();
        path
    }
    /// A directory for the saved file and one for the signals between
    /// processes, so that the first holds only what the library leaves there.
    #[cfg(unix)]
    fn target_and_signals(&self) -> (PathBuf, PathBuf) {
        (self.directory("target"), self.directory("signals"))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn signal(signals: &Path, name: &str) {
    fs::write(signals.join(name), b"").unwrap();
}

/// Waits for another process, without assuming how long it takes.
fn await_signal(signals: &Path, name: &str) {
    let path = signals.join(name);
    let start = Instant::now();
    while !path.exists() {
        assert!(start.elapsed() < WAIT_LIMIT, "no signal {name}");
        std::thread::yield_now();
    }
}

/// The child process, started by the tests of this file; ignored in a normal
/// run, and it does nothing unless a parent test sets its variables.
#[test]
#[ignore]
fn history_fault_child() {
    let Ok(mode) = std::env::var("GEL_HISTORY_FAULT_MODE") else {
        return;
    };
    let var = |name: &str| std::env::var(name).unwrap();
    let path = PathBuf::from(var("GEL_HISTORY_FAULT_PATH"));
    match mode.as_str() {
        "save" => save_versions(
            &path,
            var("GEL_HISTORY_FAULT_SEED").parse().unwrap(),
            Path::new(&var("GEL_HISTORY_FAULT_SIGNALS")),
            &var("GEL_HISTORY_FAULT_NAME"),
            &var("GEL_HISTORY_FAULT_AWAIT"),
        ),
        "leftovers" => save_over_leftovers(
            &path,
            var("GEL_HISTORY_FAULT_SEED").parse().unwrap(),
            var("GEL_HISTORY_FAULT_LEFTOVERS").parse().unwrap(),
        ),
        other => panic!("unknown mode {other}"),
    }
}

/// Saves every version in turn. After the first save it signals `<name>-saved`;
/// before the last it waits for `wait_for` when that is not empty, so the time
/// of these saves overlaps whatever the other side does.
fn save_versions(path: &Path, seed: u64, signals: &Path, name: &str, wait_for: &str) {
    let states = walk(FIRST_STATES + VERSIONS - 1, seed);
    let mut history = history_of(&states[..FIRST_STATES - 1]);
    for (v, state) in states[FIRST_STATES - 1..].iter().enumerate() {
        if v + 1 == VERSIONS && !wait_for.is_empty() {
            await_signal(signals, wait_for);
        }
        history.append(*state).unwrap();
        history.write_atomic(path).unwrap();
        open(path).unwrap();
        if v == 0 {
            signal(signals, &format!("{name}-saved"));
        }
    }
}

fn leftover_path(path: &Path, sequence: usize) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}-{sequence}", std::process::id()));
    PathBuf::from(tmp)
}

/// Part of the next version, as a writer killed in the middle of its temporary
/// file leaves it.
fn partial(bytes: &[u8], sequence: usize) -> &[u8] {
    &bytes[..sequence * 7 % bytes.len()]
}

/// This process is fresh, so its first temporary names are `<path>.tmp-<pid>-0`
/// and upwards: the state a restart under a reused process id finds.
fn save_over_leftovers(path: &Path, seed: u64, leftovers: usize) {
    let next = history_of(&walk(FIRST_STATES + 1, seed));
    let bytes = next.to_bytes().unwrap();
    for sequence in 0..leftovers {
        fs::write(leftover_path(path, sequence), partial(&bytes, sequence)).unwrap();
    }
    let saved = next.write_atomic(path);
    for sequence in 0..leftovers {
        assert_eq!(
            fs::read(leftover_path(path, sequence)).unwrap(),
            partial(&bytes, sequence),
            "leftover {sequence} changed"
        );
    }
    match saved {
        Ok(()) if leftovers < TEMP_TRIES => {}
        Err(GelError::Io(message))
            if leftovers == TEMP_TRIES && message.contains("could not allocate") => {}
        other => panic!("{leftovers} leftovers: {other:?}"),
    }
}

fn child(mode: &str, path: &Path, vars: &[(&str, String)]) -> Child {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "history_fault_child",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("GEL_HISTORY_FAULT_MODE", mode)
        .env("GEL_HISTORY_FAULT_PATH", path)
        .stdout(std::process::Stdio::null());
    for (name, value) in vars {
        command.env(format!("GEL_HISTORY_FAULT_{name}"), value);
    }
    command.spawn().unwrap()
}

#[cfg(unix)]
fn saver(path: &Path, seed: u64, signals: &Path, name: &str, wait_for: &str) -> Child {
    child(
        "save",
        path,
        &[
            ("SEED", seed.to_string()),
            ("SIGNALS", signals.display().to_string()),
            ("NAME", name.to_string()),
            ("AWAIT", wait_for.to_string()),
        ],
    )
}

fn assert_success(child: &mut Child) {
    let status = child.wait().unwrap();
    assert!(status.success(), "child: {status}");
}

/// Every version of the writers with these seeds, sorted for binary search.
#[cfg(unix)]
fn known_versions(seeds: &[u64]) -> Vec<Vec<u8>> {
    let mut known: Vec<Vec<u8>> = seeds
        .iter()
        .flat_map(|&seed| (0..VERSIONS).map(move |v| version(seed, v)))
        .collect();
    known.sort();
    known
}

/// Reads `path` from three threads until all `children` have ended, and returns
/// the first read that is not a whole saved version, if any. The signal `read`
/// is sent once a read has seen a version.
#[cfg(unix)]
fn read_beside(
    path: &Path,
    known: &[Vec<u8>],
    signals: &Path,
    children: &mut [Child],
) -> Option<String> {
    use std::sync::atomic::AtomicBool;
    let done = AtomicBool::new(false);
    // A writer waiting for a read must not wait for one that will not come.
    let bad = |reason: String| {
        signal(signals, "read");
        Some(reason)
    };
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..3)
            .map(|_| {
                scope.spawn(|| {
                    let mut told = false;
                    while !done.load(Ordering::Relaxed) {
                        // Once the file exists it is only ever replaced.
                        if !path.exists() {
                            continue;
                        }
                        match fs::read(path) {
                            Ok(bytes) if known.binary_search(&bytes).is_ok() => {
                                if !told {
                                    signal(signals, "read");
                                    told = true;
                                }
                            }
                            Ok(bytes) => return bad(format!("a read of {} bytes", bytes.len())),
                            Err(e) => return bad(e.to_string()),
                        }
                        match open(path) {
                            Ok(found) => {
                                let bytes = found.to_bytes().unwrap();
                                if known.binary_search(&bytes).is_err() {
                                    return bad("an unknown history opened".into());
                                }
                            }
                            Err(e) => return bad(e.to_string()),
                        }
                    }
                    None
                })
            })
            .collect();
        let statuses: Vec<_> = children.iter_mut().map(|c| c.wait().unwrap()).collect();
        done.store(true, Ordering::Relaxed);
        let bad_read = readers.into_iter().find_map(|r| r.join().unwrap());
        assert!(statuses.iter().all(|s| s.success()), "{statuses:?}");
        bad_read
    })
}

#[test]
fn every_cut_of_a_saved_history_is_refused_on_disk() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let states = walk(30, 41);
    let history = history_of(&states);
    history.write_atomic(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 1678);
    let mut refused = 0;
    for len in 0..bytes.len() {
        fs::write(&path, &bytes[..len]).unwrap();
        let expected = if len < 48 {
            GelError::InvalidHeader("history file shorter than header")
        } else {
            GelError::InvalidLength {
                expected: bytes.len(),
                actual: len,
            }
        };
        assert_eq!(open(&path), Err(expected), "length {len}");
        assert_eq!(fs::read(&path).unwrap(), &bytes[..len], "length {len}");
        refused += 1;
    }
    assert_eq!(refused, 1678);
    // A save replaces what was cut.
    history.write_atomic(&path).unwrap();
    assert_eq!(open(&path).unwrap().exact_history().unwrap(), states);
}

#[test]
fn damage_that_keeps_the_length_of_a_saved_history_is_refused() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let first = history_of(&walk(30, 42)).to_bytes().unwrap();
    let second = history_of(&walk(30, 43)).to_bytes().unwrap();
    assert_eq!(first.len(), second.len());

    let mut damaged: Vec<(String, Vec<u8>)> = Vec::new();
    // A write torn between two versions, at every byte.
    for at in 1..first.len() {
        let spliced = [&first[..at], &second[at..]].concat();
        if spliced != first && spliced != second {
            damaged.push((format!("splice at {at}"), spliced));
        }
    }
    // A block the file system left zeroed, at each block size that fits.
    for block in [64, 512, 1024] {
        for start in (0..first.len()).step_by(block) {
            let mut zeroed = first.clone();
            zeroed[start..(start + block).min(first.len())].fill(0);
            damaged.push((format!("{block} zero bytes at {start}"), zeroed));
        }
    }
    // Two blocks exchanged, and one block of the other version.
    let mut exchanged = first.clone();
    let (head, tail) = exchanged[64..].split_at_mut(512);
    head[..512].swap_with_slice(&mut tail[..512]);
    damaged.push(("two blocks exchanged".into(), exchanged));
    let mut stale = first.clone();
    stale[512..1024].copy_from_slice(&second[512..1024]);
    damaged.push(("one stale block".into(), stale));

    // 1,611 splices differ from both versions, then 27 + 4 + 2 zero blocks and two.
    assert_eq!(damaged.len(), 1646);
    for (label, bytes) in damaged {
        assert_eq!(bytes.len(), first.len());
        fs::write(&path, &bytes).unwrap();
        assert!(
            matches!(
                open(&path),
                Err(GelError::InvalidMagic | GelError::CorruptHeader | GelError::CorruptStore)
            ),
            "{label}: {:?}",
            open(&path)
        );
    }
}

#[cfg(unix)]
#[test]
fn two_processes_saving_one_history_leave_one_complete_file() {
    let scratch = Scratch::new();
    let (target, signals) = scratch.target_and_signals();
    let path = target.join(FILE);
    let seeds = [51, 52];
    let known = known_versions(&seeds);
    let mut children = [
        saver(&path, seeds[0], &signals, "first", "second-saved"),
        saver(&path, seeds[1], &signals, "second", "first-saved"),
    ];
    let bad_read = read_beside(&path, &known, &signals, &mut children);
    assert_eq!(bad_read, None);
    // The last save of either writer is the last replacement of the file.
    let last = fs::read(&path).unwrap();
    assert!(seeds.iter().any(|&s| last == version(s, VERSIONS - 1)));
    assert_eq!(open(&path).unwrap().to_bytes().unwrap(), last);
    assert_eq!(names(&target), [FILE]);
}

#[cfg(unix)]
#[test]
fn reading_during_saves_sees_whole_histories_and_changes_no_byte() {
    let seed = 53;
    let expected = version(seed, VERSIONS - 1);
    // The control: the same writer with nobody reading.
    let quiet = Scratch::new();
    let (target, signals) = quiet.target_and_signals();
    let path = target.join(FILE);
    assert_success(&mut saver(&path, seed, &signals, "writer", ""));
    assert_eq!(fs::read(&path).unwrap(), expected);

    let observed = Scratch::new();
    let (target, signals) = observed.target_and_signals();
    let path = target.join(FILE);
    let known = known_versions(&[seed]);
    // The writer holds its last save until a reader has seen a version.
    let mut children = [saver(&path, seed, &signals, "writer", "read")];
    let bad_read = read_beside(&path, &known, &signals, &mut children);
    assert_eq!(bad_read, None);
    assert!(signals.join("read").exists());
    assert_eq!(fs::read(&path).unwrap(), expected);
    assert_eq!(names(&target), [FILE]);
}

#[test]
fn a_history_restart_meeting_its_own_leftover_names_skips_them_and_then_fails_closed() {
    let scratch = Scratch::new();
    let seed = 54;
    let previous = history_of(&walk(FIRST_STATES, seed));
    let next = history_of(&walk(FIRST_STATES + 1, seed));
    let previous_bytes = previous.to_bytes().unwrap();
    // A directory per child: the id of an ended process can come back, and a
    // child under it would meet the names of the one before as its own.
    let run = |name: &str, leftovers: usize| {
        let dir = scratch.directory(name);
        let path = dir.join(FILE);
        previous.write_atomic(&path).unwrap();
        let vars = [
            ("SEED", seed.to_string()),
            ("LEFTOVERS", leftovers.to_string()),
        ];
        assert_success(&mut child("leftovers", &path, &vars));
        (dir, path)
    };

    // Every name taken: the save fails and the previous history stays.
    let (dir, path) = run("every-name-taken", TEMP_TRIES);
    assert_eq!(fs::read(&path).unwrap(), previous_bytes);
    assert_eq!(names(&dir).len(), TEMP_TRIES + 1);

    // The last name free: the save takes it.
    let (dir, path) = run("last-name-free", TEMP_TRIES - 1);
    assert_eq!(fs::read(&path).unwrap(), next.to_bytes().unwrap());
    assert_eq!(names(&dir).len(), TEMP_TRIES);

    // A fresh process resumes from the file, whatever the leftovers hold.
    let mut resumed = open(&path).unwrap();
    resumed
        .append(walk(FIRST_STATES + 2, seed)[FIRST_STATES + 1])
        .unwrap();
    resumed.write_atomic(&path).unwrap();
    assert_eq!(
        open(&path).unwrap().exact_history().unwrap(),
        walk(FIRST_STATES + 2, seed)
    );
}

/// Bytes, time of change, permissions and the names in the directory; the time
/// of access is not compared, because a read may change it.
#[derive(Debug, PartialEq)]
struct OnDisk {
    bytes: Vec<u8>,
    modified: SystemTime,
    permissions: fs::Permissions,
    names: Vec<String>,
}

fn on_disk(path: &Path) -> OnDisk {
    let metadata = fs::metadata(path).unwrap();
    OnDisk {
        bytes: fs::read(path).unwrap(),
        modified: metadata.modified().unwrap(),
        permissions: metadata.permissions(),
        names: names(path.parent().unwrap()),
    }
}

/// Makes `path` read-only with a time of change long past, so that a rewrite
/// with the same bytes shows and a reader that asks for write access fails.
/// Returns the permissions to put back.
fn freeze(path: &Path) -> fs::Permissions {
    let file = File::options().write(true).open(path).unwrap();
    file.set_modified(UNIX_EPOCH + Duration::from_secs(1_000_000_000))
        .unwrap();
    drop(file);
    let original = fs::metadata(path).unwrap().permissions();
    let mut read_only = original.clone();
    read_only.set_readonly(true);
    fs::set_permissions(path, read_only).unwrap();
    original
}

#[test]
fn opening_a_saved_history_changes_nothing_on_disk() {
    let scratch = Scratch::new();
    let good = scratch.0.join("good");
    let bad = scratch.0.join("bad");
    let history = history_of(&walk(9, 55));
    history.write_atomic(&good).unwrap();
    let mut changed = fs::read(&good).unwrap();
    changed[60] ^= 1;
    fs::write(&bad, &changed).unwrap();
    let good_permissions = freeze(&good);
    let bad_permissions = freeze(&bad);

    let before = (on_disk(&good), on_disk(&bad));
    assert_eq!(open(&good), Ok(history));
    assert_eq!(
        RecordHistory::open_verified(&good, 1, NO_LIMIT),
        Err(GelError::LimitExceeded("history record count"))
    );
    // A refused file stays where it is, as it is.
    assert_eq!(open(&bad), Err(GelError::CorruptStore));
    assert_eq!((on_disk(&good), on_disk(&bad)), before);

    fs::set_permissions(&good, good_permissions).unwrap();
    fs::set_permissions(&bad, bad_permissions).unwrap();
}
