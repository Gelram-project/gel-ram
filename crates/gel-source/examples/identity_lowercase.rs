//! Identity check for the lowercasing of non-ASCII document lines:
//! `lowercase_into` (from `src/lowercase.rs`) against an exact copy of the
//! earlier per-character loop on every Unicode scalar value and on seeded
//! lines, and the public `document::search` against a copy of the v0.6.1
//! search on seeded multilingual texts (Polish, Greek with both sigma forms,
//! Turkish dotted and dotless i, German sharp s, Cyrillic, Hangul, CJK, kana
//! and others), compared on passages, counts and errors. Further cases cover
//! the block tables above U+0800: lines from eight threads that fill the
//! tables at the same time (run first, before any table is filled), every
//! scalar of seven cased supplementary scripts (Deseret, Osage, Vithkuqi, Old
//! Hungarian, Warang Citi, Medefaidrin, Adlam) alone and in lines, seeded
//! lines that mix BMP and supplementary text, and the first and last scalar
//! of every block with their neighbours across the block edge.
//!
//! Usage: `cargo run --release -p gel-source --example identity_lowercase`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when a case
//! made no comparison or when no case ran); exit 0 on PASS, 1 on FAIL. The
//! integration test `tests/identity_lowercase.rs` runs the same comparison at
//! a smaller scale. Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::document::{self, MAX_LINE, MAX_QUERY, MAX_TEXT};

#[allow(dead_code)]
#[path = "../src/lowercase.rs"]
mod lowercase;

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// Exact copies of the earlier code.
mod reference {
    use gel_source::document::{Search, MAX_LINE, MAX_PASSAGES, MAX_QUERY, MAX_TEXT};

    /// The loop of the earlier `normalize_line_into`, after its NFC step.
    pub fn lowercase_loop(s: &str, out: &mut String) {
        for c in s.chars() {
            if c == 'Σ' || c == 'ς' {
                out.push('σ');
            } else {
                out.extend(c.to_lowercase());
            }
        }
    }

    /// The lowercasing of v0.6.1 `normalized_words_text`, after its NFC step.
    pub fn lowercase_str(s: &str) -> String {
        s.to_lowercase().replace('ς', "σ")
    }

    fn words(s: &str) -> Vec<String> {
        lowercase_str(s)
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn phrase_in_tokens<'a>(
        tokens: impl Iterator<Item = &'a str>,
        query: &[String],
        ascii: bool,
    ) -> bool {
        let tokens: Vec<&str> = tokens.collect();
        tokens.windows(query.len()).any(|w| {
            w.iter().zip(query).all(|(got, wanted)| {
                if ascii {
                    got.eq_ignore_ascii_case(wanted)
                } else {
                    got == wanted
                }
            })
        })
    }

    /// The search of v0.6.1 for texts and phrases that are already in NFC.
    pub fn search(text: &str, phrase: &str) -> Result<Search, &'static str> {
        if text.len() > MAX_TEXT || phrase.len() > MAX_QUERY {
            return Err("DOCUMENT_SEARCH_LIMIT");
        }
        if phrase.chars().any(char::is_control) {
            return Err("DOCUMENT_QUERY_CONTROL");
        }
        let query = words(phrase);
        if query.is_empty() || query.len() > 32 {
            return Err("DOCUMENT_QUERY_WORDS");
        }
        let mut result = Search {
            passages: Vec::new(),
            matching_lines: 0,
            skipped_long_lines: 0,
        };
        let mut offset = 0;
        for raw in text.split_inclusive(['\r', '\n']) {
            let line = raw.trim_end_matches(['\r', '\n']);
            if line.len() > MAX_LINE {
                result.skipped_long_lines += 1;
            } else {
                let found = if line.is_ascii() {
                    phrase_in_tokens(
                        line.split(|c: char| !c.is_ascii_alphanumeric())
                            .filter(|s| !s.is_empty()),
                        &query,
                        true,
                    )
                } else {
                    let normalized = lowercase_str(line);
                    phrase_in_tokens(
                        normalized
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|s| !s.is_empty()),
                        &query,
                        false,
                    )
                };
                if found {
                    result.matching_lines += 1;
                    if result.passages.len() < MAX_PASSAGES {
                        result.passages.push(offset..offset + line.len());
                    }
                }
            }
            offset += raw.len();
        }
        Ok(result)
    }
}

