//! Phrase search where the per-line work is mostly finding the line end and
//! checking that the line is ASCII, for paired timing of two source trees.
//! Deterministic synthetic text, built outside the timer.
//!
//! Cases, each a fixed list of queries run once per run:
//! - `ascii_scan_16mib`: 16 MiB of short ASCII lines, queries with non-ASCII
//!   words only, so no ASCII line is tokenized;
//! - `ascii_long_scan_16mib`: the same with ASCII lines of 2 to 4 KiB;
//! - `ascii_match_16mib`: the text of `ascii_scan_16mib` with ASCII queries,
//!   so every line is tokenized;
//! - `ascii_scan_512kib`: 512 KiB of short ASCII lines (searched on the
//!   calling thread), the non-ASCII queries 16 times;
//! - `polish_16mib`: 16 MiB of Polish lines, mostly non-ASCII, including the
//!   query `za`, an ASCII piece of a non-ASCII word, which matches only when a
//!   non-ASCII line is taken for ASCII;
//! - `over_limit_16mib`: 16 MiB of ASCII lines longer than `MAX_LINE`, all
//!   skipped.
//!
//! Usage: `speed_line_ascii [RUNS]` (1..=999, default 9).
//!
//! Output: `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of run 0, which must be equal
//! on both trees. Every later run must give the same results; a difference
//! ends the run with `SPEED_FAIL reason=run_differs`. Output grammar:
//! `docs/SPEED.md`.
use gel_source::document::{self, MAX_LINE, MAX_TEXT};
use std::{fmt::Write as _, hint::black_box, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const ASCII_WORDS: [&str; 16] = [
    "Ada", "did", "not", "approve", "Bob's", "proposal", "gate", "is", "open", "the", "source",
    "record", "line", "of", "text", "42",
];
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
const NON_ASCII_QUERIES: [&str; 4] = ["łódź", "zażółć gęślą", "źródło nie", "ŻÓŁW"];
const ASCII_QUERIES: [&str; 4] = ["Ada did not approve", "gate is open", "missing token", "42"];
const POLISH_QUERIES: [&str; 5] = ["za", "zażółć gęślą", "jest otwarta", "łódź", "42"];
const SMALL_REPEATS: usize = 16;

/// Lines of words until `size` bytes; each line is at most a length drawn
/// from `lo..=hi` bytes before its "." and line end, and as many words as fit.
fn text(seed: u64, words: &[&str], size: usize, lo: usize, hi: usize) -> String {
    let mut state = seed;
    let mut next = |m: usize| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as usize) % m
    };
    let mut out = String::with_capacity(size);
    let mut line = String::new();
    loop {
        line.clear();
        let limit = lo + next(hi - lo + 1);
        loop {
            let gap = if line.is_empty() {
                ""
            } else if next(8) == 0 {
                ", "
            } else {
                " "
            };
            let word = words[next(words.len())];
            if !line.is_empty() && line.len() + gap.len() + word.len() > limit {
                break;
            }
            line.push_str(gap);
            line.push_str(word);
        }
        line.push_str(if next(16) == 0 { ".\r\n" } else { ".\n" });
        if out.len() + line.len() > size {
            return out;
        }
        out.push_str(&line);
    }
}

fn record(out: &mut String, case: &str, phrase: &str, r: &document::Search) {
    let _ = writeln!(
        out,
        "{case} {phrase:?} {} {} {:?}",
        r.matching_lines, r.skipped_long_lines, r.passages
    );
}

fn main() -> Result<(), String> {
    let args = speed::args("speed_line_ascii", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let ascii = text(1, &ASCII_WORDS, MAX_TEXT, 16, 128);
    let long = text(2, &ASCII_WORDS, MAX_TEXT, 2048, MAX_LINE - 1);
    let small = text(3, &ASCII_WORDS, 512 << 10, 16, 128);
    let polish = text(4, &POLISH_WORDS, MAX_TEXT, 16, 160);
    let over = text(5, &ASCII_WORDS, MAX_TEXT, MAX_LINE + 64, 2 * MAX_LINE);
    speed::setup(
        "speed_line_ascii",
        runs,
        &[
            ("ascii_bytes", ascii.len().to_string()),
            ("long_bytes", long.len().to_string()),
            ("small_bytes", small.len().to_string()),
            ("small_repeats", SMALL_REPEATS.to_string()),
            ("polish_bytes", polish.len().to_string()),
            ("over_bytes", over.len().to_string()),
        ],
    );
    let cases: [(&str, &String, &[&str], usize); 6] = [
        ("ascii_scan_16mib", &ascii, &NON_ASCII_QUERIES, 1),
        ("ascii_long_scan_16mib", &long, &NON_ASCII_QUERIES, 1),
        ("ascii_match_16mib", &ascii, &ASCII_QUERIES, 1),
        (
            "ascii_scan_512kib",
            &small,
            &NON_ASCII_QUERIES,
            SMALL_REPEATS,
        ),
        ("polish_16mib", &polish, &POLISH_QUERIES, 1),
        ("over_limit_16mib", &over, &ASCII_QUERIES, 1),
    ];
    let mut first: Option<String> = None;
    for run in 0..runs {
        let mut results = String::new();
        for (name, source, queries, repeats) in cases {
            let at = Instant::now();
            for _ in 0..repeats {
                for &phrase in queries {
                    let r = document::search(black_box(source), black_box(phrase))?;
                    record(&mut results, name, phrase, &r);
                }
            }
            speed::line(name, run, at.elapsed().as_nanos());
        }
        match &first {
            None => first = Some(results),
            Some(f) if *f != results => speed::fail("run_differs"),
            Some(_) => {}
        }
    }
    let results = first.expect("RUNS is at least 1");
    speed::check_digest(&speed::sha256(results.as_bytes()));
    Ok(())
}
