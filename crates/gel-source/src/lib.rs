//! Immutable source catalog and exact quoted readout; independent of any speaker.
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
mod builder;
mod bundle;
pub mod collection;
pub mod context;
pub mod document;
pub use builder::{CorpusBuilder, EncodedCorpus};
pub use bundle::{import_text, load_bundle, read_regular, write_bundle_new, BundleError};
mod parts;
pub use parts::LeadPartsError;
use sha2::{Digest, Sha256};
pub type Hash = [u8; 32];
pub fn digest(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
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
impl Corpus {
    /// Exact immutable source payload after catalogue/pin validation.
    /// Adjacent records may belong to different documents; this accessor does
    /// not insert boundaries or establish semantic continuity between them.
    pub fn source_text(&self) -> &str {
        &self.text
    }
    /// Both pins must come from the caller's trusted local manifest, not the remote payload.
    /// Catalog: GELCORPUS082\n then node,role,entry,start,len,sha256,titleHex,sectionHex (TSV).
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
