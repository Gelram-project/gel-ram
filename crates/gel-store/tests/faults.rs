// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Faults of a saved store file, on disk and between processes: damage that
//! keeps the length, two processes saving one path with readers beside them, a
//! writer killed between its acknowledgements and a restart after it, leftover
//! temporary files, and what reading a file leaves behind. A child process is
//! this test binary run again. Killing a process is not a power cut; nothing
//! here shows power-loss durability. `write_if_newer` is not a compare-and-swap
//! between processes, so only `write_atomic` is raced here.
#![forbid(unsafe_code)]

use gel_core::{checksum64, splitmix64, GelError, Generation, GEL_MAGIC_V1, ORB_WORDS};
use gel_orb::Orb1024;
use gel_store::{verify_file, Header, OpenLimits, RamStore, HEADER_BYTES};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const FILE: &str = "bank.gel";
/// Records of the stores two writers race on, and the saves of each writer.
const RACE_RECORDS: usize = 64;
const RACE_SAVES: u64 = 40;
/// Two write blocks, so that a save takes more than one write.
const KILL_RECORDS: usize = 2 * 8192;
const WAIT_LIMIT: Duration = Duration::from_secs(120);
/// `create_temp_file` tries this many names before it gives up.
const TEMP_TRIES: usize = 128;

