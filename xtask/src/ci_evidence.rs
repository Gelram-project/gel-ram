// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
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

/// Whole tests compiled only for Unix or only for Linux, as (name, condition,
/// reason). Where excluded, the name must be absent; elsewhere it must have run
/// and passed exactly once. A compile-time exclusion is never reported as a
/// success on the excluding platform. A new platform-only test must be added
/// here: a unit test below reads the sources for one that is not, and
/// `xtask platform-diff` compares a Unix and a Windows TESTS.txt and fails on
/// any undeclared difference.
const EXCLUSIONS: &[(&str, &str, &str)] = &[
    (
        "sigkill_after_save_ack_keeps_a_readable_snapshot",
        "unix",
        "SIGKILL of a child process",
    ),
    (
        "sigkill_at_observable_publication_never_installs_a_partial_snapshot",
        "unix",
        "SIGKILL of a child process",
    ),
    (
        "symlink_input_and_output_rejected",
        "unix",
        "Unix symlink creation",
    ),
    (
        "tests::rust_only_gate_rejects_symlinks_and_executable_files",
        "unix",
        "Unix symlink and execute-bit creation",
    ),
    (
        "source_bundle::tests::symlink_file_and_directory_rejected",
        "unix",
        "Unix symlink creation",
    ),
    (
        "tests::atomic_write_is_private_by_default_and_preserves_existing_mode",
        "unix",
        "Unix permission bits",
    ),
    (
        "kernel_permission_denial_keeps_previous_snapshot_and_leaves_no_file",
        "unix",
        "Unix permission bits",
    ),
    (
        "commitment::tests::a_link_is_recorded_by_its_target_and_not_followed",
        "unix",
        "Unix symlink creation",
    ),
    (
        "license_check::tests::a_unicode_license_link_or_directory_is_not_counted",
        "unix",
        "Unix symlink creation",
    ),
    (
        "license_check::tests::symlinks_are_skipped_not_followed",
        "unix",
        "Unix symlink creation",
    ),
    (
        "source_headers::tests::a_linked_source_file_is_refused",
        "unix",
        "Unix symlink creation",
    ),
    (
        "speed_ab::tests::a_symbolic_link_on_the_way_is_refused_before_anything_is_written",
        "unix",
        "Unix symlink creation",
    ),
    (
        "license_check::tests::a_fifo_is_refused_without_being_opened",
        "unix",
        "Unix FIFO creation",
    ),
    (
        "license_check::tests::an_unreadable_root_is_named_dot_not_an_empty_path",
        "unix",
        "Unix permission bits",
    ),
    (
        "source_headers::tests::a_failed_write_changes_no_file_and_leaves_no_temporary_file",
        "unix",
        "Unix permission bits",
    ),
    (
        "bundle::tests::read_regular_reads_past_a_zero_length_hint",
        "linux",
        "Linux procfs file reporting length zero",
    ),
    (
        "literal::tests::a_file_reporting_length_zero_is_bounded_by_its_bytes",
        "linux",
        "Linux procfs file reporting length zero",
    ),
    (
        "literal::tests::a_file_reporting_length_zero_with_more_than_dim_bytes_reads_to_its_end",
        "linux",
        "Linux procfs file reporting length zero",
    ),
    (
        "a_killed_store_writer_keeps_acknowledged_saves_and_a_restart_resumes",
        "unix",
        "SIGKILL of a child process",
    ),
    (
        "two_processes_saving_one_history_leave_one_complete_file",
        "unix",
        "rename over a file another process has open",
    ),
    (
        "two_processes_saving_one_store_leave_one_complete_file",
        "unix",
        "rename over a file another process has open",
    ),
    (
        "reading_during_saves_sees_whole_histories_and_changes_no_byte",
        "unix",
        "rename over a file another process has open",
    ),
    (
        "the_control_shows_its_known_peak_and_a_changed_record_is_refused",
        "linux",
        "memory-run reads /proc",
    ),
];

/// Test-named entry points of a child process: `#[ignore]`d on every platform
/// and run only by another test. Nothing else may be listed as ignored.
const IGNORED_HELPERS: &[&str] = &[
    "torn_child",
    "history_fault_child",
    "store_fault_child",
    "source_fault_child",
    "memory::tests::memory_waiting_child",
];

/// Whether a test or branch of this condition is compiled on `os`.
fn compiled_on(condition: &str, os: &str) -> bool {
    match condition {
        "unix" => os != "windows",
        "linux" => os == "linux",
        _ => false,
    }
}

