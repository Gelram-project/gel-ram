//! A save of one Q8 record, the file `gel-live-lab --literal FILE --save
//! NEW_PATH` writes, stopped at known points. A child process (this test binary
//! run again) publishes the record through `gel_source::write_file_new_with`
//! with I/O steps that write 64 bytes at a time and end the process at one
//! point: nothing after that point runs, no destructor and no cleanup. The new
//! path must then hold nothing or the whole record under its pin, never part of
//! it. A naive writer stopped at the same points must leave part of a record,
//! so the check is shown to see one. Ending a process is not a power cut.
use gel_live_lab::literal;
use gel_phase_quad::fixture;
use gel_source::{digest, write_file_new, write_file_new_with, BundleError, Hash, PublicationIo};
use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const CHUNK: usize = 64;
const MARKER: &str = "TORN_POINT_REACHED";
/// The exit status of a child that reached its point.
const STOPPED: i32 = 86;
const TARGET: &str = "record.q8";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Point {
    /// After this many 64-byte pieces of the temporary file.
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
            Point::Written(i) => format!("written-{i}"),
            Point::Synced => "synced".into(),
            Point::Linked => "linked".into(),
            Point::ParentSynced => "parent-synced".into(),
        }
    }
    fn parse(name: &str) -> Option<Self> {
        match name {
            "synced" => Some(Point::Synced),
            "linked" => Some(Point::Linked),
            "parent-synced" => Some(Point::ParentSynced),
            _ => name
                .strip_prefix("written-")?
                .parse()
                .ok()
                .map(Point::Written),
        }
    }
}

/// One literal record, the same bytes every run.
fn record_bytes() -> Vec<u8> {
    let text = "Zażółć gęślą jaźń: one record, the same bytes every run. ".repeat(20);
    let lit = literal::from_bytes(text.as_bytes()).unwrap();
    literal::encode(lit.record()).unwrap()
}

/// Every piece boundary of the temporary file, then the three later steps.
fn points(len: usize) -> Vec<Point> {
    (0..=len.div_ceil(CHUNK))
        .map(Point::Written)
        .chain([Point::Synced, Point::Linked, Point::ParentSynced])
        .collect()
}

fn stop(at: Point) -> ! {
    println!("{MARKER} {}", at.name());
    let _ = std::io::stdout().flush();
    std::process::exit(STOPPED)
}

/// Real I/O, 64 bytes at a time, ending the process at one point.
struct StopAt(Point);
impl PublicationIo for StopAt {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        if self.0 == Point::Written(0) {
            stop(self.0);
        }
        for (i, piece) in bytes.chunks(CHUNK).enumerate() {
            file.write_all(piece)?;
            if self.0 == Point::Written(i + 1) {
                stop(self.0);
            }
        }
        Ok(())
    }
    fn sync(&mut self, file: &File) -> std::io::Result<()> {
        file.sync_all()?;
        if self.0 == Point::Synced {
            stop(self.0);
        }
        Ok(())
    }
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        fs::hard_link(tmp, path)?;
        if self.0 == Point::Linked {
            stop(self.0);
        }
        Ok(())
    }
    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = parent;
        if self.0 == Point::ParentSynced {
            stop(self.0);
        }
        Ok(())
    }
}

/// The control: the record written straight to the new path, the same pieces.
fn naive(path: &Path, bytes: &[u8], at: Point) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    if at == Point::Written(0) {
        stop(at);
    }
    for (i, piece) in bytes.chunks(CHUNK).enumerate() {
        file.write_all(piece)?;
        if at == Point::Written(i + 1) {
            stop(at);
        }
    }
    file.sync_all()?;
    if at == Point::Synced {
        stop(at);
    }
    Ok(())
}

