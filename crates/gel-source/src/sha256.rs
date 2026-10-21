//! SHA-256 as specified in FIPS 180-4, written for this crate in safe Rust.
//!
//! It produces the standard SHA-256 digest, so every pin made earlier with
//! another implementation or with `sha256sum` stays valid. The tests check the
//! FIPS 180-2 example messages and compare it with an independent
//! implementation for every length up to 1,024 bytes and every way of feeding
//! a short message in pieces. Without `unsafe` it cannot use the processor's
//! SHA instructions, so it is slower than a hardware-accelerated one.

/// The first 32 bits of the fractional parts of the cube roots of the first 64 primes.
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

/// The first 32 bits of the fractional parts of the square roots of the first 8 primes.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Message word `j` of the first 16 rounds, as read from the block.
#[inline(always)]
fn load(w: &mut [u32; 16], j: usize) -> u32 {
    w[j & 15]
}

/// Message word of round `i` for `i` in 16..64, where `j = i % 16`. The
/// 16-word window holds rounds `i - 16..i`, so the new word replaces the one
/// it no longer needs: `w[i] = w[i-16] + s0(w[i-15]) + w[i-7] + s1(w[i-2])`.
#[inline(always)]
fn expand(w: &mut [u32; 16], j: usize) -> u32 {
    let x = w[(j + 1) & 15];
    let y = w[(j + 14) & 15];
    let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
    let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
    let word = w[j & 15]
        .wrapping_add(s0)
        .wrapping_add(w[(j + 9) & 15])
        .wrapping_add(s1);
    w[j & 15] = word;
    word
}

/// Round `$i`. Instead of shifting the eight working words, the caller renames
/// them: after this round `$h` holds the new `a` and `$d` the new `e`.
macro_rules! round {
    ($word:ident, $w:ident, $i:expr, $a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident, $g:ident, $h:ident) => {{
        let i: usize = $i;
        let t1 = $h
            .wrapping_add($e.rotate_right(6) ^ $e.rotate_right(11) ^ $e.rotate_right(25))
            .wrapping_add(($e & $f) ^ (!$e & $g))
            .wrapping_add(K[i])
            .wrapping_add($word(&mut $w, i & 15));
        let t2 = ($a.rotate_right(2) ^ $a.rotate_right(13) ^ $a.rotate_right(22))
            .wrapping_add(($a & $b) ^ ($a & $c) ^ ($b & $c));
        $d = $d.wrapping_add(t1);
        $h = t1.wrapping_add(t2);
    }};
}

/// Sixteen rounds from `$base`; after two full turns of the names every
/// working word is back under its own name.
macro_rules! sixteen_rounds {
    ($base:expr, $word:ident, $w:ident, $a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident, $g:ident, $h:ident) => {
        round!($word, $w, $base, $a, $b, $c, $d, $e, $f, $g, $h);
        round!($word, $w, $base + 1, $h, $a, $b, $c, $d, $e, $f, $g);
        round!($word, $w, $base + 2, $g, $h, $a, $b, $c, $d, $e, $f);
        round!($word, $w, $base + 3, $f, $g, $h, $a, $b, $c, $d, $e);
        round!($word, $w, $base + 4, $e, $f, $g, $h, $a, $b, $c, $d);
        round!($word, $w, $base + 5, $d, $e, $f, $g, $h, $a, $b, $c);
        round!($word, $w, $base + 6, $c, $d, $e, $f, $g, $h, $a, $b);
        round!($word, $w, $base + 7, $b, $c, $d, $e, $f, $g, $h, $a);
        round!($word, $w, $base + 8, $a, $b, $c, $d, $e, $f, $g, $h);
        round!($word, $w, $base + 9, $h, $a, $b, $c, $d, $e, $f, $g);
        round!($word, $w, $base + 10, $g, $h, $a, $b, $c, $d, $e, $f);
        round!($word, $w, $base + 11, $f, $g, $h, $a, $b, $c, $d, $e);
        round!($word, $w, $base + 12, $e, $f, $g, $h, $a, $b, $c, $d);
        round!($word, $w, $base + 13, $d, $e, $f, $g, $h, $a, $b, $c);
        round!($word, $w, $base + 14, $c, $d, $e, $f, $g, $h, $a, $b);
        round!($word, $w, $base + 15, $b, $c, $d, $e, $f, $g, $h, $a);
    };
}