/// Tests that run everywhere but contain an extra platform-only assertion block.
/// Each must run and pass exactly once on every platform.
const PARTIAL_BRANCHES: &[(&str, &str, &str)] = &[
    (
        "reproduction::tests::report_inventory_rejects_links_and_preserves_complete_hashes",
        "unix",
        "report symlink rejection assertion",
    ),
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

fn check_exclusions(tests: &str, os: &str) -> Result<String, String> {
    let mut lines = String::new();
    for (name, condition, reason) in EXCLUSIONS {
        let (ran, listed) = workspace_counts(tests, name);
        let here = match (compiled_on(condition, os), ran, listed) {
            (true, 1, 1) => "RAN_PASSED",
            (false, 0, 0) => "EXCLUDED_NOT_COUNTED",
            _ => {
                return Err(format!(
                    "platform exclusion mismatch for {name}: os={os} passed={ran} listed={listed}"
                ))
            }
        };
        lines.push_str(&format!(
            "PLATFORM_EXCLUSION test={name} condition={condition} here={here} reason={reason}\n"
        ));
    }
    for (name, condition, detail) in PARTIAL_BRANCHES {
        if workspace_counts(tests, name) != (1, 1) {
            return Err(format!(
                "partial-branch test {name} did not run and pass once"
            ));
        }
        lines.push_str(&format!(
            "PARTIAL_PLATFORM_BRANCH test={name} condition={condition} here=RAN_PASSED branch={} detail={detail}\n",
            if compiled_on(condition, os) { "active" } else { "inactive" }
        ));
    }
    Ok(lines)
}

const CASE_IDS: &[&str] = &[
    "workspace-debug",
    "core-release",
    "doctests-debug",
    "saved-r1-release",
];

/// Per-name counts of passing debug-workspace executions. Every line must be a
/// projected test line of a known case; any other line or status fails.
fn passing_names(tests: &str) -> Result<std::collections::BTreeMap<&str, usize>, String> {
    let mut names = std::collections::BTreeMap::new();
    for line in tests.lines() {
        let (case, rest) = line
            .split_once('\t')
            .filter(|(case, _)| CASE_IDS.contains(case))
            .ok_or_else(|| format!("unrecognised TESTS.txt line: {line}"))?;
        let body = rest
            .strip_prefix("test ")
            .and_then(|r| r.split_once(" ... "))
            .filter(|(name, _)| safe_test(&format!("test {name} ... ok")).is_some())
            .ok_or_else(|| format!("malformed TESTS.txt line: {line}"))?;
        let (name, status) = body;
        if status == "ignored" && IGNORED_HELPERS.contains(&name) {
            continue;
        }
        if status != "ok" {
            return Err(format!("non-passing test in report: {name} {status}"));
        }
        if case == "workspace-debug" {
            let count = names.entry(name).or_insert(0usize);
            *count = count.checked_add(1).ok_or("test count overflow")?;
        }
    }
    Ok(names)
}

/// Listed executions of an ignored helper, in every case of one TESTS.txt.
fn ignored_helper_lines(tests: &str, helper: &str) -> usize {
    let line = format!("test {helper} ... ignored");
    tests
        .lines()
        .filter(|l| l.split_once('\t').is_some_and(|(_, rest)| rest == line))
        .count()
}

/// Two test lists with no report: the first is taken as Linux, so every
/// declared name must be on it.
fn compare_platforms(unix: &str, windows: &str) -> Result<String, String> {
    compare_lists(unix, windows, "linux", None)
}

/// Every name that passes on Unix but not on Windows must be a declared
/// exclusion that ran exactly once on Unix, and nothing may pass on Windows
/// only. A Linux-only name must be absent from a macOS list. With the first
/// platform's REPORT.txt, a name that report does not declare was declared
/// after that revision: it may be missing there, the rest of the rule stands.
fn compare_lists(
    unix: &str,
    windows: &str,
    unix_os: &str,
    unix_report: Option<&str>,
) -> Result<String, String> {
    let (u, w) = (passing_names(unix)?, passing_names(windows)?);
    let declared: std::collections::BTreeSet<&str> =
        EXCLUSIONS.iter().map(|(name, _, _)| *name).collect();
    let mut problems = Vec::new();
    for (name, &count) in &u {
        let other = w.get(name).copied().unwrap_or(0);
        let expected = if declared.contains(name) { 0 } else { count };
        if other != expected {
            problems.push(format!("{name}: unix={count} windows={other}"));
        }
        if declared.contains(name) && count != 1 {
            problems.push(format!(
                "{name}: declared exclusion ran {count} times on unix"
            ));
        }
    }
    for (name, &count) in &w {
        if !u.contains_key(name) {
            problems.push(format!("{name}: unix=0 windows={count}"));
        }
    }
    for (name, condition, _) in EXCLUSIONS {
        let listed = u.contains_key(name);
        let in_report = unix_report
            .is_none_or(|report| report.contains(&format!("PLATFORM_EXCLUSION test={name} ")));
        if !compiled_on(condition, unix_os) && listed {
            problems.push(format!("{name}: declared {condition} but ran on {unix_os}"));
        }
        if compiled_on(condition, unix_os) && in_report && !listed {
            problems.push(format!("{name}: declared but absent on unix"));
        }
    }
    for helper in IGNORED_HELPERS {
        let (here, there) = (
            ignored_helper_lines(unix, helper),
            ignored_helper_lines(windows, helper),
        );
        if here != there {
            problems.push(format!(
                "{helper}: ignored unix={here} windows={there} times"
            ));
        }
    }
    if problems.is_empty() {
        Ok(format!(
            "PLATFORM_DIFF=PASS unix_passed={} windows_passed={} declared_unix_only={}",
            u.values().sum::<usize>(),
            w.values().sum::<usize>(),
            declared.iter().filter(|n| u.contains_key(*n)).count()
        ))
    } else {
        Err(format!("PLATFORM_DIFF=FAIL {}", problems.join("; ")))
    }
}

/// `KEY=value` of a collector REPORT.txt; exactly one occurrence required.
fn report_field<'a>(report: &'a str, key: &str) -> Result<&'a str, String> {
    let mut values = report
        .lines()
        .filter_map(|l| l.strip_prefix(key).and_then(|r| r.strip_prefix('=')));
    match (values.next(), values.next()) {
        (Some(value), None) => Ok(value),
        _ => Err(format!("REPORT.txt needs exactly one {key}= line")),
    }
}

/// Both reports must describe the same commit, a Unix and a Windows host.
fn same_commit(unix: &str, windows: &str) -> Result<String, String> {
    let commit = report_field(unix, "COMMIT")?;
    if report_field(windows, "COMMIT")? != commit {
        return Err("PLATFORM_DIFF=FAIL reports are from different commits".into());
    }
    if !matches!(report_field(unix, "OS")?, "linux" | "macos") {
        return Err("PLATFORM_DIFF=FAIL first report is not from Linux or macOS".into());
    }
    if report_field(windows, "OS")? != "windows" {
        return Err("PLATFORM_DIFF=FAIL second report is not from Windows".into());
    }
    Ok(commit.to_owned())
}

pub fn platform_diff(args: &[String]) -> Result<(), String> {
    let read = |path: &String| fs::read_to_string(path).map_err(|e| format!("{path}: {e}"));
    match args {
        [unix, windows] => println!("{}", compare_platforms(&read(unix)?, &read(windows)?)?),
        [unix, windows, unix_report, windows_report] => {
            let unix_report = read(unix_report)?;
            let commit = same_commit(&unix_report, &read(windows_report)?)?;
            let os = report_field(&unix_report, "OS")?;
            println!(
                "{} commit={commit}",
                compare_lists(&read(unix)?, &read(windows)?, os, Some(&unix_report))?
            );
        }
        _ => {
            return Err(
                "platform-diff requires UNIX_TESTS WINDOWS_TESTS [UNIX_REPORT WINDOWS_REPORT]"
                    .into(),
            )
        }
    }
    Ok(())
}

const SCOPE: &str = "workspace tests (debug), core-path tests (release), doctests, saved R1 recheck; not full CI or a fresh benchmark";
const REDACTION: &str =
    "only strict test identifiers and derived counts; raw stdout/stderr excluded";

/// Revisions whose reports are kept in `docs/evidence-ci`, each with the push
/// run that printed them. Recording one more means adding it here.
const RECORDED: &[(&str, &str)] = &[
    ("2b29236", "36251557201"),
    ("fba6c43", "36257696669"),
    ("79f6ac5", "42077523488"),
    ("a9b5c6a", "42772483068"),
];
const PLATFORMS: [&str; 3] = ["linux", "macos", "windows"];
const RECORDED_KINDS: [&str; 3] = ["REPORT", "TESTS", "SHA256SUMS"];

fn sha256_hex(bytes: &[u8]) -> String {
    gel_source::hex(&gel_source::digest(bytes))
}

