//! Offline exact readout of a pinned, MIT-licensed Rust Book excerpt.
//! This is not semantic search, an ORB encoder, or a language model.
use gel_source::{hex, Corpus, Error, Hash, LeadPartsError};

const TITLE: &str = "Rust ownership";
const TEXT: &[u8] = include_bytes!("../fixtures/rust-book/ownership.txt");
const CATALOG: &[u8] = include_bytes!("../fixtures/rust-book/catalog.txt");
const TEXT_PIN: &str = "5284e31747fcb796ef577c64343c36d1627ac44a16b683a4dc4e81fa520dec71";
const CATALOG_PIN: &str = "f5e81e25d4f6f5c4ae1f01c7516ed8313a46c2821bf33731cb540ccd06d5a3a8";
const UPSTREAM: &str = "https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch04-01-what-is-ownership.md";
const UPSTREAM_START: usize = 23;
const EXPECTED_PARTS: usize = 3;

fn pin(value: &str) -> Hash {
    assert_eq!(value.len(), 64);
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[2 * i..2 * i + 2], 16).expect("compiled fixture pin");
    }
    out
}

fn load(catalog: &[u8], text: &[u8]) -> Result<Corpus, Error> {
    // Independent reviewed constants: NEVER trust a hash supplied with received data.
    Corpus::load(catalog, text, pin(CATALOG_PIN), pin(TEXT_PIN))
}

fn run(title: &str) -> Result<(), String> {
    let corpus = load(CATALOG, TEXT).map_err(|e| format!("fixture: {e:?}"))?;
    let parts = corpus
        .lead_parts(title, Some(EXPECTED_PARTS))
        .map_err(|e| format!("title/parts: {e:?}"))?;
    println!("TITLE={TITLE}\nSOURCE_URL={UPSTREAM}\nSOURCE_LICENSE=MIT");
    println!("TEXT_PIN={TEXT_PIN}\nCATALOG_PIN={CATALOG_PIN}");
    println!("SCOPE=exact excerpt readout, not truth certification or semantic retrieval");
    for passage in parts {
        corpus
            .validate(&passage)
            .map_err(|e| format!("quote: {e:?}"))?;
        let r = passage.record();
        println!(
            "PART={} ADDRESS={}:{} ENTRY={} EXCERPT_BYTES={}..{} UPSTREAM_BYTES={}..{} SHA256={} GENERATION={}\nQUOTE_BEGIN\n{}\nQUOTE_END",
            r.section, r.address.node, r.address.role, r.entry, r.start, r.end,
            UPSTREAM_START + r.start, UPSTREAM_START + r.end,
            hex(&r.hash), hex(&passage.root()), passage.quote()
        );
    }
    // A plausible same-length factual change must not pass the original approval.
    let mut changed = TEXT.to_vec();
    let at = changed
        .windows(4)
        .position(|s| s == b"Rust")
        .ok_or("fixture word missing")?;
    changed[at..at + 4].copy_from_slice(b"GEL!");
    if load(CATALOG, &changed).unwrap_err() != Error::Integrity {
        return Err("modified text was not rejected by integrity gate".into());
    }
    let mut changed_catalog = CATALOG.to_vec();
    changed_catalog[12] ^= 1;
    if load(&changed_catalog, TEXT).unwrap_err() != Error::Integrity {
        return Err("modified catalog was not rejected by integrity gate".into());
    }
    if corpus.lead_parts(TITLE, Some(4))
        != Err(LeadPartsError::CountMismatch {
            expected: 4,
            actual: 3,
        })
    {
        return Err("expected-count mismatch was not rejected".into());
    }
    println!("MODIFIED_TEXT=REJECTED\nMODIFIED_CATALOG=REJECTED\nEXPECTED_COUNT_MISMATCH=REJECTED\nSOURCE_REAL_E2E=PASS");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => run(TITLE),
        [arg] if arg == "--list" => {
            println!("{TITLE}");
            Ok(())
        }
        [arg] if arg == "--help" => {
            println!("source_real [\"Rust ownership\" | --list]\nOffline built-in MIT excerpt; no external files or model.");
            Ok(())
        }
        [title] if !title.starts_with('-') => run(title),
        _ => Err("usage: source_real [\"Rust ownership\" | --list | --help]".into()),
    };
    if let Err(error) = result {
        eprintln!("SOURCE_REAL_E2E=FAIL {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_fixture_pins_and_complete_demo() {
        assert_eq!(gel_source::hex(&gel_source::digest(TEXT)), TEXT_PIN);
        assert_eq!(gel_source::hex(&gel_source::digest(CATALOG)), CATALOG_PIN);
        run(TITLE).unwrap();
    }

    #[test]
    fn shuffled_catalog_returns_exact_ordered_paragraphs() {
        let c = load(CATALOG, TEXT).unwrap();
        let parts = c.lead_parts(TITLE, Some(3)).unwrap();
        assert_eq!(parts.len(), 3);
        for (i, p) in parts.iter().enumerate() {
            assert_eq!(p.record().section, format!("Lead ({})", i + 1));
            assert_eq!(
                p.quote().as_bytes(),
                &TEXT[p.record().start..p.record().end]
            );
            c.validate(p).unwrap();
        }
        assert_eq!(
            parts
                .iter()
                .map(|p| p.quote())
                .collect::<Vec<_>>()
                .join("\n\n")
                .as_bytes(),
            TEXT
        );
    }

    #[test]
    fn unknown_title_does_not_return_unrelated_text() {
        assert!(run("Rust garbage collection guarantees").is_err());
    }

    #[test]
    fn every_single_byte_mutation_rejected_with_original_pins() {
        for i in 0..TEXT.len() {
            let mut changed = TEXT.to_vec();
            changed[i] ^= 1;
            assert_eq!(load(CATALOG, &changed).unwrap_err(), Error::Integrity);
        }
        for i in 0..CATALOG.len() {
            let mut changed = CATALOG.to_vec();
            changed[i] ^= 1;
            assert_eq!(load(&changed, TEXT).unwrap_err(), Error::Integrity);
        }
    }

    #[test]
    fn missing_tail_and_invalid_utf8_rejected() {
        assert_eq!(
            load(CATALOG, &TEXT[..TEXT.len() - 1]).unwrap_err(),
            Error::Integrity
        );
        let mut changed = TEXT.to_vec();
        changed[0] = 255;
        assert_eq!(load(CATALOG, &changed).unwrap_err(), Error::Integrity);
    }
}
