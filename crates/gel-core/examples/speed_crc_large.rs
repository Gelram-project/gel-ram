//! Timing harness for CRC-64/ECMA on large in-memory buffers. Uses only
//! `crc64_ecma`, `Crc64Ecma` and `splitmix64`, so the same file builds on
//! trees before and after the multi-thread path. Prints per-run wall time in
//! nanoseconds and a SHA-256 over every CRC value produced.
//!
//! Usage: cargo run --release -p gel-core --example speed_crc_large -- [RUNS]
//! (RUNS 1..=999, default 9). Input: 256 MiB of deterministic splitmix64 bytes.
//! Cases: one call over 256 MiB; 4 MiB, 1 MiB and 128-byte pieces over
//! 256 MiB; one call over 4 MiB and over 3 MiB, 16 times at different offsets.
//! Output grammar: `docs/SPEED.md`.

use gel_core::{crc64_ecma, splitmix64, Crc64Ecma};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const MIB: usize = 1 << 20;
const INPUT_BYTES: usize = 256 * MIB;
const REPEATS: usize = 16;

type CrcCase = fn(&[u8]) -> u64;

fn input() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(INPUT_BYTES + 8);
    let mut i = 0u64;
    while bytes.len() < INPUT_BYTES {
        bytes.extend_from_slice(&splitmix64(0x4745_4C00_5357_3243 ^ i).to_le_bytes());
        i += 1;
    }
    bytes.truncate(INPUT_BYTES);
    bytes
}

fn streamed(bytes: &[u8], piece: usize) -> u64 {
    let mut crc = Crc64Ecma::new();
    for chunk in bytes.chunks(piece) {
        crc.update(chunk);
    }
    crc.finish()
}

/// `REPEATS` one-shot CRCs of `len` bytes at offsets `k * 13 MiB + k`,
/// folded into one value so every result is used.
fn repeated(bytes: &[u8], len: usize) -> u64 {
    let mut acc = 0u64;
    for k in 0..REPEATS {
        let at = k * 13 * MIB + k;
        acc = acc.rotate_left(7) ^ crc64_ecma(&bytes[at..at + len]);
    }
    acc
}

fn main() {
    let args = speed::args("speed_crc_large", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let bytes = input();
    speed::setup(
        "speed_crc_large",
        runs,
        &[("bytes", INPUT_BYTES.to_string())],
    );
    let cases: [(&str, CrcCase); 6] = [
        ("oneshot_256mib", crc64_ecma),
        ("blocks4mib_256mib", |b| streamed(b, 4 * MIB)),
        ("oneshot_4mib", |b| repeated(b, 4 * MIB)),
        ("blocks1mib_256mib", |b| streamed(b, MIB)),
        ("records128_256mib", |b| streamed(b, 128)),
        ("oneshot_3mib", |b| repeated(b, 3 * MIB)),
    ];
    let mut hash = speed::Sha256::new();
    for (name, case) in cases {
        for run in 0..runs {
            let start = Instant::now();
            let crc = black_box(case(black_box(&bytes)));
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            hash.update(name.as_bytes());
            hash.update(&crc.to_be_bytes());
        }
    }
    speed::check(hash);
}
