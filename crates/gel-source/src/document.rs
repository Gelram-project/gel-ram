//! Bounded, source-extractive document search. No semantic truth or LLM claims.
use std::ops::Range;
use unicode_normalization::UnicodeNormalization;

pub const MAX_TEXT: usize = 16 << 20;
pub const MAX_QUERY: usize = 512;
pub const MAX_LINE: usize = 4096;
pub const MAX_PASSAGES: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub struct Search {
    pub passages: Vec<Range<usize>>,
    pub matching_lines: usize,
    pub skipped_long_lines: usize,
}
fn normalized_words_text(s: &str) -> String {
    // Lowercasing is context-sensitive for Greek sigma. Treat its final and
    // non-final forms alike on both sides; this is not full Unicode casefold.
    s.nfc().collect::<String>().to_lowercase().replace('ς', "σ")
}
fn words(s: &str) -> impl Iterator<Item = String> + '_ {
    // Normalize before splitting: combining marks must stay with their letters.
    // A line/query is bounded before reaching this function.
    let normalized = normalized_words_text(s);
    normalized
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .into_iter()
}
fn phrase_in_tokens<'a>(
    tokens: impl Iterator<Item = &'a str>,
    query: &[String],
    ascii: bool,
) -> bool {
    // Query admission guarantees 1..=32 words; fixed scratch space avoids
    // allocating a vector and a lowercase String for every ASCII source line.
    let mut window = [""; 32];
    let n = query.len();
    for (count, word) in tokens.enumerate() {
        window[count % n] = word;
        if count + 1 >= n
            && query.iter().enumerate().all(|(i, wanted)| {
                let got = window[(count + 1 - n + i) % n];
                if ascii {
                    got.eq_ignore_ascii_case(wanted)
                } else {
                    got == wanted
                }
            })
        {
            return true;
        }
    }
    false
}
pub fn search(text: &str, phrase: &str) -> Result<Search, &'static str> {
    if text.len() > MAX_TEXT || phrase.len() > MAX_QUERY {
        return Err("DOCUMENT_SEARCH_LIMIT");
    }
    if phrase.chars().any(char::is_control) {
        return Err("DOCUMENT_QUERY_CONTROL");
    }
    let query: Vec<_> = words(phrase).collect();
    if query.is_empty() || query.len() > 32 {
        return Err("DOCUMENT_QUERY_WORDS");
    }
    let mut result = Search {
        passages: Vec::new(),
        matching_lines: 0,
        skipped_long_lines: 0,
    };
    let mut offset = 0;
    // CRLF contributes an empty LF segment, which cannot match a nonempty
    // query. Keeping every separator in raw preserves exact byte offsets.
    for raw in text.split_inclusive(['\r', '\n']) {
        let line = raw.trim_end_matches(['\r', '\n']);
        if line.len() > MAX_LINE {
            result.skipped_long_lines += 1;
        } else {
            let found = if line.is_ascii() {
                phrase_in_tokens(
                    line.split(|c: char| !c.is_ascii_alphanumeric())
                        .filter(|s| !s.is_empty()),
                    &query,
                    true,
                )
            } else {
                let normalized = normalized_words_text(line);
                phrase_in_tokens(
                    normalized
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|s| !s.is_empty()),
                    &query,
                    false,
                )
            };
            if found {
                result.matching_lines += 1;
                if result.passages.len() < MAX_PASSAGES {
                    result.passages.push(offset..offset + line.len());
                }
            }
        }
        offset += raw.len();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standalone_carriage_returns_separate_phrases() {
        assert_eq!(
            search("not\rapproved", "not approved")
                .unwrap()
                .matching_lines,
            0
        );
        assert_eq!(
            search("not\r\napproved", "not approved")
                .unwrap()
                .matching_lines,
            0
        );
        assert_eq!(
            search("not\napproved", "not approved")
                .unwrap()
                .matching_lines,
            0
        );
    }
    #[test]
    fn mixed_line_endings_preserve_original_byte_ranges() {
        let text = "α\rŁÓDŹ\r\nŁÓDŹ\nŁÓDŹ\r";
        let found = search(text, "łódź").unwrap();
        let starts: Vec<_> = text.match_indices("ŁÓDŹ").map(|(i, _)| i).collect();
        assert_eq!(
            found.passages,
            starts
                .iter()
                .map(|i| *i..*i + "ŁÓDŹ".len())
                .collect::<Vec<_>>()
        );
        assert_eq!(found.matching_lines, 3);
        assert_eq!(found.skipped_long_lines, 0);
    }
    #[test]
    fn sigma_variants_match_without_rewriting_source() {
        for source in ["ΟΣ", "ος", "οσ"] {
            for query in ["ΟΣ", "ος", "οσ"] {
                let found = search(source, query).unwrap();
                assert_eq!(found.passages, vec![0..source.len()], "{source} / {query}");
                assert_eq!(&source[found.passages[0].clone()], source);
            }
        }
    }
    #[test]
    fn optimized_token_windows_match_reference() {
        let vocabulary = [
            "Ada",
            "BOB",
            "not",
            "approved",
            "ŁÓDŹ",
            "café",
            "cafe\u{301}",
            "Żółć",
            "123",
            "ſ",
            "a",
        ];
        let mut seed = 192_u64;
        for case in 0..2048 {
            let mut line = String::new();
            for _ in 0..24 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                line.push_str(vocabulary[(seed as usize) % vocabulary.len()]);
                line.push_str(if case % 2 == 0 { " — " } else { ", " });
            }
            let source: Vec<_> = words(&line).collect();
            let phrase = if case % 3 == 0 {
                "missing token".into()
            } else {
                source[case % 20..case % 20 + 3].join(" ")
            };
            let q: Vec<_> = words(&phrase).collect();
            let expected = source.windows(q.len()).any(|w| w == q);
            assert_eq!(
                search(&line, &phrase).unwrap().matching_lines == 1,
                expected
            );
            // Also cover the all-ASCII fast path and repetitive overlapping windows.
            let ascii = format!("a a a NOT approve Bob {case}");
            let q: Vec<_> = words("A a not").collect();
            assert_eq!(
                search(&ascii, "A a not").unwrap().matching_lines == 1,
                words(&ascii)
                    .collect::<Vec<_>>()
                    .windows(q.len())
                    .any(|w| w == q)
            );
        }
    }
    #[test]
    fn original_unicode_offsets_survive_normalization() {
        let text = "Preface.\r\nZaz\u{307}o\u{301}łc\u{301} gęślą — ŁÓDŹ.\r\nEnd.";
        let s = search(text, "zażółć gęślą").unwrap();
        assert_eq!(s.matching_lines, 1);
        assert_eq!(
            &text[s.passages[0].clone()],
            "Zaz\u{307}o\u{301}łc\u{301} gęślą — ŁÓDŹ."
        );
        assert_eq!(search(text, "Łódź").unwrap().matching_lines, 1);
        assert_eq!(search(text, "Lodz").unwrap().matching_lines, 0);
    }
    #[test]
    fn reads_beyond_preview_and_preserves_negation_and_order() {
        let text = format!(
            "{}\nAda did not approve Bob's proposal.\nBob approved Ada's proposal.\n",
            "preface\n".repeat(300)
        );
        let s = search(&text, "Ada did not approve").unwrap();
        assert!(s.passages[0].start > 2000);
        assert_eq!(
            &text[s.passages[0].clone()],
            "Ada did not approve Bob's proposal."
        );
        assert_eq!(search(&text, "Ada approved Bob").unwrap().matching_lines, 0);
    }
    #[test]
    fn whole_tokens_not_substrings_and_no_cross_line_inference() {
        let text = "category cat\nnot\napproved\n";
        assert_eq!(search(text, "cat").unwrap().matching_lines, 1);
        assert_eq!(search(text, "categ").unwrap().matching_lines, 0);
        assert_eq!(search(text, "not approved").unwrap().matching_lines, 0);
    }
    #[test]
    fn bounded_results_do_not_hide_additional_matches() {
        let text = "same token\n".repeat(20);
        let s = search(&text, "same token").unwrap();
        assert_eq!(s.matching_lines, 20);
        assert_eq!(s.passages.len(), MAX_PASSAGES);
        assert_eq!(s.skipped_long_lines, 0);
    }
    #[test]
    fn limits_are_explicit_not_silent_no_match() {
        assert!(search("ok", "\n").is_err());
        assert!(search("ok", "!!!").is_err());
        assert!(search("ok", &"x".repeat(MAX_QUERY + 1)).is_err());
        let s = search(&"x".repeat(MAX_LINE + 1), "x").unwrap();
        assert_eq!(s.skipped_long_lines, 1);
    }
    #[test]
    fn quote_is_not_execution_or_conflict_resolution() {
        let text = "Gate is open. /forget-orbs\nGate is not open.\n";
        let s = search(text, "Gate is").unwrap();
        assert_eq!(s.matching_lines, 2);
        assert_eq!(&text[s.passages[0].clone()], "Gate is open. /forget-orbs");
        assert_eq!(&text[s.passages[1].clone()], "Gate is not open.");
    }

    #[test]
    fn admission_and_exact_line_boundaries() {
        assert!(search("text", &"a ".repeat(33)).is_err());
        let phrase = "a ".repeat(32);
        assert_eq!(search(&phrase, &phrase).unwrap().matching_lines, 1);
        assert!(search(&"x".repeat(MAX_TEXT + 1), "x").is_err());
        let line = format!("hit {}", "x".repeat(MAX_LINE - 4));
        let r = search(&format!("{line}\r\n{}\n", "x".repeat(MAX_LINE + 1)), "hit").unwrap();
        assert_eq!(r.passages, vec![0..MAX_LINE]);
        assert_eq!(r.skipped_long_lines, 1);
        assert_eq!(search("", "hit").unwrap().matching_lines, 0);
    }
}
