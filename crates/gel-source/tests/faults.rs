// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Faults of saved snapshots, on disk and between processes: a cut file under
//! its own pin, damage that keeps the length, two processes publishing one path,
//! a save stopped at every byte and resumed, a reader at every step of a
//! publication, leftover temporary files, and what loading leaves behind. A child
//! process is this test binary run again. Ending a process is not a power cut;
//! nothing here shows power-loss durability.
#![forbid(unsafe_code)]

use gel_source::{
    collection::Collection, digest, hex, import_text, load_bundle, read_regular, write_bundle_new,
    write_file_new, write_file_new_with, BundleError, Error, PublicationIo, MAX_TEXT,
};
use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const FILE: &str = "snapshot";
const WAIT_LIMIT: Duration = Duration::from_secs(120);
/// The publisher retries this many taken names after the first.
const TEMP_RETRIES: usize = 32;
/// The bytes each step of the publication the reader follows writes.
const STEP_BYTES: usize = 16;
/// The exit status of a child that reached its point.
const STOPPED: i32 = 86;
const MARKER: &str = "FAULT_POINT_REACHED";

fn previous_collection() -> Collection {
    let mut collection = Collection::new();
    collection.add("First", "Zażółć gęślą.\nŁÓDŹ").unwrap();
    collection
}

fn next_collection() -> Collection {
    let mut collection = previous_collection();
    collection.add("Second", "exact source bytes").unwrap();
    collection
}

/// Two collections of one length, built from different words.
fn twins() -> (Collection, Collection) {
    let build = |a: &str, b: &str| {
        let mut collection = Collection::new();
        collection.add("One", a).unwrap();
        collection.add("Two", b).unwrap();
        collection
    };
    let (x, y) = (
        build("alpha ".repeat(40).trim(), "beta"),
        build("gamma ".repeat(40).trim(), "zeta"),
    );
    assert_eq!(x.to_bytes().len(), y.to_bytes().len());
    (x, y)
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "gel-source-faults-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
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

/// The publisher's own I/O: the trait's default methods.
struct Plain;
impl PublicationIo for Plain {}

/// Waits at the link until the other writer is there too, so that both link
/// with their temporary files complete and neither has been refused.
struct MeetAtLink<'a> {
    signals: &'a Path,
    name: &'a str,
    other: &'a str,
}
impl PublicationIo for MeetAtLink<'_> {
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        signal(self.signals, self.name);
        await_signal(self.signals, self.other);
        Plain.publish(tmp, path)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Point {
    /// After this many bytes of the temporary file.
    Written(usize),
    /// After the temporary file is flushed, before it is linked.
    Synced,
    /// After the temporary file is linked to the new path.
    Linked,
    /// After the directory is flushed, before the temporary file is removed.
    ParentSynced,
}

impl Point {
    fn name(self) -> String {
        match self {
            Point::Written(n) => format!("written-{n}"),
            Point::Synced => "synced".into(),
            Point::Linked => "linked".into(),
            Point::ParentSynced => "parent-synced".into(),
        }
    }
    fn parse(name: &str) -> Self {
        match name {
            "synced" => Point::Synced,
            "linked" => Point::Linked,
            "parent-synced" => Point::ParentSynced,
            other => Point::Written(other.strip_prefix("written-").unwrap().parse().unwrap()),
        }
    }
}

fn stop(at: Point) -> ! {
    println!("\n{MARKER} {}", at.name());
    let _ = std::io::stdout().flush();
    std::process::exit(STOPPED)
}

/// Real I/O, one byte at a time, ending the process at one point: nothing after
/// it runs, no destructor and no cleanup.
struct StopAt(Point);
impl PublicationIo for StopAt {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        if self.0 == Point::Written(0) {
            stop(self.0);
        }
        for (i, byte) in bytes.iter().enumerate() {
            file.write_all(std::slice::from_ref(byte))?;
            if self.0 == Point::Written(i + 1) {
                stop(self.0);
            }
        }
        Ok(())
    }
    fn sync(&mut self, file: &File) -> std::io::Result<()> {
        Plain.sync(file)?;
        if self.0 == Point::Synced {
            stop(self.0);
        }
        Ok(())
    }
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        Plain.publish(tmp, path)?;
        if self.0 == Point::Linked {
            stop(self.0);
        }
        Ok(())
    }
    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        Plain.sync_parent(parent)?;
        if self.0 == Point::ParentSynced {
            stop(self.0);
        }
        Ok(())
    }
}

