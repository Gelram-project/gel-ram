//! Synthetic, public E2E: title -> pinned passage -> source -> rejected modification.
#![forbid(unsafe_code)]
use gel_source::{digest, hex, Address, Corpus, Error};

const TITLE: &str = "Demo vessel";
const TEXT: &str =
    "Demo vessel pressure is 2 bar. This is synthetic test data. Łódź — próba Unicode.";
fn fixture() -> (String, Corpus) {
    let catalog = format!(
        "GELCORPUS082\n1\t0\t1\t0\t{}\t{}\t{}\t{}\n",
        TEXT.len(),
        hex(&digest(TEXT.as_bytes())),
        hex(TITLE.as_bytes()),
        hex(b"Lead")
    );
    // Pins originate in this trusted built-in fixture, never in an input's claim.
    let corpus = Corpus::load(
        catalog.as_bytes(),
        TEXT.as_bytes(),
        digest(catalog.as_bytes()),
        digest(TEXT.as_bytes()),
    )
    .unwrap();
    (catalog, corpus)
}
fn demo(title: &str) -> Result<(), String> {
    let (catalog, corpus) = fixture();
    let ids = corpus.find_title(title);
    let node = *ids
        .first()
        .ok_or("UNKNOWN: title not found (exact normalized title lookup)")?;
    let passage = corpus
        .quote(Address { node, role: 0 })
        .map_err(|e| format!("{e:?}"))?;
    corpus.validate(&passage).map_err(|e| format!("{e:?}"))?;
    println!(
        "TITLE: {}\nQUOTE: {}",
        passage.record().title,
        passage.quote()
    );
    println!(
        "SOURCE: built-in synthetic fixture / entry={} / section={} / byte_range={}..{}",
        passage.record().entry,
        passage.record().section,
        passage.record().start,
        passage.record().end
    );
    println!(
        "CATALOG_SHA256: {}\nTEXT_SHA256: {}",
        hex(&corpus.root()),
        hex(&corpus.text_hash())
    );
    let changed = TEXT.replace("2 bar", "9 bar");
    let rejected = Corpus::load(
        catalog.as_bytes(),
        changed.as_bytes(),
        corpus.root(),
        corpus.text_hash(),
    );
    if !matches!(rejected, Err(Error::Integrity)) {
        return Err("modified data was not rejected".into());
    }
    println!("TAMPER: 2 bar -> 9 bar; original pins retained; REJECTED\nSOURCE_E2E=PASS\nIntegrity proves approved bytes, NOT factual truth or semantic relevance.");
    Ok(())
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() > 1 {
        eprintln!("usage: source_readout [TITLE]");
        return std::process::ExitCode::from(2);
    }
    match demo(args.first().map_or(TITLE, String::as_str)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::from(2)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_and_tamper_e2e() {
        demo("DEMO VESSEL").unwrap();
    }
    #[test]
    fn absent_title_is_unknown() {
        assert!(demo("another vessel").unwrap_err().starts_with("UNKNOWN"));
    }
    #[test]
    fn exact_unicode_quote() {
        let (_, c) = fixture();
        assert_eq!(c.quote(Address { node: 1, role: 0 }).unwrap().quote(), TEXT);
    }
}
