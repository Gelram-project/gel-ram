//! Bounded independent documents, exact citations and pinned plaintext snapshots.
//! No semantic ranking, encryption, background import or inferred truth.
use crate::{digest, document, read_regular, Hash};
use std::{collections::BTreeMap, ops::Range, path::Path, sync::OnceLock};

pub const MAX_DOCUMENTS: usize = 1024;
pub const MAX_TOTAL_TEXT: usize = 64 << 20;
pub const MAX_TITLE: usize = 512;
pub const MAX_RESULTS: usize = 16;
const MAGIC: &[u8; 8] = b"GELSET01";
const MAX_BUNDLE: usize = MAX_TOTAL_TEXT + MAX_DOCUMENTS * (20 + MAX_TITLE) + 32;

#[derive(Clone, Debug)]
pub struct Document {
    id: u64,
    title: String,
    text: String,
    hash: Hash,
}
impl Document {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn hash(&self) -> Hash {
        self.hash
    }
}

#[derive(Clone, Debug)]
pub struct Hit {
    root: Hash,
    document_id: u64,
    document_hash: Hash,
    span: Range<usize>,
    quote: String,
}
impl Hit {
    pub fn document_id(&self) -> u64 {
        self.document_id
    }
    pub fn document_hash(&self) -> Hash {
        self.document_hash
    }
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
    pub fn quote(&self) -> &str {
        &self.quote
    }
    pub fn root(&self) -> Hash {
        self.root
    }
}
#[derive(Debug)]
pub struct Search {
    pub hits: Vec<Hit>,
    pub matching_lines: usize,
    pub skipped_long_lines: usize,
    pub documents_examined: usize,
}
impl Search {
    pub fn status(&self) -> &'static str {
        if self.skipped_long_lines > 0 {
            "INCOMPLETE"
        } else if self.matching_lines == 0 {
            "UNKNOWN"
        } else {
            "HIT"
        }
    }
}

