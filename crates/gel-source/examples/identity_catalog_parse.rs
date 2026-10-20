//! Identity check for the catalog reader (`Corpus::load`, `EncodedCorpus::load`),
//! the catalog writer (`CorpusBuilder::push`/`finish`) and `hex`, against
//! exact copies of the earlier code, on seeded and exhaustive inputs including
//! malformed ones. Every comparison requires the same `Result`: on success the
//! same records, root, text pin, text and title lookups; on failure the same
//! `Error` variant.
//!
//! `import_text` is unchanged; its body is copied here onto the earlier
//! writer, so `import_text_parts` compares the current writer and reader with
//! the earlier ones on texts from one part up to `MAX_TEXT`.
//!
//! Usage: `cargo run --release -p gel-source --example identity_catalog_parse`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when no case
//! ran or a case ran no comparison). Exit status 0 on PASS, 1 on FAIL. The
//! integration test `tests/identity_catalog_parse.rs` runs the same comparison
//! at a smaller scale. Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_source::{
    digest, hex, import_text, Address, Corpus, CorpusBuilder, EncodedCorpus, Error, Hash,
    MAX_PASSAGE, MAX_TEXT,
};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// Exact copies of the earlier catalog code.
mod reference {
    use super::{digest, Address, Error, Hash};
    use gel_source::{Record, MAX_CATALOG, MAX_PASSAGE, MAX_RECORDS, MAX_TEXT};
    use std::collections::{BTreeMap, BTreeSet};

    pub fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            write!(s, "{b:02x}").unwrap();
        }
        s
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

    pub struct Corpus {
        pub root: Hash,
        pub text_hash: Hash,
        pub text: String,
        pub records: BTreeMap<Address, Record>,
        pub titles: BTreeMap<String, Vec<u64>>,
    }
    pub fn load(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
    ) -> Result<Corpus, Error> {
        if catalog.len() > MAX_CATALOG || text.len() > MAX_TEXT {
            return Err(Error::Limit);
        }
        if digest(catalog) != catalog_pin || digest(text) != text_pin {
            return Err(Error::Integrity);
        }
        load_verified(catalog, text, catalog_pin, text_pin)
    }
    fn load_verified(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
    ) -> Result<Corpus, Error> {
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
        Ok(Corpus {
            root: digest(&binding),
            text_hash: text_pin,
            text: text.to_owned(),
            records,
            titles,
        })
    }

    pub struct CorpusBuilder {
        text: Vec<u8>,
        catalog: String,
        addresses: BTreeSet<Address>,
        nodes: BTreeMap<u64, (u64, String)>,
    }
    pub struct Encoded {
        pub text: Vec<u8>,
        pub catalog: Vec<u8>,
        pub text_pin: Hash,
        pub catalog_pin: Hash,
    }
    impl CorpusBuilder {
        pub fn new() -> Self {
            Self {
                text: Vec::new(),
                catalog: "GELCORPUS082\n".into(),
                addresses: BTreeSet::new(),
                nodes: BTreeMap::new(),
            }
        }
        pub fn push(
            &mut self,
            address: Address,
            entry: u64,
            title: &str,
            section: &str,
            quote: &str,
        ) -> Result<(), Error> {
            for label in [title, section] {
                if label.len() > 512 {
                    return Err(Error::Limit);
                }
                if label.trim().is_empty() || label.chars().any(char::is_control) {
                    return Err(Error::Format);
                }
            }
            if quote.is_empty() || quote.len() > MAX_PASSAGE || self.addresses.len() >= MAX_RECORDS
            {
                return Err(Error::Limit);
            }
            if self.addresses.contains(&address) {
                return Err(Error::Duplicate);
            }
            if self
                .nodes
                .get(&address.node)
                .is_some_and(|(old_entry, old_title)| *old_entry != entry || old_title != title)
            {
                return Err(Error::Format);
            }
            let end = self
                .text
                .len()
                .checked_add(quote.len())
                .ok_or(Error::Limit)?;
            if end > MAX_TEXT {
                return Err(Error::Limit);
            }
            let row = format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                address.node,
                address.role,
                entry,
                self.text.len(),
                quote.len(),
                hex(&digest(quote.as_bytes())),
                hex(title.as_bytes()),
                hex(section.as_bytes())
            );
            if self
                .catalog
                .len()
                .checked_add(row.len())
                .ok_or(Error::Limit)?
                > MAX_CATALOG
            {
                return Err(Error::Limit);
            }
            self.text.extend_from_slice(quote.as_bytes());
            self.catalog.push_str(&row);
            self.addresses.insert(address);
            self.nodes
                .entry(address.node)
                .or_insert_with(|| (entry, title.to_owned()));
            Ok(())
        }
        /// Not in the earlier code: lets the check see how full the catalog is.
        pub fn catalog_len(&self) -> usize {
            self.catalog.len()
        }
        pub fn finish(self) -> Result<Encoded, Error> {
            if self.addresses.is_empty() {
                return Err(Error::Format);
            }
            Ok(Encoded {
                text_pin: digest(&self.text),
                catalog_pin: digest(self.catalog.as_bytes()),
                text: self.text,
                catalog: self.catalog.into_bytes(),
            })
        }
    }
    /// The body of the unchanged `import_text`, filling the earlier writer.
    pub fn import_text(bytes: &[u8], title: &str) -> Result<Encoded, Error> {
        if bytes.is_empty() || bytes.len() > MAX_TEXT {
            return Err(Error::Limit);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Error::Format)?;
        let mut builder = CorpusBuilder::new();
        let mut start = 0;
        let mut part = 1;
        while start < text.len() {
            let mut end = (start + MAX_PASSAGE).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            builder.push(
                Address {
                    node: 1,
                    role: part,
                },
                1,
                title,
                &format!("Lead ({part})"),
                &text[start..end],
            )?;
            start = end;
            part += 1;
        }
        builder.finish()
    }
}

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
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