/// One 64-byte block into the eight state words.
fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 16];
    for (word, bytes) in w.iter_mut().zip(block.chunks_exact(4)) {
        *word = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    sixteen_rounds!(0, load, w, a, b, c, d, e, f, g, h);
    sixteen_rounds!(16, expand, w, a, b, c, d, e, f, g, h);
    sixteen_rounds!(32, expand, w, a, b, c, d, e, f, g, h);
    sixteen_rounds!(48, expand, w, a, b, c, d, e, f, g, h);
    for (word, add) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *word = word.wrapping_add(add);
    }
}

/// A SHA-256 computation fed in pieces; the digest equals that of the pieces joined.
pub(crate) struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    filled: usize,
    length: u64,
}

impl Sha256 {
    pub(crate) fn new() -> Self {
        Self {
            state: H0,
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }

    pub(crate) fn update(&mut self, bytes: impl AsRef<[u8]>) {
        let mut bytes = bytes.as_ref();
        self.length = self.length.wrapping_add(bytes.len() as u64);
        if self.filled > 0 {
            let take = (64 - self.filled).min(bytes.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled < 64 {
                return;
            }
            compress(&mut self.state, &self.block);
            self.filled = 0;
        }
        let mut blocks = bytes.chunks_exact(64);
        for block in &mut blocks {
            let whole = <&[u8; 64]>::try_from(block).expect("chunks_exact(64) yields 64 bytes");
            compress(&mut self.state, whole);
        }
        let rest = blocks.remainder();
        self.block[..rest.len()].copy_from_slice(rest);
        self.filled = rest.len();
    }

    /// Pads with one 1 bit, zeros to 56 bytes of the last block and the message
    /// length in bits as a big-endian u64, then returns the eight state words.
    pub(crate) fn finalize(mut self) -> [u8; 32] {
        let bits = self.length.wrapping_mul(8);
        // `update` always leaves fewer than 64 bytes waiting.
        let filled = self.filled;
        self.block[filled] = 0x80;
        if filled >= 56 {
            self.block[filled + 1..].fill(0);
            compress(&mut self.state, &self.block);
            self.block[..56].fill(0);
        } else {
            self.block[filled + 1..56].fill(0);
        }
        self.block[56..].copy_from_slice(&bits.to_be_bytes());
        compress(&mut self.state, &self.block);
        let mut out = [0u8; 32];
        for (bytes, word) in out.chunks_exact_mut(4).zip(self.state) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

/// The SHA-256 digest of `bytes`.
pub(crate) fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finalize()
}

/// The straightforward form this module had before its rounds were unrolled,
/// kept only as the reference the differential tests compare against.
#[cfg(test)]
mod reference {
    use super::{H0, K};

    pub(super) fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
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

    pub(super) struct Sha256 {
        state: [u32; 8],
        block: [u8; 64],
        filled: usize,
        length: u64,
    }

    impl Sha256 {
        pub(super) fn new() -> Self {
            Self {
                state: H0,
                block: [0; 64],
                filled: 0,
                length: 0,
            }
        }

        pub(super) fn update(&mut self, bytes: impl AsRef<[u8]>) {
            let mut bytes = bytes.as_ref();
            self.length = self.length.wrapping_add(bytes.len() as u64);
            if self.filled > 0 {
                let take = (64 - self.filled).min(bytes.len());
                self.block[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
                self.filled += take;
                bytes = &bytes[take..];
                if self.filled < 64 {
                    return;
                }
                compress(&mut self.state, &self.block);
                self.filled = 0;
            }
            let mut blocks = bytes.chunks_exact(64);
            for block in &mut blocks {
                let mut whole = [0u8; 64];
                whole.copy_from_slice(block);
                compress(&mut self.state, &whole);
            }
            let rest = blocks.remainder();
            self.block[..rest.len()].copy_from_slice(rest);
            self.filled = rest.len();
        }

        pub(super) fn finalize(mut self) -> [u8; 32] {
            let bits = self.length.wrapping_mul(8);
            self.update([0x80u8]);
            while self.filled != 56 {
                self.update([0u8]);
            }
            self.update(bits.to_be_bytes());
            let mut out = [0u8; 32];
            for (bytes, word) in out.chunks_exact_mut(4).zip(self.state) {
                bytes.copy_from_slice(&word.to_be_bytes());
            }
            out
        }
    }

    pub(super) fn sha256(bytes: &[u8]) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(bytes);
        hash.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    /// A small seeded generator for the differential tests.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            // splitmix64
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
        fn bytes(&mut self, len: usize) -> Vec<u8> {
            (0..len).map(|_| self.next() as u8).collect()
        }
    }

    #[test]
    fn compress_matches_the_reference_on_seeded_and_extreme_blocks() {
        let mut rng = Rng(0x5348_4132_3536_0001);
        let mut extremes: Vec<([u32; 8], [u8; 64])> = Vec::new();
        for fill in [0x00u8, 0xff, 0x80, 0x01, 0x55, 0xaa] {
            for state in [H0, [0; 8], [u32::MAX; 8], [0x8000_0000; 8]] {
                extremes.push((state, [fill; 64]));
            }
        }
        for (state, block) in extremes {
            let mut fast = state;
            let mut slow = state;
            compress(&mut fast, &block);
            reference::compress(&mut slow, &block);
            assert_eq!(fast, slow, "state {state:08x?} block {:02x}", block[0]);
        }
        for case in 0..20_000 {
            let mut state = [0u32; 8];
            for word in state.iter_mut() {
                *word = rng.next() as u32;
            }
            let mut block = [0u8; 64];
            block.copy_from_slice(&rng.bytes(64));
            let mut fast = state;
            let mut slow = state;
            // Chain a few blocks so errors in the state update also show.
            for _ in 0..3 {
                compress(&mut fast, &block);
                reference::compress(&mut slow, &block);
                assert_eq!(fast, slow, "case {case}");
                block[rng.below(64)] ^= 1 << rng.below(8);
            }
        }
    }

    #[test]
    fn every_length_and_seed_matches_the_reference_digest() {
        for seed in 0..4u64 {
            let mut rng = Rng(0x5348_4132_3536_1000 ^ seed);
            let m = rng.bytes(4096);
            for len in 0..=4096 {
                assert_eq!(
                    sha256(&m[..len]),
                    reference::sha256(&m[..len]),
                    "seed {seed} length {len}"
                );
            }
        }
        // Uniform messages near every padding boundary.
        for fill in [0x00u8, 0xff, 0x80] {
            for len in [0, 1, 55, 56, 57, 63, 64, 65, 119, 120, 121, 127, 128, 129] {
                let m = vec![fill; len];
                assert_eq!(
                    sha256(&m),
                    reference::sha256(&m),
                    "fill {fill} length {len}"
                );
            }
        }
        // A large input with every tail length.
        let mut rng = Rng(0x5348_4132_3536_2000);
        let big = rng.bytes((1 << 18) + 64);
        for tail in 0..=64 {
            let m = &big[..(1 << 18) + tail];
            assert_eq!(sha256(m), reference::sha256(m), "256 KiB plus {tail}");
        }
    }

    #[test]
    fn random_pieces_match_the_reference_digest() {
        let mut rng = Rng(0x5348_4132_3536_3000);
        for case in 0..3_000 {
            let len = match case % 3 {
                0 => rng.below(200),
                1 => rng.below(1100),
                _ => rng.below(9000),
            };
            let m = rng.bytes(len);
            let mut fast = Sha256::new();
            let mut slow = reference::Sha256::new();
            let mut at = 0;
            while at < len {
                // Empty pieces too, and pieces of every size class.
                let piece = match rng.below(4) {
                    0 => 0,
                    1 => rng.below(8),
                    2 => rng.below(130),
                    _ => rng.below(1000),
                }
                .min(len - at);
                fast.update(&m[at..at + piece]);
                slow.update(&m[at..at + piece]);
                at += piece;
            }
            if rng.below(2) == 0 {
                fast.update([0u8; 0]);
                slow.update([0u8; 0]);
            }
            let want = slow.finalize();
            assert_eq!(fast.finalize(), want, "case {case} length {len}");
            assert_eq!(sha256(&m), want, "case {case} length {len} whole");
        }
    }

    /// Deterministic bytes that are not a simple repetition.
    fn message(len: usize) -> Vec<u8> {
        let mut state = 0x9e37_79b9_7f4a_7c15u64 ^ len as u64;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 24) as u8
            })
            .collect()
    }

