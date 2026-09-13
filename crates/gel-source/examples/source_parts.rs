//! Author-created PL/EN fixtures. No private files, network or model required.
use gel_source::{digest, hex, Corpus, Error, LeadPartsError};
fn main() {
    let rows = [
        (
            0,
            9,
            "Demo light",
            "Lead (2)",
            "The lamp is switched off in this fictional example.",
        ),
        (
            0,
            2,
            "Demo light",
            "Lead (1)",
            "This is a synthetic lamp, not a measured device.",
        ),
        (
            1,
            4,
            "Żółta latarnia",
            "WSTĘP (2)",
            "Druga część: zażółć gęślą jaźń — dokładny cytat.",
        ),
        (
            1,
            8,
            "Żółta latarnia",
            "WSTĘP (1)",
            "To autorski przykład testowy, nie opis rzeczywistego urządzenia.",
        ),
    ];
    let mut catalog = String::from("GELCORPUS082\n");
    let mut text = String::new();
    for (node, role, title, section, body) in rows {
        catalog += &format!(
            "{node}\t{role}\t{node}\t{}\t{}\t{}\t{}\t{}\n",
            text.len(),
            body.len(),
            hex(&digest(body.as_bytes())),
            hex(title.as_bytes()),
            hex(section.as_bytes())
        );
        text += body;
    }
    // Pins are computed here ONLY because these are our own built-in fixtures.
    let cp = digest(catalog.as_bytes());
    let tp = digest(text.as_bytes());
    let corpus = Corpus::load(catalog.as_bytes(), text.as_bytes(), cp, tp).unwrap();
    for title in ["Demo light", "Żółta latarnia"] {
        println!("TITLE={title:?}");
        for p in corpus.lead_parts(title, Some(2)).unwrap() {
            corpus.validate(&p).unwrap();
            let r = p.record();
            println!(
                "PART={:?} QUOTE={:?} SOURCE={}:{} ENTRY={} BYTES={}..{} SHA256={} GENERATION={}",
                r.section,
                p.quote(),
                r.address.node,
                r.address.role,
                r.entry,
                r.start,
                r.end,
                hex(&r.hash),
                hex(&p.root())
            );
        }
    }
    assert_eq!(
        corpus.lead_parts("Demo light", Some(3)),
        Err(LeadPartsError::CountMismatch {
            expected: 3,
            actual: 2
        })
    );
    println!("EXPECTED_PART_COUNT_MISMATCH=REJECTED");
    let mut changed = text.into_bytes();
    changed[0] ^= 1;
    assert_eq!(
        Corpus::load(catalog.as_bytes(), &changed, cp, tp).unwrap_err(),
        Error::Integrity
    );
    println!("MODIFIED_TEXT_WITH_ORIGINAL_PINS=REJECTED");
    println!("SOURCE_PARTS_E2E=PASS");
}
