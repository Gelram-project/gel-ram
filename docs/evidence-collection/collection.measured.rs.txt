//! Bounded independent documents, exact citations and pinned plaintext snapshots.
//! No semantic ranking, encryption, background import or inferred truth.
use crate::{digest, document, read_regular, Hash};
use std::{collections::BTreeMap, ops::Range, path::Path};

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
    root: Hash,
}
impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}
impl Collection {
    pub fn new() -> Self {
        let mut c = Self {
            documents: BTreeMap::new(),
            revision: 0,
            next_id: 1,
            total_text: 0,
            root: [0; 32],
        };
        c.refresh_root();
        c
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
        self.root
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
        self.refresh_root();
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
        self.refresh_root();
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
        self.refresh_root();
        Ok(())
    }
    pub fn search(&self, phrase: &str) -> Result<Search, String> {
        // Validate even an empty collection. Never concatenate document boundaries.
        document::search("", phrase)?;
        let mut out = Search {
            hits: Vec::new(),
            matching_lines: 0,
            skipped_long_lines: 0,
            documents_examined: 0,
        };
        for d in self.documents.values() {
            let found = document::search(&d.text, phrase)?;
            out.documents_examined += 1;
            out.matching_lines += found.matching_lines;
            out.skipped_long_lines += found.skipped_long_lines;
            for span in found.passages {
                if out.hits.len() < MAX_RESULTS {
                    out.hits.push(Hit {
                        root: self.root,
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
        if hit.root != self.root {
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
    fn refresh_root(&mut self) {
        self.root = digest(&self.to_bytes());
    }
    pub fn from_bytes(bytes: &[u8], trusted_pin: Hash) -> Result<Self, String> {
        if bytes.len() > MAX_BUNDLE {
            return Err("COLLECTION_BUNDLE_LIMIT".into());
        }
        if digest(bytes) != trusted_pin {
            return Err("COLLECTION_INTEGRITY".into());
        }
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
        let mut documents = BTreeMap::new();
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
            Self::admit(title, text)?;
            documents.insert(
                id,
                Document {
                    id,
                    title: title.into(),
                    text: text.into(),
                    hash: digest(text.as_bytes()),
                },
            );
            total_text += text_len;
            previous = id;
        }
        if r.offset != bytes.len() {
            return Err("COLLECTION_TRAILING_BYTES".into());
        }
        Ok(Self {
            documents,
            revision,
            next_id,
            total_text,
            root: trusted_pin,
        })
    }
    /// Same owner-controlled-directory and durability limits as write_bundle_new.
    pub fn save_new(&self, path: &Path) -> Result<Hash, String> {
        crate::bundle::write_bytes_new(path, &self.to_bytes())
            .map_err(|e| format!("COLLECTION_SAVE: {e:?}"))
    }
    pub fn load(path: &Path, trusted_pin: Hash) -> Result<Self, String> {
        let b = read_regular(path, MAX_BUNDLE).map_err(|e| format!("COLLECTION_READ: {e:?}"))?;
        Self::from_bytes(&b, trusted_pin)
    }
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
