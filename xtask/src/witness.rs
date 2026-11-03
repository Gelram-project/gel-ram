// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! `witness NEW_DIR [FILE]`: one command that runs the short checks of
//! docs/REPRODUCE.md and prints one block to paste into a report.
//!
//! Each step runs as its own process, its whole output goes to
//! `NEW_DIR/<step>.log`, and its result is read from the marker line the program
//! itself prints (never from a test harness): a step passes only when the
//! process succeeded and its last marker line starts with the expected value.
//! Every step runs even when an earlier one failed. The block names the tool
//! version, the revision, the host (system, CPU, cores, memory, rustc) and each
//! step's result, exit status, wall time and marker, ends with its own SHA-256,
//! and is written to NEW_DIR/WITNESS.txt. Before it is written the step lines
//! pass the privacy filter of `host_report`, and a block in which a second
//! check still finds this host's name, the user's name or a directory of the
//! run is refused (docs/CHECK-ON-YOUR-HARDWARE.md).
use gel_source::{digest, hex};
use std::{
    fmt::Write as _,
    fs,
    path::Path,
    process::{Command, ExitStatus},
    time::Instant,
};

/// The exit status as a number, or the signal that ended the process.
fn exit_text(status: &ExitStatus) -> String {
    if let Some(code) = status.code() {
        return code.to_string();
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal-{signal}");
        }
    }
    "unknown".into()
}

/// One step: name, the marker prefix it prints and the value that passes.
struct Step {
    name: &'static str,
    marker: &'static str,
    pass: &'static str,
    command: Vec<String>,
    /// Why the step cannot run on this host, if it cannot.
    skip: Option<&'static str>,
}

/// The result of one step from its exit status and its output: PASS, FAIL or
/// FAIL with "no marker"; a missing marker is never a pass.
fn classify(success: bool, output: &str, marker: &str, pass: &str) -> (&'static str, String) {
    let prefix = format!("{marker}=");
    match output.lines().rev().find(|l| l.starts_with(&prefix)) {
        Some(line) if success && line.starts_with(pass) => ("PASS", line.to_string()),
        Some(line) => ("FAIL", line.to_string()),
        None => ("FAIL", format!("{marker}=NO_MARKER exit_success={success}")),
    }
}

