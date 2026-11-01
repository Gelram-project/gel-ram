// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! `gel-cli --version` and `--license`: the exact lines, exit 0, empty stderr;
//! with any other argument the flags keep their earlier meaning.
use std::process::Command;

const EXE: &str = env!("CARGO_BIN_EXE_gel-cli");

fn stdout_of(args: &[&str]) -> String {
    let out = Command::new(EXE).args(args).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{args:?}");
    assert!(out.stderr.is_empty(), "{args:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn version_and_license_print_the_exact_lines() {
    let identity = format!(
        "GEL_TOOL=gel-cli VERSION={} LICENSE=GEL-RAM-NCRL-1.0\n",
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(stdout_of(&["--version"]), identity);
    assert_eq!(
        stdout_of(&["--license"]),
        format!(
            "{identity}GEL RAM Noncommercial Reciprocal License 1.0\n\
             Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)\n\
             Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. \
             Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com\n\
             Third-party material in this program, including the Unicode data compiled into it, \
             remains under its own license: see THIRD-PARTY-NOTICES.md.\n"
        )
    );
}

#[test]
fn a_flag_with_another_argument_is_an_unknown_command() {
    for args in [["--version", "verify"], ["--license", "selftest"]] {
        let out = Command::new(EXE).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        let err = String::from_utf8(out.stderr).unwrap();
        assert!(err.contains("unknown command"), "{err}");
    }
}
