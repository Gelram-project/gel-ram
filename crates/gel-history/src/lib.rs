#![forbid(unsafe_code)]

//! Exact history of one 128-byte ORB128 record.
//!
//! Each appended state is stored as a literal copy or as the XOR residual from
//! the state before it, at most two residuals from a literal; every stored state
//! is rebuilt bit for bit. Storage grows with every append.
//!
//! A saved history is one GELHIS01 file: a 48-byte header (magic, version,
//! reserved field, entry count, payload length, payload CRC64, header CRC64)
//! followed by the entries. A literal entry takes 129 bytes. A residual entry
//! takes 10 bytes (tag, parent index, depth and a 4-byte length) plus the
//! serialized length of its sparse [`Residual`], that is 13 + ceil(10k/8) bytes
//! for k changed bits. The decoder accepts only the bytes the encoder writes;
//! see `docs/RECORD-HISTORY.md` for the contract and its limits.

use gel_core::{crc64_ecma, GelError, ORB_BYTES};
use gel_orb::Orb1024;
use gel_structural::Residual;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const MAGIC: [u8; 8] = *b"GELHIS01";
const VERSION: u32 = 1;
const HEADER_BYTES: usize = 48;
const LITERAL_TAG: u8 = 0;
const RESIDUAL_TAG: u8 = 1;
const SPARSE_FORM: u8 = 0;
const DENSE_FORM: u8 = 1;
/// Tag and state of a literal entry.
const LITERAL_ENTRY_BYTES: usize = 1 + ORB_BYTES;
/// Tag, parent index, depth and the 4-byte length of the packed positions; the
/// residual's own serialized bytes follow.
const RESIDUAL_ENTRY_EXTRA_BYTES: usize = 10;
/// The smallest entry: a residual with no changed bit.
const MIN_ENTRY_BYTES: u64 = 13;

/// The most residuals between a stored state and its literal.
pub const MAX_RESIDUAL_DEPTH: u8 = 2;

/// One stored state of a [`RecordHistory`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryEntry {
    /// The state itself.
    Literal(Orb1024),
    /// The XOR residual from the entry at `parent`, `depth` residuals from a literal.
    Residual {
        parent: u32,
        depth: u8,
        residual: Residual,
    },
}

/// Every state appended to one record, in order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordHistory {
    entries: Vec<HistoryEntry>,
}

impl RecordHistory {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Appends an exact state and returns its index.
    ///
    /// The state is stored as the residual from the state before it when that
    /// entry is smaller than a literal and at most [`MAX_RESIDUAL_DEPTH`]
    /// residuals from a literal; otherwise it is stored as a literal.
    pub fn append(&mut self, state: Orb1024) -> Result<usize, GelError> {
        let entry = self.next_entry(self.entries.len(), state)?;
        self.entries.push(entry);
        Ok(self.entries.len() - 1)
    }

    /// Rebuilds the state at `index` from its literal and at most
    /// [`MAX_RESIDUAL_DEPTH`] residuals.
    pub fn reconstruct(&self, index: usize) -> Result<Orb1024, GelError> {
        // An iterative walk with a hard hop bound, so no entry can make it recurse
        // or walk further than the depth limit.
        let mut chain: Vec<&Residual> = Vec::with_capacity(MAX_RESIDUAL_DEPTH as usize);
        let mut cursor = index;
        let mut state = loop {
            let entry = self
                .entries
                .get(cursor)
                .ok_or(GelError::InvalidResidual("index outside history"))?;
            match entry {
                HistoryEntry::Literal(state) => break *state,
                HistoryEntry::Residual {
                    parent, residual, ..
                } => {
                    let parent = *parent as usize;
                    if parent >= cursor {
                        return Err(GelError::InvalidResidual("parent must precede child"));
                    }
                    if chain.len() == MAX_RESIDUAL_DEPTH as usize {
                        return Err(GelError::InvalidResidual(
                            "residual chain longer than MAX_RESIDUAL_DEPTH",
                        ));
                    }
                    chain.push(residual);
                    cursor = parent;
                }
            }
        };
        for residual in chain.iter().rev() {
            state = residual.apply(&state)?;
        }
        Ok(state)
    }