fn orb(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

fn records(count: usize, seed: u64) -> Vec<Orb1024> {
    (0..count as u64).map(|i| orb(seed + i)).collect()
}

/// The store a writer saves at `generation`: its first records change with it.
fn store(count: usize, seed: u64, generation: u64) -> RamStore {
    let mut orbs = records(count, seed);
    orbs[0] = orb(seed ^ (generation << 32));
    orbs[count - 1] = orb(seed ^ (generation << 40));
    RamStore::from_orbs(orbs, Generation(generation))
}

fn payload(orbs: &[Orb1024]) -> Vec<u8> {
    orbs.iter().flat_map(|orb| orb.to_le_bytes()).collect()
}

fn v2_bytes(store: &RamStore) -> Vec<u8> {
    [store.header().encode().to_vec(), payload(store.orbs())].concat()
}

/// A version 1 file written by hand from its format.
fn v1_bytes(orbs: &[Orb1024]) -> Vec<u8> {
    let payload = payload(orbs);
    let mut bytes = vec![0u8; HEADER_BYTES];
    bytes[0..8].copy_from_slice(&GEL_MAGIC_V1);
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1024u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&128u32.to_le_bytes());
    bytes[24..32].copy_from_slice(&(orbs.len() as u64).to_le_bytes());
    bytes[32..40].copy_from_slice(&7u64.to_le_bytes());
    bytes[40..48].copy_from_slice(&checksum64(&payload).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

fn open(path: &Path) -> Result<RamStore, GelError> {
    RamStore::open_verified(path)
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "gel-store-faults-{}-{}",
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

/// The child process, started by the tests of this file; ignored in a normal
/// run, and it does nothing unless a parent test sets its variables.
#[test]
#[ignore]
fn store_fault_child() {
    let Ok(mode) = std::env::var("GEL_STORE_FAULT_MODE") else {
        return;
    };
    let var = |name: &str| std::env::var(name).unwrap();
    let path = PathBuf::from(var("GEL_STORE_FAULT_PATH"));
    let seed: u64 = var("GEL_STORE_FAULT_SEED").parse().unwrap();
    match mode.as_str() {
        "race" => race_saves(
            &path,
            seed,
            Path::new(&var("GEL_STORE_FAULT_SIGNALS")),
            &var("GEL_STORE_FAULT_NAME"),
            &var("GEL_STORE_FAULT_OTHER"),
        ),
        "acknowledge" => acknowledge_saves(&path, seed),
        "leftovers" => save_over_leftovers(
            &path,
            seed,
            var("GEL_STORE_FAULT_LEFTOVERS").parse().unwrap(),
        ),
        other => panic!("unknown mode {other}"),
    }
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

/// Saves `RACE_SAVES` generations. The last waits for the first save of the
/// other writer, so the time of the two writers overlaps whatever the scheduler
/// does.
fn race_saves(path: &Path, seed: u64, signals: &Path, name: &str, other: &str) {
    for generation in 1..=RACE_SAVES {
        if generation == RACE_SAVES {
            await_signal(signals, other);
        }
        store(RACE_RECORDS, seed, generation)
            .write_atomic(path)
            .unwrap();
        open(path).unwrap();
        if generation == 1 {
            fs::write(signals.join(name), b"").unwrap();
        }
    }
}

/// Saves generation after generation, acknowledging each on standard output,
/// until the parent kills it. The next store is built before the acknowledgement,
/// so the save starts right after it.
fn acknowledge_saves(path: &Path, seed: u64) {
    let mut next = store(KILL_RECORDS, seed, 1);
    for generation in 1.. {
        next.write_atomic(path).unwrap();
        next = store(KILL_RECORDS, seed, generation + 1);
        println!("\nACK {generation}");
    }
}

fn leftover_path(path: &Path, sequence: usize) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}-{sequence}", std::process::id()));
    PathBuf::from(tmp)
}

/// Part of the next file, as a writer killed in the middle of its temporary file
/// leaves it.
fn partial(bytes: &[u8], sequence: usize) -> &[u8] {
    &bytes[..sequence * 97 % bytes.len()]
}

/// This process is fresh, so its first temporary names are `<path>.tmp-<pid>-0`
/// and upwards: the state a restart under a reused process id finds.
fn save_over_leftovers(path: &Path, seed: u64, leftovers: usize) {
    let next = store(RACE_RECORDS, seed, 2);
    let bytes = v2_bytes(&next);
    for sequence in 0..leftovers {
        fs::write(leftover_path(path, sequence), partial(&bytes, sequence)).unwrap();
    }
    let saved = next.write_if_newer(path);
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

fn child(mode: &str, path: &Path, seed: u64, vars: &[(&str, String)]) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "store_fault_child",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("GEL_STORE_FAULT_MODE", mode)
        .env("GEL_STORE_FAULT_PATH", path)
        .env("GEL_STORE_FAULT_SEED", seed.to_string());
    for (name, value) in vars {
        command.env(format!("GEL_STORE_FAULT_{name}"), value);
    }
    command
}

fn assert_success(child: &mut Child) {
    let status = child.wait().unwrap();
    assert!(status.success(), "child: {status}");
}

/// Reads `path` from three threads until all `children` have ended, and returns
/// the first read that is not a whole saved file, if any.
#[cfg(unix)]
fn read_beside(path: &Path, known: &[Vec<u8>], children: &mut [Child]) -> Option<String> {
    use std::sync::atomic::AtomicBool;
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..3)
            .map(|_| {
                scope.spawn(|| {
                    while !done.load(Ordering::Relaxed) {
                        // Once the file exists it is only ever replaced.
                        if !path.exists() {
                            continue;
                        }
                        match fs::read(path) {
                            Ok(bytes) if known.binary_search(&bytes).is_ok() => {}
                            Ok(bytes) => return Some(format!("a read of {} bytes", bytes.len())),
                            Err(e) => return Some(e.to_string()),
                        }
                        let verified = verify_file(path, OpenLimits::SAFE_DEFAULT);
                        match (verified, open(path)) {
                            (Ok(_), Ok(found)) => {
                                if known.binary_search(&v2_bytes(&found)).is_err() {
                                    return Some("an unknown store opened".into());
                                }
                            }
                            (a, b) => return Some(format!("{:?} {:?}", a.err(), b.err())),
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

#[cfg(unix)]
#[test]
fn two_processes_saving_one_store_leave_one_complete_file() {
    let scratch = Scratch::new();
    let (target, signals) = scratch.target_and_signals();
    let path = target.join(FILE);
    let seeds = [61u64, 1 << 20];
    let mut known: Vec<Vec<u8>> = seeds
        .iter()
        .flat_map(|&seed| (1..=RACE_SAVES).map(move |g| v2_bytes(&store(RACE_RECORDS, seed, g))))
        .collect();
    known.sort();
    let race = |seed: u64, name: &str, other: &str| {
        child(
            "race",
            &path,
            seed,
            &[
                ("SIGNALS", signals.display().to_string()),
                ("NAME", name.to_string()),
                ("OTHER", other.to_string()),
            ],
        )
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
    };
    let mut children = [
        race(seeds[0], "first", "second"),
        race(seeds[1], "second", "first"),
    ];
    assert_eq!(read_beside(&path, &known, &mut children), None);
    // The last save of either writer is the last replacement of the file.
    let last = fs::read(&path).unwrap();
    assert!(seeds
        .iter()
        .any(|&seed| last == v2_bytes(&store(RACE_RECORDS, seed, RACE_SAVES))));
    assert_eq!(v2_bytes(&open(&path).unwrap()), last);
    assert_eq!(names(&target), [FILE]);
}

#[cfg(unix)]
#[test]
fn a_killed_store_writer_keeps_acknowledged_saves_and_a_restart_resumes() {
    use std::io::{BufRead, BufReader};
    let scratch = Scratch::new();
    let seed = 62;
    for trial in 0..8 {
        let target = scratch.0.join(format!("trial{trial}"));
        fs::create_dir(&target).unwrap();
        let path = target.join(FILE);
        let mut writer = child("acknowledge", &path, seed, &[])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        // The writer starts its next save right after an acknowledgement. The
        // kill follows it by a delay that grows with the trial, so the trials
        // land along that save; what must hold does not depend on where.
        let wanted = 1 + trial % 2;
        let delay = Duration::from_micros(250 * trial);
        let mut acknowledged = 0;
        let mut lines = BufReader::new(writer.stdout.take().unwrap()).lines();
        for line in lines.by_ref() {
            if let Some(generation) = line.unwrap().strip_prefix("ACK ") {
                acknowledged = generation.parse().unwrap();
                if acknowledged == wanted {
                    break;
                }
            }
        }
        let acknowledged_at = Instant::now();
        while acknowledged_at.elapsed() < delay {
            std::hint::spin_loop();
        }
        writer.kill().unwrap();
        writer.wait().unwrap();
        drop(lines);
        assert_eq!(acknowledged, wanted, "the writer stopped acknowledging");

        // The file holds the last acknowledged generation or a later whole one.
        let found = open(&path).unwrap();
        let generation = found.header().generation.0;
        assert!(generation >= acknowledged, "trial {trial}");
        let expected = store(KILL_RECORDS, seed, generation);
        assert_eq!(found.orbs(), expected.orbs(), "trial {trial}");
        assert_eq!(fs::read(&path).unwrap(), v2_bytes(&expected));

        // A fresh process refuses what is not newer and resumes from the file.
        assert_eq!(
            expected.write_if_newer(&path),
            Err(GelError::GenerationRollback {
                current: generation,
                attempted: generation
            })
        );
        let next = store(KILL_RECORDS, seed, generation + 1);
        next.write_if_newer(&path).unwrap();
        assert_eq!(fs::read(&path).unwrap(), v2_bytes(&next));
        // A kill leaves one temporary file at most, and nothing ever opens it.
        assert!(names(&target).len() <= 2, "{:?}", names(&target));
    }
}

#[test]
fn a_store_restart_meeting_its_own_leftover_names_skips_them_and_then_fails_closed() {
    let scratch = Scratch::new();
    let seed = 63;
    let previous = store(RACE_RECORDS, seed, 1);
    let previous_bytes = v2_bytes(&previous);
    // A directory per child: the id of an ended process can come back, and a
    // child under it would meet the names of the one before as its own.
    let run = |name: &str, leftovers: usize| {
        let dir = scratch.directory(name);
        let path = dir.join(FILE);
        previous.write_atomic(&path).unwrap();
        let vars = [("LEFTOVERS", leftovers.to_string())];
        let mut command = child("leftovers", &path, seed, &vars);
        assert_success(&mut command.stdout(Stdio::null()).spawn().unwrap());
        (dir, path)
    };

    // Every name taken: the save fails and the previous file stays.
    let (dir, path) = run("every-name-taken", TEMP_TRIES);
    assert_eq!(fs::read(&path).unwrap(), previous_bytes);
    assert_eq!(names(&dir).len(), TEMP_TRIES + 1);

    // The last name free: the save takes it.
    let (dir, path) = run("last-name-free", TEMP_TRIES - 1);
    let next = store(RACE_RECORDS, seed, 2);
    assert_eq!(fs::read(&path).unwrap(), v2_bytes(&next));
    assert_eq!(names(&dir).len(), TEMP_TRIES);

    // A fresh process resumes from the file, whatever the leftovers hold.
    assert_eq!(open(&path).unwrap().header().generation, Generation(2));
    store(RACE_RECORDS, seed, 3).write_if_newer(&path).unwrap();
    assert_eq!(open(&path).unwrap().header().generation, Generation(3));
}

/// Writes `bytes` to `path`; loading and streaming verification must both refuse
/// them with `expected`.
fn assert_refused(path: &Path, bytes: &[u8], expected: &GelError, label: &str) {
    fs::write(path, bytes).unwrap();
    assert_eq!(open(path).err().as_ref(), Some(expected), "{label}");
    assert_eq!(
        verify_file(path, OpenLimits::SAFE_DEFAULT).err().as_ref(),
        Some(expected),
        "{label}"
    );
}

/// The damage to one file that keeps its length: a block zeroed and two blocks
/// exchanged, at the sizes of a record and of a file system.
fn same_length_damage(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut damaged = Vec::new();
    for block in [128, 512, 4096] {
        for start in (HEADER_BYTES..bytes.len()).step_by(block) {
            let mut zeroed = bytes.to_vec();
            zeroed[start..(start + block).min(bytes.len())].fill(0);
            damaged.push((format!("{block} zero bytes at {start}"), zeroed));
        }
        let mut exchanged = bytes.to_vec();
        let (left, right) = exchanged[HEADER_BYTES..].split_at_mut(block);
        left.swap_with_slice(&mut right[..block]);
        damaged.push((format!("two blocks of {block} exchanged"), exchanged));
    }
    damaged
}

#[test]
fn damage_that_keeps_the_length_of_a_saved_store_is_refused() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let count = 64;
    let older = store(count, 64, 8);
    let newer = store(count, 64, 9);
    newer.write_atomic(&path).unwrap();
    let new_bytes = fs::read(&path).unwrap();
    assert_eq!(new_bytes, v2_bytes(&newer));
    let old_bytes = v2_bytes(&older);
    assert_eq!(new_bytes.len(), HEADER_BYTES + 64 * 128);

    let mut damaged = same_length_damage(&new_bytes);
    // A write torn between two generations: the header of one over the payload
    // of the other, at every byte of the two records that differ.
    for at in (HEADER_BYTES..HEADER_BYTES + 128).chain(new_bytes.len() - 128..new_bytes.len()) {
        let torn = [&new_bytes[..at], &old_bytes[at..]].concat();
        if torn != new_bytes {
            damaged.push((format!("new up to byte {at}, then old"), torn));
        }
    }
    let old_payload = [&new_bytes[..HEADER_BYTES], &old_bytes[HEADER_BYTES..]].concat();
    damaged.push(("a new header over an old payload".into(), old_payload));
    assert_eq!(damaged.len(), 64 + 16 + 2 + 3 + 256 + 1);
    for (label, bytes) in &damaged {
        assert_refused(&path, bytes, &GelError::CorruptStore, label);
    }

    // The first bytes of a cut file under a header sealed for that shorter file:
    // the header is whole, and the payload digest of the longer file refuses it.
    for kept in 1..count {
        let header = Header::new(kept as u64, Generation(9), newer.header().payload_crc64);
        let cut = HEADER_BYTES + kept * 128;
        let bytes = [&header.encode()[..], &new_bytes[HEADER_BYTES..cut]].concat();
        assert_refused(&path, &bytes, &GelError::CorruptStore, "sealed cut");
    }

    // A version 1 file: the same damage, and every cut.
    let legacy = v1_bytes(&records(count, 64));
    for (label, bytes) in same_length_damage(&legacy) {
        assert_refused(&path, &bytes, &GelError::CorruptStore, &label);
    }
    for len in HEADER_BYTES..legacy.len() {
        let expected = GelError::InvalidLength {
            expected: legacy.len(),
            actual: len,
        };
        assert_refused(&path, &legacy[..len], &expected, "a cut version 1 file");
    }
    fs::write(&path, &legacy).unwrap();
    assert_eq!(open(&path).unwrap().orbs(), records(count, 64));
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
fn reading_a_saved_store_changes_nothing_on_disk() {
    let scratch = Scratch::new();
    let good = scratch.0.join("good");
    let bad = scratch.0.join("bad");
    let legacy = scratch.0.join("legacy");
    let saved = store(32, 65, 4);
    saved.write_atomic(&good).unwrap();
    let mut changed = fs::read(&good).unwrap();
    changed[HEADER_BYTES + 3] ^= 1;
    fs::write(&bad, &changed).unwrap();
    fs::write(&legacy, v1_bytes(saved.orbs())).unwrap();
    let permissions = [freeze(&good), freeze(&bad), freeze(&legacy)];

    let before = [on_disk(&good), on_disk(&bad), on_disk(&legacy)];
    assert_eq!(open(&good).unwrap().orbs(), saved.orbs());
    assert!(verify_file(&good, OpenLimits::SAFE_DEFAULT).is_ok());
    // A refused file stays where it is, as it is.
    assert_eq!(open(&bad).err(), Some(GelError::CorruptStore));
    assert_eq!(
        verify_file(&bad, OpenLimits::SAFE_DEFAULT).err(),
        Some(GelError::CorruptStore)
    );
    assert!(open(&legacy).is_ok());
    // A refused save leaves the file and no temporary file behind.
    assert_eq!(
        store(32, 65, 3).write_if_newer(&good),
        Err(GelError::GenerationRollback {
            current: 4,
            attempted: 3
        })
    );
    assert_eq!(
        store(32, 65, 5).write_if_newer(&bad).err(),
        Some(GelError::CorruptStore)
    );
    assert_eq!(
        store(32, 65, 5).write_if_newer(&legacy),
        Err(GelError::LegacyGenerationUntrusted)
    );
    assert_eq!([on_disk(&good), on_disk(&bad), on_disk(&legacy)], before);

    for (path, original) in [&good, &bad, &legacy].into_iter().zip(permissions) {
        fs::set_permissions(path, original).unwrap();
    }
}
