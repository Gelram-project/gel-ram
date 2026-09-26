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

fn add(sum: &mut u64, value: u64) -> Result<(), String> {
    *sum = sum.checked_add(value).ok_or("test summary overflow")?;
    Ok(())
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
        // Plain unsigned decimal digits only; no sign, no separators.
        let number = |index: usize, label: &str| -> Result<u64, String> {
            fields
                .get(index)
                .and_then(|f| f.strip_suffix(label))
                .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| format!("malformed test summary field {label}"))
        };
        add(&mut t.passed, number(0, " passed")?)?;
        add(&mut t.failed, number(1, " failed")?)?;
        add(&mut t.ignored, number(2, " ignored")?)?;
        number(3, " measured")?;
        add(&mut t.filtered, number(4, " filtered out")?)?;
        t.summaries = t.summaries.checked_add(1).ok_or("test summary overflow")?;
    }
    Ok(t)
}

/// Whole tests compiled only for Unix targets. Where excluded, the name must be
/// absent; elsewhere it must have run and passed exactly once. A compile-time
/// exclusion is never reported as a success on the excluding platform. A new
/// Unix-only test must be added here; `xtask platform-diff` compares a Unix and
/// a Windows TESTS.txt and fails on any undeclared difference.
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
    (
        "kernel_permission_denial_keeps_previous_snapshot_and_leaves_no_file",
        "Unix permission bits",
    ),
];

/// Tests that run everywhere but contain an extra platform-only assertion block.
/// Each must run and pass exactly once on every platform.
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

/// (passed, listed) executions of `name` in the debug workspace pass.
fn workspace_counts(tests: &str, name: &str) -> (usize, usize) {
    let mut passed = 0;
    let mut listed = 0;
    for rest in tests
        .lines()
        .filter_map(|l| l.strip_prefix("workspace-debug\ttest "))
    {
        if let Some((n, status)) = rest.split_once(" ... ") {
            if n == name {
                listed += 1;
                passed += usize::from(status == "ok");
            }
        }
    }
    (passed, listed)
}

fn check_exclusions(tests: &str, unix: bool) -> Result<String, String> {
    let mut lines = String::new();
    for (name, reason) in EXCLUSIONS {
        let (ran, listed) = workspace_counts(tests, name);
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
        if workspace_counts(tests, name) != (1, 1) {
            return Err(format!(
                "partial-branch test {name} did not run and pass once"
            ));
        }
        lines.push_str(&format!(
            "PARTIAL_PLATFORM_BRANCH test={name} condition={condition} here=RAN_PASSED detail={detail}\n"
        ));
    }
    Ok(lines)
}

/// Per-name counts of passing debug-workspace executions; any other status fails.
fn passing_names(tests: &str) -> Result<std::collections::BTreeMap<&str, usize>, String> {
    let mut names = std::collections::BTreeMap::new();
    for rest in tests
        .lines()
        .filter_map(|l| l.strip_prefix("workspace-debug\ttest "))
    {
        let (name, status) = rest.split_once(" ... ").ok_or("malformed TESTS.txt line")?;
        if status != "ok" {
            return Err(format!("non-passing test in report: {name} {status}"));
        }
        let count = names.entry(name).or_insert(0usize);
        *count = count.checked_add(1).ok_or("test count overflow")?;
    }
    Ok(names)
}

/// Every name that passes on Unix but not on Windows must be a declared
/// exclusion, and nothing may pass on Windows only.
fn compare_platforms(unix: &str, windows: &str) -> Result<String, String> {
    let (u, w) = (passing_names(unix)?, passing_names(windows)?);
    let declared: std::collections::BTreeSet<&str> =
        EXCLUSIONS.iter().map(|(name, _)| *name).collect();
    let mut problems = Vec::new();
    for (name, &count) in &u {
        let other = w.get(name).copied().unwrap_or(0);
        let expected = if declared.contains(name) { 0 } else { count };
        if other != expected {
            problems.push(format!("{name}: unix={count} windows={other}"));
        }
    }
    for (name, &count) in &w {
        if !u.contains_key(name) {
            problems.push(format!("{name}: unix=0 windows={count}"));
        }
    }
    for name in &declared {
        if !u.contains_key(name) {
            problems.push(format!("{name}: declared but absent on unix"));
        }
    }
    if problems.is_empty() {
        Ok(format!(
            "PLATFORM_DIFF=PASS unix_passed={} windows_passed={} declared_unix_only={}",
            u.values().sum::<usize>(),
            w.values().sum::<usize>(),
            declared.len()
        ))
    } else {
        Err(format!("PLATFORM_DIFF=FAIL {}", problems.join("; ")))
    }
}

