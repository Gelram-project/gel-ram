// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! gel-license: exact stdout and exit codes of every command.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const LICENSE: &[u8] = include_bytes!("../../../LICENSE");
const NOTICE: &[u8] = include_bytes!("../../../NOTICE");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const NOTE: &str =
    "NOTE=answers are your own; this program reads the license, it does not interpret your facts";
const CHECK_TAIL: &str = "NOTE=this reads only the items listed above; it does not show that every copyright and third-party notice was kept, nor that any use is licensed\nCONTACT=gelram.licensing@gmail.com\nLEGAL_CERTIFICATION=NO\n";
const KEYS: [&str; 7] = [
    "money",
    "customer_production",
    "hosted",
    "paid_work",
    "cost_reduction",
    "evaluation_only",
    "noncommercial_purpose",
];

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "gel-license-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn put(&self, rel: &str, bytes: &[u8]) -> String {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path.to_str().unwrap().to_string()
    }
    fn dir(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn tool(args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_gel-license"))
        .args(args)
        .output()
        .unwrap();
    (
        o.status.code().unwrap(),
        String::from_utf8(o.stdout).unwrap(),
        String::from_utf8(o.stderr).unwrap(),
    )
}

#[test]
fn version_and_license_are_exact() {
    assert_eq!(
        tool(&["--version"]),
        (
            0,
            format!("GEL_TOOL=gel-license VERSION={VERSION} LICENSE=GEL-RAM-NCRL-1.0\n"),
            String::new()
        )
    );
    assert_eq!(
        tool(&["--license"]),
        (
            0,
            format!(
                "GEL_TOOL=gel-license VERSION={VERSION} LICENSE=GEL-RAM-NCRL-1.0\n\
                 GEL RAM Noncommercial Reciprocal License 1.0\n\
                 Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)\n\
                 Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com\n\
                 Third-party material in this program, including the Unicode data compiled into it, remains under its own license: see THIRD-PARTY-NOTICES.md.\n"
            ),
            String::new()
        )
    );
}

#[test]
fn version_and_license_must_be_the_only_argument() {
    for args in [
        &["--version", "x"][..],
        &["--license", "--version"],
        &[],
        &["check"],
        &["check", "a", "b"],
        &["use"],
        &["unknown"],
    ] {
        let (code, out, err) = tool(args);
        assert_eq!((code, out.as_str()), (2, ""), "{args:?}");
        assert!(
            err.starts_with("usage:\n  gel-license check DIR\n"),
            "{args:?}"
        );
    }
}

#[test]
fn check_of_an_exact_distribution_has_the_listed_items() {
    let t = Scratch::new();
    t.put("LICENSE", LICENSE);
    t.put("NOTICE", NOTICE);
    t.put(
        "my crate/Cargo.toml",
        b"[dependencies]\ngel-source.workspace = true\n",
    );
    assert_eq!(
        tool(&["check", t.dir()]),
        (
            0,
            format!("LICENSE_FILE=EXACT\nREQUIRED_NOTICE=KEPT\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=gel-source\nSCAN_FILES=3 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT\n{CHECK_TAIL}"),
            String::new()
        )
    );
}

#[test]
fn check_of_a_changed_license_misses_items_and_of_an_unrelated_tree_finds_none() {
    let t = Scratch::new();
    let mut changed = LICENSE.to_vec();
    changed[100] ^= 0x20;
    t.put("LICENSE", &changed);
    t.put(
        "src/lib.rs",
        "// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.\nfn f() {}\n"
            .as_bytes(),
    );
    assert_eq!(
        tool(&["check", t.dir()]),
        (
            3,
            format!("LICENSE_FILE=CHANGED\nREQUIRED_NOTICE=KEPT\nGEL_HEADER_FILES=1 HEADER_INTACT=0 HEADER_ALTERED=1\nGEL_CRATES=none\nSCAN_FILES=2 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=LISTED_ITEMS_MISSING\n{CHECK_TAIL}"),
            String::new()
        )
    );
    let other = Scratch::new();
    other.put("README.md", b"An unrelated project.\n");
    other.put("Cargo.toml", b"[dependencies]\ngel-tokio = \"0.10\"\n");
    assert_eq!(
        tool(&["check", other.dir()]),
        (
            4,
            format!("LICENSE_FILE=MISSING\nREQUIRED_NOTICE=MISSING\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=none\nSCAN_FILES=2 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=NO_GEL_NOTICES_FOUND\n{CHECK_TAIL}"),
            String::new()
        )
    );
}

#[test]
fn check_of_a_crlf_license_names_the_line_endings() {
    let t = Scratch::new();
    let mut crlf = Vec::new();
    for b in LICENSE {
        if *b == b'\n' {
            crlf.push(b'\r');
        }
        crlf.push(*b);
    }
    t.put("LICENSE", &crlf);
    let (code, out, _) = tool(&["check", t.dir()]);
    assert_eq!(code, 3);
    assert!(
        out.starts_with(
            "LICENSE_FILE=CHANGED\nLICENSE_DETAIL=line_endings\nREQUIRED_NOTICE=KEPT\n"
        ),
        "{out}"
    );
    assert!(out.ends_with(&format!(
        "DISTRIBUTION_NOTICES=LISTED_ITEMS_MISSING\n{CHECK_TAIL}"
    )));
}

#[test]
fn check_of_a_file_is_refused() {
    let t = Scratch::new();
    let file = t.put("LICENSE", LICENSE);
    assert_eq!(
        tool(&["check", &file]),
        (
            2,
            "CHECK=REFUSED reason=not a directory\n".into(),
            String::new()
        )
    );
}

fn answers(values: [&str; 7]) -> String {
    KEYS.iter()
        .zip(values)
        .map(|(k, v)| format!("{k}={v}\n"))
        .collect::<Vec<_>>()
        .concat()
}

const COMMERCIAL: &str = "USE_VERDICT=COMMERCIAL_AGREEMENT_REQUIRED\nLICENSE_SECTIONS=2,5\nLICENSE_QUOTE=No Monetized Use is licensed by these public terms. Before beginning a Monetized Use, You must obtain a Commercial Agreement from the Licensor.\n";

#[test]
fn use_prints_each_verdict_exactly() {
    let t = Scratch::new();
    let conflict = "NOTE=evaluation_only=yes and an item of Monetized Use answered yes: a Monetized Use is not Evaluation; section 2 defines Monetized Use as use \"other than permitted Evaluation\"\n";
    let cases = [
        (
            ["no", "no", "no", "no", "no", "no", "yes"],
            "USE_VERDICT=NONCOMMERCIAL\nLICENSE_SECTIONS=2,3\nLICENSE_QUOTE=Subject to this license, the Licensor grants You a worldwide, non-exclusive, non-transferable, non-sublicensable, royalty-free copyright license to reproduce, run, study, modify, and share the Software for Noncommercial Purposes. … You may share the Software or a Modification with other people for Noncommercial Purposes, including private sharing, research collaboration, teaching, hobby work, community experimentation, and noncommercial redistribution of demonstration media, provided You preserve this license and the notices required by section 8.\n".to_string(),
        ),
        (
            ["no", "no", "no", "no", "no", "yes", "yes"],
            "USE_VERDICT=EVALUATION\nLICENSE_SECTIONS=2,4\nLICENSE_QUOTE=Evaluation may use ordinary rented or owned computing infrastructure, but the Software or a Modification may not be deployed as part of a paid plan, customer-facing production environment, revenue-generating service, or other Monetized Use under this section. Evaluation does not grant commercial rights. If Evaluation becomes a Monetized Use, commercial rights are required before that Monetized Use begins.\n".to_string(),
        ),
        (
            ["no", "no", "no", "no", "yes", "no", "yes"],
            COMMERCIAL.to_string(),
        ),
        // Not a Noncommercial Purpose, all five items no.
        (
            ["no", "no", "no", "no", "no", "no", "no"],
            COMMERCIAL.to_string(),
        ),
        // A Monetized item and evaluation_only both yes: one extra note.
        (
            ["no", "yes", "no", "no", "no", "yes", "yes"],
            format!("{COMMERCIAL}{conflict}"),
        ),
        (
            ["no", "no", "unsure", "no", "no", "no", "yes"],
            "USE_VERDICT=ASK_THE_LICENSOR\nLICENSE_SECTIONS=2,19\nLICENSE_QUOTE=Commercial licensing and licensing questions: gelram.licensing@gmail.com\n".to_string(),
        ),
    ];
    for (i, (values, head)) in cases.into_iter().enumerate() {
        let file = t.put(&format!("answers {i}"), answers(values).as_bytes());
        assert_eq!(
            tool(&["use", &file]),
            (
                0,
                format!(
                    "{head}LEGAL_CERTIFICATION=NO\nCONTACT=gelram.licensing@gmail.com\n{NOTE}\n"
                ),
                String::new()
            ),
            "{values:?}"
        );
    }
}

#[test]
fn use_refuses_incomplete_or_invalid_answers() {
    let t = Scratch::new();
    let all_no = answers(["no"; 7]);
    for (text, reason) in [
        (all_no.replace("hosted=no\n", ""), "missing keys: hosted"),
        (format!("{all_no}money=no\n"), "line 8: duplicate key money"),
        (format!("{all_no}grant=yes\n"), "line 8: unknown key grant"),
        (
            all_no.replacen("money=no", "money=perhaps", 1),
            "line 1: money must be yes, no or unsure",
        ),
        // Escaped once: a control character and a second byte order mark.
        (
            format!("{all_no}gr\u{7}ant=yes\n"),
            "line 8: unknown key gr\\u{7}ant",
        ),
        (
            format!("\u{feff}\u{feff}{all_no}"),
            "line 1: unknown key \\u{feff}money",
        ),
    ] {
        let file = t.put("answers", text.as_bytes());
        assert_eq!(
            tool(&["use", &file]),
            (
                2,
                format!("USE_VERDICT=REFUSED reason={reason}\n"),
                String::new()
            )
        );
    }
    let (code, out, _) = tool(&["use", &t.0.join("absent").to_string_lossy()]);
    assert_eq!(code, 2);
    assert!(out.starts_with("USE_VERDICT=REFUSED reason=cannot read the answers file"));
    // One leading byte order mark is accepted.
    let file = t.put(
        "bom",
        format!("\u{feff}{}", answers(["unsure"; 7])).as_bytes(),
    );
    let (code, out, _) = tool(&["use", &file]);
    assert_eq!(code, 0);
    assert!(out.starts_with("USE_VERDICT=ASK_THE_LICENSOR\n"), "{out}");
}

#[test]
fn use_template_is_printed_and_refused_until_answered() {
    let (code, template, err) = tool(&["use", "--template"]);
    assert_eq!((code, err.as_str()), (0, ""));
    assert!(
        template.starts_with("# Your own answers about one use of GEL RAM (GEL-RAM-NCRL-1.0).\n")
    );
    assert!(template.contains("\n# money: Is this \"use of the Software or a Modification,"));
    assert!(template.contains("\n# noncommercial_purpose: Is the use a \"use that does not"));
    assert!(template.ends_with("\nnoncommercial_purpose=\n"));
    let t = Scratch::new();
    let file = t.put("template", template.as_bytes());
    let (code, out, _) = tool(&["use", &file]);
    assert_eq!(code, 2);
    assert!(
        out.ends_with(": money must be yes, no or unsure\n"),
        "{out}"
    );
}
