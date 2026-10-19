//! Timing harness for the Q8 scoring path: `Reader::read` and single-worker
//! `Reader::scan_into` over a deterministic synthetic bank of 8192 records
//! (9,437,184 bytes of records, the largest Q8DEMO01 fixture size).
//!
//! Usage: `cargo run --release -p gel-phase-quad --example speed_q8_read -- [R]`
//! with R repetitions per case (1..=999, default 9). Prints one
//! `SPEED name=<case> run=<i> ns=<n>` line per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over the bits of every score produced, so two
//! builds can be compared for identical output. Uses only the public API.
#![forbid(unsafe_code)]
use gel_phase_quad::{grid::DIM, Policy, Reader, Record, SharedScore};
use std::hint::black_box;
use std::time::Instant;

const BANK: usize = 8192;
const QUERIES: usize = 8;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn record(&mut self, active_per_256: u64) -> Record {
        let phase = std::array::from_fn(|_| self.next() as u8);
        let active = std::array::from_fn(|_| self.next() % 256 < active_per_256);
        Record::new(phase, &active)
    }
    /// Exactly `count` active positions, chosen at random.
    fn sparse(&mut self, count: usize) -> Record {
        let phase = std::array::from_fn(|_| self.next() as u8);
        let mut active = [false; DIM];
        let mut set = 0;
        while set < count {
            let j = (self.next() % DIM as u64) as usize;
            if !active[j] {
                active[j] = true;
                set += 1;
            }
        }
        Record::new(phase, &active)
    }
}

/// Minimal SHA-256 (FIPS 180-4), streaming, for the output check only.
struct Sha256 {
    h: [u32; 8],
    block: [u8; 64],
    filled: usize,
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
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }
    fn compress(&mut self) {
        let mut w = [0u32; 64];
        for (i, word) in self.block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.h;
        for (&k, &wi) in K.iter().zip(&w) {
            let t1 = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(k)
                .wrapping_add(wi);
            let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, v) in self.h.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(v);
        }
    }
    fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.block[self.filled] = byte;
            self.filled += 1;
            if self.filled == 64 {
                self.compress();
                self.filled = 0;
            }
        }
        self.length = self.length.wrapping_add(bytes.len() as u64);
    }
    fn finish(mut self) -> String {
        let bits = self.length.wrapping_mul(8);
        self.update(&[0x80]);
        while self.filled != 56 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        let mut hex = String::with_capacity(64);
        for v in self.h {
            hex.push_str(&format!("{v:08x}"));
        }
        hex
    }
}

fn record_scores(hash: &mut Sha256, scores: &[SharedScore]) {
    for s in scores {
        hash.update(&s.value().to_bits().to_le_bytes());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rounds = match args.as_slice() {
        [] => 9,
        [r] => match r.parse::<usize>() {
            Ok(n) if (1..=999).contains(&n) => n,
            _ => {
                eprintln!("R must be an integer in 1..=999");
                std::process::exit(2);
            }
        },
        _ => {
            eprintln!("usage: speed_q8_read [R]");
            std::process::exit(2);
        }
    };
    for (text, expected) in [
        (
            &b"abc"[..],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            &b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"[..],
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        let mut check = Sha256::new();
        check.update(text);
        assert_eq!(check.finish(), expected, "SHA-256 self-check");
    }
    let reader = Reader::new(210_021);
    let mut rng = Rng(0x0051_ed27_0b0d_1c3a);
    // Bodies: about two thirds of positions active, as in the witness bank,
    // with every eighth body fully active and every sixteenth silent.
    let bank: Vec<Record> = (0..BANK)
        .map(|n| match n % 16 {
            0 => rng.record(0),
            8 => rng.record(256),
            _ => rng.record(171),
        })
        .collect();
    let dense: Vec<_> = (0..QUERIES)
        .map(|_| reader.prepare(rng.record(171)))
        .collect();
    let sparse: Vec<_> = (0..QUERIES)
        .map(|_| reader.prepare(rng.sparse(64)))
        .collect();
    println!(
        "SPEED_SETUP bank={BANK} record_bytes={} queries={QUERIES} rounds={rounds} workers=1",
        BANK * 1152
    );
    let cases = [
        ("scan_active_dense", &dense, Policy::BodyActivity, true),
        ("scan_archive_dense", &dense, Policy::Archive, true),
        ("scan_active_sparse64", &sparse, Policy::BodyActivity, true),
        ("read_active_dense", &dense, Policy::BodyActivity, false),
    ];
    let mut hash = Sha256::new();
    let mut out: Vec<SharedScore> = Vec::with_capacity(BANK);
    let mut produced = vec![SharedScore::default(); QUERIES * BANK];
    for (name, queries, policy, scan) in cases {
        for run in 0..rounds {
            let start = Instant::now();
            for (q, slot) in queries.iter().zip(produced.chunks_exact_mut(BANK)) {
                if scan {
                    let scored = reader
                        .scan_into(black_box(q), black_box(&bank), policy, 1, &mut out)
                        .expect("valid worker budget");
                    assert!(scored, "every query has an active position");
                    slot.copy_from_slice(&out);
                } else {
                    for (body, score) in black_box(&bank).iter().zip(slot.iter_mut()) {
                        *score = reader
                            .read(black_box(q), body, policy)
                            .expect("every query has an active position");
                    }
                }
            }
            black_box(&produced);
            let ns = start.elapsed().as_nanos();
            println!("SPEED name={name} run={run} ns={ns}");
            record_scores(&mut hash, &produced);
        }
    }
    println!("SPEED_CHECK sha256={}", hash.finish());
}
