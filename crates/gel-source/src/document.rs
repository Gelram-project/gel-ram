//! Bounded, source-extractive document search. No semantic truth or LLM claims.
use std::ops::Range;

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
    crate::nfc::nfc(s).to_lowercase().replace('ς', "σ")
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
/// Lowercases an NFC line into `out`, mapping both sigma forms to 'σ'. This is
/// `normalized_words_text` without its intermediate strings: `str::to_lowercase`
/// differs from per-character lowercasing only for 'Σ', and both of its
/// results become 'σ'; no other character lowercases to 'ς'. The per-character
/// lowercasing is `lowercase::lowercase_into`.
fn normalize_line_into(line: &str, out: &mut String, nfc: &mut crate::nfc::Scratch) {
    out.clear();
    crate::lowercase::lowercase_into(crate::nfc::nfc_in(line, nfc), out);
}

/// An admitted query: 1..=32 normalized words.
pub(crate) struct Query {
    words: Vec<String>,
    /// False when some word is not ASCII; such a word never equals a token of
    /// an all-ASCII line, so those lines cannot match.
    ascii_possible: bool,
}
impl Query {
    /// The query checks of `search`, in the same order, for a text that is
    /// within its limit.
    pub(crate) fn parse(phrase: &str) -> Result<Self, &'static str> {
        if phrase.len() > MAX_QUERY {
            return Err("DOCUMENT_SEARCH_LIMIT");
        }
        if phrase.chars().any(char::is_control) {
            return Err("DOCUMENT_QUERY_CONTROL");
        }
        let words: Vec<_> = words(phrase).collect();
        if words.is_empty() || words.len() > 32 {
            return Err("DOCUMENT_QUERY_WORDS");
        }
        let ascii_possible = words.iter().all(|w| w.is_ascii());
        Ok(Self {
            words,
            ascii_possible,
        })
    }
}

/// True when the tokens of an all-ASCII line contain the query words in order.
/// Tokens are the maximal runs of ASCII letters and digits, as in the split of
/// the line on every other character.
fn ascii_line_matches(line: &[u8], query: &[String]) -> bool {
    let n = query.len();
    let last = query[n - 1].as_bytes();
    let mut window: [&[u8]; 32] = [&[]; 32];
    let mut count = 0;
    let mut i = 0;
    while i < line.len() {
        if !line[i].is_ascii_alphanumeric() {
            i += 1;
            continue;
        }
        let start = i;
        while i < line.len() && line[i].is_ascii_alphanumeric() {
            i += 1;
        }
        let word = &line[start..i];
        window[count % n] = word;
        if count + 1 >= n
            && word.eq_ignore_ascii_case(last)
            && query[..n - 1].iter().enumerate().all(|(k, wanted)| {
                window[(count + 1 - n + k) % n].eq_ignore_ascii_case(wanted.as_bytes())
            })
        {
            return true;
        }
        count += 1;
    }
    false
}

/// Counts and the first `MAX_PASSAGES` line ranges of one piece of a text.
#[derive(Clone, Copy, Default)]
struct Partial {
    matching_lines: usize,
    skipped_long_lines: usize,
    passages: [(usize, usize); MAX_PASSAGES],
    kept: usize,
}

/// Searches the lines of `text[start..end]`. `start` and `end` are each 0, the
/// text length or a position just after a '\r' or '\n' byte, so the lines are
/// exactly the `split_inclusive(['\r', '\n'])` segments of the whole text.
/// Buffers one thread reuses for every non-ASCII line it searches.
#[derive(Default)]
struct LineBuffers {
    words: String,
    nfc: crate::nfc::Scratch,
}

