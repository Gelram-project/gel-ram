//! Repeatable timing of the source-catalog reader and writer.
//!
//! Usage: `speed_catalog_parse [RUNS]` (RUNS in 1..=999, default 9). Every
//! input is generated from fixed seeds, so two builds can be compared by
//! their `SPEED_CHECK` line. Only the public API is used.
//!
//! Inputs: 50,000 records, 48-byte ASCII quotes, 5,000 nodes of 10 roles each
//! (one title of about 32 bytes per node, sections of about 20 bytes).
//!
//! Cases, in order within each run:
//! - `builder_push_k50`: 50,000 `CorpusBuilder::push` calls (no `finish`).
//! - `encoded_load_k50`: `EncodedCorpus::load` of that corpus.
//! - `hex_digest_x100k`: 100,000 `hex` calls on 32-byte digests.
//! - `import_load_8mib`: `import_text` of 8 MiB of text, then `load`.
//! - `quote_k50`: `Corpus::quote` of all 50,000 addresses.
//! - `find_title_k5`: `Corpus::find_title` of all 5,000 titles, upper case.
//!
//! Output: one `SPEED_SETUP` line, `SPEED name=<case> run=<i> ns=<n>` per case
//! and run, then `SPEED_CHECK sha256=<hex>` over every produced result, the
//! digest computed by `tools/speed_support.rs`. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::{digest, hex, import_text, Address, Corpus, CorpusBuilder};
use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const RECORDS: usize = 50_000;
const ROLES: usize = 10;
const NODES: usize = RECORDS / ROLES;
const QUOTE_BYTES: usize = 48;
const HEX_CALLS: usize = 100_000;
const IMPORT_BYTES: usize = 8 << 20;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn text(rng: &mut Rng, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        let z = rng.next();
        s.push_str(WORDS[(z % 8) as usize]);
        s.push(if z % 13 == 0 { '\n' } else { ' ' });
    }
    s.truncate(len);
    s
}

/// Lowercase hexadecimal from the shared helper, so the check line does not
/// depend on the code under test.
fn check_hex(bytes: &[u8]) -> String {
    speed::hex(bytes)
}

struct Row {
    address: Address,
    entry: u64,
    title: String,
    section: String,
    quote: String,
}

fn rows() -> Vec<Row> {
    let mut rng = Rng(0x6361_7461_6c6f_6721);
    let mut rows = Vec::with_capacity(RECORDS);
    for node in 0..NODES as u64 {
        let title = format!("Node title {node:05} of the source");
        for role in 0..ROLES as u64 {
            rows.push(Row {
                address: Address {
                    node: 100_000 + node,
                    role,
                },
                entry: node * 3 + 1,
                title: title.clone(),
                section: format!("Section {role} of {node}"),
                quote: text(&mut rng, QUOTE_BYTES),
            });
        }
    }
    rows
}

fn push_all(rows: &[Row]) -> Result<CorpusBuilder, String> {
    let mut builder = CorpusBuilder::new();
    for r in rows {
        builder
            .push(r.address, r.entry, &r.title, &r.section, &r.quote)
            .map_err(|e| format!("PUSH {e:?}"))?;
    }
    Ok(builder)
}

fn report(name: &str, run: usize, start: Instant) {
    speed::line(name, run, start.elapsed().as_nanos());
}

/// The parsed records of a corpus, as one line of text per record.
fn summary(corpus: &Corpus) -> Vec<u8> {
    let mut out = Vec::new();
    for r in corpus.records() {
        out.extend_from_slice(
            format!(
                "{} {} {} {} {} {} {:?} {:?}\n",
                r.address.node,
                r.address.role,
                r.entry,
                r.start,
                r.end,
                check_hex(&r.hash),
                r.title,
                r.section
            )
            .as_bytes(),
        );
    }
    out
}

