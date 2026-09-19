//! Build a public source catalog without private banks or an LLM.
use gel_source::{Address, Corpus, CorpusBuilder, Error};
fn main() {
    let mut writer = CorpusBuilder::new();
    for (i, quote) in [
        "Exact text: Zażółć gęślą jaźń.\n",
        "Unicode stays unchanged: cafe\u{301}, café, 🦀.\n",
    ]
    .iter()
    .enumerate()
    {
        writer
            .push(
                Address {
                    node: 1,
                    role: (i + 1) as u64,
                },
                1,
                "Unicode notebook",
                &format!("Lead ({})", i + 1),
                quote,
            )
            .unwrap();
    }
    let encoded = writer.finish().unwrap();
    let corpus = encoded.load().unwrap();
    for part in corpus.lead_parts("Unicode notebook", Some(2)).unwrap() {
        corpus.validate(&part).unwrap();
        println!("{}: {}", part.record().section, part.quote());
    }
    println!(
        "TEXT_BYTES={} CATALOG_BYTES={}",
        encoded.text_bytes().len(),
        encoded.catalog_bytes().len()
    );
    let mut changed = encoded.text_bytes().to_vec();
    changed[0] ^= 1;
    assert!(matches!(
        Corpus::load(
            encoded.catalog_bytes(),
            &changed,
            encoded.catalog_pin(),
            encoded.text_pin()
        ),
        Err(Error::Integrity)
    ));
    println!("MODIFIED_TEXT=REJECTED\nSOURCE_BUILD_E2E=PASS");
    println!("SCOPE=exact UTF-8 source catalog; not Q8 quantization or semantic understanding");
}
