//! Unicode Normalization Form C (UAX #15), written for this crate.
//!
//! A string that passes the NFC_Quick_Check rule is returned as it is.
//! Otherwise it is decomposed canonically (Hangul syllables by arithmetic,
//! other characters by the recursive canonical decomposition mappings), its
//! combining marks are put in canonical order, and it is recomposed: each
//! starter is combined with the next unblocked character that forms a primary
//! composite with it. Each character's combining class is looked up once, so
//! the cost grows as n log n in the length of a run of combining marks. The
//! tables in `nfc_tables.rs` come from the Unicode Character Database; the
//! tests run the Unicode NormalizationTest of the same version and of the
//! previous version the tables followed (15.1.0), and compare every code point
//! with an earlier implementation.
//!
//! The properties of a code point (combining class, NFC_Quick_Check not Yes,
//! second code point of a primary composite, canonical decomposition) are
//! read from one two-level table, and primary composites from a hash table;
//! both are built at compile time from the tables in `nfc_tables.rs`. A
//! string that fails the quick check is normalized from the last character
//! before the failure that has combining class 0 and NFC_Quick_Check Yes; the
//! text before it is copied as it is.
use crate::nfc_tables::{
    CANONICAL_COMBINING_CLASS, CANONICAL_DECOMPOSITION, NFC_QUICK_CHECK, PRIMARY_COMPOSITE,
};

const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

/// Every code point below this one has combining class 0 and NFC_Quick_Check
/// Yes (checked against the tables by a test).
const FIRST_NOT_TRIVIAL: u32 = 0x0300;
/// The Hangul Jamo block, U+1100..U+11FF.
#[cfg(test)]
const JAMO_BLOCK: std::ops::RangeInclusive<u32> = 0x1100..=0x11FF;

