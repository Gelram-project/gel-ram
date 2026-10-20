//! Identity program for loading a bundle and a collection snapshot with the
//! full-file digest overlapped with the part digests on scoped threads, which
//! stop early under a wrong pin.
//!
//! Usage: `identity_load_overlap [DIR]` (DIR defaults to the system temporary
//! directory; a fresh subdirectory is created there and removed at the end).
//!
//! It compares `load_bundle` and `Collection::from_bytes` with an exact copy
//! of the earlier one-thread code (module `reference`), on inputs generated
//! from fixed seeds: valid bundles and snapshots below, at and above the size
//! where the threaded path starts, the same bytes under a wrong pin, every
//! header byte changed, seeded changes in the catalog, the text and the
//! document headers, cuts, appended bytes, broken UTF-8 and changed lengths,
//! texts and documents a few bytes either side of a multiple of the piece
//! size in which one part is hashed, and snapshots of many documents under
//! every one-bit change of a pin byte.
//! Every changed input is loaded with the original pin and with its own pin,
//! so both the first error after the pin check and the pin check itself are
//! compared. A bundle is compared on the `Debug` form of the whole corpus or
//! error; a snapshot on every document, the revision, the next id, the text
//! size, the root and the serialization, or on the error text.
//!
//! Output: `IDENTITY name=<case> cases=<n> equal=<n>` per case, then
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL`. Exit 0 on PASS, 1 on
//! FAIL. Output grammar: `docs/SPEED.md`.
use gel_source::{
    collection::{Collection, CollectionBuilder},
    digest, import_text, load_bundle, CorpusBuilder, EncodedCorpus, Hash,
};
use std::path::{Path, PathBuf};

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

/// The earlier one-thread load, copied unchanged except that the bundle path
/// uses the public `Corpus::load` with pins equal to the digests, which is
/// the crate-internal reader after the same two digest comparisons.
mod reference {
    use gel_source::{
        collection::{MAX_DOCUMENTS, MAX_TITLE, MAX_TOTAL_TEXT},
        digest, document, Corpus, Error, Hash, MAX_CATALOG, MAX_TEXT,
    };

    pub const HEADER: usize = 24;
    const MAGIC: &[u8; 8] = b"GELSRC01";

    pub fn decode(bytes: &[u8], trusted_pin: Hash) -> Result<Corpus, Error> {
        if bytes.len() > HEADER + MAX_CATALOG + MAX_TEXT {
            return Err(Error::Limit);
        }
        if digest(bytes) != trusted_pin {
            return Err(Error::Integrity);
        }
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err(Error::Format);
        }
        let catalog_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let text_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        if catalog_len > MAX_CATALOG as u64 || text_len > MAX_TEXT as u64 {
            return Err(Error::Limit);
        }
        let split = HEADER + catalog_len as usize;
        if split + text_len as usize != bytes.len() {
            return Err(Error::Format);
        }
        let catalog = &bytes[HEADER..split];
        let text = &bytes[split..];
        Corpus::load(catalog, text, digest(catalog), digest(text))
    }

    const SET_MAGIC: &[u8; 8] = b"GELSET01";
    pub const MAX_BUNDLE: usize = MAX_TOTAL_TEXT + MAX_DOCUMENTS * (20 + MAX_TITLE) + 32;

    /// The fields of a loaded collection, as the earlier `from_bytes` set them.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Loaded {
        pub documents: Vec<(u64, String, String, Hash)>,
        pub revision: u64,
        pub next_id: u64,
        pub total_text: usize,
        pub root: Hash,
        pub serialized: Vec<u8>,
    }

    fn admit(title: &str, text: &str) -> Result<(), String> {
        if title.trim().is_empty() || title.len() > MAX_TITLE || title.chars().any(char::is_control)
        {
            return Err("COLLECTION_TITLE".into());
        }
        if text.is_empty() || text.len() > document::MAX_TEXT {
            return Err("COLLECTION_DOCUMENT_LIMIT".into());
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8], trusted_pin: Hash) -> Result<Loaded, String> {
        if bytes.len() > MAX_BUNDLE {
            return Err("COLLECTION_BUNDLE_LIMIT".into());
        }
        if digest(bytes) != trusted_pin {
            return Err("COLLECTION_INTEGRITY".into());
        }
        let mut documents = Vec::new();
        let shape = parse(bytes, |id, title, text| {
            documents.push((id, title.into(), text.into(), digest(text.as_bytes())));
        })?;
        let mut loaded = Loaded {
            documents,
            revision: shape.revision,
            next_id: shape.next_id,
            total_text: shape.total_text,
            root: trusted_pin,
            serialized: Vec::new(),
        };
        loaded.serialized = to_bytes(&loaded);
        Ok(loaded)
    }

    pub fn to_bytes(c: &Loaded) -> Vec<u8> {
        let mut b = Vec::with_capacity(c.total_text + 32);
        b.extend_from_slice(SET_MAGIC);
        for n in [c.revision, c.next_id, c.documents.len() as u64] {
            b.extend_from_slice(&n.to_le_bytes());
        }
        for (id, title, text, _) in &c.documents {
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&(title.len() as u32).to_le_bytes());
            b.extend_from_slice(&(text.len() as u64).to_le_bytes());
            b.extend_from_slice(title.as_bytes());
            b.extend_from_slice(text.as_bytes());
        }
        b
    }

    struct Shape {
        revision: u64,
        next_id: u64,
        total_text: usize,
    }

    fn parse<'a>(
        bytes: &'a [u8],
        mut each: impl FnMut(u64, &'a str, &'a str),
    ) -> Result<Shape, String> {
        let mut r = Cursor { bytes, offset: 0 };
        if r.take(8)? != SET_MAGIC {
            return Err("COLLECTION_MAGIC".into());
        }
        let revision = r.u64()?;
        let next_id = r.u64()?;
        let count = r.u64()?;
        if next_id == 0 || next_id - 1 > revision || count > MAX_DOCUMENTS as u64 {
            return Err("COLLECTION_HEADER".into());
        }
        let mut total_text = 0;
        let mut previous = 0;
        for _ in 0..count {
            let id = r.u64()?;
            let title_len =
                u32::from_le_bytes(r.take(4)?.try_into().map_err(|_| "COLLECTION_LENGTH")?)
                    as usize;
            let text_len = usize::try_from(r.u64()?).map_err(|_| "COLLECTION_LENGTH")?;
            if id <= previous
                || id >= next_id
                || title_len > MAX_TITLE
                || text_len > document::MAX_TEXT
                || text_len > MAX_TOTAL_TEXT - total_text
            {
                return Err("COLLECTION_RECORD".into());
            }
            let title = std::str::from_utf8(r.take(title_len)?).map_err(|_| "COLLECTION_UTF8")?;
            let text = std::str::from_utf8(r.take(text_len)?).map_err(|_| "COLLECTION_UTF8")?;
            admit(title, text)?;
            each(id, title, text);
            total_text += text_len;
            previous = id;
        }
        if r.offset != bytes.len() {
            return Err("COLLECTION_TRAILING_BYTES".into());
        }
        Ok(Shape {
            revision,
            next_id,
            total_text,
        })
    }

    struct Cursor<'a> {
        bytes: &'a [u8],
        offset: usize,
    }
    impl<'a> Cursor<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
            let end = self.offset.checked_add(n).ok_or("COLLECTION_LENGTH")?;
            let b = self
                .bytes
                .get(self.offset..end)
                .ok_or("COLLECTION_TRUNCATED")?;
            self.offset = end;
            Ok(b)
        }
        fn u64(&mut self) -> Result<u64, String> {
            Ok(u64::from_le_bytes(
                self.take(8)?.try_into().map_err(|_| "COLLECTION_LENGTH")?,
            ))
        }
    }
}

// How many inputs a run generates: `Scale::Full` is the verdict;
// `Scale::Small` keeps the threaded sizes but fewer seeds, for the test
// wrapper.

/// Inputs this long or longer take the threaded load path.
const THRESHOLD: usize = 1 << 20;

/// The piece size in which the threaded path hashes one part, stopping
/// between pieces when the pin does not match (copied from the crate).
const PIECE: usize = 256 << 10;

/// An ASCII text of exactly `len` bytes whose import keeps every byte, so the
/// text part of its bundle is `len` bytes long.
fn ascii_text(m: &mut Mix, len: usize) -> Vec<u8> {
    let mut text: Vec<u8> = (0..len)
        .map(|_| [b'a', b'b', b' ', b'\n'][m.below(4)])
        .collect();
    if let Some(last) = text.last_mut() {
        *last = b'z';
    }
    text
}

/// Bundles whose text is a few bytes either side of a multiple of [`PIECE`],
/// with the changed inputs of [`bundle_cases`].
fn bundle_piece_cases(m: &mut Mix, b: &mut Bundles, seeds: usize, step: usize) {
    for len in [4 * PIECE - 1, 4 * PIECE, 4 * PIECE + 1, 5 * PIECE + 1] {
        let corpus = import_text(&ascii_text(m, len), "Pieces").unwrap();
        assert_eq!(corpus.text_bytes().len(), len, "text part length");
        let bytes = envelope(&corpus);
        bundle_cases(m, &bytes, b, seeds, step);
    }
}

/// A snapshot whose documents are a few bytes either side of multiples of
/// [`PIECE`] (at least [`THRESHOLD`] bytes in all after any removal).
fn piece_sizes() -> Vec<usize> {
    vec![PIECE - 1, PIECE, PIECE + 1, 2 * PIECE - 1, 2 * PIECE + 1, 1]
}

/// A snapshot of many documents, some longer than [`PIECE`], loaded under its
/// pin, under every one-bit change of its first and last pin byte and with
/// seeded changes, so the queue of document digests is stopped part way.
fn snapshot_wrong_pin_cases(m: &mut Mix, s: &mut Snapshots, docs: usize, seeds: usize) {
    let sizes: Vec<usize> = (0..docs)
        .map(|i| {
            if i % 8 == 0 {
                PIECE + 1 + m.below(PIECE)
            } else {
                1 + m.below(PIECE / 4)
            }
        })
        .collect();
    let bytes = snapshot(m, &sizes);
    assert!(
        bytes.len() >= THRESHOLD,
        "snapshot reaches the threaded path"
    );
    let pin = digest(&bytes);
    s.one(&bytes, pin);
    for at in [0usize, 31] {
        for bit in 0..8 {
            let mut wrong = pin;
            wrong[at] ^= 1 << bit;
            s.one(&bytes, wrong);
        }
    }
    let mut changed = bytes.clone();
    for _ in 0..seeds {
        let n = m.below(bytes.len());
        changed[n] ^= [1u8, 0x80, 0xff][m.below(3)];
        s.both(&changed, pin);
        changed[n] = bytes[n];
    }
    s.both(&bytes[..bytes.len() - 1], pin);
}

struct Mix(u64);
impl Mix {
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
    /// Valid UTF-8 of exactly `len` bytes: ASCII words, newlines and two-,
    /// three- and four-byte characters.
    fn text(&mut self, len: usize) -> String {
        const PIECES: [&str; 10] = [
            "gel ", "source ", "quote\n", "ł", "ż", "€", "🦀", "a", " ", "\r\n",
        ];
        let mut s = String::with_capacity(len + 4);
        while s.len() < len {
            let piece = PIECES[self.below(PIECES.len())];
            if s.len() + piece.len() <= len {
                s.push_str(piece);
            } else {
                s.push('x');
            }
        }
        s
    }
}

fn envelope(e: &EncodedCorpus) -> Vec<u8> {
    let mut bytes = b"GELSRC01".to_vec();
    bytes.extend_from_slice(&(e.catalog_bytes().len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(e.text_bytes().len() as u64).to_le_bytes());
    bytes.extend_from_slice(e.catalog_bytes());
    bytes.extend_from_slice(e.text_bytes());
    bytes
}

/// The bundle of an imported ASCII text, exactly `total` bytes long (checked
/// by the caller). Every prefix of ASCII text is a valid import.
fn bundle_of_size(m: &mut Mix, total: usize) -> Vec<u8> {
    let base: Vec<u8> = (0..total)
        .map(|_| [b'a', b'b', b' ', b'\n'][m.below(4)])
        .collect();
    let bundle = |len: usize| envelope(&import_text(&base[..len], "Sized").unwrap());
    let mut len = total - 8192;
    for _ in 0..16 {
        let now = bundle(len).len();
        if now == total {
            break;
        }
        len = len + total - now;
    }
    // Lengths whose catalog gains a digit can skip one size; step back then.
    let mut bytes = bundle(len);
    while bytes.len() > total {
        len -= 1;
        bytes = bundle(len);
    }
    bytes
}

/// Several nodes and sections, with titles that collide after lowercasing.
fn built_bundle(m: &mut Mix, at_least: usize) -> Vec<u8> {
    let mut b = CorpusBuilder::new();
    let mut size = 0;
    let mut node = 0;
    while size < at_least {
        let title = ["Notes", "notes ", "Łódź", "Other"][m.below(4)];
        for role in 0..1 + m.below(6) {
            let len = 1 + m.below(32_768);
            let quote = m.text(len);
            size += quote.len();
            b.push(
                gel_source::Address {
                    node,
                    role: role as u64,
                },
                node,
                title,
                &format!("Part {role}"),
                &quote,
            )
            .unwrap();
        }
        node += 1;
    }
    envelope(&b.finish().unwrap())
}

struct Bundles<'a> {
    path: &'a Path,
    tally: Tally,
}
impl Bundles<'_> {
    fn one(&mut self, bytes: &[u8], pin: Hash) {
        std::fs::write(self.path, bytes).expect("write the bundle file");
        let new = format!("{:?}", load_bundle(self.path, pin));
        let old = format!(
            "{:?}",
            reference::decode(bytes, pin).map_err(gel_source::BundleError::Data)
        );
        self.tally.check(new == old);
    }
    /// `bytes` with its own pin and with `original`.
    fn both(&mut self, bytes: &[u8], original: Hash) {
        self.one(bytes, digest(bytes));
        self.one(bytes, original);
    }
}

/// The changed inputs of one bundle. Header bytes are changed at every
/// `step`-th offset; `seeds` seeded changes and cuts are added.
fn bundle_cases(m: &mut Mix, bytes: &[u8], b: &mut Bundles, seeds: usize, step: usize) {
    let pin = digest(bytes);
    b.one(bytes, pin);
    b.one(bytes, [0; 32]);
    let catalog_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let split = reference::HEADER + catalog_len;
    let mut changed = bytes.to_vec();
    for n in (0..reference::HEADER).step_by(step) {
        for flip in [1u8, 0x80] {
            changed[n] ^= flip;
            b.both(&changed, pin);
            changed[n] = bytes[n];
        }
    }
    for _ in 0..seeds {
        for (from, to) in [(reference::HEADER, split), (split, bytes.len())] {
            if from < to {
                let n = from + m.below(to - from);
                changed[n] ^= [1u8, 0x20, 0x80, 0xff][m.below(4)];
                b.both(&changed, pin);
                changed[n] = bytes[n];
            }
        }
    }
    let mut cuts = vec![
        0,
        1,
        reference::HEADER - 1,
        reference::HEADER,
        reference::HEADER + 1,
        split - 1,
        split,
        split + 1,
        bytes.len() - 1,
        THRESHOLD - 1,
        THRESHOLD,
    ];
    cuts.extend((0..seeds).map(|_| m.below(bytes.len())));
    for cut in cuts.into_iter().filter(|&c| c < bytes.len()) {
        b.both(&bytes[..cut], pin);
    }
    let mut longer = bytes.to_vec();
    longer.push(b'\n');
    b.both(&longer, pin);
    // Broken UTF-8 in the text, and a changed quote digest in the catalog.
    let mut broken = bytes.to_vec();
    if split < broken.len() {
        let n = split + m.below(broken.len() - split);
        broken[n] = 0xff;
        b.both(&broken, pin);
    }
    let mut quote_hash = bytes.to_vec();
    if let Some(tab) = quote_hash[reference::HEADER..split]
        .windows(66)
        .position(|w| w[0] == b'\t' && w[65] == b'\t' && w[1..65].iter().all(u8::is_ascii_hexdigit))
    {
        let at = reference::HEADER + tab + 1;
        quote_hash[at] = if quote_hash[at] == b'0' { b'1' } else { b'0' };
        b.both(&quote_hash, pin);
    }
    for (field, value) in [
        (8, (gel_source::MAX_CATALOG as u64) + 1),
        (16, (gel_source::MAX_TEXT as u64) + 1),
        (16, u64::MAX),
        (8, 0),
    ] {
        let mut lengths = bytes.to_vec();
        lengths[field..field + 8].copy_from_slice(&value.to_le_bytes());
        b.both(&lengths, pin);
    }
}

struct Snapshots {
    tally: Tally,
}
impl Snapshots {
    fn one(&mut self, bytes: &[u8], pin: Hash) {
        let new = Collection::from_bytes(bytes, pin).map(|c| {
            let serialized = c.to_bytes();
            reference::Loaded {
                documents: c
                    .documents()
                    .map(|d| (d.id(), d.title().into(), d.text().into(), d.hash()))
                    .collect(),
                revision: c.revision(),
                next_id: u64::from_le_bytes(serialized[16..24].try_into().unwrap()),
                total_text: c.text_bytes(),
                root: c.root(),
                serialized,
            }
        });
        let old = reference::from_bytes(bytes, pin);
        self.tally.check(new == old);
    }
    fn both(&mut self, bytes: &[u8], original: Hash) {
        self.one(bytes, digest(bytes));
        self.one(bytes, original);
    }
}

/// A snapshot of `docs` documents of the given sizes, some removed or
/// replaced so the ids, the revision and the next id differ.
fn snapshot(m: &mut Mix, sizes: &[usize]) -> Vec<u8> {
    let mut builder = CollectionBuilder::new();
    for (i, &len) in sizes.iter().enumerate() {
        let title = match m.below(4) {
            0 => "t".repeat(512),
            1 => "Łódź 🦀".to_string(),
            _ => format!("document {i}"),
        };
        builder.add(&title, &m.text(len)).unwrap();
    }
    let mut c = builder.build();
    if sizes.len() > 2 && m.below(2) == 0 {
        c.remove(1 + m.below(sizes.len()) as u64).unwrap();
    }
    if m.below(2) == 0 {
        let id = c.documents().next().map(|d| d.id());
        if let Some(id) = id {
            let len = c.get(id).unwrap().text().len();
            c.replace(id, &m.text(len)).unwrap();
        }
    }
    c.to_bytes()
}

/// The changed inputs of one snapshot. Bytes of the snapshot header and of
/// the first, second and last document header are changed at every `step`-th
/// offset; `seeds` seeded changes and cuts are added.
fn snapshot_cases(m: &mut Mix, bytes: &[u8], s: &mut Snapshots, seeds: usize, step: usize) {
    let pin = digest(bytes);
    s.one(bytes, pin);
    s.one(bytes, [0; 32]);
    let count = u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;
    // Offsets of every document header, from the bytes themselves.
    let mut heads = Vec::new();
    let mut at = 32;
    for _ in 0..count {
        heads.push(at);
        let title = u32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap()) as usize;
        let text = u64::from_le_bytes(bytes[at + 12..at + 20].try_into().unwrap()) as usize;
        at += 20 + title + text;
    }
    let mut changed = bytes.to_vec();
    let mut flip_range = |from: usize, to: usize, s: &mut Snapshots| {
        for n in (from..to.min(bytes.len())).step_by(step) {
            for flip in [1u8, 0x80, 0xff] {
                changed[n] ^= flip;
                s.both(&changed, pin);
                changed[n] = bytes[n];
            }
        }
    };
    flip_range(0, 32, s);
    let picked: Vec<usize> = if heads.len() <= 3 {
        heads.clone()
    } else {
        vec![heads[0], heads[1], heads[heads.len() - 1]]
    };
    for &h in &picked {
        flip_range(h, h + 20, s);
    }
    let mut changed = bytes.to_vec();
    for _ in 0..if bytes.len() > 32 { seeds } else { 0 } {
        let n = 32 + m.below(bytes.len() - 32);
        let flip = [1u8, 0x80, 0xff, 0x20][m.below(4)];
        changed[n] ^= flip;
        s.both(&changed, pin);
        changed[n] = bytes[n];
    }
    let mut cuts = vec![
        0,
        7,
        8,
        31,
        32,
        33,
        bytes.len() - 1,
        THRESHOLD - 1,
        THRESHOLD,
    ];
    for &h in &heads {
        cuts.extend([h - 1, h, h + 1, h + 20]);
    }
    cuts.extend((0..seeds).map(|_| m.below(bytes.len())));
    for cut in cuts.into_iter().filter(|&c| c < bytes.len()) {
        s.both(&bytes[..cut], pin);
    }
    let mut longer = bytes.to_vec();
    longer.push(0);
    s.both(&longer, pin);
    // A control character in a title, broken UTF-8 in a text, a zero length.
    if let Some(&h) = heads.first() {
        let mut bad = bytes.to_vec();
        bad[h + 20] = b'\t';
        s.both(&bad, pin);
        let title = u32::from_le_bytes(bytes[h + 8..h + 12].try_into().unwrap()) as usize;
        let mut bad = bytes.to_vec();
        bad[h + 20 + title] = 0xff;
        s.both(&bad, pin);
        let mut bad = bytes.to_vec();
        bad[h + 12..h + 20].copy_from_slice(&0u64.to_le_bytes());
        s.both(&bad, pin);
    }
}

/// The small scale: a few inputs above the threshold (an unoptimized build
/// hashes slowly) and the changed inputs of short bundles and snapshots.
fn run_small(dir: &Path) -> Vec<Tally> {
    let path = dir.join("case.gelsrc");
    let mut m = Mix(0x5eed_2026_1020_0006);
    let mut large = Bundles {
        path: &path,
        tally: Tally::new("bundle_threshold"),
    };
    let text = m.text(THRESHOLD + 4096);
    let bytes = envelope(&import_text(text.as_bytes(), "Imported").unwrap());
    let pin = digest(&bytes);
    large.one(&bytes, pin);
    large.one(&bytes, [0; 32]);
    let mut changed = bytes.clone();
    changed[0] ^= 1;
    large.both(&changed, pin);
    changed[0] = bytes[0];
    changed[30] ^= 0x20;
    large.one(&changed, digest(&changed));
    large.one(&bytes[..bytes.len() - 1], digest(&bytes[..bytes.len() - 1]));
    let mut small = Bundles {
        path: &path,
        tally: Tally::new("bundle_small"),
    };
    for _ in 0..2 {
        let at_least = 1 + m.below(4000);
        let bytes = built_bundle(&mut m, at_least);
        bundle_cases(&mut m, &bytes, &mut small, 2, 1);
    }
    let mut threshold = Snapshots {
        tally: Tally::new("snapshot_threshold"),
    };
    let mut b = CollectionBuilder::new();
    b.add("document 0", &m.text(THRESHOLD / 2)).unwrap();
    b.add("document 1", &m.text(THRESHOLD / 2)).unwrap();
    let bytes = b.build().to_bytes();
    let pin = digest(&bytes);
    threshold.one(&bytes, pin);
    threshold.one(&bytes, [0; 32]);
    let mut changed = bytes.clone();
    changed[24] ^= 0x80;
    threshold.both(&changed, pin);
    changed[24] = bytes[24];
    changed[bytes.len() - 1] ^= 0x80;
    threshold.one(&changed, digest(&changed));
    threshold.one(&bytes[..bytes.len() - 1], digest(&bytes[..bytes.len() - 1]));
    let mut short = Snapshots {
        tally: Tally::new("snapshot_small"),
    };
    for _ in 0..2 {
        let docs = m.below(5);
        let sizes: Vec<usize> = (0..docs).map(|_| 1 + m.below(300)).collect();
        let bytes = snapshot(&mut m, &sizes);
        snapshot_cases(&mut m, &bytes, &mut short, 2, 1);
    }
    let mut pieces = Snapshots {
        tally: Tally::new("snapshot_piece_sizes"),
    };
    let bytes = snapshot(&mut m, &piece_sizes());
    pieces.one(&bytes, digest(&bytes));
    pieces.one(&bytes, [0; 32]);
    vec![
        large.tally,
        small.tally,
        threshold.tally,
        short.tally,
        pieces.tally,
    ]
}

/// The comparison with its files in `dir`.
fn run_at(scale: Scale, dir: &Path) -> Vec<Tally> {
    if scale == Scale::Small {
        return run_small(dir);
    }
    let seeds = 24;
    let step = 1;
    let path = dir.join("case.gelsrc");
    let mut m = Mix(0x5eed_2026_1020_0005);

    let mut sized = Bundles {
        path: &path,
        tally: Tally::new("bundle_threshold"),
    };
    for total in [THRESHOLD - 1, THRESHOLD, THRESHOLD + 1] {
        let bytes = bundle_of_size(&mut m, total);
        assert_eq!(bytes.len(), total, "bundle size");
        sized.one(&bytes, digest(&bytes));
        sized.one(&bytes, [0; 32]);
        bundle_cases(&mut m, &bytes, &mut sized, 2, step);
    }

    let mut imported = Bundles {
        path: &path,
        tally: Tally::new("bundle_imported"),
    };
    for size in [1 << 20, 3 << 19, 3 << 20] {
        let text = m.text(size);
        let bytes = envelope(&import_text(text.as_bytes(), "Imported").unwrap());
        bundle_cases(&mut m, &bytes, &mut imported, seeds, step);
    }

    let mut built = Bundles {
        path: &path,
        tally: Tally::new("bundle_built"),
    };
    for _ in 0..3 {
        let bytes = built_bundle(&mut m, THRESHOLD + 1);
        bundle_cases(&mut m, &bytes, &mut built, seeds, step);
    }

    let mut small = Bundles {
        path: &path,
        tally: Tally::new("bundle_small"),
    };
    for _ in 0..seeds {
        let at_least = 1 + m.below(40_000);
        let bytes = built_bundle(&mut m, at_least);
        bundle_cases(&mut m, &bytes, &mut small, 2, 1);
    }

    let mut limit = Bundles {
        path: &path,
        tally: Tally::new("bundle_limit"),
    };
    let over = vec![0u8; reference::HEADER + gel_source::MAX_CATALOG + gel_source::MAX_TEXT + 1];
    limit.both(&over, [0; 32]);
    limit.both(&[], [0; 32]);

    let mut threshold = Snapshots {
        tally: Tally::new("snapshot_threshold"),
    };
    // GELSET01 header 32 bytes, one document header 20 bytes, title 10 bytes.
    for total in [THRESHOLD - 1, THRESHOLD, THRESHOLD + 1] {
        let mut b = CollectionBuilder::new();
        b.add("document 0", &"x".repeat(total - 62)).unwrap();
        let bytes = b.build().to_bytes();
        assert_eq!(bytes.len(), total);
        threshold.both(&bytes, [0; 32]);
        snapshot_cases(&mut m, &bytes, &mut threshold, 2, step);
    }

    let mut many = Snapshots {
        tally: Tally::new("snapshot_many"),
    };
    for _ in 0..6 {
        let docs = 2 + m.below(40);
        let mut sizes: Vec<usize> = (0..docs).map(|_| 1 + m.below(200_000)).collect();
        let total: usize = sizes.iter().sum();
        if total < THRESHOLD {
            sizes.push(THRESHOLD - total + 1);
        }
        let bytes = snapshot(&mut m, &sizes);
        snapshot_cases(&mut m, &bytes, &mut many, seeds, step);
    }

    let mut widest = Snapshots {
        tally: Tally::new("snapshot_max_documents"),
    };
    let sizes = vec![1100; gel_source::collection::MAX_DOCUMENTS];
    let bytes = snapshot(&mut m, &sizes);
    snapshot_cases(&mut m, &bytes, &mut widest, seeds, step);

    let mut large = Snapshots {
        tally: Tally::new("snapshot_large_documents"),
    };
    // One document of the largest admitted size: loaded, not changed.
    let bytes = snapshot(&mut m, &[gel_source::document::MAX_TEXT, 1]);
    large.one(&bytes, digest(&bytes));
    large.one(&bytes, [0; 32]);
    let bytes = snapshot(&mut m, &[3 << 20, 1, 2 << 20]);
    snapshot_cases(&mut m, &bytes, &mut large, seeds, step);

    let mut short = Snapshots {
        tally: Tally::new("snapshot_small"),
    };
    for _ in 0..seeds {
        let docs = m.below(9);
        let sizes: Vec<usize> = (0..docs).map(|_| 1 + m.below(3000)).collect();
        let bytes = snapshot(&mut m, &sizes);
        snapshot_cases(&mut m, &bytes, &mut short, 2, 1);
    }
    let over = vec![0u8; reference::MAX_BUNDLE + 1];
    short.both(&over, [0; 32]);
    short.both(&[], [0; 32]);

    let mut bundle_pieces = Bundles {
        path: &path,
        tally: Tally::new("bundle_piece_sizes"),
    };
    bundle_piece_cases(&mut m, &mut bundle_pieces, 4, step);

    let mut pieces = Snapshots {
        tally: Tally::new("snapshot_piece_sizes"),
    };
    for _ in 0..2 {
        let bytes = snapshot(&mut m, &piece_sizes());
        assert!(
            bytes.len() >= THRESHOLD,
            "snapshot reaches the threaded path"
        );
        snapshot_cases(&mut m, &bytes, &mut pieces, 4, step);
    }

    let mut stopped = Snapshots {
        tally: Tally::new("snapshot_wrong_pin_many"),
    };
    snapshot_wrong_pin_cases(&mut m, &mut stopped, 64, 8);
    snapshot_wrong_pin_cases(&mut m, &mut stopped, 256, 4);

    vec![
        sized.tally,
        imported.tally,
        built.tally,
        small.tally,
        limit.tally,
        threshold.tally,
        many.tally,
        widest.tally,
        large.tally,
        short.tally,
        bundle_pieces.tally,
        pieces.tally,
        stopped.tally,
    ]
}

/// Runs the comparison in a fresh subdirectory of `base`, which is removed
/// afterwards.
pub fn run_in(scale: Scale, base: &Path) -> Vec<Tally> {
    let dir = base.join(format!("gel-identity-load-overlap-{}", std::process::id()));
    std::fs::create_dir(&dir).expect("create a fresh directory");
    let tallies = run_at(scale, &dir);
    std::fs::remove_dir_all(&dir).expect("remove the directory");
    tallies
}

/// Runs the comparison in a fresh subdirectory of the system temporary
/// directory.
pub fn run(scale: Scale) -> Vec<Tally> {
    run_in(scale, &std::env::temp_dir())
}

fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let base: Option<PathBuf> = args.next().map(PathBuf::from);
    if args.next().is_some() {
        eprintln!("usage: identity_load_overlap [DIR]");
        return std::process::ExitCode::from(2);
    }
    match base {
        None => identity::report(&run(Scale::Full)),
        Some(base) => identity::report(&run_in(Scale::Full, &base)),
    }
}
