//! Identity check for the catalog loader: `Corpus::load`,
//! `Corpus::load_with_threads` (0, 1, 2, 3, 8 and 64 threads) and
//! `EncodedCorpus::load` against `reference`, an exact copy of the earlier
//! loader that computed every passage digest in row order on one thread.
//!
//! Inputs are generated from fixed seeds: valid catalogs (small, large,
//! multibyte, overlapping passages), a digest mismatch in every row combined
//! with a second error of each kind in every row, several mismatches, the
//! record limit, random byte changes of catalogs and texts with both pins
//! recomputed, and rows that repeat a few passage ranges (each hashed once
//! by the current loader) with a wrong digest in one copy. A loaded corpus is compared field by field through its full
//! `Debug` form; an error is compared by value, so the reported error must be
//! the one the row order reaches first.
//!
//! Output: `IDENTITY name=<case> cases=<n> equal=<n>` per case, then exactly
//! one `IDENTITY_VERDICT=PASS|FAIL`. Exit status 0 on PASS, 1 on FAIL. No
//! timing. Output grammar: `docs/SPEED.md`.
use gel_source::{digest, hex, Address, Corpus, CorpusBuilder, Error, Hash};

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// The earlier loader, unchanged except that it returns its own `Corpus`
/// with the same fields in the same order, so both `Debug` forms agree.
pub mod reference {
    use gel_source::{
        digest, Address, Error, Hash, Record, MAX_CATALOG, MAX_PASSAGE, MAX_RECORDS, MAX_TEXT,
    };
    use std::collections::BTreeMap;

    // The fields are read only through the derived `Debug` form.
    #[allow(dead_code)]
    #[derive(Debug)]
    pub struct Corpus {
        root: Hash,
        text_hash: Hash,
        text: String,
        records: BTreeMap<Address, Record>,
        titles: BTreeMap<String, Vec<u64>>,
    }
    fn unhex(s: &str) -> Result<Vec<u8>, Error> {
        if s.len() % 2 != 0
            || !s
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::Format);
        }
        s.as_bytes()
            .chunks_exact(2)
            .map(|p| {
                let n = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
                Ok(n(p[0]) * 16 + n(p[1]))
            })
            .collect()
    }
    fn number(s: &str) -> Result<u64, Error> {
        let v: u64 = s.parse().map_err(|_| Error::Format)?;
        if v.to_string() != s {
            return Err(Error::Format);
        }
        Ok(v)
    }
    fn label(s: &str) -> Result<String, Error> {
        if s.len() > 1024 {
            return Err(Error::Limit);
        }
        let v = String::from_utf8(unhex(s)?).map_err(|_| Error::Format)?;
        if v.trim().is_empty() || v.chars().any(char::is_control) {
            return Err(Error::Format);
        }
        Ok(v)
    }
    impl Corpus {
        pub fn load(
            catalog: &[u8],
            text: &[u8],
            catalog_pin: Hash,
            text_pin: Hash,
        ) -> Result<Self, Error> {
            if catalog.len() > MAX_CATALOG || text.len() > MAX_TEXT {
                return Err(Error::Limit);
            }
            if digest(catalog) != catalog_pin || digest(text) != text_pin {
                return Err(Error::Integrity);
            }
            Self::load_verified(catalog, text, catalog_pin, text_pin)
        }
        pub fn load_verified(
            catalog: &[u8],
            text: &[u8],
            catalog_pin: Hash,
            text_pin: Hash,
        ) -> Result<Self, Error> {
            if catalog.len() > MAX_CATALOG || text.len() > MAX_TEXT {
                return Err(Error::Limit);
            }
            let raw = std::str::from_utf8(catalog).map_err(|_| Error::Format)?;
            let text = std::str::from_utf8(text).map_err(|_| Error::Format)?;
            if !raw.starts_with("GELCORPUS082\n") || !raw.ends_with('\n') || raw.contains('\r') {
                return Err(Error::Format);
            }
            let mut records = BTreeMap::new();
            let mut titles: BTreeMap<String, Vec<u64>> = BTreeMap::new();
            let mut nodes = BTreeMap::new();
            for line in raw[13..].split_terminator('\n') {
                if records.len() >= MAX_RECORDS {
                    return Err(Error::Limit);
                }
                let f: Vec<_> = line.splitn(9, '\t').collect();
                if f.len() != 8 {
                    return Err(Error::Format);
                }
                let address = Address {
                    node: number(f[0])?,
                    role: number(f[1])?,
                };
                let entry = number(f[2])?;
                let start = usize::try_from(number(f[3])?).map_err(|_| Error::Limit)?;
                let len = usize::try_from(number(f[4])?).map_err(|_| Error::Limit)?;
                if len == 0 || len > MAX_PASSAGE {
                    return Err(Error::Limit);
                }
                let end = start.checked_add(len).ok_or(Error::Limit)?;
                let quote = text.get(start..end).ok_or(Error::Format)?;
                if f[5].len() != 64 {
                    return Err(Error::Format);
                }
                let hash: Hash = unhex(f[5])?.try_into().map_err(|_| Error::Format)?;
                if digest(quote.as_bytes()) != hash {
                    return Err(Error::Integrity);
                }
                let title = label(f[6])?;
                let section = label(f[7])?;
                match nodes.entry(address.node) {
                    std::collections::btree_map::Entry::Occupied(old) => {
                        if old.get() != &(entry, title.clone()) {
                            return Err(Error::Format);
                        }
                    }
                    std::collections::btree_map::Entry::Vacant(slot) => {
                        slot.insert((entry, title.clone()));
                        titles
                            .entry(title.trim().to_lowercase())
                            .or_default()
                            .push(address.node);
                    }
                }
                let record = Record {
                    address,
                    entry,
                    start,
                    end,
                    hash,
                    title,
                    section,
                };
                if records.insert(address, record).is_some() {
                    return Err(Error::Duplicate);
                }
            }
            if records.is_empty() {
                return Err(Error::Format);
            }
            let mut binding = b"GELCORPUS082\0".to_vec();
            binding.extend_from_slice(&catalog_pin);
            binding.extend_from_slice(&text_pin);
            Ok(Self {
                root: digest(&binding),
                text_hash: text_pin,
                text: text.to_owned(),
                records,
                titles,
            })
        }
    }
}