/// Real I/O that stops after each step until the parent has looked at the path.
struct Lockstep<'a> {
    signals: &'a Path,
    step: usize,
}
impl Lockstep<'_> {
    fn reached(&mut self) {
        signal(self.signals, &format!("step-{}", self.step));
        await_signal(self.signals, &format!("seen-{}", self.step));
        self.step += 1;
    }
}
impl PublicationIo for Lockstep<'_> {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        self.reached();
        for piece in bytes.chunks(STEP_BYTES) {
            Plain.write(file, piece)?;
            self.reached();
        }
        Ok(())
    }
    fn sync(&mut self, file: &File) -> std::io::Result<()> {
        Plain.sync(file)?;
        self.reached();
        Ok(())
    }
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        Plain.publish(tmp, path)?;
        self.reached();
        Ok(())
    }
    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        Plain.sync_parent(parent)?;
        self.reached();
        Ok(())
    }
}

/// The child process, started by the tests of this file; ignored in a normal
/// run, and it does nothing unless a parent test sets its variables.
#[test]
#[ignore]
fn source_fault_child() {
    let Ok(mode) = std::env::var("GEL_SOURCE_FAULT_MODE") else {
        return;
    };
    let var = |name: &str| std::env::var(name).unwrap();
    let path = PathBuf::from(var("GEL_SOURCE_FAULT_PATH"));
    let next = next_collection().to_bytes();
    match mode.as_str() {
        "stop" => {
            let point = Point::parse(&var("GEL_SOURCE_FAULT_POINT"));
            let _ = write_file_new_with(&path, &next, &mut StopAt(point));
            // Reaching this line means the point was never reached; the parent fails.
        }
        "lockstep" => {
            let signals = PathBuf::from(var("GEL_SOURCE_FAULT_SIGNALS"));
            let mut io = Lockstep {
                signals: &signals,
                step: 0,
            };
            write_file_new_with(&path, &next, &mut io).unwrap();
        }
        "race" => {
            let (signals, name) = (
                var("GEL_SOURCE_FAULT_SIGNALS"),
                var("GEL_SOURCE_FAULT_NAME"),
            );
            let mut io = MeetAtLink {
                signals: Path::new(&signals),
                name: &name,
                other: &var("GEL_SOURCE_FAULT_OTHER"),
            };
            let bytes = race_bytes(&name);
            match write_file_new_with(&path, &bytes, &mut io) {
                Ok(pin) => println!("\nWON {}", hex(&pin)),
                Err(BundleError::Io(e)) if e.kind() == ErrorKind::AlreadyExists => {
                    println!("\nLOST")
                }
                Err(e) => panic!("{e:?}"),
            }
        }
        "leftovers" => {
            save_over_leftovers(&path, var("GEL_SOURCE_FAULT_LEFTOVERS").parse().unwrap())
        }
        other => panic!("unknown mode {other}"),
    }
}

fn race_bytes(name: &str) -> Vec<u8> {
    let mut collection = previous_collection();
    collection.add(name, "the bytes of one writer").unwrap();
    collection.to_bytes()
}

fn leftover_path(path: &Path, id: usize) -> PathBuf {
    path.with_file_name(format!(".gel-source-{}-{id}.tmp", std::process::id()))
}

/// Part of the next snapshot, as a writer killed in the middle of its temporary
/// file leaves it.
fn partial(bytes: &[u8], id: usize) -> &[u8] {
    &bytes[..id * 11 % bytes.len()]
}

