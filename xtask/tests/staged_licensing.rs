//! Text-regression checks for the inactive draft, NOT legal validation.
//! A full reviewed license digest is still required on activation.
const LICENSE: &str = include_str!("../../docs/licensing-next/LICENSE");
const GUIDE: &str = include_str!("../../docs/licensing-next/LICENSING.md");
const COMMERCIAL: &str = include_str!("../../docs/licensing-next/COMMERCIAL-LICENSE.md");
const CLA: &str = include_str!("../../docs/licensing-next/CLA.md");
const PRIVACY: &str = include_str!("../../docs/licensing-next/CLA-PRIVACY.md");

const REQUIRED: &[&str] = &[
    "STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.",
    "Noncommercial Purposes privately without a time limit",
    "Such private use alone does not require publication",
    "It does not authorize a Commercial Purpose.",
    "no later than the first such distribution, access, or provision of functionality",
    "whether access is public, invitation-only, or limited to friends",
    "An intermediary or wrapper does not remove this obligation",
    "User Content is not Corresponding Source",
    "do not become User Content merely because they are encoded in an ORB",
    "first runs the Software on behalf of Your Group for Evaluation",
    "does not restart or create another Evaluation period",
    "separate written agreement signed by the Licensor and the commercial licensee",
    "must specify the applicable fees or royalties and payment conditions",
    "Unless that agreement expressly permits closed Modifications",
    "Payment or source publication alone, without a signed agreement, does not grant",
    "including fair use, fair dealing, or other mandatory exceptions",
    "grant each recipient directly, as their Modification Author",
    "No person grants rights it does not have",
    "does not grant the Project Licensor commercial relicensing rights in another author's portions",
    "does not by itself make that Modification exempt Third-Party Material",
    "Mere communication through an API or protocol does not by itself make a component a Modification",
    "This exception does not excuse withholding GEL RAM-derived source",
    "does not override the rights of Modification Authors",
    "rights terminated under this section are reinstated automatically upon full cure",
    "If these conditions are not met, reinstatement requires written confirmation",
    "does not reinstate a patent license terminated under section 9",
];

fn check(text: &str) -> Result<(), &'static str> {
    for phrase in REQUIRED {
        if !text.contains(phrase) {
            return Err(phrase);
        }
    }
    for obsolete in [
        "keep it private only during a temporary evaluation",
        "first accesses any GEL RAM release",
    ] {
        if text.contains(obsolete) {
            return Err("obsolete private-use or evaluation rule");
        }
    }
    Ok(())
}

#[test]
fn draft_keeps_selected_policy_boundaries() {
    assert_eq!(check(LICENSE), Ok(()));
}

#[test]
fn each_required_clause_omission_is_detected() {
    for phrase in REQUIRED {
        let edited = LICENSE.replacen(phrase, "[omitted]", 1);
        assert!(check(&edited).is_err(), "missing {phrase}");
    }
}

#[test]
fn rejects_reintroduced_private_deadline_and_page_view_clock() {
    for obsolete in [
        "keep it private only during a temporary evaluation",
        "first accesses any GEL RAM release",
    ] {
        assert!(check(&format!("{LICENSE}\n{obsolete}")).is_err());
    }
}

#[test]
fn guide_and_commercial_notice_explain_selected_boundaries() {
    for phrase in [
        "Solely private noncommercial use of a Modification has no publication deadline",
        "User Content is not Corresponding Source",
        "P2P node, invitation-only group or service for friends is not exempt",
        "Permission for closed shared or hosted Modifications must be express",
    ] {
        assert!(GUIDE.contains(phrase), "guide: {phrase}");
    }
    for phrase in [
        "not itself a commercial license grant",
        "The agreement must specify fees or royalties and payment conditions",
        "An unsolicited payment does not create a license",
        "Permission to keep such Modifications closed must be expressly negotiated",
    ] {
        assert!(COMMERCIAL.contains(phrase), "commercial notice: {phrase}");
    }
}

#[test]
fn keeps_contributor_ownership_bilateral_acceptance_and_no_retroactivity() {
    for phrase in [
        "The Contributor retains ownership",
        "not an assignment of copyright ownership",
        "Project Licensor has countersigned",
        "does not by itself become covered retroactively",
        "must be kept privately",
    ] {
        assert!(CLA.contains(phrase), "CLA: {phrase}");
    }
}

#[test]
fn privacy_draft_distinguishes_voluntary_submission_and_effective_acceptance() {
    for phrase in [
        "STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.",
        "Submitting a contribution and seeking a CLA are voluntary",
        "Ordinary\npermitted use of the Software or publication of a fork",
        "### Right to object",
        "### Automated decisions",
        "CI acknowledgement\nchecks do not establish identity",
    ] {
        assert!(PRIVACY.contains(phrase), "privacy: {phrase}");
    }
    assert!(!LICENSE.contains("rights may be reinstated automatically"));
}