// ---- Comparison of the current reader with the reference ----

fn same_corpus(a: &Corpus, b: &reference::Corpus) -> bool {
    a.root() == b.root
        && a.text_hash() == b.text_hash
        && a.source_text() == b.text
        && a.records().eq(b.records.values())
        && b.titles
            .iter()
            .all(|(key, nodes)| a.find_title(key) == nodes.as_slice())
}

/// Which outcome a load produced: 0 = Ok, then one slot per `Error` variant.
fn outcome<T>(r: &Result<T, Error>) -> usize {
    match r {
        Ok(_) => 0,
        Err(Error::Limit) => 1,
        Err(Error::Integrity) => 2,
        Err(Error::Format) => 3,
        Err(Error::Duplicate) => 4,
        Err(Error::Missing) => 5,
        Err(Error::Stale) => 6,
    }
}

/// Loads with both readers; returns (equal, outcome of the current reader).
fn compare_load(catalog: &[u8], text: &[u8], catalog_pin: Hash, text_pin: Hash) -> (bool, usize) {
    let new = Corpus::load(catalog, text, catalog_pin, text_pin);
    let old = reference::load(catalog, text, catalog_pin, text_pin);
    let kind = outcome(&new);
    let equal = match (&new, &old) {
        (Ok(a), Ok(b)) => same_corpus(a, b),
        (Err(a), Err(b)) => a == b,
        _ => false,
    };
    (equal, kind)
}

fn compare_pinned(catalog: &[u8], text: &[u8]) -> (bool, usize) {
    compare_load(catalog, text, digest(catalog), digest(text))
}

/// One catalog row from its fields, formatted independently of the code under test.
fn row(fields: &[&str; 8]) -> String {
    let mut s = fields.join("\t");
    s.push('\n');
    s
}

fn catalog(rows: &[String]) -> Vec<u8> {
    let mut s = String::from("GELCORPUS082\n");
    for r in rows {
        s.push_str(r);
    }
    s.into_bytes()
}

// ---- Cases ----

