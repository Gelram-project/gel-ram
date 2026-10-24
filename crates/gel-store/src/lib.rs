#![forbid(unsafe_code)]

//! RAM-resident GEL store with v2 header+payload CRC64 integrity.
//! The v0.2 writer emits only v2. v1 can still be read explicitly for migration.

use gel_core::{
    crc64_ecma, crc64_ecma_combine, Checksum64, Crc64Ecma, GelError, Generation,
    CRC64_PARALLEL_MIN_BYTES, GEL_FORMAT_VERSION, GEL_MAGIC_V1, GEL_MAGIC_V2, ORB_BITS, ORB_BYTES,
};
use gel_orb::Orb1024;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub const HEADER_BYTES: usize = 64;
pub const V2_HEADER_CRC_OFFSET: usize = 48;
pub const DEFAULT_MAX_PAYLOAD_BYTES: u64 = 256 * 1024 * 1024;
pub const DEFAULT_MAX_RECORDS: u64 = DEFAULT_MAX_PAYLOAD_BYTES / ORB_BYTES as u64;
pub const DEFAULT_MAX_FILE_BYTES: u64 = HEADER_BYTES as u64 + DEFAULT_MAX_PAYLOAD_BYTES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreFormat {
    LegacyV1,
    V2,
}

impl StoreFormat {
    pub const fn version(self) -> u32 {
        match self {
            Self::LegacyV1 => 1,
            Self::V2 => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: u32,
    pub orb_bits: u32,
    pub record_bytes: u32,
    pub flags: u32,
    pub record_count: u64,
    pub generation: Generation,
    pub payload_crc64: u64,
    pub header_crc64: u64,
}

/// Result of validating an on-disk store.
///
/// `source_format` reports what was actually read. `header` is always the
/// protected v2 header that would be emitted by a subsequent write; legacy v1
/// payloads are upgraded in memory only after their legacy checksum passes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Verification {
    pub source_format: StoreFormat,
    pub header: Header,
}

impl Header {
    pub fn new(record_count: u64, generation: Generation, payload_crc64: u64) -> Self {
        let mut header = Self {
            version: GEL_FORMAT_VERSION,
            orb_bits: ORB_BITS as u32,
            record_bytes: ORB_BYTES as u32,
            flags: 0,
            record_count,
            generation,
            payload_crc64,
            header_crc64: 0,
        };
        header.header_crc64 = header.compute_crc64();
        header
    }

    pub fn encode(self) -> [u8; HEADER_BYTES] {
        let mut out = self.encode_without_header_crc();
        let crc = self.compute_crc64();
        out[V2_HEADER_CRC_OFFSET..V2_HEADER_CRC_OFFSET + 8].copy_from_slice(&crc.to_le_bytes());
        out
    }

    fn encode_without_header_crc(self) -> [u8; HEADER_BYTES] {
        let mut out = [0u8; HEADER_BYTES];
        out[0..8].copy_from_slice(&GEL_MAGIC_V2);
        out[8..12].copy_from_slice(&self.version.to_le_bytes());
        out[12..16].copy_from_slice(&self.orb_bits.to_le_bytes());
        out[16..20].copy_from_slice(&self.record_bytes.to_le_bytes());
        out[20..24].copy_from_slice(&self.flags.to_le_bytes());
        out[24..32].copy_from_slice(&self.record_count.to_le_bytes());
        out[32..40].copy_from_slice(&self.generation.0.to_le_bytes());
        out[40..48].copy_from_slice(&self.payload_crc64.to_le_bytes());
        // 48..56 is zero while CRC is calculated.
        // 56..64 is reserved and must remain zero.
        out
    }

    fn compute_crc64(self) -> u64 {
        crc64_ecma(&self.encode_without_header_crc())
    }

