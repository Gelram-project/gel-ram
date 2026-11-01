// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
#![forbid(unsafe_code)]

//! GEL RAM core contracts. No external dependencies.

use core::fmt;

pub const ORB_BITS: usize = 1024;
pub const ORB_WORDS: usize = ORB_BITS / 64;
pub const ORB_BYTES: usize = ORB_BITS / 8;
pub const GEL_FORMAT_VERSION: u32 = 2;
pub const GEL_MAGIC_V1: [u8; 8] = *b"GELORB01";
pub const GEL_MAGIC_V2: [u8; 8] = *b"GELORB02";
pub const GEL_MAGIC: [u8; 8] = GEL_MAGIC_V2;
pub const CRC64_ECMA_POLY: u64 = 0x42F0_E1EB_A9EA_3693;

// The license name and the Required Notice are written once, here; the
// constants below and the source header are all built from these two.
macro_rules! license_name {
    () => {
        "GEL RAM Noncommercial Reciprocal License 1.0"
    };
}
macro_rules! required_notice {
    () => {
        "Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)"
    };
}
macro_rules! source_header_line_1 {
    () => {
        concat!(
            "// GEL RAM — ",
            license_name!(),
            "; see LICENSE and NOTICE."
        )
    };
}
macro_rules! source_header_line_2 {
    () => {
        concat!("// ", required_notice!())
    };
}

/// Identifier of the license of GEL RAM-owned material in this workspace.
pub const LICENSE_ID: &str = "GEL-RAM-NCRL-1.0";
/// The full license name, the first line of the root LICENSE file.
pub const LICENSE_NAME: &str = license_name!();
/// The `Required Notice:` line of the root LICENSE and NOTICE files; LICENSE
/// section 8 names the `Required Notice:` lines among what each recipient
/// receives.
pub const REQUIRED_NOTICE: &str = required_notice!();
/// Line 1 of the source header of every GEL RAM-owned `.rs` file:
/// `// GEL RAM — ` + [`LICENSE_NAME`] + `; see LICENSE and NOTICE.`
pub const SOURCE_HEADER_LINE_1: &str = source_header_line_1!();
/// Line 2 of the source header: `// ` + [`REQUIRED_NOTICE`].
pub const SOURCE_HEADER_LINE_2: &str = source_header_line_2!();
/// Both source header lines, each ended by a line feed: the first bytes of
/// every GEL RAM-owned `.rs` file. The header carries the license name and the
/// Required Notice and should stay with the file.
pub const SOURCE_HEADER: &str =
    concat!(source_header_line_1!(), "\n", source_header_line_2!(), "\n");
/// The UTF-8 byte order mark. A source file that starts with it is refused by
/// the source-header gate and counted as altered by `gel-license check`.
pub const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
/// Fourth line of `--license`, built only from NOTICE wording; it adds no term.
const LICENSE_SUMMARY: &str = "Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com";
/// Last line of `--license`: third-party material compiled into a program keeps
/// its own license (LICENSE section 2, "Third-Party Material").
const THIRD_PARTY_LINE: &str = "Third-party material in this program, including the Unicode data compiled into it, remains under its own license: see THIRD-PARTY-NOTICES.md.";

/// Whether one line starts like a source header line: a plain `//` comment
/// (not a `//!` or `///` doc comment) whose text, after spaces and tabs, opens
/// with `GEL RAM` or `Required Notice`. A CR before the line's end is ignored.
/// The source-header gate of `xtask` and `gel-license check` both use this rule.
pub fn source_header_like(line: &[u8]) -> bool {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let Some(rest) = line.strip_prefix(b"//") else {
        return false;
    };
    let start = rest
        .iter()
        .position(|b| *b != b' ' && *b != b'\t')
        .unwrap_or(rest.len());
    let rest = &rest[start..];
    rest.starts_with(b"GEL RAM") || rest.starts_with(b"Required Notice")
}

