//! The catalog loader before the bundle reader handed its buffer over: the
//! text was checked as `&str` and copied into the corpus. The bodies are
//! unchanged apart from their names (`Self` became `Corpus`). Tests compare
//! the current loaders with it.
use crate::{
    digest, distinct_ranges, label, number, quotes_match, unhex32, Address, Corpus, Error, Hash,
    QuoteCheck, Record, Rows, MAX_CATALOG, MAX_PASSAGE, MAX_RECORDS, MAX_TEXT,
};
use std::collections::BTreeMap;

pub(crate) fn load_unhashed_reference(
    catalog: &[u8],
    text: &[u8],
    catalog_pin: Hash,
    text_pin: Hash,
    threads: Option<usize>,
) -> Result<Corpus, Error> {
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
    // error that comes first. A range stated by several rows is hashed
    // once; whether all recorded digests match does not depend on that.
    let mut checks = Vec::new();
    let parsed = parse_rows_reference(raw, text, &mut checks);
    if !distinct_ranges(&mut checks) || !quotes_match(text.as_bytes(), &checks, threads) {
        return Err(Error::Integrity);
    }
    let (records, titles) = parsed?;
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
/// The catalog rows after the header, up to the first error other than a
/// passage digest mismatch; `checks` receives `(start, end, digest)` of
/// every passage whose digest the row order reaches before that error.
fn parse_rows_reference(
    raw: &str,
    text: &str,
    checks: &mut Vec<QuoteCheck>,
) -> Result<Rows, Error> {
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