/// Every string of length 0..=3 over [0-9 + - a] and the u64 boundaries, in
/// each of the five numeric fields of the second of two rows.
fn number_exhaustive(t: &mut Tally, kinds: &mut [bool; 7]) {
    let text = "x".repeat(2048);
    let quote_hash = reference::hex(&digest(b"xxxx"));
    let title = reference::hex(b"Title");
    let section = reference::hex(b"Section");
    let first = row(&["3", "4", "5", "0", "4", &quote_hash, &title, &section]);
    let alphabet = [
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "-", "a",
    ];
    let mut strings = vec![String::new()];
    let mut level = vec![String::new()];
    for _ in 0..3 {
        level = level
            .iter()
            .flat_map(|p| alphabet.iter().map(move |c| format!("{p}{c}")))
            .collect();
        strings.extend(level.iter().cloned());
    }
    for edge in [
        "18446744073709551615",
        "18446744073709551616",
        "18446744073709551614",
        "99999999999999999999",
        "100000000000000000000",
        "018446744073709551615",
        "+18446744073709551615",
        "9223372036854775807",
        "32768",
        "32769",
        "2044",
        "2045",
        "007",
        "00",
        "-0",
        "+0",
        " 5",
        "5 ",
        "\u{0661}",
        "\u{0661}\u{0662}",
        "\u{ff15}",
    ] {
        strings.push(edge.to_owned());
    }
    for field in 0..5 {
        for s in &strings {
            let mut fields = ["3", "5", "5", "1", "4", &quote_hash, &title, &section];
            fields[field] = s;
            let cat = catalog(&[first.clone(), row(&fields)]);
            let (equal, kind) = compare_pinned(&cat, text.as_bytes());
            kinds[kind] = true;
            t.check(equal);
        }
    }
}

fn hex_bytes(t: &mut Tally, scale: Scale) {
    for b in 0..=255u8 {
        t.check(hex(&[b]) == reference::hex(&[b]));
    }
    let mut rng = Rng(0x6865_7862_7974_6573);
    let rounds = if scale == Scale::Full { 20_000 } else { 2_000 };
    for _ in 0..rounds {
        let len = rng.below(301);
        let bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        t.check(hex(&bytes) == reference::hex(&bytes));
    }
    t.check(hex(&[]) == reference::hex(&[]));
}

fn hash_field(t: &mut Tally, kinds: &mut [bool; 7]) {
    let text = "Zażółć gęślą jaźń 🦀 exact quote";
    // "Zażół": the first 8 bytes end on a character boundary.
    let good = reference::hex(&digest(&text.as_bytes()[..8]));
    let title = reference::hex(b"T");
    let section = reference::hex(b"S");
    let replacements = [
        "0", "9", "a", "f", "A", "F", "g", "+", "-", " ", "\u{105}", "\t", "", "00",
    ];
    for at in 0..64 {
        for r in replacements {
            let mut hash = good.clone();
            hash.replace_range(at..at + 1, r);
            let cat = catalog(&[row(&["1", "1", "1", "0", "8", &hash, &title, &section])]);
            let (equal, kind) = compare_pinned(&cat, text.as_bytes());
            kinds[kind] = true;
            t.check(equal);
        }
    }
}

fn labels(t: &mut Tally, kinds: &mut [bool; 7]) {
    let good = reference::hex(b"Title");
    let mut variants: Vec<String> = vec![
        good.clone(),
        good[..good.len() - 1].to_owned(),
        good.to_uppercase(),
        format!("{good}g0"),
        String::new(),
        reference::hex(&[0xc3, 0x28]),
        reference::hex(&[0xff]),
        reference::hex(b"\x07bell"),
        reference::hex(b"a\nb"),
        reference::hex(b"a\tb"),
        reference::hex(b"   "),
        reference::hex(" \u{3000} ".as_bytes()),
        reference::hex("\u{85}".as_bytes()),
        reference::hex("Łódź".as_bytes()),
        reference::hex("łÓDŹ ".as_bytes()),
        reference::hex(&[b'a'; 512]),
        reference::hex(&[b'a'; 513]),
    ];
    let mut odd = reference::hex(&[b'a'; 512]);
    odd.push('a');
    variants.push(odd);
    let text = "abcdefgh";
    let good_hash = reference::hex(&digest(b"abcd"));
    let bad_hash = reference::hex(&digest(b"abce"));
    for title in &variants {
        for section in &variants {
            for hash in [&good_hash, &bad_hash] {
                let rows = [
                    row(&["2", "1", "7", "0", "4", hash, title, section]),
                    row(&[
                        "2",
                        "2",
                        "7",
                        "4",
                        "4",
                        &reference::hex(&digest(b"efgh")),
                        &good,
                        section,
                    ]),
                ];
                let (equal, kind) = compare_pinned(&catalog(&rows), text.as_bytes());
                kinds[kind] = true;
                t.check(equal);
            }
        }
    }
}

