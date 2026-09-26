//! Deliberately small public evidence projection: never copies raw process logs.
use std::{fs, io::Write, path::Path, process::Command};

fn safe_test(line: &str) -> Option<&str> {
    let body = line.strip_prefix("test ")?;
    let (name, status) = body.split_once(" ... ")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_:".contains(&b))
        || !matches!(status, "ok" | "FAILED" | "ignored")
    {
        return None;
    }
    Some(line)
}

/// Totals from cargo's strict `test result:` summary lines of one command.
#[derive(Default, Debug, PartialEq, Eq)]
struct Totals {
    summaries: usize,
    passed: u64,
    failed: u64,
    ignored: u64,
    filtered: u64,
}

fn totals(text: &str) -> Result<Totals, String> {
    let mut t = Totals::default();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("test result: ") else {
            continue;
        };
        let rest = rest
            .strip_prefix("ok. ")
            .or_else(|| rest.strip_prefix("FAILED. "))
            .ok_or("unrecognised test summary")?;
        let fields: Vec<&str> = rest.split("; ").collect();
        let number = |index: usize, label: &str| -> Result<u64, String> {
            fields
                .get(index)
                .and_then(|f| f.strip_suffix(label))
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| format!("malformed test summary field {label}"))
        };
        t.passed += number(0, " passed")?;
        t.failed += number(1, " failed")?;
        t.ignored += number(2, " ignored")?;
        number(3, " measured")?;
        t.filtered += number(4, " filtered out")?;
        t.summaries += 1;
    }
    Ok(t)
}

/// Whole tests compiled only for some targets. Where excluded, the name must be
/// absent; elsewhere it must have run and passed exactly once. A compile-time
/// exclusion is never reported as a success on the excluding platform.
const EXCLUSIONS: &[(&str, &str)] = &[
    (
        "sigkill_after_save_ack_keeps_a_readable_snapshot",
        "SIGKILL of a child process",
    ),
    (
        "sigkill_at_observable_publication_never_installs_a_partial_snapshot",
        "SIGKILL of a child process",
    ),
    ("symlink_input_and_output_rejected", "Unix symlink creation"),
    (
        "tests::rust_only_gate_rejects_symlinks_and_executable_files",
        "Unix symlink and execute-bit creation",
    ),
    (
        "source_bundle::tests::symlink_file_and_directory_rejected",
        "Unix symlink creation",
    ),
    (
        "tests::atomic_write_is_private_by_default_and_preserves_existing_mode",
        "Unix permission bits",
    ),
];

/// Tests that run everywhere but contain an extra platform-only assertion block.
const PARTIAL_BRANCHES: &[(&str, &str, &str)] = &[
    (
        "persisted_roundtrip_and_no_temporary_files",
        "unix",
        "0600 file mode assertion",
    ),
    (
        "recorder_usage_and_closed_display_fail_closed",
        "linux",
        "/dev/full display assertion",
    ),
];

fn check_exclusions(tests: &str, unix: bool) -> Result<String, String> {
    let mut lines = String::new();
    for (name, reason) in EXCLUSIONS {
        let ran = tests
            .lines()
            .filter(|l| {
                l.strip_prefix("workspace-debug\ttest ")
                    .and_then(|rest| rest.strip_suffix(" ... ok"))
                    == Some(*name)
            })
            .count();
        let listed = tests
            .lines()
            .filter(|l| {
                l.strip_prefix("workspace-debug\ttest ")
                    .and_then(|rest| rest.split_once(" ... "))
                    .is_some_and(|(n, _)| n == *name)
            })
            .count();
        let here = match (unix, ran, listed) {
            (true, 1, 1) => "RAN_PASSED",
            (false, 0, 0) => "EXCLUDED_NOT_COUNTED",
            _ => {
                return Err(format!(
                "platform exclusion mismatch for {name}: unix={unix} passed={ran} listed={listed}"
            ))
            }
        };
        lines.push_str(&format!(
            "PLATFORM_EXCLUSION test={name} condition=unix here={here} reason={reason}\n"
        ));
    }
    for (name, condition, detail) in PARTIAL_BRANCHES {
        lines.push_str(&format!(
            "PARTIAL_PLATFORM_BRANCH test={name} condition={condition} detail={detail}\n"
        ));
    }
    Ok(lines)
}

