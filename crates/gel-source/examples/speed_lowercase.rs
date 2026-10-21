//! Phrase search over documents whose lines are not ASCII, for paired timing
//! of two source trees. Uses only `document::search`, so the same file builds
//! on an earlier tree. Deterministic synthetic text, built outside the timer.
//!
//! Each case searches 16 documents of 512 KiB (each one below the size at
//! which a search is split over threads) with 2 queries, the documents shared
//! out over 1 or `THREADS` threads. Scripts: `ascii` (only ASCII lines, a
//! control case for code outside the change), `polish`, `greek` (with both
//! sigma forms), `cjk_hangul` (Hangul, CJK and kana with ASCII spaces) and
//! `mixed` (Turkish dotted and dotless i, German sharp s, Cyrillic).
//!
//! Usage: `speed_lowercase [RUNS] [THREADS]` (defaults 9 and 8; RUNS in
//! 1..=999, THREADS in 1..=256).
//!
//! Output (stdout): one `SPEED_SETUP example=speed_lowercase runs=<R>
//! threads=<T> documents=16 document_bytes=524288 queries=2` line, one
//! `SPEED name=<case> run=<i> ns=<n>` line per case and run, and a last
//! `SPEED_CHECK sha256=<hex>` line over the case names and every result of
//! every run, which must be equal on both trees. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::document;
use std::{fmt::Write as _, hint::black_box, process::ExitCode, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const DOCUMENTS: usize = 16;
const DOCUMENT_BYTES: usize = 512 << 10;

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

/// (name, words, a query that occurs, a query that does not).
const SCRIPTS: [(&str, &[&str; 16], &str, &str); 5] = [
    ("ascii", &ASCII_WORDS, "gate is", "missing token"),
    ("polish", &POLISH_WORDS, "łódź", "brak słowa"),
    ("greek", &GREEK_WORDS, "σοφός", "λείπει λέξη"),
    ("cjk_hangul", &CJK_HANGUL_WORDS, "서울", "없는 단어"),
    ("mixed", &MIXED_WORDS, "istanbul", "yok kelime"),
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

/// The searches of one document, written as one line.
fn search_document(source: &str, queries: [&str; 2]) -> Result<String, &'static str> {
    let mut out = String::new();
    for phrase in queries {
        let r = document::search(black_box(source), black_box(phrase))?;
        let _ = write!(
            out,
            "{phrase:?} {} {} {:?}; ",
            r.matching_lines, r.skipped_long_lines, r.passages
        );
    }
    Ok(out)
}

/// Searches every document on `threads` threads, each taking every
/// `threads`-th document; results in document order.
fn search_all(
    documents: &[String],
    queries: [&str; 2],
    threads: usize,
) -> Result<Vec<String>, &'static str> {
    if threads <= 1 {
        return documents
            .iter()
            .map(|d| search_document(d, queries))
            .collect();
    }
    let mut slots: Vec<Option<Result<String, &'static str>>> = vec![None; documents.len()];
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|first| {
                scope.spawn(move || {
                    (first..documents.len())
                        .step_by(threads)
                        .map(|k| (k, search_document(&documents[k], queries)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for worker in workers {
            for (k, result) in worker
                .join()
                .unwrap_or_else(|e| std::panic::resume_unwind(e))
            {
                slots[k] = Some(result);
            }
        }
    });
    slots
        .into_iter()
        .map(|slot| slot.unwrap_or(Err("missing result")))
        .collect()
}

fn main() -> ExitCode {
    let args = speed::args("speed_lowercase", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let threads = speed::number(&args, 1, "THREADS", speed::THREADS_DEFAULT, 1..=256);
    speed::self_check();
    let corpora: Vec<Vec<String>> = SCRIPTS
        .iter()
        .enumerate()
        .map(|(s, &(_, words, _, _))| {
            (0..DOCUMENTS)
                .map(|d| text((100 * s + d) as u64 + 1, words, DOCUMENT_BYTES))
                .collect()
        })
        .collect();
    let mut thread_counts = vec![1];
    if threads > 1 {
        thread_counts.push(threads);
    }
    speed::setup(
        "speed_lowercase",
        runs,
        &[
            ("threads", threads.to_string()),
            ("documents", DOCUMENTS.to_string()),
            ("document_bytes", DOCUMENT_BYTES.to_string()),
            ("queries", "2".to_string()),
        ],
    );
    let mut record = String::new();
    for run in 0..runs {
        for (&(script, _, hit, miss), documents) in SCRIPTS.iter().zip(&corpora) {
            for &t in &thread_counts {
                let name = format!("{script}_16x512kib_t{t}");
                let at = Instant::now();
                let results = search_all(documents, [hit, miss], t);
                let ns = at.elapsed().as_nanos();
                let results = match results {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{name}: {e}");
                        speed::fail("search_error");
                    }
                };
                speed::line(&name, run, ns);
                let _ = writeln!(record, "{name}");
                for line in results {
                    let _ = writeln!(record, "  {line}");
                }
            }
        }
    }
    speed::check_digest(&speed::sha256(record.as_bytes()));
    ExitCode::SUCCESS
}
