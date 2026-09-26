use gel_source::{
    collection::{Collection, CollectionBuilder},
    digest,
};

#[test]
fn staged_bank_matches_sequential_bytes_pins_and_readout() {
    for n in [0, 1, 8, 64, 256] {
        let mut ordinary = Collection::new();
        let mut staged = CollectionBuilder::new();
        for i in 0..n {
            let title = format!("Źródło {i}");
            let text = format!("Synthetic 🦀 evidence {i}.\r\nDo not\nopen the valve.\n");
            assert_eq!(ordinary.add(&title, &text), staged.add(&title, &text));
        }
        let mut built = staged.build();
        assert_eq!(built.to_bytes(), ordinary.to_bytes());
        assert_eq!(built.root(), ordinary.root());
        assert_eq!(built.root(), digest(&built.to_bytes()));
        let restored = Collection::from_bytes(&built.to_bytes(), built.root()).unwrap();
        assert_eq!(restored.to_bytes(), ordinary.to_bytes());
        assert_eq!(built.revision(), n);
        if n > 0 {
            let hits = built.search("open the valve").unwrap();
            assert_eq!(hits.matching_lines, n as usize);
            assert_eq!(
                hits.hits.len(),
                (n as usize).min(gel_source::collection::MAX_RESULTS)
            );
            built.replace(1, "Revised synthetic evidence.").unwrap();
            ordinary.replace(1, "Revised synthetic evidence.").unwrap();
            assert_eq!(built.to_bytes(), ordinary.to_bytes());
            assert_eq!(built.root(), ordinary.root());
        }
        assert_eq!(
            built.add("Next", "New data"),
            ordinary.add("Next", "New data")
        );
        assert_eq!(built.to_bytes(), ordinary.to_bytes());
        assert_eq!(built.root(), ordinary.root());
    }
}

#[test]
fn rejected_staged_document_does_not_advance_ids_revision_or_data() {
    let mut expected = Collection::new();
    let mut staged = CollectionBuilder::new();
    for (title, text) in [
        ("First", "One"),
        ("", "Invalid title"),
        ("Empty", ""),
        ("Control\n", "Invalid"),
        ("Second", "Two"),
    ] {
        assert_eq!(staged.add(title, text), expected.add(title, text));
    }
    let built = staged.build();
    assert_eq!(built.revision(), 2);
    assert_eq!(built.to_bytes(), expected.to_bytes());
    assert_eq!(built.root(), expected.root());
}

#[test]
fn builder_enforces_document_limit_before_publishing() {
    let mut staged = CollectionBuilder::new();
    for _ in 0..gel_source::collection::MAX_DOCUMENTS {
        staged.add("Small", "Synthetic").unwrap();
    }
    assert_eq!(
        staged.add("Extra", "Rejected").unwrap_err(),
        "COLLECTION_LIMIT"
    );
    let built = staged.build();
    assert_eq!(
        built.documents().count(),
        gel_source::collection::MAX_DOCUMENTS
    );
    assert_eq!(
        built.revision(),
        gel_source::collection::MAX_DOCUMENTS as u64
    );
    assert_eq!(built.root(), digest(&built.to_bytes()));
    assert!(Collection::from_bytes(&built.to_bytes(), built.root()).is_ok());
}