// `Scale::Small`: fewer and smaller inputs, for the test wrapper in debug
// builds. `Scale::Full`: the verdict run.

/// Comparisons of one case, with the reference outcomes it reached.
pub struct Case {
    pub name: &'static str,
    pub cases: u64,
    pub equal: u64,
    /// Reference outcomes seen: ok, limit, integrity, format, duplicate.
    pub outcomes: [u64; 5],
}
impl Case {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            cases: 0,
            equal: 0,
            outcomes: [0; 5],
        }
    }
    fn check(&mut self, equal: bool) {
        self.cases += 1;
        if equal {
            self.equal += 1;
        }
    }
    fn saw(&mut self, old: &Result<reference::Corpus, Error>) {
        let k = match old {
            Ok(_) => 0,
            Err(Error::Limit) => 1,
            Err(Error::Integrity) => 2,
            Err(Error::Format) => 3,
            Err(_) => 4,
        };
        self.outcomes[k] += 1;
    }
}

/// One stderr line per case: how many inputs ended in each reference outcome.
pub fn coverage(tallies: &[Case]) -> Vec<String> {
    tallies
        .iter()
        .map(|t| {
            let [ok, limit, integrity, format, duplicate] = t.outcomes;
            format!(
                "COVERAGE name={} ok={ok} limit={limit} integrity={integrity} format={format} duplicate={duplicate}",
                t.name
            )
        })
        .collect()
}

/// SplitMix64: a fixed sequence for every seed.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const THREADS: [usize; 6] = [0, 1, 2, 3, 8, 64];

fn same(old: &Result<reference::Corpus, Error>, new: &Result<Corpus, Error>) -> bool {
    match (old, new) {
        (Ok(a), Ok(b)) => format!("{a:?}") == format!("{b:?}"),
        (Err(a), Err(b)) => a == b,
        _ => false,
    }
}

