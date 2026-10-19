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
    pub fn update(&mut self, bytes: &[u8]) {
        let t = &CRC64_TABLES;
        let mut crc = self.state;
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
        self.state = crc;
    }

    pub const fn finish(self) -> u64 {
        self.state
    }
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
