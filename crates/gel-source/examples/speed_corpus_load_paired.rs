//! Same-binary timing of `Corpus::load_with_threads`: one thread (the
//! reference side, `_ref`) against THREADS threads (`_t<THREADS>`), on the
//! inputs of `speed_corpus_load`. Needs `Corpus::load_with_threads`.
//!
//! Usage: `speed_corpus_load_paired [RUNS] [THREADS]` (RUNS in 1..=999,
//! default 9; THREADS in 1..=256, default 8).
//!
//! Cases (each run times them in this order):
//! - `load_64mib_ref`, `load_64mib_t<T>`: a 64 MiB imported text
//!   (2048 passages of 32 KiB), both pins checked.
//! - `load_r50k_16mib_ref`, `load_r50k_16mib_t<T>`: 50 000 passages of
//!   320 bytes.
//!
//! In every run the two sides of a pair must give the same result; otherwise
//! the example prints `SPEED_FAIL reason=result_mismatch` on stderr and exits 1.
//!
//! Output (stdout): one `SPEED_SETUP example=speed_corpus_load_paired runs=<R>
//! threads=1,<T> ...` line, one `SPEED name=<case> run=<i> ns=<n>` line per
//! case and run, and one final `SPEED_CHECK sha256=<hex>` over the case name
//! and result of every case and run. Output grammar: `docs/SPEED.md`.
#[path = "speed_corpus_load.rs"]
#[allow(dead_code)]
mod single;

// `single` carries its own copy of the helper; this one is the example's.
#[allow(clippy::duplicate_mod)]
#[path = "../../../tools/speed_support.rs"]
mod speed;

use gel_source::{import_text, Corpus, EncodedCorpus};
use single::{many, summary, text, BIG_TEXT, MANY_RECORDS};
use std::time::Instant;

fn load(e: &EncodedCorpus, threads: usize) -> Result<Corpus, gel_source::Error> {
    Corpus::load_with_threads(
        e.catalog_bytes(),
        e.text_bytes(),
        e.catalog_pin(),
        e.text_pin(),
        threads,
    )
}

fn main() {
    let args = speed::args("speed_corpus_load_paired", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let threads = speed::number(&args, 1, "THREADS", speed::THREADS_DEFAULT, 1..=256);
    speed::self_check();
    let big = import_text(text(1, BIG_TEXT).as_bytes(), "Large").expect("large corpus");
    let many = many(3);
    speed::setup(
        "speed_corpus_load_paired",
        runs,
        &[
            ("threads", format!("1,{threads}")),
            ("big_bytes", big.text_bytes().len().to_string()),
            (
                "big_records",
                big.load().map_or(0, |c| c.records().count()).to_string(),
            ),
            ("many_bytes", many.text_bytes().len().to_string()),
            ("many_records", MANY_RECORDS.to_string()),
        ],
    );
    let pairs: [(&str, &EncodedCorpus); 2] = [("load_64mib", &big), ("load_r50k_16mib", &many)];
    let mut check = speed::Sha256::new();
    for run in 0..runs {
        for (base, corpus) in pairs {
            let mut seen = Vec::with_capacity(2);
            for (name, count) in [
                (format!("{base}_ref"), 1),
                (format!("{base}_t{threads}"), threads),
            ] {
                let start = Instant::now();
                let result = load(corpus, count);
                let ns = start.elapsed().as_nanos();
                speed::line(&name, run, ns);
                let s = summary(&result);
                check.update(name.as_bytes());
                check.update(b"\n");
                check.update(s.as_bytes());
                check.update(b"\n");
                seen.push(s);
            }
            if seen[0] != seen[1] {
                speed::fail("result_mismatch");
            }
        }
    }
    speed::check(check);
}
