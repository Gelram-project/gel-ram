//! The comparison behind `examples/identity_nfc_fast.rs`: the current `nfc`
//! module (properties from one two-level table, composites from a hash table,
//! reused buffers, the verified prefix copied as it is) against
//! `nfc_reference`, the earlier implementation, unchanged. Every scalar value
//! is compared for each property and for the NFC of one character; pairs,
//! the Unicode NormalizationTest, the seeded sets of the pinned tests and
//! seeded strings with a prefix that passes the quick check are compared for
//! the whole normalization. Last, `line_end::find_ascii` is compared with the
//! earlier line end search and ASCII check of `document::scan`.
use crate::nfc as fast;
use crate::nfc_reference as reference;
use crate::nfc_tables::{
    CANONICAL_COMBINING_CLASS, CANONICAL_DECOMPOSITION, NFC_QUICK_CHECK, PRIMARY_COMPOSITE,
};

/// How much to compare: `Full` in the identity program, `Small` in the unit
/// test, which runs unoptimized.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scale {
    Full,
    Small,
}

/// The cases of one comparison and how many of them were equal.
pub(crate) struct Tally {
    pub(crate) name: &'static str,
    pub(crate) cases: u64,
    pub(crate) equal: u64,
}

impl Tally {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            cases: 0,
            equal: 0,
        }
    }
    fn check(&mut self, equal: bool) {
        self.cases += 1;
        self.equal += u64::from(equal);
    }
}

/// PASS needs at least one case, and every case of every comparison equal.
pub(crate) fn verdict(tallies: &[Tally]) -> bool {
    !tallies.is_empty() && tallies.iter().all(|t| t.cases > 0 && t.equal == t.cases)
}

