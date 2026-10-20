//! Timing harness that compares, in one binary, the one-thread CRC-64/ECMA
//! path (`update_with_threads(.., 1)`, case `crc_256mib_ref`) with 2, 4 and 8
//! threads (`crc_256mib_t2`, `_t4`, `_t8`) and with the automatic path of
//! `crc64_ecma` (`crc_256mib_auto`). Needs `Crc64Ecma::update_with_threads`,
//! so it builds only on trees that have it.
//!
//! Usage: cargo run --release -p gel-core --example speed_crc_threads_paired -- [RUNS]
//! (RUNS 1..=999, default 9). Input: 256 MiB of deterministic splitmix64 bytes.
//! Every CRC must equal the first one; a difference ends the run with
//! `SPEED_FAIL reason=crc_differs`. `SPEED_CHECK` is the SHA-256 of every CRC
//! of every run. Output grammar: `docs/SPEED.md`.

use gel_core::{crc64_ecma, splitmix64, Crc64Ecma};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const INPUT_BYTES: usize = 256 << 20;

/// A named way of computing the CRC of the whole input.
type Case = (&'static str, fn(&[u8]) -> u64);

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

fn threads(bytes: &[u8], count: usize) -> u64 {
    let mut crc = Crc64Ecma::new();
    crc.update_with_threads(bytes, count);
    crc.finish()
}

fn main() {
    let args = speed::args("speed_crc_threads_paired", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let bytes = input();
    speed::setup(
        "speed_crc_threads_paired",
        runs,
        &[
            ("bytes", INPUT_BYTES.to_string()),
            ("threads", "1,2,4,8,auto".to_string()),
        ],
    );
    let cases: [Case; 5] = [
        ("crc_256mib_ref", |b| threads(b, 1)),
        ("crc_256mib_t2", |b| threads(b, 2)),
        ("crc_256mib_t4", |b| threads(b, 4)),
        ("crc_256mib_t8", |b| threads(b, 8)),
        ("crc_256mib_auto", crc64_ecma),
    ];
    let mut first = None;
    let mut hash = speed::Sha256::new();
    for (name, case) in cases {
        for run in 0..runs {
            let start = Instant::now();
            let crc = black_box(case(black_box(&bytes)));
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            match first {
                None => first = Some(crc),
                Some(value) if value == crc => {}
                Some(_) => speed::fail("crc_differs"),
            }
            hash.update(&crc.to_be_bytes());
        }
    }
    speed::check(hash);
}
