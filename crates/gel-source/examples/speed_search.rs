//! Repeated phrase search over the largest admitted inputs, for paired timing
//! of two source trees. Deterministic synthetic text, built outside the timer.
//!
//! Cases: one 16 MiB ASCII document, one 16 MiB Polish document (non-ASCII
//! lines), one 64 KiB ASCII document, and a 64-document collection of 1 MiB
//! each (64 MiB, the collection limit). Each case runs all of its queries once
//! per run. Usage: `speed_search [RUNS]` (1..=999, default 9).
//!
//! Output: `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result of run 0, which must be equal
//! on both trees. Every later run must give the same results; a difference
//! ends the run with `SPEED_FAIL reason=run_differs`. Output grammar:
//! `docs/SPEED.md`.
use gel_source::{
    collection::{Collection, CollectionBuilder},
    document::{self, MAX_TEXT},
    hex,
};
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
const DOCUMENT_QUERIES: [&str; 5] = [
    "Ada did not approve",
    "gate is open",
    "zażółć gęślą",
    "łódź",
    "missing token",
];

fn text(seed: u64, words: &[&str], size: usize) -> String {
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
        for i in 0..3 + next(18) {
            if i > 0 {
                line.push_str(if next(8) == 0 { ", " } else { " " });
            }
            line.push_str(words[next(words.len())]);
        }
        line.push_str(if next(16) == 0 { ".\r\n" } else { ".\n" });
        if out.len() + line.len() > size {
            return out;
        }
        out.push_str(&line);
    }
}

fn record_document(out: &mut String, case: &str, phrase: &str, r: &document::Search) {
    let _ = writeln!(
        out,
        "{case} {phrase:?} {} {} {:?}",
        r.matching_lines, r.skipped_long_lines, r.passages
    );
}

fn record_collection(out: &mut String, phrase: &str, r: &gel_source::collection::Search) {
    let _ = writeln!(
        out,
        "collection {phrase:?} {} {} {}",
        r.matching_lines, r.skipped_long_lines, r.documents_examined
    );
    for h in &r.hits {
        let _ = writeln!(
            out,
            "  {} {} {} {:?} {:?}",
            hex(&h.root()),
            h.document_id(),
            hex(&h.document_hash()),
            h.span(),
            h.quote()
        );
    }
}

fn main() -> Result<(), String> {
    let args = speed::args("speed_search", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let ascii = text(1, &ASCII_WORDS, MAX_TEXT);
    let polish = text(2, &POLISH_WORDS, MAX_TEXT);
    let small = text(3, &ASCII_WORDS, 64 << 10);
    let mut builder = CollectionBuilder::new();
    for i in 0..64_u64 {
        let words: &[&str] = if i % 2 == 0 {
            &ASCII_WORDS
        } else {
            &POLISH_WORDS
        };
        builder.add(&format!("Document {i}"), &text(10 + i, words, 1 << 20))?;
    }
    let collection: Collection = builder.build();
    speed::setup(
        "speed_search",
        runs,
        &[
            ("ascii_bytes", ascii.len().to_string()),
            ("polish_bytes", polish.len().to_string()),
            ("small_bytes", small.len().to_string()),
            ("collection_bytes", collection.text_bytes().to_string()),
            ("documents", collection.documents().count().to_string()),
        ],
    );
    let documents = [
        ("doc_ascii_16mib", &ascii),
        ("doc_polish_16mib", &polish),
        ("doc_ascii_64kib", &small),
    ];
    let mut first: Option<String> = None;
    for run in 0..runs {
        let mut results = String::new();
        for (name, source) in documents {
            let at = Instant::now();
            for phrase in DOCUMENT_QUERIES {
                let r = document::search(black_box(source), black_box(phrase))?;
                record_document(&mut results, name, phrase, &r);
            }
            speed::line(name, run, at.elapsed().as_nanos());
        }
        let at = Instant::now();
        for phrase in DOCUMENT_QUERIES {
            let r = black_box(&collection).search(black_box(phrase))?;
            record_collection(&mut results, phrase, &r);
        }
        speed::line("collection_64x1mib", run, at.elapsed().as_nanos());
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
