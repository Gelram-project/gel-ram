#![forbid(unsafe_code)]

//! Repeatable timing of `top_k` on deterministic 524,288-ORB banks (64 MiB).
//! It uses only API that predates `top_k_threads`, so the same file builds on
//! older trees and the printed output digests can be compared between them.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_topk -- [RUNS]`
//! (default 9). Prints `SPEED name=<case> run=<i> ns=<n>` per run and, at the
//! end, `SPEED_CHECK sha256=<hex>` over every result of every run.

use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_orb::Orb1024;
use gel_reader::top_k;
use std::hint::black_box;
use std::time::Instant;

const ORBS: usize = 524_288;

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

/// An ORB whose score against the all-zero query is exactly `score`.
fn orb_with_score(score: usize, rotate: usize) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for j in 0..ORB_BITS - score {
        let bit = (rotate + j) % ORB_BITS;
        words[bit / 64] |= 1 << (bit % 64);
    }
    Orb1024::from_words(words)
}

/// Deterministic bank and query for one case.
fn case(name: &str) -> (Vec<Orb1024>, Orb1024) {
    let mut seed = 0x2026_1019_544f_504b;
    let zero = Orb1024::from_words([0; ORB_WORDS]);
    match name {
        "uniform" => {
            let bank = (0..ORBS).map(|_| random_orb(&mut seed)).collect();
            (bank, random_orb(&mut seed))
        }
        "clustered" => {
            let centers: Vec<_> = (0..16).map(|_| random_orb(&mut seed)).collect();
            let bank = (0..ORBS)
                .map(|i| {
                    let mut orb = centers[i % centers.len()];
                    for _ in 0..16 {
                        let bit = next(&mut seed) as usize % ORB_BITS;
                        orb.words_mut()[bit / 64] ^= 1 << (bit % 64);
                    }
                    orb
                })
                .collect();
            let mut query = centers[3];
            query.words_mut()[0] ^= 0xff;
            (bank, query)
        }
        "ties" => (vec![tie_orb(); ORBS], zero),
        // 255 perfect scores, then a rising saw: candidates keep entering the
        // full set at its last position.
        "tail_entry" => {
            let bank = (0..ORBS)
                .map(|i| {
                    if i < 255 {
                        orb_with_score(ORB_BITS, i)
                    } else {
                        orb_with_score((i - 255) % ORB_BITS, i)
                    }
                })
                .collect();
            (bank, zero)
        }
        // Scores rise in equal steps from 0 to 1024; each step admits a full
        // set of equal-score entries at every position of the kept list.
        "staircase" => {
            let bank = (0..ORBS)
                .map(|i| orb_with_score(i * (ORB_BITS + 1) / ORBS, i))
                .collect();
            (bank, zero)
        }
        _ => unreachable!("unknown case {name}"),
    }
}

/// One fixed ORB repeated across the whole bank: every score ties.
fn tie_orb() -> Orb1024 {
    let mut seed = 0x7469_6573;
    random_orb(&mut seed)
}

fn main() {
    let runs = std::env::args()
        .nth(1)
        .map(|v| v.parse::<usize>().expect("RUNS must be a positive integer"))
        .unwrap_or(9);
    assert!(runs >= 1, "RUNS must be a positive integer");
    assert_eq!(
        hex(&sha256_of(&[b"abc"])),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "SHA-256 self-check"
    );
    let mut digest = Sha256::new();
    for (name, k) in [
        ("uniform", 256),
        ("uniform", 8),
        ("clustered", 256),
        ("ties", 256),
        ("tail_entry", 256),
        ("staircase", 256),
    ] {
        let (bank, query) = case(name);
        let label = format!("{name}_k{k}");
        for run in 1..=runs {
            let start = Instant::now();
            let out = top_k(black_box(&bank), black_box(&query), black_box(k));
            let ns = start.elapsed().as_nanos();
            black_box(&out);
            println!("SPEED name={label} run={run} ns={ns}");
            digest.update(label.as_bytes());
            digest.update(&(out.len() as u64).to_le_bytes());
            for (index, score) in out {
                digest.update(&(index as u64).to_le_bytes());
                digest.update(&score.to_le_bytes());
            }
        }
    }
    println!("SPEED_CHECK sha256={}", hex(&digest.finish()));
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        write!(out, "{b:02x}").expect("writing to a String cannot fail");
        out
    })
}

fn sha256_of(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part);
    }
    h.finish()
}

/// Minimal FIPS 180-4 SHA-256, self-checked in `main`.
struct Sha256 {
    state: [u32; 8],
    block: Vec<u8>,
    length: u64,
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

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            block: Vec::with_capacity(64),
            length: 0,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        for &byte in bytes {
            self.block.push(byte);
            if self.block.len() == 64 {
                self.compress();
            }
        }
    }

    fn compress(&mut self) {
        let mut w = [0u32; 64];
        for (word, b) in w.iter_mut().zip(self.block.chunks_exact(4)) {
            *word = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
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
        for (word, add) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *word = word.wrapping_add(add);
        }
        self.block.clear();
    }

    fn finish(mut self) -> [u8; 32] {
        let bits = self.length.wrapping_mul(8);
        self.update(&[0x80]);
        while self.block.len() != 56 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        let mut out = [0u8; 32];
        for (bytes, word) in out.chunks_exact_mut(4).zip(self.state) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}
