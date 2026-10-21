#![allow(dead_code)]
//! Shared helpers for the timing examples, `crates/*/examples/speed_*.rs`.
//!
//! Included from an example with
//! `#[path = "../../../tools/speed_support.rs"] mod speed;`
//! so no crate gains a dependency. Pure `std`. The output check is a SHA-256
//! (FIPS 180-4) computed here, never by the code being timed.
//!
//! Output grammar (stdout, see `docs/SPEED.md`):
//! - `SPEED_SETUP example=<stem> runs=<R>[ <key>=<value>]*` once, first;
//! - `SPEED name=<case> run=<i> ns=<n>` per case and run, `i` from 0;
//! - `SPEED_CHECK sha256=<64 lowercase hex>` once, last.
//!
//! A failed check prints `SPEED_FAIL reason=<slug>` on stderr, no
//! `SPEED_CHECK` line, and exits 1. A bad argument exits 2.

use std::ops::RangeInclusive;

/// Default number of runs per case.
pub const RUNS_DEFAULT: usize = 9;
/// Admitted numbers of runs per case.
pub const RUNS_RANGE: RangeInclusive<usize> = 1..=999;
/// Default thread count of examples that take a `THREADS` argument.
pub const THREADS_DEFAULT: usize = 8;
/// Admitted thread counts.
pub const THREADS_RANGE: RangeInclusive<usize> = 1..=999;

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

const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Streaming SHA-256 (FIPS 180-4).
pub struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    filled: usize,
    length: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: H0,
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

    /// Appends `bytes` to the message.
    pub fn update(&mut self, mut bytes: &[u8]) {
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

    /// The 32-byte digest of everything appended so far.
    pub fn finish(mut self) -> [u8; 32] {
        let bits = self.length.wrapping_mul(8);
        let mut pad = [0u8; 72];
        pad[0] = 0x80;
        let zeros = (119 - self.filled) % 64;
        pad[1 + zeros..9 + zeros].copy_from_slice(&bits.to_be_bytes());
        self.update(&pad[..9 + zeros]);
        debug_assert_eq!(self.filled, 0);
        let mut out = [0u8; 32];
        for (bytes, word) in out.chunks_exact_mut(4).zip(self.state) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        out
    }

    /// The digest as 64 lowercase hex digits.
    pub fn finish_hex(self) -> String {
        hex(&self.finish())
    }
}

/// SHA-256 of one message.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finish()
}

/// Lowercase hex of `bytes`.
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 15)]));
    }
    out
}

/// Panics unless the digest reproduces the FIPS 180-4 and common known answers,
/// whole and fed in uneven pieces.
pub fn self_check() {
    let long = [b'a'; 200];
    for (text, expected) in [
        (
            &b""[..],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            &b"abc"[..],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            &b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"[..],
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            &long[..],
            "c2a908d98f5df987ade41b5fce213067efbcc21ef2240212a41e54b5e7c28ae5",
        ),
    ] {
        assert_eq!(hex(&sha256(text)), expected, "SHA-256 self-check");
        for piece in [1, 3, 63, 64, 65] {
            let mut hash = Sha256::new();
            for chunk in text.chunks(piece) {
                hash.update(chunk);
            }
            assert_eq!(hash.finish_hex(), expected, "SHA-256 self-check");
        }
    }
}

/// The command-line arguments after the program name. More than `max` of them
/// prints `usage: <stem> <usage>` on stderr and exits 2.
pub fn args(stem: &str, usage: &str, max: usize) -> Vec<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > max {
        eprintln!("usage: {stem} {usage}");
        std::process::exit(2);
    }
    args
}

/// Positional integer argument `index` (0 is `RUNS`), `default` when absent.
/// A value outside `range` prints `<name> must be an integer in <lo>..=<hi>`
/// on stderr and exits 2.
pub fn number(
    args: &[String],
    index: usize,
    name: &str,
    default: usize,
    range: RangeInclusive<usize>,
) -> usize {
    match args.get(index) {
        None => default,
        Some(text) => match text.parse::<usize>() {
            Ok(n) if range.contains(&n) => n,
            _ => {
                eprintln!(
                    "{name} must be an integer in {}..={}",
                    range.start(),
                    range.end()
                );
                std::process::exit(2);
            }
        },
    }
}

/// `RUNS`, the first positional argument: default 9, admitted 1..=999.
pub fn runs(args: &[String]) -> usize {
    number(args, 0, "RUNS", RUNS_DEFAULT, RUNS_RANGE)
}

/// `THREADS` at position `index`: default 8, admitted 1..=999.
pub fn threads(args: &[String], index: usize) -> usize {
    number(args, index, "THREADS", THREADS_DEFAULT, THREADS_RANGE)
}

fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Runs the digest self-check, then prints
/// `SPEED_SETUP example=<example> runs=<runs>[ <key>=<value>]*`.
/// Keys are `[a-z0-9_]+`; values contain no white space.
pub fn setup(example: &str, runs: usize, fields: &[(&str, String)]) {
    self_check();
    assert!(is_token(example), "example stem {example:?}");
    let mut line = format!("SPEED_SETUP example={example} runs={runs}");
    for (key, value) in fields {
        assert!(
            is_token(key) && *key != "example" && *key != "runs",
            "setup key {key:?}"
        );
        assert!(
            !value.is_empty() && !value.chars().any(char::is_whitespace),
            "setup value {value:?}"
        );
        line.push(' ');
        line.push_str(key);
        line.push('=');
        line.push_str(value);
    }
    println!("{line}");
}

/// Prints `SPEED name=<name> run=<run> ns=<ns>`.
pub fn line(name: &str, run: usize, ns: u128) {
    println!("SPEED name={name} run={run} ns={ns}");
}

/// Prints `SPEED_CHECK sha256=<hex>` of everything `hash` received.
pub fn check(hash: Sha256) {
    check_digest(&hash.finish());
}

/// Prints `SPEED_CHECK sha256=<hex>` of an already finished digest.
pub fn check_digest(digest: &[u8; 32]) {
    println!("SPEED_CHECK sha256={}", hex(digest));
}

/// Prints `SPEED_FAIL reason=<reason>` on stderr and exits 1.
pub fn fail(reason: &str) -> ! {
    eprintln!("SPEED_FAIL reason={reason}");
    std::process::exit(1);
}

/// The SplitMix64 sequence, the same as the local generators of the Q8 examples.
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}
