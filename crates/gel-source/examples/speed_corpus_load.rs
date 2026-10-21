//! Repeatable timing of catalog loading through `Corpus::load` and
//! `EncodedCorpus::load`. Only API present before `Corpus::load_with_threads`
//! is used, so the same file builds on earlier trees.
//!
//! Usage: `speed_corpus_load [RUNS]` (RUNS in 1..=999, default 9). Every
//! input is generated from fixed seeds.
//!
//! Cases (each run times every case once, in this order):
//! - `corpus_load_64mib`: `Corpus::load` of a 64 MiB imported text
//!   (2048 passages of 32 KiB), both pins checked.
//! - `encoded_load_64mib`: `EncodedCorpus::load` of the same corpus.
//! - `encoded_load_r50k_16mib`: `EncodedCorpus::load` of 50 000 passages of
//!   320 bytes.
//! - `small_load_256kib`: `EncodedCorpus::load` of a 256 KiB imported text
//!   (8 passages).
//! - `reject_first_64mib`: `Corpus::load` of the 64 MiB corpus whose first
//!   row states a wrong passage digest (pins match the bytes); the result is
//!   `Integrity`.
//!
//! Output (stdout): one `SPEED_SETUP example=speed_corpus_load runs=<R> ...`
//! line, one `SPEED name=<case> run=<i> ns=<n>` line per case and run, and one
//! final `SPEED_CHECK sha256=<hex>` over the case name and result (root, text
//! pin and record count, or the error) of every case and run. The digest is
//! computed by `tools/speed_support.rs`, not by the crate. Output grammar:
//! `docs/SPEED.md`. `speed_corpus_load_paired` includes this file for its
//! inputs.
use gel_source::{digest, import_text, Address, Corpus, CorpusBuilder, EncodedCorpus, Error};
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

pub const BIG_TEXT: usize = 64 << 20;
pub const SMALL_TEXT: usize = 256 << 10;
pub const MANY_RECORDS: usize = 50_000;
pub const MANY_PASSAGE: usize = 320;

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
pub fn text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut state = seed;
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        s.push_str(WORDS[(z % 8) as usize]);
        s.push(if z % 13 == 0 { '\n' } else { ' ' });
    }
    s.truncate(len);
    s
}

/// `MANY_RECORDS` passages of `MANY_PASSAGE` bytes, four per node.
pub fn many(seed: u64) -> EncodedCorpus {
    let body = text(seed, MANY_RECORDS * MANY_PASSAGE);
    let mut b = CorpusBuilder::new();
    for role in 0..MANY_RECORDS {
        let node = (role / 4) as u64;
        b.push(
            Address {
                node,
                role: role as u64,
            },
            node % 11,
            &format!("Title {}", node % 97),
            &format!("Part {role}"),
            &body[role * MANY_PASSAGE..(role + 1) * MANY_PASSAGE],
        )
        .expect("generated passage");
    }
    b.finish().expect("generated corpus")
}

/// The catalog with one hex digit of the first row's passage digest changed.
fn wrong_first_digest(catalog: &[u8]) -> Vec<u8> {
    let mut out = catalog.to_vec();
    let row = 13;
    let mut tabs = 0;
    let mut k = row;
    while tabs < 5 {
        if out[k] == b'\t' {
            tabs += 1;
        }
        k += 1;
    }
    out[k] = if out[k] == b'0' { b'1' } else { b'0' };
    out
}

/// Root, text pin and record count of a loaded corpus, or the error.
pub fn summary(result: &Result<Corpus, Error>) -> String {
    match result {
        Ok(c) => format!(
            "ok {} {} {}",
            gel_source::hex(&c.root()),
            gel_source::hex(&c.text_hash()),
            c.records().count()
        ),
        Err(e) => format!("err {e:?}"),
    }
}

fn main() {
    let args = speed::args("speed_corpus_load", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::self_check();
    let big = import_text(text(1, BIG_TEXT).as_bytes(), "Large").expect("large corpus");
    let small = import_text(text(2, SMALL_TEXT).as_bytes(), "Small").expect("small corpus");
    let many = many(3);
    let bad = wrong_first_digest(big.catalog_bytes());
    let bad_pin = digest(&bad);
    speed::setup(
        "speed_corpus_load",
        runs,
        &[
            ("big_bytes", big.text_bytes().len().to_string()),
            (
                "big_records",
                big.load().map_or(0, |c| c.records().count()).to_string(),
            ),
            ("many_bytes", many.text_bytes().len().to_string()),
            ("many_records", MANY_RECORDS.to_string()),
            ("small_bytes", small.text_bytes().len().to_string()),
            (
                "small_records",
                small.load().map_or(0, |c| c.records().count()).to_string(),
            ),
        ],
    );
    let mut check = speed::Sha256::new();
    for run in 0..runs {
        type Case<'a> = (&'a str, Box<dyn Fn() -> Result<Corpus, Error> + 'a>);
        let cases: [Case; 5] = [
            (
                "corpus_load_64mib",
                Box::new(|| {
                    Corpus::load(
                        big.catalog_bytes(),
                        big.text_bytes(),
                        big.catalog_pin(),
                        big.text_pin(),
                    )
                }),
            ),
            ("encoded_load_64mib", Box::new(|| big.load())),
            ("encoded_load_r50k_16mib", Box::new(|| many.load())),
            ("small_load_256kib", Box::new(|| small.load())),
            (
                "reject_first_64mib",
                Box::new(|| Corpus::load(&bad, big.text_bytes(), bad_pin, big.text_pin())),
            ),
        ];
        for (name, case) in &cases {
            let start = Instant::now();
            let result = case();
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            check.update(name.as_bytes());
            check.update(b"\n");
            check.update(summary(&result).as_bytes());
            check.update(b"\n");
        }
    }
    speed::check(check);
}
