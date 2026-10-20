#![forbid(unsafe_code)]

//! Identity check for the store payload CRC paths: the multi-thread record
//! CRC of `RamStore::from_orbs` and the 8 MiB read blocks of `verify_file`
//! and `open_verified_with_limits`.
//!
//! `mod reference` is an exact copy of the earlier code (one 128-byte CRC
//! update per record when building, 1 MiB read blocks when reading), built
//! from public items only. Every case compares the library result with the
//! reference result on seeded and edge-case stores: headers, file bytes,
//! `Result` values including the error value (the first error reported, so
//! error order is covered), loaded records, and `write_if_newer` outcomes.
//!
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` line. FAIL if any value
//! differs, if any case ran zero comparisons, or if no case ran. Exit code 0
//! only on PASS, 1 on FAIL, 2 on a bad argument.
//!
//! Usage: `identity_store_crc [DIR]` (default: the OS temporary directory).
//! Files are written to a fresh subdirectory that is removed afterwards.
//! Output grammar: `docs/SPEED.md`.

use gel_core::{splitmix64, GelError, Generation, ORB_BYTES, ORB_WORDS};
use gel_orb::Orb1024;
use gel_store::{
    verify_file, Header, OpenLimits, RamStore, StoreFormat, Verification, DEFAULT_MAX_RECORDS,
    HEADER_BYTES,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// An exact copy of the earlier gel-store code paths, from public items.
pub mod reference {
    use gel_core::{
        Checksum64, Crc64Ecma, GelError, Generation, GEL_MAGIC_V1, GEL_MAGIC_V2, ORB_BITS,
        ORB_BYTES,
    };
    use gel_orb::Orb1024;
    use gel_store::{Header, OpenLimits, StoreFormat, Verification, HEADER_BYTES};
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    /// The earlier `crc_orbs`.
    pub fn crc_orbs(orbs: &[Orb1024]) -> u64 {
        let mut crc = Crc64Ecma::new();
        for orb in orbs {
            crc.update(&orb.to_le_bytes());
        }
        crc.finish()
    }

    /// The header `RamStore::from_orbs` carried.
    pub fn header(orbs: &[Orb1024], generation: Generation) -> Header {
        Header::new(orbs.len() as u64, generation, crc_orbs(orbs))
    }

    /// The bytes `write_atomic` wrote: header, then every record.
    pub fn file_bytes(orbs: &[Orb1024], generation: Generation) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_BYTES + orbs.len() * ORB_BYTES);
        bytes.extend_from_slice(&header(orbs, generation).encode());
        for orb in orbs {
            bytes.extend_from_slice(&orb.to_le_bytes());
        }
        bytes
    }

    const IO_BLOCK_RECORDS: usize = 8192;

    fn stream_payload<R: Read>(
        reader: &mut R,
        count: usize,
        check: &mut PayloadCheck,
        mut orbs: Option<&mut Vec<Orb1024>>,
    ) -> Result<(), GelError> {
        let mut buffer = vec![0u8; count.min(IO_BLOCK_RECORDS) * ORB_BYTES];
        let mut remaining = count;
        while remaining > 0 {
            let records = remaining.min(IO_BLOCK_RECORDS);
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

    struct Validated {
        file: File,
        count: usize,
        check: PayloadCheck,
    }

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
                        header: Header::new(record_count, Generation(0), crc.finish()),
                    })
                }
            }
        }
    }

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
        if header_bytes[20..24].iter().any(|&b| b != 0)
            || header_bytes[48..64].iter().any(|&b| b != 0)
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

    /// The earlier `open_verified_with_limits`, as header, format, records.
    pub fn open_verified_with_limits(
        path: &Path,
        limits: OpenLimits,
    ) -> Result<(Header, StoreFormat, Vec<Orb1024>), GelError> {
        let Validated {
            mut file,
            count,
            mut check,
        } = open_validated(path, limits)?;
        let mut orbs = Vec::new();
        orbs.try_reserve_exact(count)
            .map_err(|_| GelError::AllocationFailed)?;
        stream_payload(&mut file, count, &mut check, Some(&mut orbs))?;
        let verification = check.finish()?;
        Ok((verification.header, verification.source_format, orbs))
    }

    /// The earlier `verify_file`.
    pub fn verify_file(path: &Path, limits: OpenLimits) -> Result<Verification, GelError> {
        let Validated {
            mut file,
            count,
            mut check,
        } = open_validated(path, limits)?;
        stream_payload(&mut file, count, &mut check, None)?;
        check.finish()
    }

    fn read_generation(path: &Path) -> Result<Generation, GelError> {
        let verification = verify_file(path, OpenLimits::UNLIMITED)?;
        match verification.source_format {
            StoreFormat::V2 => Ok(verification.header.generation),
            StoreFormat::LegacyV1 => Err(GelError::LegacyGenerationUntrusted),
        }
    }

    /// The gate of the earlier `write_if_newer`: `Ok(())` when it went on
    /// to write.
    pub fn write_if_newer_gate(path: &Path, generation: Generation) -> Result<(), GelError> {
        if path.exists() {
            let current = read_generation(path)?;
            if generation.0 <= current.0 {
                return Err(GelError::GenerationRollback {
                    current: current.0,
                    attempted: generation.0,
                });
            }
        }
        Ok(())
    }

    fn le_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed slice"))
    }

    fn le_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed slice"))
    }
}

// `Scale::Full` covers every record count and file variant listed in the
// header comment; `Scale::Small` is a short subset for `cargo test` in debug
// builds.

const MIB: usize = 1 << 20;

#[derive(Clone, Copy)]
enum Fill {
    Seeded(u64),
    Zeros,
    Ones,
}

fn records(count: usize, fill: Fill) -> Vec<Orb1024> {
    (0..count)
        .map(|i| {
            let mut words = [0u64; ORB_WORDS];
            for (w, word) in words.iter_mut().enumerate() {
                *word = match fill {
                    Fill::Seeded(seed) => splitmix64(
                        seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((w as u64) << 56),
                    ),
                    Fill::Zeros => 0,
                    Fill::Ones => u64::MAX,
                };
            }
            Orb1024::from_words(words)
        })
        .collect()
}

fn counts(scale: Scale) -> Vec<usize> {
    let t = 4 * MIB / ORB_BYTES; // 32768: the multi-thread threshold
    let b = 8 * MIB / ORB_BYTES; // 65536: one read block
    match scale {
        Scale::Small => vec![0, 1, 8193, t + 1, b + 1],
        Scale::Full => {
            let mut counts = vec![
                0,
                1,
                2,
                8191,
                8192,
                8193,
                t - 1,
                t,
                t + 1,
                40_000,
                49_151,
                57_344,
                b - 1,
                b,
                b + 1,
                2 * b + 1,
                300_000,
            ];
            for seed in 0..4u64 {
                counts.push((splitmix64(seed + 0x5702) % 400_000) as usize);
            }
            counts.push(DEFAULT_MAX_RECORDS as usize);
            counts.push(DEFAULT_MAX_RECORDS as usize + 1);
            counts
        }
    }
}

/// The v1 file bytes for `orbs` (legacy checksum, unprotected generation).
fn v1_file_bytes(orbs: &[Orb1024], generation: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(orbs.len() * ORB_BYTES);
    for orb in orbs {
        payload.extend_from_slice(&orb.to_le_bytes());
    }
    let mut bytes = vec![0u8; HEADER_BYTES];
    bytes[0..8].copy_from_slice(&gel_core::GEL_MAGIC_V1);
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&(gel_core::ORB_BITS as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&(ORB_BYTES as u32).to_le_bytes());
    bytes[24..32].copy_from_slice(&(orbs.len() as u64).to_le_bytes());
    bytes[32..40].copy_from_slice(&generation.to_le_bytes());
    bytes[40..48].copy_from_slice(&gel_core::checksum64(&payload).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

/// Replaces the v2 header of `bytes` by `header` re-encoded (valid CRC).
fn with_header(bytes: &[u8], header: Header) -> Vec<u8> {
    let mut out = bytes.to_vec();
    out[..HEADER_BYTES].copy_from_slice(
        &Header::new(header.record_count, header.generation, header.payload_crc64).encode(),
    );
    out
}

/// File variants of a store: the original, payload flips (at block and
/// threshold edges), length changes, header damage, and several faults at
/// once (only the first reported error is observable).
fn variants(original: &[u8], legacy: bool, full: bool) -> Vec<Vec<u8>> {
    let payload = original.len() - HEADER_BYTES;
    let mut out = vec![original.to_vec()];
    let flip = |bytes: &[u8], at: usize| {
        let mut v = bytes.to_vec();
        v[at] ^= 0x5A;
        v
    };
    let big = payload > 40 * MIB;
    if payload > 0 {
        let mut offsets = vec![0, payload / 2, payload - 1];
        if full && !big {
            for edge in [MIB, 4 * MIB, 8 * MIB, 16 * MIB] {
                for at in [edge - 1, edge, edge + 1] {
                    offsets.push(at);
                }
            }
        } else {
            offsets.push(8 * MIB);
        }
        for at in offsets {
            if at < payload {
                out.push(flip(original, HEADER_BYTES + at));
            }
        }
    }
    // Length changes.
    let mut short = original.to_vec();
    short.pop();
    out.push(short);
    if payload >= ORB_BYTES && !big {
        out.push(original[..original.len() - ORB_BYTES].to_vec());
    }
    if !big {
        let mut long = original.to_vec();
        long.push(0);
        out.push(long);
    }
    // Header damage.
    let header_offsets: &[usize] = if legacy {
        &[0, 8, 12, 20, 40, 48, 56]
    } else {
        &[0, 8, 12, 20, 24, 40, 48, 56]
    };
    for &at in header_offsets {
        if full || at == 40 || at == 48 {
            out.push(flip(original, at));
        }
    }
    if !legacy {
        let mut header_bytes = [0u8; HEADER_BYTES];
        header_bytes.copy_from_slice(&original[..HEADER_BYTES]);
        let header = Header::decode(&header_bytes).expect("own header");
        // Valid header CRC, wrong payload CRC: CorruptStore after the read.
        let mut wrong_crc = header;
        wrong_crc.payload_crc64 ^= 1;
        out.push(with_header(original, wrong_crc));
        // Valid header CRC, one more record: length error before the read.
        let mut wrong_count = header;
        wrong_count.record_count += 1;
        out.push(with_header(original, wrong_count));
        // Several faults at once: wrong payload CRC and short file.
        let mut both = with_header(original, wrong_crc);
        both.pop();
        out.push(both);
        // Payload flip and header flip: the header error comes first.
        if payload > 0 {
            let mut both = flip(original, HEADER_BYTES + payload - 1);
            both[20] ^= 1;
            out.push(both);
        }
        if full && !big {
            // Record count whose payload size overflows u64, valid CRC.
            let mut overflow = header;
            overflow.record_count = u64::MAX / ORB_BYTES as u64 + 1;
            out.push(with_header(original, overflow));
            let mut huge = header;
            huge.record_count = u64::MAX;
            out.push(with_header(original, huge));
        }
    } else if payload > 0 {
        // Checksum flip and payload flip together.
        let mut both = flip(original, 40);
        both[HEADER_BYTES] ^= 0x01;
        out.push(both);
    }
    out
}

fn limit_sets(count: usize, len: usize, full: bool) -> Vec<OpenLimits> {
    let mut out = vec![OpenLimits::UNLIMITED];
    if full {
        out.push(OpenLimits::SAFE_DEFAULT);
        out.push(OpenLimits {
            max_records: (count as u64).saturating_sub(1),
            max_file_bytes: u64::MAX,
        });
        out.push(OpenLimits {
            max_records: u64::MAX,
            max_file_bytes: (len as u64).saturating_sub(1),
        });
        out.push(OpenLimits {
            max_records: count as u64,
            max_file_bytes: len as u64,
        });
    }
    out
}

type Loaded = Result<(Header, StoreFormat, Vec<Orb1024>), GelError>;

fn open_new(path: &Path, limits: OpenLimits) -> Loaded {
    RamStore::open_verified_with_limits(path, limits)
        .map(|store| (store.header(), store.source_format(), store.orbs().to_vec()))
}

/// Compares verify and open with the reference on every limit set.
fn compare_file(
    path: &Path,
    count: usize,
    len: usize,
    full: bool,
    tallies: [&mut Tally; 2],
    seen: &mut BTreeSet<&'static str>,
) {
    let [verify, open] = tallies;
    for limits in limit_sets(count, len, full) {
        let new: Result<Verification, GelError> = verify_file(path, limits);
        let old = reference::verify_file(path, limits);
        seen.insert(kind(&old));
        verify.check(new == old);
        let new = open_new(path, limits);
        let old = reference::open_verified_with_limits(path, limits);
        seen.insert(kind(&old));
        open.check(new == old);
    }
}

/// The outcome kind of a reference result, to show which outcomes the
/// comparisons actually reached.
fn kind<T>(result: &Result<T, GelError>) -> &'static str {
    match result {
        Ok(_) => "Ok",
        Err(GelError::InvalidLength { .. }) => "InvalidLength",
        Err(GelError::InvalidMagic) => "InvalidMagic",
        Err(GelError::UnsupportedVersion(_)) => "UnsupportedVersion",
        Err(GelError::InvalidHeader(_)) => "InvalidHeader",
        Err(GelError::CorruptHeader) => "CorruptHeader",
        Err(GelError::CorruptStore) => "CorruptStore",
        Err(GelError::AllocationFailed) => "AllocationFailed",
        Err(GelError::InvalidView(_)) => "InvalidView",
        Err(GelError::InvalidResidual(_)) => "InvalidResidual",
        Err(GelError::GenerationRollback { .. }) => "GenerationRollback",
        Err(GelError::LegacyGenerationUntrusted) => "LegacyGenerationUntrusted",
        Err(GelError::LimitExceeded(_)) => "LimitExceeded",
        Err(GelError::Io(_)) => "Io",
    }
}

/// Outcome kinds every run must reach; a run that misses one compared
/// fewer paths than it claims.
fn required_kinds(scale: Scale) -> &'static [&'static str] {
    match scale {
        Scale::Small => &[
            "Ok",
            "CorruptStore",
            "CorruptHeader",
            "InvalidLength",
            "InvalidHeader",
            "Io",
            "GenerationRollback",
            "LegacyGenerationUntrusted",
        ],
        Scale::Full => &[
            "Ok",
            "CorruptStore",
            "CorruptHeader",
            "InvalidLength",
            "InvalidHeader",
            "InvalidMagic",
            "UnsupportedVersion",
            "LimitExceeded",
            "Io",
            "GenerationRollback",
            "LegacyGenerationUntrusted",
        ],
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write variant");
}

/// The comparison in a fresh subdirectory of the OS temporary directory.
pub fn run(scale: Scale) -> Vec<Tally> {
    run_in(scale, &std::env::temp_dir())
}

/// The comparison in a fresh subdirectory of `dir`.
pub fn run_in(scale: Scale, dir: &Path) -> Vec<Tally> {
    let full = scale == Scale::Full;
    let work = dir.join(format!(
        "identity-store-crc-{}-{}",
        std::process::id(),
        splitmix64(std::process::id() as u64)
    ));
    fs::create_dir_all(&work).expect("create work directory");
    let path = work.join("store.gel");

    let mut build = Tally::new("build_header");
    let mut written = Tally::new("write_bytes");
    let mut verify_v2 = Tally::new("verify_v2");
    let mut open_v2 = Tally::new("open_v2");
    let mut verify_v1 = Tally::new("verify_v1");
    let mut open_v1 = Tally::new("open_v1");
    let mut newer = Tally::new("write_if_newer");
    let mut paths = Tally::new("paths");
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();

    for (index, count) in counts(scale).into_iter().enumerate() {
        let fills: Vec<Fill> = if count <= 70_000 && count > 0 {
            vec![Fill::Seeded(0xC0DE + index as u64), Fill::Zeros, Fill::Ones]
        } else {
            vec![Fill::Seeded(0xC0DE + index as u64)]
        };
        for (f, fill) in fills.into_iter().enumerate() {
            let orbs = records(count, fill);
            let generation = Generation(1 + index as u64 * 3 + f as u64);

            // Building: header and records.
            let store = RamStore::from_orbs(orbs.clone(), generation);
            let header_ref = reference::header(&orbs, generation);
            build.check(store.header() == header_ref && store.orbs() == orbs.as_slice());
            let _ = fs::remove_file(&path);
            store.write_atomic(&path).expect("write_atomic");
            let bytes = fs::read(&path).expect("read store");
            let expected = reference::file_bytes(&orbs, generation);
            written.check(bytes == expected);
            drop(store);
            if f > 0 {
                // Zero and one fills only check building and writing.
                continue;
            }

            // Reading v2 variants.
            for variant in variants(&expected, false, full) {
                write_bytes(&path, &variant);
                let tallies = [&mut verify_v2, &mut open_v2];
                compare_file(&path, count, variant.len(), full, tallies, &mut seen);
            }
            drop(expected);
            drop(bytes);

            // Reading v1 variants (the legacy checksum runs byte by byte).
            if count <= 400_000 {
                let legacy = v1_file_bytes(&orbs, generation.0);
                for variant in variants(&legacy, true, full) {
                    write_bytes(&path, &variant);
                    let tallies = [&mut verify_v1, &mut open_v1];
                    compare_file(&path, count, variant.len(), full, tallies, &mut seen);
                }
            }

            // write_if_newer over an existing file: valid v2, corrupt v2, v1.
            if count <= 140_000 {
                let replacement = records(count, Fill::Seeded(0xBEEF ^ index as u64));
                let existing_v2 = reference::file_bytes(&orbs, generation);
                let mut corrupt = existing_v2.clone();
                if corrupt.len() > HEADER_BYTES {
                    let last = corrupt.len() - 1;
                    corrupt[last] ^= 0x80;
                }
                let legacy = v1_file_bytes(&orbs, generation.0);
                let existing: [Option<&[u8]>; 4] = [
                    None,
                    Some(existing_v2.as_slice()),
                    Some(corrupt.as_slice()),
                    Some(legacy.as_slice()),
                ];
                for before in existing {
                    for attempted in [generation.0 - 1, generation.0, generation.0 + 1] {
                        let _ = fs::remove_file(&path);
                        if let Some(bytes) = before {
                            write_bytes(&path, bytes);
                        }
                        let candidate =
                            RamStore::from_orbs(replacement.clone(), Generation(attempted));
                        let result = candidate.write_if_newer(&path);
                        let after = fs::read(&path).ok();

                        let _ = fs::remove_file(&path);
                        if let Some(bytes) = before {
                            write_bytes(&path, bytes);
                        }
                        let expected_result =
                            reference::write_if_newer_gate(&path, Generation(attempted));
                        let expected_after = if expected_result.is_ok() {
                            Some(reference::file_bytes(&replacement, Generation(attempted)))
                        } else {
                            before.map(<[u8]>::to_vec)
                        };
                        seen.insert(kind(&expected_result));
                        newer.check(result == expected_result && after == expected_after);
                    }
                }
            }
        }
    }

    // Paths that fail before any payload: missing file, directory.
    let missing = work.join("missing.gel");
    for target in [missing.as_path(), work.as_path()] {
        for limits in [OpenLimits::UNLIMITED, OpenLimits::SAFE_DEFAULT] {
            let old = reference::verify_file(target, limits);
            seen.insert(kind(&old));
            paths.check(verify_file(target, limits) == old);
            let old = reference::open_verified_with_limits(target, limits);
            seen.insert(kind(&old));
            paths.check(open_new(target, limits) == old);
        }
    }
    let _ = fs::remove_dir_all(&work);

    // Every required outcome kind was reached by the reference.
    let mut kinds = Tally::new("outcome_kinds");
    for required in required_kinds(scale) {
        kinds.check(seen.contains(required));
    }
    vec![
        build, written, verify_v2, open_v2, verify_v1, open_v1, newer, paths, kinds,
    ]
}

fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let dir: Option<PathBuf> = args.next().map(PathBuf::from);
    if args.next().is_some() {
        eprintln!("usage: identity_store_crc [DIR]");
        return std::process::ExitCode::from(2);
    }
    match dir {
        None => identity::report(&run(Scale::Full)),
        Some(dir) => identity::report(&run_in(Scale::Full, &dir)),
    }
}