    pub fn decode(bytes: &[u8; HEADER_BYTES]) -> Result<Self, GelError> {
        if bytes[0..8] != GEL_MAGIC_V2 {
            return Err(GelError::InvalidMagic);
        }
        let version = le_u32(bytes, 8);
        if version != GEL_FORMAT_VERSION {
            return Err(GelError::UnsupportedVersion(version));
        }
        let orb_bits = le_u32(bytes, 12);
        let record_bytes = le_u32(bytes, 16);
        let flags = le_u32(bytes, 20);
        if orb_bits as usize != ORB_BITS {
            return Err(GelError::InvalidHeader("orb_bits != 1024"));
        }
        if record_bytes as usize != ORB_BYTES {
            return Err(GelError::InvalidHeader("record_bytes != 128"));
        }
        if flags != 0 {
            return Err(GelError::InvalidHeader("unsupported nonzero flags"));
        }
        if bytes[56..64].iter().any(|&b| b != 0) {
            return Err(GelError::InvalidHeader("reserved bytes must be zero"));
        }
        let header = Self {
            version,
            orb_bits,
            record_bytes,
            flags,
            record_count: le_u64(bytes, 24),
            generation: Generation(le_u64(bytes, 32)),
            payload_crc64: le_u64(bytes, 40),
            header_crc64: le_u64(bytes, 48),
        };
        if header.compute_crc64() != header.header_crc64 {
            return Err(GelError::CorruptHeader);
        }
        Ok(header)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenLimits {
    pub max_records: u64,
    pub max_file_bytes: u64,
}

impl OpenLimits {
    /// Default allocation budget: at most 256 MiB of ORB payload.
    pub const SAFE_DEFAULT: Self = Self {
        max_records: DEFAULT_MAX_RECORDS,
        max_file_bytes: DEFAULT_MAX_FILE_BYTES,
    };

    /// Explicit trusted-input opt-in; format and integrity checks still apply.
    pub const UNLIMITED: Self = Self {
        max_records: u64::MAX,
        max_file_bytes: u64::MAX,
    };
}

#[derive(Debug)]
pub struct RamStore {
    header: Header,
    source_format: StoreFormat,
    orbs: Vec<Orb1024>,
}

impl RamStore {
    pub fn from_orbs(orbs: Vec<Orb1024>, generation: Generation) -> Self {
        let payload_crc64 = crc_orbs(&orbs);
        let header = Header::new(orbs.len() as u64, generation, payload_crc64);
        Self {
            header,
            source_format: StoreFormat::V2,
            orbs,
        }
    }

    pub fn header(&self) -> Header {
        self.header
    }
    pub fn source_format(&self) -> StoreFormat {
        self.source_format
    }
    pub fn orbs(&self) -> &[Orb1024] {
        &self.orbs
    }
    pub fn len(&self) -> usize {
        self.orbs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.orbs.is_empty()
    }

    /// Single-writer atomic publish. On Unix the parent directory sync is part
    /// of the success contract. On non-Unix the file itself is synced but the
    /// standard library has no portable directory-fsync primitive.
    pub fn write_atomic(&self, path: impl AsRef<Path>) -> Result<(), GelError> {
        write_atomic_bytes(path.as_ref(), self.header, &self.orbs)
    }

    /// Rejects stale or equal generations before publishing. This assumes the
    /// project single-writer rule; it is not a multi-process compare-and-swap.
    pub fn write_if_newer(&self, path: impl AsRef<Path>) -> Result<(), GelError> {
        let path = path.as_ref();
        if path.exists() {
            let current = read_generation(path)?;
            if self.header.generation.0 <= current.0 {
                return Err(GelError::GenerationRollback {
                    current: current.0,
                    attempted: self.header.generation.0,
                });
            }
        }
        self.write_atomic(path)
    }

    /// Loads at most 256 MiB of payload. Use explicit limits for larger stores.
    pub fn open_verified(path: impl AsRef<Path>) -> Result<Self, GelError> {
        Self::open_verified_with_limits(path, OpenLimits::SAFE_DEFAULT)
    }

    /// Loads a trusted-size file without an allocation budget.
    pub fn open_verified_unbounded(path: impl AsRef<Path>) -> Result<Self, GelError> {
        Self::open_verified_with_limits(path, OpenLimits::UNLIMITED)
    }

    pub fn open_verified_with_limits(
        path: impl AsRef<Path>,
        limits: OpenLimits,
    ) -> Result<Self, GelError> {
        let Validated {
            mut file,
            count,
            mut check,
        } = open_validated(path.as_ref(), limits)?;
        let mut orbs = Vec::new();
        orbs.try_reserve_exact(count)
            .map_err(|_| GelError::AllocationFailed)?;
        stream_payload(&mut file, count, &mut check, Some(&mut orbs))?;
        let verification = check.finish()?;
        Ok(RamStore {
            header: verification.header,
            source_format: verification.source_format,
            orbs,
        })
    }
}

/// Verifies a store file without loading it.
///
/// Runs exactly the header, limit and file-length checks of
/// [`RamStore::open_verified_with_limits`], then streams the payload through
/// one fixed buffer of at most 16 MiB (two 8 MiB halves) to check the payload digest. No payload
/// is retained: memory use is constant regardless of file size. The report keeps
/// the actual on-disk source format separate from the protected v2 header a
/// loaded store would carry.
pub fn verify_file(path: impl AsRef<Path>, limits: OpenLimits) -> Result<Verification, GelError> {
    let Validated {
        mut file,
        count,
        mut check,
    } = open_validated(path.as_ref(), limits)?;
    stream_payload(&mut file, count, &mut check, None)?;
    check.finish()
}

/// Records per payload write: 8192 records, 1 MiB.
const IO_BLOCK_RECORDS: usize = 8192;

/// Records per payload read: 65,536 records, 8 MiB. A block of at least
/// [`CRC64_PARALLEL_MIN_BYTES`] is hashed by [`Crc64Ecma::update`] on
/// several threads; the CRC value does not depend on the block size. Payloads
/// longer than one block overlap the read of a block with the digest of the
/// previous one (see [`stream_payload`]).
const READ_BLOCK_RECORDS: usize = 65_536;

/// Reads `count` records in blocks of [`READ_BLOCK_RECORDS`], feeds every
/// block to the digests in order and, when `orbs` is given, appends the
/// decoded records. One 16 MiB buffer is used as two halves of 8 MiB: while
/// the calling thread reads block k+1 into one half, scoped threads digest
/// (and decode) block k from the other. A payload of one block or less is
/// read with one buffer of at most 8 MiB and no extra thread.
///
/// The digests are streaming, so the result does not depend on the block
/// size or on which thread fed a block. A failed read returns the error
/// `read_exact` reports, exactly as a read of one record at a time would;
/// the records read before it cannot change the outcome because decoding
/// 128 bytes cannot fail.
fn stream_payload<R: Read>(
    reader: &mut R,
    count: usize,
    check: &mut PayloadCheck,
    orbs: Option<&mut Vec<Orb1024>>,
) -> Result<(), GelError> {
    stream_payload_with(reader, count, check, orbs, true)
}

/// [`stream_payload`]; `threads == false` runs every step on the calling
/// thread, in the same order (the path taken when a thread cannot start).
fn stream_payload_with<R: Read>(
    reader: &mut R,
    count: usize,
    check: &mut PayloadCheck,
    mut orbs: Option<&mut Vec<Orb1024>>,
    threads: bool,
) -> Result<(), GelError> {
    if count == 0 {
        return Ok(());
    }
    if count <= READ_BLOCK_RECORDS {
        let mut block = vec![0u8; count * ORB_BYTES];
        reader.read_exact(&mut block)?;
        check.update(&block);
        if let Some(orbs) = orbs {
            decode_block(&block, orbs)?;
        }
        return Ok(());
    }
    let block_bytes = READ_BLOCK_RECORDS * ORB_BYTES;
    // One allocation for both halves: two separate 8 MiB allocations freed
    // together can push the heap past the allocator's trim threshold, so
    // that the next buffer, even a small file's, touches fresh pages again.
    let mut buffer = vec![0u8; 2 * block_bytes];
    let (mut current, mut next) = buffer.split_at_mut(block_bytes);
    reader.read_exact(current)?;
    let mut records = READ_BLOCK_RECORDS;
    let mut remaining = count - records;
    while remaining > 0 {
        let following = remaining.min(READ_BLOCK_RECORDS);
        let target = &mut next[..following * ORB_BYTES];
        let read = absorb_while(
            &current[..records * ORB_BYTES],
            check,
            orbs.as_deref_mut(),
            threads,
            || reader.read_exact(target),
        )?;
        read?;
        std::mem::swap(&mut current, &mut next);
        records = following;
        remaining -= following;
    }
    // Nothing is left to read: only a load has a second task to overlap.
    let threads = threads && orbs.is_some();
    absorb_while(&current[..records * ORB_BYTES], check, orbs, threads, || ())
}

/// Appends the records of `block` (a whole number of records) to `orbs`.
fn decode_block(block: &[u8], orbs: &mut Vec<Orb1024>) -> Result<(), GelError> {
    for record in block.chunks_exact(ORB_BYTES) {
        orbs.push(Orb1024::from_le_bytes(record)?);
    }
    Ok(())
}

/// Feeds `block` to `check` and decodes it into `orbs` (when given) while the
/// calling thread runs `work`, then returns what `work` returned. With
/// `threads`, the digest and the decoding each run on a scoped thread; a
/// task whose thread cannot start, or every task without `threads`, runs on
/// the calling thread after `work`. The digest and decoding of `block` are
/// complete when this returns, and a decoding error takes precedence over
/// the value of `work`, as in a loop that finishes block k before it reads
/// block k+1. A panic of a task is resumed on the calling thread.
fn absorb_while<T>(
    block: &[u8],
    check: &mut PayloadCheck,
    mut orbs: Option<&mut Vec<Orb1024>>,
    threads: bool,
    work: impl FnOnce() -> T,
) -> Result<T, GelError> {
    let (digested, decoded, value) = std::thread::scope(|scope| {
        let digest = if threads {
            std::thread::Builder::new()
                .spawn_scoped(scope, || check.update(block))
                .ok()
        } else {
            None
        };
        let decode = match orbs.as_deref_mut() {
            Some(sink) if threads => std::thread::Builder::new()
                .spawn_scoped(scope, move || decode_block(block, sink))
                .ok(),
            _ => None,
        };
        let value = work();
        let digested = digest.map(|handle| handle.join());
        let decoded = decode.map(|handle| handle.join());
        (digested, decoded, value)
    });
    match digested {
        Some(Ok(())) => {}
        Some(Err(panic)) => std::panic::resume_unwind(panic),
        None => check.update(block),
    }
    match decoded {
        Some(Ok(result)) => result?,
        Some(Err(panic)) => std::panic::resume_unwind(panic),
        None => {
            if let Some(sink) = orbs {
                decode_block(block, sink)?;
            }
        }
    }
    Ok(value)
}

/// Writes the records in blocks of [`IO_BLOCK_RECORDS`], one `write_all` per
/// block. The bytes are the records' little-endian encodings in order.
fn write_payload<W: Write>(writer: &mut W, orbs: &[Orb1024]) -> Result<(), GelError> {
    let mut buffer = vec![0u8; orbs.len().min(IO_BLOCK_RECORDS) * ORB_BYTES];
    for records in orbs.chunks(IO_BLOCK_RECORDS) {
        let block = &mut buffer[..records.len() * ORB_BYTES];
        for (bytes, orb) in block.chunks_exact_mut(ORB_BYTES).zip(records) {
            bytes.copy_from_slice(&orb.to_le_bytes());
        }
        writer.write_all(block)?;
    }
    Ok(())
}

/// A file whose header passed every check that precedes the payload.
struct Validated {
    file: File,
    count: usize,
    check: PayloadCheck,
}

/// Header fields the payload is checked against, with the running digests.
enum PayloadCheck {
    V2 {
        header: Header,
        crc: Crc64Ecma,
    },
    V1 {
        record_count: u64,
        expected_checksum: u64,
        checksum: Checksum64,
        crc: Crc64Ecma,
    },
}

impl PayloadCheck {
    fn record_count(&self) -> u64 {
        match self {
            Self::V2 { header, .. } => header.record_count,
            Self::V1 { record_count, .. } => *record_count,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        match self {
            Self::V2 { crc, .. } => crc.update(bytes),
            Self::V1 { checksum, crc, .. } => {
                checksum.update(bytes);
                crc.update(bytes);
            }
        }
    }

    /// Compares the digests with the header and returns the header a loaded
    /// store carries. A v1 header is upgraded so any subsequent write emits
    /// protected v2.
    fn finish(self) -> Result<Verification, GelError> {
        match self {
            Self::V2 { header, crc } => {
                if crc.finish() != header.payload_crc64 {
                    return Err(GelError::CorruptStore);
                }
                Ok(Verification {
                    source_format: StoreFormat::V2,
                    header,
                })
            }
            Self::V1 {
                record_count,
                expected_checksum,
                checksum,
                crc,
            } => {
                if checksum.finish() != expected_checksum {
                    return Err(GelError::CorruptStore);
                }
                Ok(Verification {
                    source_format: StoreFormat::LegacyV1,
                    // v1 does not checksum its generation. Never promote
                    // unprotected metadata into trusted monotonic state.
                    header: Header::new(record_count, Generation(0), crc.finish()),
                })
            }
        }
    }
}

/// Opens the file and runs every check that precedes the payload, in this
/// order: file-size budget, magic, version, geometry, flags, reserved bytes,
/// header CRC (v2 only), record budget, exact file length. Shared by loading
/// and by streaming verification so both reject identically.
fn open_validated(path: &Path, limits: OpenLimits) -> Result<Validated, GelError> {
    let mut file = File::open(path)?;
    let actual_file = file.metadata()?.len();
    if actual_file > limits.max_file_bytes {
        return Err(GelError::LimitExceeded("file bytes"));
    }
    let mut header_bytes = [0u8; HEADER_BYTES];
    file.read_exact(&mut header_bytes)?;
    let check = if header_bytes[0..8] == GEL_MAGIC_V2 {
        PayloadCheck::V2 {
            header: Header::decode(&header_bytes)?,
            crc: Crc64Ecma::new(),
        }
    } else if header_bytes[0..8] == GEL_MAGIC_V1 {
        parse_v1_header(&header_bytes)?
    } else {
        return Err(GelError::InvalidMagic);
    };
    let record_count = check.record_count();
    validate_count_and_size(record_count, actual_file, limits)?;
    let count = usize::try_from(record_count)
        .map_err(|_| GelError::InvalidHeader("record_count too large"))?;
    Ok(Validated { file, count, check })
}

fn parse_v1_header(header_bytes: &[u8; HEADER_BYTES]) -> Result<PayloadCheck, GelError> {
    let version = le_u32(header_bytes, 8);
    if version != 1 {
        return Err(GelError::UnsupportedVersion(version));
    }
    if le_u32(header_bytes, 12) as usize != ORB_BITS
        || le_u32(header_bytes, 16) as usize != ORB_BYTES
    {
        return Err(GelError::InvalidHeader("legacy geometry mismatch"));
    }
    if header_bytes[20..24].iter().any(|&b| b != 0) || header_bytes[48..64].iter().any(|&b| b != 0)
    {
        return Err(GelError::InvalidHeader(
            "legacy reserved bytes must be zero",
        ));
    }
    Ok(PayloadCheck::V1 {
        record_count: le_u64(header_bytes, 24),
        expected_checksum: le_u64(header_bytes, 40),
        checksum: Checksum64::new(),
        crc: Crc64Ecma::new(),
    })
}

fn validate_count_and_size(
    record_count: u64,
    actual_file: u64,
    limits: OpenLimits,
) -> Result<(), GelError> {
    if record_count > limits.max_records {
        return Err(GelError::LimitExceeded("record count"));
    }
    let payload = record_count
        .checked_mul(ORB_BYTES as u64)
        .ok_or(GelError::InvalidHeader("payload size overflow"))?;
    let expected = (HEADER_BYTES as u64)
        .checked_add(payload)
        .ok_or(GelError::InvalidHeader("file size overflow"))?;
    if actual_file != expected {
        return Err(GelError::InvalidLength {
            expected: usize::try_from(expected).unwrap_or(usize::MAX),
            actual: usize::try_from(actual_file).unwrap_or(usize::MAX),
        });
    }
    Ok(())
}

fn write_atomic_bytes(path: &Path, header: Header, orbs: &[Orb1024]) -> Result<(), GelError> {
    let (tmp, mut file) = create_temp_file(path)?;
    let write_result = (|| -> Result<(), GelError> {
        file.write_all(&header.encode())?;
        write_payload(&mut file, orbs)?;
        file.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&tmp);
        return Err(error);
    }
    if let Err(error) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(error.into());
    }
    sync_parent_dir(path)?;
    Ok(())
}

#[cfg(unix)]
fn sync_parent_dir(path: &Path) -> Result<(), GelError> {
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
fn sync_parent_dir(_path: &Path) -> Result<(), GelError> {
    Ok(())
}

fn read_generation(path: &Path) -> Result<Generation, GelError> {
    // Streaming validation uses constant memory even for large files.
    // The caller's single-writer contract excludes concurrent mutation.
    let verification = verify_file(path, OpenLimits::UNLIMITED)?;
    match verification.source_format {
        StoreFormat::V2 => Ok(verification.header.generation),
        StoreFormat::LegacyV1 => Err(GelError::LegacyGenerationUntrusted),
    }
}

/// CRC-64/ECMA of the records' little-endian encodings in order. A payload
/// of at least [`CRC64_PARALLEL_MIN_BYTES`] is split across scoped threads
/// (at most [`CRC_PARALLEL_MAX_THREADS`], segments of at least
/// [`CRC_PARALLEL_SEGMENT_RECORDS`] records); the value is the same as the
/// one-thread loop. No payload buffer is allocated.
fn crc_orbs(orbs: &[Orb1024]) -> u64 {
    if orbs.len().saturating_mul(ORB_BYTES) >= CRC64_PARALLEL_MIN_BYTES {
        let threads = crc_parallel_threads().min(orbs.len() / CRC_PARALLEL_SEGMENT_RECORDS);
        if threads > 1 {
            return crc_orbs_with_threads(orbs, threads);
        }
    }
    crc_orbs_serial(orbs)
}

/// Smallest segment one thread hashes in [`crc_orbs`]: 8192 records, 1 MiB.
const CRC_PARALLEL_SEGMENT_RECORDS: usize = 8192;

/// Upper bound on threads in [`crc_orbs`].
const CRC_PARALLEL_MAX_THREADS: usize = 8;

fn crc_parallel_threads() -> usize {
    static THREADS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *THREADS.get_or_init(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .clamp(1, CRC_PARALLEL_MAX_THREADS)
    })
}

/// The one-thread loop: one 128-byte update per record.
fn crc_orbs_serial(orbs: &[Orb1024]) -> u64 {
    let mut crc = Crc64Ecma::new();
    for orb in orbs {
        crc.update(&orb.to_le_bytes());
    }
    crc.finish()
}

/// Hashes at most `threads` consecutive segments of `ceil(len / threads)`
/// records. The calling thread hashes the first segment; every other segment
/// is hashed from a zero register on its own scoped thread, and the segment
/// CRCs are joined left to right with [`crc64_ecma_combine`]. A segment whose
/// thread cannot be started is hashed on the calling thread. Equals
/// [`crc_orbs_serial`] for every input and every `threads`; `0` and `1` mean
/// one thread.
fn crc_orbs_with_threads(orbs: &[Orb1024], threads: usize) -> u64 {
    let threads = threads.max(1).min(orbs.len());
    if threads <= 1 {
        return crc_orbs_serial(orbs);
    }
    let segment = orbs.len().div_ceil(threads);
    let (first, rest) = orbs.split_at(segment);
    std::thread::scope(|scope| {
        let handles: Vec<_> = rest
            .chunks(segment)
            .map(|piece| {
                let handle = std::thread::Builder::new()
                    .spawn_scoped(scope, move || crc_orbs_serial(piece))
                    .ok();
                (piece, handle)
            })
            .collect();
        let mut crc = crc_orbs_serial(first);
        for (piece, handle) in handles {
            let piece_crc = match handle.map(|h| h.join()) {
                Some(Ok(value)) => value,
                _ => crc_orbs_serial(piece),
            };
            crc = crc64_ecma_combine(crc, piece_crc, (piece.len() * ORB_BYTES) as u64);
        }
        crc
    })
}

fn temp_path(path: &Path) -> PathBuf {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(
        ".tmp-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
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
        let tmp = temp_path(path);
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
        "could not allocate a unique GEL temporary path".into(),
    ))
}

#[inline]
fn le_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("fixed header slice"),
    )
}

