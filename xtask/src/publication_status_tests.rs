#[test]
fn unapproved_candidate_can_be_tested_but_remains_unapproved() {
    assert!(!super::publication_status(
        "REVIEW_PUBLICATION_APPROVED=NO\nPUBLICATION_APPROVED=NO\n"
    )
    .unwrap());
    assert!(!super::publication_status(
        "REVIEW_PUBLICATION_APPROVED=YES\nPUBLICATION_APPROVED=NO\n"
    )
    .unwrap());
    assert!(super::publication_status(
        "REVIEW_PUBLICATION_APPROVED=YES\nPUBLICATION_APPROVED=YES\n"
    )
    .unwrap());
}

#[test]
fn ambiguous_or_missing_status_is_rejected() {
    for text in [
        "# REVIEW_PUBLICATION_APPROVED=YES\nPUBLICATION_APPROVED=YES",
        "REVIEW_PUBLICATION_APPROVED=NO\nPUBLICATION_APPROVED=YES",
        "REVIEW_PUBLICATION_APPROVED=YES\nPUBLICATION_APPROVED=maybe",
        "REVIEW_PUBLICATION_APPROVED=YES\nPUBLICATION_APPROVED=NO\nPUBLICATION_APPROVED=YES",
        "REVIEW_PUBLICATION_APPROVED=YES\nREVIEW_PUBLICATION_APPROVED=NO\nPUBLICATION_APPROVED=NO",
    ] {
        assert!(super::publication_status(text).is_err(), "{text}");
    }
}
