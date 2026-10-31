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
/// The catalog loader before bundles handed their buffer over to the corpus,
/// unchanged. Tests compare the current loaders with it.
#[cfg(test)]
mod corpus_load_reference;
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
/// Catalogs whose distinct passage ranges total at least this many bytes have
/// the passage digests in [`Corpus::load`] computed on scoped threads; smaller
/// ones stay on the calling thread. A range stated by several rows counts once.
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
/// Keeps each byte range of `checks` once, so that its digest is computed
/// once. Returns false when two rows state different digests for the same
/// range: no digest equals both, so the answer of [`quotes_match`] over the
/// original list is false as well. Otherwise every digest of the original
/// list matches iff every digest of the shortened list does. Checks in
/// strictly increasing `(start, end)` order, as the builder writes them, hold
/// no repeat and stay as they are; any other order is sorted.
fn distinct_ranges(checks: &mut Vec<QuoteCheck>) -> bool {
    if checks
        .windows(2)
        .all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1))
    {
        return true;
    }
    checks.sort_unstable();
    let mut consistent = true;
    // Sorted by (start, end, digest): the rows of one range are adjacent.
    checks.dedup_by(|later, kept| {
        if (later.0, later.1) != (kept.0, kept.1) {
            return false;
        }
        if later.2 != kept.2 {
            consistent = false;
        }
        true
    });
    consistent
}
/// Whether `start..end` lies inside `text` with both ends on UTF-8 character
/// boundaries: for a `text` that is valid UTF-8, exactly whether
/// `str::get(start..end)` is `Some`. A boundary is the end of the text or a
/// byte that does not continue a character, as in `str::is_char_boundary`.
fn passage_in(text: &[u8], start: usize, end: usize) -> bool {
    let boundary = |at: usize| at == text.len() || text.get(at).is_some_and(|&b| b & 0xc0 != 0x80);
    start <= end && end <= text.len() && boundary(start) && boundary(end)
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
    /// Each distinct passage range is hashed once, however many rows state it.
    /// Distinct ranges totalling [`QUOTE_PARALLEL_MIN_BYTES`] or more have their
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
        let rows = Self::check_rows(raw, text.as_bytes(), threads)?;
        Ok(Self::assemble(catalog_pin, text_pin, text.to_owned(), rows))
    }
    /// [`Corpus::load_verified`] of `bytes[catalog]` and of the text
    /// `bytes[catalog.end..]`, for a caller that hands over the buffer holding
    /// both, such as a whole bundle file: the text stays in that buffer, moved
    /// to its front, instead of being copied, so the peak memory of a load is
    /// the buffer alone. The result, errors included, is the same.
    /// `catalog.start <= catalog.end <= bytes.len()`.
    pub(crate) fn load_verified_owned(
        mut bytes: Vec<u8>,
        catalog: std::ops::Range<usize>,
        catalog_pin: Hash,
        text_pin: Hash,
    ) -> Result<Self, Error> {
        let text_start = catalog.end;
        if catalog.len() > MAX_CATALOG || bytes.len() - text_start > MAX_TEXT {
            return Err(Error::Limit);
        }
        let raw = std::str::from_utf8(&bytes[catalog]).map_err(|_| Error::Format)?;
        // The text is checked as UTF-8 once, when it becomes the `String`
        // below, so here after the rows rather than before them. Both checks
        // give Format, and the rows are checked on bytes: whenever the text is
        // not UTF-8, any error of the rows is replaced by Format, the error
        // `load_verified` reports first for such a text.
        let rows = match Self::check_rows(raw, &bytes[text_start..], None) {
            Ok(rows) => rows,
            Err(e) if std::str::from_utf8(&bytes[text_start..]).is_ok() => return Err(e),
            Err(_) => return Err(Error::Format),
        };
        bytes.drain(..text_start);
        // Shrinking returns the catalog's share of the buffer, so the corpus
        // holds as much memory as one with a copied text.
        bytes.shrink_to_fit();
        let text = String::from_utf8(bytes).map_err(|_| Error::Format)?;
        Ok(Self::assemble(catalog_pin, text_pin, text, rows))
    }
    /// The checks of a load after the size and UTF-8 checks, on the catalog
    /// `raw` and the text as bytes, with their errors in the load's order. For
    /// a text that is valid UTF-8 the answer is that of the load.
    fn check_rows(raw: &str, text: &[u8], threads: Option<usize>) -> Result<Rows, Error> {
        if !raw.starts_with("GELCORPUS082\n") || !raw.ends_with('\n') || raw.contains('\r') {
            return Err(Error::Format);
        }
        // Rows are parsed in order; each passage digest is recorded at the
        // point where it is due and compared afterwards. Every recorded digest
        // precedes the first other error in row order, so any mismatch is the
        // error that comes first. A range stated by several rows is hashed
        // once; whether all recorded digests match does not depend on that.
        let mut checks = Vec::new();
        let parsed = Self::parse_rows(raw, text, &mut checks);
        if !distinct_ranges(&mut checks) || !quotes_match(text, &checks, threads) {
            return Err(Error::Integrity);
        }
        let (records, titles) = parsed?;
        if records.is_empty() {
            return Err(Error::Format);
        }
        Ok((records, titles))
    }
    fn assemble(catalog_pin: Hash, text_pin: Hash, text: String, (records, titles): Rows) -> Self {
        let mut binding = b"GELCORPUS082\0".to_vec();
        binding.extend_from_slice(&catalog_pin);
        binding.extend_from_slice(&text_pin);
        Self {
            root: digest(&binding),
            text_hash: text_pin,
            text,
            records,
            titles,
        }
    }
    /// The catalog rows after the header, up to the first error other than a
    /// passage digest mismatch; `checks` receives `(start, end, digest)` of
    /// every passage whose digest the row order reaches before that error.
    fn parse_rows(raw: &str, text: &[u8], checks: &mut Vec<QuoteCheck>) -> Result<Rows, Error> {
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
            if !passage_in(text, start, end) {
                return Err(Error::Format);
            }
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

#[cfg(test)]
mod distinct_range_tests {
    use super::*;
    /// The earlier answer: every recorded digest, one by one, in row order.
    fn all_match_reference(text: &[u8], checks: &[QuoteCheck]) -> bool {
        checks.iter().all(|&(s, e, h)| digest(&text[s..e]) == h)
    }
    /// The current answer for `threads`, as `load_unhashed` computes it.
    fn all_match(text: &[u8], checks: &[QuoteCheck], threads: Option<usize>) -> bool {
        let mut checks = checks.to_vec();
        distinct_ranges(&mut checks) && quotes_match(text, &checks, threads)
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
    fn text(rng: &mut Rng, len: usize) -> Vec<u8> {
        (0..len).map(|_| b'a' + rng.below(4) as u8).collect()
    }
    #[test]
    fn distinct_ranges_answer_equals_the_row_by_row_answer() {
        let mut rng = Rng(0x5eed_d15c_2026_1024);
        let text = text(&mut rng, 4096);
        let mut seen = [0usize; 2];
        // Inputs whose every digest is true while two different ranges share
        // their start, or share their end: a rule that compared only one end
        // of a range would merge two different passages there.
        let mut shared = [0usize; 2];
        for input in 0..3000 {
            // Few ranges, so most of them repeat; some lists stay increasing.
            // A new range may take the start or the end of an earlier one.
            let mut ranges: Vec<(usize, usize)> = Vec::new();
            for _ in 0..1 + rng.below(6) {
                let range = match (ranges.len(), rng.below(3)) {
                    (0, _) | (_, 0) => {
                        let s = rng.below(text.len());
                        (s, s + 1 + rng.below((text.len() - s).min(64)))
                    }
                    (n, 1) => {
                        let s = ranges[rng.below(n)].0;
                        (s, s + 1 + rng.below((text.len() - s).min(64)))
                    }
                    (n, _) => {
                        let e = ranges[rng.below(n)].1;
                        (e - 1 - rng.below(e.min(64)), e)
                    }
                };
                ranges.push(range);
            }
            let n = rng.below(80);
            let mut checks: Vec<QuoteCheck> = (0..n)
                .map(|_| {
                    let (s, e) = ranges[rng.below(ranges.len())];
                    (s, e, digest(&text[s..e]))
                })
                .collect();
            if input % 5 == 0 {
                checks.sort_unstable_by_key(|&(s, e, _)| (s, e));
                checks.dedup_by_key(|&mut (s, e, _)| (s, e));
            }
            // A wrong digest in no row, one row, or several rows.
            for _ in 0..rng.below(3) {
                if !checks.is_empty() {
                    let k = rng.below(checks.len());
                    checks[k].2[rng.below(32)] ^= 1 << rng.below(8);
                }
            }
            let old = all_match_reference(&text, &checks);
            seen[usize::from(old)] += 1;
            for threads in [None, Some(0), Some(1), Some(2), Some(3), Some(8)] {
                assert_eq!(all_match(&text, &checks, threads), old, "{checks:?}");
            }
            let mut stated: Vec<(usize, usize)> = checks.iter().map(|&(s, e, _)| (s, e)).collect();
            stated.sort_unstable();
            stated.dedup();
            for (side, count) in shared.iter_mut().enumerate() {
                let key = |r: &(usize, usize)| if side == 0 { r.0 } else { r.1 };
                let pair = stated
                    .iter()
                    .enumerate()
                    .any(|(i, a)| stated[i + 1..].iter().any(|b| key(a) == key(b)));
                *count += usize::from(old && pair);
            }
        }
        assert!(seen[0] > 100 && seen[1] > 100, "{seen:?}");
        assert!(shared[0] > 100 && shared[1] > 100, "{shared:?}");
    }
    #[test]
    fn conflicting_digests_of_one_range_fail_without_hashing() {
        let text = b"abcdefgh";
        let good = digest(&text[2..6]);
        let mut bad = good;
        bad[31] ^= 1;
        let mut checks = vec![(2, 6, good), (0, 1, digest(&text[0..1])), (2, 6, bad)];
        assert!(!all_match_reference(text, &checks));
        assert!(!distinct_ranges(&mut checks));
        // Increasing lists are left untouched.
        let mut increasing = vec![(0, 1, good), (0, 2, good), (1, 2, good)];
        let before = increasing.clone();
        assert!(distinct_ranges(&mut increasing));
        assert_eq!(increasing, before);
        // A repeat with one digest collapses to one entry.
        let mut repeated = vec![(2, 6, good), (0, 1, bad), (2, 6, good)];
        assert!(distinct_ranges(&mut repeated));
        assert_eq!(repeated, vec![(0, 1, bad), (2, 6, good)]);
    }
    /// One catalog row; `hash` None states the true digest of the range.
    fn row(text: &[u8], role: u64, start: usize, len: usize, hash: Option<Hash>) -> String {
        let h = hash.unwrap_or_else(|| digest(&text[start..start + len]));
        format!(
            "0\t{role}\t0\t{start}\t{len}\t{}\t{}\t{}\n",
            hex(&h),
            hex(b"Same"),
            hex(format!("S{role}").as_bytes())
        )
    }
    fn load(catalog: &str, text: &[u8]) -> Result<Corpus, Error> {
        let c = catalog.as_bytes();
        Corpus::load(c, text, digest(c), digest(text))
    }
    #[test]
    fn repeated_ranges_keep_the_error_order() {
        let text = b"repeated ranges of one text".to_vec();
        // Sorts after the true digest, so keeping the smallest digest of a
        // range would not hide it.
        let wrong = [0xff; 32];
        assert_ne!(wrong, digest(&text[0..8]));
        let head = "GELCORPUS082\n";
        // The same range in every row, all digests true: loads.
        let mut ok = String::from(head);
        for role in 0..40 {
            ok += &row(&text, role, 0, 8, None);
        }
        assert_eq!(load(&ok, &text).map(|c| c.records().count()), Ok(40));
        // One copy of the range states a wrong digest: Integrity.
        let mut conflict = String::from(head);
        for role in 0..40 {
            let h = (role == 33).then_some(wrong);
            conflict += &row(&text, role, 0, 8, h);
        }
        assert_eq!(load(&conflict, &text).err(), Some(Error::Integrity));
        // The wrong copy after a Format error is never reached: Format.
        let mut late = String::from(head);
        for role in 0..40 {
            if role == 20 {
                late += "0\t020\tbad\n";
            }
            let h = (role == 33).then_some(wrong);
            late += &row(&text, role, 0, 8, h);
        }
        assert_eq!(load(&late, &text).err(), Some(Error::Format));
        // The wrong copy before a duplicate address: Integrity, as before.
        let mut early = String::from(head);
        for role in 0..40 {
            let h = (role == 5).then_some(wrong);
            early += &row(&text, role, 0, 8, h);
        }
        early += &row(&text, 7, 0, 8, None);
        assert_eq!(load(&early, &text).err(), Some(Error::Integrity));
        early = early.replace(
            &row(&text, 5, 0, 8, Some(wrong)),
            &row(&text, 5, 0, 8, None),
        );
        assert_eq!(load(&early, &text).err(), Some(Error::Duplicate));
    }
}

#[cfg(test)]
mod owned_load_tests {
    use super::corpus_load_reference::load_unhashed_reference;
    use super::*;
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
    const PIECES: [&str; 8] = ["a", "ł", "€", "🦀", "\r\n", " ", "Zażółć", "\n"];
    fn mixed(rng: &mut Rng, pieces: usize) -> String {
        (0..pieces)
            .map(|_| PIECES[rng.below(PIECES.len())])
            .collect()
    }
    fn random_corpus(rng: &mut Rng) -> EncodedCorpus {
        let mut b = CorpusBuilder::new();
        for node in 0..1 + rng.below(4) as u64 {
            for role in 0..1 + rng.below(5) as u64 {
                let pieces = 1 + rng.below(40);
                let quote = mixed(rng, pieces);
                let title = ["Notes", "notes ", "Łódź", "Other"][rng.below(4)];
                let _ = b.push(
                    Address { node, role },
                    node,
                    title,
                    &format!("Part {role}"),
                    &quote,
                );
            }
        }
        b.finish().unwrap()
    }
    /// The earlier loader, the borrowed reader and the reader that takes the
    /// buffer over, on one catalog and text with pins equal to their digests:
    /// all three must give the same `Debug` form, errors included, and a
    /// corpus from the last one holds no spare capacity. Returns the result
    /// of the last one.
    fn same(catalog: &[u8], text: &[u8]) -> Result<(), Error> {
        let (catalog_pin, text_pin) = (digest(catalog), digest(text));
        let old = load_unhashed_reference(catalog, text, catalog_pin, text_pin, None);
        let old = format!("{old:?}");
        let borrowed = Corpus::load_verified(catalog, text, catalog_pin, text_pin);
        assert_eq!(format!("{borrowed:?}"), old, "borrowed reader");
        // A header before the catalog that is not UTF-8, as in a bundle.
        let mut buffer = vec![0xff_u8; 24];
        buffer.extend_from_slice(catalog);
        buffer.extend_from_slice(text);
        let range = 24..24 + catalog.len();
        let owned = Corpus::load_verified_owned(buffer, range, catalog_pin, text_pin);
        assert_eq!(format!("{owned:?}"), old, "owned reader");
        // The corpus keeps no share of the buffer beyond its text, as one
        // with a copied text.
        if let Ok(corpus) = &owned {
            assert_eq!(corpus.text.capacity(), corpus.text.len(), "owned capacity");
        }
        owned.map(|_| ())
    }
    /// `passage_in` on bytes is `str::get` on the same valid UTF-8.
    #[test]
    fn passage_in_equals_str_get_on_valid_utf8() {
        let mut rng = Rng(0x0b5e_55ed_2026_1025);
        let mut texts = vec![String::new(), "a".into(), "ł🦀a€\r\nż".into()];
        for _ in 0..20 {
            let pieces = rng.below(30);
            texts.push(mixed(&mut rng, pieces));
        }
        for text in &texts {
            for start in 0..text.len() + 3 {
                for end in 0..text.len() + 3 {
                    assert_eq!(
                        passage_in(text.as_bytes(), start, end),
                        text.get(start..end).is_some(),
                        "{text:?} {start}..{end}"
                    );
                }
            }
        }
    }
    /// Seeded corpora, every byte of the text changed (often into broken
    /// UTF-8), changed catalog bytes, and changed catalogs over a broken
    /// text: the three readers agree on every input.
    #[test]
    fn owned_and_borrowed_readers_equal_the_earlier_loader_on_seeded_changes() {
        let mut seen = std::collections::BTreeMap::new();
        let mut tally = |r: Result<(), Error>| *seen.entry(format!("{r:?}")).or_insert(0usize) += 1;
        for seed in 0..48u64 {
            let mut rng = Rng(seed);
            let e = random_corpus(&mut rng);
            let (catalog, text) = (e.catalog_bytes().to_vec(), e.text_bytes().to_vec());
            assert_eq!(same(&catalog, &text), Ok(()));
            for n in 0..text.len() {
                let mut t = text.clone();
                t[n] ^= [1u8, 0x40, 0x80, 0xff][n % 4];
                tally(same(&catalog, &t));
            }
            let mut broken = text.clone();
            broken.push(0xff);
            for n in 0..catalog.len() {
                let mut c = catalog.clone();
                c[n] ^= [1u8, 0x20, 0x80, 0x0f][n % 4];
                tally(same(&c, &text));
                tally(same(&c, &broken));
            }
            tally(same(&catalog, &broken));
            tally(same(&catalog, &[]));
            tally(same(&[], &text));
        }
        // Limit is reached by the explicit cases of the next test.
        for kind in ["Ok(())", "Err(Format)", "Err(Integrity)", "Err(Duplicate)"] {
            assert!(seen.get(kind).is_some_and(|&n| n > 0), "{kind}: {seen:?}");
        }
    }
    fn row(node: u64, role: u64, start: usize, len: usize, hash: &Hash) -> String {
        format!(
            "{node}\t{role}\t0\t{start}\t{len}\t{}\t{}\t{}\n",
            hex(hash),
            hex(b"Notes"),
            hex(format!("Part {role}").as_bytes())
        )
    }
    /// Over a valid text every catalog below fails with its own row error;
    /// over the same text with a byte that breaks UTF-8, inside a passage or
    /// after every passage, all three readers report Format.
    #[test]
    fn a_text_that_is_not_utf8_reports_format_before_every_row_error() {
        let text = "Zażółć gęślą jaźń 🦀 notes".as_bytes().to_vec();
        let good = digest(&text[0..6]);
        let head = "GELCORPUS082\n".to_string();
        let ok = head.clone() + &row(1, 1, 0, 6, &good);
        let many: String = (0..=MAX_RECORDS as u64)
            .map(|role| row(1, role, 0, 6, &good))
            .collect();
        let cases = [
            (ok.clone() + &row(1, 2, 0, 0, &good), Error::Limit),
            (
                ok.clone() + &row(1, 2, 0, MAX_PASSAGE + 1, &good),
                Error::Limit,
            ),
            (ok.clone() + &row(1, 1, 0, 6, &good), Error::Duplicate),
            (ok.clone() + &row(1, 2, 0, 6, &[7; 32]), Error::Integrity),
            (ok.clone() + &row(1, 2, 1, 6, &good), Error::Format),
            (ok.clone() + &row(1, 2, text.len(), 1, &good), Error::Format),
            (ok.clone() + "1\t2\n", Error::Format),
            (head.clone(), Error::Format),
            (head + &many, Error::Limit),
        ];
        let mut inside = text.clone();
        inside[2] = 0x80;
        let mut after = text.clone();
        after.push(0xc3);
        for (catalog, error) in &cases {
            assert_eq!(
                same(catalog.as_bytes(), &text),
                Err(*error),
                "{catalog:.80}"
            );
            for broken in [&inside, &after] {
                assert_eq!(same(catalog.as_bytes(), broken), Err(Error::Format));
            }
        }
        assert_eq!(same(ok.as_bytes(), &text), Ok(()));
        assert_eq!(same(ok.as_bytes(), &after), Err(Error::Format));
    }
}
