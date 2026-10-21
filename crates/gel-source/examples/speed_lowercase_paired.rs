//! The lowercasing of document lines alone, the earlier per-character loop
//! (`<script>_ref`) and `lowercase_into` from `src/lowercase.rs`
//! (`<script>_fast`), timed one after the other in the same binary on the
//! same lines. Needs `src/lowercase.rs`, so it does not build on an earlier
//! tree. Deterministic synthetic lines of 4 MiB per script, built outside the
//! timer: `polish`, `greek` (with both sigma forms), `cjk_hangul`, `mixed`
//! (Turkish dotted and dotless i, German sharp s, Cyrillic), `deseret` (only
//! cased characters U+10400..U+1044F, served by a block table),
//! `supplementary` (only cased characters of Osage, Adlam, Warang Citi,
//! Medefaidrin, Old Hungarian and Vithkuqi), `bmp_astral` (words of BMP
//! scripts and of supplementary scripts in one line) and `dotted_i` (only
//! U+0130, whose lowercase form has two characters: a control case where both
//! sides call the standard library for every character).
//!
//! Usage: `speed_lowercase_paired [RUNS]` (default 9, in 1..=999).
//!
//! Output (stdout): one `SPEED_SETUP example=speed_lowercase_paired runs=<R>
//! script_bytes=4194304` line, one `SPEED name=<case> run=<i> ns=<n>` line
//! per case and run, and a last `SPEED_CHECK sha256=<hex>` line over every
//! output line of run 0 and a fold of the outputs of every run. Before that
//! line the program compares both sides on every line and the folds of every
//! run; a difference prints `SPEED_FAIL reason=<slug>` on stderr and exits 1.
//! Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use std::{fmt::Write as _, hint::black_box, process::ExitCode, time::Instant};

#[allow(dead_code)]
#[path = "../src/lowercase.rs"]
mod lowercase;

#[path = "../../../tools/speed_support.rs"]
mod speed;

/// The loop of the earlier `normalize_line_into`, after its NFC step.
fn lowercase_ref(s: &str, out: &mut String) {
    for c in s.chars() {
        if c == 'Σ' || c == 'ς' {
            out.push('σ');
        } else {
            out.extend(c.to_lowercase());
        }
    }
}

const SCRIPT_BYTES: usize = 4 << 20;

const POLISH_WORDS: [&str; 16] = [
    "Zażółć",
    "gęślą",
    "jaźń",
    "Łódź",
    "źródło",
    "nie",
    "zatwierdził",
    "wniosku",
    "brama",
    "jest",
    "otwarta",
    "zapis",
    "wiersz",
    "tekstu",
    "ŻÓŁW",
    "42",
];
const GREEK_WORDS: [&str; 16] = [
    "Ο",
    "ΣΟΦΟΣ",
    "σοφός",
    "άνθρωπος",
    "λόγος",
    "ΛΟΓΟΣ",
    "είναι",
    "και",
    "της",
    "Αθήνα",
    "θάλασσα",
    "ΚΟΣΜΟΣ",
    "νύχτα",
    "ήλιος",
    "πόλις",
    "42",
];
const CJK_HANGUL_WORDS: [&str; 16] = [
    "한국어",
    "서울",
    "대한민국",
    "사람",
    "문장",
    "中文",
    "日本語",
    "東京",
    "漢字",
    "学校",
    "検索",
    "記録",
    "ひらがな",
    "カタカナ",
    "です",
    "42",
];
const MIXED_WORDS: [&str; 16] = [
    "İstanbul",
    "ışık",
    "IŞIK",
    "Türkçe",
    "çiçek",
    "Ankara",
    "Straße",
    "GRÖSSE",
    "ẞ",
    "Größe",
    "Mädchen",
    "Москва",
    "ПРИВЕТ",
    "ёжик",
    "Слово",
    "42",
];
const DESERET_WORDS: [&str; 8] = ["𐐀𐐨𐐮", "𐐔𐐯𐑅", "𐐊𐐯𐑊", "𐐝𐐲𐐻", "𐐢𐐨𐐯", "𐐄𐐮𐐻𐐆", "𐐎𐐯𐐍", "𐐐𐐒𐐓"];

