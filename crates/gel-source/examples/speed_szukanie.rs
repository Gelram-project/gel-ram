//! Repeated phrase search over the largest admitted inputs, for paired timing
//! of two source trees. Deterministic synthetic text, built outside the timer.
//!
//! Cases: one 16 MiB ASCII document, one 16 MiB Polish document (non-ASCII
//! lines), one 64 KiB ASCII document, and a 64-document collection of 1 MiB
//! each (64 MiB, the collection limit). Each case runs all of its queries once
//! per run. Usage: `speed_szukanie [RUNS]` (default 9).
//!
//! Output: `SPEED name=<case> run=<i> ns=<n>` per case and run, then
//! `SPEED_CHECK sha256=<hex>` over every result, which must be equal on both
//! trees.
use gel_source::{
    collection::{Collection, CollectionBuilder},
    digest,
    document::{self, MAX_TEXT},
    hex,
};
use std::{fmt::Write as _, hint::black_box, time::Instant};

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
    let runs: usize = std::env::args()
        .nth(1)
        .map_or(Ok(9), |a| a.parse())
        .map_err(|e| format!("RUNS: {e}"))?;
    if runs == 0 {
        return Err("RUNS must be at least 1".into());
    }
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
    eprintln!(
        "INPUT ascii_bytes={} polish_bytes={} small_bytes={} collection_bytes={} documents={}",
        ascii.len(),
        polish.len(),
        small.len(),
        collection.text_bytes(),
        collection.documents().count()
    );
    let documents = [
        ("doc-ascii-16mib", &ascii),
        ("doc-polish-16mib", &polish),
        ("doc-ascii-64kib", &small),
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
            println!("SPEED name={name} run={run} ns={}", at.elapsed().as_nanos());
        }
        let at = Instant::now();
        for phrase in DOCUMENT_QUERIES {
            let r = black_box(&collection).search(black_box(phrase))?;
            record_collection(&mut results, phrase, &r);
        }
        println!(
            "SPEED name=collection-64x1mib run={run} ns={}",
            at.elapsed().as_nanos()
        );
        match &first {
            None => first = Some(results),
            Some(f) if *f != results => return Err(format!("run {run} changed the results")),
            Some(_) => {}
        }
    }
    let results = first.unwrap_or_default();
    println!("SPEED_CHECK sha256={}", hex(&digest(results.as_bytes())));
    Ok(())
}