fn accepted(id: &str, exited_successfully: bool, text: &str, count: usize) -> bool {
    exited_successfully
        && !text.lines().any(|line| {
            line.starts_with("test result: FAILED")
                || safe_test(line).is_some_and(|test| test.ends_with(" ... FAILED"))
        })
        && match id {
            "workspace-debug" | "core-release" => {
                count > 0
                    && text
                        .lines()
                        .filter_map(safe_test)
                        .any(|test| test.ends_with(" ... ok"))
                    && text
                        .lines()
                        .any(|line| line.starts_with("test result: ok."))
            }
            "saved-r1-release" => text.lines().any(|l| l == "COLLECTION_RECHECK=PASS"),
            "doctests-debug" => text.contains("test result: ok."),
            _ => false,
        }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|_| "cannot run git")?;
    if !out.status.success() {
        return Err("git identity failed".into());
    }
    String::from_utf8(out.stdout).map_err(|_| "invalid git identity".into())
}

fn identity(root: &Path) -> Result<String, String> {
    if !git(root, &["status", "--porcelain", "--untracked-files=all"])?
        .trim()
        .is_empty()
    {
        return Err("evidence requires a clean committed checkout".into());
    }
    let sha = git(root, &["rev-parse", "HEAD"])?;
    let sha = sha.trim();
    if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid commit SHA".into());
    }
    Ok(sha.into())
}

fn numeric_env(key: &str) -> Result<String, String> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) => Ok(v),
        Err(std::env::VarError::NotPresent) => Ok("LOCAL".into()),
        _ => Err(format!("invalid {key}")),
    }
}

fn write_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(name))
        .map_err(|_| "cannot create new evidence file")?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|_| "cannot persist evidence".into())
}

