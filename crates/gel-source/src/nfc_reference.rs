//! The earlier NFC implementation, unchanged except for the visibility of its
//! functions: every property was found by a binary search in the tables, and
//! each normalized string was built in fresh vectors. The identity check
//! (`examples/identity_nfc_fast.rs` and its unit test) compares the current
//! `nfc` module with it.
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
const JAMO_BLOCK: std::ops::RangeInclusive<u32> = 0x1100..=0x11FF;

#[cfg(test)]
thread_local! {
    /// Combining-class lookups made by this thread, for the cost test.
    static LOOKUPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// The value of the range of `table` that contains `c`.
pub(crate) fn in_range<T: Copy>(table: &[(u32, u32, T)], c: u32) -> Option<T> {
    table
        .binary_search_by(|&(first, last, _)| {
            if last < c {
                std::cmp::Ordering::Less
            } else if first > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .ok()
        .map(|i| table[i].2)
}

pub(crate) fn combining_class(c: u32) -> u8 {
    #[cfg(test)]
    LOOKUPS.with(|n| n.set(n.get() + 1));
    in_range(CANONICAL_COMBINING_CLASS, c).unwrap_or(0)
}

/// True when `s` is certainly in NFC by the NFC_Quick_Check rule of UAX #15:
/// no character has NFC_Quick_Check No or Maybe, and the nonzero combining
/// classes never decrease.
pub(crate) fn is_nfc_by_quick_check(s: &str) -> bool {
    let mut last = 0;
    for c in s.chars() {
        let c = c as u32;
        if c < FIRST_NOT_TRIVIAL {
            last = 0;
            continue;
        }
        if in_range(NFC_QUICK_CHECK, c).is_some() {
            return false;
        }
        let class = combining_class(c);
        if class != 0 && last > class {
            return false;
        }
        last = class;
    }
    true
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
    // The Hangul Jamo block has no decompositions and no nonzero classes, and
    // nothing below the first decomposition decomposes (both checked by a test).
    if JAMO_BLOCK.contains(&c) {
        out.push((0, c));
        return;
    }
    if c < CANONICAL_DECOMPOSITION[0].0 {
        out.push((combining_class(c), c));
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
        Err(_) => out.push((combining_class(c), c)),
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
    PRIMARY_COMPOSITE
        .binary_search_by_key(&(first, second), |&(a, b, _)| (a, b))
        .ok()
        .map(|i| PRIMARY_COMPOSITE[i].2)
}

/// `s` in Normalization Form C.
pub(crate) fn nfc(s: &str) -> String {
    nfc_cow(s).into_owned()
}

/// `s` in Normalization Form C, borrowed when `s` already passes the quick check.
pub(crate) fn nfc_cow(s: &str) -> std::borrow::Cow<'_, str> {
    if is_nfc_by_quick_check(s) {
        std::borrow::Cow::Borrowed(s)
    } else {
        std::borrow::Cow::Owned(normalize(s))
    }
}

/// Decomposition, canonical ordering and composition of a string that did not
/// pass the quick check.
pub(crate) fn normalize(s: &str) -> String {
    let mut decomposed: Vec<(u8, u32)> = Vec::with_capacity(s.len());
    for c in s.chars() {
        decompose(c as u32, &mut decomposed);
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
    // Canonical composition. `last` is the class of the last character kept
    // after the current starter; None means the next character is adjacent.
    let mut out: Vec<u32> = Vec::with_capacity(decomposed.len());
    let mut starter: Option<usize> = None;
    let mut last: Option<u8> = None;
    for (class, c) in decomposed {
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