/// This process is fresh, so its first temporary names are ids 0 and upwards:
/// the state a restart under a reused process id finds.
fn save_over_leftovers(path: &Path, leftovers: usize) {
    let next = next_collection().to_bytes();
    for id in 0..leftovers {
        fs::write(leftover_path(path, id), partial(&next, id)).unwrap();
    }
    let saved = write_file_new(path, &next);
    for id in 0..leftovers {
        assert_eq!(
            fs::read(leftover_path(path, id)).unwrap(),
            partial(&next, id),
            "leftover {id} changed"
        );
    }
    match saved {
        Ok(pin) if leftovers <= TEMP_RETRIES => assert_eq!(pin, digest(&next)),
        Err(BundleError::Io(e))
            if leftovers > TEMP_RETRIES && e.kind() == ErrorKind::AlreadyExists => {}
        other => panic!("{leftovers} leftovers: {other:?}"),
    }
}

fn child(mode: &str, path: &Path, vars: &[(&str, String)]) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "source_fault_child",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("GEL_SOURCE_FAULT_MODE", mode)
        .env("GEL_SOURCE_FAULT_PATH", path);
    for (name, value) in vars {
        command.env(format!("GEL_SOURCE_FAULT_{name}"), value);
    }
    command
}

#[test]
fn every_prefix_of_a_saved_collection_is_refused_even_under_its_own_pin() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let collection = next_collection();
    let pin = collection.save_new(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    for len in 0..bytes.len() {
        let cut = &bytes[..len];
        fs::write(&path, cut).unwrap();
        // The pin kept with the file, and a pin taken from the cut file: there
        // the structure alone has to refuse it.
        assert_eq!(
            Collection::load(&path, pin).err().as_deref(),
            Some("COLLECTION_INTEGRITY"),
            "length {len}"
        );
        assert_eq!(
            Collection::load(&path, digest(cut)).err().as_deref(),
            Some("COLLECTION_TRUNCATED"),
            "length {len}"
        );
    }
}

/// From 1 MiB a load digests on several threads where the host has them. One
/// long document puts the fields of two short ones past that size; the cuts are
/// at each of those fields, a stated subset, since every byte of a file this size
/// would take hours.
#[test]
fn cuts_at_the_fields_of_a_large_collection_are_refused_on_the_threaded_load() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let mut collection = Collection::new();
    let long = "exact source line\n".repeat(60 * 1024);
    collection.add("Long", &long).unwrap();
    let short = ["one", "two two"];
    for text in short {
        collection.add(text, text).unwrap();
    }
    let pin = collection.save_new(&path).unwrap();
    let bytes = fs::read(&path).unwrap();

    let mut cuts = vec![1 << 20];
    let mut record = 32 + 20 + "Long".len() + long.len();
    assert!(record > 1 << 20);
    for text in short {
        let title = record + 20;
        let body = title + text.len();
        cuts.extend([record, record + 8, record + 12, title, body]);
        record = body + text.len();
    }
    cuts.push(record - 1);
    assert_eq!(record, bytes.len());
    assert_eq!(cuts.len(), 12);
    for cut in cuts {
        let cut_bytes = &bytes[..cut];
        fs::write(&path, cut_bytes).unwrap();
        assert_eq!(
            Collection::load(&path, pin).err().as_deref(),
            Some("COLLECTION_INTEGRITY"),
            "length {cut}"
        );
        assert_eq!(
            Collection::load(&path, digest(cut_bytes)).err().as_deref(),
            Some("COLLECTION_TRUNCATED"),
            "length {cut}"
        );
    }
}

#[test]
fn every_prefix_of_a_saved_bundle_is_refused_even_under_its_own_pin() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let encoded = import_text("Zażółć 🦀\r\nIt's exact.".as_bytes(), "Notes").unwrap();
    let pin = write_bundle_new(&path, &encoded).unwrap();
    let bytes = fs::read(&path).unwrap();
    for len in 0..bytes.len() {
        let cut = &bytes[..len];
        fs::write(&path, cut).unwrap();
        assert!(
            matches!(
                load_bundle(&path, pin),
                Err(BundleError::Data(Error::Integrity))
            ),
            "length {len}"
        );
        assert!(
            matches!(
                load_bundle(&path, digest(cut)),
                Err(BundleError::Data(Error::Format))
            ),
            "length {len}"
        );
    }
}