/// Compares the reference with `load` and every `load_with_threads` count.
fn compare(tally: &mut Case, catalog: &[u8], text: &[u8], catalog_pin: Hash, text_pin: Hash) {
    let old = reference::Corpus::load(catalog, text, catalog_pin, text_pin);
    tally.saw(&old);
    tally.check(same(
        &old,
        &Corpus::load(catalog, text, catalog_pin, text_pin),
    ));
    for threads in THREADS {
        tally.check(same(
            &old,
            &Corpus::load_with_threads(catalog, text, catalog_pin, text_pin, threads),
        ));
    }
}

/// `compare` with both pins computed from the bytes.
fn compare_pinned(tally: &mut Case, catalog: &[u8], text: &[u8]) {
    compare(tally, catalog, text, digest(catalog), digest(text));
}

const WORDS: [&str; 10] = [
    "gel", "source", "exact", "quote", "żółw", "łąka", "über", "naïve", "line\n", "x",
];
const TITLES: [&str; 5] = ["Alpha", "beta", " Alpha ", "Gamma ray", "Zeta"];

/// Text of at least one byte and at most `max` bytes, on character boundaries.
fn passage(rng: &mut Rng, max: usize, multibyte: bool) -> String {
    let target = 1 + rng.below(max);
    let mut s = String::new();
    while s.len() < target {
        let w = if multibyte {
            WORDS[rng.below(WORDS.len())]
        } else {
            WORDS[[0, 1, 2, 3, 8, 9][rng.below(6)]]
        };
        s.push_str(w);
        s.push(' ');
    }
    let mut end = target.min(s.len());
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        end = s.chars().next().map_or(1, char::len_utf8);
    }
    s.truncate(end);
    s
}

/// A valid corpus from the public builder; nodes repeat so titles and
/// entries are shared, and distinct nodes share titles.
fn build(rng: &mut Rng, records: usize, max: usize, multibyte: bool) -> gel_source::EncodedCorpus {
    let mut b = CorpusBuilder::new();
    let nodes = 1 + records / 3;
    for role in 0..records as u64 {
        let node = rng.below(nodes) as u64;
        let title = TITLES[node as usize % TITLES.len()];
        let section = format!("Part {role}");
        let quote = passage(rng, max, multibyte);
        b.push(Address { node, role }, node % 7, title, &section, &quote)
            .unwrap();
    }
    b.finish().unwrap()
}

/// Catalog rows after the header, each split into its eight fields.
fn rows(catalog: &[u8]) -> Vec<Vec<String>> {
    let raw = std::str::from_utf8(catalog).unwrap();
    raw[13..]
        .split_terminator('\n')
        .map(|l| l.split('\t').map(str::to_owned).collect())
        .collect()
}
fn join(rows: &[Vec<String>]) -> Vec<u8> {
    let mut s = String::from("GELCORPUS082\n");
    for r in rows {
        s.push_str(&r.join("\t"));
        s.push('\n');
    }
    s.into_bytes()
}

/// Changes one hex digit of the stated digest: valid hex, wrong value.
fn mismatch(row: &mut [String]) {
    let mut h: Vec<u8> = row[5].clone().into_bytes();
    h[7] = if h[7] == b'0' { b'1' } else { b'0' };
    row[5] = String::from_utf8(h).unwrap();
}

/// A second error of `kind` in row `i`; kinds 0..=5 come before the row's
/// digest in the earlier order, 6..=9 after it.
const KINDS: usize = 10;
fn break_row(rows: &mut [Vec<String>], i: usize, kind: usize, text_len: usize) {
    match kind {
        0 => rows[i][0] = format!("0{}", rows[i][0]),
        1 => rows[i][4] = "0".into(),
        2 => rows[i][3] = text_len.to_string(),
        3 => rows[i][5] = "g".repeat(64),
        4 => rows[i][5].truncate(62),
        5 => rows[i][7].push_str("\textra"),
        6 => rows[i][6] = "zz".into(),
        7 => rows[i][7] = hex(b"   "),
        8 => {
            // Node of row 0 with a different entry (row 0 itself: plain change).
            rows[i][0] = rows[0][0].clone();
            rows[i][2] = (rows[0][2].parse::<u64>().unwrap() + 1).to_string();
        }
        _ => {
            // Duplicate address of the previous row, consistent node fields.
            let p = if i == 0 { rows.len() - 1 } else { i - 1 };
            for f in [0, 1, 2, 6] {
                rows[i][f] = rows[p][f].clone();
            }
        }
    }
}