pub fn platform_diff(args: &[String]) -> Result<(), String> {
    let [unix, windows] = args else {
        return Err("platform-diff requires UNIX_TESTS.txt WINDOWS_TESTS.txt".into());
    };
    let read = |path: &String| fs::read_to_string(path).map_err(|e| format!("{path}: {e}"));
    println!("{}", compare_platforms(&read(unix)?, &read(windows)?)?);
    Ok(())
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
        let mut other = String::from("workspace-debug\ttest unrelated::case ... ok\n");
        for (name, _, _) in PARTIAL_BRANCHES {
            other.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        // Unix: every excluded-elsewhere test must have run and passed once.
        let lines = check_exclusions(&format!("{other}{ran}"), true).unwrap();
        assert_eq!(
            lines.matches("here=RAN_PASSED").count(),
            EXCLUSIONS.len() + PARTIAL_BRANCHES.len()
        );
        assert!(check_exclusions(&other, true).is_err());
        assert!(check_exclusions(&format!("{other}{ran}{ran}"), true).is_err());
        let ignored = ran.replacen(" ... ok", " ... ignored", 1);
        assert!(check_exclusions(&format!("{other}{ignored}"), true).is_err());
        // Other targets: excluded tests are absent and never counted as success.
        let lines = check_exclusions(&other, false).unwrap();
        assert_eq!(
            lines.matches("here=EXCLUDED_NOT_COUNTED").count(),
            EXCLUSIONS.len()
        );
        assert!(check_exclusions(&format!("{other}{ran}"), false).is_err());
        // Another case id with the same name does not satisfy the workspace check.
        let release_only = ran.replace("workspace-debug\t", "core-release\t");
        assert!(check_exclusions(&format!("{other}{release_only}"), true).is_err());
        // Partial-branch tests are checked, not merely listed.
        assert!(check_exclusions(&ran, true).is_err());
        let failed_branch = other.replacen(" ... ok", " ... FAILED", 2);
        assert!(check_exclusions(&failed_branch, false).is_err());
    }

    #[test]
    fn platform_diff_rejects_undeclared_stale_and_windows_only_tests() {
        let mut shared = String::from("workspace-debug\ttest shared::case ... ok\n");
        for (name, _, _) in PARTIAL_BRANCHES {
            shared.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let mut declared = String::new();
        for (name, _) in EXCLUSIONS {
            declared.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let unix = format!("{shared}{declared}");
        assert!(compare_platforms(&unix, &shared)
            .unwrap()
            .starts_with("PLATFORM_DIFF=PASS"));
        // A Unix-only test that is not declared is caught.
        let undeclared = format!("{unix}workspace-debug\ttest new::unix_only ... ok\n");
        assert!(compare_platforms(&undeclared, &shared).is_err());
        // A declared test that no longer exists on Unix is caught.
        let first = EXCLUSIONS[0].0;
        let stale = unix.replace(&format!("test {first} ... ok\n"), "");
        assert!(compare_platforms(&stale, &shared).is_err());
        // A test passing only on Windows, or a declared test running there, is caught.
        let win_only = format!("{shared}workspace-debug\ttest win::only ... ok\n");
        assert!(compare_platforms(&unix, &win_only).is_err());
        assert!(compare_platforms(&unix, &unix).is_err());
        // A non-passing line in either report fails.
        let failed = unix.replacen(" ... ok", " ... FAILED", 1);
        assert!(compare_platforms(&failed, &shared).is_err());
    }

    #[test]
    fn summary_overflow_and_signed_counts_are_rejected() {
        let max = format!(
            "test result: ok. {} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",
            u64::MAX
        );
        assert!(totals(&max).is_ok());
        assert!(totals(&format!("{max}{max}")).is_err());
        assert!(totals(
            "test result: ok. +3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        )
        .is_err());
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