/// The ways a file keeps its length and loses its bytes: a block zeroed, at a
/// few sizes, and every byte position where a write stopped between two files.
fn same_length_damage(first: &[u8], second: &[u8]) -> Vec<(String, Vec<u8>)> {
    assert_eq!(first.len(), second.len());
    let mut damaged = Vec::new();
    for at in 1..first.len() {
        let spliced = [&first[..at], &second[at..]].concat();
        if spliced != first && spliced != second {
            damaged.push((format!("splice at {at}"), spliced));
        }
    }
    for block in [16, 64, 256] {
        for start in (0..first.len()).step_by(block) {
            let mut zeroed = first.to_vec();
            zeroed[start..(start + block).min(first.len())].fill(0);
            damaged.push((format!("{block} zero bytes at {start}"), zeroed));
        }
    }
    damaged
}

#[test]
fn damage_that_keeps_the_length_of_a_snapshot_is_refused_under_its_pin() {
    let scratch = Scratch::new();
    let path = scratch.0.join(FILE);
    let (x, y) = twins();
    let (bytes, other) = (x.to_bytes(), y.to_bytes());
    let pin = digest(&bytes);
    let damaged = same_length_damage(&bytes, &other);
    assert_eq!(damaged.len(), 291);
    for (label, changed) in &damaged {
        fs::write(&path, changed).unwrap();
        assert_eq!(
            Collection::load(&path, pin).err().as_deref(),
            Some("COLLECTION_INTEGRITY"),
            "{label}"
        );
    }

    // Two bundles of one length, in one pass.
    let first = scratch.0.join("first");
    let second = scratch.0.join("second");
    let pin = write_bundle_new(&first, &import_text(b"same length one", "Notes").unwrap()).unwrap();
    write_bundle_new(&second, &import_text(b"same length two", "Notes").unwrap()).unwrap();
    let bytes = fs::read(&first).unwrap();
    for (label, changed) in same_length_damage(&bytes, &fs::read(&second).unwrap()) {
        fs::write(&path, &changed).unwrap();
        assert!(
            matches!(
                load_bundle(&path, pin),
                Err(BundleError::Data(Error::Integrity))
            ),
            "{label}"
        );
    }

    // The limit beside the refusals: exchanging two texts of one length gives
    // bytes that are a valid collection, so only the pin kept apart from the
    // file tells them from the saved ones.
    let mut pair = Collection::new();
    pair.add("One", "abc").unwrap();
    pair.add("Two", "xyz").unwrap();
    let saved = pair.to_bytes();
    let mut exchanged = saved.clone();
    // After the 32-byte header: a 20-byte record head, the title and the text.
    let (a, b) = (55, 81);
    assert_eq!(
        (&saved[a..a + 3], &saved[b..b + 3]),
        (&b"abc"[..], &b"xyz"[..])
    );
    exchanged[a..a + 3].copy_from_slice(b"xyz");
    exchanged[b..b + 3].copy_from_slice(b"abc");
    fs::write(&path, &exchanged).unwrap();
    assert_eq!(
        Collection::load(&path, digest(&saved)).err().as_deref(),
        Some("COLLECTION_INTEGRITY")
    );
    assert_eq!(
        Collection::load(&path, digest(&exchanged))
            .unwrap()
            .to_bytes(),
        exchanged
    );
}