fn valid(scale: Scale, rng: &mut Rng) -> Case {
    let mut t = Case::new("valid");
    let shapes: &[(usize, usize, bool)] = match scale {
        Scale::Small => &[(1, 40, false), (17, 300, true), (40, 2000, true)],
        Scale::Full => &[
            (1, 40, false),
            (1, 32_768, true),
            (15, 300, false),
            (16, 300, true),
            (17, 300, true),
            (33, 5000, true),
            (200, 32_768, true),
            (300, 32_768, false),
            (5000, 400, true),
        ],
    };
    for &(records, max, multibyte) in shapes {
        for _ in 0..3 {
            let e = build(rng, records, max, multibyte);
            compare(
                &mut t,
                e.catalog_bytes(),
                e.text_bytes(),
                e.catalog_pin(),
                e.text_pin(),
            );
        }
    }
    t
}

fn encoded(scale: Scale, rng: &mut Rng) -> Case {
    let mut t = Case::new("encoded_load");
    let shapes: &[(usize, usize)] = match scale {
        Scale::Small => &[(1, 50), (40, 3000)],
        Scale::Full => &[
            (1, 50),
            (16, 32_768),
            (64, 32_768),
            (300, 32_768),
            (4000, 600),
        ],
    };
    for &(records, max) in shapes {
        let e = build(rng, records, max, true);
        let old = reference::Corpus::load_verified(
            e.catalog_bytes(),
            e.text_bytes(),
            e.catalog_pin(),
            e.text_pin(),
        );
        t.saw(&old);
        t.check(same(&old, &e.load()));
    }
    for size in [1usize, 40_000, 3 << 20] {
        if scale == Scale::Small && size > 40_000 {
            continue;
        }
        let text = passage(rng, size, true);
        let e = gel_source::import_text(text.as_bytes(), "Imported").unwrap();
        let old = reference::Corpus::load_verified(
            e.catalog_bytes(),
            e.text_bytes(),
            e.catalog_pin(),
            e.text_pin(),
        );
        t.saw(&old);
        t.check(same(&old, &e.load()));
    }
    t
}

/// Hand-written rows whose passages overlap and start anywhere, including
/// inside a multibyte character (an error) and at the end of the text.
fn overlap(scale: Scale, rng: &mut Rng) -> Case {
    let mut t = Case::new("overlap");
    let inputs = if scale == Scale::Small { 20 } else { 200 };
    for _ in 0..inputs {
        let multibyte = rng.below(2) == 0;
        let text = passage(rng, 200_000, multibyte);
        let n = 1 + rng.below(120);
        let mut s = String::from("GELCORPUS082\n");
        for role in 0..n {
            let start = rng.below(text.len());
            let len = 1 + rng.below((text.len() - start).min(32_768));
            let hash = text
                .as_bytes()
                .get(start..start + len)
                .map_or([0; 32], digest);
            s.push_str(&format!(
                "{}\t{role}\t1\t{start}\t{len}\t{}\t{}\t{}\n",
                rng.below(4),
                hex(&hash),
                hex(b"Same"),
                hex(format!("S{role}").as_bytes())
            ));
        }
        compare_pinned(&mut t, s.as_bytes(), text.as_bytes());
    }
    t
}

fn edges(rng: &mut Rng) -> Case {
    let mut t = Case::new("edges");
    let e = build(rng, 20, 100, true);
    let cat = e.catalog_bytes();
    let text = e.text_bytes();
    let mut no_newline = cat.to_vec();
    no_newline.pop();
    let mut cr = cat.to_vec();
    cr.insert(20, b'\r');
    let inputs: Vec<(Vec<u8>, Vec<u8>)> = vec![
        (b"GELCORPUS082\n".to_vec(), text.to_vec()),
        (b"GELCORPUS081\n".to_vec(), text.to_vec()),
        (Vec::new(), text.to_vec()),
        (no_newline, text.to_vec()),
        (cr, text.to_vec()),
        (cat.to_vec(), Vec::new()),
        (cat.to_vec(), vec![0xff; 64]),
        (vec![0xff; 64], text.to_vec()),
        (cat.to_vec(), text[..text.len() / 2].to_vec()),
    ];
    for (c, x) in &inputs {
        compare_pinned(&mut t, c, x);
    }
    // Wrong pins and oversize inputs are rejected before any row.
    compare(&mut t, cat, text, [0; 32], digest(text));
    compare(&mut t, cat, text, digest(cat), [0; 32]);
    let big = vec![b'a'; gel_source::MAX_TEXT + 1];
    compare(&mut t, cat, &big, digest(cat), [0; 32]);
    let big = vec![b'a'; gel_source::MAX_CATALOG + 1];
    compare(&mut t, &big, text, [0; 32], digest(text));
    t
}

