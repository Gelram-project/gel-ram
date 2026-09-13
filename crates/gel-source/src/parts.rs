//! Ordered exact passages, not generated summaries or semantic retrieval.
use crate::{Address, Corpus, Error, Passage, MAX_RECORDS, MAX_TEXT};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeadPartsError {
    MissingTitle,
    AmbiguousTitle,
    MissingLead,
    MixedLayout,
    InvalidNumbering,
    DuplicatePart,
    Gap,
    InvalidExpectedCount,
    OutputLimit,
    CountMismatch { expected: usize, actual: usize },
    Source(Error),
}

// Parenthesis-shaped labels in these reserved families must be canonical.
// Unrelated section names are ignored. No accent removal or Unicode guessing.
fn classify(label: &str) -> Result<Option<(usize, Option<usize>)>, LeadPartsError> {
    for (family, base) in ["Lead", "WSTĘP"].iter().enumerate() {
        if label == *base {
            return Ok(Some((family, None)));
        }
        if let Some(suffix) = label.strip_prefix(base) {
            if suffix.trim_start().starts_with('(') {
                let number = suffix
                    .strip_prefix(" (")
                    .and_then(|s| s.strip_suffix(')'))
                    .ok_or(LeadPartsError::InvalidNumbering)?;
                let n: usize = number
                    .parse()
                    .map_err(|_| LeadPartsError::InvalidNumbering)?;
                if n == 0 || n > MAX_RECORDS || number != n.to_string() {
                    return Err(LeadPartsError::InvalidNumbering);
                }
                return Ok(Some((family, Some(n))));
            }
        }
    }
    Ok(None)
}

impl Corpus {
    /// Read one exact, unambiguous title's Lead/WSTĘP passages in part order.
    /// Accepts a single unnumbered label OR one numbered family starting at1.
    /// `expected_parts` must be independently known to detect a missing final part.
    /// Without it, success proves only that the AVAILABLE numbering has no gaps.
    /// Returns separate source-bound passages; never concatenates their text.
    pub fn lead_parts(
        &self,
        title: &str,
        expected_parts: Option<usize>,
    ) -> Result<Vec<Passage>, LeadPartsError> {
        if expected_parts.is_some_and(|n| n == 0 || n > MAX_RECORDS) {
            return Err(LeadPartsError::InvalidExpectedCount);
        }
        let node = match self.find_title(title) {
            [] => return Err(LeadPartsError::MissingTitle),
            [node] => *node,
            _ => return Err(LeadPartsError::AmbiguousTitle),
        };
        let mut layout = None;
        let mut parts: BTreeMap<usize, Address> = BTreeMap::new();
        let mut output_bytes = 0usize;
        for record in self.sections(node) {
            let Some((family, number)) = classify(&record.section)? else {
                continue;
            };
            let current = (family, number.is_some());
            if layout.is_some_and(|old| old != current) {
                return Err(LeadPartsError::MixedLayout);
            }
            layout = Some(current);
            output_bytes = output_bytes
                .checked_add(record.end - record.start)
                .ok_or(LeadPartsError::OutputLimit)?;
            if output_bytes > MAX_TEXT {
                return Err(LeadPartsError::OutputLimit);
            }
            if parts.insert(number.unwrap_or(1), record.address).is_some() {
                return Err(LeadPartsError::DuplicatePart);
            }
        }
        if parts.is_empty() {
            return Err(LeadPartsError::MissingLead);
        }
        if parts.keys().copied().ne(1..=parts.len()) {
            return Err(LeadPartsError::Gap);
        }
        if let Some(expected) = expected_parts {
            if expected != parts.len() {
                return Err(LeadPartsError::CountMismatch {
                    expected,
                    actual: parts.len(),
                });
            }
        }
        parts
            .values()
            .map(|address| self.quote(*address).map_err(LeadPartsError::Source))
            .collect()
    }
}