/// The child process, started by the other tests of this file; ignored in a
/// normal run, and it does nothing unless a parent test sets its variables.
#[test]
#[ignore]
fn torn_child() {
    let (Ok(point), Ok(path), Ok(writer)) = (
        std::env::var("GEL_TORN_POINT"),
        std::env::var("GEL_TORN_PATH"),
        std::env::var("GEL_TORN_WRITER"),
    ) else {
        return;
    };
    let point = Point::parse(&point).expect("a known point");
    let bytes = record_bytes();
    let path = Path::new(&path);
    if writer == "library" {
        let _ = write_file_new_with(path, &bytes, &mut StopAt(point));
    } else {
        let _ = naive(path, &bytes, point);
    }
    // Reaching this line means the point was never reached; the parent fails.
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let p = std::env::temp_dir().join(format!(
                "gel-torn-write-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs the child; true when it printed the marker of `point` and exited with `STOPPED`.
fn stopped_at(dir: &Path, point: Point, writer: &str) -> bool {
    let out = Command::new(std::env::current_exe().unwrap())
        .args([
            "torn_child",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("GEL_TORN_POINT", point.name())
        .env("GEL_TORN_PATH", dir.join(TARGET))
        .env("GEL_TORN_WRITER", writer)
        .output()
        .unwrap();
    let marker = format!("{MARKER} {}", point.name());
    let printed = String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.trim_end().ends_with(&marker));
    printed && out.status.code() == Some(STOPPED)
}

#[derive(Debug, PartialEq)]
enum State {
    /// No file at the new path.
    Before,
    /// The whole record, under its pin, reopened as gel-live-lab --reopen does.
    After,
    /// Anything else.
    Partial,
}

fn state(path: &Path, bytes: &[u8], pin: &Hash) -> State {
    match fs::read(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => State::Before,
        Err(e) => panic!("{}: {e}", path.display()),
        Ok(found) if found == bytes && literal::reopen(path, pin).is_ok() => State::After,
        Ok(_) => State::Partial,
    }
}

/// Sizes of the files other than the new path, in the scratch directory.
fn temporaries(dir: &Path) -> Vec<u64> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.file_name() != TARGET)
        .map(|e| e.metadata().unwrap().len())
        .collect()
}

#[test]
fn a_save_stopped_at_any_known_point_leaves_nothing_or_the_whole_record() {
    let bytes = record_bytes();
    let pin = digest(&bytes);
    assert_eq!(bytes.len(), literal::SAVED_BYTES);
    let points = points(bytes.len());
    assert_eq!(points.len(), 23);
    for point in points {
        let dir = Scratch::new();
        let path = dir.0.join(TARGET);
        assert!(
            stopped_at(&dir.0, point, "library"),
            "{point:?}: the child did not stop there"
        );
        let expected = match point {
            Point::Written(_) | Point::Synced => State::Before,
            Point::Linked | Point::ParentSynced => State::After,
        };
        assert_eq!(state(&path, &bytes, &pin), expected, "{point:?}");
        // The temporary file stays, holding what was written before the stop.
        let written = match point {
            Point::Written(i) => (CHUNK * i).min(bytes.len()),
            _ => bytes.len(),
        } as u64;
        assert_eq!(temporaries(&dir.0), vec![written], "{point:?}");
        // A fresh save: the path is free before, and never replaced after.
        match (expected, write_file_new(&path, &bytes)) {
            (State::Before, Ok(saved)) => assert_eq!(saved, pin),
            (State::After, Err(BundleError::Io(e))) => {
                assert_eq!(e.kind(), ErrorKind::AlreadyExists, "{point:?}")
            }
            (_, other) => panic!("{point:?}: unexpected fresh save {other:?}"),
        }
        assert_eq!(state(&path, &bytes, &pin), State::After, "{point:?}");
    }
}

#[test]
fn a_naive_writer_stopped_at_the_same_points_leaves_part_of_a_record() {
    let bytes = record_bytes();
    let pin = digest(&bytes);
    let mut partial = 0;
    for point in points(bytes.len())
        .into_iter()
        .filter(|p| matches!(p, Point::Written(_) | Point::Synced))
    {
        let dir = Scratch::new();
        assert!(
            stopped_at(&dir.0, point, "naive"),
            "{point:?}: the child did not stop there"
        );
        match state(&dir.0.join(TARGET), &bytes, &pin) {
            State::Partial => partial += 1,
            State::After => {}
            State::Before => panic!("{point:?}: the naive writer creates the path first"),
        }
    }
    assert_eq!(partial, 19);
}

#[test]
fn cut_files_are_refused_and_a_zeroed_block_needs_the_pin() {
    let bytes = record_bytes();
    // Header (8-byte magic and a 4-byte count), then 1024 values and a 128-byte mask.
    for cut in [0, 8, 12, 12 + 1024, bytes.len() - 1] {
        assert!(fixture::decode(&bytes[..cut]).is_err(), "cut at {cut}");
    }
    let mut longer = bytes.clone();
    longer.push(0);
    assert!(fixture::decode(&longer).is_err());

    let records: Vec<_> = (0..8)
        .map(|k| {
            let text = format!("record {k} of eight. ").repeat(60);
            literal::from_bytes(text.as_bytes())
                .unwrap()
                .record()
                .clone()
        })
        .collect();
    let file = fixture::encode(&records).unwrap();
    assert_eq!(file.len(), 12 + 8 * fixture::RECORD_BYTES);
    let mut zeroed = file.clone();
    zeroed[4096..8192].fill(0);
    // Every byte is a valid value or mask byte, so the zeroed file still decodes;
    // only the pin kept with the file tells it apart.
    assert_eq!(fixture::decode(&zeroed).unwrap().len(), 8);
    assert_ne!(digest(&zeroed), digest(&file));
}