#[derive(Clone)]
pub struct Collection {
    documents: BTreeMap<u64, Document>,
    revision: u64,
    next_id: u64,
    total_text: usize,
    /// SHA-256 of the GELSET01 serialization of the fields above, computed on
    /// first use. Every mutation empties it.
    root: OnceLock<Hash>,
}
/// New-bank staging only. No lookup, persistence or root is exposed until build.
/// Accepted documents have the same IDs and revision as sequential `add` calls.
/// A rejected add leaves previous staged documents intact; it is not a live-bank
/// transaction and never mutates an existing collection.
pub struct CollectionBuilder {
    staged: Collection,
}
impl Default for CollectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl CollectionBuilder {
    pub fn new() -> Self {
        Self {
            staged: Collection::new(),
        }
    }
    pub fn add(&mut self, title: &str, text: &str) -> Result<u64, String> {
        self.staged.insert_without_refresh(title, text)
    }
    pub fn build(self) -> Collection {
        self.staged.root();
        self.staged
    }
}
impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}
impl Collection {
    pub fn new() -> Self {
        Self {
            documents: BTreeMap::new(),
            revision: 0,
            next_id: 1,
            total_text: 0,
            root: OnceLock::new(),
        }
    }
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }
    pub fn get(&self, id: u64) -> Option<&Document> {
        self.documents.get(&id)
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn root(&self) -> Hash {
        *self.root.get_or_init(|| self.compute_root())
    }
    pub fn text_bytes(&self) -> usize {
        self.total_text
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
    pub fn add(&mut self, title: &str, text: &str) -> Result<u64, String> {
        self.insert_without_refresh(title, text)
    }
    fn insert_without_refresh(&mut self, title: &str, text: &str) -> Result<u64, String> {
        Self::admit(title, text)?;
        if self.documents.len() >= MAX_DOCUMENTS || text.len() > MAX_TOTAL_TEXT - self.total_text {
            return Err("COLLECTION_LIMIT".into());
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("COLLECTION_REVISION_OVERFLOW")?;
        let next = self
            .next_id
            .checked_add(1)
            .ok_or("COLLECTION_ID_OVERFLOW")?;
        let id = self.next_id;
        self.documents.insert(
            id,
            Document {
                id,
                title: title.into(),
                text: text.into(),
                hash: digest(text.as_bytes()),
            },
        );
        self.total_text += text.len();
        self.revision = revision;
        self.next_id = next;
        self.root = OnceLock::new();
        Ok(id)
    }
    pub fn replace(&mut self, id: u64, text: &str) -> Result<(), String> {
        let old = self.documents.get(&id).ok_or("COLLECTION_MISSING_ID")?;
        Self::admit(&old.title, text)?;
        let total = self.total_text - old.text.len() + text.len();
        if total > MAX_TOTAL_TEXT {
            return Err("COLLECTION_LIMIT".into());
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("COLLECTION_REVISION_OVERFLOW")?;
        let d = self.documents.get_mut(&id).ok_or("COLLECTION_MISSING_ID")?;
        d.text = text.into();
        d.hash = digest(text.as_bytes());
        self.total_text = total;
        self.revision = revision;
        self.root = OnceLock::new();
        Ok(())
    }
    pub fn remove(&mut self, id: u64) -> Result<(), String> {
        let d = self.documents.get(&id).ok_or("COLLECTION_MISSING_ID")?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("COLLECTION_REVISION_OVERFLOW")?;
        self.total_text -= d.text.len();
        self.documents.remove(&id);
        self.revision = revision;
        self.root = OnceLock::new();
        Ok(())
    }
    pub fn search(&self, phrase: &str) -> Result<Search, String> {
        self.search_with_plan(phrase, None)
    }
    fn search_with_plan(
        &self,
        phrase: &str,
        plan: Option<document::Plan>,
    ) -> Result<Search, String> {
        // Validate even an empty collection. Never concatenate document boundaries.
        let query = document::Query::parse(phrase)?;
        let texts: Vec<&str> = self.documents.values().map(|d| d.text.as_str()).collect();
        if texts.iter().any(|t| t.len() > document::MAX_TEXT) {
            return Err("DOCUMENT_SEARCH_LIMIT".into());
        }
        let mut out = Search {
            hits: Vec::new(),
            matching_lines: 0,
            skipped_long_lines: 0,
            documents_examined: 0,
        };
        let results = document::search_texts(&texts, &query, plan);
        for (d, found) in self.documents.values().zip(results) {
            out.documents_examined += 1;
            out.matching_lines += found.matching_lines;
            out.skipped_long_lines += found.skipped_long_lines;
            for span in found.passages {
                if out.hits.len() < MAX_RESULTS {
                    out.hits.push(Hit {
                        root: self.root(),
                        document_id: d.id,
                        document_hash: d.hash,
                        quote: d.text[span.clone()].into(),
                        span,
                    });
                }
            }
        }
        Ok(out)
    }
    pub fn validate(&self, hit: &Hit) -> Result<(), String> {
        if hit.root != self.root() {
            return Err("COLLECTION_STALE".into());
        }
        let d = self.get(hit.document_id).ok_or("COLLECTION_MISSING_ID")?;
        if d.hash != hit.document_hash || d.text.get(hit.span.clone()) != Some(hit.quote.as_str()) {
            return Err("COLLECTION_CITATION_MISMATCH".into());
        }
        Ok(())
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(self.total_text + 32);
        b.extend_from_slice(MAGIC);
        for n in [self.revision, self.next_id, self.documents.len() as u64] {
            b.extend_from_slice(&n.to_le_bytes());
        }
        for d in self.documents.values() {
            b.extend_from_slice(&d.id.to_le_bytes());
            b.extend_from_slice(&(d.title.len() as u32).to_le_bytes());
            b.extend_from_slice(&(d.text.len() as u64).to_le_bytes());
            b.extend_from_slice(d.title.as_bytes());
            b.extend_from_slice(d.text.as_bytes());
        }
        b
    }
    /// The SHA-256 of exactly the bytes `to_bytes` writes, streamed without the copy.
    fn compute_root(&self) -> Hash {
        let mut hash = crate::sha256::Sha256::new();
        hash.update(MAGIC);
        for n in [self.revision, self.next_id, self.documents.len() as u64] {
            hash.update(n.to_le_bytes());
        }
        for d in self.documents.values() {
            hash.update(d.id.to_le_bytes());
            hash.update((d.title.len() as u32).to_le_bytes());
            hash.update((d.text.len() as u64).to_le_bytes());
            hash.update(d.title.as_bytes());
            hash.update(d.text.as_bytes());
        }
        hash.finalize()
    }
    pub fn from_bytes(bytes: &[u8], trusted_pin: Hash) -> Result<Self, String> {
        if bytes.len() > MAX_BUNDLE {
            return Err("COLLECTION_BUNDLE_LIMIT".into());
        }
        if digest(bytes) != trusted_pin {
            return Err("COLLECTION_INTEGRITY".into());
        }
        let mut documents = BTreeMap::new();
        let shape = parse(bytes, |id, title, text| {
            documents.insert(
                id,
                Document {
                    id,
                    title: title.into(),
                    text: text.into(),
                    hash: digest(text.as_bytes()),
                },
            );
        })?;
        Ok(Self {
            documents,
            revision: shape.revision,
            next_id: shape.next_id,
            total_text: shape.total_text,
            root: OnceLock::from(trusted_pin),
        })
    }
    /// The document count and revision of a snapshot whose bytes the caller has
    /// already matched to its trusted pin. Accepts and refuses exactly what
    /// `from_bytes` accepts and refuses for such bytes, with the same error,
    /// without hashing them again or copying any document.
    pub(crate) fn verified_shape(bytes: &[u8]) -> Result<(usize, u64), String> {
        if bytes.len() > MAX_BUNDLE {
            return Err("COLLECTION_BUNDLE_LIMIT".into());
        }
        let mut documents = 0;
        let shape = parse(bytes, |_, _, _| documents += 1)?;
        Ok((documents, shape.revision))
    }
    /// Same owner-controlled-directory and durability limits as write_bundle_new.
    pub fn save_new(&self, path: &Path) -> Result<Hash, String> {
        let bytes = self.to_bytes();
        // The root is the SHA-256 of exactly these bytes, so it is their pin.
        let pin = *self.root.get_or_init(|| digest(&bytes));
        crate::bundle::write_bytes_new_pinned(path, &bytes, pin)
            .map_err(|e| format!("COLLECTION_SAVE: {e:?}"))
    }
    pub fn load(path: &Path, trusted_pin: Hash) -> Result<Self, String> {
        let b = read_regular(path, MAX_BUNDLE).map_err(|e| format!("COLLECTION_READ: {e:?}"))?;
        Self::from_bytes(&b, trusted_pin)
    }
}
/// Header fields and total text length of one parsed GELSET01 serialization.
struct Shape {
    revision: u64,
    next_id: u64,
    total_text: usize,
}
/// Checks the whole GELSET01 structure after the length and pin checks and
/// hands each accepted document, in file order, to `each`.
fn parse<'a>(
    bytes: &'a [u8],
    mut each: impl FnMut(u64, &'a str, &'a str),
) -> Result<Shape, String> {
    let mut r = Cursor { bytes, offset: 0 };
    if r.take(8)? != MAGIC {
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
            u32::from_le_bytes(r.take(4)?.try_into().map_err(|_| "COLLECTION_LENGTH")?) as usize;
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
        Collection::admit(title, text)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The collection search of v0.6.1 over the reference document search.
    fn search_reference(c: &Collection, phrase: &str) -> Result<Search, String> {
        document::search_reference("", phrase)?;
        let mut out = Search {
            hits: Vec::new(),
            matching_lines: 0,
            skipped_long_lines: 0,
            documents_examined: 0,
        };
        for d in c.documents.values() {
            let found = document::search_reference(&d.text, phrase)?;
            out.documents_examined += 1;
            out.matching_lines += found.matching_lines;
            out.skipped_long_lines += found.skipped_long_lines;
            for span in found.passages {
                if out.hits.len() < MAX_RESULTS {
                    out.hits.push(Hit {
                        root: c.root(),
                        document_id: d.id,
                        document_hash: d.hash,
                        quote: d.text[span.clone()].into(),
                        span,
                    });
                }
            }
        }
        Ok(out)
    }

    fn summary(s: &Result<Search, String>) -> Result<String, String> {
        let s = s.as_ref().map_err(Clone::clone)?;
        let mut out = format!(
            "{} {} {}\n",
            s.matching_lines, s.skipped_long_lines, s.documents_examined
        );
        for h in &s.hits {
            out.push_str(&format!(
                "{:?} {} {:?} {:?} {:?}\n",
                h.root, h.document_id, h.document_hash, h.span, h.quote
            ));
        }
        Ok(out)
    }

    #[test]
    fn parallel_collection_search_matches_reference() {
        let mut seed = 1234_u64;
        for case in 0..60 {
            let mut builder = CollectionBuilder::new();
            for i in 0..case % 23 {
                let text = match (case + i) % 4 {
                    0 => "Ada not approved\r\n".repeat(1 + i % 7),
                    _ => {
                        let t = document::sample_text(&mut seed, 1 + (case + i) % 25);
                        if t.is_empty() {
                            "x".into()
                        } else {
                            t
                        }
                    }
                };
                builder.add(&format!("Document {i}"), &text).unwrap();
            }
            let mut c = builder.build();
            if case % 5 == 4 && c.documents.len() > 2 {
                // Gaps in the id order.
                let id = *c.documents.keys().nth(1).unwrap();
                c.remove(id).unwrap();
            }
            for phrase in [
                "ada not",
                "Ada not approved",
                "łódź",
                "οσ",
                "a",
                "missing token",
                "!!!",
                "",
                "x\ty",
            ] {
                let expected = summary(&search_reference(&c, phrase));
                assert_eq!(
                    summary(&c.search(phrase)),
                    expected,
                    "case {case} {phrase:?}"
                );
                for plan in [document::Plan::forced(3, 1), document::Plan::forced(6, 41)] {
                    assert_eq!(
                        summary(&c.search_with_plan(phrase, Some(plan))),
                        expected,
                        "case {case} {phrase:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn large_collection_takes_the_parallel_path_and_matches_reference() {
        let mut seed = 99_u64;
        let mut builder = CollectionBuilder::new();
        for i in 0..40 {
            let mut text = String::new();
            while text.len() < 64 << 10 {
                text.push_str(&document::sample_text(&mut seed, 32));
            }
            builder.add(&format!("Document {i}"), &text).unwrap();
        }
        let c = builder.build();
        assert!(c.text_bytes() > 2 << 20);
        for phrase in ["ada not", "łódź", "οσ", "x123", "missing token"] {
            assert_eq!(
                summary(&c.search(phrase)),
                summary(&search_reference(&c, phrase)),
                "{phrase:?}"
            );
        }
    }
}

#[cfg(test)]
mod lazy_root_tests {
    //! The current collection against the earlier source, which computed its
    //! root on every change, kept unchanged in `collection_eager_reference`.
    use super::*;
    use crate::collection_eager_reference as eager;

    struct Mix(u64);
    impl Mix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        fn text(&mut self, max_chars: u64) -> String {
            const PIECES: [&str; 9] = ["a", "b", "ab ", " ", "\n", "\r\n", "ł", "🦀", "x"];
            let n = self.below(max_chars + 1);
            (0..n)
                .map(|_| PIECES[self.below(PIECES.len() as u64) as usize])
                .collect()
        }
        fn title(&mut self) -> String {
            match self.below(12) {
                0 => String::new(),
                1 => "  ".into(),
                2 => "bad\ttitle".into(),
                3 => "t".repeat(MAX_TITLE + 1),
                4 => "t".repeat(MAX_TITLE),
                _ => format!("doc {}", self.below(50)),
            }
        }
        fn id(&mut self, next_id: u64) -> u64 {
            self.below(next_id + 2)
        }
    }

    fn same(lazy: &Collection, old: &eager::Collection) {
        let bytes = lazy.to_bytes();
        assert_eq!(bytes, old.to_bytes());
        assert_eq!(lazy.revision(), old.revision());
        assert_eq!(lazy.text_bytes(), old.text_bytes());
        let a: Vec<_> = lazy
            .documents()
            .map(|d| (d.id(), d.title(), d.text(), d.hash()))
            .collect();
        let b: Vec<_> = old
            .documents()
            .map(|d| (d.id(), d.title(), d.text(), d.hash()))
            .collect();
        assert_eq!(a, b);
        assert_eq!(lazy.root(), old.root());
        assert_eq!(lazy.root(), digest(&bytes));
    }

    fn same_hit(a: &Hit, b: &eager::Hit) {
        assert_eq!(a.root(), b.root());
        assert_eq!(a.document_id(), b.document_id());
        assert_eq!(a.document_hash(), b.document_hash());
        assert_eq!(a.span(), b.span());
        assert_eq!(a.quote(), b.quote());
    }

    fn same_search(
        lazy: &Collection,
        old: &eager::Collection,
        phrase: &str,
    ) -> Vec<(Hit, eager::Hit)> {
        let (a, b) = (lazy.search(phrase), old.search(phrase));
        match (a, b) {
            (Ok(a), Ok(b)) => {
                assert_eq!(a.matching_lines, b.matching_lines);
                assert_eq!(a.skipped_long_lines, b.skipped_long_lines);
                assert_eq!(a.documents_examined, b.documents_examined);
                assert_eq!(a.status(), b.status());
                assert_eq!(a.hits.len(), b.hits.len());
                for (x, y) in a.hits.iter().zip(&b.hits) {
                    same_hit(x, y);
                }
                a.hits.into_iter().zip(b.hits).collect()
            }
            (Err(a), Err(b)) => {
                assert_eq!(a, b);
                Vec::new()
            }
            (a, b) => panic!("search differs: {:?} / {:?}", a.is_ok(), b.is_ok()),
        }
    }

    /// Random add, replace, drop, search, proof, clone and reopen sequences.
    /// The root is read at random moments, so a change is applied both while
    /// the root is cached and while it is not, and several changes may pass
    /// before it is read.
    #[test]
    fn lazy_root_matches_the_eager_reference_on_random_sequences() {
        for seed in 0..96u64 {
            let mut m = Mix(seed);
            let mut lazy = Collection::new();
            let mut old = eager::Collection::new();
            if m.below(2) == 0 {
                same(&lazy, &old);
            }
            let mut hits: Vec<(Hit, eager::Hit)> = Vec::new();
            let steps = 1 + m.below(60);
            for _ in 0..steps {
                let next_id = old.documents().map(|d| d.id()).max().unwrap_or(0) + 1;
                match m.below(10) {
                    0..=3 => {
                        let (title, text) = (m.title(), m.text(40));
                        assert_eq!(lazy.add(&title, &text), old.add(&title, &text));
                    }
                    4 | 5 => {
                        let (id, text) = (m.id(next_id), m.text(40));
                        assert_eq!(lazy.replace(id, &text), old.replace(id, &text));
                    }
                    6 => {
                        let id = m.id(next_id);
                        assert_eq!(lazy.remove(id), old.remove(id));
                    }
                    7 => {
                        let phrase = ["a", "ab", "ł", "🦀", "x b", "", "zz"][m.below(7) as usize];
                        hits.extend(same_search(&lazy, &old, phrase));
                    }
                    8 => {
                        let b = lazy.to_bytes();
                        let pin = if m.below(2) == 0 {
                            lazy.root()
                        } else {
                            digest(&b)
                        };
                        lazy = Collection::from_bytes(&b, pin).unwrap();
                        old = eager::Collection::from_bytes(&old.to_bytes(), old.root()).unwrap();
                    }
                    _ => {
                        let copy = lazy.clone();
                        lazy = copy;
                    }
                }
                for (a, b) in &hits {
                    assert_eq!(lazy.validate(a), old.validate(b));
                }
                if m.below(3) == 0 {
                    same(&lazy, &old);
                }
            }
            same(&lazy, &old);
        }
    }

    #[test]
    fn builder_and_empty_collection_match_the_eager_reference() {
        same(&Collection::new(), &eager::Collection::new());
        same(&Collection::default(), &eager::Collection::default());
        same(
            &CollectionBuilder::new().build(),
            &eager::CollectionBuilder::new().build(),
        );
        for seed in 0..32u64 {
            let mut m = Mix(1000 + seed);
            let mut lazy = CollectionBuilder::new();
            let mut old = eager::CollectionBuilder::new();
            for _ in 0..m.below(40) {
                let (title, text) = (m.title(), m.text(30));
                assert_eq!(lazy.add(&title, &text), old.add(&title, &text));
            }
            same(&lazy.build(), &old.build());
        }
    }

    #[test]
    fn limits_match_the_eager_reference() {
        let mut lazy = Collection::new();
        let mut old = eager::Collection::new();
        let large = "x".repeat(document::MAX_TEXT + 1);
        assert_eq!(lazy.add("large", &large), old.add("large", &large));
        for n in 0..MAX_DOCUMENTS {
            let title = n.to_string();
            assert_eq!(lazy.add(&title, "x"), old.add(&title, "x"));
            if n % 257 == 0 {
                same(&lazy, &old);
            }
        }
        assert_eq!(lazy.add("extra", "x"), old.add("extra", "x"));
        same(&lazy, &old);
        assert_eq!(lazy.remove(1), old.remove(1));
        assert_eq!(lazy.add("again", "y"), old.add("again", "y"));
        same(&lazy, &old);
    }

    #[test]
    fn total_text_limit_matches_the_eager_reference() {
        let mut lazy = Collection::new();
        let mut old = eager::CollectionBuilder::new();
        let full = "z".repeat(document::MAX_TEXT);
        for n in 0..MAX_TOTAL_TEXT / document::MAX_TEXT {
            let title = format!("part {n}");
            assert_eq!(lazy.add(&title, &full), old.add(&title, &full));
        }
        assert_eq!(lazy.add("over", "z"), old.add("over", "z"));
        let old = old.build();
        assert_eq!(lazy.text_bytes(), MAX_TOTAL_TEXT);
        assert_eq!(lazy.to_bytes(), old.to_bytes());
        assert_eq!(lazy.root(), old.root());
        let mut old = old;
        assert_eq!(lazy.replace(2, "z"), old.replace(2, "z"));
        assert_eq!(lazy.replace(3, &full), old.replace(3, &full));
        assert_eq!(lazy.root(), old.root());
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gel-source-lazy-root-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn save_returns_the_digest_of_the_written_file_with_and_without_a_cached_root() {
        let dir = scratch("save");
        for seed in 0..24u64 {
            let mut m = Mix(5000 + seed);
            let mut c = Collection::new();
            for _ in 0..m.below(20) {
                let _ = c.add(&m.title(), &m.text(50));
            }
            if seed % 2 == 0 {
                c.root();
            }
            let path = dir.join(format!("bank-{seed}"));
            let pin = c.save_new(&path).unwrap();
            let written = std::fs::read(&path).unwrap();
            assert_eq!(written, c.to_bytes());
            assert_eq!(pin, digest(&written));
            assert_eq!(pin, c.root());
            assert!(c.save_new(&path).is_err());
            let reopened = Collection::load(&path, pin).unwrap();
            assert_eq!(reopened.root(), pin);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn shape_by_reference(bytes: &[u8]) -> Result<(usize, u64), String> {
        Collection::from_bytes(bytes, digest(bytes)).map(|c| (c.documents().count(), c.revision()))
    }

    /// The backup parse accepts and refuses exactly what `from_bytes` does for
    /// pin-matched bytes, with the same error value.
    #[test]
    fn verified_shape_matches_from_bytes_on_valid_cut_and_changed_snapshots() {
        for seed in 0..48u64 {
            let mut m = Mix(9000 + seed);
            let mut c = Collection::new();
            for _ in 0..m.below(8) {
                let _ = c.add(&m.title(), &m.text(12));
            }
            if m.below(2) == 0 {
                let _ = c.remove(1);
            }
            let bytes = c.to_bytes();
            assert_eq!(
                Collection::verified_shape(&bytes),
                Ok((c.documents().count(), c.revision()))
            );
            for n in 0..=bytes.len() {
                assert_eq!(
                    Collection::verified_shape(&bytes[..n]),
                    shape_by_reference(&bytes[..n])
                );
            }
            let mut changed = bytes.clone();
            for n in 0..changed.len() {
                for flip in [1u8, 0x80, 0xff] {
                    changed[n] ^= flip;
                    assert_eq!(
                        Collection::verified_shape(&changed),
                        shape_by_reference(&changed)
                    );
                    changed[n] ^= flip;
                }
            }
            let mut longer = bytes.clone();
            longer.push(0);
            assert_eq!(
                Collection::verified_shape(&longer),
                shape_by_reference(&longer)
            );
        }
        let over = vec![0; MAX_BUNDLE + 1];
        assert_eq!(
            Collection::verified_shape(&over),
            Err("COLLECTION_BUNDLE_LIMIT".to_string())
        );
        assert_eq!(Collection::verified_shape(&over), shape_by_reference(&over));
        assert_eq!(Collection::verified_shape(&[]), shape_by_reference(&[]));
    }
}
