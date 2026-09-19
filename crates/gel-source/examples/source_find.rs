//! Bounded phrase extraction from an independently pinned source, not semantic AI.
use gel_source::{digest, document, hex, Corpus, Hash};
use std::time::Instant;

const TEXT: &[u8] = include_bytes!("../fixtures/rust-book/ownership.txt");
const CATALOG: &[u8] = include_bytes!("../fixtures/rust-book/catalog.txt");
const TEXT_PIN: &str = "5284e31747fcb796ef577c64343c36d1627ac44a16b683a4dc4e81fa520dec71";
const CATALOG_PIN: &str = "f5e81e25d4f6f5c4ae1f01c7516ed8313a46c2821bf33731cb540ccd06d5a3a8";

fn pin(value: &str) -> Hash {
    let mut hash = [0; 32];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[2 * i..2 * i + 2], 16).expect("compiled pin");
    }
    hash
}

// This is a renderer, not source rewriting: byte ranges always refer to TEXT.
fn terminal_safe(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_control()
            || matches!(c, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
        {
            out.extend(c.escape_unicode());
        } else {
            out.push(c);
        }
    }
    out
}

fn approved(text: &[u8], catalog: &[u8]) -> Result<(), String> {
    let corpus = Corpus::load(catalog, text, pin(CATALOG_PIN), pin(TEXT_PIN))
        .map_err(|e| format!("source gate: {e:?}"))?;
    for passage in corpus
        .lead_parts("Rust ownership", Some(3))
        .map_err(|e| format!("parts: {e:?}"))?
    {
        corpus
            .validate(&passage)
            .map_err(|e| format!("passage: {e:?}"))?;
    }
    Ok(())
}

fn run(phrase: &str) -> Result<(), String> {
    let total = Instant::now();
    approved(TEXT, CATALOG)?;
    let text = std::str::from_utf8(TEXT).map_err(|e| e.to_string())?;
    let kernel = Instant::now();
    let result = document::search(text, phrase)?;
    let search_ns = kernel.elapsed().as_nanos();
    let verified_search_ns = total.elapsed().as_nanos();
    println!("SOURCE=Rust Book ownership excerpt\nSOURCE_LICENSE=MIT\nSOURCE_SHA256={TEXT_PIN}");
    println!("SOURCE_EXTRACT_ONLY; phrase lookup, not semantic truth or an ORB search");
    println!(
        "matching_lines={} shown={} skipped_long_lines={}",
        result.matching_lines,
        result.passages.len(),
        result.skipped_long_lines
    );
    for span in &result.passages {
        let quote = text.get(span.clone()).ok_or("invalid source range")?;
        println!(
            "UTF8 {}..{} | SOURCE_BOUND_QUOTE\n{}",
            span.start,
            span.end,
            terminal_safe(quote)
        );
    }
    if result.skipped_long_lines > 0 {
        println!("INCOMPLETE_SEARCH: overlong lines were skipped");
    }
    if result.matching_lines == 0 {
        println!("UNKNOWN: phrase not found in examined lines; not proof of falsehood");
    }
    println!("search_ns={search_ns} verified_search_ns={verified_search_ns}");
    println!("TIMING_SCOPE=source validation plus search; excludes startup and printing; not a benchmark");
    let mut changed = TEXT.to_vec();
    changed[0] ^= 1;
    if approved(&changed, CATALOG).is_ok() || hex(&digest(TEXT)) != TEXT_PIN {
        return Err("integrity rejection failed".into());
    }
    println!("MODIFIED_SOURCE=REJECTED\nSOURCE_FIND_E2E=PASS");
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => run("ownership"),
        [arg] if arg == "--help" => {
            println!("source_find [\"phrase\"]\nPinned real-source fixture; no model, network or private data.");
            Ok(())
        }
        [arg] => run(arg),
        _ => Err("usage: source_find [\"phrase\"]".into()),
    };
    if let Err(error) = result {
        eprintln!("SOURCE_FIND_E2E=FAIL {}", terminal_safe(&error));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_source_and_actual_cli_path() {
        run("ownership").unwrap();
        run("a nonexistent phrase").unwrap();
        assert!(run("\x1b").is_err());
    }

    #[test]
    fn exact_original_spans_are_recoverable() {
        approved(TEXT, CATALOG).unwrap();
        let text = std::str::from_utf8(TEXT).unwrap();
        let r = document::search(text, "garbage collection").unwrap();
        assert_eq!(r.matching_lines, 1);
        assert_eq!(
            &text[r.passages[0].clone()],
            "Some languages have garbage collection that regularly looks for no-longer-used"
        );
    }

    #[test]
    fn changes_cannot_bypass_pinned_gate() {
        for i in 0..TEXT.len() {
            let mut changed = TEXT.to_vec();
            changed[i] ^= 1;
            assert!(approved(&changed, CATALOG).is_err());
        }
        let mut catalog = CATALOG.to_vec();
        catalog[0] ^= 1;
        assert!(approved(TEXT, &catalog).is_err());
    }

    #[test]
    fn terminal_controls_are_visible_not_executed() {
        let input = "café\x1b[2J\r\u{202e}spoof\u{2066}\n";
        let rendered = terminal_safe(input);
        assert!(rendered.starts_with("café\\u{1b}[2J"));
        assert!(rendered.contains("\\u{202e}"));
        assert!(rendered.contains("\\u{2066}"));
        assert!(!rendered.chars().any(char::is_control));
    }
}
