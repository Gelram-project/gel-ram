//! Immutable source catalog and exact quoted readout; independent of any answer generator.
//! Integrity proves correspondence to approved bytes, NOT truth, encoder correctness,
//! semantic relevance or authenticity of an untrusted catalog's claimed ORB addresses.
#![forbid(unsafe_code)]
// The Rust examples in these public guides compile and run as doctests.
#[cfg(doctest)]
#[doc = include_str!("../../../docs/SOURCE-BUILDER.md")]
struct SourceBuilderGuide;
#[cfg(doctest)]
#[doc = include_str!("../../../docs/COLLECTION-BUILDER.md")]
struct CollectionBuilderGuide;
pub mod backup;
mod builder;
mod bundle;
pub mod collection;
/// The earlier collection source, unchanged, which computed its root on every
/// change. Tests compare the current collection with it.
#[cfg(test)]
#[allow(dead_code)]
mod collection_eager_reference;
pub mod context;
pub mod document;
mod load_digests;
pub use builder::{CorpusBuilder, EncodedCorpus};
pub use bundle::{
    import_text, load_bundle, read_regular, write_bundle_new, write_file_new, write_file_new_with,
    BundleError, PublicationIo,
};
mod parts;
pub use parts::LeadPartsError;
mod line_end;
mod lowercase;
mod nfc;
/// The comparison of the current NFC with the earlier one; the example
/// `identity_nfc_fast` runs it at full scale.
#[cfg(test)]
mod nfc_fast_identity;
/// The earlier NFC source, unchanged except for visibility.
#[cfg(test)]
#[allow(dead_code)]
mod nfc_reference;
mod nfc_tables;
mod sha256;
pub type Hash = [u8; 32];
/// The SHA-256 digest of `bytes` (FIPS 180-4, this crate's own implementation).
pub fn digest(bytes: &[u8]) -> Hash {
    sha256::sha256(bytes)
}
/// The SHA-256 digest of everything `reader` yields, read in 64 KiB pieces;
/// equal to `digest` of the same bytes. Interrupted reads are retried.
pub fn digest_reader(mut reader: impl std::io::Read) -> std::io::Result<Hash> {
    let mut hash = sha256::Sha256::new();
    let mut buffer = vec![0u8; 64 << 10];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(hash.finalize()),
            Ok(n) => hash.update(&buffer[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}
use std::collections::BTreeMap;

pub const MAX_TEXT: usize = 64 * 1024 * 1024;
pub const MAX_CATALOG: usize = 16 * 1024 * 1024;
pub const MAX_RECORDS: usize = 50_000;
pub const MAX_PASSAGE: usize = 32_768;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub node: u64,
    pub role: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Limit,
    Integrity,
    Format,
    Duplicate,
    Missing,
    Stale,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub address: Address,
    pub entry: u64,
    pub start: usize,
    pub end: usize,
    pub hash: Hash,
    pub title: String,
    pub section: String,
}
#[derive(Debug)]
pub struct Corpus {
    root: Hash,
    text_hash: Hash,
    text: String,
    records: BTreeMap<Address, Record>,
    titles: BTreeMap<String, Vec<u64>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Passage {
    root: Hash,
    record: Record,
    quote: String,
}
impl Passage {
    pub fn root(&self) -> Hash {
        self.root
    }
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn quote(&self) -> &str {
        &self.quote
    }
}
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
/// Lowercase hexadecimal, two digits per byte.
pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    push_hex(&mut s, bytes);
    s
}
/// Appends `hex(bytes)` to `out` without a temporary string.
pub(crate) fn push_hex(out: &mut String, bytes: &[u8]) {
    out.reserve(bytes.len() * 2);
    for &b in bytes {
        out.push(char::from(HEX_DIGITS[usize::from(b >> 4)]));
        out.push(char::from(HEX_DIGITS[usize::from(b & 15)]));
    }
}
/// Appends the decimal digits of `v`, as `Display` writes them.
pub(crate) fn push_decimal(out: &mut String, mut v: u64) {
    let mut digits = [0u8; 20];
    let mut at = digits.len();
    loop {
        at -= 1;
        digits[at] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    for &d in &digits[at..] {
        out.push(char::from(d));
    }
}
/// The number of bytes `push_decimal(_, v)` appends.
pub(crate) fn decimal_len(mut v: u64) -> usize {
    let mut len = 1;
    while v >= 10 {
        v /= 10;
        len += 1;
    }
    len
}
/// The value of one lowercase hexadecimal digit.
fn nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}
/// Decodes lowercase hexadecimal of even length onto `out`.
fn unhex_into(s: &str, out: &mut Vec<u8>) -> Result<(), Error> {
    if s.len() % 2 != 0 {
        return Err(Error::Format);
    }
    for p in s.as_bytes().chunks_exact(2) {
        match (nibble(p[0]), nibble(p[1])) {
            (Some(high), Some(low)) => out.push(high * 16 + low),
            _ => return Err(Error::Format),
        }
    }
    Ok(())
}
/// Decodes exactly 64 lowercase hexadecimal digits.
fn unhex32(s: &str) -> Result<Hash, Error> {
    let s = s.as_bytes();
    if s.len() != 64 {
        return Err(Error::Format);
    }
    let mut out = [0u8; 32];
    for (byte, p) in out.iter_mut().zip(s.chunks_exact(2)) {
        match (nibble(p[0]), nibble(p[1])) {
            (Some(high), Some(low)) => *byte = high * 16 + low,
            _ => return Err(Error::Format),
        }
    }
    Ok(out)
}
/// A u64 in canonical decimal: ASCII digits only, no sign, no leading zero
/// except "0" itself.
fn number(s: &str) -> Result<u64, Error> {
    let s = s.as_bytes();
    if s.is_empty() || (s.len() > 1 && s[0] == b'0') {
        return Err(Error::Format);
    }
    let mut v: u64 = 0;
    for &c in s {
        if !c.is_ascii_digit() {
            return Err(Error::Format);
        }
        v = v
            .checked_mul(10)
            .and_then(|v| v.checked_add(u64::from(c - b'0')))
            .ok_or(Error::Format)?;
    }
    Ok(v)
}
fn label(s: &str) -> Result<String, Error> {
    if s.len() > 1024 {
        return Err(Error::Limit);
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    unhex_into(s, &mut bytes)?;
    let v = String::from_utf8(bytes).map_err(|_| Error::Format)?;
    if v.trim().is_empty() || v.chars().any(char::is_control) {
        return Err(Error::Format);
    }
    Ok(v)
}
/// A passage byte range of the source text and the digest its catalog row states.
type QuoteCheck = (usize, usize, Hash);
/// Records by address and node numbers by normalized title.
type Rows = (BTreeMap<Address, Record>, BTreeMap<String, Vec<u64>>);
/// Catalogs whose passages total at least this many bytes have the passage
/// digests in [`Corpus::load`] computed on scoped threads; smaller ones stay
/// on the calling thread.
pub const QUOTE_PARALLEL_MIN_BYTES: usize = 1 << 20;
/// Bytes of passages per thread below which the automatic path adds no thread.
const QUOTE_BYTES_PER_THREAD: usize = 256 << 10;
/// Upper bound on threads for the automatic path.
const QUOTE_MAX_THREADS: usize = 8;
/// Passages a worker claims at a time.
const QUOTE_BLOCK: usize = 16;
fn quote_auto_threads(total: usize) -> usize {
    static AVAILABLE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    if total < QUOTE_PARALLEL_MIN_BYTES {
        return 1;
    }
    let available =
        *AVAILABLE.get_or_init(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    available
        .min(QUOTE_MAX_THREADS)
        .min(total / QUOTE_BYTES_PER_THREAD)
}
/// Whether every passage digest equals the stated one. `threads` None picks
/// the count from the total passage length; the answer is the same for every
/// count. Workers claim blocks in row order and stop after any mismatch; the
/// calling thread works too, so a thread that cannot be started loses nothing.
fn quotes_match(text: &[u8], checks: &[QuoteCheck], threads: Option<usize>) -> bool {
    let threads = threads
        .unwrap_or_else(|| quote_auto_threads(checks.iter().map(|&(s, e, _)| e - s).sum()))
        .min(checks.len().div_ceil(QUOTE_BLOCK));
    if threads <= 1 {
        return checks.iter().all(|&(s, e, h)| digest(&text[s..e]) == h);
    }
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
    let next = AtomicUsize::new(0);
    let mismatch = AtomicBool::new(false);
    let work = || loop {
        let first = next.fetch_add(QUOTE_BLOCK, Relaxed);
        if first >= checks.len() {
            return;
        }
        for &(s, e, h) in &checks[first..(first + QUOTE_BLOCK).min(checks.len())] {
            if mismatch.load(Relaxed) {
                return;
            }
            if digest(&text[s..e]) != h {
                mismatch.store(true, Relaxed);
                return;
            }
        }
    };
    std::thread::scope(|scope| {
        let work = &work;
        let workers: Vec<_> = (1..threads)
            .filter_map(|_| std::thread::Builder::new().spawn_scoped(scope, work).ok())
            .collect();
        work();
        for worker in workers {
            // A worker cannot panic short of a bug; propagate it if it does.
            worker
                .join()
                .unwrap_or_else(|e| std::panic::resume_unwind(e));
        }
    });
    !mismatch.load(Relaxed)
}
impl Corpus {
    /// Exact immutable source payload after catalogue/pin validation.
    /// Adjacent records may belong to different documents; this accessor does
    /// not insert boundaries or establish semantic continuity between them.
    pub fn source_text(&self) -> &str {
        &self.text
    }
    /// Both pins must come from the caller's trusted local manifest, not the remote payload.
    /// Catalog: GELCORPUS082\n then node,role,entry,start,len,sha256,titleHex,sectionHex (TSV).
    ///
    /// Passages totalling [`QUOTE_PARALLEL_MIN_BYTES`] or more have their
    /// digests computed on scoped threads; the result, including which error
    /// is reported when the inputs hold several, is the same as on one thread.
    pub fn load(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
    ) -> Result<Self, Error> {
        Self::load_pinned(catalog, text, catalog_pin, text_pin, None)
    }
    /// [`Corpus::load`] with the passage digests computed on at most `threads`
    /// threads at any size; `0` and `1` mean the calling thread only. The
    /// result, including which error is reported when the inputs hold several,
    /// equals [`Corpus::load`] for every `threads`.
    pub fn load_with_threads(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
        threads: usize,
    ) -> Result<Self, Error> {
        Self::load_pinned(catalog, text, catalog_pin, text_pin, Some(threads))
    }
    fn load_pinned(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
        threads: Option<usize>,
    ) -> Result<Self, Error> {
        if catalog.len() > MAX_CATALOG || text.len() > MAX_TEXT {
            return Err(Error::Limit);
        }
        if digest(catalog) != catalog_pin || digest(text) != text_pin {
            return Err(Error::Integrity);
        }
        Self::load_unhashed(catalog, text, catalog_pin, text_pin, threads)
    }
    /// `load` for a caller that already holds `catalog_pin == digest(catalog)` and
    /// `text_pin == digest(text)`: the same result without hashing either again.
    pub(crate) fn load_verified(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
    ) -> Result<Self, Error> {
        Self::load_unhashed(catalog, text, catalog_pin, text_pin, None)
    }
    fn load_unhashed(
        catalog: &[u8],
        text: &[u8],
        catalog_pin: Hash,
        text_pin: Hash,
        threads: Option<usize>,
    ) -> Result<Self, Error> {
        if catalog.len() > MAX_CATALOG || text.len() > MAX_TEXT {
            return Err(Error::Limit);
        }
        let raw = std::str::from_utf8(catalog).map_err(|_| Error::Format)?;
        let text = std::str::from_utf8(text).map_err(|_| Error::Format)?;
        if !raw.starts_with("GELCORPUS082\n") || !raw.ends_with('\n') || raw.contains('\r') {
            return Err(Error::Format);
        }
        // Rows are parsed in order; each passage digest is recorded at the
        // point where it is due and compared afterwards. Every recorded digest
        // precedes the first other error in row order, so any mismatch is the
        // error that comes first.
        let mut checks = Vec::new();
        let parsed = Self::parse_rows(raw, text, &mut checks);
        if !quotes_match(text.as_bytes(), &checks, threads) {
            return Err(Error::Integrity);
        }
        let (records, titles) = parsed?;
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
    /// The catalog rows after the header, up to the first error other than a
    /// passage digest mismatch; `checks` receives `(start, end, digest)` of
    /// every passage whose digest the row order reaches before that error.
    fn parse_rows(raw: &str, text: &str, checks: &mut Vec<QuoteCheck>) -> Result<Rows, Error> {
        let mut records = BTreeMap::new();
        let mut titles: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        let mut nodes = BTreeMap::new();
        for line in raw[13..].split_terminator('\n') {
            if records.len() >= MAX_RECORDS {
                return Err(Error::Limit);
            }
            // Exactly eight tab-separated fields.
            let mut fields = line.split('\t');
            let mut f = [""; 8];
            for slot in &mut f {
                *slot = fields.next().ok_or(Error::Format)?;
            }
            if fields.next().is_some() {
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
            // The passage must lie on character boundaries inside the text.
            text.get(start..end).ok_or(Error::Format)?;
            if f[5].len() != 64 {
                return Err(Error::Format);
            }
            let hash = unhex32(f[5])?;
            checks.push((start, end, hash));
            let title = label(f[6])?;
            let section = label(f[7])?;
            match nodes.entry(address.node) {
                std::collections::btree_map::Entry::Occupied(old) => {
                    let (old_entry, old_title) = old.get();
                    if *old_entry != entry || *old_title != title {
                        return Err(Error::Format);
                    }
                }
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert((entry, title.clone()));
                    // Append once per node, avoiding a quadratic scan when many
                    // distinct nodes share the same normalized title.
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
        Ok((records, titles))
    }
    pub fn root(&self) -> Hash {
        self.root
    }
    pub fn text_hash(&self) -> Hash {
        self.text_hash
    }
    pub fn records(&self) -> impl Iterator<Item = &Record> {
        self.records.values()
    }
    /// Exact, lowercased title lookup; not a semantic search or relevance claim.
    pub fn find_title(&self, title: &str) -> &[u64] {
        self.titles
            .get(&title.trim().to_lowercase())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn sections(&self, node: u64) -> impl Iterator<Item = &Record> {
        self.records
            .range(
                Address { node, role: 0 }..=Address {
                    node,
                    role: u64::MAX,
                },
            )
            .map(|(_, r)| r)
    }
    pub fn quote(&self, address: Address) -> Result<Passage, Error> {
        let record = self.records.get(&address).ok_or(Error::Missing)?.clone();
        Ok(Passage {
            root: self.root,
            quote: self.text[record.start..record.end].to_owned(),
            record,
        })
    }
    pub fn validate(&self, p: &Passage) -> Result<(), Error> {
        if p.root != self.root {
            return Err(Error::Stale);
        }
        if self.quote(p.record.address)? != *p {
            return Err(Error::Integrity);
        }
        Ok(())
    }
}

#[cfg(test)]
mod field_tests {
    use super::*;
    /// The earlier `number`: parse, then require the canonical round trip.
    fn number_reference(s: &str) -> Result<u64, Error> {
        let v: u64 = s.parse().map_err(|_| Error::Format)?;
        if v.to_string() != s {
            return Err(Error::Format);
        }
        Ok(v)
    }
    /// The earlier `unhex`.
    fn unhex_reference(s: &str) -> Result<Vec<u8>, Error> {
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
    #[test]
    fn number_equals_the_round_trip_on_every_short_string() {
        let alphabet = [
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "-", "a",
        ];
        let mut all = vec![String::new()];
        let mut level = vec![String::new()];
        for _ in 0..3 {
            level = level
                .iter()
                .flat_map(|p| alphabet.iter().map(move |c| format!("{p}{c}")))
                .collect();
            all.extend(level.iter().cloned());
        }
        for edge in [
            "18446744073709551615",
            "18446744073709551616",
            "99999999999999999999",
            "018446744073709551615",
            "+18446744073709551615",
            "007",
            " 5",
            "5 ",
            "\u{0661}",
            "١٢",
        ] {
            all.push(edge.to_owned());
        }
        for s in &all {
            assert_eq!(number(s), number_reference(s), "{s:?}");
        }
    }
    #[test]
    fn hex_and_decimal_equal_the_formatter() {
        for b in 0..=255u8 {
            assert_eq!(hex(&[b]), format!("{b:02x}"));
        }
        for v in [0, 1, 9, 10, 99, 100, u64::MAX / 10, u64::MAX - 1, u64::MAX] {
            let mut s = String::new();
            push_decimal(&mut s, v);
            assert_eq!(s, v.to_string());
            assert_eq!(decimal_len(v), s.len());
        }
    }
    #[test]
    fn unhex32_equals_the_earlier_decoder() {
        let good = hex(&digest(b"unhex"));
        assert_eq!(unhex32(&good).map(Vec::from), unhex_reference(&good));
        for at in 0..64 {
            for c in ['0', '9', 'a', 'f', 'A', 'F', 'g', '+', ' '] {
                let mut s = good.clone().into_bytes();
                s[at] = c as u8;
                let s = String::from_utf8(s).unwrap();
                assert_eq!(unhex32(&s).map(Vec::from), unhex_reference(&s), "{s}");
                let mut v = Vec::new();
                assert_eq!(unhex_into(&s, &mut v).map(|_| v), unhex_reference(&s));
            }
        }
    }
}
