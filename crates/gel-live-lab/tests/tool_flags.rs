// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! `--version` and `--license` of gel-live-lab, gel-evidence and gel-backup:
//! the exact lines, exit 0, empty stderr; with another argument a flag keeps
//! its earlier meaning, a usage error.
use std::process::Command;

const TOOLS: [(&str, &str); 3] = [
    ("gel-live-lab", env!("CARGO_BIN_EXE_gel-live-lab")),
    ("gel-evidence", env!("CARGO_BIN_EXE_gel-evidence")),
    ("gel-backup", env!("CARGO_BIN_EXE_gel-backup")),
];

fn stdout_of(exe: &str, args: &[&str]) -> String {
    let out = Command::new(exe).args(args).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{exe} {args:?}");
    assert!(out.stderr.is_empty(), "{exe} {args:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn version_and_license_print_the_exact_lines() {
    for (tool, exe) in TOOLS {
        let identity = format!(
            "GEL_TOOL={tool} VERSION={} LICENSE=GEL-RAM-NCRL-1.0\n",
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(stdout_of(exe, &["--version"]), identity);
        assert_eq!(
            stdout_of(exe, &["--license"]),
            format!(
                "{identity}GEL RAM Noncommercial Reciprocal License 1.0\n\
                 Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)\n\
                 Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. \
                 Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com\n\
                 Third-party material in this program, including the Unicode data compiled into it, \
                 remains under its own license: see THIRD-PARTY-NOTICES.md.\n"
            ),
            "{tool}"
        );
    }
}

#[test]
fn a_flag_with_another_argument_is_a_usage_error() {
    for (tool, exe) in TOOLS {
        for args in [["--version", "--help"], ["--license", "--help"]] {
            let out = Command::new(exe).args(args).output().unwrap();
            let err = String::from_utf8(out.stderr).unwrap();
            let (code, needle) = match tool {
                "gel-live-lab" => (1, "GEL_LIVE_LAB=FAIL usage: gel-live-lab --help"),
                "gel-evidence" => (1, "GEL_EVIDENCE=FAIL gel-evidence --help"),
                _ => (2, "usage:\n  gel-backup create"),
            };
            assert_eq!(out.status.code(), Some(code), "{tool} {args:?} {err}");
            assert!(out.stdout.is_empty(), "{tool} {args:?}");
            assert!(err.starts_with(needle), "{tool} {args:?} {err}");
        }
    }
}