// How many inputs each case compares: `Scale::Full` from `main`,
// `Scale::Small` from the test wrapper.

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

/// Words made only of characters that are their own NFC (Quick_Check Yes,
/// class 0, never the second character of a composition), so that a text
/// built from them and the gaps below is already in NFC.
const NFC_WORDS: [&str; 64] = [
    "Ada",
    "ADA",
    "not",
    "approved",
    "x123",
    "ŁÓDŹ",
    "łódź",
    "Zażółć",
    "gęślą",
    "JAŹŃ",
    "Źródło",
    "ŻÓŁW",
    "ΟΣ",
    "ος",
    "οσ",
    "ΣΟΦΟΣ",
    "σοφός",
    "Ἀθῆναι",
    "ΆΈΉΊΌΎΏ",
    "ΐΰ",
    "İstanbul",
    "istanbul",
    "ISTANBUL",
    "ıspanak",
    "IŞIK",
    "ışık",
    "İ",
    "ı",
    "I",
    "i",
    "Straße",
    "STRASSE",
    "ẞ",
    "ß",
    "Größe",
    "Москва",
    "МОСКВА",
    "ЁЖИК",
    "ёжик",
    "Հայաստան",
    "한국어",
    "\u{1100}\u{1100}",
    "中文",
    "日本語",
    "ひらがな",
    "カタカナ",
    "が",
    "ﾊﾝｶｸ",
    "ＡＢＣ",
    "ａｂｃ",
    "Ⅻ",
    "ⅻ",
    "ǅ",
    "ǆ",
    "ſ",
    "Ꭰ",
    "Ა",
    "ა",
    "Tiếng",
    "VIỆT",
    "𐐀𐐨",
    "𐐨𐐀",
    "😀",
    "42",
];
const GAPS: [&str; 7] = [" ", ", ", " — ", "-", "  ", "'", "·"];
const ENDS: [&str; 6] = ["\n", "\r\n", "\r", "\n\n", "\r\r", ""];

/// Pieces for the lowercasing comparison; any text is allowed here.
const LINE_PIECES: [&str; 26] = [
    "Σ",
    "ς",
    "σ",
    "ΣΑΣ ",
    "İ",
    "\u{0130}\u{0130}",
    "ı",
    "I",
    "ẞ",
    "ß",
    "\u{212A}",
    "\u{2126}",
    "a\u{301}",
    "e\u{0323}\u{0302}",
    "\u{1100}\u{1161}\u{11A8}",
    "\u{3099}",
    "Ǆ",
    "ǅ",
    "ﬀ",
    "ŉ",
    "Ⰰ",
    "Ꭰ",
    "\u{10C0}",
    "𞤀",
    "\u{0345}",
    "ΐ",
];

/// A random scalar value, most often from the lower planes.
fn scalar(rng: &mut Rng) -> char {
    loop {
        let limit = match rng.below(4) {
            0 => 0x80,
            1 => 0x800,
            2 => 0x1_0000,
            _ => 0x11_0000,
        };
        if let Some(c) = char::from_u32((rng.next() % limit) as u32) {
            return c;
        }
    }
}

fn random_line(rng: &mut Rng) -> String {
    let mut line = String::new();
    for _ in 0..rng.below(24) {
        match rng.below(6) {
            0 => {
                // An ASCII run of any length, mixed case, punctuation included.
                for _ in 0..rng.below(300) {
                    line.push(char::from(0x20 + rng.below(0x5f) as u8));
                }
            }
            1 => line.push_str(rng.pick(&LINE_PIECES)),
            2 => line.push(scalar(rng)),
            _ => line.push_str(rng.pick(&NFC_WORDS)),
        }
        if rng.below(3) == 0 {
            line.push_str(rng.pick(&GAPS));
        }
    }
    line
}