const TEXT_PIECES: [&str; 9] = [
    "a",
    "b",
    " ",
    "\u{142}",
    "\u{17c}",
    "\u{1f980}",
    "\n",
    "q",
    "Z",
];
const TITLES: [&str; 6] = [
    "Alpha",
    " Beta ",
    "\u{141}\u{f3}d\u{17a}",
    "\u{142}\u{f3}d\u{17a}",
    "gamma delta",
    "BETA",
];
const SECTIONS: [&str; 4] = ["Lead (1)", "Lead (2)", "notes", "\u{1f980}"];

fn random_text(rng: &mut Rng, min: usize, spread: usize) -> String {
    let mut s = String::new();
    let len = min + rng.below(spread);
    while s.len() < len {
        let piece: &&str = rng.pick(&TEXT_PIECES);
        s.push_str(piece);
    }
    s
}

/// A seeded catalog over `text`: mostly well-formed, with shared nodes,
/// occasional conflicting labels, duplicates, out-of-range or split ranges.
fn random_catalog(rng: &mut Rng, text: &str) -> Vec<String> {
    let n = 1 + rng.below(40);
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let node = match rng.below(20) {
            0 => u64::MAX,
            _ => rng.below(6) as u64,
        };
        let role = match rng.below(20) {
            0 => u64::MAX,
            _ => rng.below(6) as u64,
        };
        let entry = if rng.below(10) == 0 {
            rng.below(3) as u64
        } else {
            node.wrapping_mul(7)
        };
        let title = if rng.below(10) == 0 {
            *rng.pick(&TITLES)
        } else {
            TITLES[(node % TITLES.len() as u64) as usize]
        };
        let section = *rng.pick(&SECTIONS);
        let mut start = rng.below(text.len());
        if rng.below(8) != 0 {
            while !text.is_char_boundary(start) {
                start -= 1;
            }
        }
        let mut len = 1 + rng.below(64);
        if rng.below(10) != 0 {
            len = len.min(text.len() - start).max(1);
            while start + len < text.len() && !text.is_char_boundary(start + len) {
                len += 1;
            }
        }
        let hash = match text.get(start..start + len) {
            Some(q) if rng.below(20) != 0 => digest(q.as_bytes()),
            _ => digest(&rng.next().to_le_bytes()),
        };
        // Rare length and range limits: empty, over-long, overflowing.
        let (start, len) = match rng.below(60) {
            0 => (start.to_string(), "0".to_owned()),
            1 => (start.to_string(), (MAX_PASSAGE + 1).to_string()),
            2 => (u64::MAX.to_string(), len.to_string()),
            _ => (start.to_string(), len.to_string()),
        };
        rows.push(row(&[
            &node.to_string(),
            &role.to_string(),
            &entry.to_string(),
            &start,
            &len,
            &reference::hex(&hash),
            &reference::hex(title.as_bytes()),
            &reference::hex(section.as_bytes()),
        ]));
    }
    rows
}

const MUTATION_BYTES: [u8; 15] = [
    b'0', b'9', b'a', b'f', b'A', b'\t', b'\n', b'\r', b'+', b'-', b' ', 0xc5, 0xff, b'x', 0x82,
];

fn mutate(rng: &mut Rng, cat: &mut Vec<u8>) {
    if cat.is_empty() {
        return;
    }
    match rng.below(6) {
        0 => {
            let at = rng.below(cat.len());
            cat[at] = *rng.pick(&MUTATION_BYTES);
        }
        1 => {
            let at = rng.below(cat.len());
            cat.remove(at);
        }
        2 => {
            let at = rng.below(cat.len() + 1);
            cat.insert(at, *rng.pick(&MUTATION_BYTES));
        }
        3 => {
            let at = rng.below(cat.len());
            cat.truncate(at);
        }
        4 => {
            // Repeat one whole line at the end.
            let lines: Vec<&[u8]> = cat.split_inclusive(|&b| b == b'\n').collect();
            let line = lines[rng.below(lines.len())].to_vec();
            cat.extend_from_slice(&line);
        }
        _ => {}
    }
}