fn run(runs: usize) -> Result<(), String> {
    let rows = rows();
    let corpus = push_all(&rows)?
        .finish()
        .map_err(|e| format!("FINISH {e:?}"))?;
    let loaded = corpus.load().map_err(|e| format!("LOAD {e:?}"))?;
    let lookups: Vec<String> = (0..NODES)
        .map(|i| rows[i * ROLES].title.to_uppercase())
        .collect();
    let digests: Vec<[u8; 32]> = (0..1024u32).map(|i| digest(&i.to_le_bytes())).collect();
    let mut rng = Rng(0x696d_706f_7274_3821);
    let import = text(&mut rng, IMPORT_BYTES);

    let mut out: Vec<u8> = Vec::new();
    let mut record = |name: &str, value: &[u8]| {
        out.extend_from_slice(name.as_bytes());
        out.push(b' ');
        out.extend_from_slice(value);
        out.push(b'\n');
    };
    for run in 0..runs {
        let start = Instant::now();
        let builder = push_all(&rows)?;
        report("builder_push_k50", run, start);
        let built = builder.finish().map_err(|e| format!("FINISH {e:?}"))?;
        record(
            "builder_push_k50",
            format!(
                "{} {} {}",
                check_hex(&built.catalog_pin()),
                check_hex(&built.text_pin()),
                built.catalog_bytes().len()
            )
            .as_bytes(),
        );
        drop(built);

        let start = Instant::now();
        let c = corpus.load().map_err(|e| format!("LOAD {e:?}"))?;
        report("encoded_load_k50", run, start);
        record("encoded_load_k50", &check_hex(&c.root()).into_bytes());
        record("encoded_load_k50", &summary(&c));
        drop(c);

        let start = Instant::now();
        let mut fold = 0u64;
        for i in 0..HEX_CALLS {
            let s = black_box(hex(&digests[i % digests.len()]));
            fold = fold
                .wrapping_mul(31)
                .wrapping_add(u64::from(s.as_bytes()[i % 64]));
        }
        report("hex_digest_x100k", run, start);
        record("hex_digest_x100k", fold.to_string().as_bytes());
        let all: String = digests.iter().map(|d| hex(d) + "\n").collect();
        record("hex_digest_x100k", all.as_bytes());

        let start = Instant::now();
        let c = import_text(import.as_bytes(), "Speed")
            .map_err(|e| format!("IMPORT {e:?}"))?
            .load()
            .map_err(|e| format!("IMPORT LOAD {e:?}"))?;
        report("import_load_8mib", run, start);
        record(
            "import_load_8mib",
            format!("{} {}", check_hex(&c.root()), c.records().count()).as_bytes(),
        );
        drop(c);

        let start = Instant::now();
        let mut quoted = 0usize;
        for r in &rows {
            let p = loaded
                .quote(r.address)
                .map_err(|e| format!("QUOTE {e:?}"))?;
            quoted += black_box(p).quote().len();
        }
        report("quote_k50", run, start);
        record("quote_k50", quoted.to_string().as_bytes());

        let start = Instant::now();
        let mut found = 0u64;
        for t in &lookups {
            for &node in loaded.find_title(t) {
                found = found.wrapping_add(node);
            }
        }
        report("find_title_k5", run, start);
        record("find_title_k5", found.to_string().as_bytes());
    }
    speed::check_digest(&speed::sha256(&out));
    Ok(())
}

fn main() -> ExitCode {
    let args = speed::args("speed_catalog_parse", "[RUNS]", 1);
    let runs = speed::runs(&args);
    speed::setup(
        "speed_catalog_parse",
        runs,
        &[
            ("records", RECORDS.to_string()),
            ("nodes", NODES.to_string()),
            ("quote_bytes", QUOTE_BYTES.to_string()),
            ("hex_calls", HEX_CALLS.to_string()),
            ("import_bytes", IMPORT_BYTES.to_string()),
        ],
    );
    match run(runs) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let reason = e.split(' ').next().unwrap_or("error").to_lowercase();
            eprintln!("{e}");
            speed::fail(&reason)
        }
    }
}
