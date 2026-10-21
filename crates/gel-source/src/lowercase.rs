//! Lowercasing of a document line for search: each character as
//! `char::to_lowercase` gives it, with both sigma forms written as 'σ'.
//!
//! The result equals the per-character loop it replaces, character for
//! character; only the work per character is smaller. Runs of ASCII bytes are
//! copied at once and lowercased in place. Characters below `TABLE_END` are
//! looked up in a table that is filled once from `char::to_lowercase` itself,
//! so it holds the same Unicode case data as the standard library in use.
//! Characters in `CASELESS` lowercase to themselves and are copied. Every other
//! character, up to U+10FFFF, is looked up in the table of its block of
//! `BLOCK_LEN` code points, which is filled the same way the first time a
//! character of that block is seen. A character whose lowercase form has more
//! than one character goes through `char::to_lowercase` as before. The
//! comparison of every scalar value with the earlier loop is in
//! `examples/identity_lowercase.rs`.
//!
//! This file uses only the standard library, so that the identity program can
//! include it as it is.
use std::sync::OnceLock;

/// Characters below this code point are lowercased through the table.
const TABLE_END: usize = 0x0800;

/// Ranges in which `char::to_lowercase` returns the character itself: Hangul
/// Jamo, CJK punctuation with Hiragana and Katakana, CJK Unified Ideographs
/// Extension A, CJK Unified Ideographs and Hangul Syllables.
pub(crate) const CASELESS: [(u32, u32); 5] = [
    (0x1100, 0x11FF),
    (0x3000, 0x30FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xAC00, 0xD7A3),
];

/// The highest code point of the `CASELESS` ranges, which are in ascending
/// order; above it no range needs to be tested.
const CASELESS_LAST: u32 = CASELESS[CASELESS.len() - 1].1;

/// Code points per block table.
const BLOCK_LEN: usize = 0x100;

/// Block tables needed to cover every code point up to U+10FFFF.
const BLOCK_COUNT: usize = 0x11_0000 / BLOCK_LEN;

/// The lowercase forms of the code points of one block.
type Block = [Option<char>; BLOCK_LEN];

/// The lowercase form of one character when it is a single character, with
/// 'Σ' and 'ς' both giving 'σ'; None when it has more than one character.
fn single_lowercase(c: char) -> Option<char> {
    if c == 'Σ' || c == 'ς' {
        return Some('σ');
    }
    let mut lower = c.to_lowercase();
    if lower.len() == 1 {
        lower.next()
    } else {
        None
    }
}

fn table() -> &'static [Option<char>; TABLE_END] {
    static TABLE: OnceLock<[Option<char>; TABLE_END]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [None; TABLE_END];
        for (code, slot) in (0u32..).zip(table.iter_mut()) {
            // No surrogate lies below TABLE_END, so every code is a char.
            *slot = char::from_u32(code).and_then(single_lowercase);
        }
        table
    })
}

/// The table of block `index` (below `BLOCK_COUNT`), filled on first use
/// from `single_lowercase`. Surrogate code points, which are no char, get
/// None; no char looks them up.
fn block(index: usize) -> &'static Block {
    static BLOCKS: [OnceLock<Box<Block>>; BLOCK_COUNT] = [const { OnceLock::new() }; BLOCK_COUNT];
    BLOCKS[index].get_or_init(|| {
        let mut entries: Box<Block> = Box::new([None; BLOCK_LEN]);
        let first = (index * BLOCK_LEN) as u32;
        for (code, slot) in (first..).zip(entries.iter_mut()) {
            *slot = char::from_u32(code).and_then(single_lowercase);
        }
        entries
    })
}

/// True when `c` lies in one of the `CASELESS` ranges.
fn caseless(c: u32) -> bool {
    CASELESS
        .iter()
        .any(|&(first, last)| first <= c && c <= last)
}

/// Appends the lowercase form of `c` that a table gives as `entry`.
#[inline]
fn push_lower(entry: Option<char>, c: char, out: &mut String) {
    match entry {
        Some(lower) => out.push(lower),
        None => out.extend(c.to_lowercase()),
    }
}

/// Appends `s` lowercased to `out`; 'Σ' and 'ς' are both appended as 'σ'.
pub(crate) fn lowercase_into(s: &str, out: &mut String) {
    let table = table();
    // The block table used last in this call, with its index, so that the
    // characters of one script do not reach the block tables again.
    let mut last: Option<(usize, &'static Block)> = None;
    let mut rest = s;
    while !rest.is_empty() {
        let ascii = rest
            .bytes()
            .position(|b| !b.is_ascii())
            .unwrap_or(rest.len());
        if ascii > 0 {
            let at = out.len();
            out.push_str(&rest[..ascii]);
            out[at..].make_ascii_lowercase();
            rest = &rest[ascii..];
        }
        let mut end = rest.len();
        for (at, c) in rest.char_indices() {
            if c.is_ascii() {
                end = at;
                break;
            }
            let code = c as u32;
            if let Some(&entry) = table.get(code as usize) {
                push_lower(entry, c, out);
            } else if code <= CASELESS_LAST && caseless(code) {
                out.push(c);
            } else {
                let index = code as usize / BLOCK_LEN;
                let entries = match last {
                    Some((held, entries)) if held == index => entries,
                    _ => {
                        let entries = block(index);
                        last = Some((index, entries));
                        entries
                    }
                };
                push_lower(entries[code as usize % BLOCK_LEN], c, out);
            }
        }
        rest = &rest[end..];
    }
}