fn catalog_fuzz(t: &mut Tally, kinds: &mut [bool; 7], scale: Scale) {
    let mut rng = Rng(0x6361_7461_6c6f_6731);
    let rounds = if scale == Scale::Full { 40_000 } else { 3_000 };
    for _ in 0..rounds {
        let text = random_text(&mut rng, 16, 400);
        let rows = random_catalog(&mut rng, &text);
        let mut cat = catalog(&rows);
        for _ in 0..rng.below(3) {
            mutate(&mut rng, &mut cat);
        }
        let (equal, kind) = if rng.below(50) == 0 {
            let mut pin = digest(&cat);
            pin[rng.below(32)] ^= 1;
            compare_load(&cat, text.as_bytes(), pin, digest(text.as_bytes()))
        } else {
            compare_pinned(&cat, text.as_bytes())
        };
        kinds[kind] = true;
        t.check(equal);
    }
}

fn max_records(t: &mut Tally, kinds: &mut [bool; 7]) {
    let hash = reference::hex(&digest(b"x"));
    let title = reference::hex(b"T");
    let section = reference::hex(b"S");
    let rows = |n: usize| -> Vec<String> {
        (0..n)
            .map(|i| row(&[&i.to_string(), "0", "0", "0", "1", &hash, &title, &section]))
            .collect()
    };
    let limit = gel_source::MAX_RECORDS;
    let garbage = "not a row\n".to_owned();
    let mut cases = vec![rows(limit), rows(limit + 1), rows(limit - 1)];
    let mut over = rows(limit);
    over.push(garbage.clone());
    cases.push(over);
    let mut under = rows(limit - 1);
    under.push(garbage);
    cases.push(under);
    for rows in cases {
        let (equal, kind) = compare_pinned(&catalog(&rows), b"x");
        kinds[kind] = true;
        t.check(equal);
    }
}

fn compare_finish(builder: CorpusBuilder, old: reference::CorpusBuilder) -> bool {
    compare_encoded(builder.finish(), old.finish())
}

/// The same `finish` result: on success the same bytes, pins and load result.
fn compare_encoded(
    new: Result<EncodedCorpus, Error>,
    old: Result<reference::Encoded, Error>,
) -> bool {
    match (new, old) {
        (Ok(a), Ok(b)) => {
            a.text_bytes() == b.text.as_slice()
                && a.catalog_bytes() == b.catalog.as_slice()
                && a.text_pin() == b.text_pin
                && a.catalog_pin() == b.catalog_pin
                && match (
                    a.load(),
                    reference::load(&b.catalog, &b.text, b.catalog_pin, b.text_pin),
                ) {
                    (Ok(x), Ok(y)) => same_corpus(&x, &y),
                    (Err(x), Err(y)) => x == y,
                    _ => false,
                }
        }
        (Err(a), Err(b)) => a == b,
        _ => false,
    }
}