#[cfg(test)]
thread_local! {
    /// Property lookups made by this thread, for the cost test.
    static LOOKUPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Code points per block of the property table.
const BLOCK_SHIFT: u32 = 6;
const BLOCK: usize = 1 << BLOCK_SHIFT;
/// Blocks covering U+0000..U+10FFFF.
const BLOCKS: usize = 0x11_0000 >> BLOCK_SHIFT;
/// Property bits: the combining class in the low 8 bits, two flags, and the
/// index in CANONICAL_DECOMPOSITION plus one (0: none) from bit 16.
pub(crate) const CLASS_MASK: u32 = 0xFF;
pub(crate) const QUICK_CHECK_NOT_YES: u32 = 1 << 8;
pub(crate) const COMPOSITE_SECOND: u32 = 1 << 9;
const DECOMPOSITION_SHIFT: u32 = 16;

/// Marks a block for every code point that has a property.
const fn used_blocks() -> [bool; BLOCKS] {
    let mut used = [false; BLOCKS];
    let mut i = 0;
    while i < CANONICAL_COMBINING_CLASS.len() {
        let (first, last, _) = CANONICAL_COMBINING_CLASS[i];
        let mut b = first >> BLOCK_SHIFT;
        while b <= last >> BLOCK_SHIFT {
            used[b as usize] = true;
            b += 1;
        }
        i += 1;
    }
    i = 0;
    while i < NFC_QUICK_CHECK.len() {
        let (first, last, _) = NFC_QUICK_CHECK[i];
        let mut b = first >> BLOCK_SHIFT;
        while b <= last >> BLOCK_SHIFT {
            used[b as usize] = true;
            b += 1;
        }
        i += 1;
    }
    i = 0;
    while i < CANONICAL_DECOMPOSITION.len() {
        used[(CANONICAL_DECOMPOSITION[i].0 >> BLOCK_SHIFT) as usize] = true;
        i += 1;
    }
    i = 0;
    while i < PRIMARY_COMPOSITE.len() {
        used[(PRIMARY_COMPOSITE[i].1 >> BLOCK_SHIFT) as usize] = true;
        i += 1;
    }
    used
}

const fn used_block_count() -> usize {
    let used = used_blocks();
    let (mut n, mut b) = (0, 0);
    while b < BLOCKS {
        if used[b] {
            n += 1;
        }
        b += 1;
    }
    n
}

/// Block 0 holds no property; every other block holds at least one.
const DATA_BLOCKS: usize = 1 + used_block_count();
// A block number is stored in one byte of `Properties::index`.
const _: () = assert!(DATA_BLOCKS <= 256);

struct Properties {
    index: [u8; BLOCKS],
    data: [[u32; BLOCK]; DATA_BLOCKS],
}

const fn build_properties() -> Properties {
    assert!(CANONICAL_DECOMPOSITION.len() < 1 << (32 - DECOMPOSITION_SHIFT));
    let used = used_blocks();
    let mut index = [0u8; BLOCKS];
    let (mut next, mut b) = (1, 0);
    while b < BLOCKS {
        if used[b] {
            index[b] = next as u8;
            next += 1;
        }
        b += 1;
    }
    let mut data = [[0u32; BLOCK]; DATA_BLOCKS];
    let mut i = 0;
    while i < CANONICAL_COMBINING_CLASS.len() {
        let (first, last, class) = CANONICAL_COMBINING_CLASS[i];
        let mut c = first;
        while c <= last {
            data[index[(c >> BLOCK_SHIFT) as usize] as usize][c as usize & (BLOCK - 1)] |=
                class as u32;
            c += 1;
        }
        i += 1;
    }
    i = 0;
    while i < NFC_QUICK_CHECK.len() {
        let (first, last, _) = NFC_QUICK_CHECK[i];
        let mut c = first;
        while c <= last {
            data[index[(c >> BLOCK_SHIFT) as usize] as usize][c as usize & (BLOCK - 1)] |=
                QUICK_CHECK_NOT_YES;
            c += 1;
        }
        i += 1;
    }
    i = 0;
    while i < CANONICAL_DECOMPOSITION.len() {
        let c = CANONICAL_DECOMPOSITION[i].0;
        data[index[(c >> BLOCK_SHIFT) as usize] as usize][c as usize & (BLOCK - 1)] |=
            ((i + 1) as u32) << DECOMPOSITION_SHIFT;
        i += 1;
    }
    i = 0;
    while i < PRIMARY_COMPOSITE.len() {
        let c = PRIMARY_COMPOSITE[i].1;
        // `compose` rejects every second code point below this one unread.
        assert!(c >= FIRST_NOT_TRIVIAL);
        data[index[(c >> BLOCK_SHIFT) as usize] as usize][c as usize & (BLOCK - 1)] |=
            COMPOSITE_SECOND;
        i += 1;
    }
    Properties { index, data }
}

static PROPERTIES: Properties = build_properties();

/// The property bits of `c`.
pub(crate) fn properties(c: u32) -> u32 {
    #[cfg(test)]
    LOOKUPS.with(|n| n.set(n.get() + 1));
    let block = PROPERTIES
        .index
        .get((c >> BLOCK_SHIFT) as usize)
        .copied()
        .unwrap_or(0);
    PROPERTIES.data[block as usize][c as usize & (BLOCK - 1)]
}

#[cfg(test)]
fn combining_class(c: u32) -> u8 {
    (properties(c) & CLASS_MASK) as u8
}

/// Slots of the primary composite hash table: more than twice the composites.
const SLOTS: usize = 2048;

const fn slot_of(first: u32, second: u32) -> usize {
    ((first.wrapping_mul(0x9E37_79B1) ^ second.wrapping_mul(0x85EB_CA77)).wrapping_mul(0xC2B2_AE3D)
        >> (32 - SLOTS.trailing_zeros())) as usize
}

/// Open addressing with linear probing: each slot holds the index in
/// PRIMARY_COMPOSITE plus one, or 0 when empty.
const fn build_composite_slots() -> [u16; SLOTS] {
    assert!(PRIMARY_COMPOSITE.len() < SLOTS / 2);
    let mut slots = [0u16; SLOTS];
    let mut i = 0;
    while i < PRIMARY_COMPOSITE.len() {
        let (first, second, _) = PRIMARY_COMPOSITE[i];
        let mut slot = slot_of(first, second);
        while slots[slot] != 0 {
            slot = (slot + 1) & (SLOTS - 1);
        }
        slots[slot] = (i + 1) as u16;
        i += 1;
    }
    slots
}

static COMPOSITE_SLOTS: [u16; SLOTS] = build_composite_slots();

/// None when `s` is certainly in NFC by the NFC_Quick_Check rule of UAX #15:
/// no character has NFC_Quick_Check No or Maybe, and the nonzero combining
/// classes never decrease. Otherwise the byte offset of the last character
/// before the first failing one that has combining class 0 and
/// NFC_Quick_Check Yes, or 0 when there is none.
pub(crate) fn quick_check(s: &str) -> Option<usize> {
    let mut last = 0;
    let mut stable = 0;
    for (at, c) in s.char_indices() {
        let c = c as u32;
        if c < FIRST_NOT_TRIVIAL {
            last = 0;
            stable = at;
            continue;
        }
        let properties = properties(c);
        if properties & QUICK_CHECK_NOT_YES != 0 {
            return Some(stable);
        }
        let class = (properties & CLASS_MASK) as u8;
        if class != 0 && last > class {
            return Some(stable);
        }
        if class == 0 {
            stable = at;
        }
        last = class;
    }
    None
}

/// Appends the full canonical decomposition of `c`, each code point with its
/// combining class.
pub(crate) fn decompose(c: u32, out: &mut Vec<(u8, u32)>) {
    if (S_BASE..S_BASE + S_COUNT).contains(&c) {
        // Hangul jamo all have combining class 0.
        let s = c - S_BASE;
        out.push((0, L_BASE + s / N_COUNT));
        out.push((0, V_BASE + (s % N_COUNT) / T_COUNT));
        if s % T_COUNT != 0 {
            out.push((0, T_BASE + s % T_COUNT));
        }
        return;
    }
    // Nothing below the first decomposition decomposes or has a nonzero
    // class (checked by a test).
    if c < CANONICAL_DECOMPOSITION[0].0 {
        out.push((0, c));
        return;
    }
    let properties = properties(c);
    match properties >> DECOMPOSITION_SHIFT {
        0 => out.push(((properties & CLASS_MASK) as u8, c)),
        i => {
            let (_, first, second) = CANONICAL_DECOMPOSITION[i as usize - 1];
            decompose(first, out);
            if second != 0 {
                decompose(second, out);
            }
        }
    }
}

pub(crate) fn compose(first: u32, second: u32) -> Option<u32> {
    if (L_BASE..L_BASE + L_COUNT).contains(&first) && (V_BASE..V_BASE + V_COUNT).contains(&second) {
        return Some(S_BASE + ((first - L_BASE) * V_COUNT + (second - V_BASE)) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&first)
        && (first - S_BASE) % T_COUNT == 0
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&second)
    {
        return Some(first + (second - T_BASE));
    }
    if second < FIRST_NOT_TRIVIAL || properties(second) & COMPOSITE_SECOND == 0 {
        return None;
    }
    let mut slot = slot_of(first, second);
    loop {
        let entry = COMPOSITE_SLOTS[slot];
        if entry == 0 {
            return None;
        }
        let (a, b, composite) = PRIMARY_COMPOSITE[entry as usize - 1];
        if (a, b) == (first, second) {
            return Some(composite);
        }
        slot = (slot + 1) & (SLOTS - 1);
    }
}

/// Buffers reused by every normalization through them.
#[derive(Default)]
pub(crate) struct Scratch {
    decomposed: Vec<(u8, u32)>,
    text: String,
}

/// `s` in Normalization Form C.
pub(crate) fn nfc(s: &str) -> String {
    match quick_check(s) {
        None => s.to_owned(),
        Some(stable) => {
            let mut scratch = Scratch::default();
            normalize_from(s, stable, &mut scratch);
            scratch.text
        }
    }
}

/// `s` in Normalization Form C: `s` itself when it passes the quick check,
/// otherwise the text of `scratch`.
pub(crate) fn nfc_in<'a>(s: &'a str, scratch: &'a mut Scratch) -> &'a str {
    match quick_check(s) {
        None => s,
        Some(stable) => {
            normalize_from(s, stable, scratch);
            &scratch.text
        }
    }
}

/// Sets the text of `scratch` to the NFC of `s`, where `s[..stable]` passed the
/// quick check and `s[stable..]` starts with a character of combining class 0
/// and NFC_Quick_Check Yes (or `stable` is 0). The full decomposition of such
/// a character starts with a starter that is never the second code point of
/// a composite (checked by a test), so nothing before it is reordered or
/// composed with anything after it, and NFC leaves the prefix as it is.
fn normalize_from(s: &str, stable: usize, scratch: &mut Scratch) {
    scratch.text.clear();
    scratch.text.reserve(s.len());
    scratch.text.push_str(&s[..stable]);
    normalize_into(&s[stable..], scratch);
}

/// Appends to the text of `scratch` the decomposition, canonical ordering and
/// composition of `s`.
fn normalize_into(s: &str, scratch: &mut Scratch) {
    let decomposed = &mut scratch.decomposed;
    decomposed.clear();
    for c in s.chars() {
        decompose(c as u32, decomposed);
    }
    // Canonical ordering: a stable sort of each run of nonzero classes.
    let mut i = 0;
    while i < decomposed.len() {
        if decomposed[i].0 == 0 {
            i += 1;
            continue;
        }
        let start = i;
        while i < decomposed.len() && decomposed[i].0 != 0 {
            i += 1;
        }
        decomposed[start..i].sort_by_key(|&(class, _)| class);
    }
    // Canonical composition, in place: the first `kept` entries are the
    // output so far. `last` is the class of the last character kept after
    // the current starter; None means the next character is adjacent.
    let mut kept = 0;
    let mut starter: Option<usize> = None;
    let mut last: Option<u8> = None;
    for i in 0..decomposed.len() {
        let (class, c) = decomposed[i];
        if let Some(at) = starter {
            let blocked = last.is_some_and(|l| l >= class);
            if !blocked {
                if let Some(composite) = compose(decomposed[at].1, c) {
                    decomposed[at].1 = composite;
                    continue;
                }
            }
        }
        if class == 0 {
            starter = Some(kept);
            last = None;
        } else if starter.is_some() {
            last = Some(class);
        }
        decomposed[kept] = (class, c);
        kept += 1;
    }
    scratch.text.extend(
        decomposed[..kept]
            .iter()
            .filter_map(|&(_, c)| char::from_u32(c)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{digest, hex};

    fn chars(field: &str) -> String {
        field
            .split(' ')
            .map(|cp| char::from_u32(u32::from_str_radix(cp, 16).unwrap()).unwrap())
            .collect()
    }

    #[test]
    fn the_tables_are_unicode_16_0_with_their_published_counts() {
        assert_eq!(crate::nfc_tables::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(CANONICAL_DECOMPOSITION.len(), 2081);
        assert_eq!(PRIMARY_COMPOSITE.len(), 961);
        assert!(CANONICAL_DECOMPOSITION.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(PRIMARY_COMPOSITE
            .windows(2)
            .all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)));
        assert!(CANONICAL_COMBINING_CLASS
            .windows(2)
            .all(|w| w[0].1 < w[1].0 && w[0].0 <= w[0].1));
        assert_eq!(CANONICAL_COMBINING_CLASS.len(), 393);
        assert_eq!(NFC_QUICK_CHECK.len(), 122);
        assert!(NFC_QUICK_CHECK
            .windows(2)
            .all(|w| w[0].1 < w[1].0 && w[0].0 <= w[0].1));
        let count = |maybe: bool| -> u32 {
            NFC_QUICK_CHECK
                .iter()
                .filter(|r| r.2 == maybe)
                .map(|r| r.1 - r.0 + 1)
                .sum()
        };
        assert_eq!((count(false), count(true)), (1120, 132));
        // The shortcuts the code takes: nothing below U+0300 has a class or a
        // Quick_Check other than Yes, and the Hangul jamo are starters.
        assert_eq!(CANONICAL_COMBINING_CLASS[0].0, FIRST_NOT_TRIVIAL);
        assert_eq!(NFC_QUICK_CHECK[0].0, FIRST_NOT_TRIVIAL);
        for c in JAMO_BLOCK {
            assert_eq!(combining_class(c), 0, "{c:04X}");
        }
        assert!(CANONICAL_DECOMPOSITION
            .iter()
            .all(|&(cp, _, _)| !JAMO_BLOCK.contains(&cp)));
        assert!(JAMO_BLOCK.contains(&L_BASE) && JAMO_BLOCK.contains(&(T_BASE + T_COUNT - 1)));
        assert_eq!(CANONICAL_DECOMPOSITION[0].0, 0x00C0);
    }

    /// The worst run of combining marks a 4096-byte line can hold: every class
    /// 220 mark comes after every class 230 mark. Sorting them by insertion,
    /// looking each class up again, took over a million lookups.
    #[test]
    fn reversed_combining_marks_cost_one_class_lookup_per_character() {
        let line = format!("a{}{}", "\u{0301}".repeat(1023), "\u{0316}".repeat(1024));
        assert_eq!(line.len(), 4095);
        let before = LOOKUPS.with(std::cell::Cell::get);
        let out = nfc(&line);
        let lookups = LOOKUPS.with(std::cell::Cell::get) - before;
        let expected = format!(
            "\u{00E1}{}{}",
            "\u{0316}".repeat(1024),
            "\u{0301}".repeat(1022)
        );
        assert_eq!(out, expected);
        assert!(lookups <= 2 * 2048, "{lookups} class lookups");
        // Text already in NFC below U+0300, as most Polish text is, needs none.
        let before = LOOKUPS.with(std::cell::Cell::get);
        assert_eq!(nfc("Zażółć gęślą jaźń"), "Zażółć gęślą jaźń");
        assert_eq!(LOOKUPS.with(std::cell::Cell::get), before);
    }

    const NORMALIZATION_TEST_16_0: &str =
        include_str!("../tests/data/NormalizationTest-16.0.0.txt");
    const NORMALIZATION_TEST_15_1: &str =
        include_str!("../tests/data/NormalizationTest-15.1.0.txt");

    /// One Unicode NormalizationTest file run through `nfc`: every data line
    /// must satisfy c2 = NFC(c1) = NFC(c2) = NFC(c3) and c4 = NFC(c4) =
    /// NFC(c5). Returns the version of its header, its number of data lines,
    /// its distinct data lines and the characters it lists in part 1.
    fn run_normalization_test(
        text: &str,
    ) -> (
        String,
        usize,
        std::collections::BTreeSet<String>,
        std::collections::BTreeSet<char>,
    ) {
        let version = text
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("# NormalizationTest-"))
            .and_then(|l| l.strip_suffix(".txt"))
            .expect("NormalizationTest header")
            .to_owned();
        let mut listed = std::collections::BTreeSet::new();
        let mut distinct = std::collections::BTreeSet::new();
        let (mut lines, mut part1) = (0, false);
        for line in text.lines() {
            if let Some(part) = line.strip_prefix("@Part") {
                part1 = part.starts_with('1');
                continue;
            }
            let data = line.split('#').next().unwrap_or("").trim();
            if data.is_empty() {
                continue;
            }
            let c: Vec<String> = data.split(';').take(5).map(chars).collect();
            assert_eq!(nfc(&c[0]), c[1], "NFC(c1) at {line}");
            assert_eq!(nfc(&c[1]), c[1], "NFC(c2) at {line}");
            assert_eq!(nfc(&c[2]), c[1], "NFC(c3) at {line}");
            assert_eq!(nfc(&c[3]), c[3], "NFC(c4) at {line}");
            assert_eq!(nfc(&c[4]), c[3], "NFC(c5) at {line}");
            if part1 {
                listed.insert(c[0].chars().next().unwrap());
            }
            distinct.insert(c.join(";"));
            lines += 1;
        }
        (version, lines, distinct, listed)
    }

    #[test]
    fn the_unicode_normalization_test_of_the_same_version_passes() {
        let (version, lines, distinct, listed) = run_normalization_test(NORMALIZATION_TEST_16_0);
        let (major, minor, update) = crate::nfc_tables::UNICODE_VERSION;
        assert_eq!(version, format!("{major}.{minor}.{update}"));
        assert_eq!((lines, distinct.len()), (19_965, 19_952));
        for c in (0..=0x10FFFFu32).filter_map(char::from_u32) {
            if !listed.contains(&c) {
                assert_eq!(nfc(&c.to_string()), c.to_string(), "{:04X}", c as u32);
            }
        }
    }

    /// The results of Unicode 15.1.0, which the 15.1.0 tables gave on every
    /// line before this crate moved to 16.0.0, are kept: every data line of
    /// NormalizationTest 15.1.0 passes with the current tables, and each of
    /// them is also a line of NormalizationTest 16.0.0, unchanged.
    #[test]
    fn every_line_of_normalization_test_15_1_gives_the_same_result_with_the_current_tables() {
        let (version, lines, old, _) = run_normalization_test(NORMALIZATION_TEST_15_1);
        assert_eq!(version, "15.1.0");
        assert_eq!((lines, old.len()), (19_074, 19_069));
        let (_, _, new, _) = run_normalization_test(NORMALIZATION_TEST_16_0);
        assert_eq!(old.difference(&new).count(), 0);
        assert_eq!(new.difference(&old).count(), 883);
    }

    /// The SHA-256 of NFC of every scalar value, each followed by the byte 0xFF,
    /// computed with unicode-normalization 0.1.25 on 2026-10-17 before that
    /// crate left the tree; ICU4X icu_normalizer 2.3.0 gives the same bytes.
    #[test]
    fn every_scalar_value_gives_the_same_bytes_as_the_earlier_implementation() {
        let mut joined = Vec::new();
        for c in (0..=0x10FFFFu32).filter_map(char::from_u32) {
            let once = nfc(&c.to_string());
            assert_eq!(
                nfc(&once),
                once,
                "NFC is not idempotent at {:04X}",
                c as u32
            );
            joined.extend_from_slice(once.as_bytes());
            joined.push(0xFF);
        }
        assert_eq!(
            hex(&digest(&joined)),
            "de0a2980f40c3e6771c8a205fcc7f21571cec94a5ffe7ab62fee0cc2599fdfde"
        );
    }

    /// Whether `c` is assigned in Unicode 15.1.0, read from its UnicodeData.txt.
    fn assigned_in_15_1() -> impl Fn(char) -> bool {
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        let mut first = None;
        for line in include_str!("../tests/data/ucd-15.1.0/UnicodeData.txt").lines() {
            let mut f = line.split(';');
            let cp = u32::from_str_radix(f.next().unwrap(), 16).unwrap();
            let name = f.next().unwrap();
            if name.ends_with(", First>") {
                first = Some(cp);
            } else if name.ends_with(", Last>") {
                ranges.push((first.take().unwrap(), cp));
            } else {
                ranges.push((cp, cp));
            }
        }
        assert!(ranges.windows(2).all(|w| w[0].1 < w[1].0));
        move |c| {
            let c = c as u32;
            let at = ranges.partition_point(|r| r.1 < c);
            ranges.get(at).is_some_and(|r| r.0 <= c)
        }
    }

    /// Every code point assigned in Unicode 15.1.0 with a combining class or a
    /// canonical decomposition, the Hangul jamo and three syllables, and a to
    /// z. Code points assigned later are left out so that the pins below,
    /// computed with earlier implementations on this pool, still apply;
    /// NormalizationTest 16.0.0 and every scalar value cover them.
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
        let assigned = assigned_in_15_1();
        let pool: Vec<char> = pool
            .into_iter()
            .filter_map(char::from_u32)
            .filter(|&c| assigned(c))
            .collect();
        assert_eq!(pool.len(), 4291);
        pool
    }

    /// SplitMix64 from `seed`.
    fn splitmix(mut state: u64) -> impl FnMut() -> u64 {
        move || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
    }

    /// 100,000 seeded strings of 1 to 8 characters drawn from the pool; the pin
    /// was computed with unicode-normalization 0.1.25 on 2026-10-17, and ICU4X
    /// icu_normalizer 2.3.0 gives the same bytes.
    #[test]
    fn random_mixtures_give_the_same_bytes_as_the_earlier_implementation() {
        let pool = pool();
        let mut next = splitmix(20_261_017);
        let mut joined = Vec::new();
        let mut previous = (String::new(), String::new());
        for _ in 0..100_000 {
            let len = 1 + (next() % 8) as usize;
            let s: String = (0..len)
                .map(|_| pool[(next() % pool.len() as u64) as usize])
                .collect();
            let once = nfc(&s);
            // Idempotent, and a line break splits normalization: search
            // normalizes each line on its own.
            assert_eq!(nfc(&once), once, "{s:?}");
            assert_eq!(
                nfc(&format!("{}\n{s}", previous.0)),
                format!("{}\n{once}", previous.1),
                "{s:?}"
            );
            joined.extend_from_slice(once.as_bytes());
            joined.push(0xFF);
            previous = (s, once);
        }
        assert_eq!(
            hex(&digest(&joined)),
            "aae6bd9d024213096cd9ff09ece256d27e526380541c84c7e57575af04649832"
        );
    }

    /// Five long lines (the worst and best order of 2047 combining marks, Polish
    /// text, decomposed and composed Hangul) and 10,000 seeded strings of 1 to 64
    /// characters, two thirds of them combining marks assigned in Unicode
    /// 15.1.0, the rest from the pool. The pin was computed on
    /// 2026-10-17 with ICU4X icu_normalizer 2.3.0, an independent
    /// implementation.
    #[test]
    fn adversarial_strings_give_the_same_bytes_as_an_independent_implementation() {
        let pool = pool();
        let assigned = assigned_in_15_1();
        let marks: Vec<char> = CANONICAL_COMBINING_CLASS
            .iter()
            .flat_map(|&(a, b, _)| a..=b)
            .filter_map(char::from_u32)
            .filter(|&c| assigned(c))
            .collect();
        assert_eq!(marks.len(), 922);
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
        let mut joined = Vec::new();
        for s in &strings {
            let once = nfc(s);
            assert_eq!(nfc(&once), once, "{s:?}");
            joined.extend_from_slice(once.as_bytes());
            joined.push(0xFF);
        }
        assert_eq!(
            hex(&digest(&joined)),
            "5bf7396d264d8230559cdd002a85997e417bb7e276d7dc39f26f09f2e390e3fe"
        );
    }

    #[test]
    fn hangul_and_a_blocked_mark_compose_as_the_standard_says() {
        // Syllable from L + V + T, and an acute that a dot below does not block.
        assert_eq!(nfc("\u{1100}\u{1161}\u{11A8}"), "\u{AC01}");
        assert_eq!(nfc("e\u{0323}\u{0301}"), "\u{1EB9}\u{0301}");
        assert_eq!(nfc("e\u{0301}\u{0323}"), "\u{1EB9}\u{0301}");
        assert_eq!(nfc("cafe\u{0301}"), "café");
    }
}
