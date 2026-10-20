//! Phrase search over lines that exercise Unicode normalization, for paired
//! timing of two source trees. Uses only `document::search`, so the same file
//! builds on an earlier tree. Inputs are deterministic and built outside the
//! timer.
//!
//! Line cases (each run is 200 searches of one line, on the calling thread):
//! an ASCII line and a Polish line in NFC, which do not reach normalization
//! or pass its quick check below U+0300; the same Polish text in NFD; Polish
//! NFC text with one combining mark at its end; 2047 combining marks in
//! reverse and in canonical order; decomposed Hangul jamo; Hangul syllables.
//! Document cases (one search per run): 4096 copies of a line (fewer when
//! they would exceed the 16 MiB text limit), searched with the thread plan
//! that `search` picks itself.
//!
//! Usage: `speed_nfc_fast [RUNS]` (RUNS in 1..=999, default 9).
//! Output: one `SPEED_SETUP` line, `SPEED name=<case> run=<i> ns=<n>` per case
//! and run, and a last `SPEED_CHECK sha256=<hex>` over the results of run 0;
//! every later run must give the same results, or the program prints
//! `SPEED_FAIL reason=<slug>` on stderr and exits 1. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::document::{search, Search, MAX_LINE, MAX_TEXT};
use std::{fmt::Write as _, hint::black_box, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const LINE_CALLS: usize = 200;
const DOCUMENT_LINES: usize = 4096;
const POLISH: &str = "Zażółć gęślą jaźń, źdźbło i łódź. ";
/// The same text in NFD: each Polish letter with a canonical decomposition is
/// a base letter and a combining mark.
const POLISH_NFD: &str = "Za\u{007A}\u{0307}o\u{0301}\u{0142}c\u{0301} ge\u{0328}s\u{0301}\u{006C}a\u{0328} \
                          jaz\u{0301}n\u{0301}, z\u{0301}dz\u{0301}b\u{0142}o i \u{0142}o\u{0301}dz\u{0301}. ";

fn record(out: &mut String, case: &str, r: &Search) {
    let _ = writeln!(
        out,
        "{case} {} {} {:?}",
        r.matching_lines, r.skipped_long_lines, r.passages
    );
}

fn main() -> ExitCode {
    let args = speed::args("speed_nfc_fast", "[RUNS]", 1);
    let runs = speed::runs(&args);

    let lines: [(&str, String, &str); 8] = [
        (
            "ascii_line",
            "plain ascii words of a source line, number 42. ".repeat(30),
            "source line",
        ),
        ("polish_nfc_line", POLISH.repeat(40), "łódź"),
        ("polish_nfd_line", POLISH_NFD.repeat(40), "łódź"),
        (
            "polish_late_mark_line",
            format!("{}cafe\u{0301}", POLISH.repeat(38)),
            "café",
        ),
        (
            "reversed_marks_line",
            format!("a{}{}", "\u{0301}".repeat(1023), "\u{0316}".repeat(1024)),
            "a",
        ),
        (
            "ordered_marks_line",
            format!("a{}{}", "\u{0316}".repeat(1024), "\u{0301}".repeat(1023)),
            "a",
        ),
        (
            "hangul_jamo_line",
            "\u{1100}\u{1161}\u{11A8}".repeat(450),
            "\u{AC01}",
        ),
        (
            "hangul_syllables_line",
            "한국어 문장은 대부분 완성형 음절로 쓴다 ".repeat(40),
            "문장은",
        ),
    ];
    for (name, line, _) in &lines {
        if line.len() > MAX_LINE {
            eprintln!("{name}: {} bytes", line.len());
            speed::fail("line_too_long");
        }
    }
    let document = |line: &str| {
        let count = (MAX_TEXT / (line.len() + 1)).min(DOCUMENT_LINES);
        format!("{line}\n").repeat(count)
    };
    let documents: Vec<(&str, String, &str)> = [
        ("doc_ascii", 0),
        ("doc_polish_nfc", 1),
        ("doc_polish_nfd", 2),
        ("doc_reversed_marks", 4),
        ("doc_hangul_jamo", 6),
        ("doc_hangul_syllables", 7),
    ]
    .into_iter()
    .map(|(name, i)| (name, document(&lines[i].1), lines[i].2))
    .collect();

    let byte_keys: Vec<(String, usize)> = lines
        .iter()
        .map(|(name, line, _)| (format!("{name}_bytes"), line.len()))
        .chain(
            documents
                .iter()
                .map(|(name, text, _)| (format!("{name}_bytes"), text.len())),
        )
        .collect();
    let mut fields = vec![
        ("line_calls", LINE_CALLS.to_string()),
        ("threads", "auto".to_string()),
    ];
    fields.extend(byte_keys.iter().map(|(k, v)| (k.as_str(), v.to_string())));
    speed::setup("speed_nfc_fast", runs, &fields);

    let mut first: Option<String> = None;
    for run in 0..runs {
        let mut results = String::new();
        for (name, line, phrase) in &lines {
            let mut last = None;
            let at = Instant::now();
            for _ in 0..LINE_CALLS {
                last = Some(search(black_box(line), black_box(phrase)));
            }
            let ns = at.elapsed().as_nanos();
            match last {
                Some(Ok(r)) => record(&mut results, name, &r),
                _ => speed::fail("search_error"),
            }
            speed::line(name, run, ns);
        }
        for (name, text, phrase) in &documents {
            let at = Instant::now();
            let r = search(black_box(text), black_box(phrase));
            let ns = at.elapsed().as_nanos();
            match r {
                Ok(r) => record(&mut results, name, &r),
                Err(_) => speed::fail("search_error"),
            }
            speed::line(name, run, ns);
        }
        match &first {
            None => first = Some(results),
            Some(f) if *f != results => speed::fail("results_changed"),
            Some(_) => {}
        }
    }
    let results = first.unwrap_or_default();
    let mut hash = speed::Sha256::new();
    hash.update(results.as_bytes());
    speed::check(hash);
    ExitCode::SUCCESS
}