fn builder_fuzz(t: &mut Tally, scale: Scale) {
    let mut rng = Rng(0x6275_696c_6465_7232);
    let long_ok = "\u{105}".repeat(256);
    let long_bad = "\u{105}".repeat(256) + "a";
    let labels: Vec<&str> = vec![
        "Alpha",
        " Beta ",
        "\u{141}\u{f3}d\u{17a}",
        &long_ok,
        &long_bad,
        " ",
        "",
        "A\tB",
        "A\nB",
        "A\u{0}B",
        "\u{85}",
        "Lead (1)",
    ];
    let max_quote = "m".repeat(MAX_PASSAGE);
    let over_quote = "m".repeat(MAX_PASSAGE + 1);
    let rounds = if scale == Scale::Full { 20_000 } else { 1_500 };
    for _ in 0..rounds {
        let mut builder = CorpusBuilder::new();
        let mut old = reference::CorpusBuilder::new();
        for _ in 0..1 + rng.below(30) {
            let node = match rng.below(15) {
                0 => u64::MAX,
                _ => rng.below(4) as u64,
            };
            let role = match rng.below(15) {
                0 => u64::MAX,
                _ => rng.below(4) as u64,
            };
            let entry = *rng.pick(&[0, 1, u64::MAX, node]);
            let title = if rng.below(6) == 0 {
                *rng.pick(&labels)
            } else {
                labels[(node % 3) as usize]
            };
            let section = *rng.pick(&labels);
            let text = random_text(&mut rng, 0, 80);
            let quote = match rng.below(200) {
                0 => max_quote.as_str(),
                1 => over_quote.as_str(),
                _ => text.as_str(),
            };
            let address = Address { node, role };
            let a = builder.push(address, entry, title, section, quote);
            let b = old.push(address, entry, title, section, quote);
            t.check(a == b);
        }
        t.check(compare_finish(builder, old));
    }
}

/// Rows with 512-byte labels until the catalog limit, then shorter sections
/// and numbers of every digit count around the exact boundary.
fn builder_catalog_limit(t: &mut Tally) {
    let mut builder = CorpusBuilder::new();
    let mut old = reference::CorpusBuilder::new();
    let title = "t".repeat(512);
    let long = "s".repeat(512);
    let mut role = 0u64;
    loop {
        let address = Address { node: 1, role };
        role += 1;
        let a = builder.push(address, 1, &title, &long, "quote");
        let b = old.push(address, 1, &title, &long, "quote");
        t.check(a == b);
        if a.is_err() || b.is_err() {
            break;
        }
    }
    // Every attempt uses a new seven-digit node with a one-byte title, so no
    // earlier row can make it a duplicate or a conflict. The row length is
    // 2 * len + digits + 88; trying the longest rows first, the first accepted
    // row fills the catalog to exactly its limit.
    for len in (1..=512u64).rev() {
        let section = "s".repeat(len as usize);
        for digits in (1..=20u32).rev() {
            let role = if digits == 20 {
                u64::MAX - len
            } else {
                let base = 10u64.pow(digits - 1);
                base + len % (9 * base)
            };
            let node = 1_000_000 + len * 32 + u64::from(digits);
            let address = Address { node, role };
            let a = builder.push(address, 1, "t", &section, "q");
            let b = old.push(address, 1, "t", &section, "q");
            t.check(a == b);
        }
    }
    t.check(old.catalog_len() == gel_source::MAX_CATALOG);
    t.check(compare_finish(builder, old));
}

/// `import_text` of `text` under each of `titles` against its body on the
/// earlier writer; `seen` records the outcome of the current code.
fn check_import(t: &mut Tally, seen: &mut [bool; 7], text: &[u8], titles: &[&str]) {
    for &title in titles {
        let new = import_text(text, title);
        seen[outcome(&new)] = true;
        t.check(compare_encoded(new, reference::import_text(text, title)));
    }
}

/// Exactly `len` bytes of ASCII words and newlines.
fn words(len: usize) -> Vec<u8> {
    let mut s = Vec::with_capacity(len + 32);
    while s.len() < len {
        s.extend_from_slice(b"gel source exact quote\n");
    }
    s.truncate(len);
    s
}