const SUPPLEMENTARY_WORDS: [&str; 12] = [
    "\u{104B0}\u{104D9}\u{104DA}",
    "\u{104C4}\u{104C5}\u{104E0}",
    "\u{1E900}\u{1E923}\u{1E924}",
    "\u{1E910}\u{1E931}\u{1E932}",
    "\u{118A0}\u{118C1}\u{118C2}",
    "\u{118B5}\u{118D6}",
    "\u{16E40}\u{16E61}\u{16E62}",
    "\u{16E55}\u{16E76}",
    "\u{10C80}\u{10CC1}\u{10CC2}",
    "\u{10CA0}\u{10CE1}",
    "\u{10570}\u{10598}\u{10599}",
    "\u{1057C}\u{105A3}",
];
const BMP_ASTRAL_WORDS: [&str; 16] = [
    "Zażółć",
    "ΣΟΦΟΣ",
    "한국어",
    "𐐀𐐨𐐮",
    "\u{1E900}\u{1E923}",
    "Straße",
    "Москва",
    "ẞ",
    "Tiếng",
    "Ⴀⴀ",
    "Ꭰꭰ",
    "\u{10C80}\u{10CC1}",
    "日本語",
    "😀",
    "approved",
    "42",
];
const DOTTED_I_WORDS: [&str; 1] = ["İ"];

/// (name, words, gap between words).
const SCRIPTS: [(&str, &[&str], &str); 8] = [
    ("polish", &POLISH_WORDS, " "),
    ("greek", &GREEK_WORDS, " "),
    ("cjk_hangul", &CJK_HANGUL_WORDS, " "),
    ("mixed", &MIXED_WORDS, " "),
    ("deseret", &DESERET_WORDS, ""),
    ("supplementary", &SUPPLEMENTARY_WORDS, ""),
    ("bmp_astral", &BMP_ASTRAL_WORDS, " "),
    ("dotted_i", &DOTTED_I_WORDS, ""),
];

fn lines(seed: u64, words: &[&str], gap: &str, size: usize) -> Vec<String> {
    let mut state = seed;
    let mut next = |m: usize| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as usize) % m
    };
    let mut out = Vec::new();
    let mut total = 0;
    loop {
        let mut line = String::new();
        for i in 0..3 + next(18) {
            if i > 0 {
                line.push_str(gap);
            }
            line.push_str(words[next(words.len())]);
        }
        total += line.len();
        if total > size {
            return out;
        }
        out.push(line);
    }
}

/// Lowercases every line with `f`; returns a fold of the outputs.
fn pass(lines: &[String], f: fn(&str, &mut String), buffer: &mut String) -> u64 {
    let mut fold = 0u64;
    for line in lines {
        buffer.clear();
        f(black_box(line), buffer);
        let last = buffer.as_bytes().last().copied().unwrap_or(0);
        fold = fold.rotate_left(5) ^ buffer.len() as u64 ^ (u64::from(last) << 32);
        black_box(&buffer);
    }
    fold
}

fn main() -> ExitCode {
    let args = speed::args("speed_lowercase_paired", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::self_check();
    let corpora: Vec<Vec<String>> = SCRIPTS
        .iter()
        .enumerate()
        .map(|(s, &(_, words, gap))| lines(s as u64 + 7, words, gap, SCRIPT_BYTES))
        .collect();
    speed::setup(
        "speed_lowercase_paired",
        runs,
        &[("script_bytes", SCRIPT_BYTES.to_string())],
    );
    let mut record = String::new();
    let mut buffer = String::new();
    for run in 0..runs {
        for (&(script, _, _), lines) in SCRIPTS.iter().zip(&corpora) {
            let at = Instant::now();
            let fold_ref = pass(lines, lowercase_ref, &mut buffer);
            let ns_ref = at.elapsed().as_nanos();
            let at = Instant::now();
            let fold_fast = pass(lines, lowercase::lowercase_into, &mut buffer);
            let ns_fast = at.elapsed().as_nanos();
            speed::line(&format!("{script}_ref"), run, ns_ref);
            speed::line(&format!("{script}_fast"), run, ns_fast);
            if fold_ref != fold_fast {
                speed::fail("fold_differs");
            }
            let _ = writeln!(record, "{script} run={run} fold={fold_fast:016x}");
        }
    }
    // Every line of both sides, compared outside the timer.
    let (mut want, mut got) = (String::new(), String::new());
    for (&(script, _, _), lines) in SCRIPTS.iter().zip(&corpora) {
        let _ = writeln!(record, "{script}");
        for line in lines {
            want.clear();
            got.clear();
            lowercase_ref(line, &mut want);
            lowercase::lowercase_into(line, &mut got);
            if want != got {
                speed::fail("output_differs");
            }
            let _ = writeln!(record, "  {got}");
        }
    }
    speed::check_digest(&speed::sha256(record.as_bytes()));
    ExitCode::SUCCESS
}