/// A digest mismatch in row j and every other error kind in row i, for
/// every pair; rows reach past several worker blocks.
fn ordered(scale: Scale, rng: &mut Rng) -> (Case, Case) {
    let mut first = Case::new("mismatch_single");
    let mut pair = Case::new("mismatch_and_error");
    let n = if scale == Scale::Small { 40 } else { 56 };
    let e = build(rng, n, 64, true);
    let text = e.text_bytes();
    let base = rows(e.catalog_bytes());
    let positions: Vec<usize> = match scale {
        Scale::Small => vec![0, 1, 15, 16, 17, 31, 32, 39],
        Scale::Full => (0..n).collect(),
    };
    for &j in &positions {
        let mut r = base.clone();
        mismatch(&mut r[j]);
        compare_pinned(&mut first, &join(&r), text);
        for &i in &positions {
            for kind in 0..KINDS {
                let mut r = base.clone();
                mismatch(&mut r[j]);
                break_row(&mut r, i, kind, text.len());
                compare_pinned(&mut pair, &join(&r), text);
            }
        }
    }
    // Each error kind alone, in every position.
    for &i in &positions {
        for kind in 0..KINDS {
            let mut r = base.clone();
            break_row(&mut r, i, kind, text.len());
            compare_pinned(&mut pair, &join(&r), text);
        }
    }
    (first, pair)
}

/// Several mismatches, optionally with one more error, on a catalog large
/// enough for the automatic path to use threads.
fn several(scale: Scale, rng: &mut Rng) -> Case {
    let mut t = Case::new("mismatch_several");
    let (n, inputs) = if scale == Scale::Small {
        (60, 30)
    } else {
        (400, 300)
    };
    let e = build(rng, n, 8000, true);
    let text = e.text_bytes();
    let base = rows(e.catalog_bytes());
    for _ in 0..inputs {
        let mut r = base.clone();
        for _ in 0..1 + rng.below(4) {
            let j = rng.below(n);
            mismatch(&mut r[j]);
        }
        if rng.below(2) == 0 {
            let i = rng.below(n);
            break_row(&mut r, i, rng.below(KINDS), text.len());
        }
        compare_pinned(&mut t, &join(&r), text);
    }
    t
}

/// Rows that repeat a few passage ranges, in row order and out of it: all
/// digests true, a wrong digest in one copy of a range, and that combined
/// with every other error kind in an earlier or a later row.
fn repeated(scale: Scale, rng: &mut Rng) -> Case {
    let mut t = Case::new("repeated_ranges");
    let n = if scale == Scale::Small { 40 } else { 120 };
    let e = build(rng, n, 3000, true);
    let text = e.text_bytes();
    let base = rows(e.catalog_bytes());
    let inputs = if scale == Scale::Small { 60 } else { 600 };
    for input in 0..inputs {
        let distinct = 1 + rng.below(5);
        let mut r = base.clone();
        for (k, row) in r.iter_mut().enumerate() {
            // Sorted repeats for some inputs, scattered ones for the others.
            let from = if input % 3 == 0 {
                k * distinct / n
            } else {
                rng.below(distinct)
            };
            for f in [3, 4, 5] {
                row[f] = base[from][f].clone();
            }
        }
        compare_pinned(&mut t, &join(&r), text);
        let j = rng.below(n);
        if input % 2 == 0 {
            mismatch(&mut r[j]);
        } else {
            // A wrong digest that sorts after every true one.
            r[j][5] = "f".repeat(64);
        }
        compare_pinned(&mut t, &join(&r), text);
        let i = rng.below(n);
        break_row(&mut r, i, rng.below(KINDS), text.len());
        compare_pinned(&mut t, &join(&r), text);
    }
    t
}

