//! The earlier `backup::restore` path, which verified the snapshot during
//! `inspect` and then read, hashed and parsed it a second time before
//! publishing it. The function bodies are kept verbatim so the current
//! `restore` can be compared with them. This file is compiled into the
//! crate's unit tests (as a child of `backup`) and, through `#[path]`, into
//! the `identity_backup_restore` example; in both places the parent module
//! provides every name imported from `super`.

use super::{
    digest, hex, read_regular, write_bytes_new_pinned, BundleError, Collection, Hash, Inspection,
    RestoreError, State, MANIFEST, MAX_MARKER, MAX_SNAPSHOT, SNAPSHOT, WITHDRAWN,
};
use std::{fs, io, path::Path};

fn io_text(e: &BundleError) -> String {
    match e {
        BundleError::Io(io) => io.to_string(),
        other => format!("{other:?}"),
    }
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

/// What a backup needs from a parsed snapshot.
struct Snapshot {
    bytes: Vec<u8>,
    documents: usize,
    revision: u64,
}

/// Reads a snapshot and accepts it only if it matches the pin and parses.
/// The bytes are hashed once; the parse accepts and refuses exactly what
/// `Collection::from_bytes` does for them, with the same error.
fn verified_snapshot(path: &Path, pin: Hash) -> Result<Snapshot, String> {
    let bytes =
        read_regular(path, MAX_SNAPSHOT).map_err(|e| format!("SNAPSHOT_READ {}", io_text(&e)))?;
    if digest(&bytes) != pin {
        return Err("SNAPSHOT_PIN_MISMATCH".into());
    }
    let (documents, revision) = Collection::verified_shape(&bytes)?;
    Ok(Snapshot {
        bytes,
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
        Ok(found)
            if found.bytes.len() == bytes
                && found.documents == documents
                && found.revision == revision =>
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
    let Snapshot { bytes, .. } =
        verified_snapshot(&dir.join(SNAPSHOT), pin).map_err(RestoreError::Refused)?;
    match write_bytes_new_pinned(target, &bytes, pin) {
        Ok(_) => Ok(bytes.len()),
        Err(e) => Err(classify(target, pin, io_text(&e))),
    }
}
