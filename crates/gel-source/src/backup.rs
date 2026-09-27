//! Verified backup directories for GELSET01 collection snapshots.
//!
//! A backup is a new directory holding a byte-identical copy of one snapshot
//! and, published last, a MANIFEST naming its length and SHA-256. The manifest
//! is the commit point: a process killed at any earlier moment leaves no
//! manifest, so the backup reads as INCOMPLETE. Nothing is ever appended to a
//! backup afterwards except an optional WITHDRAWN marker, which keeps the bytes
//! but makes restore refuse. The manifest is not a signature: the caller's
//! independently retained snapshot pin decides which backup is trusted.
use crate::{
    bundle::write_bytes_new,
    collection::{Collection, MAX_DOCUMENTS, MAX_TITLE, MAX_TOTAL_TEXT},
    digest, hex, read_regular, BundleError, Hash,
};
use std::{fs, io, path::Path};

pub const SNAPSHOT: &str = "collection.gelset";
pub const MANIFEST: &str = "MANIFEST";
pub const WITHDRAWN: &str = "WITHDRAWN";
/// Same bound as `Collection::load`.
const MAX_SNAPSHOT: usize = MAX_TOTAL_TEXT + MAX_DOCUMENTS * (20 + MAX_TITLE) + 32;
const MAX_MARKER: usize = 4096;
const MAX_REASON: usize = 200;

