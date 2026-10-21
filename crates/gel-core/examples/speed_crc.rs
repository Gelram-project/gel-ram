//! Timing harness for CRC-64/ECMA. Prints per-run wall time in
//! nanoseconds and a SHA-256 over every CRC value produced, so two builds
//! can be compared for both speed and output.
//!
//! Usage: cargo run --release -p gel-core --example speed_crc -- [RUNS]
//! (RUNS 1..=999, default 9). Input: 64 MiB of deterministic splitmix64 bytes.
//! Output grammar: `docs/SPEED.md`.

use gel_core::{crc64_ecma, splitmix64, Crc64Ecma, ORB_BYTES};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const INPUT_BYTES: usize = 64 << 20;
const ODD_PIECE: usize = 13;

type CrcCase = fn(&[u8]) -> u64;

fn input() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(INPUT_BYTES);
    let mut i = 0u64;
    while bytes.len() < INPUT_BYTES {
        bytes.extend_from_slice(&splitmix64(0x4745_4C00_4352_4336 ^ i).to_le_bytes());
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

fn main() {
    let args = speed::args("speed_crc", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let bytes = input();
    speed::setup("speed_crc", runs, &[("bytes", INPUT_BYTES.to_string())]);
    let cases: [(&str, CrcCase); 3] = [
        ("oneshot_64mib", crc64_ecma),
        ("records128_64mib", |b| streamed(b, ORB_BYTES)),
        ("pieces13_64mib", |b| streamed(b, ODD_PIECE)),
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