fn is_hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_number(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

/// REPORT, TESTS and checksum list as printed between the markers of one job
/// log, with the line prefix of the log removed and the printed checksums
/// proved against the rebuilt bytes.
fn rebuild_from_log(log: &str) -> Result<[String; 3], String> {
    let texts: Vec<&str> = log
        .lines()
        .map(|line| line.split_once(' ').map_or(line, |(_, text)| text))
        .collect();
    let only = |marker: &str| -> Result<usize, String> {
        let mut at = texts.iter().enumerate().filter(|(_, t)| **t == marker);
        match (at.next(), at.next()) {
            (Some((index, _)), None) => Ok(index),
            _ => Err(format!("log needs exactly one {marker} line")),
        }
    };
    let (begin, end) = (
        only("CI_EVIDENCE_REPORT_BEGIN")?,
        only("CI_EVIDENCE_REPORT_END")?,
    );
    let block = texts
        .get(begin + 1..end)
        .filter(|b| b.len() >= 2)
        .ok_or("log report block is out of order or too short")?;
    let (body, printed) = block.split_at(block.len() - 2);
    let first_test = body
        .iter()
        .position(|line| line.contains('\t'))
        .ok_or("log report block has no test lines")?;
    let (report, tests) = body.split_at(first_test);
    if tests.iter().any(|line| !line.contains('\t')) {
        return Err("log report block mixes report and test lines".into());
    }
    let file = |lines: &[&str]| lines.join("\n") + "\n";
    let (report, tests) = (file(report), file(tests));
    let expected = [
        format!("{}  REPORT.txt", sha256_hex(report.as_bytes())),
        format!("{}  TESTS.txt", sha256_hex(tests.as_bytes())),
    ];
    if printed != expected {
        return Err("rebuilt files do not match the checksums printed in the log".into());
    }
    let sums = file(printed);
    Ok([report, tests, sums])
}

/// `ci-evidence-import JOB_LOG PLATFORM REVISION_DIRECTORY`: writes the three
/// recorded files of one platform; existing files are never replaced.
pub fn import(args: &[String]) -> Result<(), String> {
    let [log, platform, directory] = args else {
        return Err("ci-evidence-import requires JOB_LOG PLATFORM REVISION_DIRECTORY".into());
    };
    if !PLATFORMS.contains(&platform.as_str()) {
        return Err(format!("unknown platform {platform}"));
    }
    let log = fs::read(log).map_err(|e| format!("{log}: {e}"))?;
    let [report, tests, sums] = rebuild_from_log(&String::from_utf8_lossy(&log))?;
    let sums = sums
        .lines()
        .map(|line| line.replacen("  ", &format!("  {platform}-"), 1) + "\n")
        .collect::<String>();
    let directory = Path::new(directory);
    fs::create_dir_all(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    for (kind, text) in RECORDED_KINDS.iter().zip([&report, &tests, &sums]) {
        write_new(
            directory,
            &format!("{platform}-{kind}.txt"),
            text.as_bytes(),
        )?;
    }
    println!(
        "CI_EVIDENCE_IMPORT=PASS platform={platform} report_lines={} test_lines={}",
        report.lines().count(),
        tests.lines().count()
    );
    Ok(())
}

/// A report line after the header: fixed line kinds, `key=value` fields of
/// plain tokens, and free text only as declared in this file, so no path, host
/// name or other runner detail has anywhere to be.
fn closed_report_line(line: &str) -> bool {
    let (tokens, text) = match line
        .split_once(" reason=")
        .or_else(|| line.split_once(" detail="))
    {
        Some((tokens, text)) => (tokens, Some(text)),
        None => (line, None),
    };
    let declared = |t: &str| {
        EXCLUSIONS.iter().any(|(_, _, reason)| *reason == t)
            || PARTIAL_BRANCHES.iter().any(|(_, _, detail)| *detail == t)
    };
    let plain = |value: &str| {
        !value.is_empty()
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_:()".contains(&b))
    };
    let mut fields = tokens.split(' ');
    let kind = fields.next().unwrap_or("");
    let known_kind = match kind.split_once('=') {
        Some(("CASE", id)) => CASE_IDS.contains(&id),
        Some(("PLATFORM_EXCLUSIONS" | "PROPERTY_MAP", value)) => value == "CHECKED",
        Some(_) => false,
        None => ["PLATFORM_EXCLUSION", "PARTIAL_PLATFORM_BRANCH", "PROPERTY"].contains(&kind),
    };
    known_kind
        && text.is_none_or(declared)
        && fields.all(|field| {
            field
                .split_once('=')
                .is_some_and(|(key, value)| plain(key) && plain(value))
        })
}

/// Value of `key=` among the space-separated fields of a report line.
fn line_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split(' ')
        .find_map(|field| field.strip_prefix(key)?.strip_prefix('='))
}

/// The header and every line of one recorded REPORT file; returns the passed
/// counts of the workspace, core and doctest cases.
fn check_recorded_report(
    report: &str,
    tests: &str,
    (revision, run): (&str, &str),
    platform: &str,
) -> Result<[u64; 3], String> {
    let field = |key: &str| report_field(report, key);
    let fail = |what: &str| Err(format!("{revision} {platform}: {what}"));
    let header = [
        "FORMAT",
        "COMMIT",
        "SOURCE_MANIFEST_SHA256",
        "RUN_ID",
        "RUN_ATTEMPT",
        "OS",
        "ARCH",
        "SCOPE",
        "RUST_VERSION",
        "REDACTION",
    ];
    let keys = report.lines().take(header.len());
    if !keys
        .map(|line| line.split('=').next().unwrap_or(""))
        .eq(header)
    {
        return fail("report header is not the collector's");
    }
    if field("FORMAT")? != "GEL_CI_EVIDENCE_2"
        || field("RUST_VERSION")? != "1.85.0"
        || field("SCOPE")? != SCOPE
        || field("REDACTION")? != REDACTION
        || !matches!(field("ARCH")?, "x86_64" | "aarch64")
        || !is_hex(field("SOURCE_MANIFEST_SHA256")?, 64)
        || !is_number(field("RUN_ATTEMPT")?)
    {
        return fail("a report header field is not the collector's");
    }
    let commit = field("COMMIT")?;
    if !is_hex(commit, 40) || !commit.starts_with(revision) {
        return fail("report commit is not this revision");
    }
    if field("RUN_ID")? != run {
        return fail("report run is not the recorded run");
    }
    if field("OS")? != platform {
        return fail("report is from another platform");
    }
    if let Some(line) = report
        .lines()
        .skip(header.len())
        .find(|line| !closed_report_line(line))
    {
        return fail(&format!("line outside the report format: {line}"));
    }
    // The property map entered the report after the first two records.
    for (line, lowest) in [
        ("PLATFORM_EXCLUSIONS=CHECKED ", 1),
        ("PROPERTY_MAP=CHECKED ", 0),
    ] {
        let count = report.lines().filter(|l| l.starts_with(line)).count();
        if count < lowest || count > 1 {
            return fail(&format!("needs one {line}line"));
        }
    }
    let mut passed = [0; 3];
    let mut slots = passed.iter_mut();
    for id in CASE_IDS {
        let prefix = format!("CASE={id} ");
        let mut lines = report.lines().filter(|l| l.starts_with(&prefix));
        let (Some(line), None) = (lines.next(), lines.next()) else {
            return fail(&format!("needs exactly one {prefix}line"));
        };
        if line_field(line, "accepted") != Some("true")
            || line_field(line, "exit_code") != Some("Some(0)")
            || line_field(line, "failed") != Some("0")
        {
            return fail(&format!("case {id} was not accepted"));
        }
        let listed = format!("{id}\t");
        let count = tests.lines().filter(|l| l.starts_with(&listed)).count();
        if line_field(line, "named_test_executions") != Some(count.to_string().as_str()) {
            return fail(&format!("case {id} does not count its listed tests"));
        }
        if *id != "saved-r1-release" {
            let number = line_field(line, "passed").and_then(|n| n.parse().ok());
            let Some(number) = number else {
                return fail(&format!("case {id} has no passed count"));
            };
            if let Some(slot) = slots.next() {
                *slot = number;
            }
        }
    }
    Ok(passed)
}

/// One recorded revision; returns its row of the table in the README.
fn check_recorded_revision(
    files: &std::collections::BTreeMap<String, String>,
    (revision, run): (&str, &str),
) -> Result<String, String> {
    let file = |platform: &str, kind: &str| {
        let key = format!("{revision}/{platform}-{kind}.txt");
        files
            .get(&key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing {key}"))
    };
    let mut passed = Vec::new();
    for platform in PLATFORMS {
        let (report, tests) = (file(platform, "REPORT")?, file(platform, "TESTS")?);
        let sums = format!(
            "{}  {platform}-REPORT.txt\n{}  {platform}-TESTS.txt\n",
            sha256_hex(report.as_bytes()),
            sha256_hex(tests.as_bytes())
        );
        if file(platform, "SHA256SUMS")? != sums {
            return Err(format!(
                "{revision} {platform}: checksums do not match the files"
            ));
        }
        passed.push(check_recorded_report(
            report,
            tests,
            (revision, run),
            platform,
        )?);
    }
    let (linux, macos, windows) = (
        file("linux", "REPORT")?,
        file("macos", "REPORT")?,
        file("windows", "REPORT")?,
    );
    for key in ["SOURCE_MANIFEST_SHA256", "RUN_ATTEMPT"] {
        let value = report_field(linux, key)?;
        if report_field(macos, key)? != value || report_field(windows, key)? != value {
            return Err(format!("{revision}: {key} differs between platforms"));
        }
    }
    for (os, report) in [("linux", linux), ("macos", macos)] {
        same_commit(report, windows).map_err(|e| format!("{revision} {os}: {e}"))?;
        compare_lists(
            file(os, "TESTS")?,
            file("windows", "TESTS")?,
            os,
            Some(report),
        )
        .map_err(|e| format!("{revision} {os}: {e}"))?;
    }
    let [[l_work, l_core, l_doc], [m_work, m_core, m_doc], [w_work, w_core, w_doc]] = passed[..]
    else {
        return Err("three platforms expected".into());
    };
    if l_doc != m_doc || l_doc != w_doc {
        return Err(format!(
            "{revision}: doctest counts differ between platforms"
        ));
    }
    let declared = l_work
        .checked_sub(w_work)
        .ok_or_else(|| format!("{revision}: Windows passed more tests than Linux"))?;
    Ok(format!(
        "| [{revision}]({revision}/) | {run} | {l_work} / {m_work} | {w_work} ({declared} declared platform-only) | {l_core} / {m_core}, {w_core} | {l_doc} | PASS (strict: commit and hosts checked) |"
    ))
}

/// Exactly the recorded files, every record valid, and the README table is
/// recomputed from the reports: each row must be the line they give.
fn check_recorded(
    files: &std::collections::BTreeMap<String, String>,
    recorded: &[(&str, &str)],
) -> Result<String, String> {
    let mut expected = std::collections::BTreeSet::from(["README.md".to_owned()]);
    for (revision, _) in recorded {
        for platform in PLATFORMS {
            for kind in RECORDED_KINDS {
                expected.insert(format!("{revision}/{platform}-{kind}.txt"));
            }
        }
    }
    if let Some(key) = expected.iter().find(|key| !files.contains_key(*key)) {
        return Err(format!("missing {key}"));
    }
    if let Some(key) = files.keys().find(|key| !expected.contains(*key)) {
        return Err(format!("unlisted recorded file {key}"));
    }
    let readme = files.get("README.md").map_or("", String::as_str);
    for (revision, run) in recorded {
        let row = check_recorded_revision(files, (revision, run))?;
        if !readme.lines().any(|line| line == row) {
            return Err(format!("README has no row: {row}"));
        }
    }
    Ok(format!(
        "CI_EVIDENCE_RECORDED=PASS revisions={} files={}",
        recorded.len(),
        files.len()
    ))
}

/// The files of a recorded directory, by path relative to it.
fn read_recorded(directory: &Path) -> Result<std::collections::BTreeMap<String, String>, String> {
    let list = |path: &Path| -> Result<Vec<(String, std::path::PathBuf, bool)>, String> {
        let error = |e: std::io::Error| format!("{}: {e}", path.display());
        let mut entries = Vec::new();
        for entry in fs::read_dir(path).map_err(error)? {
            let entry = entry.map_err(error)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| format!("{}: name is not text", path.display()))?;
            let is_dir = entry.file_type().map_err(error)?.is_dir();
            entries.push((name, entry.path(), is_dir));
        }
        Ok(entries)
    };
    let text =
        |path: &Path| fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()));
    let mut files = std::collections::BTreeMap::new();
    for (name, path, is_dir) in list(directory)? {
        if !is_dir {
            files.insert(name, text(&path)?);
            continue;
        }
        for (inner, path, is_dir) in list(&path)? {
            if is_dir {
                return Err(format!("unexpected directory {name}/{inner}"));
            }
            files.insert(format!("{name}/{inner}"), text(&path)?);
        }
    }
    Ok(files)
}

