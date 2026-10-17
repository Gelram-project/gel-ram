//! Unicode Normalization Form C (UAX #15), written for this crate.
//!
//! A string is decomposed canonically (Hangul syllables by arithmetic, other
//! characters by the recursive canonical decomposition mappings), its
//! combining marks are put in canonical order, and it is recomposed: each
//! starter is combined with the next unblocked character that forms a primary
//! composite with it. The tables in `nfc_tables.rs` come from the Unicode
//! Character Database; the tests run the Unicode NormalizationTest of the same
//! version and compare every code point with an earlier implementation.
use crate::nfc_tables::{CANONICAL_COMBINING_CLASS, CANONICAL_DECOMPOSITION, PRIMARY_COMPOSITE};

const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

fn combining_class(c: u32) -> u8 {
    match CANONICAL_COMBINING_CLASS.binary_search_by(|&(first, last, _)| {
        if last < c {
            std::cmp::Ordering::Less
        } else if first > c {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }) {
        Ok(i) => CANONICAL_COMBINING_CLASS[i].2,
        Err(_) => 0,
    }
}

fn decompose(c: u32, out: &mut Vec<u32>) {
    if (S_BASE..S_BASE + S_COUNT).contains(&c) {
        let s = c - S_BASE;
        out.push(L_BASE + s / N_COUNT);
        out.push(V_BASE + (s % N_COUNT) / T_COUNT);
        if s % T_COUNT != 0 {
            out.push(T_BASE + s % T_COUNT);
        }
        return;
    }
    match CANONICAL_DECOMPOSITION.binary_search_by_key(&c, |&(cp, _, _)| cp) {
        Ok(i) => {
            let (_, first, second) = CANONICAL_DECOMPOSITION[i];
            decompose(first, out);
            if second != 0 {
                decompose(second, out);
            }
        }
        Err(_) => out.push(c),
    }
}

fn compose(first: u32, second: u32) -> Option<u32> {
    if (L_BASE..L_BASE + L_COUNT).contains(&first) && (V_BASE..V_BASE + V_COUNT).contains(&second) {
        return Some(S_BASE + ((first - L_BASE) * V_COUNT + (second - V_BASE)) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&first)
        && (first - S_BASE) % T_COUNT == 0
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&second)
    {
        return Some(first + (second - T_BASE));
    }
    PRIMARY_COMPOSITE
        .binary_search_by_key(&(first, second), |&(a, b, _)| (a, b))
        .ok()
        .map(|i| PRIMARY_COMPOSITE[i].2)
}

/// `s` in Normalization Form C.
pub(crate) fn nfc(s: &str) -> String {
    let mut decomposed: Vec<u32> = Vec::with_capacity(s.len());
    for c in s.chars() {
        decompose(c as u32, &mut decomposed);
    }
    // Canonical ordering: a stable sort of each run of nonzero classes.
    for i in 1..decomposed.len() {
        let class = combining_class(decomposed[i]);
        if class == 0 {
            continue;
        }
        let mut j = i;
        while j > 0 && combining_class(decomposed[j - 1]) > class {
            decomposed.swap(j - 1, j);
            j -= 1;
        }
    }
    // Canonical composition. `last` is the class of the last character kept
    // after the current starter; None means the next character is adjacent.
    let mut out: Vec<u32> = Vec::with_capacity(decomposed.len());
    let mut starter: Option<usize> = None;
    let mut last: Option<u8> = None;
    for c in decomposed {
        let class = combining_class(c);
        if let Some(at) = starter {
            let blocked = last.is_some_and(|l| l >= class);
            if !blocked {
                if let Some(composite) = compose(out[at], c) {
                    out[at] = composite;
                    continue;
                }
            }
        }
        if class == 0 {
            starter = Some(out.len());
            last = None;
        } else if starter.is_some() {
            last = Some(class);
        }
        out.push(c);
    }
    out.into_iter().filter_map(char::from_u32).collect()
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
    fn the_tables_are_unicode_15_1_with_their_published_counts() {
        assert_eq!(crate::nfc_tables::UNICODE_VERSION, (15, 1, 0));
        assert_eq!(CANONICAL_DECOMPOSITION.len(), 2061);
        assert_eq!(PRIMARY_COMPOSITE.len(), 941);
        assert!(CANONICAL_DECOMPOSITION.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(PRIMARY_COMPOSITE
            .windows(2)
            .all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)));
        assert!(CANONICAL_COMBINING_CLASS
            .windows(2)
            .all(|w| w[0].1 < w[1].0 && w[0].0 <= w[0].1));
    }

    #[test]
    fn the_unicode_normalization_test_of_the_same_version_passes() {
        let text = include_str!("../tests/data/NormalizationTest-15.1.0.txt");
        let mut listed = std::collections::BTreeSet::new();
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
            lines += 1;
        }
        assert_eq!(lines, 19_074);
        for c in (0..=0x10FFFFu32).filter_map(char::from_u32) {
            if !listed.contains(&c) {
                assert_eq!(nfc(&c.to_string()), c.to_string(), "{:04X}", c as u32);
            }
        }
    }

    /// The SHA-256 of NFC of every scalar value, each followed by the byte 0xFF,
    /// computed with unicode-normalization 0.1.25 on 2026-10-17 before that
    /// crate left the tree.
    #[test]
    fn every_scalar_value_gives_the_same_bytes_as_the_earlier_implementation() {
        let mut joined = Vec::new();
        for c in (0..=0x10FFFFu32).filter_map(char::from_u32) {
            joined.extend_from_slice(nfc(&c.to_string()).as_bytes());
            joined.push(0xFF);
        }
        assert_eq!(
            hex(&digest(&joined)),
            "de0a2980f40c3e6771c8a205fcc7f21571cec94a5ffe7ab62fee0cc2599fdfde"
        );
    }

    /// 100,000 seeded strings of 1 to 8 characters drawn from every code point
    /// with a combining class or a canonical decomposition, the Hangul jamo and
    /// three syllables, and a to z; the pin was computed with
    /// unicode-normalization 0.1.25 on 2026-10-17.
    #[test]
    fn random_mixtures_give_the_same_bytes_as_the_earlier_implementation() {
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
        let pool: Vec<char> = pool.into_iter().filter_map(char::from_u32).collect();
        assert_eq!(pool.len(), 4291);
        let mut state = 20_261_017u64;
        let mut next = || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };
        let mut joined = Vec::new();
        for _ in 0..100_000 {
            let len = 1 + (next() % 8) as usize;
            let s: String = (0..len)
                .map(|_| pool[(next() % pool.len() as u64) as usize])
                .collect();
            joined.extend_from_slice(nfc(&s).as_bytes());
            joined.push(0xFF);
        }
        assert_eq!(
            hex(&digest(&joined)),
            "aae6bd9d024213096cd9ff09ece256d27e526380541c84c7e57575af04649832"
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