pub fn report(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("ci-evidence requires NEW_DIRECTORY_OUTSIDE_CHECKOUT".into());
    }
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|_| "missing checkout")?;
    let sha = identity(&root)?;
    let manifest =
        fs::read(root.join("SOURCE-SHA256SUMS.txt")).map_err(|_| "missing source manifest")?;
    let manifest_pin = gel_source::hex(&gel_source::digest(&manifest));
    super::source_bundle::validated_identity(&root, &manifest_pin)?;
    let run_id = numeric_env("GITHUB_RUN_ID")?;
    let attempt = numeric_env("GITHUB_RUN_ATTEMPT")?;
    let output = Path::new(&args[0]);
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if parent
        .canonicalize()
        .map_err(|_| "missing output parent")?
        .starts_with(&root)
    {
        return Err("evidence output must be outside checkout".into());
    }
    fs::create_dir(output).map_err(|_| "new output directory required")?;
    let mut report = format!("FORMAT=GEL_CI_EVIDENCE_2\nCOMMIT={sha}\nSOURCE_MANIFEST_SHA256={manifest_pin}\nRUN_ID={run_id}\nRUN_ATTEMPT={attempt}\nOS={}\nARCH={}\nSCOPE=workspace tests (debug), core-path tests (release), doctests, saved R1 recheck; not full CI or a fresh benchmark\n", std::env::consts::OS, std::env::consts::ARCH);
    let rust = Command::new("rustc")
        .arg("-V")
        .output()
        .map_err(|_| "rustc unavailable")?;
    // Toolchain pin comes from a strict comparison, never an arbitrary environment dump.
    if !rust.status.success() || !String::from_utf8_lossy(&rust.stdout).starts_with("rustc 1.85.0 ")
    {
        return Err("expected pinned Rust 1.85.0".into());
    }
    report.push_str("RUST_VERSION=1.85.0\nREDACTION=only strict test identifiers and derived counts; raw stdout/stderr excluded\n");
    let cases: &[(&str, &[&str])] = &[
        (
            "workspace-debug",
            &[
                "test",
                "--locked",
                "--offline",
                "--workspace",
                "--all-targets",
            ],
        ),
        (
            "core-release",
            &[
                "test",
                "--locked",
                "--offline",
                "--release",
                "-p",
                "gel-source",
                "-p",
                "gel-store",
                "-p",
                "gel-live-lab",
                "--all-targets",
            ],
        ),
        (
            "doctests-debug",
            &["test", "--locked", "--offline", "--workspace", "--doc"],
        ),
        (
            "saved-r1-release",
            &[
                "run",
                "--locked",
                "--offline",
                "--release",
                "-p",
                "gel-source",
                "--example",
                "collection_recheck",
                "--",
                "docs/evidence-collection/r1",
            ],
        ),
    ];
    let mut tests = String::new();
    let mut success = true;
    for (id, command) in cases {
        let out = Command::new("cargo")
            .args(*command)
            .current_dir(&root)
            // A Windows test build must not replace the running collector.
            .env("CARGO_TARGET_DIR", root.join("target/ci-evidence-tests"))
            .output()
            .map_err(|_| "cannot execute evidence command")?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut count = 0;
        for line in text.lines().filter_map(safe_test) {
            tests.push_str(&format!("{id}\t{line}\n"));
            count += 1;
        }
        // Malformed summaries fail closed; test commands need at least one summary.
        let summary = totals(&text);
        let summarised = match &summary {
            Ok(t) => *id == "saved-r1-release" || (t.summaries > 0 && t.failed == 0),
            Err(_) => false,
        };
        let accepted = accepted(id, out.status.success(), &text, count) && summarised;
        success &= accepted;
        let t = summary.unwrap_or_default();
        // Executions whose names are not projected (e.g. path-bearing doctests).
        let unlisted = (t.passed + t.failed + t.ignored).saturating_sub(count as u64);
        report.push_str(&format!("CASE={id} accepted={accepted} exit_code={:?} named_test_executions={count} summaries={} passed={} failed={} ignored={} filtered_out={} unlisted_executions={unlisted} stdout_bytes={} stderr_bytes={}\n", out.status.code(), t.summaries, t.passed, t.failed, t.ignored, t.filtered, out.stdout.len(), out.stderr.len()));
        println!("CI_EVIDENCE_CASE={id} accepted={accepted}");
    }
    match check_exclusions(&tests, cfg!(unix)) {
        Ok(lines) => {
            report.push_str(&lines);
            report.push_str(&format!(
                "PLATFORM_EXCLUSIONS=CHECKED whole_tests={} partial_branches={}\n",
                EXCLUSIONS.len(),
                PARTIAL_BRANCHES.len()
            ));
        }
        Err(e) => {
            success = false;
            report.push_str(&format!("PLATFORM_EXCLUSIONS=MISMATCH {e}\n"));
        }
    }
    if identity(&root)? != sha {
        return Err("checkout changed during evidence collection".into());
    }
    super::source_bundle::validated_identity(&root, &manifest_pin)?;
    write_new(output, "REPORT.txt", report.as_bytes())?;
    write_new(output, "TESTS.txt", tests.as_bytes())?;
    let hashes = format!(
        "{}  REPORT.txt\n{}  TESTS.txt\n",
        gel_source::hex(&gel_source::digest(report.as_bytes())),
        gel_source::hex(&gel_source::digest(tests.as_bytes()))
    );
    write_new(output, "SHA256SUMS.txt", hashes.as_bytes())?;
    println!("CI_EVIDENCE_REPORT_BEGIN\n{report}{tests}{hashes}CI_EVIDENCE_REPORT_END");
    if !success {
        return Err("evidence checks failed; no COMPLETE".into());
    }
    write_new(output, "COMPLETE", b"CI_EVIDENCE_SCOPE=PASS\n")?;
    println!("CI_EVIDENCE=PASS");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn printed_success_never_overrides_exit_failure_or_missing_evidence() {
        let marker = "COLLECTION_RECHECK=PASS\ntest result: ok.";
        for id in ["workspace-debug", "doctests-debug", "saved-r1-release"] {
            assert!(!accepted(id, false, marker, 10));
        }
        assert!(!accepted("workspace-debug", true, marker, 0));
        assert!(!accepted(
            "saved-r1-release",
            true,
            "prefix COLLECTION_RECHECK=PASS",
            0
        ));
        assert!(!accepted("unrecognised", true, marker, 10));
        // Neither an early nor a middle failure can be overwritten by the last PASS.
        for sequence in [[false, true, true], [true, false, true]] {
            let mut complete = true;
            for passed in sequence {
                complete &= passed;
            }
            assert!(!complete);
        }
    }
    #[test]
    fn zero_exit_cannot_hide_failed_or_only_ignored_workspace_tests() {
        let good = "test sample::ok ... ok\ntest result: ok. 1 passed; 0 failed;";
        assert!(accepted("workspace-debug", true, good, 1));
        assert!(!accepted("workspace-debug", false, good, 1));
        for text in [
            "test sample::skip ... ignored\ntest result: ok. 0 passed; 0 failed;",
            "test sample::bad ... FAILED\ntest result: ok. 1 passed; 0 failed;",
            "test sample::ok ... ok\ntest result: FAILED. 1 passed; 1 failed;",
            "test sample::ok ... ok",
        ] {
            assert!(!accepted("workspace-debug", true, text, 1), "{text}");
        }
    }

    #[test]
    fn summary_totals_are_summed_and_malformed_lines_rejected() {
        let text = "test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 2 filtered out; finished in 0.01s\n\
                    test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s\n";
        assert_eq!(
            totals(text).unwrap(),
            Totals {
                summaries: 2,
                passed: 7,
                failed: 1,
                ignored: 1,
                filtered: 2
            }
        );
        assert_eq!(totals("no summaries").unwrap(), Totals::default());
        for bad in [
            "test result: ok. x passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
            "test result: ok. 1 passed; 0 failed",
            "test result: maybe. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        ] {
            assert!(totals(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn platform_exclusions_are_checked_in_both_directions() {
        let mut ran = String::new();
        for (name, _) in EXCLUSIONS {
            ran.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let other = "workspace-debug\ttest unrelated::case ... ok\n";
        // Unix: every excluded-elsewhere test must have run and passed once.
        let lines = check_exclusions(&format!("{other}{ran}"), true).unwrap();
        assert_eq!(lines.matches("here=RAN_PASSED").count(), EXCLUSIONS.len());
        assert!(check_exclusions(other, true).is_err());
        assert!(check_exclusions(&format!("{ran}{ran}"), true).is_err());
        let ignored = ran.replacen(" ... ok", " ... ignored", 1);
        assert!(check_exclusions(&ignored, true).is_err());
        // Other targets: excluded tests are absent and never counted as success.
        let lines = check_exclusions(other, false).unwrap();
        assert_eq!(
            lines.matches("here=EXCLUDED_NOT_COUNTED").count(),
            EXCLUSIONS.len()
        );
        assert!(check_exclusions(&ran, false).is_err());
        // Another case id with the same name does not satisfy the workspace check.
        let release_only = ran.replace("workspace-debug\t", "core-release\t");
        assert!(check_exclusions(&release_only, true).is_err());
        assert_eq!(
            lines.matches("PARTIAL_PLATFORM_BRANCH").count(),
            PARTIAL_BRANCHES.len()
        );
    }

    #[test]
    fn projection_accepts_only_test_identifiers_and_known_statuses() {
        assert!(safe_test("test module::check_1 ... ok").is_some());
        assert!(safe_test("test reject ... FAILED").is_some());
        for s in [
            "test /home/private ... ok",
            "test secret@example ... ok",
            "test x ... ignored, private reason",
            "test x ... ok\u{1b}",
            "secret=value",
            "test  ... ok",
        ] {
            assert!(safe_test(s).is_none(), "{s:?}");
        }
    }
}
