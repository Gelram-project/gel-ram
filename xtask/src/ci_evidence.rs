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
    let mut report = format!("FORMAT=GEL_CI_EVIDENCE_1\nCOMMIT={sha}\nSOURCE_MANIFEST_SHA256={manifest_pin}\nRUN_ID={run_id}\nRUN_ATTEMPT={attempt}\nOS={}\nARCH={}\nSCOPE=workspace tests, doctests, saved R1 recheck; not full CI or a fresh benchmark\n", std::env::consts::OS, std::env::consts::ARCH);
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
            .output()
            .map_err(|_| "cannot execute evidence command")?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut count = 0;
        for line in text.lines().filter_map(safe_test) {
            tests.push_str(&format!("{id}\t{line}\n"));
            count += 1;
        }
        let accepted = out.status.success()
            && match *id {
                "workspace-debug" => count > 0,
                "saved-r1-release" => text.lines().any(|l| l == "COLLECTION_RECHECK=PASS"),
                _ => text.contains("test result: ok."),
            };
        success &= accepted;
        report.push_str(&format!("CASE={id} accepted={accepted} exit_code={:?} named_test_executions={count} stdout_bytes={} stderr_bytes={}\n", out.status.code(), out.stdout.len(), out.stderr.len()));
        println!("CI_EVIDENCE_CASE={id} accepted={accepted}");
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
    if !success {
        return Err("evidence checks failed; no COMPLETE".into());
    }
    write_new(output, "COMPLETE", b"CI_EVIDENCE_SCOPE=PASS\n")?;
    println!("CI_EVIDENCE_REPORT_BEGIN\n{report}{tests}{hashes}CI_EVIDENCE_REPORT_END");
    println!("CI_EVIDENCE=PASS");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
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
