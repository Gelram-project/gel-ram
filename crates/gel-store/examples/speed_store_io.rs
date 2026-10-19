#![forbid(unsafe_code)]

//! Timing harness for store write, streaming verification and verified load.
//!
//! Usage: `speed_store_io [RUNS] [DIR] [RECORDS]`
//!
//! - `RUNS` (default 9): repetitions of every case.
//! - `DIR` (default: the OS temporary directory): where the store file lives.
//! - `RECORDS` (default 2,097,152): records in the store. The default is the
//!   largest store the default open budget admits: 256 MiB of payload, a
//!   268,435,520-byte file.
//!
//! Every run writes the same deterministic store with `write_atomic`, checks
//! it with `verify_file` and loads it with `open_verified_with_limits`. Each
//! case prints `SPEED name=<case> run=<i> ns=<n>`. The last line,
//! `SPEED_CHECK sha256=<hex>`, is the SHA-256 of the file bytes, the
//! verification report, the loaded header and format, and the loaded records,
//! so two builds can be compared for identical output. Uses only public API.

use gel_core::Generation;
use gel_orb::Orb1024;
use gel_store::{verify_file, OpenLimits, RamStore, DEFAULT_MAX_RECORDS};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

const SEED: u64 = 0x5eed_2026_1019_0001;

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 SHA-256, fed in pieces.
struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    filled: usize,
    length: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }

    fn compress(state: &mut [u32; 8], block: &[u8]) {
        let mut w = [0u32; 64];
        for (word, bytes) in w.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
        for (&k, &wi) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (word, add) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *word = word.wrapping_add(add);
        }
    }

    fn update(&mut self, mut bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        if self.filled > 0 {
            let take = (64 - self.filled).min(bytes.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled < 64 {
                return;
            }
            Self::compress(&mut self.state, &self.block);
            self.filled = 0;
        }
        let mut blocks = bytes.chunks_exact(64);
        for block in &mut blocks {
            Self::compress(&mut self.state, block);
        }
        let rest = blocks.remainder();
        self.block[..rest.len()].copy_from_slice(rest);
        self.filled = rest.len();
    }

    fn hex(mut self) -> String {
        let bits = self.length.wrapping_mul(8);
        self.update(&[0x80]);
        while self.filled != 56 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        let mut out = String::with_capacity(64);
        for word in self.state {
            let _ = write!(out, "{word:08x}");
        }
        out
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let runs: usize = args
        .next()
        .map(|a| a.parse().expect("RUNS must be an unsigned integer"))
        .unwrap_or(9);
    let dir: PathBuf = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let records: usize = args
        .next()
        .map(|a| a.parse().expect("RECORDS must be an unsigned integer"))
        .unwrap_or(DEFAULT_MAX_RECORDS as usize);

    let mut probe = Sha256::new();
    probe.update(b"abc");
    assert_eq!(
        probe.hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    let mut state = SEED;
    let orbs: Vec<Orb1024> = (0..records)
        .map(|_| {
            let mut words = [0u64; 16];
            for word in &mut words {
                *word = next(&mut state);
            }
            Orb1024::from_words(words)
        })
        .collect();
    let store = RamStore::from_orbs(orbs, Generation(7));
    let limits = OpenLimits {
        max_records: u64::MAX,
        max_file_bytes: u64::MAX,
    };
    let path = dir.join(format!("speed-store-io-{}.gel", std::process::id()));
    let mib = records * 128 / (1024 * 1024);
    println!(
        "SPEED_INPUT records={records} payload_bytes={} runs={runs}",
        records * 128
    );

    let mut report = String::new();
    let mut last: Option<RamStore> = None;
    for run in 0..runs {
        drop(last.take());
        let start = Instant::now();
        store.write_atomic(&path).expect("write_atomic");
        let ns = start.elapsed().as_nanos();
        println!("SPEED name=write_{mib}mib run={run} ns={ns}");

        let start = Instant::now();
        let verification = verify_file(&path, limits).expect("verify_file");
        let ns = start.elapsed().as_nanos();
        println!("SPEED name=verify_{mib}mib run={run} ns={ns}");

        let start = Instant::now();
        let loaded = RamStore::open_verified_with_limits(&path, limits).expect("open_verified");
        let ns = start.elapsed().as_nanos();
        println!("SPEED name=open_{mib}mib run={run} ns={ns}");

        assert_eq!(verification.header, store.header());
        assert_eq!(loaded.header(), store.header());
        assert!(loaded.orbs() == store.orbs());
        report = format!(
            "{verification:?}|{:?}|{:?}",
            loaded.header(),
            loaded.source_format()
        );
        last = Some(loaded);
    }

    let mut hash = Sha256::new();
    if let Some(loaded) = last {
        let bytes = fs::read(&path).expect("read store file");
        hash.update(&bytes);
        drop(bytes);
        hash.update(report.as_bytes());
        for orb in loaded.orbs() {
            hash.update(&orb.to_le_bytes());
        }
        let _ = fs::remove_file(&path);
    }
    println!("SPEED_CHECK sha256={}", hash.hex());
}