/// `import_text` on one part, both sides of a part boundary, multi-byte
/// characters across boundaries, bad UTF-8 and bad titles; at full scale also
/// 8 MiB (256 parts), 8 MiB + 1 (257), over 320 parts with start offsets
/// above 1 MiB, `MAX_TEXT` and `MAX_TEXT + 1`. The current code must reach
/// success, `Limit` and `Format`, or the comparisons prove less than they claim.
fn import_text_parts(t: &mut Tally, scale: Scale) {
    let max_title = "t".repeat(512);
    let long_title = "t".repeat(513);
    let titles = [
        "Speed",
        "\u{141}\u{f3}d\u{17a} \u{2014} notes",
        "",
        " ",
        "A\tB",
        max_title.as_str(),
        long_title.as_str(),
    ];
    let speed = &titles[..1];
    let m = |n: usize| "m".repeat(n);
    let mut seen = [false; 7];
    check_import(t, &mut seen, b"", speed);
    check_import(t, &mut seen, b"a", &titles);
    check_import(t, &mut seen, m(MAX_PASSAGE - 1).as_bytes(), speed);
    check_import(t, &mut seen, m(MAX_PASSAGE).as_bytes(), speed);
    check_import(t, &mut seen, m(MAX_PASSAGE + 1).as_bytes(), &titles);
    let split2 = m(MAX_PASSAGE - 1) + "\u{105}" + &m(10);
    check_import(t, &mut seen, split2.as_bytes(), speed);
    let split4 = m(MAX_PASSAGE - 2) + "\u{1f980}" + &m(MAX_PASSAGE);
    check_import(t, &mut seen, split4.as_bytes(), speed);
    let three_byte = "\u{20ac}".repeat(2 * MAX_PASSAGE + 1);
    check_import(t, &mut seen, three_byte.as_bytes(), speed);
    check_import(t, &mut seen, b"\xff", speed);
    let cut = m(MAX_PASSAGE + 7) + "\u{105}";
    check_import(t, &mut seen, &cut.as_bytes()[..cut.len() - 1], speed);
    let mut rng = Rng(0x696d_706f_7274_7331);
    let mixed = random_text(&mut rng, 5 * MAX_PASSAGE, MAX_PASSAGE);
    check_import(t, &mut seen, mixed.as_bytes(), speed);
    if scale == Scale::Full {
        check_import(t, &mut seen, &words(8 << 20), speed);
        check_import(t, &mut seen, &words((8 << 20) + 1), speed);
        let large = random_text(&mut rng, 10 << 20, 1 << 16);
        check_import(t, &mut seen, large.as_bytes(), speed);
        drop(large);
        check_import(t, &mut seen, &words(MAX_TEXT), speed);
        check_import(t, &mut seen, &words(MAX_TEXT + 1), speed);
    }
    // Slots 0, 1 and 3: success, Limit and Format.
    for kind in [0, 1, 3] {
        t.check(seen[kind]);
    }
}

pub fn run(scale: Scale) -> Vec<Tally> {
    let mut tallies = Vec::new();
    let mut kinds = [false; 7];

    let mut t = Tally::new("number_exhaustive");
    number_exhaustive(&mut t, &mut kinds);
    tallies.push(t);

    let mut t = Tally::new("hex_bytes");
    hex_bytes(&mut t, scale);
    tallies.push(t);

    let mut t = Tally::new("hash_field");
    hash_field(&mut t, &mut kinds);
    tallies.push(t);

    let mut t = Tally::new("labels");
    labels(&mut t, &mut kinds);
    tallies.push(t);

    let mut fuzz_kinds = [false; 7];
    let mut t = Tally::new("catalog_fuzz");
    catalog_fuzz(&mut t, &mut fuzz_kinds, scale);
    tallies.push(t);

    let mut t = Tally::new("max_records");
    max_records(&mut t, &mut kinds);
    tallies.push(t);

    let mut t = Tally::new("builder_fuzz");
    builder_fuzz(&mut t, scale);
    tallies.push(t);

    let mut t = Tally::new("builder_catalog_limit");
    builder_catalog_limit(&mut t);
    tallies.push(t);

    let mut t = Tally::new("import_text_parts");
    import_text_parts(&mut t, scale);
    tallies.push(t);

    // The seeded catalogs must reach success and every reader error, or the
    // comparisons above prove less than they claim. Slots 0..5 are Ok, Limit,
    // Integrity, Format and Duplicate; a load never returns Missing or Stale.
    let mut t = Tally::new("catalog_fuzz_outcomes");
    for &seen in &fuzz_kinds[..5] {
        t.check(seen);
    }
    tallies.push(t);
    let mut t = Tally::new("fixed_case_outcomes");
    for &seen in &kinds[..5] {
        t.check(seen);
    }
    tallies.push(t);
    tallies
}

fn main() -> ExitCode {
    if std::env::args().len() > 1 {
        eprintln!("usage: identity_catalog_parse");
        return ExitCode::from(2);
    }
    identity::report(&run(Scale::Full))
}
