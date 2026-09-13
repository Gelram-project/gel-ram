#![forbid(unsafe_code)]
use gel_source::*;
fn catalog(text: &[u8], title: &str) -> Vec<u8> {
    format!(
        "GELCORPUS082\n0\t0\t42\t0\t{}\t{}\t{}\t{}\n",
        text.len(),
        hex(&digest(text)),
        hex(title.as_bytes()),
        hex(b"Lead")
    )
    .into_bytes()
}
fn load(c: &[u8], t: &[u8]) -> Result<Corpus, Error> {
    Corpus::load(c, t, digest(c), digest(t))
}
#[test]
fn roundtrip_source_bytes_and_lowercased_title() {
    let t = "Zażółć — knowledge.\n".as_bytes();
    let c = catalog(t, "GEL");
    let b = load(&c, t).unwrap();
    assert_eq!(b.find_title(" gel "), [0]);
    let p = b.quote(Address { node: 0, role: 0 }).unwrap();
    assert_eq!(p.quote().as_bytes(), t);
    assert_eq!(b.validate(&p), Ok(()));
}
#[test]
fn payload_and_catalog_pins_are_both_required() {
    let t = b"text";
    let c = catalog(t, "A");
    assert_eq!(
        Corpus::load(&c, b"Text", digest(&c), digest(t)).unwrap_err(),
        Error::Integrity
    );
    let mut bad = c.clone();
    bad[13] = b'1';
    assert_eq!(
        Corpus::load(&bad, t, digest(&c), digest(t)).unwrap_err(),
        Error::Integrity
    );
}
#[test]
fn trusting_new_document_hash_does_not_hide_wrong_fragment_hash() {
    let c = catalog(b"old", "A");
    assert_eq!(load(&c, b"new").unwrap_err(), Error::Integrity);
}
#[test]
fn rejects_duplicate_address_and_node_source_disagreement() {
    let c = catalog(b"abc", "A");
    let row = std::str::from_utf8(&c[13..]).unwrap();
    let dup = format!("{}{}", std::str::from_utf8(&c).unwrap(), row);
    assert_eq!(load(dup.as_bytes(), b"abc").unwrap_err(), Error::Duplicate);
    let wrong = row.replacen("0\t0\t42", "0\t1\t43", 1);
    assert_eq!(
        load(
            format!("{}{}", std::str::from_utf8(&c).unwrap(), wrong).as_bytes(),
            b"abc"
        )
        .unwrap_err(),
        Error::Format
    );
}
#[test]
fn utf8_boundaries_range_and_canonical_numbers_are_checked() {
    let c = String::from_utf8(catalog("ąx".as_bytes(), "A")).unwrap();
    for bad in [
        c.replace("\t0\t3\t", "\t1\t2\t"),
        c.replace("\t0\t3\t", "\t9\t3\t"),
        c.replacen("\n0\t", "\n00\t", 1),
    ] {
        assert_eq!(
            load(bad.as_bytes(), "ąx".as_bytes()).unwrap_err(),
            Error::Format
        );
    }
}
#[test]
fn strict_header_suffix_hash_and_empty_catalog() {
    let c = catalog(b"abc", "A");
    for bad in [
        b"GELCORPUS082\n".to_vec(),
        c[..c.len() - 1].to_vec(),
        [c.clone(), b"\n".to_vec()].concat(),
        c.iter()
            .map(|b| if *b == b'\n' { b'\r' } else { *b })
            .collect(),
    ] {
        assert!(load(&bad, b"abc").is_err());
    }
}
#[test]
fn root_binds_metadata_even_for_identical_text() {
    let a = load(&catalog(b"abc", "A"), b"abc").unwrap();
    let b = load(&catalog(b"abc", "B"), b"abc").unwrap();
    let p = a.quote(Address { node: 0, role: 0 }).unwrap();
    assert_eq!(b.validate(&p), Err(Error::Stale));
}
#[test]
fn duplicate_titles_stay_ambiguous_and_missing_role_fails() {
    let c = String::from_utf8(catalog(b"abc", "A")).unwrap();
    let row = c[13..].replacen("0\t0\t42", "1\t0\t43", 1);
    let b = load((c + &row).as_bytes(), b"abc").unwrap();
    assert_eq!(b.find_title("A"), [0, 1]);
    assert_eq!(b.quote(Address { node: 0, role: 1 }), Err(Error::Missing));
}
#[test]
fn limits_and_control_labels() {
    assert_eq!(
        load(&vec![0; MAX_CATALOG + 1], b"x").unwrap_err(),
        Error::Limit
    );
    let t = vec![b'x'; MAX_PASSAGE + 1];
    assert_eq!(load(&catalog(&t, "A"), &t).unwrap_err(), Error::Limit);
    assert_eq!(
        load(&catalog(b"x", "bad\nlabel"), b"x").unwrap_err(),
        Error::Format
    );
}

#[test]
fn large_ambiguous_title_preserves_order_without_duplicate_ids() {
    use std::fmt::Write;
    let text = b"x";
    let hash = hex(&digest(text));
    let mut c = String::from("GELCORPUS082\n");
    let ids: Vec<_> = (0..10_000u64).rev().collect();
    for &node in &ids {
        for role in 0..2 {
            writeln!(c, "{node}\t{role}\t{node}\t0\t1\t{hash}\t41\t42").unwrap();
        }
    }
    let corpus = load(c.as_bytes(), text).unwrap();
    assert_eq!(corpus.find_title("a"), ids);
    assert_eq!(corpus.records().count(), 20_000);
    assert_eq!(corpus.sections(9999).count(), 2);
}

#[test]
fn record_limit_is_enforced_and_labels_cannot_inject_controls() {
    use std::fmt::Write;
    let mut c = String::from("GELCORPUS082\n");
    let hash = hex(&digest(b"x"));
    for node in 0..MAX_RECORDS {
        writeln!(c, "{node}\t0\t{node}\t0\t1\t{hash}\t41\t42").unwrap();
    }
    assert_eq!(
        load(c.as_bytes(), b"x").unwrap().records().count(),
        MAX_RECORDS
    );
    writeln!(c, "{MAX_RECORDS}\t0\t{MAX_RECORDS}\t0\t1\t{hash}\t41\t42").unwrap();
    assert_eq!(load(c.as_bytes(), b"x").unwrap_err(), Error::Limit);
    for label in ["\u{1b}[31m", "x\0y", "a\tb"] {
        assert_eq!(
            load(&catalog(b"x", label), b"x").unwrap_err(),
            Error::Format
        );
    }
}

#[test]
fn approved_content_is_returned_as_data_not_interpreted() {
    let text = b"<script>synthetic_fixture()</script>\x1b[31m";
    let corpus = load(&catalog(text, "Synthetic"), text).unwrap();
    assert_eq!(
        corpus
            .quote(Address { node: 0, role: 0 })
            .unwrap()
            .quote()
            .as_bytes(),
        text
    );
}