    /// The last appended state, or `None` for an empty history.
    pub fn latest(&self) -> Result<Option<Orb1024>, GelError> {
        self.entries
            .len()
            .checked_sub(1)
            .map(|index| self.reconstruct(index))
            .transpose()
    }

    /// Every stored state, rebuilt in order.
    pub fn exact_history(&self) -> Result<Vec<Orb1024>, GelError> {
        (0..self.entries.len())
            .map(|index| self.reconstruct(index))
            .collect()
    }

    /// Length in bytes of [`RecordHistory::to_bytes`], header included.
    pub fn encoded_len(&self) -> usize {
        HEADER_BYTES + self.entries.iter().map(entry_len).sum::<usize>()
    }

    /// The GELHIS01 bytes of this history.
    pub fn to_bytes(&self) -> Result<Vec<u8>, GelError> {
        let mut out = Vec::with_capacity(self.encoded_len());
        out.extend_from_slice(&[0u8; HEADER_BYTES]);
        encode_entries(&self.entries, &mut out)?;
        let payload = &out[HEADER_BYTES..];
        let header = encode_header(
            self.entries.len() as u64,
            payload.len() as u64,
            crc64_ecma(payload),
        );
        out[..HEADER_BYTES].copy_from_slice(&header);
        Ok(out)
    }

    /// Reads GELHIS01 bytes.
    ///
    /// The caller's entry and payload limits are checked before any allocation.
    /// A wrong magic, version or reserved field, either CRC64 mismatch, a length
    /// that differs from the header, trailing bytes, an entry that cannot be
    /// rebuilt, and any bytes other than those [`RecordHistory::to_bytes`]
    /// writes for the same states are refused.
    pub fn from_bytes(
        bytes: &[u8],
        max_records: u64,
        max_payload_bytes: u64,
    ) -> Result<Self, GelError> {
        let header: &[u8; HEADER_BYTES] = bytes
            .get(..HEADER_BYTES)
            .and_then(|header| header.try_into().ok())
            .ok_or(GelError::InvalidHeader("history file shorter than header"))?;
        let (record_count, payload_len, payload_crc) =
            check_header(header, max_records, max_payload_bytes)?;
        let payload = &bytes[HEADER_BYTES..];
        if payload.len() as u64 != payload_len {
            return Err(GelError::InvalidLength {
                expected: usize::try_from(payload_len)
                    .map_or(usize::MAX, |len| len.saturating_add(HEADER_BYTES)),
                actual: bytes.len(),
            });
        }
        if crc64_ecma(payload) != payload_crc {
            return Err(GelError::CorruptStore);
        }
        let history = Self {
            entries: decode_entries(payload, record_count)?,
        };
        // Canonical form: every entry must be the one `append` writes for the
        // same states, so one history has exactly one file.
        for index in 0..history.len() {
            let state = history.reconstruct(index)?;
            if history.entries[index] != history.next_entry(index, state)? {
                return Err(GelError::InvalidResidual(
                    "history file is not in canonical form",
                ));
            }
        }
        Ok(history)
    }

    /// Writes the whole history to a sibling temporary file, flushes it and
    /// renames it over `path`; on Unix the directory is then flushed too.
    ///
    /// The temporary file is `<path>.tmp-<pid>-<n>`, created exclusively; a name
    /// that already exists is skipped. On Unix a new file has mode `0600` and a
    /// replaced file keeps its mode. An error from the final directory flush
    /// means the file at `path` has already been replaced.
    pub fn write_atomic(&self, path: impl AsRef<Path>) -> Result<(), GelError> {
        let path = path.as_ref();
        let bytes = self.to_bytes()?;
        let (tmp, mut file) = create_temp_file(path)?;
        let written = file.write_all(&bytes).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = written.and_then(|()| fs::rename(&tmp, path)) {
            let _ = fs::remove_file(&tmp);
            return Err(error.into());
        }
        sync_parent(path)
    }

