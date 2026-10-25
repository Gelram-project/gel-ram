//! Repeatable timing of `Corpus::load` and `EncodedCorpus::load` on catalogs
//! whose rows repeat passage ranges, next to catalogs whose ranges are all
//! distinct. Only API present before the loader hashed each distinct range
//! once is used, so the same file builds on earlier trees.
//!
//! Usage: `speed_quote_ranges [RUNS]` (RUNS in 1..=999, default 9). Every
//! input is generated from fixed seeds.
//!
//! Cases (each run times every case once, in this order):
//! - `repeat_2049x32kib`: `Corpus::load` of a 32 KiB text and 2049 rows that
//!   all state the same 32 KiB range with its true digest.
//! - `repeat_cycle_r8192_64x32kib`: `Corpus::load` of a 2 MiB text and 8192
//!   rows that cycle through its 64 ranges of 32 KiB, so that the rows are
//!   not in range order.
//! - `repeat_conflict_2049x32kib`: `repeat_2049x32kib` with a wrong digest in
//!   the last row (pins match the bytes); the result is `Integrity`.
//! - `distinct_encoded_64mib`: `EncodedCorpus::load` of a 64 MiB imported
//!   text (2048 distinct passages of 32 KiB).
//! - `distinct_r50k_16mib`: `EncodedCorpus::load` of 50 000 distinct passages
//!   of 320 bytes.
//! - `reversed_r50k_16mib`: `Corpus::load` of the same 50 000 rows in reverse
//!   order: distinct ranges, not in range order.
//!
//! Output (stdout): one `SPEED_SETUP example=speed_quote_ranges runs=<R> ...`
//! line, one `SPEED name=<case> run=<i> ns=<n>` line per case and run, and one
//! final `SPEED_CHECK sha256=<hex>` over the case name and result (root, text
//! pin and record count, or the error) of every case and run. The digest is
//! computed by `tools/speed_support.rs`, not by the crate. Output grammar:
//! `docs/SPEED.md`.
use gel_source::{digest, hex, import_text, Address, Corpus, CorpusBuilder, Error, Hash};
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const PASSAGE: usize = 32 << 10;
const REPEATS: usize = 2049;
const CYCLE_RANGES: usize = 64;
const CYCLE_ROWS: usize = 8192;
const BIG_TEXT: usize = 64 << 20;
const MANY_RECORDS: usize = 50_000;
const MANY_PASSAGE: usize = 320;

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut rng = speed::SplitMix64(seed);
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        let z = rng.next();
        s.push_str(WORDS[(z % 8) as usize]);
        s.push(if z % 13 == 0 { '\n' } else { ' ' });
    }
    s.truncate(len);
    s
}

/// A catalog whose row `role` states range `range(role)` of `text`; row
/// `wrong`, if any, states 32 bytes of 0xff as its digest, which sorts after
/// the true digest of the same range.
fn catalog(
    text: &[u8],
    rows: usize,
    range: impl Fn(usize) -> (usize, usize),
    wrong: Option<usize>,
) -> Vec<u8> {
    let mut s = String::from("GELCORPUS082\n");
    let title = hex(b"Repeated");
    for role in 0..rows {
        let (start, len) = range(role);
        let h: Hash = if wrong == Some(role) {
            [0xff; 32]
        } else {
            digest(&text[start..start + len])
        };
        s.push_str(&format!(
            "0\t{role}\t0\t{start}\t{len}\t{}\t{title}\t{}\n",
            hex(&h),
            hex(format!("Part {role}").as_bytes())
        ));
    }
    s.into_bytes()
}

/// The catalog with its rows in reverse order.
fn reversed(catalog: &[u8]) -> Vec<u8> {
    let raw = std::str::from_utf8(catalog).expect("catalog is UTF-8");
    let mut out = String::from("GELCORPUS082\n");
    for line in raw[13..].split_terminator('\n').rev() {
        out.push_str(line);
        out.push('\n');
    }
    out.into_bytes()
}

/// Root, text pin and record count of a loaded corpus, or the error.
fn summary(result: &Result<Corpus, Error>) -> String {
    match result {
        Ok(c) => format!(
            "ok {} {} {}",
            hex(&c.root()),
            hex(&c.text_hash()),
            c.records().count()
        ),
        Err(e) => format!("err {e:?}"),
    }
}

/// Catalog, text and both pins of one `Corpus::load` input.
struct Input {
    catalog: Vec<u8>,
    text: Vec<u8>,
    catalog_pin: Hash,
    text_pin: Hash,
}
impl Input {
    fn new(catalog: Vec<u8>, text: Vec<u8>) -> Self {
        Self {
            catalog_pin: digest(&catalog),
            text_pin: digest(&text),
            catalog,
            text,
        }
    }
    fn load(&self) -> Result<Corpus, Error> {
        Corpus::load(&self.catalog, &self.text, self.catalog_pin, self.text_pin)
    }
}

fn main() {
    let args = speed::args("speed_quote_ranges", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::self_check();
    let one = text(1, PASSAGE).into_bytes();
    let repeat = Input::new(catalog(&one, REPEATS, |_| (0, PASSAGE), None), one.clone());
    let conflict = Input::new(
        catalog(&one, REPEATS, |_| (0, PASSAGE), Some(REPEATS - 1)),
        one,
    );
    let cycle_text = text(2, CYCLE_RANGES * PASSAGE).into_bytes();
    let cycle = Input::new(
        catalog(
            &cycle_text,
            CYCLE_ROWS,
            |role| ((role % CYCLE_RANGES) * PASSAGE, PASSAGE),
            None,
        ),
        cycle_text,
    );
    let big = import_text(text(3, BIG_TEXT).as_bytes(), "Large").expect("large corpus");
    let body = text(4, MANY_RECORDS * MANY_PASSAGE);
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
    let many = b.finish().expect("generated corpus");
    let back = Input::new(reversed(many.catalog_bytes()), many.text_bytes().to_vec());
    speed::setup(
        "speed_quote_ranges",
        runs,
        &[
            ("passage_bytes", PASSAGE.to_string()),
            ("repeat_rows", REPEATS.to_string()),
            ("cycle_ranges", CYCLE_RANGES.to_string()),
            ("cycle_rows", CYCLE_ROWS.to_string()),
            ("big_bytes", big.text_bytes().len().to_string()),
            ("many_records", MANY_RECORDS.to_string()),
            ("many_bytes", many.text_bytes().len().to_string()),
        ],
    );
    let mut check = speed::Sha256::new();
    for run in 0..runs {
        type Case<'a> = (&'a str, Box<dyn Fn() -> Result<Corpus, Error> + 'a>);
        let cases: [Case; 6] = [
            ("repeat_2049x32kib", Box::new(|| repeat.load())),
            ("repeat_cycle_r8192_64x32kib", Box::new(|| cycle.load())),
            ("repeat_conflict_2049x32kib", Box::new(|| conflict.load())),
            ("distinct_encoded_64mib", Box::new(|| big.load())),
            ("distinct_r50k_16mib", Box::new(|| many.load())),
            ("reversed_r50k_16mib", Box::new(|| back.load())),
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