/// `ci-evidence-recorded [DIRECTORY]`: `docs/evidence-ci` unless another copy
/// is named.
pub fn recorded(args: &[String]) -> Result<(), String> {
    let directory = match args {
        [] => super::workspace_root()?.join("docs").join("evidence-ci"),
        [directory] => Path::new(directory).to_path_buf(),
        _ => return Err("ci-evidence-recorded takes at most one DIRECTORY".into()),
    };
    println!("{}", check_recorded(&read_recorded(&directory)?, RECORDED)?);
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
    let mut report = format!("FORMAT=GEL_CI_EVIDENCE_2\nCOMMIT={sha}\nSOURCE_MANIFEST_SHA256={manifest_pin}\nRUN_ID={run_id}\nRUN_ATTEMPT={attempt}\nOS={}\nARCH={}\nSCOPE={SCOPE}\n", std::env::consts::OS, std::env::consts::ARCH);
    let rust = Command::new("rustc")
        .arg("-V")
        .output()
        .map_err(|_| "rustc unavailable")?;
    // Toolchain pin comes from a strict comparison, never an arbitrary environment dump.
    if !rust.status.success() || !String::from_utf8_lossy(&rust.stdout).starts_with("rustc 1.85.0 ")
    {
        return Err("expected pinned Rust 1.85.0".into());
    }
    report.push_str(&format!("RUST_VERSION=1.85.0\nREDACTION={REDACTION}\n"));
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
    match check_exclusions(&tests, std::env::consts::OS) {
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
    // Every test named in the property map ran and passed once, or is a
    // declared platform-only test on another platform.
    let excluded_here = |name: &str| {
        EXCLUSIONS
            .iter()
            .any(|(n, condition, _)| *n == name && !compiled_on(condition, std::env::consts::OS))
    };
    match super::property_map::read(&root).and_then(|rows| {
        super::property_map::check(&rows, |n| workspace_counts(&tests, n), excluded_here)
    }) {
        Ok(lines) => report.push_str(&lines),
        Err(e) => {
            success = false;
            report.push_str(&format!("PROPERTY_MAP=MISMATCH {e}\n"));
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
        for (name, _, _) in EXCLUSIONS {
            ran.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let mut other = String::from("workspace-debug\ttest unrelated::case ... ok\n");
        for (name, _, _) in PARTIAL_BRANCHES {
            other.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        // Linux: every excluded-elsewhere test must have run and passed once.
        let lines = check_exclusions(&format!("{other}{ran}"), "linux").unwrap();
        assert_eq!(
            lines.matches("here=RAN_PASSED").count(),
            EXCLUSIONS.len() + PARTIAL_BRANCHES.len()
        );
        assert!(check_exclusions(&other, "linux").is_err());
        assert!(check_exclusions(&format!("{other}{ran}{ran}"), "linux").is_err());
        let ignored = ran.replacen(" ... ok", " ... ignored", 1);
        assert!(check_exclusions(&format!("{other}{ignored}"), "linux").is_err());
        // Other targets: excluded tests are absent and never counted as success.
        let lines = check_exclusions(&other, "windows").unwrap();
        assert_eq!(
            lines.matches("here=EXCLUDED_NOT_COUNTED").count(),
            EXCLUSIONS.len()
        );
        assert!(check_exclusions(&format!("{other}{ran}"), "windows").is_err());
        // Another case id with the same name does not satisfy the workspace check.
        let release_only = ran.replace("workspace-debug\t", "core-release\t");
        assert!(check_exclusions(&format!("{other}{release_only}"), "linux").is_err());
        // Partial-branch tests are checked, not merely listed.
        assert!(check_exclusions(&ran, "linux").is_err());
        let failed_branch = other.replacen(" ... ok", " ... FAILED", 2);
        assert!(check_exclusions(&failed_branch, "windows").is_err());
    }

    #[test]
    fn a_linux_only_test_is_required_on_linux_and_refused_on_macos() {
        let line = |name: &str| format!("workspace-debug\ttest {name} ... ok\n");
        let mut shared = String::new();
        for (name, _, _) in PARTIAL_BRANCHES {
            shared.push_str(&line(name));
        }
        let (mut unix, mut linux) = (String::new(), String::new());
        for (name, condition, _) in EXCLUSIONS {
            match *condition {
                "unix" => unix.push_str(&line(name)),
                _ => linux.push_str(&line(name)),
            }
        }
        assert!(!linux.is_empty());
        let macos = format!("{shared}{unix}");
        let on_linux = format!("{macos}{linux}");
        // One declaration fits all three lists.
        let lines = check_exclusions(&macos, "macos").unwrap();
        assert_eq!(
            lines.matches("here=EXCLUDED_NOT_COUNTED").count(),
            linux.lines().count()
        );
        assert!(check_exclusions(&on_linux, "linux").is_ok());
        assert!(check_exclusions(&on_linux, "macos").is_err());
        assert!(check_exclusions(&macos, "linux").is_err());
        assert!(compare_lists(&macos, &shared, "macos", None).is_ok());
        assert!(compare_lists(&on_linux, &shared, "linux", None).is_ok());
        assert!(compare_lists(&on_linux, &shared, "macos", None).is_err());
        assert!(compare_lists(&macos, &shared, "linux", None).is_err());
        // A partial branch is active only where its condition holds.
        let lines = check_exclusions(&macos, "macos").unwrap();
        assert!(lines.contains("condition=linux here=RAN_PASSED branch=inactive"));
        assert!(lines.contains("condition=unix here=RAN_PASSED branch=active"));
    }

    #[test]
    fn a_name_declared_after_a_revision_may_be_missing_only_with_that_report() {
        let line = |name: &str| format!("workspace-debug\ttest {name} ... ok\n");
        let mut unix = String::new();
        for (name, _, _) in EXCLUSIONS {
            unix.push_str(&line(name));
        }
        let first = EXCLUSIONS[0].0;
        let without = unix.replace(&line(first), "");
        // Without a report, or when the report declares it, it must be there.
        let declares = format!("PLATFORM_EXCLUSION test={first} condition=unix\n");
        assert!(compare_lists(&unix, "", "linux", Some(&declares)).is_ok());
        assert!(compare_lists(&without, "", "linux", None).is_err());
        assert!(compare_lists(&without, "", "linux", Some(&declares)).is_err());
        // A report that does not declare it describes an earlier revision.
        assert!(compare_lists(&without, "", "linux", Some("OS=linux\n")).is_ok());
        // Such a test still may not run on Windows.
        assert!(compare_lists(&without, &line(first), "linux", Some("OS=linux\n")).is_err());
    }

    #[test]
    fn only_a_declared_helper_may_be_ignored_and_on_both_sides() {
        let mut unix = String::new();
        for (name, _, _) in EXCLUSIONS {
            unix.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let helper = format!("workspace-debug\ttest {} ... ignored\n", IGNORED_HELPERS[0]);
        let release = helper.replace("workspace-debug", "core-release");
        let both = format!("{helper}{release}");
        assert!(compare_platforms(&format!("{unix}{both}"), &both).is_ok());
        assert!(compare_platforms(&unix, "").is_ok());
        // Ignored on one side only.
        assert!(compare_platforms(&format!("{unix}{both}"), "").is_err());
        assert!(compare_platforms(&format!("{unix}{both}"), &helper).is_err());
        // Any other ignored test is a non-passing test.
        let other = "workspace-debug\ttest another ... ignored\n";
        assert!(compare_platforms(&format!("{unix}{other}"), other).is_err());
    }

    fn refit_sums(files: &mut std::collections::BTreeMap<String, String>, key: &str) {
        let (revision, name) = key.split_once('/').unwrap();
        let (platform, _) = name.split_once('-').unwrap();
        let text = |kind: &str| files[&format!("{revision}/{platform}-{kind}.txt")].clone();
        let sums = format!(
            "{}  {platform}-REPORT.txt\n{}  {platform}-TESTS.txt\n",
            sha256_hex(text("REPORT").as_bytes()),
            sha256_hex(text("TESTS").as_bytes())
        );
        files.insert(format!("{revision}/{platform}-SHA256SUMS.txt"), sums);
    }

    fn the_records() -> std::collections::BTreeMap<String, String> {
        read_recorded(&crate::workspace_root().unwrap().join("docs/evidence-ci")).unwrap()
    }

    /// Changes one report or test list, with a checksum list that fits, so the
    /// content check has to answer for it.
    fn refused_with_fitting_sums(key: &str, change: impl Fn(&str) -> String, expected: &str) {
        let mut files = the_records();
        let before = files[key].clone();
        let after = change(&before);
        assert_ne!(before, after, "{key}");
        files.insert(key.to_owned(), after);
        refit_sums(&mut files, key);
        let error = check_recorded(&files, RECORDED).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }

    #[test]
    fn the_recorded_revisions_pass_and_cover_three_platforms() {
        let line = check_recorded(&the_records(), RECORDED).unwrap();
        assert_eq!(
            line,
            format!(
                "CI_EVIDENCE_RECORDED=PASS revisions={} files={}",
                RECORDED.len(),
                1 + RECORDED.len() * 9
            )
        );
    }

    #[test]
    fn a_recorded_file_altered_missing_or_extra_is_refused() {
        let files = the_records();
        for key in files.keys().filter(|key| key.contains('/')) {
            // One changed byte in any file of any platform.
            let mut changed = files.clone();
            let text = changed.get_mut(key).unwrap();
            let last = text.pop().unwrap();
            text.push(if last == '\n' { ' ' } else { '\n' });
            assert!(check_recorded(&changed, RECORDED).is_err(), "{key}");
            // A missing file of any platform.
            let mut missing = files.clone();
            missing.remove(key);
            let error = check_recorded(&missing, RECORDED).unwrap_err();
            assert!(error.contains(key.as_str()), "{error}");
        }
        for extra in ["2b29236/notes.txt", "1234567/linux-REPORT.txt", "notes.txt"] {
            let mut more = files.clone();
            more.insert(extra.to_owned(), String::new());
            let error = check_recorded(&more, RECORDED).unwrap_err();
            assert!(error.contains(extra), "{error}");
        }
    }

    #[test]
    fn a_recorded_report_with_fitting_sums_is_still_checked_for_content() {
        let (report, tests) = ("79f6ac5/linux-REPORT.txt", "79f6ac5/linux-TESTS.txt");
        let add = |line: &'static str| move |text: &str| format!("{text}{line}\n");
        // A runner path or host name has no line to live in.
        refused_with_fitting_sums(
            report,
            add("HOST=runner-1"),
            "line outside the report format",
        );
        refused_with_fitting_sums(
            report,
            |t| t.replace("reason=Unix symlink creation", "reason=C:\\a\\runner\\b"),
            "line outside the report format",
        );
        refused_with_fitting_sums(
            report,
            add("PROPERTY id=/home/runner"),
            "line outside the report format",
        );
        // The header states the recorded revision, run, platform and scope.
        refused_with_fitting_sums(
            report,
            |t| t.replace("RUN_ID=42077523488", "RUN_ID=42077523489"),
            "not the recorded run",
        );
        refused_with_fitting_sums(
            report,
            |t| t.replace("COMMIT=79f6ac5", "COMMIT=79f6ac6"),
            "not this revision",
        );
        refused_with_fitting_sums(
            report,
            |t| t.replace("OS=linux", "OS=macos"),
            "another platform",
        );
        refused_with_fitting_sums(
            report,
            |t| t.replacen("not full CI", "full CI", 1),
            "header field",
        );
        // A case that failed, or a count that disagrees with the list.
        refused_with_fitting_sums(
            report,
            |t| t.replacen("accepted=true", "accepted=false", 1),
            "was not accepted",
        );
        refused_with_fitting_sums(
            report,
            |t| t.replacen("failed=0", "failed=1", 1),
            "was not accepted",
        );
        refused_with_fitting_sums(
            tests,
            add("workspace-debug\ttest extra::case ... ok"),
            "does not count its listed tests",
        );
        // Platforms of one revision must agree.
        refused_with_fitting_sums(
            report,
            |t| t.replace("RUN_ATTEMPT=1", "RUN_ATTEMPT=2"),
            "RUN_ATTEMPT differs",
        );
    }

    #[test]
    fn a_recorded_test_list_that_breaks_the_strict_diff_is_refused() {
        let report = "79f6ac5/linux-REPORT.txt";
        let tests = "79f6ac5/linux-TESTS.txt";
        let name = "workspace-debug\ttest shared::case ... ok\n";
        // An undeclared Unix-only test: a name only on Linux, counted in its report.
        let mut files = the_records();
        let list = files[tests].clone();
        files.insert(tests.to_owned(), format!("{list}{name}"));
        let text =
            files[report].replacen("named_test_executions=813", "named_test_executions=814", 1);
        files.insert(report.to_owned(), text);
        refit_sums(&mut files, report);
        let error = check_recorded(&files, RECORDED).unwrap_err();
        assert!(error.contains("shared::case: unix=1 windows=0"), "{error}");
        // A failed test is a non-passing line even with fitting counts.
        refused_with_fitting_sums(
            tests,
            |t| t.replacen(" ... ok", " ... FAILED", 1),
            "non-passing test",
        );
    }

    #[test]
    fn the_readme_table_must_state_what_the_records_state() {
        let files = the_records();
        let readme = &files["README.md"];
        for (revision, _) in RECORDED {
            let row = format!("| [{revision}]({revision}/) |");
            let mut changed = files.clone();
            // The row is gone, or it claims another number.
            let without: Vec<&str> = readme
                .lines()
                .filter(|line| !line.starts_with(&row))
                .collect();
            changed.insert("README.md".to_owned(), without.join("\n"));
            assert!(check_recorded(&changed, RECORDED).is_err(), "{revision}");
            let wrong: Vec<String> = readme
                .lines()
                .map(|line| {
                    if line.starts_with(&row) {
                        line.replacen(" / ", " / 1", 1)
                    } else {
                        line.to_owned()
                    }
                })
                .collect();
            changed.insert("README.md".to_owned(), wrong.join("\n"));
            assert!(check_recorded(&changed, RECORDED).is_err(), "{revision}");
        }
    }

    /// The bytes of one recorded platform as a job log prints them.
    fn job_log(report: &str, tests: &str, line_end: &str) -> String {
        let mut log = format!("2026-01-01T00:00:00.0Z before{line_end}");
        let sums = format!(
            "{}  REPORT.txt\n{}  TESTS.txt\n",
            sha256_hex(report.as_bytes()),
            sha256_hex(tests.as_bytes())
        );
        let block =
            format!("CI_EVIDENCE_REPORT_BEGIN\n{report}{tests}{sums}CI_EVIDENCE_REPORT_END");
        for line in block.lines() {
            log.push_str(&format!("2026-01-01T00:00:00.1Z {line}{line_end}"));
        }
        log.push_str(&format!("2026-01-01T00:00:00.2Z after{line_end}"));
        log
    }

    #[test]
    fn a_job_log_rebuilds_the_exact_recorded_bytes_or_is_refused() {
        let files = the_records();
        let (report, tests) = (
            &files["79f6ac5/windows-REPORT.txt"],
            &files["79f6ac5/windows-TESTS.txt"],
        );
        for line_end in ["\n", "\r\n"] {
            let [rebuilt_report, rebuilt_tests, sums] =
                rebuild_from_log(&job_log(report, tests, line_end)).unwrap();
            assert_eq!(&rebuilt_report, report);
            assert_eq!(&rebuilt_tests, tests);
            assert_eq!(sums.lines().count(), 2);
        }
        let log = job_log(report, tests, "\n");
        // One changed byte in the block no longer fits the printed checksum.
        assert!(rebuild_from_log(&log.replacen("OS=windows", "OS=linux", 1)).is_err());
        // A stray line between the markers does not either.
        let stray = log.replacen("RUN_ATTEMPT=1", "RUN_ATTEMPT=1\n2026 /home/runner/work", 1);
        assert!(rebuild_from_log(&stray).is_err());
        // Missing, doubled or reversed markers.
        let begin = "2026-01-01T00:00:00.1Z CI_EVIDENCE_REPORT_BEGIN\n";
        let end = "2026-01-01T00:00:00.1Z CI_EVIDENCE_REPORT_END\n";
        assert!(rebuild_from_log(&log.replace(begin, "")).is_err());
        assert!(rebuild_from_log(&log.replace(end, "")).is_err());
        assert!(rebuild_from_log(&format!("{begin}{log}")).is_err());
        let reversed = log
            .replace("REPORT_BEGIN", "REPORT_SWAP")
            .replace("REPORT_END", "REPORT_BEGIN")
            .replace("REPORT_SWAP", "REPORT_END");
        assert!(rebuild_from_log(&reversed).is_err());
        assert!(rebuild_from_log("").is_err());
    }

    #[test]
    fn platform_diff_rejects_undeclared_stale_and_windows_only_tests() {
        let mut shared = String::from("workspace-debug\ttest shared::case ... ok\n");
        for (name, _, _) in PARTIAL_BRANCHES {
            shared.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let mut declared = String::new();
        for (name, _, _) in EXCLUSIONS {
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
    fn platform_diff_is_strict_about_lines_duplicates_and_commits() {
        let mut shared = String::new();
        for (name, _, _) in PARTIAL_BRANCHES {
            shared.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        let mut unix = shared.clone();
        for (name, _, _) in EXCLUSIONS {
            unix.push_str(&format!("workspace-debug\ttest {name} ... ok\n"));
        }
        assert!(compare_platforms(&unix, &shared).is_ok());
        // Unknown or malformed lines are errors, not silently skipped.
        assert!(compare_platforms(&format!("{unix}garbage line\n"), &shared).is_err());
        assert!(compare_platforms(&format!("{unix}other-case\ttest x ... ok\n"), &shared).is_err());
        assert!(compare_platforms(
            &format!("{unix}workspace-debug\ttest /path ... ok\n"),
            &shared
        )
        .is_err());
        // A declared exclusion must run exactly once on Unix.
        let first = EXCLUSIONS[0].0;
        let twice = format!("{unix}workspace-debug\ttest {first} ... ok\n");
        assert!(compare_platforms(&twice, &shared).is_err());
        // Reports must share one commit and come from a Unix and a Windows host.
        let report = |commit: &str, os: &str| {
            format!("FORMAT=GEL_CI_EVIDENCE_2\nCOMMIT={commit}\nOS={os}\n")
        };
        assert_eq!(
            same_commit(&report("abc", "linux"), &report("abc", "windows")).unwrap(),
            "abc"
        );
        assert!(same_commit(&report("abc", "macos"), &report("abc", "windows")).is_ok());
        assert!(same_commit(&report("abc", "linux"), &report("def", "windows")).is_err());
        assert!(same_commit(&report("abc", "windows"), &report("abc", "windows")).is_err());
        assert!(same_commit(&report("abc", "linux"), &report("abc", "linux")).is_err());
        let doubled = format!("{}COMMIT=abc\n", report("abc", "linux"));
        assert!(same_commit(&doubled, &report("abc", "windows")).is_err());
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

    /// The index just after the comment or literal starting at `at`, if one does.
    fn literal_end(s: &[u8], at: usize) -> Option<usize> {
        let after = |needle: &[u8], from: usize| {
            s.get(from..)
                .and_then(|rest| rest.windows(needle.len()).position(|w| w == needle))
                .map_or(s.len(), |p| from + p + needle.len())
        };
        let rest = &s[at..];
        if rest.starts_with(b"//") {
            return Some(after(b"\n", at));
        }
        if rest.starts_with(b"/*") {
            let (mut depth, mut i) = (1, at + 2);
            while depth > 0 && i < s.len() {
                if s[i..].starts_with(b"/*") {
                    (depth, i) = (depth + 1, i + 2);
                } else if s[i..].starts_with(b"*/") {
                    (depth, i) = (depth - 1, i + 2);
                } else {
                    i += 1;
                }
            }
            return Some(i);
        }
        if rest[0] == b'"' {
            let mut i = at + 1;
            while i < s.len() && s[i] != b'"' {
                i += 1 + usize::from(s[i] == b'\\');
            }
            return Some((i + 1).min(s.len()));
        }
        if rest[0] == b'\'' {
            if s.get(at + 1) == Some(&b'\\') {
                // The escaped character may itself be a quote.
                return Some(after(b"'", at + 3));
            }
            // A lifetime has no closing quote after one character.
            let width = match *s.get(at + 1)? {
                b if b < 0x80 => 1,
                b if b >= 0xf0 => 4,
                b if b >= 0xe0 => 3,
                _ => 2,
            };
            return (s.get(at + 1 + width) == Some(&b'\'')).then_some(at + 2 + width);
        }
        let start = at + rest.iter().take_while(|b| **b == b'b').count().min(1);
        if s.get(start) == Some(&b'r') {
            let hashes = s[start + 1..].iter().take_while(|b| **b == b'#').count();
            let open = start + 1 + hashes;
            if s.get(open) == Some(&b'"') {
                let mut close = vec![b'"'];
                close.resize(1 + hashes, b'#');
                return Some(after(&close, open + 1));
            }
        }
        None
    }

    /// The condition an attribute puts on a platform, if it puts one: `unix` and
    /// `linux` are the two a declaration can hold. Any other predicate naming a
    /// platform comes back as written, which no declaration matches, so it
    /// fails instead of being read as no condition. `cfg_attr` counts: an
    /// attribute given on one platform only makes a test differ there.
    fn platform_condition(attribute: &str) -> Option<String> {
        const PLATFORM_WORDS: &[&str] = &[
            "unix",
            "windows",
            "target_os",
            "target_family",
            "target_arch",
            "target_env",
            "target_vendor",
        ];
        match attribute {
            "cfg(unix)" => Some("unix".into()),
            "cfg(target_os=\"linux\")" => Some("linux".into()),
            _ if attribute.starts_with("cfg")
                && PLATFORM_WORDS.iter().any(|w| attribute.contains(w)) =>
            {
                Some(attribute.into())
            }
            _ => None,
        }
    }

    /// The module path of a source file, as the name of its tests starts:
    /// `src/a/b.rs` gives `a::b`; a crate root, `tests/x.rs` and an example
    /// give nothing.
    fn module_of(relative: &Path) -> String {
        let parts: Vec<&str> = relative
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        let Some(at) = parts
            .iter()
            .position(|p| matches!(*p, "src" | "tests" | "examples" | "benches"))
        else {
            return String::new();
        };
        let inner = &parts[at + 1..];
        let root =
            inner.len() == 1 && (parts[at] != "src" || matches!(inner[0], "lib.rs" | "main.rs"));
        if root || (parts[at] == "src" && inner.first() == Some(&"bin")) {
            return String::new();
        }
        inner
            .iter()
            .map(|p| p.strip_suffix(".rs").unwrap_or(p))
            .filter(|p| *p != "mod")
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Every test of one Rust source compiled only on some platforms, as (name,
    /// condition). A condition may stand on the test, on an enclosing module or
    /// at the head of the file; several are joined, which no declaration
    /// matches. A gated `mod name;` is refused: its file would look ungated.
    fn platform_tests(module: &str, text: &str) -> Result<Vec<(String, String)>, String> {
        let s = text.as_bytes();
        let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let skip_space = |mut i: usize| {
            while i < s.len() && s[i].is_ascii_whitespace() {
                i += 1;
            }
            i
        };
        // One entry per open brace: a module's name, and the conditions on it.
        let mut scopes: Vec<(Option<String>, Vec<String>)> = vec![(None, Vec::new())];
        let mut pending: Vec<String> = Vec::new();
        let mut found = Vec::new();
        let mut i = 0;
        while i < s.len() {
            if let Some(end) = literal_end(s, i) {
                i = end;
                continue;
            }
            match s[i] {
                b'#' => {
                    let mut j = skip_space(i + 1);
                    let inner = s.get(j) == Some(&b'!');
                    if inner {
                        j = skip_space(j + 1);
                    }
                    if s.get(j) != Some(&b'[') {
                        i += 1;
                        continue;
                    }
                    let (open, mut depth) = (j + 1, 0usize);
                    loop {
                        if j >= s.len() {
                            return Err(format!("{module}: unterminated attribute"));
                        }
                        if let Some(end) = literal_end(s, j) {
                            j = end;
                            continue;
                        }
                        depth = match s[j] {
                            b'[' => depth + 1,
                            b']' => depth - 1,
                            _ => depth,
                        };
                        if depth == 0 {
                            break;
                        }
                        j += 1;
                    }
                    let attribute: String = text[open..j]
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    if inner {
                        if let (Some(condition), Some(scope)) =
                            (platform_condition(&attribute), scopes.last_mut())
                        {
                            scope.1.push(condition);
                        }
                    } else {
                        pending.push(attribute);
                    }
                    i = j + 1;
                }
                b'{' => {
                    scopes.push((None, Vec::new()));
                    pending.clear();
                    i += 1;
                }
                b'}' => {
                    if scopes.len() == 1 {
                        return Err(format!("{module}: unbalanced closing brace"));
                    }
                    scopes.pop();
                    pending.clear();
                    i += 1;
                }
                b';' => {
                    pending.clear();
                    i += 1;
                }
                b if word(b) => {
                    let mut end = i;
                    while end < s.len() && word(s[end]) {
                        end += 1;
                    }
                    let keyword = &text[i..end];
                    i = end;
                    if keyword != "mod" && keyword != "fn" {
                        continue;
                    }
                    let start = skip_space(end);
                    let mut stop = start;
                    while stop < s.len() && word(s[stop]) {
                        stop += 1;
                    }
                    let name = &text[start..stop];
                    i = stop;
                    let own: Vec<String> = pending
                        .iter()
                        .filter_map(|a| platform_condition(a))
                        .collect();
                    let all: Vec<&str> = scopes
                        .iter()
                        .flat_map(|scope| &scope.1)
                        .chain(&own)
                        .map(String::as_str)
                        .collect();
                    if keyword == "mod" {
                        let next = skip_space(stop);
                        if s.get(next) == Some(&b'{') {
                            i = next + 1;
                            scopes.push((Some(name.to_string()), own));
                        } else if !all.is_empty() {
                            return Err(format!("{module}: mod {name} is gated in another file"));
                        }
                    } else if pending.iter().any(|a| a == "test") && !all.is_empty() {
                        let path = scopes.iter().filter_map(|scope| scope.0.as_deref());
                        let parts: Vec<&str> = (!module.is_empty())
                            .then_some(module)
                            .into_iter()
                            .chain(path)
                            .chain([name])
                            .collect();
                        found.push((parts.join("::"), all.join(" and ")));
                    }
                    pending.clear();
                }
                _ => i += 1,
            }
        }
        if scopes.len() != 1 {
            return Err(format!("{module}: unbalanced opening brace"));
        }
        Ok(found)
    }

    /// What the sources and the declarations disagree on, in both directions.
    fn declaration_problems(
        found: &[(String, String)],
        declared: &[(&str, &str, &str)],
    ) -> Vec<String> {
        let mut problems = Vec::new();
        for (name, condition) in found {
            match declared.iter().find(|(d, _, _)| d == name) {
                None => problems.push(format!("{name}: compiled only on {condition}, undeclared")),
                Some((_, d, _)) if d != condition => {
                    problems.push(format!(
                        "{name}: compiled only on {condition}, declared {d}"
                    ));
                }
                Some(_) => {}
            }
        }
        for (name, condition, _) in declared {
            if !found.iter().any(|(f, _)| f == name) {
                problems.push(format!(
                    "{name}: declared {condition}, no such gate in the sources"
                ));
            }
        }
        problems
    }

    #[test]
    fn the_roadmap_and_the_registry_count_the_declarations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let count = |condition: &str| EXCLUSIONS.iter().filter(|e| e.1 == condition).count();
        let (unix, linux) = (count("unix"), count("linux"));
        assert_eq!(unix + linux, EXCLUSIONS.len());
        for (file, words) in [
            (
                "docs/ROADMAP.md",
                format!(
                    "list {} declared platform-only tests ({unix} Unix-only, {linux} Linux-only)",
                    unix + linux
                ),
            ),
            (
                "xtask/src/claims.rs",
                format!("platform-only whole tests ({unix} Unix, {linux} Linux)"),
            ),
        ] {
            let text = fs::read_to_string(root.join(file)).unwrap();
            assert!(text.contains(&words), "{file}: {words}");
        }
    }

    #[test]
    fn every_platform_only_test_of_the_sources_is_declared() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut found = Vec::new();
        for top in ["crates", "xtask"] {
            let tree = crate::tree_scan::walk(&root.join(top)).unwrap();
            for path in &tree.paths {
                if path.extension().is_some_and(|e| e == "rs") {
                    let text = fs::read_to_string(path).unwrap();
                    let module = module_of(path.strip_prefix(root).unwrap());
                    found.extend(platform_tests(&module, &text).unwrap());
                }
            }
        }
        let problems = declaration_problems(&found, EXCLUSIONS);
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn the_source_scan_finds_gates_where_they_stand() {
        let names = |source: &str| platform_tests("m", source).unwrap();
        let one = |name: &str, condition: &str| vec![(name.to_string(), condition.to_string())];
        assert_eq!(
            names("#[cfg(unix)]\n#[test]\nfn a() {}"),
            one("m::a", "unix")
        );
        // The order of attributes, and what stands between them and the name.
        assert_eq!(
            names("#[test]\n#[cfg( target_os = \"linux\" )]\npub async fn a() { }"),
            one("m::a", "linux")
        );
        // File level, and an inner attribute of an inline module.
        assert_eq!(
            names("#![cfg(target_os = \"linux\")]\n#[test]\nfn a() {}"),
            one("m::a", "linux")
        );
        assert_eq!(
            names("mod t { #![cfg(unix)]\n#[test] fn a() {} }\n#[test] fn b() {}"),
            one("m::t::a", "unix")
        );
        // An enclosing module, which ends at its brace.
        assert_eq!(
            names("#[cfg(unix)] mod t { mod u { #[test] fn a() { let _ = '{'; } } }\n#[test] fn b() {}"),
            one("m::t::u::a", "unix")
        );
        // Two conditions and any other predicate fail instead of passing as one.
        assert_eq!(
            names("#[cfg(unix)] mod t { #[cfg(target_os = \"linux\")] #[test] fn a() {} }"),
            one("m::t::a", "unix and linux")
        );
        for gate in [
            "cfg(not(unix))",
            "cfg(windows)",
            "cfg(not(windows))",
            "cfg(all(test, unix))",
            "cfg(target_os = \"macos\")",
            "cfg_attr(windows, ignore)",
        ] {
            let found = names(&format!("#[{gate}]\n#[test]\nfn a() {{}}"));
            assert_eq!(found.len(), 1, "{gate}");
            assert!(
                declaration_problems(&found, &[("m::a", "unix", ""), ("m::a", "linux", "")])
                    .iter()
                    .any(|p| p.contains("compiled only on")),
                "{gate}"
            );
        }
        // Not a platform, not a test, not code.
        for source in [
            "#[cfg(target_pointer_width = \"64\")]\n#[test]\nfn a() {}",
            "#[cfg(panic = \"unwind\")]\n#[test]\nfn a() {}",
            "#[cfg(test)]\nmod t { #[test] fn a() {} }",
            "#[cfg(unix)]\nfn a() {}\n#[test]\nfn b() {}",
            "#[cfg(unix)]\nuse x;\n#[test]\nfn b() {}",
            "#[cfg(unix)]\nstruct S { a: u8 }\n#[test]\nfn b() {}",
            "// #[cfg(unix)]\n#[test]\nfn a() {}",
            "/* /* */ #[cfg(unix)] */\n#[test]\nfn a() {}",
            "const A: &str = \"#[cfg(unix)]\";\n#[test]\nfn a() {}",
            "const A: &str = r#\"\n#[cfg(unix)]\n\"\n\"#;\n#[test]\nfn a() {}",
            "const A: &[u8] = b\"\\\"#[cfg(unix)]\";\n#[test]\nfn a() {}",
            "fn a<'a>(x: &'a str, y: char) { let _ = ('}', '\\'', \"}\"); }\n#[test]\nfn b() {}",
        ] {
            assert_eq!(names(source), Vec::new(), "{source}");
        }
        for broken in [
            "#[cfg(unix)]\nmod a;",
            "#![cfg(unix)]\nmod a;",
            "}",
            "{",
            "#[cfg(unix",
        ] {
            assert!(platform_tests("m", broken).is_err(), "{broken}");
        }
        assert_eq!(names("#[cfg(test)]\nmod a;"), Vec::new());
    }

    #[test]
    fn a_test_name_starts_with_the_module_of_its_file() {
        for (path, module) in [
            ("xtask/src/main.rs", ""),
            ("crates/gel-source/src/lib.rs", ""),
            ("crates/gel-source/src/bundle.rs", "bundle"),
            ("crates/a/src/b/c.rs", "b::c"),
            ("crates/a/src/b/mod.rs", "b"),
            ("crates/a/src/bin/tool.rs", ""),
            ("xtask/tests/memory_cli.rs", ""),
            ("crates/a/examples/demo.rs", ""),
            ("crates/a/tests/support/mod.rs", "support"),
        ] {
            assert_eq!(module_of(Path::new(path)), module, "{path}");
        }
    }

    #[test]
    fn sources_and_declarations_must_agree_in_both_directions() {
        let found = vec![("t::a".to_string(), "unix".to_string())];
        assert!(declaration_problems(&found, &[("t::a", "unix", "r")]).is_empty());
        let undeclared = declaration_problems(&found, &[]);
        assert!(undeclared[0].contains("undeclared"), "{undeclared:?}");
        let other = declaration_problems(&found, &[("t::a", "linux", "r")]);
        assert!(other[0].contains("declared linux"), "{other:?}");
        let stale = declaration_problems(&[], &[("t::a", "unix", "r")]);
        assert!(stale[0].contains("no such gate"), "{stale:?}");
    }
}