    /// Opens a saved history with [`RecordHistory::from_bytes`].
    ///
    /// The header is read and the limits are checked before the payload is
    /// allocated; one byte more than the header declares is read, so a longer
    /// file is refused.
    pub fn open_verified(
        path: impl AsRef<Path>,
        max_records: u64,
        max_payload_bytes: u64,
    ) -> Result<Self, GelError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::with_capacity(HEADER_BYTES);
        (&mut file)
            .take(HEADER_BYTES as u64)
            .read_to_end(&mut bytes)?;
        if let Ok(header) = <&[u8; HEADER_BYTES]>::try_from(bytes.as_slice()) {
            let (_, payload_len, _) = check_header(header, max_records, max_payload_bytes)?;
            let payload = usize::try_from(payload_len)
                .map_err(|_| GelError::LimitExceeded("history payload address space"))?;
            bytes
                .try_reserve_exact(payload.saturating_add(1))
                .map_err(|_| GelError::AllocationFailed)?;
            file.take(payload_len.saturating_add(1))
                .read_to_end(&mut bytes)?;
        }
        Self::from_bytes(&bytes, max_records, max_payload_bytes)
    }

    /// The entry `append` writes for `state` after the first `upto` entries.
    fn next_entry(&self, upto: usize, state: Orb1024) -> Result<HistoryEntry, GelError> {
        let Some(parent) = upto.checked_sub(1) else {
            return Ok(HistoryEntry::Literal(state));
        };
        let parent_depth = self
            .entries
            .get(parent)
            .map(entry_depth)
            .ok_or(GelError::InvalidResidual("index outside history"))?;
        let residual = Residual::from_exact_xor(&state, &self.reconstruct(parent)?);
        let depth = parent_depth + 1;
        if depth > MAX_RESIDUAL_DEPTH || residual_entry_len(&residual) >= LITERAL_ENTRY_BYTES {
            return Ok(HistoryEntry::Literal(state));
        }
        Ok(HistoryEntry::Residual {
            parent: u32::try_from(parent)
                .map_err(|_| GelError::LimitExceeded("history parent index exceeds u32"))?,
            depth,
            residual,
        })
    }
}

fn entry_depth(entry: &HistoryEntry) -> u8 {
    match entry {
        HistoryEntry::Literal(_) => 0,
        HistoryEntry::Residual { depth, .. } => *depth,
    }
}

/// Bytes of a residual entry; one source for the size decision and the file length.
fn residual_entry_len(residual: &Residual) -> usize {
    RESIDUAL_ENTRY_EXTRA_BYTES + residual.serialized_len()
}

fn entry_len(entry: &HistoryEntry) -> usize {
    match entry {
        HistoryEntry::Literal(_) => LITERAL_ENTRY_BYTES,
        HistoryEntry::Residual { residual, .. } => residual_entry_len(residual),
    }
}

fn encode_entries(entries: &[HistoryEntry], out: &mut Vec<u8>) -> Result<(), GelError> {
    for entry in entries {
        match entry {
            HistoryEntry::Literal(state) => {
                out.push(LITERAL_TAG);
                out.extend_from_slice(&state.to_le_bytes());
            }
            HistoryEntry::Residual {
                parent,
                depth,
                residual,
            } => {
                let Residual::Sparse {
                    count,
                    packed_positions,
                } = residual
                else {
                    return Err(GelError::InvalidResidual(
                        "dense residual is not written by this format",
                    ));
                };
                out.push(RESIDUAL_TAG);
                out.extend_from_slice(&parent.to_le_bytes());
                out.push(*depth);
                out.push(SPARSE_FORM);
                out.extend_from_slice(&count.to_le_bytes());
                out.extend_from_slice(&(packed_positions.len() as u32).to_le_bytes());
                out.extend_from_slice(packed_positions);
            }
        }
    }
    Ok(())
}