/// What the files on disk say about one backup directory.
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// The manifest is committed and the snapshot matches it and the pin.
    Complete {
        bytes: usize,
        documents: usize,
        revision: u64,
    },
    /// No committed manifest: creation never finished.
    Incomplete,
    /// A committed manifest names a different snapshot than the trusted pin.
    Untrusted,
    /// Present files contradict each other or the formats.
    Corrupt(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Inspection {
    pub state: State,
    pub withdrawn: bool,
    /// Leftover temporary files of an interrupted publication; never part of a backup.
    pub temporaries: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestoreError {
    /// Nothing was written.
    Refused(String),
    /// Publication failed and the target does not hold the snapshot.
    NotPublished(String),
    /// The complete snapshot is at the target, but a later step (directory
    /// sync or temporary cleanup) failed, so the durability of its name is not
    /// confirmed.
    PublishedUnconfirmed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteError {
    /// Nothing was removed.
    Refused(String),
    /// The file was removed, but syncing its directory failed.
    RemovedUnconfirmed(String),
}

fn io_text(e: &BundleError) -> String {
    match e {
        BundleError::Io(io) => io.to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> io::Result<()> {
    fs::File::open(dir)?.sync_all()
}
#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> io::Result<()> {
    Ok(())
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

fn manifest(bytes: usize, pin: &Hash, documents: usize, revision: u64) -> String {
    format!(
        "GEL_BACKUP=1\nsnapshot={SNAPSHOT}\nbytes={bytes}\nsha256={}\ndocuments={documents}\nrevision={revision}\n",
        hex(pin)
    )
}

/// Parses exactly the six lines `manifest` writes.
fn parse_manifest(text: &str) -> Option<(usize, String, usize, u64)> {
    let mut lines = text.strip_suffix('\n')?.split('\n');
    let mut field = |key: &str| lines.next()?.strip_prefix(key).map(str::to_string);
    if field("GEL_BACKUP=")? != "1" || field("snapshot=")? != SNAPSHOT {
        return None;
    }
    let bytes = field("bytes=")?.parse().ok()?;
    let sha256 = field("sha256=")?;
    let documents = field("documents=")?.parse().ok()?;
    let revision = field("revision=")?.parse().ok()?;
    (sha256.len() == 64 && sha256.bytes().all(|b| b.is_ascii_hexdigit()) && lines.next().is_none())
        .then_some((bytes, sha256, documents, revision))
}

/// Reads a snapshot and accepts it only if it matches the pin and parses.
fn verified_snapshot(path: &Path, pin: Hash) -> Result<(Vec<u8>, Collection), String> {
    let bytes =
        read_regular(path, MAX_SNAPSHOT).map_err(|e| format!("SNAPSHOT_READ {}", io_text(&e)))?;
    if digest(&bytes) != pin {
        return Err("SNAPSHOT_PIN_MISMATCH".into());
    }
    let collection = Collection::from_bytes(&bytes, pin)?;
    Ok((bytes, collection))
}

/// Copies a pinned snapshot into a new backup directory; the manifest is
/// published last and is the only commit point.
pub fn create(snapshot: &Path, pin: Hash, dir: &Path) -> Result<State, String> {
    let (bytes, collection) = verified_snapshot(snapshot, pin)?;
    fs::create_dir(dir).map_err(|e| format!("BACKUP_DIR {e}"))?;
    sync_dir(parent(dir)).map_err(|e| format!("BACKUP_DIR_SYNC {e}"))?;
    write_bytes_new(&dir.join(SNAPSHOT), &bytes)
        .map_err(|e| format!("SNAPSHOT_COPY {}", io_text(&e)))?;
    let documents = collection.documents().count();
    let revision = collection.revision();
    let text = manifest(bytes.len(), &pin, documents, revision);
    write_bytes_new(&dir.join(MANIFEST), text.as_bytes())
        .map_err(|e| format!("MANIFEST {}", io_text(&e)))?;
    Ok(State::Complete {
        bytes: bytes.len(),
        documents,
        revision,
    })
}

fn is_temporary(name: &str) -> bool {
    name.starts_with(".gel-source-") && name.ends_with(".tmp")
}

fn state(dir: &Path, pin: Hash) -> Result<State, String> {
    let manifest_path = dir.join(MANIFEST);
    let text = match read_regular(&manifest_path, MAX_MARKER) {
        Ok(text) => text,
        Err(BundleError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(State::Incomplete)
        }
        Err(e) => return Ok(State::Corrupt(format!("MANIFEST_READ {}", io_text(&e)))),
    };
    let Some((bytes, sha256, documents, revision)) =
        std::str::from_utf8(&text).ok().and_then(parse_manifest)
    else {
        return Ok(State::Corrupt("MANIFEST_FORMAT".into()));
    };
    if sha256 != hex(&pin) {
        return Ok(State::Untrusted);
    }
    match verified_snapshot(&dir.join(SNAPSHOT), pin) {
        Ok((found, collection))
            if found.len() == bytes
                && collection.documents().count() == documents
                && collection.revision() == revision =>
        {
            Ok(State::Complete {
                bytes,
                documents,
                revision,
            })
        }
        Ok(_) => Ok(State::Corrupt("MANIFEST_SNAPSHOT_DISAGREE".into())),
        Err(e) => Ok(State::Corrupt(e)),
    }
}

/// Reports the backup state from the files alone; writes nothing.
pub fn inspect(dir: &Path, pin: Hash) -> Result<Inspection, String> {
    let mut withdrawn = false;
    let mut temporaries = 0;
    let mut unexpected = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("BACKUP_DIR {e}"))? {
        let entry = entry.map_err(|e| format!("BACKUP_DIR {e}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        match name.as_str() {
            SNAPSHOT | MANIFEST => {}
            WITHDRAWN => withdrawn = true,
            n if is_temporary(n) => temporaries += 1,
            _ => unexpected.push(name),
        }
    }
    let state = if unexpected.is_empty() {
        state(dir, pin)?
    } else {
        State::Corrupt(format!("UNEXPECTED_ENTRY {}", unexpected.join(",")))
    };
    Ok(Inspection {
        state,
        withdrawn,
        temporaries,
    })
}

/// Tells "nothing published" apart from "published, name not yet durable"
/// after a failed publication, by what is actually at the target.
fn classify(target: &Path, pin: Hash, error: String) -> RestoreError {
    match read_regular(target, MAX_SNAPSHOT) {
        Ok(bytes) if digest(&bytes) == pin => RestoreError::PublishedUnconfirmed(error),
        _ => RestoreError::NotPublished(error),
    }
}

/// Publishes the backed-up snapshot at a new path; never replaces a file.
pub fn restore(dir: &Path, pin: Hash, target: &Path) -> Result<usize, RestoreError> {
    let refused = RestoreError::Refused;
    let found = inspect(dir, pin).map_err(refused)?;
    if found.withdrawn {
        return Err(RestoreError::Refused("WITHDRAWN".into()));
    }
    match found.state {
        State::Complete { .. } => {}
        State::Incomplete => return Err(RestoreError::Refused("INCOMPLETE".into())),
        State::Untrusted => return Err(RestoreError::Refused("UNTRUSTED".into())),
        State::Corrupt(why) => return Err(RestoreError::Refused(format!("CORRUPT {why}"))),
    }
    if fs::symlink_metadata(target).is_ok() {
        return Err(RestoreError::Refused("TARGET_EXISTS".into()));
    }
    let (bytes, _) = verified_snapshot(&dir.join(SNAPSHOT), pin).map_err(RestoreError::Refused)?;
    match write_bytes_new(target, &bytes) {
        Ok(_) => Ok(bytes.len()),
        Err(e) => Err(classify(target, pin, io_text(&e))),
    }
}

/// Marks a backup as withdrawn. Its bytes stay; restore refuses it.
pub fn withdraw(dir: &Path, reason: &str) -> Result<(), String> {
    if reason.is_empty() || reason.len() > MAX_REASON || reason.chars().any(char::is_control) {
        return Err("REASON_REQUIRED one line, at most 200 bytes".into());
    }
    if !fs::symlink_metadata(dir)
        .map_err(|e| format!("BACKUP_DIR {e}"))?
        .is_dir()
    {
        return Err("BACKUP_DIR not a directory".into());
    }
    let text = format!("GEL_BACKUP_WITHDRAWN=1\nreason={reason}\n");
    match write_bytes_new(&dir.join(WITHDRAWN), text.as_bytes()) {
        Ok(_) => Ok(()),
        Err(BundleError::Io(e)) if e.kind() == io::ErrorKind::AlreadyExists => {
            Err("ALREADY_WITHDRAWN".into())
        }
        Err(e) => Err(format!("WITHDRAW {}", io_text(&e))),
    }
}

/// Removes one pinned file. Other copies, backups and earlier snapshots are
/// untouched, and the storage blocks are not securely erased.
pub fn delete_pinned(path: &Path, pin: Hash) -> Result<usize, DeleteError> {
    let bytes = read_regular(path, MAX_SNAPSHOT)
        .map_err(|e| DeleteError::Refused(format!("READ {}", io_text(&e))))?;
    if digest(&bytes) != pin {
        return Err(DeleteError::Refused("PIN_MISMATCH".into()));
    }
    fs::remove_file(path).map_err(|e| DeleteError::Refused(format!("REMOVE {e}")))?;
    sync_dir(parent(path)).map_err(|e| DeleteError::RemovedUnconfirmed(e.to_string()))?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "gel-backup-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn snapshot(s: &Scratch) -> (PathBuf, Hash) {
        let mut c = Collection::new();
        c.add("a.txt", "RAM is volatile.\nSnapshots can be reopened.")
            .unwrap();
        c.add("b.txt", "Zażółć gęślą jaźń.").unwrap();
        let path = s.0.join("bank.gelset");
        let pin = c.save_new(&path).unwrap();
        (path, pin)
    }

    #[test]
    fn create_inspect_restore_and_reload_by_the_same_pin() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let dir = s.0.join("backup");
        let made = create(&snap, pin, &dir).unwrap();
        let found = inspect(&dir, pin).unwrap();
        assert_eq!(found.state, made);
        assert!(!found.withdrawn && found.temporaries == 0);
        let target = s.0.join("restored.gelset");
        let bytes = restore(&dir, pin, &target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), fs::read(&snap).unwrap());
        assert_eq!(bytes, fs::read(&snap).unwrap().len());
        let reopened = Collection::load(&target, pin).unwrap();
        assert_eq!(reopened.documents().count(), 2);
    }

    #[test]
    fn creation_refuses_a_wrong_pin_and_an_existing_directory() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let mut wrong = pin;
        wrong[0] ^= 1;
        assert!(create(&snap, wrong, &s.0.join("x")).is_err());
        assert!(!s.0.join("x").exists());
        fs::create_dir(s.0.join("taken")).unwrap();
        assert!(create(&snap, pin, &s.0.join("taken")).is_err());
        assert_eq!(fs::read_dir(s.0.join("taken")).unwrap().count(), 0);
    }

    /// Every state a killed creation can leave has no committed manifest, so
    /// it reads as INCOMPLETE and cannot be restored.
    #[test]
    fn every_interrupted_state_is_incomplete_and_refused() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let bytes = fs::read(&snap).unwrap();
        let copy = bytes.as_slice();
        let states: [&[(&str, &[u8])]; 4] = [
            &[],
            &[(".gel-source-1-0.tmp", &b"partial"[..])],
            &[(SNAPSHOT, copy)],
            &[
                (SNAPSHOT, copy),
                (".gel-source-1-1.tmp", &b"GEL_BACKUP=1\n"[..]),
            ],
        ];
        for (n, files) in states.iter().enumerate() {
            let dir = s.0.join(format!("interrupted-{n}"));
            fs::create_dir(&dir).unwrap();
            for (name, content) in files.iter() {
                fs::write(dir.join(name), content).unwrap();
            }
            assert_eq!(
                inspect(&dir, pin).unwrap().state,
                State::Incomplete,
                "state {n}"
            );
            let target = s.0.join(format!("never-{n}"));
            assert_eq!(
                restore(&dir, pin, &target),
                Err(RestoreError::Refused("INCOMPLETE".into()))
            );
            assert!(!target.exists());
        }
    }

    #[test]
    fn tampering_foreign_backups_and_unexpected_files_are_not_restorable() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let dir = s.0.join("backup");
        create(&snap, pin, &dir).unwrap();
        let mut other = pin;
        other[31] ^= 1;
        assert_eq!(inspect(&dir, other).unwrap().state, State::Untrusted);

        let copy = dir.join(SNAPSHOT);
        let mut bytes = fs::read(&copy).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::remove_file(&copy).unwrap();
        fs::write(&copy, &bytes).unwrap();
        assert!(matches!(
            inspect(&dir, pin).unwrap().state,
            State::Corrupt(_)
        ));
        assert!(matches!(
            restore(&dir, pin, &s.0.join("t")),
            Err(RestoreError::Refused(why)) if why.starts_with("CORRUPT")
        ));

        let dir2 = s.0.join("backup2");
        create(&snap, pin, &dir2).unwrap();
        fs::write(dir2.join("extra.txt"), "x").unwrap();
        assert!(matches!(
            inspect(&dir2, pin).unwrap().state,
            State::Corrupt(_)
        ));
    }

    #[test]
    fn restore_never_replaces_an_existing_target() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let dir = s.0.join("backup");
        create(&snap, pin, &dir).unwrap();
        let target = s.0.join("occupied");
        fs::write(&target, "keep me").unwrap();
        assert_eq!(
            restore(&dir, pin, &target),
            Err(RestoreError::Refused("TARGET_EXISTS".into()))
        );
        assert_eq!(fs::read(&target).unwrap(), b"keep me");
    }

    #[test]
    fn a_failed_publication_is_classified_by_what_is_at_the_target() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let absent = s.0.join("absent");
        assert!(matches!(
            classify(&absent, pin, "e".into()),
            RestoreError::NotPublished(_)
        ));
        assert!(matches!(
            classify(&snap, pin, "sync".into()),
            RestoreError::PublishedUnconfirmed(e) if e == "sync"
        ));
        let other = s.0.join("other");
        fs::write(&other, "different").unwrap();
        assert!(matches!(
            classify(&other, pin, "e".into()),
            RestoreError::NotPublished(_)
        ));
    }

    #[test]
    fn withdrawn_backups_keep_their_bytes_and_refuse_restore() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let dir = s.0.join("backup");
        create(&snap, pin, &dir).unwrap();
        assert!(withdraw(&dir, "").is_err());
        assert!(withdraw(&dir, "two\nlines").is_err());
        withdraw(&dir, "contains a superseded document").unwrap();
        assert_eq!(withdraw(&dir, "again"), Err("ALREADY_WITHDRAWN".into()));
        let found = inspect(&dir, pin).unwrap();
        assert!(found.withdrawn);
        assert!(matches!(found.state, State::Complete { .. }));
        assert_eq!(
            restore(&dir, pin, &s.0.join("t")),
            Err(RestoreError::Refused("WITHDRAWN".into()))
        );
        assert_eq!(
            fs::read(dir.join(SNAPSHOT)).unwrap(),
            fs::read(&snap).unwrap()
        );
    }

    #[test]
    fn delete_removes_only_the_pinned_file() {
        let s = Scratch::new();
        let (snap, pin) = snapshot(&s);
        let dir = s.0.join("backup");
        create(&snap, pin, &dir).unwrap();
        let mut wrong = pin;
        wrong[5] ^= 1;
        assert_eq!(
            delete_pinned(&snap, wrong),
            Err(DeleteError::Refused("PIN_MISMATCH".into()))
        );
        assert!(snap.exists());
        delete_pinned(&snap, pin).unwrap();
        assert!(!snap.exists());
        // The backup of the deleted snapshot is untouched and still restorable.
        assert!(matches!(
            inspect(&dir, pin).unwrap().state,
            State::Complete { .. }
        ));
        restore(&dir, pin, &s.0.join("back")).unwrap();
    }

    #[test]
    fn the_manifest_parser_accepts_only_its_own_format() {
        let pin = [7u8; 32];
        let good = manifest(10, &pin, 2, 3);
        assert_eq!(parse_manifest(&good).map(|m| m.0), Some(10));
        for bad in [
            good.trim_end().to_string(),
            good.replace("GEL_BACKUP=1", "GEL_BACKUP=2"),
            good.replace("snapshot=collection.gelset", "snapshot=../x"),
            format!("{good}extra=1\n"),
            good.replace(&hex(&pin), "zz"),
        ] {
            assert_eq!(parse_manifest(&bad), None, "{bad}");
        }
    }
}