#[test]
fn two_processes_publishing_one_path_have_one_winner_and_one_whole_file() {
    for round in 0..4 {
        let scratch = Scratch::new();
        let (target, signals) = scratch.target_and_signals();
        let path = target.join(FILE);
        let start = |name: &str, other: &str| {
            let vars = [
                ("SIGNALS", signals.display().to_string()),
                ("NAME", name.to_string()),
                ("OTHER", other.to_string()),
            ];
            child("race", &path, &vars)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
        };
        let writers = [
            ("first", start("first", "second")),
            ("second", start("second", "first")),
        ];
        let mut won = Vec::new();
        for (name, writer) in writers {
            let out = writer.wait_with_output().unwrap();
            assert!(out.status.success(), "round {round}: {name}: {out:?}");
            let stdout = String::from_utf8(out.stdout).unwrap();
            let result = stdout
                .lines()
                .find(|l| l.starts_with("WON ") || *l == "LOST")
                .unwrap_or_else(|| panic!("round {round}: {name}: {stdout}"));
            if let Some(pin) = result.strip_prefix("WON ") {
                won.push((name, pin.to_string()));
            }
        }
        assert_eq!(won.len(), 1, "round {round}: {won:?}");
        let (winner, pin) = &won[0];
        let bytes = race_bytes(winner);
        assert_eq!(hex(&digest(&bytes)), *pin);
        assert_eq!(fs::read(&path).unwrap(), bytes, "round {round}");
        Collection::load(&path, digest(&bytes)).unwrap();
        // Both temporary files are gone: the loser's was never linked.
        assert_eq!(names(&target), [FILE], "round {round}");
    }
}

#[test]
fn a_save_stopped_at_every_byte_resumes_to_the_uninterrupted_collection() {
    let next = next_collection();
    let expected = next.to_bytes();
    let pin = digest(&expected);
    let points: Vec<Point> = (0..=expected.len())
        .map(Point::Written)
        .chain([Point::Synced, Point::Linked, Point::ParentSynced])
        .collect();
    let kept = Scratch::new();
    let previous = kept.0.join("previous");
    let previous_pin = previous_collection().save_new(&previous).unwrap();
    let previous_bytes = fs::read(&previous).unwrap();
    for point in points {
        let scratch = Scratch::new();
        let (target, _signals) = scratch.target_and_signals();
        let path = target.join(FILE);

        let vars = [("POINT", point.name())];
        let out = child("stop", &path, &vars).output().unwrap();
        let marker = format!("{MARKER} {}", point.name());
        assert!(
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line == marker)
                && out.status.code() == Some(STOPPED),
            "{point:?}: the child did not stop there: {out:?}"
        );

        // Nothing or everything is at the path, and the temporary file holds
        // what was written before the stop.
        let linked = matches!(point, Point::Linked | Point::ParentSynced);
        if linked {
            assert_eq!(fs::read(&path).unwrap(), expected, "{point:?}");
        } else {
            assert_eq!(
                fs::metadata(&path).map_err(|e| e.kind()).err(),
                Some(ErrorKind::NotFound),
                "{point:?}"
            );
        }
        let temporary: Vec<String> = names(&target).into_iter().filter(|n| n != FILE).collect();
        assert_eq!(temporary.len(), 1, "{point:?}");
        let temporary = target.join(&temporary[0]);
        let held = fs::read(&temporary).unwrap();
        let written = match point {
            Point::Written(n) => n,
            _ => expected.len(),
        };
        assert_eq!(held, &expected[..written], "{point:?}");

        // A restart: the last saved snapshot opens, the same change is made
        // again, and the save either happens or finds the whole file there.
        let mut resumed = Collection::load(&previous, previous_pin).unwrap();
        resumed.add("Second", "exact source bytes").unwrap();
        match resumed.save_new(&path) {
            Ok(saved) => {
                assert!(!linked, "{point:?}: a published file was replaced");
                assert_eq!(saved, pin, "{point:?}");
            }
            Err(e) => assert!(linked && e.contains("AlreadyExists"), "{point:?}: {e}"),
        }
        assert_eq!(fs::read(&path).unwrap(), expected, "{point:?}");
        assert_eq!(Collection::load(&path, pin).unwrap().root(), next.root());
        assert_eq!(fs::read(&previous).unwrap(), previous_bytes, "{point:?}");
        assert_eq!(fs::read(&temporary).unwrap(), held, "{point:?}");
    }
}