/// SplitMix64 from `seed`, as in the pinned tests of `nfc.rs`.
fn splitmix(mut state: u64) -> impl FnMut() -> u64 {
    move || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// The pool of the pinned tests of `nfc.rs`: every code point with a
/// combining class or a canonical decomposition, the Hangul jamo and three
/// syllables, and a to z.
fn pool() -> Vec<char> {
    let mut pool = std::collections::BTreeSet::new();
    for &(first, last, _) in CANONICAL_COMBINING_CLASS {
        pool.extend(first..=last);
    }
    for &(cp, a, b) in CANONICAL_DECOMPOSITION {
        pool.extend([cp, a]);
        if b != 0 {
            pool.insert(b);
        }
    }
    pool.extend(
        (0x1100..=0x1112)
            .chain(0x1161..=0x1175)
            .chain(0x11A8..=0x11C2)
            .chain([0xAC00, 0xAC01, 0xD7A3])
            .chain(u32::from(b'a')..=u32::from(b'z')),
    );
    pool.into_iter().filter_map(char::from_u32).collect()
}

fn marks() -> Vec<char> {
    CANONICAL_COMBINING_CLASS
        .iter()
        .flat_map(|&(a, b, _)| a..=b)
        .filter_map(char::from_u32)
        .collect()
}

/// Every second code point of a primary composite, sorted.
fn composite_seconds() -> Vec<u32> {
    let mut seconds: Vec<u32> = PRIMARY_COMPOSITE.iter().map(|t| t.1).collect();
    seconds.sort_unstable();
    seconds.dedup();
    seconds
}

/// Compares the NFC of `s` by `nfc`, by `nfc_in` with the shared `scratch`,
/// and the quick check, with the reference. `copied` counts the strings
/// whose normalization started after a copied prefix.
fn compare_string(
    s: &str,
    scratch: &mut fast::Scratch,
    nfc: &mut Tally,
    quick: &mut Tally,
    copied: &mut Tally,
) -> bool {
    let expected = reference::nfc(s);
    let by_nfc = fast::nfc(s) == expected;
    let by_scratch = fast::nfc_in(s, scratch) == expected;
    nfc.check(by_nfc && by_scratch);
    let stable = fast::quick_check(s);
    let same_quick = stable.is_none() == reference::is_nfc_by_quick_check(s);
    quick.check(same_quick);
    if stable.is_some_and(|p| p > 0) {
        copied.check(by_nfc && by_scratch);
    }
    by_nfc && by_scratch && same_quick
}

fn chars_of(field: &str) -> String {
    field
        .split(' ')
        .filter_map(|cp| u32::from_str_radix(cp, 16).ok())
        .filter_map(char::from_u32)
        .collect()
}

pub(crate) fn run(scale: Scale) -> Vec<Tally> {
    let full = scale == Scale::Full;
    let mut tallies = Vec::new();
    let scalars = || (0..=0x10_FFFFu32).filter(|&c| char::from_u32(c).is_some());
    let seconds = composite_seconds();

    // Per scalar: class, Quick_Check, composite second, decomposition, and
    // the boundary rule that the prefix copy relies on.
    let mut class = Tally::new("class_every_scalar");
    let mut quick = Tally::new("quick_check_flag_every_scalar");
    let mut second = Tally::new("composite_second_every_scalar");
    let mut decomposition = Tally::new("decompose_every_scalar");
    let mut boundary = Tally::new("prefix_boundary_rule_every_starter");
    let (mut got, mut want) = (Vec::new(), Vec::new());
    for c in scalars() {
        let properties = fast::properties(c);
        class.check((properties & fast::CLASS_MASK) as u8 == reference::combining_class(c));
        quick.check(
            (properties & fast::QUICK_CHECK_NOT_YES != 0)
                == reference::in_range(NFC_QUICK_CHECK, c).is_some(),
        );
        second
            .check((properties & fast::COMPOSITE_SECOND != 0) == seconds.binary_search(&c).is_ok());
        got.clear();
        want.clear();
        fast::decompose(c, &mut got);
        reference::decompose(c, &mut want);
        decomposition.check(got == want);
        if reference::combining_class(c) == 0 && reference::in_range(NFC_QUICK_CHECK, c).is_none() {
            let (head_class, head) = want[0];
            boundary.check(
                head_class == 0
                    && seconds.binary_search(&head).is_err()
                    && !(0x1161..=0x1175).contains(&head)
                    && !(0x11A8..=0x11C2).contains(&head),
            );
        }
    }
    tallies.extend([class, quick, second, decomposition, boundary]);

    // compose: every first against every second that can compose (the
    // composite seconds, the Hangul vowels and trailing consonants, and
    // the edges of those ranges), and the firsts that can compose against
    // every second.
    let mut second_pool: Vec<u32> = seconds.clone();
    second_pool.extend(0x1160..=0x1176);
    second_pool.extend(0x11A6..=0x11C3);
    second_pool.extend([0, 0x41, 0x61, 0x2FF, 0x300, 0xAC00, 0x10_FFFF]);
    let mut first_pool: Vec<u32> = PRIMARY_COMPOSITE.iter().map(|t| t.0).collect();
    first_pool.extend(0x10FF..=0x1113);
    first_pool.extend([
        0xABFF, 0xAC00, 0xAC01, 0xAC1B, 0xAC1C, 0xD788, 0xD7A3, 0xD7A4,
    ]);
    let mut next = splitmix(0x6e66_6331);
    for _ in 0..64 {
        // A syllable without a trailing consonant, and any syllable.
        first_pool.push(0xAC00 + (next() % (19 * 21)) as u32 * 28);
        first_pool.push(0xAC00 + (next() % 11_172) as u32);
    }
    first_pool.sort_unstable();
    first_pool.dedup();
    let stride = if full { 1 } else { 101 };
    let mut by_first = Tally::new("compose_every_first");
    for first in scalars().step_by(stride) {
        for &second in &second_pool {
            by_first.check(fast::compose(first, second) == reference::compose(first, second));
        }
    }
    let mut by_second = Tally::new("compose_every_second");
    for second in scalars().step_by(stride) {
        for &first in &first_pool {
            by_second.check(fast::compose(first, second) == reference::compose(first, second));
        }
    }
    let mut random_pairs = Tally::new("compose_random_pairs");
    let pool = pool();
    let count = if full { 4_000_000 } else { 200_000 };
    for _ in 0..count {
        let mut pick = || match next() % 3 {
            0 => (next() % 0x11_0000) as u32,
            _ => u32::from(pool[(next() % pool.len() as u64) as usize]),
        };
        let (first, second) = (pick(), pick());
        random_pairs.check(fast::compose(first, second) == reference::compose(first, second));
    }
    tallies.extend([by_first, by_second, random_pairs]);

    // Whole strings. One scratch serves every string, as it serves every
    // line of a search.
    let mut scratch = fast::Scratch::default();
    let mut copied = Tally::new("nfc_after_copied_prefix");
    let mut one = Tally::new("nfc_every_scalar");
    let mut one_quick = Tally::new("quick_check_every_scalar");
    for c in scalars().filter_map(char::from_u32) {
        compare_string(
            &c.to_string(),
            &mut scratch,
            &mut one,
            &mut one_quick,
            &mut copied,
        );
    }
    tallies.extend([one, one_quick]);

    let mut unicode = Tally::new("nfc_normalization_test_15_1");
    let mut unicode_quick = Tally::new("quick_check_normalization_test_15_1");
    let text = include_str!("../tests/data/NormalizationTest-15.1.0.txt");
    for line in text.lines() {
        let data = line.split('#').next().unwrap_or("").trim();
        if data.is_empty() || data.starts_with('@') {
            continue;
        }
        for field in data.split(';').take(5) {
            compare_string(
                &chars_of(field),
                &mut scratch,
                &mut unicode,
                &mut unicode_quick,
                &mut copied,
            );
        }
    }
    tallies.extend([unicode, unicode_quick]);

    // The 100,000 strings of the pinned random test, each also after the
    // previous one and a line break.
    let mut random = Tally::new("nfc_random_pool_100000");
    let mut random_quick = Tally::new("quick_check_random_pool_100000");
    let mut next = splitmix(20_261_017);
    let mut previous = String::new();
    for _ in 0..100_000 {
        let len = 1 + (next() % 8) as usize;
        let s: String = (0..len)
            .map(|_| pool[(next() % pool.len() as u64) as usize])
            .collect();
        compare_string(
            &s,
            &mut scratch,
            &mut random,
            &mut random_quick,
            &mut copied,
        );
        compare_string(
            &format!("{previous}\n{s}"),
            &mut scratch,
            &mut random,
            &mut random_quick,
            &mut copied,
        );
        previous = s;
    }
    tallies.extend([random, random_quick]);

    // The 10,005 strings of the pinned adversarial test.
    let marks = marks();
    let mut adversarial = Tally::new("nfc_adversarial_10005");
    let mut adversarial_quick = Tally::new("quick_check_adversarial_10005");
    let mut next = splitmix(18);
    let mut strings = vec![
        format!("a{}{}", "\u{0301}".repeat(1023), "\u{0316}".repeat(1024)),
        format!("a{}{}", "\u{0316}".repeat(1024), "\u{0301}".repeat(1023)),
        "Zażółć gęślą jaźń, źdźbło i łódź. ".repeat(40),
        "\u{1100}\u{1161}\u{11A8}".repeat(450),
        "한국어 문장은 대부분 완성형 음절로 쓴다 ".repeat(40),
    ];
    for _ in 0..10_000 {
        let len = 1 + (next() % 64) as usize;
        strings.push(
            (0..len)
                .map(|_| {
                    if next() % 3 == 0 {
                        pool[(next() % pool.len() as u64) as usize]
                    } else {
                        marks[(next() % marks.len() as u64) as usize]
                    }
                })
                .collect(),
        );
    }
    for s in &strings {
        compare_string(
            s,
            &mut scratch,
            &mut adversarial,
            &mut adversarial_quick,
            &mut copied,
        );
    }
    tallies.extend([adversarial, adversarial_quick]);

    // A prefix of characters that pass the quick check on their own (letters,
    // Polish, Hangul, CJK, marks of Quick_Check Yes), then a tail drawn from
    // the pool and the marks, so the quick check fails at many depths.
    let yes: Vec<char> = pool
        .iter()
        .copied()
        .chain("AaZzłŁóÓąęćńśźżé한국어漢字ЖжΣσ".chars())
        .filter(|&c| reference::in_range(NFC_QUICK_CHECK, c as u32).is_none())
        .collect();
    let mut split = Tally::new("nfc_prefix_then_tail");
    let mut split_quick = Tally::new("quick_check_prefix_then_tail");
    let mut next = splitmix(0x7072_6566);
    let count = if full { 300_000 } else { 30_000 };
    for _ in 0..count {
        let head = (next() % 48) as usize;
        let tail = (next() % 12) as usize;
        let mut s: String = (0..head)
            .map(|_| yes[(next() % yes.len() as u64) as usize])
            .collect();
        s.extend((0..tail).map(|_| match next() % 3 {
            0 => pool[(next() % pool.len() as u64) as usize],
            1 => marks[(next() % marks.len() as u64) as usize],
            _ => yes[(next() % yes.len() as u64) as usize],
        }));
        compare_string(&s, &mut scratch, &mut split, &mut split_quick, &mut copied);
    }
    tallies.extend([split, split_quick]);

    // Lines up to the 4096-byte line limit of a search: the timed lines and
    // seeded mixtures of ASCII, Polish, decomposed Polish, marks and jamo.
    let mut long = Tally::new("nfc_long_lines");
    let mut long_quick = Tally::new("quick_check_long_lines");
    let polish = "Zażółć gęślą jaźń, źdźbło i łódź. ";
    let polish_nfd = "Za\u{007A}\u{0307}o\u{0301}\u{0142}c\u{0301} ge\u{0328}s\u{0301}\u{006C}a\u{0328} \
                      jaz\u{0301}n\u{0301}, z\u{0301}dz\u{0301}b\u{0142}o i \u{0142}o\u{0301}dz\u{0301}. ";
    let mut lines = vec![
        polish.repeat(40),
        polish_nfd.repeat(40),
        format!("{}cafe\u{0301}", polish.repeat(38)),
        format!("{}\u{0301}", polish.repeat(38)),
        format!("\u{0301}{}", polish.repeat(38)),
        "\u{1100}\u{1161}\u{11A8}".repeat(450),
        "\u{1100}\u{1161}".repeat(600),
        "한국어 문장은 대부분 완성형 음절로 쓴다 ".repeat(40),
        format!("a{}{}", "\u{0301}".repeat(1023), "\u{0316}".repeat(1024)),
        format!("a{}{}", "\u{0316}".repeat(1024), "\u{0301}".repeat(1023)),
    ];
    let pieces: [&str; 8] = [
        "plain ascii words ",
        polish,
        polish_nfd,
        "\u{1100}\u{1161}\u{11A8}",
        "e\u{0323}\u{0301}",
        "\u{0316}\u{0301}\u{0345}",
        "한국어 ",
        "Ω\u{0301}ΣΣς ",
    ];
    let mut next = splitmix(0x6c69_6e65);
    let count = if full { 3_000 } else { 200 };
    for _ in 0..count {
        let mut line = String::new();
        let target = 1 + (next() % 4096) as usize;
        while line.len() < target {
            if next() % 4 == 0 {
                line.push(marks[(next() % marks.len() as u64) as usize]);
            } else {
                line.push_str(pieces[(next() % pieces.len() as u64) as usize]);
            }
        }
        lines.push(line);
    }
    for line in &lines {
        compare_string(line, &mut scratch, &mut long, &mut long_quick, &mut copied);
    }
    tallies.extend([long, long_quick, copied]);
    tallies.extend(line_end_cases(full, &lines));
    tallies
}

/// The line end search of `document::scan` before `line_end::find`, unchanged.
fn line_end_reference(bytes: &[u8], start: usize, end: usize) -> usize {
    bytes[start..end]
        .iter()
        .position(|&b| b == b'\r' || b == b'\n')
        .map_or(end, |p| start + p)
}

/// The two steps of `document::scan` before `line_end::find_ascii`: the line
/// end, then `is_ascii` of the line.
fn line_reference(bytes: &[u8], start: usize, end: usize) -> (usize, bool) {
    let line_end = line_end_reference(bytes, start, end);
    (line_end, bytes[start..line_end].is_ascii())
}

/// Compares `line_end::find_ascii` with the reference for every `start..end`
/// of `bytes` with `start <= starts`.
fn compare_line_ends(bytes: &[u8], starts: usize, tally: &mut Tally) {
    for start in 0..=bytes.len().min(starts) {
        for end in start..=bytes.len() {
            tally.check(
                crate::line_end::find_ascii(bytes, start, end) == line_reference(bytes, start, end),
            );
        }
    }
}

/// `line_end::find_ascii` against the earlier search: short buffers with one or two
/// marked bytes at every position (word boundaries and remainders), seeded
/// buffers up to past the line limit, and texts made of the long lines above
/// with '\n' and "\r\n", walked line by line as `scan` walks them.
fn line_end_cases(full: bool, lines: &[String]) -> Vec<Tally> {
    // The two line ends, bytes next to them, and bytes whose low seven bits
    // match a line end.
    const MARKED: [u8; 11] = [
        b'\n', b'\r', 0x00, 0x09, 0x0B, 0x0C, 0x0E, 0x8A, 0x8D, 0xFF, b'a',
    ];
    let max_len = if full { 40 } else { 24 };
    let mut one = Tally::new("line_end_one_marked_byte");
    let mut buffer = Vec::new();
    for len in 0..=max_len {
        buffer.clear();
        buffer.resize(len, b'x');
        compare_line_ends(&buffer, len, &mut one);
        for at in 0..len {
            for &b in &MARKED {
                buffer[at] = b;
                compare_line_ends(&buffer, len, &mut one);
            }
            buffer[at] = b'x';
        }
    }
    let mut two = Tally::new("line_end_two_marked_bytes");
    let pair_bytes = [b'\n', b'\r', 0x0B, 0x8A, 0x8D];
    let max_len = if full { 24 } else { 17 };
    for len in 0..=max_len {
        for i in 0..len {
            for j in i + 1..len {
                for &a in &pair_bytes {
                    for &b in &pair_bytes {
                        buffer.clear();
                        buffer.resize(len, b'y');
                        buffer[i] = a;
                        buffer[j] = b;
                        compare_line_ends(&buffer, 9, &mut two);
                    }
                }
            }
        }
    }
    let mut seeded = Tally::new("line_end_seeded_buffers");
    let mut next = splitmix(0x6c65_6e64);
    let count = if full { 20_000 } else { 1_000 };
    for _ in 0..count {
        let len = (next() % 4113) as usize;
        // One byte in `density` (0: none) is a line end.
        let density = next() % 6;
        buffer.clear();
        buffer.extend((0..len).map(|_| {
            let r = next();
            if density != 0 && r % (1 << (2 * density)) == 0 {
                if r & (1 << 40) == 0 {
                    b'\n'
                } else {
                    b'\r'
                }
            } else {
                MARKED[2 + ((r >> 8) % 9) as usize] ^ ((r >> 16) as u8 & 0x70)
            }
        }));
        for _ in 0..8 {
            let a = (next() % (len as u64 + 1)) as usize;
            let b = (next() % (len as u64 + 1)) as usize;
            let (start, end) = (a.min(b), a.max(b));
            seeded.check(
                crate::line_end::find_ascii(&buffer, start, end)
                    == line_reference(&buffer, start, end),
            );
        }
    }
    let mut texts = Tally::new("line_end_text_lines");
    for separator in ["\n", "\r\n"] {
        let mut text = String::new();
        for line in lines {
            text.push_str(line);
            text.push_str(separator);
        }
        text.push_str("plain ascii words of a source line, number 42. ");
        let bytes = text.as_bytes();
        let mut start = 0;
        while start < bytes.len() {
            let got = crate::line_end::find_ascii(bytes, start, bytes.len());
            let want = line_reference(bytes, start, bytes.len());
            texts.check(got == want);
            start = if want.0 < bytes.len() {
                want.0 + 1
            } else {
                bytes.len()
            };
        }
    }
    vec![one, two, seeded, texts]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_nfc_matches_the_earlier_implementation() {
        let tallies = run(Scale::Small);
        for t in &tallies {
            assert_eq!(t.equal, t.cases, "{}", t.name);
        }
        assert!(verdict(&tallies));
    }
}
