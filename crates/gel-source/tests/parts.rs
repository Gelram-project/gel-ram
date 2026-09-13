use gel_source::{digest, hex, Corpus, Error, LeadPartsError as E, MAX_RECORDS};

#[test]
fn overlapping_ranges_cannot_amplify_total_quotes_beyond_text_budget() {
    let text = vec![b'x'; gel_source::MAX_PASSAGE];
    let mut cat = String::from("GELCORPUS082\n");
    let hash = hex(&digest(&text));
    for part in 1..=(gel_source::MAX_TEXT / text.len() + 1) {
        cat += &format!(
            "0\t{part}\t0\t0\t{}\t{hash}\t{}\t{}\n",
            text.len(),
            hex(b"Demo"),
            hex(format!("Lead ({part})").as_bytes())
        );
    }
    let c = Corpus::load(cat.as_bytes(), &text, digest(cat.as_bytes()), digest(&text)).unwrap();
    assert_eq!(c.lead_parts("Demo", None), Err(E::OutputLimit));
}

fn fixture(rows: &[(u64, u64, &str, &str, &str)]) -> (Vec<u8>, Vec<u8>) {
    let mut catalog = String::from("GELCORPUS082\n");
    let mut text = Vec::new();
    for &(node, role, title, section, body) in rows {
        catalog += &format!(
            "{node}\t{role}\t{node}\t{}\t{}\t{}\t{}\t{}\n",
            text.len(),
            body.len(),
            hex(&digest(body.as_bytes())),
            hex(title.as_bytes()),
            hex(section.as_bytes())
        );
        text.extend_from_slice(body.as_bytes());
    }
    (catalog.into_bytes(), text)
}
fn load(rows: &[(u64, u64, &str, &str, &str)]) -> Corpus {
    let (c, t) = fixture(rows);
    Corpus::load(&c, &t, digest(&c), digest(&t)).unwrap()
}
fn labels(names: &[&str]) -> Corpus {
    let rows: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(i, s)| (0, i as u64, "Demo", *s, "own synthetic text"))
        .collect();
    load(&rows)
}
#[test]
fn orders_by_number_not_catalog_role_or_offset() {
    for family in ["Lead", "WSTĘP"] {
        let a = format!("{family} (1)");
        let b = format!("{family} (2)");
        let c = load(&[
            (7, 1, "Żółć & knowledge", &b, "Druga — café 🦀."),
            (7, 900, "Żółć & knowledge", &a, "First: don’t guess."),
            (7, 2, "Żółć & knowledge", "History", "unrelated"),
        ]);
        let p = c.lead_parts(" żółć & KNOWLEDGE ", Some(2)).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].quote(), "First: don’t guess.");
        assert_eq!(p[1].quote(), "Druga — café 🦀.");
        assert_eq!(p[0].record().address.role, 900);
        assert!(p[0].record().start > p[1].record().start);
        for passage in p {
            assert_eq!(passage.root(), c.root());
            assert_eq!(passage.record().hash, digest(passage.quote().as_bytes()));
            assert_eq!(passage.record().entry, 7);
            assert_eq!(c.validate(&passage), Ok(()));
        }
    }
}
#[test]
fn single_and_numbered_one_are_supported() {
    for s in ["Lead", "WSTĘP", "Lead (1)", "WSTĘP (1)"] {
        assert_eq!(labels(&[s]).lead_parts("Demo", Some(1)).unwrap().len(), 1);
    }
}
#[test]
fn missing_and_ambiguous_titles_are_not_guessed() {
    assert_eq!(
        labels(&["Lead"]).lead_parts("absent", None),
        Err(E::MissingTitle)
    );
    let c = load(&[(0, 0, "Demo", "Lead", "one"), (1, 0, "DEMO", "Lead", "two")]);
    assert_eq!(c.lead_parts("demo", None), Err(E::AmbiguousTitle));
    assert_eq!(
        labels(&["History"]).lead_parts("Demo", None),
        Err(E::MissingLead)
    );
}
#[test]
fn rejects_duplicate_and_mixed_families_or_layouts() {
    for names in [["Lead (1)", "Lead (1)"], ["WSTĘP", "WSTĘP"]] {
        assert_eq!(
            labels(&names).lead_parts("Demo", None),
            Err(E::DuplicatePart)
        );
    }
    for names in [
        ["Lead", "Lead (1)"],
        ["Lead (1)", "WSTĘP (2)"],
        ["Lead", "WSTĘP"],
    ] {
        assert_eq!(labels(&names).lead_parts("Demo", None), Err(E::MixedLayout));
    }
}
#[test]
fn rejects_noncanonical_and_overflow_numbering() {
    for s in [
        "Lead (0)",
        "Lead (01)",
        "Lead (+1)",
        "Lead (-1)",
        "Lead (١)",
        "Lead (１)",
        "Lead (1.0)",
        "Lead (1 )",
        "Lead ( 1)",
        "Lead(1)",
        "Lead  (1)",
        "Lead (1",
        "Lead (1) extra",
        "Lead (999999999999999999999999999)",
        "Lead (50001)",
        "WSTĘP (2x)",
    ] {
        assert_eq!(
            labels(&[s]).lead_parts("Demo", None),
            Err(E::InvalidNumbering),
            "{s}"
        );
    }
}
#[test]
fn detects_initial_and_internal_gaps() {
    for names in [vec!["Lead (2)"], vec!["Lead (1)", "Lead (3)"]] {
        assert_eq!(labels(&names).lead_parts("Demo", None), Err(E::Gap));
    }
}
#[test]
fn missing_tail_needs_independent_expected_count() {
    let c = labels(&["Lead (1)", "Lead (2)"]);
    assert!(c.lead_parts("Demo", None).is_ok());
    assert_eq!(
        c.lead_parts("Demo", Some(3)),
        Err(E::CountMismatch {
            expected: 3,
            actual: 2
        })
    );
    assert_eq!(
        c.lead_parts("Demo", Some(1)),
        Err(E::CountMismatch {
            expected: 1,
            actual: 2
        })
    );
    for n in [0, MAX_RECORDS + 1, usize::MAX] {
        assert_eq!(c.lead_parts("Demo", Some(n)), Err(E::InvalidExpectedCount));
    }
}
#[test]
fn exact_unicode_not_accent_stripping_or_normalization() {
    let c = load(&[(0, 0, "Żółć", "Lead", "e\u{301} ≠ é — quoted exactly")]);
    assert_eq!(c.lead_parts("Zolc", None), Err(E::MissingTitle));
    assert_eq!(c.lead_parts("Z\u{307}ółć", None), Err(E::MissingTitle));
    assert_eq!(
        c.lead_parts("ŻÓŁĆ", None).unwrap()[0].quote(),
        "e\u{301} ≠ é — quoted exactly"
    );
}
#[test]
fn refuses_changed_pins_and_cross_generation_passages() {
    let rows = [
        (0, 0, "Demo", "Lead (1)", "A"),
        (0, 1, "Demo", "Lead (2)", "B"),
    ];
    let (c, t) = fixture(&rows);
    let original = load(&rows);
    let mut bad = t.clone();
    bad[0] = b'Z';
    assert_eq!(
        Corpus::load(&c, &bad, digest(&c), digest(&t)).unwrap_err(),
        Error::Integrity
    );
    let replacement = load(&[
        (0, 0, "Demo", "Lead (1)", "Z"),
        (0, 1, "Demo", "Lead (2)", "B"),
    ]);
    for p in original.lead_parts("Demo", None).unwrap() {
        assert_eq!(replacement.validate(&p), Err(Error::Stale));
    }
}
#[test]
fn arbitrary_other_sections_do_not_enter_the_lead() {
    let c = labels(&["Lead poisoning", "Leadership", "History", "Lead (1)"]);
    assert_eq!(c.lead_parts("Demo", Some(1)).unwrap().len(), 1);
}
