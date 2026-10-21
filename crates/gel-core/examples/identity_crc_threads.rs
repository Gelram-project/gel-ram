//! Identity check for the multi-thread CRC-64/ECMA path.
//!
//! Compares `crc64_ecma`, `Crc64Ecma::update`, `Crc64Ecma::update_with_threads`
//! and `crc64_ecma_combine` with an exact copy of the one-thread table update
//! (own tables) and with a bit-serial reference, on seeded inputs and edge
//! cases. Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one
//! final `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` line. FAIL if any
//! value differs or if no case ran.
//!
//! Usage: cargo run --release -p gel-core --example identity_crc_threads
//! Exit status 0 on PASS, 1 on FAIL. Output grammar: `docs/SPEED.md`.

use gel_core::{
    crc64_ecma, crc64_ecma_combine, splitmix64, Crc64Ecma, CRC64_ECMA_POLY,
    CRC64_PARALLEL_MIN_BYTES,
};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
mod identity;
use identity::{Scale, Tally};

const MIB: usize = 1 << 20;

/// Reference tables, built here independently of the library.
fn reference_tables() -> Vec<[u64; 256]> {
    let mut tables = vec![[0u64; 256]; 8];
    for b in 0..256usize {
        tables[0][b] = bitwise(0, &[b as u8]);
    }
    for k in 1..8 {
        for b in 0..256 {
            let prev = tables[k - 1][b];
            tables[k][b] = (prev << 8) ^ tables[0][(prev >> 56) as usize];
        }
    }
    tables
}