/// Header fields, the caller's limits and the smallest possible payload for
/// the declared count; all before anything is allocated.
fn check_header(
    header: &[u8; HEADER_BYTES],
    max_records: u64,
    max_payload_bytes: u64,
) -> Result<(u64, u64, u64), GelError> {
    let (record_count, payload_len, payload_crc) = decode_header(header)?;
    if record_count > max_records {
        return Err(GelError::LimitExceeded("history record count"));
    }
    if payload_len > max_payload_bytes {
        return Err(GelError::LimitExceeded("history payload bytes"));
    }
    if record_count > payload_len / MIN_ENTRY_BYTES {
        return Err(GelError::InvalidHeader("history count/payload mismatch"));
    }
    Ok((record_count, payload_len, payload_crc))
}

fn decode_entries(payload: &[u8], record_count: u64) -> Result<Vec<HistoryEntry>, GelError> {
    let count = usize::try_from(record_count)
        .map_err(|_| GelError::LimitExceeded("history record address space"))?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(count)
        .map_err(|_| GelError::AllocationFailed)?;
    let mut cursor = 0usize;
    while entries.len() < count {
        let entry = match take_array::<1>(payload, &mut cursor)?[0] {
            LITERAL_TAG => HistoryEntry::Literal(Orb1024::from_le_bytes(take(
                payload,
                &mut cursor,
                ORB_BYTES,
            )?)?),
            RESIDUAL_TAG => decode_residual(payload, &mut cursor, &entries)?,
            _ => return Err(GelError::InvalidResidual("unknown history entry tag")),
        };
        entries.push(entry);
    }
    if cursor != payload.len() {
        return Err(GelError::InvalidLength {
            expected: cursor,
            actual: payload.len(),
        });
    }
    Ok(entries)
}

/// One residual entry after its tag. The depth is counted from the chain the
/// earlier entries form, never taken from the depth byte alone.
fn decode_residual(
    payload: &[u8],
    cursor: &mut usize,
    earlier: &[HistoryEntry],
) -> Result<HistoryEntry, GelError> {
    let parent = u32::from_le_bytes(take_array(payload, cursor)?);
    let [depth] = take_array::<1>(payload, cursor)?;
    let parent_depth = earlier
        .get(parent as usize)
        .map(entry_depth)
        .ok_or(GelError::InvalidResidual("parent must precede child"))?;
    if depth != parent_depth + 1 {
        return Err(GelError::InvalidResidual(
            "residual depth does not continue its parent chain",
        ));
    }
    if depth > MAX_RESIDUAL_DEPTH {
        return Err(GelError::InvalidResidual(
            "residual chain longer than MAX_RESIDUAL_DEPTH",
        ));
    }
    let residual = match take_array::<1>(payload, cursor)?[0] {
        SPARSE_FORM => {
            let count = u16::from_le_bytes(take_array(payload, cursor)?);
            let len = u32::from_le_bytes(take_array(payload, cursor)?) as usize;
            Residual::Sparse {
                count,
                packed_positions: take(payload, cursor, len)?.to_vec(),
            }
        }
        DENSE_FORM => {
            return Err(GelError::InvalidResidual(
                "dense residual is not written by this format",
            ))
        }
        _ => return Err(GelError::InvalidResidual("unknown residual form")),
    };
    Ok(HistoryEntry::Residual {
        parent,
        depth,
        residual,
    })
}