/// The one line a GEL RAM tool prints for `--version`, without a newline:
/// `GEL_TOOL=<tool> VERSION=<version> LICENSE=GEL-RAM-NCRL-1.0`.
/// Binaries pass their own name and `env!("CARGO_PKG_VERSION")`.
pub fn tool_identity(tool: &str, version: &str) -> String {
    format!("GEL_TOOL={tool} VERSION={version} LICENSE={LICENSE_ID}")
}

/// The five lines a GEL RAM tool prints for `--license`, each ending in `\n`:
/// the [`tool_identity`] line, [`LICENSE_NAME`], [`REQUIRED_NOTICE`], the
/// NOTICE summary of what is permitted and where commercial terms are asked,
/// and a line saying that third-party material in the program keeps its own
/// license. The LICENSE file, not this text, holds the terms.
pub fn tool_license(tool: &str, version: &str) -> String {
    format!(
        "{}\n{LICENSE_NAME}\n{REQUIRED_NOTICE}\n{LICENSE_SUMMARY}\n{THIRD_PARTY_LINE}\n",
        tool_identity(tool, version)
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OrbId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Generation(pub u64);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GelError {
    InvalidLength { expected: usize, actual: usize },
    InvalidMagic,
    UnsupportedVersion(u32),
    InvalidHeader(&'static str),
    CorruptHeader,
    CorruptStore,
    AllocationFailed,
    InvalidView(&'static str),
    InvalidResidual(&'static str),
    GenerationRollback { current: u64, attempted: u64 },
    LegacyGenerationUntrusted,
    LimitExceeded(&'static str),
    Io(String),
}

impl fmt::Display for GelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(f, "invalid length: expected {expected}, got {actual}")
            }
            Self::InvalidMagic => write!(f, "invalid GEL magic"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported GEL format version {v}"),
            Self::InvalidHeader(msg) => write!(f, "invalid GEL header: {msg}"),
            Self::CorruptHeader => write!(f, "GEL header CRC64-ECMA mismatch"),
            Self::CorruptStore => write!(f, "GEL payload checksum mismatch"),
            Self::AllocationFailed => write!(f, "GEL store does not fit in available memory"),
            Self::InvalidView(msg) => write!(f, "invalid GEL reader view: {msg}"),
            Self::InvalidResidual(msg) => write!(f, "invalid structural residual: {msg}"),
            Self::GenerationRollback { current, attempted } => write!(
                f,
                "generation rollback rejected: current={current}, attempted={attempted}"
            ),
            Self::LimitExceeded(msg) => write!(f, "configured GEL open limit exceeded: {msg}"),
            Self::LegacyGenerationUntrusted => write!(f, "legacy v1 generation is unprotected; migrate explicitly before monotonic publication"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for GelError {}

impl From<std::io::Error> for GelError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

/// CRC-64/ECMA-182, poly=0x42F0E1EBA9EA3693, init=0, refin=false,
/// refout=false, xorout=0. The canonical check value for "123456789" is
/// 0x6C40DF5F0B497347.
#[inline]
pub fn crc64_ecma(bytes: &[u8]) -> u64 {
    let mut crc = Crc64Ecma::new();
    crc.update(bytes);
    crc.finish()
}

/// Byte tables for CRC-64/ECMA-182 (MSB-first). `[0][b]` is the CRC of
/// the single byte `b` from a zero register; `[k][b]` is `[k - 1][b]`
/// advanced by one more zero byte. Built at compile time.
static CRC64_TABLES: [[u64; 256]; 8] = crc64_tables();

const fn crc64_tables() -> [[u64; 256]; 8] {
    let mut tables = [[0u64; 256]; 8];
    let mut b = 0;
    while b < 256 {
        let mut crc = (b as u64) << 56;
        let mut bit = 0;
        while bit < 8 {
            crc = if (crc & 0x8000_0000_0000_0000) != 0 {
                (crc << 1) ^ CRC64_ECMA_POLY
            } else {
                crc << 1
            };
            bit += 1;
        }
        tables[0][b] = crc;
        b += 1;
    }
    let mut k = 1;
    while k < 8 {
        let mut b = 0;
        while b < 256 {
            let prev = tables[k - 1][b];
            tables[k][b] = (prev << 8) ^ tables[0][(prev >> 56) as usize];
            b += 1;
        }
        k += 1;
    }
    tables
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Crc64Ecma {
    state: u64,
}

impl Crc64Ecma {
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    /// Feeds `bytes` into the running CRC. Eight bytes are folded per step
    /// with eight 256-entry tables; the remaining tail goes one byte at a
    /// time. The state between calls is only the CRC register, so any split
    /// of the input gives the same result.
    ///
    /// A slice of [`CRC64_PARALLEL_MIN_BYTES`] or more is split into
    /// segments that are hashed on scoped threads and joined with
    /// [`crc64_ecma_combine`] in input order; the result is the same value
    /// as the one-thread path for every length and every thread count.
    pub fn update(&mut self, bytes: &[u8]) {
        if bytes.len() >= CRC64_PARALLEL_MIN_BYTES {
            let threads = crc64_auto_threads().min(bytes.len() / CRC64_PARALLEL_SEGMENT_MIN);
            if threads > 1 {
                self.update_with_threads(bytes, threads);
                return;
            }
        }
        self.state = crc64_update_serial(self.state, bytes);
    }

    /// Feeds `bytes` as at most `threads` consecutive segments of
    /// `ceil(len / threads)` bytes. The calling thread hashes the first
    /// segment from the running register; every other segment is hashed
    /// from a zero register on its own scoped thread, and the segment CRCs
    /// are joined left to right with [`crc64_ecma_combine`]. A segment
    /// whose thread cannot be started is hashed on the calling thread. The
    /// result equals [`Crc64Ecma::update`] for every input and every
    /// `threads`; `0` and `1` mean one thread.
    pub fn update_with_threads(&mut self, bytes: &[u8], threads: usize) {
        let threads = threads.max(1).min(bytes.len());
        if threads <= 1 {
            self.state = crc64_update_serial(self.state, bytes);
            return;
        }
        let segment = bytes.len().div_ceil(threads);
        let (first, rest) = bytes.split_at(segment);
        let start = self.state;
        self.state = std::thread::scope(|scope| {
            let handles: Vec<_> = rest
                .chunks(segment)
                .map(|piece| {
                    let handle = std::thread::Builder::new()
                        .spawn_scoped(scope, move || crc64_update_serial(0, piece))
                        .ok();
                    (piece, handle)
                })
                .collect();
            let mut crc = crc64_update_serial(start, first);
            for (piece, handle) in handles {
                let piece_crc = match handle.map(|h| h.join()) {
                    Some(Ok(value)) => value,
                    _ => crc64_update_serial(0, piece),
                };
                crc = crc64_ecma_combine(crc, piece_crc, piece.len() as u64);
            }
            crc
        });
    }

    pub const fn finish(self) -> u64 {
        self.state
    }
}

/// Slices at least this long go through the multi-thread path of
/// [`Crc64Ecma::update`]; shorter slices stay on the calling thread.
pub const CRC64_PARALLEL_MIN_BYTES: usize = 4 << 20;

/// Smallest segment the automatic path of [`Crc64Ecma::update`] gives to
/// one thread.
const CRC64_PARALLEL_SEGMENT_MIN: usize = 1 << 20;

/// Upper bound on threads for the automatic path.
const CRC64_PARALLEL_MAX_THREADS: usize = 8;

fn crc64_auto_threads() -> usize {
    static THREADS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *THREADS.get_or_init(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .clamp(1, CRC64_PARALLEL_MAX_THREADS)
    })
}

/// The one-thread table update: the register after `bytes` starting from
/// `state`.
#[inline]
fn crc64_update_serial(state: u64, bytes: &[u8]) -> u64 {
    let t = &CRC64_TABLES;
    let mut crc = state;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let mut word = [0u8; 8];
        word.copy_from_slice(chunk);
        let x = crc ^ u64::from_be_bytes(word);
        crc = t[7][(x >> 56) as usize]
            ^ t[6][((x >> 48) as u8) as usize]
            ^ t[5][((x >> 40) as u8) as usize]
            ^ t[4][((x >> 32) as u8) as usize]
            ^ t[3][((x >> 24) as u8) as usize]
            ^ t[2][((x >> 16) as u8) as usize]
            ^ t[1][((x >> 8) as u8) as usize]
            ^ t[0][(x as u8) as usize];
    }
    for &byte in chunks.remainder() {
        crc = (crc << 8) ^ t[0][(((crc >> 56) as u8) ^ byte) as usize];
    }
    crc
}

/// `a * b mod P` over GF(2), with bit `i` holding the coefficient of `x^i`
/// and `P = x^64 + CRC64_ECMA_POLY`.
const fn crc64_mulmod(a: u64, b: u64) -> u64 {
    let mut product = 0u64;
    let mut bit = 64;
    while bit > 0 {
        bit -= 1;
        product = if (product & 0x8000_0000_0000_0000) != 0 {
            (product << 1) ^ CRC64_ECMA_POLY
        } else {
            product << 1
        };
        if (b >> bit) & 1 != 0 {
            product ^= a;
        }
    }
    product
}

/// `x^(8 * bytes) mod P`: the factor that moves a register past `bytes`
/// zero bytes.
const fn crc64_zero_bytes_factor(mut bytes: u64) -> u64 {
    let mut factor = 1u64;
    // x^8, then x^16, x^32, ... by squaring.
    let mut square = 1u64 << 8;
    while bytes != 0 {
        if bytes & 1 != 0 {
            factor = crc64_mulmod(factor, square);
        }
        square = crc64_mulmod(square, square);
        bytes >>= 1;
    }
    factor
}

/// CRC-64/ECMA of `A` followed by `B`, from `crc_a = crc64_ecma(A)`,
/// `crc_b = crc64_ecma(B)` and `len_b = B.len()`. Exact for every input:
/// with init 0 and xorout 0 the register is linear, so
/// `crc(A || B) = crc(A) * x^(8 * len_b) mod P  xor  crc(B)`.
pub const fn crc64_ecma_combine(crc_a: u64, crc_b: u64, len_b: u64) -> u64 {
    crc64_mulmod(crc_a, crc64_zero_bytes_factor(len_b)) ^ crc_b
}

/// Legacy v1 checksum. Kept only so v0.2 can explicitly read v0.1 stores.
/// New v2 stores use CRC64-ECMA for both header and payload integrity.
#[inline]
pub fn checksum64(bytes: &[u8]) -> u64 {
    let mut checksum = Checksum64::new();
    checksum.update(bytes);
    checksum.finish()
}

#[derive(Clone, Copy, Debug)]
pub struct Checksum64 {
    state: u64,
    len: u64,
}

impl Default for Checksum64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Checksum64 {
    pub const fn new() -> Self {
        Self {
            state: 0xcbf2_9ce4_8422_2325,
            len: 0,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(0x0000_0100_0000_01B3);
        }
        self.len = self.len.wrapping_add(bytes.len() as u64);
    }

    pub fn finish(self) -> u64 {
        avalanche64(self.state ^ self.len)
    }
}

#[inline]
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[inline]
fn avalanche64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^ (x >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_exact() {
        assert_eq!(ORB_BITS, 1024);
        assert_eq!(ORB_WORDS, 16);
        assert_eq!(ORB_BYTES, 128);
        assert_eq!(GEL_FORMAT_VERSION, 2);
    }

    #[test]
    fn crc64_ecma_has_canonical_check_value() {
        assert_eq!(crc64_ecma(b"123456789"), 0x6C40_DF5F_0B49_7347);
    }

    #[test]
    fn crc64_detects_single_bit_change() {
        let a = [0u8; 128];
        let mut b = a;
        b[63] ^= 0x20;
        assert_ne!(crc64_ecma(&a), crc64_ecma(&b));
    }

    #[test]
    fn incremental_crc_matches_one_shot() {
        let bytes = (0..=255).collect::<Vec<u8>>();
        let mut incremental = Crc64Ecma::new();
        incremental.update(&bytes[..17]);
        incremental.update(&bytes[17..129]);
        incremental.update(&bytes[129..]);
        assert_eq!(incremental.finish(), crc64_ecma(&bytes));
    }

    /// The original bit-serial update, kept as the reference for the
    /// table-driven one. `state` is the running register.
    fn crc64_bitwise(state: u64, bytes: &[u8]) -> u64 {
        let mut crc = state;
        for &byte in bytes {
            crc ^= (byte as u64) << 56;
            for _ in 0..8 {
                crc = if (crc & 0x8000_0000_0000_0000) != 0 {
                    (crc << 1) ^ CRC64_ECMA_POLY
                } else {
                    crc << 1
                };
            }
        }
        crc
    }

    /// A second, separately written bit-serial form (top bit tested by
    /// shift, literal polynomial), so the reference does not rest on one text.
    fn crc64_independent(bytes: &[u8]) -> u64 {
        let mut crc = 0u64;
        for &byte in bytes {
            crc ^= u64::from(byte) << 56;
            for _ in 0..8 {
                crc = if crc >> 63 == 1 {
                    (crc << 1) ^ 0x42f0_e1eb_a9ea_3693
                } else {
                    crc << 1
                };
            }
        }
        crc
    }

    fn seeded_bytes(seed: u64, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        let mut i = 0u64;
        while out.len() < len {
            let word = splitmix64(seed ^ i.wrapping_mul(0xA076_1D64_78BD_642F));
            for byte in word.to_le_bytes() {
                if out.len() < len {
                    out.push(byte);
                }
            }
            i += 1;
        }
        out
    }

    #[test]
    fn crc_tables_match_their_definition() {
        for (k, table) in CRC64_TABLES.iter().enumerate() {
            for (b, &entry) in table.iter().enumerate() {
                // Byte `b` followed by `k` zero bytes.
                let mut bytes = vec![0u8; k + 1];
                bytes[0] = b as u8;
                assert_eq!(entry, crc64_bitwise(0, &bytes), "T{k}[{b}]");
            }
        }
    }

    #[test]
    fn table_crc_matches_bitwise_reference_on_seeded_inputs() {
        // Edge cases: empty, all-zero and all-one bytes, every length that
        // crosses the 8-byte step, and a nonzero starting register.
        for len in 0..=64usize {
            for fill in [0x00u8, 0xFF, 0x80, 0x01] {
                let bytes = vec![fill; len];
                assert_eq!(
                    crc64_ecma(&bytes),
                    crc64_bitwise(0, &bytes),
                    "fill {fill} len {len}"
                );
            }
        }
        for seed in 0..64u64 {
            let buffer = seeded_bytes(seed, 2048);
            for len in 0..=2048usize {
                let bytes = &buffer[..len];
                let expected = crc64_bitwise(0, bytes);
                assert_eq!(crc64_ecma(bytes), expected, "seed {seed} len {len}");
                if len % 97 == 0 || len < 32 {
                    assert_eq!(crc64_independent(bytes), expected, "seed {seed} len {len}");
                }
            }
            // Streaming with random 1..=15 byte pieces, so pieces start at
            // every phase of the 8-byte step.
            let mut rng = splitmix64(seed ^ 0x5EED);
            let mut at = 0usize;
            let mut streamed = Crc64Ecma::new();
            while at < buffer.len() {
                rng = splitmix64(rng);
                let piece = 1 + (rng % 15) as usize;
                let end = (at + piece).min(buffer.len());
                streamed.update(&buffer[at..end]);
                assert_eq!(
                    streamed.finish(),
                    crc64_bitwise(0, &buffer[..end]),
                    "seed {seed} prefix {end}"
                );
                at = end;
            }
            // A nonzero register entering update.
            let start = splitmix64(seed ^ 0xC0FFEE);
            for len in [0usize, 1, 7, 8, 9, 15, 16, 17, 127, 128, 129, 2047] {
                let mut crc = Crc64Ecma { state: start };
                crc.update(&buffer[..len]);
                assert_eq!(
                    crc.finish(),
                    crc64_bitwise(start, &buffer[..len]),
                    "seed {seed} start len {len}"
                );
            }
        }
    }

    #[test]
    fn table_crc_matches_bitwise_reference_on_a_large_buffer() {
        let bytes = seeded_bytes(0xB16, (1 << 20) + 13);
        assert_eq!(crc64_ecma(&bytes), crc64_bitwise(0, &bytes));
        let mut records = Crc64Ecma::new();
        for chunk in bytes.chunks(ORB_BYTES) {
            records.update(chunk);
        }
        assert_eq!(records.finish(), crc64_independent(&bytes));
    }

    #[test]
    fn crc_combine_matches_bitwise_reference() {
        assert_eq!(crc64_zero_bytes_factor(0), 1);
        assert_eq!(
            crc64_mulmod(0x1234_5678_9ABC_DEF0, 1),
            0x1234_5678_9ABC_DEF0
        );
        for seed in 0..32u64 {
            let buffer = seeded_bytes(seed ^ 0xC0B1, 700);
            let mut rng = splitmix64(seed);
            for _ in 0..64 {
                rng = splitmix64(rng);
                let a = (rng % 350) as usize;
                let b = ((rng >> 20) % 350) as usize;
                let (left, right) = buffer[..a + b].split_at(a);
                assert_eq!(
                    crc64_ecma_combine(crc64_ecma(left), crc64_ecma(right), b as u64),
                    crc64_bitwise(0, &buffer[..a + b]),
                    "seed {seed} a {a} b {b}"
                );
            }
        }
        // Moving a register past n zero bytes is the same as feeding them.
        for n in [0usize, 1, 7, 8, 9, 64, 1000, 4097] {
            let state = splitmix64(n as u64);
            assert_eq!(
                crc64_mulmod(state, crc64_zero_bytes_factor(n as u64)),
                crc64_bitwise(state, &vec![0u8; n]),
                "zeros {n}"
            );
        }
    }

    #[test]
    fn threaded_crc_matches_bitwise_reference_for_every_thread_count() {
        let buffer = seeded_bytes(0x7EAD, 5000);
        let start = splitmix64(0x5747);
        for len in (0..=200usize).chain([999, 1000, 1001, 4093, 4096, 4099, 5000]) {
            let bytes = &buffer[..len];
            for state in [0, start] {
                let expected = crc64_bitwise(state, bytes);
                for threads in (0..=16usize).chain([len, len + 1, usize::MAX]) {
                    let mut crc = Crc64Ecma { state };
                    crc.update_with_threads(bytes, threads);
                    assert_eq!(crc.finish(), expected, "len {len} threads {threads}");
                }
            }
        }
    }

    #[test]
    fn automatic_threaded_path_matches_the_serial_path() {
        let bytes = seeded_bytes(0xA070, CRC64_PARALLEL_MIN_BYTES + (1 << 20) + 13);
        for len in [
            CRC64_PARALLEL_MIN_BYTES - 1,
            CRC64_PARALLEL_MIN_BYTES,
            CRC64_PARALLEL_MIN_BYTES + 1,
            bytes.len(),
        ] {
            let expected = crc64_update_serial(0, &bytes[..len]);
            assert_eq!(crc64_ecma(&bytes[..len]), expected, "len {len}");
            if len == bytes.len() {
                assert_eq!(crc64_bitwise(0, &bytes[..len]), expected);
            }
        }
        let start = splitmix64(0xFEED);
        let mut streamed = Crc64Ecma { state: start };
        streamed.update(&bytes[..17]);
        streamed.update(&bytes[17..]);
        assert_eq!(streamed.finish(), crc64_bitwise(start, &bytes));
    }

    #[test]
    fn legacy_checksum_contract_is_preserved() {
        let bytes = (0..=255).rev().collect::<Vec<u8>>();
        let mut reference = 0xcbf2_9ce4_8422_2325u64;
        for &byte in &bytes {
            reference ^= byte as u64;
            reference = reference.wrapping_mul(0x0000_0100_0000_01B3);
        }
        reference = avalanche64(reference ^ bytes.len() as u64);
        assert_eq!(checksum64(&bytes), reference);
    }
}
