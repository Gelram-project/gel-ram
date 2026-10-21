//! Wall time of the whole-tree `xtask` gates, each run as one child process:
//! `rust-only`, `licensing`, `docs-refs`, `disclosure`, the `ci-policy` gate
//! as a control (its code reads four workflow files and does not walk the
//! tree) and, on request, `recorder-lint`. The example only starts an `xtask`
//! binary, so the same file times an older tree as well.
//!
//! Build (release shown; the dev profile, which `cargo xtask` uses, works the
//! same way without `--release`):
//!   cargo build --release -p xtask
//!   cargo build --release -p gel-source --example speed_tree_gates
//! Usage: speed_tree_gates [RUNS] [BIN_DIR] [RECORDER]
//! RUNS is 1..=999 (default 9). BIN_DIR holds the `xtask` binary (default:
//! the directory above this example's own directory). RECORDER 1 adds the
//! recorder-lint case (default 0); that gate writes probe copies under the
//! build directory, or under `target/` of the tree when an older `xtask` runs.
//! Input: the checkout the `xtask` binary was built from; the gates read the
//! source tree recorded at build time.
//!
//! Prints `SPEED_SETUP`, one `SPEED name=<case> run=<i> ns=<n>` per case and
//! run, and `SPEED_CHECK sha256=<hex>` over the case names, exit codes and
//! standard output of run 0 (without the disclosure gate's text file count,
//! which differs between trees by the files of a change). A later run with
//! other output, or a gate that fails, ends in `SPEED_FAIL` on standard error
//! and exit 1. Output grammar: `docs/SPEED.md`.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

/// (case name, xtask argument)
const CASES: &[(&str, &str)] = &[
    ("rust_only", "rust-only"),
    ("licensing", "licensing"),
    ("docs_refs", "docs-refs"),
    ("disclosure", "disclosure"),
    ("ci_policy", "ci-policy"),
];
const RECORDER_CASE: (&str, &str) = ("recorder_lint", "recorder-lint");

/// Standard output with the disclosure gate's file count left out: it counts
/// the text files of the tree, and two trees differ by the files of a change.
fn comparable(stdout: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(stdout);
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        match line.split_once(" text_files=") {
            Some((head, _)) if head.starts_with("PUBLIC_DISCLOSURE_GATE=") => {
                out.push_str(head);
                out.push_str(" text_files=*\n");
            }
            _ => out.push_str(line),
        }
    }
    out.into_bytes()
}

fn fail(reason: &str, detail: &str) -> ! {
    eprintln!("{detail}");
    speed::fail(reason)
}

fn main() -> ExitCode {
    let args = speed::args("speed_tree_gates", "[RUNS] [BIN_DIR] [RECORDER]", 3);
    let runs = speed::runs(&args);
    let bin_dir = match args.get(1) {
        Some(dir) => PathBuf::from(dir),
        None => match std::env::current_exe() {
            Ok(exe) => match exe.parent().and_then(|dir| dir.parent()) {
                Some(dir) => dir.to_path_buf(),
                None => fail("no_bin_dir", "cannot locate the build directory"),
            },
            Err(e) => fail("no_bin_dir", &format!("cannot locate this example: {e}")),
        },
    };
    let recorder = match args.get(2).map(String::as_str) {
        None | Some("0") => false,
        Some("1") => true,
        Some(_) => {
            eprintln!("RECORDER must be 0 or 1");
            eprintln!("usage: speed_tree_gates [RUNS] [BIN_DIR] [RECORDER]");
            return ExitCode::from(2);
        }
    };
    speed::self_check();
    let xtask = bin_dir.join(format!("xtask{}", std::env::consts::EXE_SUFFIX));
    if !xtask.is_file() {
        fail("missing_xtask", &format!("missing {}", xtask.display()));
    }
    let mut cases = CASES.to_vec();
    if recorder {
        cases.push(RECORDER_CASE);
    }
    let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
    speed::setup(
        "speed_tree_gates",
        runs,
        &[
            ("cases", cases.len().to_string()),
            ("recorder", u8::from(recorder).to_string()),
            ("cpus", cpus.to_string()),
        ],
    );

    let mut first: Vec<Vec<u8>> = Vec::new();
    for run in 0..runs {
        for (index, (name, argument)) in cases.iter().enumerate() {
            let start = Instant::now();
            let output = Command::new(&xtask).arg(argument).output();
            let ns = start.elapsed().as_nanos();
            let output = match output {
                Ok(output) => output,
                Err(e) => fail("spawn", &format!("{name}: {e}")),
            };
            if !output.status.success() {
                fail(
                    "gate_failed",
                    &format!(
                        "{name}: {}\n{}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                );
            }
            speed::line(name, run, ns);
            let mut record = Vec::new();
            record.extend_from_slice(name.as_bytes());
            record.push(0);
            record.extend_from_slice(format!("{:?}", output.status.code()).as_bytes());
            record.push(0);
            record.extend_from_slice(&comparable(&output.stdout));
            if run == 0 {
                first.push(record);
            } else if first[index] != record {
                fail(
                    "output_changed",
                    &format!("{name}: run {run} differs from run 0"),
                );
            }
        }
    }
    let mut check = speed::Sha256::new();
    for record in &first {
        check.update(&(record.len() as u64).to_le_bytes());
        check.update(record);
    }
    speed::check(check);
    ExitCode::SUCCESS
}
