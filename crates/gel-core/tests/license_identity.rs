// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! The license constants of gel-core against the files they name, and the
//! exact `--version` / `--license` texts every GEL RAM tool prints.
use gel_core::{
    source_header_like, tool_identity, tool_license, LICENSE_ID, LICENSE_NAME, REQUIRED_NOTICE,
    SOURCE_HEADER, SOURCE_HEADER_LINE_1, SOURCE_HEADER_LINE_2, UTF8_BOM,
};

const LICENSE: &str = include_str!("../../../LICENSE");
const NOTICE: &str = include_str!("../../../NOTICE");
const LICENSE_MODE: &str = include_str!("../../../docs/LICENSE-MODE.txt");
const THIRD_PARTY_NOTICES: &str = include_str!("../../../THIRD-PARTY-NOTICES.md");

#[test]
fn license_name_is_the_first_line_of_license() {
    assert_eq!(LICENSE.lines().next(), Some(LICENSE_NAME));
}

#[test]
fn required_notice_is_a_whole_line_of_license_and_notice() {
    for (name, text) in [("LICENSE", LICENSE), ("NOTICE", NOTICE)] {
        assert!(
            text.lines().any(|line| line == REQUIRED_NOTICE),
            "{name} has no line equal to REQUIRED_NOTICE"
        );
    }
}

#[test]
fn source_header_is_the_license_name_and_the_required_notice() {
    assert_eq!(
        SOURCE_HEADER_LINE_1,
        format!(
            "// GEL RAM — {}; see LICENSE and NOTICE.",
            LICENSE.lines().next().unwrap()
        )
    );
    for (name, text) in [("LICENSE", LICENSE), ("NOTICE", NOTICE)] {
        assert!(
            text.lines().any(|line| line.starts_with("Required Notice:")
                && format!("// {line}") == SOURCE_HEADER_LINE_2),
            "{name} has no Required Notice line equal to line 2 of the source header"
        );
    }
    assert_eq!(
        SOURCE_HEADER,
        format!("{SOURCE_HEADER_LINE_1}\n{SOURCE_HEADER_LINE_2}\n")
    );
}

#[test]
fn license_id_is_the_active_license_mode() {
    let mode = LICENSE_MODE.lines().next().unwrap();
    assert_eq!(mode.split(" + ").next(), Some(LICENSE_ID), "{mode}");
}

#[test]
fn tool_texts_are_exact_and_use_only_notice_wording() {
    let identity = "GEL_TOOL=gel-x VERSION=1.2.3 LICENSE=GEL-RAM-NCRL-1.0";
    assert_eq!(tool_identity("gel-x", "1.2.3"), identity);
    let summary = "Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com";
    let third_party = "Third-party material in this program, including the Unicode data compiled into it, remains under its own license: see THIRD-PARTY-NOTICES.md.";
    assert_eq!(
        tool_license("gel-x", "1.2.3"),
        format!("{identity}\n{LICENSE_NAME}\n{REQUIRED_NOTICE}\n{summary}\n{third_party}\n")
    );
    // The summary adds no term: both of its sentences come from NOTICE.
    for phrase in [
        "Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the ",
        "Monetized Use requires a separate written Commercial Agreement",
        "gelram.licensing@gmail.com",
    ] {
        assert!(NOTICE.contains(phrase), "{phrase}");
    }
    // The third-party line names a file that says the Unicode tables are
    // compiled into the release binaries.
    assert!(THIRD_PARTY_NOTICES.contains("tables are compiled into the release binaries"));
}

#[test]
fn header_like_lines_are_plain_comments_naming_gel_ram_or_the_notice() {
    for line in [
        SOURCE_HEADER_LINE_1,
        SOURCE_HEADER_LINE_2,
        "//GEL RAM",
        "//  GEL RAM — tweaked",
        "// \tRequired Notice: someone else",
        "// GEL RAM — x\r",
    ] {
        assert!(source_header_like(line.as_bytes()), "{line:?}");
    }
    for line in [
        "//! GEL RAM crate docs.",
        "/// GEL RAM item.",
        "// Frozen code.",
        "fn main() {}",
        " // GEL RAM — indented",
        "// GEL  RAM — two spaces",
        "",
    ] {
        assert!(!source_header_like(line.as_bytes()), "{line:?}");
    }
    let bom_line = [UTF8_BOM, SOURCE_HEADER_LINE_1.as_bytes()].concat();
    assert!(!source_header_like(&bom_line));
    assert_eq!(UTF8_BOM, "\u{feff}".as_bytes());
}