#[inline]
fn le_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("fixed header slice"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gel_core::{checksum64, splitmix64, GEL_MAGIC_V1, ORB_WORDS};

    fn orb(seed: u64) -> Orb1024 {
        let mut words = [0u64; ORB_WORDS];
        for (i, w) in words.iter_mut().enumerate() {
            *w = splitmix64(seed + i as u64);
        }
        Orb1024::from_words(words)
    }

    fn path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "gel-store-v2-{name}-{}-{}.gel",
            std::process::id(),
            splitmix64(9)
        ))
    }

    #[test]
    fn v2_header_is_exactly_64_bytes_and_self_protected() {
        let header = Header::new(3, Generation(7), 0x1234);
        assert_eq!(header.encode().len(), 64);
        assert_eq!(Header::decode(&header.encode()).unwrap(), header);
    }

    #[test]
    fn every_single_header_bit_flip_is_rejected() {
        let header = Header::new(3, Generation(0x1122_3344_5566_7788), 0x8877_6655_4433_2211);
        let original = header.encode();
        let mut rejected = 0usize;
        for bit in 0..HEADER_BYTES * 8 {
            let mut mutated = original;
            mutated[bit >> 3] ^= 1u8 << (bit & 7);
            if Header::decode(&mutated).is_err() {
                rejected += 1;
            }
        }
        assert_eq!(rejected, HEADER_BYTES * 8);
    }

    #[test]
    fn persistence_roundtrip_and_payload_corruption_rejection() {
        let path = path("roundtrip");
        let store = RamStore::from_orbs(vec![orb(1), orb(2), orb(3)], Generation(9));
        store.write_atomic(&path).unwrap();
        let reopened = RamStore::open_verified(&path).unwrap();
        assert_eq!(reopened.orbs(), store.orbs());
        let mut bytes = fs::read(&path).unwrap();
        bytes[HEADER_BYTES + 13] ^= 0x80;
        fs::write(&path, bytes).unwrap();
        assert!(matches!(
            RamStore::open_verified(&path),
            Err(GelError::CorruptStore)
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn generation_rollback_and_equal_generation_are_rejected() {
        let path = path("generation");
        RamStore::from_orbs(vec![orb(1)], Generation(10))
            .write_atomic(&path)
            .unwrap();
        assert!(matches!(
            RamStore::from_orbs(vec![orb(2)], Generation(9)).write_if_newer(&path),
            Err(GelError::GenerationRollback {
                current: 10,
                attempted: 9
            })
        ));
        assert!(matches!(
            RamStore::from_orbs(vec![orb(2)], Generation(10)).write_if_newer(&path),
            Err(GelError::GenerationRollback {
                current: 10,
                attempted: 10
            })
        ));
        RamStore::from_orbs(vec![orb(3)], Generation(11))
            .write_if_newer(&path)
            .unwrap();
        assert_eq!(
            RamStore::open_verified(&path).unwrap().header().generation,
            Generation(11)
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn configured_open_limits_fail_before_large_allocation() {
        let path = path("limits");
        RamStore::from_orbs(vec![orb(1), orb(2)], Generation(1))
            .write_atomic(&path)
            .unwrap();
        let limits = OpenLimits {
            max_records: 1,
            max_file_bytes: u64::MAX,
        };
        assert!(matches!(
            RamStore::open_verified_with_limits(&path, limits),
            Err(GelError::LimitExceeded(_))
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn default_budget_rejects_oversized_sparse_file_before_read_or_allocation() {
        let path = path("default-budget");
        let file = File::create(&path).unwrap();
        file.set_len(DEFAULT_MAX_FILE_BYTES + 1).unwrap();
        drop(file);
        assert_eq!(DEFAULT_MAX_RECORDS, 2_097_152);
        assert!(matches!(
            RamStore::open_verified(&path),
            Err(GelError::LimitExceeded("file bytes"))
        ));
        assert!(matches!(
            verify_file(&path, OpenLimits::SAFE_DEFAULT),
            Err(GelError::LimitExceeded("file bytes"))
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn record_budget_boundary_and_explicit_unbounded_are_preserved() {
        assert!(validate_count_and_size(
            DEFAULT_MAX_RECORDS,
            DEFAULT_MAX_FILE_BYTES,
            OpenLimits::SAFE_DEFAULT
        )
        .is_ok());
        assert!(matches!(
            validate_count_and_size(
                DEFAULT_MAX_RECORDS + 1,
                DEFAULT_MAX_FILE_BYTES + ORB_BYTES as u64,
                OpenLimits::SAFE_DEFAULT
            ),
            Err(GelError::LimitExceeded("record count"))
        ));
        let path = path("unbounded");
        RamStore::from_orbs(vec![orb(7)], Generation(1))
            .write_atomic(&path)
            .unwrap();
        assert_eq!(
            RamStore::open_verified_unbounded(&path).unwrap().orbs(),
            &[orb(7)]
        );
        assert!(matches!(
            RamStore::open_verified_with_limits(
                &path,
                OpenLimits {
                    max_records: 0,
                    max_file_bytes: u64::MAX
                }
            ),
            Err(GelError::LimitExceeded("record count"))
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn corrupt_v2_cannot_supply_generation_or_be_silently_overwritten() {
        let path = path("corrupt-generation");
        let original = v2_file_bytes(&[orb(1), orb(2)], 5);
        let replacement = RamStore::from_orbs(vec![orb(3)], Generation(6));
        let mut payload_flip = original.clone();
        payload_flip[HEADER_BYTES] ^= 1;
        let mut header_flip = original.clone();
        header_flip[32] ^= 1;
        let mut trailing = original.clone();
        trailing.push(0);
        for damaged in [
            payload_flip,
            header_flip,
            trailing,
            original[..HEADER_BYTES].to_vec(),
        ] {
            fs::write(&path, &damaged).unwrap();
            assert!(replacement.write_if_newer(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), damaged);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_generation_is_discarded_and_never_authorizes_publication() {
        let path = path("legacy-generation");
        for generation in [0, 1, 77, u64::MAX] {
            let bytes = legacy_v1_file_bytes(&[orb(9)], generation);
            fs::write(&path, &bytes).unwrap();
            let loaded = RamStore::open_verified(&path).unwrap();
            assert_eq!(loaded.header().generation, Generation(0));
            assert_eq!(loaded.orbs(), &[orb(9)]);
            assert_eq!(
                RamStore::from_orbs(vec![orb(10)], Generation(u64::MAX)).write_if_newer(&path),
                Err(GelError::LegacyGenerationUntrusted)
            );
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        // Explicit migration, then normal monotonic state, remains available.
        RamStore::open_verified(&path)
            .unwrap()
            .write_atomic(&path)
            .unwrap();
        RamStore::from_orbs(vec![orb(10)], Generation(1))
            .write_if_newer(&path)
            .unwrap();
        assert_eq!(
            RamStore::open_verified(&path).unwrap().header().generation,
            Generation(1)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn malformed_lengths_and_trailing_bytes_are_rejected() {
        let base = path("invalid");
        RamStore::from_orbs(vec![orb(1)], Generation(11))
            .write_atomic(&base)
            .unwrap();
        let original = fs::read(&base).unwrap();
        for length in [0usize, HEADER_BYTES - 1, original.len() - 1] {
            let path = base.with_extension(format!("truncated-{length}.gel"));
            fs::write(&path, &original[..length]).unwrap();
            assert!(RamStore::open_verified(&path).is_err());
            let _ = fs::remove_file(path);
        }
        let trailing = base.with_extension("trailing.gel");
        let mut bytes = original;
        bytes.push(0);
        fs::write(&trailing, bytes).unwrap();
        assert!(RamStore::open_verified(&trailing).is_err());
        let _ = fs::remove_file(trailing);
        let _ = fs::remove_file(base);
    }

    #[test]
    fn legacy_v1_store_is_verified_and_migrates_to_v2_on_write() {
        let source = path("legacy-v1");
        let migrated = path("legacy-v2");
        let records = [orb(31), orb(32)];
        let mut payload = Vec::new();
        for record in &records {
            payload.extend_from_slice(&record.to_le_bytes());
        }
        let mut header = [0u8; HEADER_BYTES];
        header[0..8].copy_from_slice(&GEL_MAGIC_V1);
        header[8..12].copy_from_slice(&1u32.to_le_bytes());
        header[12..16].copy_from_slice(&(ORB_BITS as u32).to_le_bytes());
        header[16..20].copy_from_slice(&(ORB_BYTES as u32).to_le_bytes());
        header[24..32].copy_from_slice(&(records.len() as u64).to_le_bytes());
        header[32..40].copy_from_slice(&77u64.to_le_bytes());
        header[40..48].copy_from_slice(&checksum64(&payload).to_le_bytes());
        let mut file = File::create(&source).unwrap();
        file.write_all(&header).unwrap();
        file.write_all(&payload).unwrap();
        file.sync_all().unwrap();

        let reopened = RamStore::open_verified(&source).unwrap();
        assert_eq!(reopened.orbs(), records);
        assert_eq!(reopened.source_format(), StoreFormat::LegacyV1);
        assert_eq!(reopened.header().version, 2);
        assert_eq!(reopened.header().generation, Generation(0));
        reopened.write_atomic(&migrated).unwrap();
        let migrated_bytes = fs::read(&migrated).unwrap();
        assert_eq!(&migrated_bytes[..8], &GEL_MAGIC_V2);
        assert_eq!(RamStore::open_verified(&migrated).unwrap().orbs(), records);
        let _ = fs::remove_file(source);
        let _ = fs::remove_file(migrated);
    }

    fn legacy_v1_file_bytes(records: &[Orb1024], generation: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        for record in records {
            payload.extend_from_slice(&record.to_le_bytes());
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

    #[test]
    fn oversized_record_count_is_rejected_before_allocation() {
        // Both headers carry a valid CRC, so only the size check can reject
        // them. Had try_reserve_exact been reached under UNLIMITED limits the
        // error would be AllocationFailed; the variants below show that
        // validate_count_and_size runs first.
        let overflow = path("count-overflow");
        fs::write(&overflow, Header::new(u64::MAX, Generation(1), 0).encode()).unwrap();
        assert!(matches!(
            RamStore::open_verified_unbounded(&overflow),
            Err(GelError::InvalidHeader("payload size overflow"))
        ));
        let _ = fs::remove_file(overflow);

        let huge = path("count-huge");
        fs::write(&huge, Header::new(1 << 40, Generation(1), 0).encode()).unwrap();
        match RamStore::open_verified_unbounded(&huge) {
            Err(GelError::InvalidLength { expected, actual }) => {
                assert_eq!(actual, HEADER_BYTES);
                assert_eq!(
                    expected,
                    usize::try_from((1u64 << 47) + HEADER_BYTES as u64).unwrap_or(usize::MAX)
                );
            }
            other => panic!("expected InvalidLength, got {other:?}"),
        }
        let _ = fs::remove_file(huge);
    }

    #[test]
    fn legacy_v1_nonzero_reserved_bytes_are_rejected() {
        let base = path("legacy-reserved");
        let original = legacy_v1_file_bytes(&[orb(41), orb(42)], 5);
        fs::write(&base, &original).unwrap();
        assert!(RamStore::open_verified(&base).is_ok());
        for offset in (20..24).chain(48..64) {
            let path = base.with_extension(format!("{offset}.gel"));
            let mut bytes = original.clone();
            bytes[offset] = 1;
            fs::write(&path, bytes).unwrap();
            assert!(
                matches!(
                    RamStore::open_verified(&path),
                    Err(GelError::InvalidHeader(
                        "legacy reserved bytes must be zero"
                    ))
                ),
                "offset {offset}"
            );
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(base);
    }

    #[test]
    fn v2_nonzero_flags_are_rejected_before_crc() {
        let valid = Header::new(1, Generation(1), 0).encode();
        // Flags set without recomputing the CRC: the flags check must report
        // before the CRC check would report CorruptHeader.
        let mut stale_crc = valid;
        stale_crc[20] = 1;
        assert!(matches!(
            Header::decode(&stale_crc),
            Err(GelError::InvalidHeader("unsupported nonzero flags"))
        ));
        // Flags set with a CRC recomputed by encode(): still rejected.
        let flagged = Header {
            flags: 1,
            ..Header::new(1, Generation(1), 0)
        }
        .encode();
        assert!(matches!(
            Header::decode(&flagged),
            Err(GelError::InvalidHeader("unsupported nonzero flags"))
        ));
        let path = path("flags");
        let mut bytes = flagged.to_vec();
        bytes.extend_from_slice(&orb(1).to_le_bytes());
        fs::write(&path, bytes).unwrap();
        assert!(matches!(
            RamStore::open_verified(&path),
            Err(GelError::InvalidHeader("unsupported nonzero flags"))
        ));
        let _ = fs::remove_file(path);
    }

    // ---- streaming verification -------------------------------------------

    /// Bytes of a v2 store exactly as `write_atomic` would emit them.
    fn v2_file_bytes(records: &[Orb1024], generation: u64) -> Vec<u8> {
        let store = RamStore::from_orbs(records.to_vec(), Generation(generation));
        let mut bytes = store.header().encode().to_vec();
        for record in records {
            bytes.extend_from_slice(&record.to_le_bytes());
        }
        bytes
    }

    /// The same Ok/Err outcome from `verify_file` and `open_verified`, with
    /// equal headers on success and equal error text on failure.
    fn assert_same_verdict(path: &Path) -> Result<Verification, GelError> {
        let streamed = verify_file(path, OpenLimits::UNLIMITED);
        let loaded = RamStore::open_verified_unbounded(path);
        match (&streamed, &loaded) {
            (Ok(verification), Ok(store)) => {
                assert_eq!(verification.header, store.header());
                assert_eq!(verification.source_format, store.source_format());
            }
            (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
            other => panic!("verify_file and open_verified disagree: {other:?}"),
        }
        streamed
    }

    #[test]
    fn verify_file_agrees_with_open_verified_and_rejects_corruption() {
        let path = path("verify-stream");
        let records = [orb(61), orb(62), orb(63)];
        let original = v2_file_bytes(&records, 44);
        fs::write(&path, &original).unwrap();
        let verification = assert_same_verdict(&path).unwrap();
        assert_eq!(verification.source_format, StoreFormat::V2);
        assert_eq!(verification.header.version, 2);
        assert_eq!(verification.header.record_count, 3);
        assert_eq!(verification.header.generation, Generation(44));

        let mut flipped = original.clone();
        flipped[HEADER_BYTES + 200] ^= 0x01;
        fs::write(&path, &flipped).unwrap();
        assert!(matches!(
            assert_same_verdict(&path),
            Err(GelError::CorruptStore)
        ));

        fs::write(&path, &original[..original.len() - 1]).unwrap();
        assert!(matches!(
            assert_same_verdict(&path),
            Err(GelError::InvalidLength { .. })
        ));

        fs::write(&path, &original).unwrap();
        let limits = OpenLimits {
            max_records: 2,
            max_file_bytes: u64::MAX,
        };
        assert!(matches!(
            verify_file(&path, limits),
            Err(GelError::LimitExceeded("record count"))
        ));
        let limits = OpenLimits {
            max_records: u64::MAX,
            max_file_bytes: original.len() as u64 - 1,
        };
        assert!(matches!(
            verify_file(&path, limits),
            Err(GelError::LimitExceeded("file bytes"))
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn verify_file_reports_legacy_v1_while_providing_upgrade_header() {
        let path = path("verify-legacy");
        let records = [orb(71), orb(72)];
        let original = legacy_v1_file_bytes(&records, 77);
        fs::write(&path, &original).unwrap();
        let verification = assert_same_verdict(&path).unwrap();
        assert_eq!(verification.source_format, StoreFormat::LegacyV1);
        assert_eq!(verification.source_format.version(), 1);
        assert_eq!(verification.header.version, 2);
        assert_eq!(verification.header.record_count, 2);
        assert_eq!(verification.header.generation, Generation(0));
        assert_eq!(
            verification.header.payload_crc64,
            crc64_ecma(&original[HEADER_BYTES..])
        );
        assert_eq!(
            Header::decode(&verification.header.encode()).unwrap(),
            verification.header
        );

        let mut flipped = original.clone();
        flipped[HEADER_BYTES + 5] ^= 0x10;
        fs::write(&path, &flipped).unwrap();
        assert!(matches!(
            assert_same_verdict(&path),
            Err(GelError::CorruptStore)
        ));
        let _ = fs::remove_file(path);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_is_private_by_default_and_preserves_existing_mode() {
        let fresh = path("private-mode-new");
        RamStore::from_orbs(vec![orb(1)], Generation(1))
            .write_atomic(&fresh)
            .unwrap();
        assert_eq!(
            fs::metadata(&fresh).unwrap().permissions().mode() & 0o777,
            0o600
        );

        fs::set_permissions(&fresh, fs::Permissions::from_mode(0o640)).unwrap();
        RamStore::from_orbs(vec![orb(2)], Generation(2))
            .write_if_newer(&fresh)
            .unwrap();
        assert_eq!(
            fs::metadata(&fresh).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(
            RamStore::open_verified(&fresh).unwrap().header().generation,
            Generation(2)
        );
        let _ = fs::remove_file(fresh);
    }

    // ---- deterministic mutation sweeps -------------------------------------

    fn pseudo_random_header(seed: u64) -> [u8; HEADER_BYTES] {
        let mut out = [0u8; HEADER_BYTES];
        for (i, chunk) in out.chunks_exact_mut(8).enumerate() {
            chunk.copy_from_slice(&splitmix64(seed * 8 + i as u64).to_le_bytes());
        }
        out
    }

    #[test]
    fn random_headers_are_rejected_and_never_panic() {
        const N: u64 = 4096;
        let mut rejected_random = 0usize;
        let mut rejected_prefixed = 0usize;
        let mut rejected_crc = 0usize;
        for seed in 0..N {
            // Uniformly pseudo-random bytes.
            assert!(Header::decode(&pseudo_random_header(seed)).is_err());
            rejected_random += 1;

            // Valid magic and version, pseudo-random remainder.
            let mut prefixed = pseudo_random_header(N + seed);
            prefixed[0..8].copy_from_slice(&GEL_MAGIC_V2);
            prefixed[8..12].copy_from_slice(&GEL_FORMAT_VERSION.to_le_bytes());
            assert!(Header::decode(&prefixed).is_err());
            rejected_prefixed += 1;

            // Fully valid header whose stored CRC is replaced by a
            // pseudo-random value: only the CRC comparison can reject it.
            let random = pseudo_random_header(2 * N + seed);
            let valid = Header::new(
                le_u64(&random, 0),
                Generation(le_u64(&random, 8)),
                le_u64(&random, 16),
            )
            .encode();
            let mut bad_crc = valid;
            bad_crc[V2_HEADER_CRC_OFFSET..V2_HEADER_CRC_OFFSET + 8]
                .copy_from_slice(&random[24..32]);
            assert!(matches!(
                Header::decode(&bad_crc),
                Err(GelError::CorruptHeader)
            ));
            rejected_crc += 1;
        }
        assert_eq!(rejected_random, 4096);
        assert_eq!(rejected_prefixed, 4096);
        assert_eq!(rejected_crc, 4096);
    }

    #[test]
    fn every_double_header_bit_flip_is_rejected() {
        let header = Header::new(3, Generation(0x1122_3344_5566_7788), 0x8877_6655_4433_2211);
        let original = header.encode();
        let bits = HEADER_BYTES * 8;
        let mut rejected = 0usize;
        let mut total = 0usize;
        for first in 0..bits {
            for second in first + 1..bits {
                let mut mutated = original;
                mutated[first >> 3] ^= 1u8 << (first & 7);
                mutated[second >> 3] ^= 1u8 << (second & 7);
                total += 1;
                if Header::decode(&mutated).is_err() {
                    rejected += 1;
                }
            }
        }
        assert_eq!(total, 130_816);
        assert_eq!(rejected, 130_816);
    }

    #[test]
    fn legacy_v1_header_single_bit_sweep_freezes_detection_boundary() {
        const GENERATION: u64 = 0x0123_4567_89ab_cdef;
        let path = path("legacy-bit-sweep");
        let records = [orb(81), orb(82)];
        let original = legacy_v1_file_bytes(&records, GENERATION);
        let mut detected = 0usize;
        let mut undetected = 0usize;
        let mut length_mismatch = 0usize;
        let mut size_overflow = 0usize;
        for bit in 0..HEADER_BYTES * 8 {
            let byte = bit >> 3;
            let mut bytes = original.clone();
            bytes[byte] ^= 1u8 << (bit & 7);
            fs::write(&path, &bytes).unwrap();
            let result = assert_same_verdict(&path);
            match byte {
                // magic, version, orb_bits, record_bytes, reserved 20..24
                0..=23 => {
                    assert!(result.is_err(), "bit {bit}");
                    detected += 1;
                }
                // record_count: the file length no longer matches, or the
                // payload size overflows for the seven highest bits.
                24..=31 => {
                    match result {
                        Err(GelError::InvalidLength { .. }) => length_mismatch += 1,
                        Err(GelError::InvalidHeader("payload size overflow")) => size_overflow += 1,
                        other => panic!("bit {bit}: {other:?}"),
                    }
                    detected += 1;
                }
                // v1 cannot detect this mutation, but v0.2.1 discards the
                // unprotected generation instead of trusting it.
                32..=39 => {
                    let verification = result.unwrap_or_else(|e| panic!("bit {bit}: {e}"));
                    assert_eq!(verification.source_format, StoreFormat::LegacyV1);
                    assert_eq!(verification.header.generation.0, 0);
                    assert_eq!(verification.header.record_count, 2);
                    undetected += 1;
                }
                // legacy checksum
                40..=47 => {
                    assert!(matches!(result, Err(GelError::CorruptStore)), "bit {bit}");
                    detected += 1;
                }
                // reserved 48..64
                _ => {
                    assert!(
                        matches!(
                            result,
                            Err(GelError::InvalidHeader(
                                "legacy reserved bytes must be zero"
                            ))
                        ),
                        "bit {bit}"
                    );
                    detected += 1;
                }
            }
        }
        let _ = fs::remove_file(path);
        assert_eq!(detected + undetected, 512);
        assert_eq!(detected, 448);
        assert_eq!(undetected, 64);
        assert_eq!(length_mismatch, 57);
        assert_eq!(size_overflow, 7);
    }

    #[test]
    fn every_payload_byte_flip_is_rejected() {
        let path = path("payload-byte-sweep");
        let original = v2_file_bytes(&[orb(91), orb(92), orb(93)], 3);
        let payload_len = original.len() - HEADER_BYTES;
        assert_eq!(payload_len, 384);
        let mut inverted = 0usize;
        let mut single_bit = 0usize;
        for index in 0..payload_len {
            let mut bytes = original.clone();
            bytes[HEADER_BYTES + index] ^= 0xFF;
            fs::write(&path, &bytes).unwrap();
            assert!(
                matches!(assert_same_verdict(&path), Err(GelError::CorruptStore)),
                "inverted byte {index}"
            );
            inverted += 1;

            let mut bytes = original.clone();
            bytes[HEADER_BYTES + index] ^= 1u8 << (index & 7);
            fs::write(&path, &bytes).unwrap();
            assert!(
                matches!(assert_same_verdict(&path), Err(GelError::CorruptStore)),
                "bit {} of byte {index}",
                index & 7
            );
            single_bit += 1;
        }
        let _ = fs::remove_file(path);
        assert_eq!(inverted, 384);
        assert_eq!(single_bit, 384);
    }

    #[test]
    fn every_truncation_length_is_rejected() {
        let path = path("truncation-sweep");
        let original = v2_file_bytes(&[orb(101)], 5);
        assert_eq!(original.len(), 192);
        let mut short_header = 0usize;
        let mut length_mismatch = 0usize;
        for length in 0..original.len() {
            fs::write(&path, &original[..length]).unwrap();
            match assert_same_verdict(&path) {
                Err(GelError::Io(_)) if length < HEADER_BYTES => short_header += 1,
                Err(GelError::InvalidLength { expected, actual }) if length >= HEADER_BYTES => {
                    assert_eq!(expected, 192);
                    assert_eq!(actual, length);
                    length_mismatch += 1;
                }
                other => panic!("length {length}: {other:?}"),
            }
        }
        let _ = fs::remove_file(path);
        assert_eq!(short_header, 64);
        assert_eq!(length_mismatch, 128);
        assert_eq!(short_header + length_mismatch, 192);
    }

    // ---- block I/O against the one-record-at-a-time reference ---------------

    /// The payload reader before block I/O: one `read_exact` per record.
    fn reference_stream_payload<R: Read>(
        reader: &mut R,
        count: usize,
        check: &mut PayloadCheck,
        mut orbs: Option<&mut Vec<Orb1024>>,
    ) -> Result<(), GelError> {
        let mut bytes = [0u8; ORB_BYTES];
        for _ in 0..count {
            reader.read_exact(&mut bytes)?;
            check.update(&bytes);
            if let Some(orbs) = orbs.as_deref_mut() {
                orbs.push(Orb1024::from_le_bytes(&bytes)?);
            }
        }
        Ok(())
    }

    /// The payload writer before block I/O: one `write_all` per record.
    fn reference_write_payload<W: Write>(writer: &mut W, orbs: &[Orb1024]) -> Result<(), GelError> {
        for orb in orbs {
            writer.write_all(&orb.to_le_bytes())?;
        }
        Ok(())
    }

    /// `open_verified_with_limits` and `verify_file` before block I/O, as
    /// one comparable text: the result, and the records on success.
    fn reference_open_and_verify(path: &Path) -> (String, String, Vec<Orb1024>) {
        let loaded = (|| -> Result<(Verification, Vec<Orb1024>), GelError> {
            let Validated {
                mut file,
                count,
                mut check,
            } = open_validated(path, OpenLimits::UNLIMITED)?;
            let mut orbs = Vec::new();
            orbs.try_reserve_exact(count)
                .map_err(|_| GelError::AllocationFailed)?;
            reference_stream_payload(&mut file, count, &mut check, Some(&mut orbs))?;
            Ok((check.finish()?, orbs))
        })();
        let streamed = (|| -> Result<Verification, GelError> {
            let Validated {
                mut file,
                count,
                mut check,
            } = open_validated(path, OpenLimits::UNLIMITED)?;
            reference_stream_payload(&mut file, count, &mut check, None)?;
            check.finish()
        })();
        match loaded {
            Ok((verification, orbs)) => {
                (format!("{verification:?}"), format!("{streamed:?}"), orbs)
            }
            Err(error) => (format!("{error:?}"), format!("{streamed:?}"), Vec::new()),
        }
    }

    fn current_open_and_verify(path: &Path) -> (String, String, Vec<Orb1024>) {
        let streamed = verify_file(path, OpenLimits::UNLIMITED);
        match RamStore::open_verified_unbounded(path) {
            Ok(store) => (
                format!(
                    "{:?}",
                    Verification {
                        source_format: store.source_format(),
                        header: store.header(),
                    }
                ),
                format!("{streamed:?}"),
                store.orbs().to_vec(),
            ),
            Err(error) => (format!("{error:?}"), format!("{streamed:?}"), Vec::new()),
        }
    }

    fn seeded_orbs(count: usize, seed: u64) -> Vec<Orb1024> {
        (0..count)
            .map(|i| orb(splitmix64(seed ^ (i as u64).wrapping_mul(0x9E37_79B9))))
            .collect()
    }

    /// Record counts around every block edge, plus seeded ones.
    fn edge_counts() -> Vec<usize> {
        let b = IO_BLOCK_RECORDS;
        let mut counts = vec![0, 1, 2, b - 1, b, b + 1, 2 * b, 2 * b + 1, 20_000];
        for seed in 0..4u64 {
            counts.push((splitmix64(seed + 700) % (3 * b as u64)) as usize);
        }
        counts
    }

    #[test]
    fn block_io_files_match_record_at_a_time_reference() {
        let path = path("block-io-differential");
        let mut cases = 0usize;
        for (index, count) in edge_counts().into_iter().enumerate() {
            let records = seeded_orbs(count, 1_000 + index as u64);
            let generation = 40 + index as u64;
            let store = RamStore::from_orbs(records.clone(), Generation(generation));
            store.write_atomic(&path).unwrap();
            let written = fs::read(&path).unwrap();
            let mut reference = store.header().encode().to_vec();
            reference_write_payload(&mut reference, &records).unwrap();
            assert_eq!(written, reference, "write bytes, {count} records");
            assert_eq!(written, v2_file_bytes(&records, generation));

            let legacy = legacy_v1_file_bytes(&records, generation);
            for (format, original) in [written, legacy].into_iter().enumerate() {
                // Release builds check both formats; debug builds (slow
                // bitwise CRC) alternate between them.
                if cfg!(debug_assertions) && (index + format) % 2 == 1 {
                    continue;
                }
                let payload = original.len() - HEADER_BYTES;
                let mut variants = vec![original.clone()];
                if payload > 0 {
                    for offset in [0, payload / 2, payload - 1] {
                        let mut flipped = original.clone();
                        flipped[HEADER_BYTES + offset] ^= 0x5A;
                        variants.push(flipped);
                    }
                    let mut short = original.clone();
                    short.pop();
                    variants.push(short);
                }
                for (variant, bytes) in variants.iter().enumerate() {
                    fs::write(&path, bytes).unwrap();
                    let expected = reference_open_and_verify(&path);
                    let actual = current_open_and_verify(&path);
                    assert_eq!(actual, expected, "{count} records, variant {variant}");
                    if variant == 0 {
                        assert_eq!(actual.2, records);
                    } else if variant <= 3 {
                        assert_eq!(actual.0, "CorruptStore");
                    }
                    cases += 1;
                }
            }
        }
        let _ = fs::remove_file(path);
        // 13 counts, 5 variants each, 1 for the empty store.
        let formats = if cfg!(debug_assertions) { 1 } else { 2 };
        assert_eq!(cases, formats * (12 * 5 + 1));
    }

    /// Serves `data[..end]` in pieces of seeded length, with seeded
    /// `Interrupted` errors, then either ends or fails.
    struct ScriptedReader<'a> {
        data: &'a [u8],
        pos: usize,
        end: usize,
        fault: Option<std::io::ErrorKind>,
        rng: u64,
    }

    impl Read for ScriptedReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.rng = splitmix64(self.rng);
            if self.rng % 5 == 0 {
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            if self.pos >= self.end {
                return match self.fault {
                    Some(kind) => Err(std::io::Error::new(kind, "scripted fault")),
                    None => Ok(0),
                };
            }
            let piece =
                [1, 7, 127, 128, 129, 4096, 1 << 20, usize::MAX][(self.rng >> 8) as usize % 8];
            let n = buf.len().min(self.end - self.pos).min(piece);
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Ok(n)
        }
    }

    /// Accepts pieces of seeded length with seeded `Interrupted` errors and
    /// fails (or accepts nothing) once `limit` bytes are taken.
    struct ScriptedWriter {
        taken: Vec<u8>,
        limit: usize,
        fault: Option<std::io::ErrorKind>,
        rng: u64,
    }

    impl Write for ScriptedWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.rng = splitmix64(self.rng);
            if self.rng % 5 == 0 {
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            if self.taken.len() >= self.limit {
                return match self.fault {
                    Some(kind) => Err(std::io::Error::new(kind, "scripted fault")),
                    None => Ok(0),
                };
            }
            let piece =
                [1, 7, 127, 128, 129, 4096, 1 << 20, usize::MAX][(self.rng >> 8) as usize % 8];
            let n = buf.len().min(self.limit - self.taken.len()).min(piece);
            self.taken.extend_from_slice(&buf[..n]);
            Ok(n)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn payload_check(records: &[Orb1024], legacy: bool, payload: &[u8]) -> PayloadCheck {
        if legacy {
            PayloadCheck::V1 {
                record_count: records.len() as u64,
                expected_checksum: checksum64(payload),
                checksum: Checksum64::new(),
                crc: Crc64Ecma::new(),
            }
        } else {
            PayloadCheck::V2 {
                header: Header::new(records.len() as u64, Generation(3), crc64_ecma(payload)),
                crc: Crc64Ecma::new(),
            }
        }
    }

    /// Runs both readers over the same scripted stream and returns their
    /// outcomes: read result, digest verdict, decoded records.
    fn both_reads(
        records: &[Orb1024],
        stream: &[u8],
        end: usize,
        fault: Option<std::io::ErrorKind>,
        legacy: bool,
        load: bool,
        seed: u64,
    ) -> [(String, String, Vec<Orb1024>); 2] {
        let payload: Vec<u8> = records.iter().flat_map(|orb| orb.to_le_bytes()).collect();
        let mut out = Vec::new();
        for reference in [false, true] {
            let mut reader = ScriptedReader {
                data: stream,
                pos: 0,
                end,
                fault,
                rng: seed ^ u64::from(reference),
            };
            let mut check = payload_check(records, legacy, &payload);
            let mut orbs = Vec::new();
            let sink = if load { Some(&mut orbs) } else { None };
            let result = if reference {
                reference_stream_payload(&mut reader, records.len(), &mut check, sink)
            } else {
                stream_payload(&mut reader, records.len(), &mut check, sink)
            };
            // On a read error the callers return it and drop the digests and
            // the records, so only the error itself is observable.
            out.push(match result {
                Ok(()) => ("Ok(())".to_string(), format!("{:?}", check.finish()), orbs),
                Err(error) => (format!("Err({error:?})"), String::new(), Vec::new()),
            });
        }
        [out.remove(0), out.remove(0)]
    }

    #[test]
    fn block_reads_match_record_reads_on_short_reads_faults_and_ends() {
        let kinds = [
            None,
            Some(std::io::ErrorKind::Other),
            Some(std::io::ErrorKind::PermissionDenied),
        ];
        let mut cases = 0usize;
        for (index, count) in edge_counts().into_iter().enumerate() {
            let records = seeded_orbs(count, 5_000 + index as u64);
            let payload: Vec<u8> = records.iter().flat_map(|orb| orb.to_le_bytes()).collect();
            let mut corrupt = payload.clone();
            if let Some(byte) = corrupt.last_mut() {
                *byte ^= 1;
            }
            for trial in 0..6u64 {
                let seed = splitmix64(((index as u64) << 8) | trial);
                let len = payload.len();
                let end = match trial {
                    0 | 1 => len,
                    2 => len.saturating_sub(1),
                    3 => len.saturating_sub(ORB_BYTES),
                    _ => (seed % (len as u64 + 1)) as usize,
                };
                let combos = [(false, false), (false, true), (true, false), (true, true)];
                let modes = kinds.iter().flat_map(|&fault| {
                    combos
                        .iter()
                        .map(move |&(legacy, load)| (fault, legacy, load))
                });
                // Release builds run every mode; debug builds (slow bitwise
                // CRC) rotate through them, one per trial.
                let pick = (index * 6 + trial as usize) % 12;
                for (mode, (fault, legacy, load)) in modes.enumerate() {
                    if cfg!(debug_assertions) && mode != pick {
                        continue;
                    }
                    let stream = if trial == 1 { &corrupt } else { &payload };
                    let [actual, expected] =
                        both_reads(&records, stream, end, fault, legacy, load, seed);
                    assert_eq!(
                        actual, expected,
                        "{count} records, end {end}, fault {fault:?}, legacy {legacy}, load {load}"
                    );
                    if end == len && trial == 0 {
                        assert_eq!(actual.0, "Ok(())");
                        assert!(actual.1.starts_with("Ok("));
                        if load {
                            assert_eq!(actual.2, records);
                        }
                    }
                    cases += 1;
                }
            }
        }
        let modes = if cfg!(debug_assertions) { 1 } else { 3 * 4 };
        assert_eq!(cases, 13 * 6 * modes);
    }

    #[test]
    fn block_writes_match_record_writes_on_short_writes_and_faults() {
        let kinds = [
            None,
            Some(std::io::ErrorKind::Other),
            Some(std::io::ErrorKind::StorageFull),
        ];
        let mut cases = 0usize;
        for (index, count) in edge_counts().into_iter().enumerate() {
            let records = seeded_orbs(count, 9_000 + index as u64);
            let len = count * ORB_BYTES;
            for trial in 0..5u64 {
                let seed = splitmix64(((index as u64) << 8) | trial | 0x8000);
                let limit = match trial {
                    0 => usize::MAX,
                    1 => len,
                    2 => len.saturating_sub(1),
                    _ => (seed % (len as u64 + 1)) as usize,
                };
                for fault in kinds {
                    let mut outcomes = Vec::new();
                    for reference in [false, true] {
                        let mut writer = ScriptedWriter {
                            taken: Vec::new(),
                            limit,
                            fault,
                            rng: seed ^ u64::from(reference),
                        };
                        let result = if reference {
                            reference_write_payload(&mut writer, &records)
                        } else {
                            write_payload(&mut writer, &records)
                        };
                        outcomes.push((format!("{result:?}"), writer.taken));
                    }
                    assert_eq!(
                        outcomes[0], outcomes[1],
                        "{count} records, limit {limit}, fault {fault:?}"
                    );
                    if limit >= len {
                        assert_eq!(outcomes[0].0, "Ok(())");
                        assert_eq!(outcomes[0].1.len(), len);
                    }
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 13 * 5 * 3);
    }

    /// The largest store the default budget admits, 2,097,152 records
    /// (256 MiB). Release builds only: the bitwise CRC makes it slow
    /// without optimization.
    #[test]
    fn block_reads_match_record_reads_at_the_default_budget() {
        if cfg!(debug_assertions) {
            return;
        }
        let count = DEFAULT_MAX_RECORDS as usize;
        let records = seeded_orbs(count, 77);
        let mut payload = Vec::with_capacity(count * ORB_BYTES);
        reference_write_payload(&mut payload, &records).unwrap();
        let mut blocked = Vec::with_capacity(count * ORB_BYTES);
        write_payload(&mut blocked, &records).unwrap();
        assert!(blocked == payload);
        drop(blocked);
        let len = payload.len();
        for (end, load) in [
            (len, true),
            (len - 1, false),
            (len - ORB_BYTES * 3 + 5, false),
        ] {
            let [actual, expected] = both_reads(&records, &payload, end, None, false, load, 3);
            assert!(actual == expected, "end {end}");
            if end == len {
                assert_eq!(actual.0, "Ok(())");
                assert!(actual.1.starts_with("Ok("));
                assert!(actual.2 == records);
            }
        }
    }

    // ---- overlapped reads against the previous block reader ------------------

    /// The block reader before reads overlapped digests: one buffer of at
    /// most 8 MiB; each block is read, digested and decoded before the next
    /// read.
    fn previous_stream_payload<R: Read>(
        reader: &mut R,
        count: usize,
        check: &mut PayloadCheck,
        mut orbs: Option<&mut Vec<Orb1024>>,
    ) -> Result<(), GelError> {
        let mut buffer = vec![0u8; count.min(READ_BLOCK_RECORDS) * ORB_BYTES];
        let mut remaining = count;
        while remaining > 0 {
            let records = remaining.min(READ_BLOCK_RECORDS);
            let block = &mut buffer[..records * ORB_BYTES];
            reader.read_exact(block)?;
            check.update(block);
            if let Some(orbs) = orbs.as_deref_mut() {
                for record in block.chunks_exact(ORB_BYTES) {
                    orbs.push(Orb1024::from_le_bytes(record)?);
                }
            }
            remaining -= records;
        }
        Ok(())
    }

    /// The readers compared: the current one with and without threads, the
    /// previous block reader and the record-at-a-time reader.
    const READERS: [&str; 4] = ["current", "current_one_thread", "previous", "record"];

    /// One scripted read: the expected records and payload, the stream served
    /// up to `end` and then `fault`, the header format and whether to load.
    struct Script<'a> {
        records: &'a [Orb1024],
        payload: &'a [u8],
        stream: &'a [u8],
        end: usize,
        fault: Option<std::io::ErrorKind>,
        legacy: bool,
        load: bool,
        seed: u64,
    }

    /// Runs reader `which` over the scripted stream and returns its outcome
    /// as in [`both_reads`].
    fn scripted_read(which: usize, script: &Script<'_>) -> (String, String, Vec<Orb1024>) {
        let mut reader = ScriptedReader {
            data: script.stream,
            pos: 0,
            end: script.end,
            fault: script.fault,
            rng: script.seed ^ which as u64,
        };
        let mut check = payload_check(script.records, script.legacy, script.payload);
        let mut orbs = Vec::new();
        let sink = if script.load { Some(&mut orbs) } else { None };
        let count = script.records.len();
        let result = match READERS[which] {
            "current" => stream_payload(&mut reader, count, &mut check, sink),
            "current_one_thread" => {
                stream_payload_with(&mut reader, count, &mut check, sink, false)
            }
            "previous" => previous_stream_payload(&mut reader, count, &mut check, sink),
            _ => reference_stream_payload(&mut reader, count, &mut check, sink),
        };
        match result {
            Ok(()) => ("Ok(())".to_string(), format!("{:?}", check.finish()), orbs),
            Err(error) => (format!("Err({error:?})"), String::new(), Vec::new()),
        }
    }

    /// Record counts around the read-block edges, where reads overlap.
    fn read_block_counts() -> Vec<usize> {
        let r = READ_BLOCK_RECORDS;
        vec![r - 1, r, r + 1, 2 * r - 1, 2 * r, 2 * r + 1, 3 * r + 7]
    }

    #[test]
    fn overlapped_reads_match_previous_and_record_reads_across_read_blocks() {
        let r = READ_BLOCK_RECORDS;
        let block = r * ORB_BYTES;
        let kinds = [None, Some(std::io::ErrorKind::Other)];
        let mut cases = 0usize;
        for (index, count) in read_block_counts().into_iter().enumerate() {
            // Debug builds (unoptimized digests) take three of the counts
            // and every other trial of each.
            if cfg!(debug_assertions) && ![1, 2, 5].contains(&index) {
                continue;
            }
            let records = seeded_orbs(count, 15_000 + index as u64);
            let payload: Vec<u8> = records.iter().flat_map(|orb| orb.to_le_bytes()).collect();
            let len = payload.len();
            let mut corrupt_last = payload.clone();
            corrupt_last[len - 1] ^= 0x10;
            let mut corrupt_first = payload.clone();
            corrupt_first[ORB_BYTES + 3] ^= 0x01;
            // (stream, end): whole streams, ends inside the first block, on
            // and around the first block edge, inside the last block.
            let mut trials: Vec<(&[u8], usize)> = vec![
                (&payload, len),
                (&corrupt_last, len),
                (&corrupt_first, len),
                (&payload, len - 1),
                (&payload, len - ORB_BYTES),
                (&payload, block / 2),
            ];
            for end in [block - 1, block, block + 1] {
                if end < len {
                    trials.push((&payload, end));
                }
            }
            let seed = splitmix64(0x0b5e_0000 + index as u64);
            trials.push((&payload, (seed % len as u64) as usize));
            for (trial, &(stream, end)) in trials.iter().enumerate() {
                if cfg!(debug_assertions) && (trial + index) % 2 != 0 {
                    continue;
                }
                let modes: Vec<_> = kinds
                    .iter()
                    .flat_map(|&fault| {
                        [(false, false), (false, true), (true, false), (true, true)]
                            .map(|(legacy, load)| (fault, legacy, load))
                    })
                    .collect();
                // Release builds run every mode; debug builds rotate through
                // them, one per trial.
                let pick = (index * 11 + trial) % modes.len();
                for (mode, &(fault, legacy, load)) in modes.iter().enumerate() {
                    if cfg!(debug_assertions) && mode != pick {
                        continue;
                    }
                    let script = Script {
                        records: &records,
                        payload: &payload,
                        stream,
                        end,
                        fault,
                        legacy,
                        load,
                        seed: splitmix64(
                            ((index as u64) << 16) | ((trial as u64) << 8) | mode as u64,
                        ),
                    };
                    let outcomes: Vec<_> = (0..READERS.len())
                        .map(|which| scripted_read(which, &script))
                        .collect();
                    for (which, outcome) in outcomes.iter().enumerate().skip(1) {
                        assert!(
                            *outcome == outcomes[0],
                            "{} differs: {count} records, trial {trial}, end {end}, \
                             fault {fault:?}, legacy {legacy}, load {load}",
                            READERS[which]
                        );
                    }
                    let outcome = &outcomes[0];
                    if end == len && std::ptr::eq(stream, payload.as_slice()) {
                        assert_eq!(outcome.0, "Ok(())");
                        assert!(outcome.1.starts_with("Ok("));
                        if load {
                            assert!(outcome.2 == records);
                        }
                    } else if end == len {
                        assert_eq!(outcome.0, "Ok(())");
                        assert_eq!(outcome.1, "Err(CorruptStore)");
                    } else {
                        assert!(outcome.0.starts_with("Err(Io("), "{}", outcome.0);
                    }
                    cases += 1;
                }
            }
        }
        // Edge ends below the length: none for r-1 records, one for r, three
        // for the five longer counts; plus 6 fixed trials and 1 seeded each.
        // Debug: 4 of the 8 trials for r records, 5 of 10 for r+1 and 2r+1.
        let expected = if cfg!(debug_assertions) {
            4 + 5 + 5
        } else {
            (7 + 8 + 5 * 10) * 8
        };
        assert_eq!(cases, expected);
    }

    #[test]
    fn ragged_last_block_matches_the_previous_reader() {
        let r = READ_BLOCK_RECORDS;
        let block = r * ORB_BYTES;
        let modes = [(false, false), (false, true), (true, false), (true, true)];
        let mut cases = 0usize;
        // A last block of one record after two whole blocks, and of seven
        // records after eight.
        for (index, (count, tail)) in [(2 * r + 1, 1usize), (8 * r + 7, 7)]
            .into_iter()
            .enumerate()
        {
            let records = seeded_orbs(count, 19_000 + index as u64);
            let payload: Vec<u8> = records.iter().flat_map(|orb| orb.to_le_bytes()).collect();
            let len = payload.len();
            assert_eq!(len % block, tail * ORB_BYTES);
            let mut corrupt_last = payload.clone();
            corrupt_last[len - 1] ^= 0x04;
            let last_block = len - tail * ORB_BYTES;
            // (stream, end, fault): whole streams, a fault on the edge of
            // the last block and an early end inside it.
            let trials: [(&[u8], usize, Option<std::io::ErrorKind>); 4] = [
                (&payload, len, None),
                (&corrupt_last, len, None),
                (&payload, last_block, Some(std::io::ErrorKind::Other)),
                (&payload, len - 1, None),
            ];
            for (trial, &(stream, end, fault)) in trials.iter().enumerate() {
                // Debug builds (unoptimized digests) take every other trial
                // of the longer count, one mode per trial, a different one
                // for each trial.
                if cfg!(debug_assertions) && index == 1 && trial % 2 != 0 {
                    continue;
                }
                for (mode, &(legacy, load)) in modes.iter().enumerate() {
                    if cfg!(debug_assertions) && mode != (trial + index) % modes.len() {
                        continue;
                    }
                    let script = Script {
                        records: &records,
                        payload: &payload,
                        stream,
                        end,
                        fault,
                        legacy,
                        load,
                        seed: splitmix64(0x7a11_0000 + ((index as u64) << 8) + trial as u64),
                    };
                    let outcomes: Vec<_> = (0..READERS.len())
                        .map(|which| scripted_read(which, &script))
                        .collect();
                    for (which, outcome) in outcomes.iter().enumerate().skip(1) {
                        assert!(
                            *outcome == outcomes[0],
                            "{} differs: {count} records, trial {trial}, legacy {legacy}, \
                             load {load}",
                            READERS[which]
                        );
                    }
                    let outcome = &outcomes[0];
                    match trial {
                        0 => {
                            assert_eq!(outcome.0, "Ok(())");
                            assert!(outcome.1.starts_with("Ok("), "{}", outcome.1);
                            assert!(!load || outcome.2 == records);
                        }
                        1 => {
                            assert_eq!(outcome.0, "Ok(())");
                            assert_eq!(outcome.1, "Err(CorruptStore)");
                        }
                        _ => assert!(outcome.0.starts_with("Err(Io("), "{}", outcome.0),
                    }
                    cases += 1;
                }
            }
        }
        let expected = if cfg!(debug_assertions) {
            4 + 2
        } else {
            2 * 4 * 4
        };
        assert_eq!(cases, expected);
    }

    #[test]
    fn overlapped_open_and_verify_match_the_previous_reader_on_files() {
        let path = path("overlapped-read-files");
        let r = READ_BLOCK_RECORDS;
        for (index, count) in [r, r + 1, 2 * r + 3].into_iter().enumerate() {
            let records = seeded_orbs(count, 17_000 + index as u64);
            let generation = 60 + index as u64;
            let v2 = v2_file_bytes(&records, generation);
            let v1 = legacy_v1_file_bytes(&records, generation);
            for (format, original) in [v2, v1].into_iter().enumerate() {
                let mut flipped = original.clone();
                let at = original.len() - 1 - (index * 977) % ORB_BYTES;
                flipped[at] ^= 0x80;
                for (variant, bytes) in [original, flipped].iter().enumerate() {
                    fs::write(&path, bytes).unwrap();
                    let expected = reference_open_and_verify(&path);
                    let actual = current_open_and_verify(&path);
                    assert!(
                        actual == expected,
                        "{count} records, format {format}, variant {variant}"
                    );
                    if variant == 0 {
                        assert!(actual.2 == records);
                    } else {
                        assert_eq!(actual.0, "CorruptStore");
                        assert_eq!(actual.1, "Err(CorruptStore)");
                    }
                }
            }
        }
        let _ = fs::remove_file(path);
    }

    #[test]
    fn threaded_record_crc_matches_the_serial_loop_for_every_thread_count() {
        let mut cases = 0usize;
        for count in [0usize, 1, 2, 3, 7, 16, 17, 100, 257] {
            let records = seeded_orbs(count, 12_000 + count as u64);
            let expected = crc_orbs_serial(&records);
            let payload: Vec<u8> = records.iter().flat_map(|orb| orb.to_le_bytes()).collect();
            assert_eq!(expected, crc64_ecma(&payload), "{count} records");
            for threads in (0..=17usize).chain([count, count + 1, usize::MAX]) {
                assert_eq!(
                    crc_orbs_with_threads(&records, threads),
                    expected,
                    "{count} records, {threads} threads"
                );
                cases += 1;
            }
        }
        assert_eq!(cases, 9 * 21);
        // The automatic path, below and above the threshold.
        for count in [
            CRC_PARALLEL_SEGMENT_RECORDS,
            CRC64_PARALLEL_MIN_BYTES / ORB_BYTES - 1,
            CRC64_PARALLEL_MIN_BYTES / ORB_BYTES,
            CRC64_PARALLEL_MIN_BYTES / ORB_BYTES + 3,
        ] {
            let records = seeded_orbs(count, 13_000 + count as u64);
            assert_eq!(crc_orbs(&records), crc_orbs_serial(&records), "{count}");
        }
    }
}