fn encode_header(record_count: u64, payload_len: u64, payload_crc: u64) -> [u8; HEADER_BYTES] {
    let mut out = [0u8; HEADER_BYTES];
    out[0..8].copy_from_slice(&MAGIC);
    out[8..12].copy_from_slice(&VERSION.to_le_bytes());
    out[12..16].copy_from_slice(&0u32.to_le_bytes());
    out[16..24].copy_from_slice(&record_count.to_le_bytes());
    out[24..32].copy_from_slice(&payload_len.to_le_bytes());
    out[32..40].copy_from_slice(&payload_crc.to_le_bytes());
    let header_crc = crc64_ecma(&out[..40]);
    out[40..48].copy_from_slice(&header_crc.to_le_bytes());
    out
}

/// Magic, header CRC64, version and reserved field, in that order.
fn decode_header(header: &[u8; HEADER_BYTES]) -> Result<(u64, u64, u64), GelError> {
    if header[..8] != MAGIC {
        return Err(GelError::InvalidMagic);
    }
    if crc64_ecma(&header[..40]) != le_u64(header, 40) {
        return Err(GelError::CorruptHeader);
    }
    let version = le_u32(header, 8);
    if version != VERSION {
        return Err(GelError::UnsupportedVersion(version));
    }
    if le_u32(header, 12) != 0 {
        return Err(GelError::InvalidHeader(
            "history reserved field must be zero",
        ));
    }
    Ok((le_u64(header, 16), le_u64(header, 24), le_u64(header, 32)))
}

fn le_u32(header: &[u8; HEADER_BYTES], at: usize) -> u32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&header[at..at + 4]);
    u32::from_le_bytes(word)
}

fn le_u64(header: &[u8; HEADER_BYTES], at: usize) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&header[at..at + 8]);
    u64::from_le_bytes(word)
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], GelError> {
    let out = cursor
        .checked_add(len)
        .and_then(|end| bytes.get(*cursor..end))
        .ok_or(GelError::InvalidResidual("truncated history payload"))?;
    *cursor += len;
    Ok(out)
}

fn take_array<const N: usize>(bytes: &[u8], cursor: &mut usize) -> Result<[u8; N], GelError> {
    let mut out = [0u8; N];
    out.copy_from_slice(take(bytes, cursor, N)?);
    Ok(out)
}

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_path(path: &Path, sequence: u64) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}-{sequence}", std::process::id()));
    PathBuf::from(tmp)
}