    #[test]
    fn the_fips_example_messages_give_their_published_digests() {
        let cases: [(&[u8], &str); 4] = [
            (b"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
            (b"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
            (
                b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
                "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
            ),
        ];
        for (input, want) in cases {
            assert_eq!(hex(&sha256(input)), want, "{} bytes", input.len());
        }
        let mut million = Sha256::new();
        for _ in 0..1000 {
            million.update([b'a'; 1000]);
        }
        assert_eq!(
            hex(&million.finalize()),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn every_length_up_to_1024_bytes_agrees_with_an_independent_implementation() {
        // The SHA-256 of the 1,025 digests of message(0..=1024) joined, computed
        // with RustCrypto sha2 0.10.9 on 2026-10-17, before that crate left the tree.
        let mut joined = Vec::new();
        for len in 0..=1024 {
            joined.extend_from_slice(&sha256(&message(len)));
        }
        assert_eq!(
            hex(&sha256(&joined)),
            "e4992c13ead0cbd6248587f54c370c9fa9ecc561be6953a72b411209eff8f3cc"
        );
    }

    #[test]
    fn feeding_a_message_in_pieces_gives_the_digest_of_the_whole() {
        for len in 0..=130 {
            let m = message(len);
            let whole = sha256(&m);
            for cut in 0..=len {
                let mut h = Sha256::new();
                h.update(&m[..cut]);
                h.update(&m[cut..]);
                assert_eq!(h.finalize(), whole, "length {len}, cut {cut}");
                let second = cut + (len - cut) / 2;
                let mut h = Sha256::new();
                h.update(&m[..cut]);
                h.update(&m[cut..second]);
                h.update(&m[second..]);
                assert_eq!(h.finalize(), whole, "length {len}, cuts {cut} and {second}");
            }
        }
        // The pattern of a collection root: small integers, then texts.
        let mut h = Sha256::new();
        let mut joined = Vec::new();
        for (i, text) in ["", "a", "Zażółć gęślą jaźń", &"x".repeat(200)]
            .iter()
            .enumerate()
        {
            for piece in [
                &(i as u64).to_le_bytes()[..],
                &(text.len() as u32).to_le_bytes()[..],
            ] {
                h.update(piece);
                joined.extend_from_slice(piece);
            }
            h.update(text.as_bytes());
            joined.extend_from_slice(text.as_bytes());
        }
        assert_eq!(h.finalize(), sha256(&joined));
    }

    /// Yields its bytes in pieces of at most `piece`, with an interruption first.
    struct Pieces<'a> {
        bytes: &'a [u8],
        piece: usize,
        interrupted: bool,
    }
    impl std::io::Read for Pieces<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            let n = self.piece.min(out.len()).min(self.bytes.len());
            out[..n].copy_from_slice(&self.bytes[..n]);
            self.bytes = &self.bytes[n..];
            Ok(n)
        }
    }

    #[test]
    fn digest_reader_gives_the_digest_of_the_whole() {
        let large = message(3 << 20);
        for m in (0..=1025).map(message).chain([large]) {
            for piece in [1, 7, 64 << 10] {
                if m.len() > 4096 && piece < 64 {
                    continue;
                }
                let reader = Pieces {
                    bytes: &m,
                    piece,
                    interrupted: false,
                };
                assert_eq!(
                    crate::digest_reader(reader).unwrap(),
                    sha256(&m),
                    "{} bytes in pieces of {piece}",
                    m.len()
                );
            }
        }
    }
}
