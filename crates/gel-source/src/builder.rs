//! Public source-catalog writer. No private encoder, quantization or semantic claims.
use crate::{
    digest, hex, Address, Corpus, Error, Hash, MAX_CATALOG, MAX_PASSAGE, MAX_RECORDS, MAX_TEXT,
};
use std::collections::{BTreeMap, BTreeSet};

/// Append exact UTF-8 passages in caller order; rejected pushes leave state unchanged.
/// Labels follow the existing reader's contract (at most 512 UTF-8 bytes).
pub struct CorpusBuilder {
    text: Vec<u8>,
    catalog: String,
    addresses: BTreeSet<Address>,
    nodes: BTreeMap<u64, (u64, String)>,
}

/// Encoded public catalog and text, not a numeric/Q8 ORB payload.
/// Pins attest byte correspondence, not truth or authenticity of a remote source.
pub struct EncodedCorpus {
    text: Vec<u8>,
    catalog: Vec<u8>,
    text_pin: Hash,
    catalog_pin: Hash,
}

impl Default for CorpusBuilder {
    fn default() -> Self {
        Self::new()
    }
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
    /// Labels/entry must agree for every passage sharing a node. Distinct nodes
    /// may share a title; the reader will report that lookup as ambiguous.
    /// No normalization, separator insertion, truncation or newline conversion.
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
        if quote.is_empty() || quote.len() > MAX_PASSAGE || self.addresses.len() >= MAX_RECORDS {
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
    pub fn finish(self) -> Result<EncodedCorpus, Error> {
        if self.addresses.is_empty() {
            return Err(Error::Format);
        }
        Ok(EncodedCorpus {
            text_pin: digest(&self.text),
            catalog_pin: digest(self.catalog.as_bytes()),
            text: self.text,
            catalog: self.catalog.into_bytes(),
        })
    }
}
impl EncodedCorpus {
    pub fn text_bytes(&self) -> &[u8] {
        &self.text
    }
    pub fn catalog_bytes(&self) -> &[u8] {
        &self.catalog
    }
    pub fn text_pin(&self) -> Hash {
        self.text_pin
    }
    pub fn catalog_pin(&self) -> Hash {
        self.catalog_pin
    }
    /// Validates through the existing reader. Serialized remote bundles must
    /// instead use pins from a separately trusted manifest with Corpus::load.
    pub fn load(&self) -> Result<Corpus, Error> {
        Corpus::load(&self.catalog, &self.text, self.catalog_pin, self.text_pin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn address(role: u64) -> Address {
        Address { node: 7, role }
    }
    fn bundle() -> EncodedCorpus {
        let mut b = CorpusBuilder::new();
        b.push(
            address(1),
            9,
            "Łódź — notes",
            "Lead (1)",
            "Zażółć gęślą jaźń.\r\n",
        )
        .unwrap();
        b.push(
            address(2),
            9,
            "Łódź — notes",
            "Lead (2)",
            "Café / cafe\u{301} 🦀 — it's exact.\n",
        )
        .unwrap();
        b.finish().unwrap()
    }
    #[test]
    fn unicode_multipart_roundtrip() {
        let b = bundle();
        let c = b.load().unwrap();
        let p = c.lead_parts("łódź — notes", Some(2)).unwrap();
        assert_eq!(p[0].quote(), "Zażółć gęślą jaźń.\r\n");
        assert_eq!(p[1].quote(), "Café / cafe\u{301} 🦀 — it's exact.\n");
        assert_eq!(p[1].record().start, p[0].quote().len());
        for p in p {
            c.validate(&p).unwrap();
        }
    }
    #[test]
    fn empty_builder_rejected() {
        assert!(matches!(CorpusBuilder::new().finish(), Err(Error::Format)));
    }
    #[test]
    fn empty_or_oversized_passage_rejected() {
        for q in [String::new(), "x".repeat(MAX_PASSAGE + 1)] {
            let mut b = CorpusBuilder::new();
            assert_eq!(b.push(address(1), 9, "A", "Lead", &q), Err(Error::Limit));
            assert!(b.text.is_empty());
            assert!(b.addresses.is_empty());
        }
    }
    #[test]
    fn maximum_passage_accepted() {
        let mut b = CorpusBuilder::new();
        let q = "x".repeat(MAX_PASSAGE);
        b.push(address(1), 9, "A", "Lead", &q).unwrap();
        assert_eq!(
            b.finish()
                .unwrap()
                .load()
                .unwrap()
                .quote(address(1))
                .unwrap()
                .quote(),
            q
        );
    }
    #[test]
    fn invalid_labels_rejected() {
        for label in [" ", "A\nB", "A\tB", "A\0B"] {
            for (title, section) in [(label, "Lead"), ("A", label)] {
                assert_eq!(
                    CorpusBuilder::new().push(address(1), 9, title, section, "q"),
                    Err(Error::Format)
                );
            }
        }
    }
    #[test]
    fn unicode_label_limit_is_bytes() {
        let mut b = CorpusBuilder::new();
        assert_eq!(
            b.push(address(1), 9, &"ą".repeat(257), "Lead", "q"),
            Err(Error::Limit)
        );
        b.push(address(1), 9, &"ą".repeat(256), "Lead", "q")
            .unwrap();
        b.finish().unwrap().load().unwrap();
    }
    #[test]
    fn duplicate_is_transactional() {
        let mut b = CorpusBuilder::new();
        b.push(address(1), 9, "A", "Lead", "first").unwrap();
        let prior = (b.text.clone(), b.catalog.clone());
        assert_eq!(
            b.push(address(1), 9, "A", "Lead", "different"),
            Err(Error::Duplicate)
        );
        assert_eq!((b.text.clone(), b.catalog.clone()), prior);
        b.finish().unwrap().load().unwrap();
    }
    #[test]
    fn conflicting_node_is_transactional() {
        for (entry, title) in [(10, "A"), (9, "B")] {
            let mut b = CorpusBuilder::new();
            b.push(address(1), 9, "A", "Lead (1)", "first").unwrap();
            let prior = (b.text.clone(), b.catalog.clone());
            assert_eq!(
                b.push(address(2), entry, title, "Lead (2)", "second"),
                Err(Error::Format)
            );
            assert_eq!((b.text.clone(), b.catalog.clone()), prior);
            b.push(address(2), 9, "A", "Lead (2)", "second").unwrap();
            b.finish().unwrap().load().unwrap();
        }
    }
    #[test]
    fn output_is_deterministic() {
        let a = bundle();
        let b = bundle();
        assert_eq!(a.catalog_bytes(), b.catalog_bytes());
        assert_eq!(a.text_bytes(), b.text_bytes());
        assert_eq!(a.catalog_pin(), b.catalog_pin());
    }
    #[test]
    fn changed_text_and_catalog_rejected() {
        let b = bundle();
        let mut text = b.text_bytes().to_vec();
        text[0] ^= 1;
        assert!(matches!(
            Corpus::load(b.catalog_bytes(), &text, b.catalog_pin(), b.text_pin()),
            Err(Error::Integrity)
        ));
        let mut cat = b.catalog_bytes().to_vec();
        cat[0] ^= 1;
        assert!(matches!(
            Corpus::load(&cat, b.text_bytes(), b.catalog_pin(), b.text_pin()),
            Err(Error::Integrity)
        ));
    }
    #[test]
    fn old_passage_not_valid_in_new_generation() {
        let a = bundle().load().unwrap();
        let p = a.quote(address(1)).unwrap();
        let mut b = CorpusBuilder::new();
        b.push(address(1), 9, "Łódź — notes", "Lead (1)", "changed")
            .unwrap();
        assert_eq!(
            b.finish().unwrap().load().unwrap().validate(&p),
            Err(Error::Stale)
        );
    }
    #[test]
    fn same_title_different_nodes_remains_ambiguous() {
        let mut b = CorpusBuilder::new();
        for node in [1, 2] {
            b.push(Address { node, role: 1 }, node, "A", "Lead", "q")
                .unwrap();
        }
        let c = b.finish().unwrap().load().unwrap();
        assert_eq!(c.find_title("A"), &[1, 2]);
        assert_eq!(
            c.lead_parts("A", None),
            Err(crate::LeadPartsError::AmbiguousTitle)
        );
    }
}