/// Exact copy of the one-thread table update as it stood before the
/// multi-thread path.
fn reference_table(t: &[[u64; 256]], state: u64, bytes: &[u8]) -> u64 {
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

/// Bit-serial reference (literal polynomial, top bit tested by shift).
fn bitwise(state: u64, bytes: &[u8]) -> u64 {
    let mut crc = state;
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
    let mut out = Vec::with_capacity(len + 8);
    let mut i = 0u64;
    while out.len() < len {
        out.extend_from_slice(
            &splitmix64(seed ^ i.wrapping_mul(0xA076_1D64_78BD_642F)).to_le_bytes(),
        );
        i += 1;
    }
    out.truncate(len);
    out
}

/// Feeds `prefix` (one thread, it is short) and then `bytes` through
/// `update_with_threads`, so the threaded path also starts from a nonzero
/// register when `prefix` is not empty.
fn with_threads(prefix: &[u8], bytes: &[u8], threads: usize) -> u64 {
    let mut crc = Crc64Ecma::new();
    crc.update(prefix);
    crc.update_with_threads(bytes, threads);
    crc.finish()
}

/// Fixed prefix that puts a nonzero value in the register.
const PREFIX: &[u8] = b"GEL CRC start register";

/// Runs every case; `Scale::Full` adds the buffers of 4 MiB and up.
pub fn run(scale: Scale) -> Vec<Tally> {
    let large = scale == Scale::Full;
    let t = reference_tables();
    let mut tallies = Vec::new();

    // Library constant and reference tables agree with the bit-serial form.
    let mut pins = Tally::new("pins");
    pins.check_eq(crc64_ecma(b"123456789"), 0x6C40_DF5F_0B49_7347);
    pins.check_eq(crc64_ecma(b""), 0);
    pins.check_eq(CRC64_ECMA_POLY, 0x42F0_E1EB_A9EA_3693);
    pins.check_eq(reference_table(&t, 0, b"123456789"), 0x6C40_DF5F_0B49_7347);
    tallies.push(pins);

    let mut combine = Tally::new("combine");
    for seed in 0..64u64 {
        let buffer = seeded_bytes(seed ^ 0xC0B1, 600);
        let mut rng = splitmix64(seed);
        for _ in 0..64 {
            rng = splitmix64(rng);
            let a = (rng % 301) as usize;
            let b = ((rng >> 20) % 301) as usize;
            let (left, right) = buffer[..a + b].split_at(a);
            let expected = reference_table(&t, 0, &buffer[..a + b]);
            combine.check_eq(
                crc64_ecma_combine(crc64_ecma(left), crc64_ecma(right), b as u64),
                expected,
            );
            combine.check_eq(expected, bitwise(0, &buffer[..a + b]));
        }
        // Empty right part leaves the left CRC; empty left part gives the right.
        let crc = crc64_ecma(&buffer[..100]);
        combine.check_eq(crc64_ecma_combine(crc, 0, 0), crc);
        combine.check_eq(crc64_ecma_combine(0, crc, 100), crc);
    }
    tallies.push(combine);

    let start = reference_table(&t, 0, PREFIX);
    let mut small = Tally::new("threads_small");
    let buffer = seeded_bytes(0x7EAD, 70_000);
    for len in 0..=300usize {
        let bytes = &buffer[..len];
        for (prefix, state) in [(&b""[..], 0), (PREFIX, start)] {
            let expected = reference_table(&t, state, bytes);
            for threads in 1..=16usize {
                small.check_eq(with_threads(prefix, bytes, threads), expected);
            }
            if len % 37 == 0 {
                small.check_eq(expected, bitwise(state, bytes));
            }
        }
    }
    tallies.push(small);

    let mut boundaries = Tally::new("threads_boundaries");
    for threads in 1..=16usize {
        for base in [1000usize, 4096, 9999, 65_536, 69_999] {
            let segment = base.div_ceil(threads);
            for k in [1usize, 2, threads.saturating_sub(1).max(1)] {
                for delta in [-1isize, 0, 1] {
                    let len = (segment * k) as isize + delta;
                    if len < 0 || len as usize > buffer.len() {
                        continue;
                    }
                    let bytes = &buffer[..len as usize];
                    let expected = reference_table(&t, 0, bytes);
                    boundaries.check_eq(with_threads(b"", bytes, threads), expected);
                    boundaries.check_eq(
                        with_threads(PREFIX, bytes, threads),
                        reference_table(&t, start, bytes),
                    );
                }
            }
        }
    }
    boundaries.check_eq(reference_table(&t, 0, &buffer), bitwise(0, &buffer));
    tallies.push(boundaries);

    let mut extreme = Tally::new("threads_extreme");
    for len in [0usize, 1, 2, 7, 8, 9, 63, 64, 65, 255] {
        let bytes = &buffer[..len];
        let expected = reference_table(&t, 0, bytes);
        for threads in [0usize, 1, len, len + 1, 2 * len + 3, 1000, usize::MAX] {
            extreme.check_eq(with_threads(b"", bytes, threads), expected);
            extreme.check_eq(
                with_threads(PREFIX, bytes, threads),
                reference_table(&t, start, bytes),
            );
        }
    }
    tallies.push(extreme);

    if large {
        let big = seeded_bytes(0xB16_B16, 64 * MIB + 5);
        let mut auto = Tally::new("auto_threshold");
        for len in [
            CRC64_PARALLEL_MIN_BYTES - 1,
            CRC64_PARALLEL_MIN_BYTES,
            CRC64_PARALLEL_MIN_BYTES + 1,
            CRC64_PARALLEL_MIN_BYTES + 7,
            5 * MIB + 3,
            8 * MIB,
            16 * MIB + 13,
            64 * MIB + 5,
        ] {
            let bytes = &big[..len];
            let expected = reference_table(&t, 0, bytes);
            auto.check_eq(crc64_ecma(bytes), expected);
            for threads in [2usize, 3, 7, 8, 16, 24] {
                auto.check_eq(with_threads(b"", bytes, threads), expected);
            }
        }
        auto.check_eq(
            reference_table(&t, 0, &big[..16 * MIB + 13]),
            bitwise(0, &big[..16 * MIB + 13]),
        );
        tallies.push(auto);

        // A stream of pieces above and below the threshold, from a nonzero
        // register reached by feeding a prefix.
        let mut stream = Tally::new("stream_mixed");
        for seed in 0..4u64 {
            let rng = splitmix64(seed ^ 0x57EA);
            let pieces = [
                17usize,
                3 * MIB,
                4 * MIB,
                4 * MIB + 1,
                9,
                9 * MIB + 5,
                MIB,
                (rng % 4096) as usize,
                6 * MIB + 7,
            ];
            let mut crc = Crc64Ecma::new();
            let mut at = 0usize;
            for piece in pieces {
                let end = (at + piece).min(big.len());
                crc.update(&big[at..end]);
                at = end;
                stream.check_eq(crc.finish(), reference_table(&t, 0, &big[..end]));
            }
        }
        tallies.push(stream);

        let mut fills = Tally::new("fills");
        for fill in [0x00u8, 0xFF, 0x80, 0x01] {
            let bytes = vec![fill; 8 * MIB + 3];
            let expected = reference_table(&t, 0, &bytes);
            fills.check_eq(crc64_ecma(&bytes), expected);
            fills.check_eq(with_threads(b"", &bytes, 5), expected);
            let mut from_prefix = Crc64Ecma::new();
            from_prefix.update(PREFIX);
            from_prefix.update(&bytes);
            fills.check_eq(from_prefix.finish(), reference_table(&t, start, &bytes));
        }
        tallies.push(fills);
    }

    tallies
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}

#[cfg(test)]
mod tests {
    use super::{identity, run, Scale};

    #[test]
    fn identity_small_cases_pass() {
        let tallies = run(Scale::Small);
        for t in &tallies {
            assert_eq!(t.cases, t.equal, "{}", t.name);
        }
        assert!(identity::verdict(&tallies));
    }
}