/// The record limit: exactly the maximum, one more, and mismatches before it.
fn limit(scale: Scale) -> Case {
    let mut t = Case::new("record_limit");
    let text: String = (0..64).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
    let row = |role: usize, hash: Hash| {
        let start = role % 64;
        format!(
            "{}\t{role}\t0\t{start}\t1\t{}\t{}\t{}\n",
            role % 1000,
            hex(&hash),
            hex(b"T"),
            hex(b"S")
        )
    };
    let total = gel_source::MAX_RECORDS + 1;
    let catalog_of = |count: usize, bad: Option<usize>| {
        let mut s = String::from("GELCORPUS082\n");
        for role in 0..count {
            let start = role % 64;
            let mut hash = digest(&text.as_bytes()[start..start + 1]);
            if bad == Some(role) {
                hash[0] ^= 1;
            }
            s.push_str(&row(role, hash));
        }
        s.into_bytes()
    };
    let cases: &[(usize, Option<usize>)] = match scale {
        Scale::Small => &[(total, Some(total - 2))],
        Scale::Full => &[
            (total - 1, None),
            (total, None),
            (total, Some(total - 2)),
            (total, Some(10)),
            (total, Some(total - 1)),
        ],
    };
    for &(count, bad) in cases {
        let catalog = catalog_of(count, bad);
        compare_pinned(&mut t, &catalog, text.as_bytes());
    }
    t
}

/// Random byte changes of a catalog and of a text, pins recomputed.
fn random_bytes(scale: Scale, rng: &mut Rng) -> (Case, Case) {
    let mut cat_t = Case::new("random_catalog_bytes");
    let mut text_t = Case::new("random_text_bytes");
    let (n, max, inputs) = if scale == Scale::Small {
        (40, 2000, 60)
    } else {
        (120, 20_000, 1500)
    };
    let e = build(rng, n, max, true);
    let cat = e.catalog_bytes();
    let text = e.text_bytes();
    const ALPHABET: &[u8] = b"0123456789abcdefg\t\n \r\xc3\xa9-";
    for _ in 0..inputs {
        let mut c = cat.to_vec();
        for _ in 0..1 + rng.below(3) {
            let k = 13 + rng.below(c.len() - 13);
            c[k] = ALPHABET[rng.below(ALPHABET.len())];
        }
        compare_pinned(&mut cat_t, &c, text);
        let mut x = text.to_vec();
        for _ in 0..1 + rng.below(3) {
            let k = rng.below(x.len());
            x[k] = if rng.below(4) == 0 {
                0xff
            } else {
                b'a' + rng.below(26) as u8
            };
        }
        compare_pinned(&mut text_t, cat, &x);
    }
    (cat_t, text_t)
}

/// Every comparison, one tally per case. At full scale the reference outcomes
/// of every case are printed on stderr (`COVERAGE` lines).
pub fn run(scale: Scale) -> Vec<Tally> {
    let cases = run_cases(scale);
    if scale == Scale::Full {
        for line in coverage(&cases) {
            eprintln!("{line}");
        }
    }
    let count = |n: u64| usize::try_from(n).expect("case count fits usize");
    cases
        .iter()
        .map(|c| Tally::counted(c.name, count(c.cases), count(c.equal)))
        .collect()
}

/// Every comparison with its reference outcomes.
pub fn run_cases(scale: Scale) -> Vec<Case> {
    let mut rng = Rng(0x5eed_c0de_2026_1020);
    let mut out = vec![
        valid(scale, &mut rng),
        encoded(scale, &mut rng),
        overlap(scale, &mut rng),
        edges(&mut rng),
    ];
    let (a, b) = ordered(scale, &mut rng);
    out.push(a);
    out.push(b);
    out.push(several(scale, &mut rng));
    out.push(limit(scale));
    let (c, d) = random_bytes(scale, &mut rng);
    out.push(c);
    out.push(d);
    out.push(repeated(scale, &mut rng));
    out
}

fn main() -> std::process::ExitCode {
    identity::report(&run(Scale::Full))
}