#[test]
fn a_reader_at_every_step_of_a_publication_sees_nothing_or_the_whole_file() {
    let expected = next_collection().to_bytes();
    let pin = digest(&expected);
    // The control: the same bytes published with nobody looking.
    let quiet = Scratch::new();
    let unobserved = quiet.0.join(FILE);
    next_collection().save_new(&unobserved).unwrap();

    let scratch = Scratch::new();
    let (target, signals) = scratch.target_and_signals();
    let path = target.join(FILE);
    let vars = [("SIGNALS", signals.display().to_string())];
    let mut writer = child("lockstep", &path, &vars)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    // The first step, a step per piece, the flush, the link and the directory.
    let pieces = expected.len().div_ceil(STEP_BYTES);
    let steps = 1 + pieces + 1 + 1 + 1;
    for step in 0..steps {
        await_signal(&signals, &format!("step-{step}"));
        let linked = step >= steps - 2;
        match Collection::load(&path, pin) {
            Ok(found) => {
                assert!(linked, "step {step}: published before the link");
                assert_eq!(found.to_bytes(), expected, "step {step}");
            }
            Err(e) => assert!(!linked && e.contains("NotFound"), "step {step}: {e}"),
        }
        assert_eq!(
            read_regular(&path, MAX_TEXT).ok().as_deref(),
            linked.then_some(&expected[..]),
            "step {step}"
        );
        signal(&signals, &format!("seen-{step}"));
    }
    assert!(writer.wait().unwrap().success());
    assert_eq!(fs::read(&path).unwrap(), fs::read(&unobserved).unwrap());
    assert_eq!(names(&target), [FILE]);
}

#[test]
fn a_source_restart_meeting_its_own_leftover_names_skips_them_and_then_fails_closed() {
    let scratch = Scratch::new();
    let expected = next_collection().to_bytes();
    // A directory per child: the id of an ended process can come back, and a
    // child under it would meet the names of the one before as its own.
    let run = |name: &str, leftovers: usize| {
        let dir = scratch.directory(name);
        let path = dir.join(FILE);
        let vars = [("LEFTOVERS", leftovers.to_string())];
        let out = child("leftovers", &path, &vars).output().unwrap();
        assert!(out.status.success(), "{leftovers} leftovers: {out:?}");
        (dir, path)
    };

    // Every name taken: the publication fails and nothing is published.
    let (dir, path) = run("every-name-taken", TEMP_RETRIES + 1);
    assert!(!path.exists());
    assert_eq!(names(&dir).len(), TEMP_RETRIES + 1);

    // The last name free: the publication takes it and removes it again.
    let (dir, path) = run("last-name-free", TEMP_RETRIES);
    assert_eq!(fs::read(&path).unwrap(), expected);
    assert_eq!(names(&dir).len(), 1 + TEMP_RETRIES);

    // A fresh process reopens the file; no leftover is taken for a snapshot.
    let loaded = Collection::load(&path, digest(&expected)).unwrap();
    assert_eq!(loaded.to_bytes(), expected);
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
fn loading_a_saved_snapshot_changes_nothing_on_disk() {
    let scratch = Scratch::new();
    let snapshot = scratch.0.join("snapshot");
    let bundle = scratch.0.join("bundle");
    let collection = next_collection();
    let collection_pin = collection.save_new(&snapshot).unwrap();
    let encoded = import_text(b"exact source bytes", "Notes").unwrap();
    let bundle_pin = write_bundle_new(&bundle, &encoded).unwrap();
    let permissions = [freeze(&snapshot), freeze(&bundle)];

    let before = [on_disk(&snapshot), on_disk(&bundle)];
    assert_eq!(
        Collection::load(&snapshot, collection_pin)
            .unwrap()
            .to_bytes(),
        before[0].bytes
    );
    load_bundle(&bundle, bundle_pin).unwrap();
    assert_eq!(read_regular(&snapshot, MAX_TEXT).unwrap(), before[0].bytes);
    // A refused load and a refused save leave the file as it is, and no
    // temporary file behind.
    assert!(Collection::load(&snapshot, bundle_pin).is_err());
    assert!(load_bundle(&bundle, collection_pin).is_err());
    assert!(collection.save_new(&snapshot).is_err());
    assert!(write_bundle_new(&bundle, &encoded).is_err());
    assert_eq!([on_disk(&snapshot), on_disk(&bundle)], before);

    for (path, original) in [&snapshot, &bundle].into_iter().zip(permissions) {
        fs::set_permissions(path, original).unwrap();
    }
}