fn steps(file: &str) -> Vec<Step> {
    let cargo = |extra: &[&str]| -> Vec<String> {
        ["cargo", "run", "--locked", "--offline", "--release", "-q"]
            .iter()
            .chain(extra)
            .map(|s| s.to_string())
            .collect()
    };
    let me = |extra: &[&str]| -> Vec<String> {
        std::iter::once("SELF")
            .chain(extra.iter().copied())
            .map(str::to_string)
            .collect()
    };
    let unix_only = if cfg!(unix) {
        None
    } else {
        Some("the kill series needs a Unix host")
    };
    vec![
        Step {
            name: "literal",
            marker: "GEL_LIVE_LAB_LITERAL",
            pass: "GEL_LIVE_LAB_LITERAL=PASS",
            command: cargo(&["-p", "gel-live-lab", "--", "--literal", file]),
            skip: None,
        },
        Step {
            name: "quad",
            marker: "Q8_QUAD_EXACT",
            pass: "Q8_QUAD_EXACT=PASS",
            command: cargo(&["-p", "gel-phase-quad", "--example", "quad_compare"]),
            skip: None,
        },
        Step {
            name: "crash-series",
            marker: "CRASH_SERIES",
            pass: "CRASH_SERIES=PASS",
            command: me(&["crash-series", "20"]),
            skip: unix_only,
        },
        Step {
            name: "history-crash-series",
            marker: "HISTORY_CRASH_SERIES",
            pass: "HISTORY_CRASH_SERIES=PASS",
            command: me(&["history-crash-series", "5", "20261010", "5"]),
            skip: unix_only,
        },
        Step {
            name: "history-mutation-matrix",
            marker: "HISTORY_MUTATION_MATRIX",
            pass: "HISTORY_MUTATION_MATRIX=PASS",
            command: me(&["history-mutation-matrix"]),
            skip: None,
        },
        Step {
            name: "same-bits",
            marker: "WITNESS",
            pass: "WITNESS=MATCH",
            command: cargo(&["-p", "gel-live-lab", "--", "--witness"]),
            skip: None,
        },
    ]
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage =
        "usage: cargo run --locked --offline -p xtask -- witness NEW_DIR_OUTSIDE_CHECKOUT [FILE]";
    let (dir, file) = match args {
        [dir] => (dir, None),
        [dir, file] => (dir, Some(file)),
        _ => return Err(usage.into()),
    };
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let out = Path::new(dir);
    if out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("output directory must be outside the source checkout".into());
    }
    fs::create_dir(out).map_err(|e| format!("new output directory required: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;
    let file = match file {
        Some(f) => f.clone(),
        None => root.join("README.md").to_string_lossy().into_owned(),
    };
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let rustc = Command::new("rustc")
        .arg("-V")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "NOT_MEASURED".into());

    // The fact lines are made here; the step lines carry what the steps printed.
    let mut facts = String::from("GEL_WITNESS_2\n");
    let _ = writeln!(
        facts,
        "tool=xtask-witness version={}",
        env!("CARGO_PKG_VERSION")
    );
    let _ = writeln!(facts, "{}", super::bench_compare::revision(&root));
    facts.push_str(&super::host_report::facts());
    let _ = writeln!(facts, "rustc={rustc}");
    let mut block = String::new();
    let (mut pass, mut fail, mut skipped) = (0, 0, 0);
    for step in steps(&file) {
        if let Some(why) = step.skip {
            skipped += 1;
            let _ = writeln!(block, "step={} result=SKIPPED reason={why}", step.name);
            continue;
        }
        let (program, rest) = step.command.split_first().ok_or("empty command")?;
        let mut command = if program == "SELF" {
            Command::new(&me)
        } else {
            Command::new(program)
        };
        let started = Instant::now();
        let (success, exit, output) = match command.args(rest).current_dir(&root).output() {
            Ok(o) => {
                let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&o.stderr));
                (o.status.success(), exit_text(&o.status), text)
            }
            Err(e) => (
                false,
                "not-started".into(),
                format!("could not start {program}: {e}\n"),
            ),
        };
        let seconds = started.elapsed().as_secs_f64();
        fs::write(out.join(format!("{}.log", step.name)), &output).map_err(|e| e.to_string())?;
        let (result, marker) = classify(success, &output, step.marker, step.pass);
        if result == "PASS" {
            pass += 1;
        } else {
            fail += 1;
        }
        let _ = writeln!(
            block,
            "step={} result={result} exit={exit} seconds={seconds:.2} marker={marker}",
            step.name
        );
    }
    // Nothing leaves this function before it has passed the filter.
    let private = super::host_report::Private::of_this_host(&[&root, &out]);
    let (mut block, redactions) = super::host_report::finish(&facts, &block, &private)
        .map_err(|e| format!("GEL_WITNESS={e}"))?;
    let _ = writeln!(block, "redactions={redactions}");
    let _ = writeln!(block, "block_sha256={}", hex(&digest(block.as_bytes())));
    let verdict = if fail == 0 { "PASS" } else { "FAIL" };
    let line = format!("GEL_WITNESS={verdict} pass={pass} fail={fail} skipped={skipped}");
    let _ = writeln!(block, "{line}");
    fs::write(out.join("WITNESS.txt"), &block).map_err(|e| e.to_string())?;
    print!("{block}");
    if fail > 0 {
        return Err(format!("{line}; the logs are in the output directory"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_passes_only_on_success_with_the_expected_marker() {
        let ok = "build\nWITNESS_TABLE=MATCH\nWITNESS=MATCH platform=linux-x86_64\n";
        assert_eq!(classify(true, ok, "WITNESS", "WITNESS=MATCH").0, "PASS");
        // A failed process, a different value or no marker at all is a failure.
        assert_eq!(classify(false, ok, "WITNESS", "WITNESS=MATCH").0, "FAIL");
        let other = "WITNESS=MISMATCH platform=windows-x86_64\n";
        assert_eq!(classify(true, other, "WITNESS", "WITNESS=MATCH").0, "FAIL");
        let none = classify(true, "compiled\n", "WITNESS", "WITNESS=MATCH");
        assert_eq!(
            none,
            ("FAIL", "WITNESS=NO_MARKER exit_success=true".to_string())
        );
        // A longer marker name is not the expected marker.
        let longer = "WITNESS_TABLE=MATCH\n";
        assert_eq!(classify(true, longer, "WITNESS", "WITNESS=MATCH").0, "FAIL");
        // The last marker line counts.
        let twice = "CRASH_SERIES=FAIL\nCRASH_SERIES=PASS trials=20\n";
        assert_eq!(
            classify(true, twice, "CRASH_SERIES", "CRASH_SERIES=PASS").0,
            "PASS"
        );
    }

    #[test]
    fn every_step_has_its_own_marker_and_the_kill_series_is_unix_only() {
        let all = steps("FILE");
        assert_eq!(all.len(), 6);
        for s in &all {
            assert!(s.pass.starts_with(&format!("{}=", s.marker)), "{}", s.name);
        }
        let unix = all.iter().filter(|s| s.name.contains("crash"));
        for s in unix {
            assert_eq!(s.skip.is_some(), !cfg!(unix), "{}", s.name);
        }
        assert!(all[0].command.contains(&"FILE".to_string()));
    }
}
