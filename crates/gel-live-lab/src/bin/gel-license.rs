// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! gel-license: checks a distribution tree, and the user's own answers about
//! one use, against GEL RAM Noncommercial Reciprocal License 1.0
//! (docs/LICENSE-DECISION.md). Offline and read-only; the rules are in
//! `gel_live_lab::license_check`. Arguments are separate words, so paths may
//! contain spaces. Reports go to stdout; usage goes to stderr. A refusal reason
//! is escaped once, here.
#![forbid(unsafe_code)]
use gel_live_lab::{license_check as rules, safe};
use std::{ffi::OsString, io::Write, path::Path, process::ExitCode};

const TOOL: &str = "gel-license";
const USAGE: &str = "usage:
  gel-license check DIR
  gel-license use ANSWERS_FILE
  gel-license use --template
  gel-license --version
  gel-license --license
exit: check 0 LISTED_ITEMS_PRESENT, 3 LISTED_ITEMS_MISSING, 4 NO_GEL_NOTICES_FOUND;
      use 0 verdict printed; 2 refused or invalid arguments";

/// Text for stdout and the exit code, or `None` for a usage error.
fn run(args: &[OsString]) -> Option<(String, u8)> {
    let word = |i: usize| args.get(i).and_then(|a| a.to_str());
    let version = env!("CARGO_PKG_VERSION");
    Some(match (word(0)?, args.len()) {
        ("--version", 1) => (format!("{}\n", gel_core::tool_identity(TOOL, version)), 0),
        ("--license", 1) => (gel_core::tool_license(TOOL, version), 0),
        ("--help", 1) => (format!("{USAGE}\n"), 0),
        ("check", 2) => match rules::check_dir(Path::new(&args[1])) {
            Ok(report) => (report.lines(), report.distribution().exit_code()),
            Err(why) => (format!("CHECK=REFUSED reason={}\n", safe(&why)), 2),
        },
        ("use", 2) if word(1) == Some("--template") => (rules::template(), 0),
        ("use", 2) => match rules::read_answers(Path::new(&args[1])) {
            Ok(answers) => (rules::use_report(&answers), 0),
            Err(why) => (format!("USE_VERDICT=REFUSED reason={}\n", safe(&why)), 2),
        },
        _ => return None,
    })
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match run(&args) {
        Some((text, code)) => {
            let mut out = std::io::stdout().lock();
            if out
                .write_all(text.as_bytes())
                .and_then(|_| out.flush())
                .is_err()
            {
                eprintln!("stdout write failed");
                return ExitCode::from(2);
            }
            ExitCode::from(code)
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