fn scan(text: &str, start: usize, end: usize, query: &Query, buffer: &mut LineBuffers) -> Partial {
    let bytes = text.as_bytes();
    let mut part = Partial::default();
    let mut line_start = start;
    while line_start < end {
        // One pass finds the line end and whether the line is ASCII.
        let (line_end, ascii) = crate::line_end::find_ascii(bytes, line_start, end);
        let line = &bytes[line_start..line_end];
        if line.len() > MAX_LINE {
            part.skipped_long_lines += 1;
        } else if !line.is_empty() {
            let found = if ascii {
                query.ascii_possible && ascii_line_matches(line, &query.words)
            } else {
                normalize_line_into(
                    &text[line_start..line_end],
                    &mut buffer.words,
                    &mut buffer.nfc,
                );
                phrase_in_tokens(
                    buffer
                        .words
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|s| !s.is_empty()),
                    &query.words,
                    false,
                )
            };
            if found {
                part.matching_lines += 1;
                if part.kept < MAX_PASSAGES {
                    part.passages[part.kept] = (line_start, line_end);
                    part.kept += 1;
                }
            }
        }
        // Skip the one separator byte; a CRLF yields an empty line next.
        line_start = if line_end < end { line_end + 1 } else { end };
    }
    part
}

/// How a search is spread over threads.
#[derive(Clone, Copy)]
pub(crate) struct Plan {
    threads: usize,
    piece: usize,
}
/// Smaller inputs are searched on the calling thread.
const PARALLEL_MIN: usize = 1 << 20;
const MAX_THREADS: usize = 8;
const MIN_PIECE: usize = 64 << 10;
impl Plan {
    fn auto(total: usize) -> Self {
        let serial = Self {
            threads: 1,
            piece: usize::MAX,
        };
        if total < PARALLEL_MIN {
            return serial;
        }
        let available = std::thread::available_parallelism().map_or(1, |n| n.get());
        let threads = available.min(MAX_THREADS).min(total / (PARALLEL_MIN / 4));
        if threads <= 1 {
            return serial;
        }
        Self {
            threads,
            piece: (total / (threads * 4)).max(MIN_PIECE),
        }
    }
    #[cfg(test)]
    pub(crate) fn forced(threads: usize, piece: usize) -> Self {
        Self { threads, piece }
    }
}

/// Searches each text independently with an admitted query; `plan` None picks
/// one from the total size. The result is the same for every plan.
pub(crate) fn search_texts(texts: &[&str], query: &Query, plan: Option<Plan>) -> Vec<Search> {
    let plan = plan.unwrap_or_else(|| Plan::auto(texts.iter().map(|t| t.len()).sum()));
    // Pieces in source order: (text index, start, end), cut only after '\r' or '\n'.
    let mut pieces = Vec::new();
    for (index, text) in texts.iter().enumerate() {
        let bytes = text.as_bytes();
        let mut start = 0;
        while start < bytes.len() {
            let target = start.saturating_add(plan.piece.max(1));
            let end = if target >= bytes.len() {
                bytes.len()
            } else {
                bytes[target..]
                    .iter()
                    .position(|&b| b == b'\r' || b == b'\n')
                    .map_or(bytes.len(), |p| target + p + 1)
            };
            pieces.push((index, start, end));
            start = end;
        }
    }
    let threads = plan.threads.min(pieces.len());
    let partials: Vec<Partial> = if threads <= 1 {
        let mut buffer = LineBuffers::default();
        pieces
            .iter()
            .map(|&(i, s, e)| scan(texts[i], s, e, query, &mut buffer))
            .collect()
    } else {
        let next = std::sync::atomic::AtomicUsize::new(0);
        let mut slots = vec![Partial::default(); pieces.len()];
        std::thread::scope(|scope| {
            let workers: Vec<_> = (0..threads)
                .map(|_| {
                    scope.spawn(|| {
                        let mut buffer = LineBuffers::default();
                        let mut done = Vec::new();
                        loop {
                            let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            let Some(&(i, s, e)) = pieces.get(k) else {
                                break;
                            };
                            done.push((k, scan(texts[i], s, e, query, &mut buffer)));
                        }
                        done
                    })
                })
                .collect();
            for worker in workers {
                // A worker cannot panic short of a bug; propagate it if it does.
                for (k, part) in worker
                    .join()
                    .unwrap_or_else(|e| std::panic::resume_unwind(e))
                {
                    slots[k] = part;
                }
            }
        });
        slots
    };
    let mut out: Vec<Search> = texts
        .iter()
        .map(|_| Search {
            passages: Vec::new(),
            matching_lines: 0,
            skipped_long_lines: 0,
        })
        .collect();
    for (&(index, _, _), part) in pieces.iter().zip(&partials) {
        let result = &mut out[index];
        result.matching_lines += part.matching_lines;
        result.skipped_long_lines += part.skipped_long_lines;
        for &(s, e) in &part.passages[..part.kept] {
            if result.passages.len() < MAX_PASSAGES {
                result.passages.push(s..e);
            }
        }
    }
    out
}