fn edge_lines() -> Vec<String> {
    let mut lines: Vec<String> = [
        "",
        "A",
        "a",
        "\u{7f}",
        "\u{80}",
        "\u{7ff}",
        "\u{800}",
        "Σ",
        "ς",
        "ΣΣ",
        "AΣ",
        "Σ.",
        "İ",
        "İİ",
        "Iİıi",
        "ẞß",
        "\u{10FFFF}",
        "\u{D7FF}\u{E000}",
        "\u{AC00}\u{D7A3}\u{D7A4}",
        "\u{10FF}\u{1100}\u{11FF}\u{1200}",
        "\u{2FFF}\u{3000}\u{30FF}\u{3100}",
        "\u{33FF}\u{3400}\u{4DBF}\u{4DC0}",
        "\u{4DFF}\u{4E00}\u{9FFF}\u{A000}",
        "\u{ABFF}\u{AC00}",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for n in [1, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, MAX_LINE] {
        lines.push("AbC".repeat(n));
        lines.push(format!("{}Ł", "Q".repeat(n)));
        lines.push(format!("Ł{}", "Q".repeat(n)));
        lines.push(format!("Σ{}ς", "Ź".repeat(n)));
    }
    lines
}

fn lines_cases(scale: Scale, loop_ref: &mut Tally, str_ref: &mut Tally) {
    let mut rng = Rng(0x10_c4a5e);
    let count = match scale {
        Scale::Small => 3_000,
        Scale::Full => 200_000,
    };
    let mut lines = edge_lines();
    lines.extend((0..count).map(|_| random_line(&mut rng)));
    for (k, line) in lines.iter().enumerate() {
        // The function appends: start from a prefix that must stay untouched.
        let prefix = if k % 3 == 0 { "" } else { "PREFIX Σ " };
        let mut got = String::from(prefix);
        lowercase::lowercase_into(line, &mut got);
        let mut want = String::from(prefix);
        reference::lowercase_loop(line, &mut want);
        loop_ref.check(got == want);
        str_ref.check(got[prefix.len()..] == reference::lowercase_str(line));
    }
}

fn every_scalar(tally: &mut Tally) {
    let mut line = String::new();
    let mut got = String::new();
    let mut want = String::new();
    for c in (0..=0x10_FFFF_u32).filter_map(char::from_u32) {
        line.clear();
        line.push(c);
        got.clear();
        want.clear();
        lowercase::lowercase_into(&line, &mut got);
        reference::lowercase_loop(&line, &mut want);
        tally.check(got == want);
    }
}

/// A multilingual text of NFC words, with every line ending and lines around
/// `MAX_LINE`; returns the text and the word sequences of its lines.
fn nfc_text(rng: &mut Rng, lines: usize) -> (String, Vec<Vec<&'static str>>) {
    let mut text = String::new();
    let mut used = Vec::new();
    for _ in 0..lines {
        match rng.below(20) {
            0 => {
                // MAX_LINE - 1, MAX_LINE or MAX_LINE + 1 bytes, non-ASCII.
                let len = MAX_LINE - 1 + rng.below(3);
                let head = rng.pick(&["łódź ος ", "İstanbul ", "Straße "]);
                let mut line = String::from(head);
                while line.len() + 2 <= len {
                    line.push('ż');
                }
                while line.len() < len {
                    line.push('x');
                }
                text.push_str(&line);
            }
            1 => text.push_str("Ada not approved"),
            _ => {
                let mut words = Vec::new();
                for w in 0..1 + rng.below(10) {
                    if w > 0 {
                        text.push_str(rng.pick(&GAPS));
                    }
                    let word = rng.pick(&NFC_WORDS);
                    text.push_str(word);
                    words.push(word);
                }
                used.push(words);
            }
        }
        text.push_str(rng.pick(&ENDS));
    }
    (text, used)
}

fn queries(rng: &mut Rng, used: &[Vec<&'static str>]) -> Vec<String> {
    let mut out: Vec<String> = [
        "ος",
        "ΟΣ σοφός",
        "istanbul",
        "İSTANBUL",
        "ı",
        "i",
        "straße",
        "ẞ",
        "łódź",
        "ŻÓŁW",
        "한국어",
        "ǆ",
        "missing token",
        "ada not",
        "",
        "!!!",
        "a\nb",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for _ in 0..6 {
        if used.is_empty() {
            break;
        }
        let words = &used[rng.below(used.len())];
        let start = rng.below(words.len());
        let len = 1 + rng.below((words.len() - start).min(4));
        out.push(words[start..start + len].join(" "));
    }
    for _ in 0..2 {
        let n = 1 + rng.below(3);
        let words: Vec<&str> = (0..n).map(|_| rng.pick(&NFC_WORDS)).collect();
        out.push(words.join(" "));
    }
    out
}

fn search_cases(scale: Scale, tally: &mut Tally) {
    let mut rng = Rng(0x5ea_4c4);
    let (texts, big) = match scale {
        Scale::Small => (150, 0),
        Scale::Full => (3_000, 3),
    };
    // Comparisons whose result has a hit on a non-ASCII line, reported on
    // stderr so that a run of only empty results can be seen.
    let mut non_ascii_hits = 0u64;
    let mut compare = |text: &str, phrase: &str| {
        let got = document::search(text, phrase);
        if let Ok(found) = &got {
            if found.passages.iter().any(|p| !text[p.clone()].is_ascii()) {
                non_ascii_hits += 1;
            }
        }
        tally.check(got == reference::search(text, phrase));
    };
    compare("", "ada");
    for case in 0..texts {
        let (text, used) = nfc_text(&mut rng, 1 + case % 48);
        for phrase in queries(&mut rng, &used) {
            compare(&text, &phrase);
        }
    }
    // Texts above the parallel threshold, so that the search runs in pieces.
    for _ in 0..big {
        let mut text = String::new();
        let mut used = Vec::new();
        while text.len() < 3 << 20 {
            let (part, words) = nfc_text(&mut rng, 64);
            text.push_str(&part);
            used.extend(words);
        }
        for phrase in queries(&mut rng, &used) {
            compare(&text, &phrase);
        }
    }
    // Limits and their order.
    compare("ok", &"ż".repeat(MAX_QUERY / 2 + 1));
    compare("ok", &"ż ".repeat(33));
    compare("ok", &"ż ".repeat(32));
    if scale == Scale::Full {
        let mut over = "ż".repeat(MAX_TEXT / 2);
        compare(&over, "ż");
        over.push('x');
        compare(&over, "ż");
        compare(&over, "a\nb");
        compare(&over, &"x".repeat(MAX_QUERY + 1));
    }
    eprintln!("search comparisons with a non-ASCII hit: {non_ascii_hits}");
}

/// Code point ranges of seven cased scripts outside the BMP: Deseret, Osage,
/// Vithkuqi, Old Hungarian, Warang Citi, Medefaidrin and Adlam (unassigned
/// code points inside a range included).
const ASTRAL_SCRIPTS: [(u32, u32); 7] = [
    (0x10400, 0x1044F),
    (0x104B0, 0x104FF),
    (0x10570, 0x105BF),
    (0x10C80, 0x10CFF),
    (0x118A0, 0x118FF),
    (0x16E40, 0x16E9F),
    (0x1E900, 0x1E95F),
];

/// Words of those scripts in capital, small and mixed letters.
const ASTRAL_WORDS: [&str; 16] = [
    "𐐀𐐨𐐮",
    "𐐔𐐯𐑅",
    "\u{10400}\u{10401}\u{10427}",
    "\u{104B0}\u{104D9}\u{104DA}",
    "\u{104C4}\u{104C5}\u{104D3}",
    "\u{10570}\u{10598}\u{10599}",
    "\u{1057C}\u{1058C}\u{10594}\u{10595}",
    "\u{10C80}\u{10CC1}\u{10CC2}",
    "\u{10CB2}\u{10CF2}",
    "\u{118A0}\u{118C1}\u{118C2}",
    "\u{118BF}\u{118DF}",
    "\u{16E40}\u{16E61}\u{16E62}",
    "\u{16E5F}\u{16E7F}",
    "\u{1E900}\u{1E923}\u{1E924}",
    "\u{1E921}\u{1E943}",
    "\u{1E910}\u{1E931}",
];

/// BMP pieces that go with them in mixed lines: ASCII, table characters,
/// both sigma forms, CASELESS text, and cased BMP characters above U+0800.
const BMP_PIECES: [&str; 20] = [
    "Ada",
    "approved",
    "x123",
    "Zażółć",
    "ŻÓŁW",
    "ΣΟΦΟΣ",
    "ος",
    "İstanbul",
    "한국어",
    "中文",
    "カタカナ",
    "Straße",
    "ẞ",
    "Tiếng",
    "VIỆT",
    "Ⴀ",
    "Ꭰ",
    "Ⰰ",
    "ＡＢＣ",
    "😀",
];

/// Prefixes the output buffer starts with; they must stay untouched.
const PREFIXES: [&str; 3] = ["", "PREFIX Σ ", "𐐀 ẞ "];

/// `lowercase_into` appended to `prefix` equals the earlier loop, and its
/// appended part equals the lowercasing of v0.6.1.
fn matches_both(line: &str, prefix: &str) -> bool {
    let mut got = String::from(prefix);
    lowercase::lowercase_into(line, &mut got);
    let mut want = String::from(prefix);
    reference::lowercase_loop(line, &mut want);
    got == want && got[prefix.len()..] == reference::lowercase_str(line)
}

/// One line per block above `0x0800` with characters of that block and of
/// its neighbours, in the order that starts at block `start`.
fn block_lines(start: usize) -> Vec<String> {
    let blocks = 0x08..0x1100_usize;
    let count = blocks.len();
    (0..count)
        .map(|k| {
            let block = (blocks.start + (start + k * 7) % count) as u32;
            let mut line = String::new();
            for code in [
                block * 0x100,
                block * 0x100 + 0x41,
                block * 0x100 + 0xFF,
                (block + 1) * 0x100,
            ] {
                if let Some(c) = char::from_u32(code) {
                    line.push(c);
                    line.push('Ł');
                }
            }
            line
        })
        .collect()
}

/// Eight threads lowercase lines that reach every block table, each from a
/// different block, while the tables are still empty.
fn threads_first_touch(tally: &mut Tally) {
    let results: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|t| {
                scope.spawn(move || {
                    block_lines(t * 541)
                        .iter()
                        .map(|line| matches_both(line, PREFIXES[t % PREFIXES.len()]))
                        .collect::<Vec<bool>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().unwrap_or_default())
            .collect()
    });
    for result in results {
        // A thread that panicked returns no results: this comparison is then
        // unequal and the case fails.
        tally.check(result.len() == 0x1100 - 0x08);
        tally.check_all(result);
    }
}

fn astral_script_lines(tally: &mut Tally) {
    for &(first, last) in &ASTRAL_SCRIPTS {
        let chars: Vec<char> = (first..=last).filter_map(char::from_u32).collect();
        for (k, &c) in chars.iter().enumerate() {
            let prefix = PREFIXES[k % PREFIXES.len()];
            tally.check(matches_both(&c.to_string(), prefix));
            tally.check(matches_both(&format!("A{c}Ł{c}中{c}z"), prefix));
            // Blocks change from character to character.
            tally.check(matches_both(&format!("{c}ẞ{c}Ⴀ{c}😀{c}Σ"), prefix));
        }
        let line: String = chars.iter().collect();
        tally.check(matches_both(&line, ""));
        let line: String = chars.iter().rev().collect();
        tally.check(matches_both(&line, "PREFIX Σ "));
        let line: String = chars.iter().flat_map(|&c| [c, ' ']).collect();
        tally.check(matches_both(&line, ""));
    }
    for (k, word) in ASTRAL_WORDS.iter().enumerate() {
        let prefix = PREFIXES[k % PREFIXES.len()];
        for form in [word.to_string(), word.to_uppercase(), word.to_lowercase()] {
            tally.check(matches_both(&form, prefix));
            tally.check(matches_both(&format!("{form} {form}{form}"), prefix));
        }
    }
    let all: String = ASTRAL_WORDS.concat();
    tally.check(matches_both(&all, ""));
    tally.check(matches_both(&ASTRAL_WORDS.join(" "), ""));
}

fn bmp_astral_lines(scale: Scale, tally: &mut Tally) {
    let mut rng = Rng(0xa57_4a1);
    let count = match scale {
        Scale::Small => 2_000,
        Scale::Full => 100_000,
    };
    for k in 0..count {
        let mut line = String::new();
        for _ in 0..rng.below(32) {
            match rng.below(8) {
                0..=2 => line.push_str(rng.pick(&ASTRAL_WORDS)),
                3 | 4 => line.push_str(rng.pick(&BMP_PIECES)),
                5 => {
                    let (first, last) = ASTRAL_SCRIPTS[rng.below(ASTRAL_SCRIPTS.len())];
                    let code = first + rng.below((last - first + 1) as usize) as u32;
                    line.extend(char::from_u32(code));
                }
                6 => line.push(scalar(&mut rng)),
                _ => line.push_str(rng.pick(&LINE_PIECES)),
            }
            if rng.below(3) == 0 {
                line.push_str(rng.pick(&GAPS));
            }
        }
        tally.check(matches_both(&line, PREFIXES[k % PREFIXES.len()]));
    }
}

fn block_edges(tally: &mut Tally) {
    for block in 0x08..0x1100_u32 {
        let first = block * 0x100;
        let edge = [first - 1, first, first + 1, first + 0xFE, first + 0xFF];
        let chars: Vec<char> = edge.iter().copied().filter_map(char::from_u32).collect();
        for &c in &chars {
            tally.check(matches_both(&c.to_string(), ""));
        }
        let line: String = chars.iter().collect();
        tally.check(matches_both(&line, "PREFIX Σ "));
        let line: String = chars.iter().flat_map(|&c| ['x', c, 'ς']).collect();
        tally.check(matches_both(&line, "𐐀 ẞ "));
    }
    for codes in [
        [0x7FF, 0x800, 0x7FF],
        [0xD7A3, 0xD7A4, 0xD7A3],
        [0xFFFF, 0x10000, 0xFFFF],
        [0x10FFFF, 0x10FFFE, 0x10FFFF],
        [0xD7FF, 0xE000, 0x10400],
    ] {
        let line: String = codes.iter().copied().filter_map(char::from_u32).collect();
        tally.check(matches_both(&line, ""));
        tally.check(matches_both(&format!("Σ{line}A{line}ς"), "PREFIX Σ "));
    }
}

pub fn run(scale: Scale) -> Vec<Tally> {
    // Before any other case, so that the threads fill the block tables.
    let mut threads = Tally::new("threads_first_touch");
    threads_first_touch(&mut threads);
    let mut scalars = Tally::new("every_scalar");
    every_scalar(&mut scalars);
    let mut loop_ref = Tally::new("seeded_lines_loop_reference");
    let mut str_ref = Tally::new("seeded_lines_str_reference");
    lines_cases(scale, &mut loop_ref, &mut str_ref);
    let mut search = Tally::new("search_reference");
    search_cases(scale, &mut search);
    let mut astral = Tally::new("astral_script_lines");
    astral_script_lines(&mut astral);
    let mut mixed = Tally::new("bmp_astral_lines");
    bmp_astral_lines(scale, &mut mixed);
    let mut edges = Tally::new("block_edges");
    block_edges(&mut edges);
    vec![
        scalars, loop_ref, str_ref, search, threads, astral, mixed, edges,
    ]
}

fn main() -> std::process::ExitCode {
    identity::report(&run(Scale::Full))
}