fn create_temp_file(path: &Path) -> Result<(PathBuf, File), GelError> {
    #[cfg(unix)]
    let existing_mode = match fs::metadata(path) {
        Ok(metadata) => Some(metadata.permissions().mode() & 0o7777),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };

    for _ in 0..128 {
        let tmp = temp_path(path, TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&tmp) {
            Ok(file) => {
                #[cfg(unix)]
                if let Some(mode) = existing_mode {
                    if let Err(error) = file.set_permissions(fs::Permissions::from_mode(mode)) {
                        drop(file);
                        let _ = fs::remove_file(&tmp);
                        return Err(error.into());
                    }
                }
                return Ok((tmp, file));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(GelError::Io(
        "could not allocate a unique history temporary path".into(),
    ))
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> Result<(), GelError> {
    if let Some(parent) = path.parent() {
        let parent = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> Result<(), GelError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gel_core::{splitmix64, ORB_WORDS};

    fn sample(seed: u64) -> Orb1024 {
        let mut words = [0u64; ORB_WORDS];
        for (i, word) in words.iter_mut().enumerate() {
            *word = splitmix64(seed.wrapping_add(i as u64));
        }
        Orb1024::from_words(words)
    }

    fn flip(mut state: Orb1024, bits: &[usize]) -> Orb1024 {
        for &bit in bits {
            state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
        }
        state
    }

    fn residual_entry(parent: u32, depth: u8, from: Orb1024, to: Orb1024) -> HistoryEntry {
        HistoryEntry::Residual {
            parent,
            depth,
            residual: Residual::from_exact_xor(&to, &from),
        }
    }

    #[test]
    fn the_residual_depth_limit_equals_the_structural_codec_limit() {
        assert_eq!(MAX_RESIDUAL_DEPTH, 2);
        assert_eq!(MAX_RESIDUAL_DEPTH, gel_structural::MAX_DELTA_DEPTH);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn an_entry_in_memory_takes_256_bytes_on_64_bit_targets() {
        assert_eq!(std::mem::size_of::<HistoryEntry>(), 256);
        assert_eq!(std::mem::size_of::<Orb1024>(), ORB_BYTES);
    }

    // Histories built here break the invariants `append` and `from_bytes` keep;
    // the walk must still stop within its hop bound and say why.
    #[test]
    fn rebuilding_stops_at_the_hop_bound_even_when_entries_break_it() {
        let s0 = sample(1);
        let s1 = flip(s0, &[1]);
        let s2 = flip(s1, &[2]);
        let s3 = flip(s2, &[3]);
        let history = RecordHistory {
            entries: vec![
                HistoryEntry::Literal(s0),
                residual_entry(0, 1, s0, s1),
                residual_entry(1, 2, s1, s2),
                residual_entry(2, 3, s2, s3),
            ],
        };
        assert_eq!(history.reconstruct(2), Ok(s2));
        assert_eq!(
            history.reconstruct(3),
            Err(GelError::InvalidResidual(
                "residual chain longer than MAX_RESIDUAL_DEPTH"
            ))
        );
        assert_eq!(
            history.reconstruct(4),
            Err(GelError::InvalidResidual("index outside history"))
        );
    }

    #[test]
    fn rebuilding_refuses_a_parent_that_does_not_precede_its_child() {
        let s0 = sample(2);
        let s1 = flip(s0, &[7]);
        for parent in [1, 2] {
            let history = RecordHistory {
                entries: vec![
                    HistoryEntry::Literal(s0),
                    residual_entry(parent, 1, s0, s1),
                    HistoryEntry::Literal(s1),
                ],
            };
            assert_eq!(
                history.reconstruct(1),
                Err(GelError::InvalidResidual("parent must precede child")),
                "parent {parent}"
            );
        }
    }

    #[test]
    fn a_dense_residual_is_never_encoded() {
        let s0 = sample(3);
        let history = RecordHistory {
            entries: vec![
                HistoryEntry::Literal(s0),
                HistoryEntry::Residual {
                    parent: 0,
                    depth: 1,
                    residual: Residual::Dense(sample(4)),
                },
            ],
        };
        assert_eq!(
            history.to_bytes(),
            Err(GelError::InvalidResidual(
                "dense residual is not written by this format"
            ))
        );
    }

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    // The only test in this file that writes files, so no other test moves the
    // temporary-name sequence while it runs.
    #[test]
    fn a_leftover_temporary_file_does_not_block_saving() {
        let dir = TempDir(std::env::temp_dir().join(format!(
            "gel-history-leftover-{}-{}",
            std::process::id(),
            splitmix64(5)
        )));
        fs::create_dir_all(&dir.0).unwrap();
        let path = dir.0.join("record.gelhis");
        let next = TEMP_SEQUENCE.load(Ordering::Relaxed);
        let leftover = temp_path(&path, next);
        fs::write(&leftover, b"left by an earlier run").unwrap();

        let mut history = RecordHistory::new();
        history.append(sample(6)).unwrap();
        history.write_atomic(&path).unwrap();

        // One name was taken, so the sequence moved by two: the skip and the write.
        assert_eq!(TEMP_SEQUENCE.load(Ordering::Relaxed), next + 2);
        assert_eq!(fs::read(&leftover).unwrap(), b"left by an earlier run");
        assert_eq!(fs::read(&path).unwrap(), history.to_bytes().unwrap());
        let mut names: Vec<_> = fs::read_dir(&dir.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        let leftover_name = leftover.file_name().unwrap().to_owned();
        assert_eq!(names, ["record.gelhis".into(), leftover_name]);
    }
}