pub fn search(text: &str, phrase: &str) -> Result<Search, &'static str> {
    if text.len() > MAX_TEXT {
        return Err("DOCUMENT_SEARCH_LIMIT");
    }
    let query = Query::parse(phrase)?;
    Ok(search_texts(&[text], &query, None)
        .pop()
        .unwrap_or_else(|| unreachable!("one text gives one result")))
}

/// The search of v0.6.1, kept as the reference for the differential tests.
#[cfg(test)]
pub(crate) fn search_reference(text: &str, phrase: &str) -> Result<Search, &'static str> {
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

/// A seeded text for the differential tests: ASCII and Unicode words (sigma,
/// decomposed marks, Hangul), every line ending, and lines around `MAX_LINE`.
#[cfg(test)]
pub(crate) fn sample_text(seed: &mut u64, lines: usize) -> String {
    const WORDS: [&str; 18] = [
        "Ada",
        "ada",
        "not",
        "approved",
        "a",
        "A",
        "x123",
        "ŁÓDŹ",
        "łódź",
        "café",
        "cafe\u{301}",
        "ΟΣ",
        "ος",
        "οσ",
        "한국",
        "\u{1100}\u{1161}",
        "ſ",
        "\u{212A}",
    ];
    const GAPS: [&str; 5] = [" ", ", ", " — ", "-", "  "];
    const ENDS: [&str; 7] = ["\n", "\r\n", "\r", "\n\n", "\r\r", "\n\r", ""];
    let mut next = |m: usize| {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*seed >> 33) as usize) % m
    };
    let mut text = String::new();
    for _ in 0..lines {
        match next(16) {
            0 => {
                // Around the line limit: MAX_LINE - 1, MAX_LINE, MAX_LINE + 1 bytes.
                let len = MAX_LINE - 1 + next(3);
                let head = if next(2) == 0 { "ada not " } else { "łódź " };
                text.push_str(head);
                text.push_str(&"x".repeat(len - head.len()));
            }
            1 => text.push_str("Ada not approved"),
            _ => {
                for w in 0..1 + next(8) {
                    if w > 0 {
                        text.push_str(GAPS[next(GAPS.len())]);
                    }
                    text.push_str(WORDS[next(WORDS.len())]);
                }
            }
        }
        text.push_str(ENDS[next(ENDS.len())]);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    const QUERIES: [&str; 22] = [
        "ada",
        "Ada not",
        "not approved",
        "ada not approved",
        "a a",
        "A",
        "X123",
        "łódź",
        "ŁÓDŹ łódź",
        "café",
        "cafe\u{301} ος",
        "οσ",
        "ΟΣ ΟΣ",
        "한국",
        "\u{AC00}",
        "ſ",
        "k",
        "\u{212A} ada",
        "missing token",
        "!!!",
        "a\nb",
        "",
    ];

    #[test]
    fn parallel_pieces_match_reference_search() {
        let mut seed = 0x5eed_u64;
        for case in 0..600 {
            let text = if case == 0 {
                String::new()
            } else {
                sample_text(&mut seed, 1 + case % 40)
            };
            for phrase in QUERIES {
                let expected = search_reference(&text, phrase);
                assert_eq!(search(&text, phrase), expected, "case {case} {phrase:?}");
                let Ok(query) = Query::parse(phrase) else {
                    continue;
                };
                for (threads, piece) in
                    [(1, 1), (2, 1), (3, 2), (4, 5), (7, 17), (5, 97), (3, 4097)]
                {
                    let mut got =
                        search_texts(&[&text], &query, Some(Plan::forced(threads, piece)));
                    assert_eq!(got.len(), 1);
                    assert_eq!(
                        Ok(got.pop().unwrap()),
                        expected,
                        "case {case} {phrase:?} threads {threads} piece {piece}"
                    );
                }
            }
        }
    }

    #[test]
    fn several_texts_match_reference_one_by_one() {
        let mut seed = 77_u64;
        for case in 0..100 {
            let texts: Vec<String> = (0..1 + case % 9)
                .map(|i| match (case + i) % 5 {
                    0 => String::new(),
                    1 => "Ada not approved\n".repeat(7 + i),
                    _ => sample_text(&mut seed, 1 + (case * 7 + i) % 30),
                })
                .collect();
            let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
            for phrase in QUERIES {
                let Ok(query) = Query::parse(phrase) else {
                    continue;
                };
                for plan in [None, Some(Plan::forced(4, 3)), Some(Plan::forced(2, 64))] {
                    let got = search_texts(&refs, &query, plan);
                    assert_eq!(got.len(), texts.len());
                    for (t, g) in texts.iter().zip(got) {
                        assert_eq!(Ok(g), search_reference(t, phrase), "case {case} {phrase:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn large_texts_take_the_parallel_path_and_match_reference() {
        let mut seed = 3_u64;
        let mut text = String::new();
        while text.len() < 3 << 20 {
            text.push_str(&sample_text(&mut seed, 64));
        }
        assert!(
            Plan::auto(text.len()).threads > 1
                || std::thread::available_parallelism().map_or(1, |n| n.get()) == 1
        );
        for phrase in ["ada not", "łódź", "οσ", "missing token", "x123"] {
            assert_eq!(
                search(&text, phrase),
                search_reference(&text, phrase),
                "{phrase:?}"
            );
        }
        // The largest admitted text, with hits only near its end and a long last line.
        let mut big = "filler line\r\n".repeat((MAX_TEXT - 64) / 13);
        big.push_str("ada NOT approved\n");
        big.push_str(&"y".repeat(MAX_TEXT - big.len()));
        assert_eq!(big.len(), MAX_TEXT);
        for phrase in ["ada not", "filler line", "y"] {
            assert_eq!(
                search(&big, phrase),
                search_reference(&big, phrase),
                "{phrase:?}"
            );
        }
        big.push('z');
        assert_eq!(search(&big, "ada"), search_reference(&big, "ada"));
    }

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
    /// The ASCII check of a line sees every byte before its end and none
    /// after it: a non-ASCII byte at each position of an eight-byte word,
    /// before and after every line end, with queries made of the ASCII pieces
    /// of non-ASCII words, which match only on the ASCII path.
    #[test]
    fn ascii_check_stops_at_the_line_end() {
        let phrases = ["za", "x za", "za x", "zaż", "caf", "cafe", "łódź", "x"];
        for pad in 0..20 {
            for tail in ["", "ż", "é", "\u{301}"] {
                for sep in ["\n", "\r\n", "\r", "\n\n"] {
                    for after in ["", "ł", "x", "żx"] {
                        let text = format!("{} za{tail}{sep}{after}za cafe{tail}", "x".repeat(pad));
                        for phrase in phrases {
                            let expected = search_reference(&text, phrase);
                            assert_eq!(search(&text, phrase), expected, "{text:?} {phrase:?}");
                            let query = Query::parse(phrase).unwrap();
                            for plan in [Plan::forced(2, 1), Plan::forced(3, 5)] {
                                let got = search_texts(&[&text], &query, Some(plan));
                                assert_eq!(Ok(&got[0]), expected.as_ref(), "{text:?} {phrase:?}");
                            }
                        }
                    }
                }
            }
        }
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
