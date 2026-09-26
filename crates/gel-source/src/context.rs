//! Bounded surrounding source bytes, never a claim of a complete sentence.
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    pub match_span: Range<usize>,
    pub context_span: Range<usize>,
    pub omitted_before: bool,
    pub omitted_after: bool,
}

/// Return up to radius bytes on either side, preserving UTF-8 and source offsets.
/// Callers must authenticate the document and the match before rendering.
pub fn surrounding(
    text: &str,
    matched: Range<usize>,
    radius: usize,
) -> Result<Context, &'static str> {
    if radius > 4096
        || matched.is_empty()
        || matched.len() > 4096
        || text.get(matched.clone()).is_none()
    {
        return Err("CONTEXT_RANGE_OR_LIMIT");
    }
    let mut start = matched.start.saturating_sub(radius);
    let mut end = matched.end.saturating_add(radius).min(text.len());
    while !text.is_char_boundary(start) {
        start += 1;
    }
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    // Avoid filling the first screen with blank padding; keep the match intact.
    while start < matched.start && text.as_bytes()[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > matched.end && text.as_bytes()[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    Ok(Context {
        match_span: matched,
        context_span: start..end,
        omitted_before: start != 0,
        omitted_after: end != text.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_negation_condition_unit_and_heading_across_mixed_lines() {
        for text in [
            "None of the features\r\nof ownership will slow down.",
            "Only if approved:\nrelease is allowed.",
            "Units: milligrams\rdose 10",
            "# Unverified claim\r\nproposed result",
        ] {
            let start = text.rfind(['\r', '\n']).unwrap() + 1;
            let c = surrounding(text, start..text.len(), 512).unwrap();
            assert_eq!(c.context_span, 0..text.len());
            assert_eq!(&text[c.match_span], &text[start..]);
            assert!(!c.omitted_before && !c.omitted_after);
        }
    }
    #[test]
    fn long_context_is_bounded_and_honestly_truncated() {
        let text = format!("{}HIT{}", "ż".repeat(10000), "🦀".repeat(10000));
        let c = surrounding(&text, 20000..20003, 511).unwrap();
        assert!(c.omitted_before && c.omitted_after);
        assert!(c.context_span.len() <= 1025);
        assert_eq!(&text[c.match_span.clone()], "HIT");
        assert!(text.get(c.context_span).is_some());
    }
    #[test]
    fn rejects_invalid_ranges_and_excessive_budget() {
        for span in [0..1, Range { start: 2, end: 1 }, 0..9, 0..0] {
            assert!(surrounding("ż", span, 16).is_err());
        }
        assert!(surrounding("hit", 0..3, 4097).is_err());
    }
}
