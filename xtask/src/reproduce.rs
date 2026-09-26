//! Local report runner. Never uploads results and never overwrites an output directory.
use std::{fs, io::Write, path::Path, process::Command};

fn accepted(success: bool, stdout: &[u8], stderr: &[u8], marker: &str) -> bool {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    success
        && out.lines().any(|line| {
            line == marker
                || line
                    .strip_prefix(marker)
                    .is_some_and(|rest| rest.starts_with(' '))
        })
        && !out.contains("DEGRADED_REFERENCE_SERIAL_FALLBACK")
        && !err.contains("DEGRADED_REFERENCE_SERIAL_FALLBACK")
}

#[derive(Debug, PartialEq, Eq)]
struct Identity {
    revision: String,
    working_tree: String,
}

fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    let result = Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot execute Git: {e}"))?;
    if !result.status.success() {
        return Err("Git metadata exists but Git failed; not treating this as an archive".into());
    }
    String::from_utf8(result.stdout).map_err(|_| "Git metadata is not UTF-8".into())
}

fn identity(root: &Path, pin: Option<&str>) -> Result<Identity, String> {
    let git_present = match fs::symlink_metadata(root.join(".git")) {
        Ok(meta) if !meta.file_type().is_symlink() && (meta.is_file() || meta.is_dir()) => true,
        Ok(_) => return Err("unsupported Git metadata type".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(format!("cannot inspect Git metadata: {e}")),
    };
    if !git_present && pin.is_none() {
        return Err("archive report requires an independently reviewed SOURCE-SHA256SUMS.txt pin as the final argument".into());
    }
    let snapshot = match pin {
        Some(pin) => {
            let (files, bytes) = super::source_bundle::validated_identity(root, pin)?;
            format!("MANIFEST_SHA256={pin}\nSOURCE_FILES={files}\nSOURCE_BYTES={bytes}\n")
        }
        None => "MANIFEST=NOT_PINNED; Git status alone is not a content snapshot\n".into(),
    };
    if !git_present {
        return Ok(Identity {
            revision: format!("SOURCE_KIND=ARCHIVE\nGIT_REVISION=UNAVAILABLE\n{snapshot}"),
            working_tree:
                "GIT_STATUS=UNAVAILABLE; archive inventory verified against supplied pin\n".into(),
        });
    }
    let revision = git_text(root, &["rev-parse", "--verify", "HEAD"])?;
    let hash = revision.trim();
    if !matches!(hash.len(), 40 | 64) || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("unexpected Git revision".into());
    }
    Ok(Identity {
        revision: format!("SOURCE_KIND=GIT\nGIT_REVISION={hash}\n{snapshot}"),
        working_tree: git_text(root, &["status", "--short", "--untracked-files=all"])?,
    })
}

fn capture(
    output: &Path,
    name: &str,
    program: &str,
    args: &[&str],
    marker: &str,
) -> Result<(), String> {
    let result = Command::new(program)
        .args(args)
        .current_dir(super::workspace_root()?)
        .output()
        .map_err(|e| e.to_string())?;
    let mut log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join(name))
        .map_err(|e| e.to_string())?;
    log.write_all(&result.stdout)
        .and_then(|_| log.write_all(&result.stderr))
        .map_err(|e| e.to_string())?;
    if !accepted(
        result.status.success(),
        &result.stdout,
        &result.stderr,
        marker,
    ) {
        return Err(format!(
            "{name} failed; partial logs retained, report is NOT complete"
        ));
    }
    println!("completed {name}");
    Ok(())
}
pub fn report(args: &[String]) -> Result<(), String> {
    if !(1..=2).contains(&args.len()) {
        return Err("usage: cargo run -p xtask -- report NEW_DIRECTORY_OUTSIDE_CHECKOUT [REVIEWED_MANIFEST_SHA256]".into());
    }
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let pin = args.get(1).map(String::as_str);
    // Validate before creating output or executing a long campaign.
    let initial_identity = identity(&root, pin)?;
    let output = Path::new(&args[0]);
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if parent
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("report must be outside the source checkout".into());
    }
    fs::create_dir(output).map_err(|e| format!("new report directory required: {e}"))?;
    let output = output.canonicalize().map_err(|e| e.to_string())?;
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(24);
    let mut hardware=format!("os={} arch={} available_parallelism={} max_requested_workers={}\nCPU_ONLY=true host_isolation=false\n",
        std::env::consts::OS,std::env::consts::ARCH,std::thread::available_parallelism().map_or(1,usize::from),workers);
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpu) = fs::read_to_string("/proc/cpuinfo") {
            if let Some(model) = cpu.lines().find(|l| l.starts_with("model name")) {
                hardware.push_str(model);
                hardware.push('\n');
            }
        }
        if let Ok(mem) = fs::read_to_string("/proc/meminfo") {
            for line in mem
                .lines()
                .filter(|l| l.starts_with("MemTotal:") || l.starts_with("MemAvailable:"))
            {
                hardware.push_str(line);
                hardware.push('\n');
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    hardware.push_str("cpu_model_and_system_RAM=NOT_MEASURED (supply manually)\n");
    hardware.push_str("Timing ratios belong to this machine/run; no fastest-only filtering.\nLogs may contain local paths: review before sharing.\n");
    fs::write(output.join("hardware.txt"), hardware).map_err(|e| e.to_string())?;
    capture(&output, "rustc.txt", "rustc", &["-Vv"], "rustc")?;
    fs::write(output.join("revision.txt"), &initial_identity.revision)
        .map_err(|e| e.to_string())?;
    fs::write(
        output.join("working-tree.txt"),
        &initial_identity.working_tree,
    )
    .map_err(|e| e.to_string())?;
    capture(
        &output,
        "verify.txt",
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "-p",
            "xtask",
            "--",
            "verify",
        ],
        "GEL_VERIFY_ALL=PASS",
    )?;
    capture(
        &output,
        "live-lab.txt",
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--",
            "--demo",
        ],
        "GEL_LIVE_LAB_DEMO=PASS",
    )?;
    capture(
        &output,
        "evidence-lab.txt",
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
        "GEL_EVIDENCE_DEMO=PASS",
    )?;
    capture(
        &output,
        "quantization-matrix.txt",
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-cli",
            "--example",
            "quantization_matrix",
        ],
        "QUANTIZATION_MATRIX=PASS",
    )?;
    let campaign = output.join("collection-campaign");
    let campaign = campaign.to_str().ok_or("report path must be UTF-8")?;
    capture(
        &output,
        "collection-campaign.txt",
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-source",
            "--example",
            "collection_campaign",
            "--",
            campaign,
        ],
        "COLLECTION_CAMPAIGN=PASS",
    )?;
    capture(
        &output,
        "collection-recheck.txt",
        "cargo",
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
            campaign,
        ],
        "COLLECTION_RECHECK=PASS",
    )?;
    let mut budgets = vec![1, workers];
    budgets.sort();
    budgets.dedup();
    for rep in 0..3 {
        for n in [32, 512, 8192] {
            for policy in if rep % 2 == 0 {
                ["active", "archive"]
            } else {
                ["archive", "active"]
            } {
                for &w in &budgets {
                    capture(
                        &output,
                        &format!("scan-{rep}-{n}-{policy}-{w}.txt"),
                        "cargo",
                        &[
                            "run",
                            "--locked",
                            "--offline",
                            "--release",
                            "-p",
                            "gel-phase-quad",
                            "--example",
                            "quad_compare",
                            "--",
                            "--orbs",
                            &n.to_string(),
                            "--rounds",
                            "9",
                            "--workers",
                            &w.to_string(),
                            "--policy",
                            policy,
                        ],
                        "Q8_QUAD_EXACT=PASS",
                    )?;
                }
            }
        }
    }
    if identity(&root, pin)? != initial_identity {
        return Err("source identity changed during report; no completion marker".into());
    }
    fs::write(output.join("COMPLETE.txt"),"REPORT_COMPLETE=PASS\nAll configured invocations passed; review every timing including slower runs.\nSource identity rechecked; see revision.txt for whether content was pinned.\nSynthetic scan-only data; not semantic accuracy or the full historical campaign.\n").map_err(|e|e.to_string())?;
    println!("REPORT_COMPLETE=PASS {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "gel-report-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn seed(&self) -> String {
            let text = b"synthetic report fixture\n";
            fs::write(self.0.join("README.md"), text).unwrap();
            let manifest = format!(
                "{}  README.md\n",
                gel_source::hex(&gel_source::digest(text))
            );
            fs::write(self.0.join("SOURCE-SHA256SUMS.txt"), &manifest).unwrap();
            gel_source::hex(&gel_source::digest(manifest.as_bytes()))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn archive_requires_independent_pin() {
        let f = Fixture::new();
        let pin = f.seed();
        assert!(identity(&f.0, None).is_err());
        assert!(identity(&f.0, Some(&"0".repeat(64))).is_err());
        let result = identity(&f.0, Some(&pin)).unwrap();
        assert!(result.revision.contains("SOURCE_KIND=ARCHIVE"));
        assert!(result.revision.contains("GIT_REVISION=UNAVAILABLE"));
        assert!(result.revision.contains(&pin));
    }
    #[test]
    fn archive_mutation_and_extra_files_are_not_approved() {
        let f = Fixture::new();
        let pin = f.seed();
        identity(&f.0, Some(&pin)).unwrap();
        fs::write(f.0.join("extra.txt"), b"extra").unwrap();
        assert!(identity(&f.0, Some(&pin)).is_err());
        fs::remove_file(f.0.join("extra.txt")).unwrap();
        fs::write(f.0.join("README.md"), b"changed").unwrap();
        assert!(identity(&f.0, Some(&pin)).is_err());
    }
    #[test]
    fn broken_git_does_not_fall_back_to_archive() {
        let f = Fixture::new();
        let pin = f.seed();
        fs::write(f.0.join(".git"), b"not a worktree pointer\n").unwrap();
        assert!(identity(&f.0, Some(&pin)).is_err());
    }
    #[test]
    fn status_and_fallback_must_pass_on_both_streams() {
        assert!(accepted(true, b"DONE=PASS\n", b"", "DONE=PASS"));
        assert!(accepted(true, b"DONE=PASS count=3\n", b"", "DONE=PASS"));
        for out in [
            "echo DONE=PASS",
            "DONE=PASS_FAKE",
            "DONE=PASS\nDEGRADED_REFERENCE_SERIAL_FALLBACK",
        ] {
            assert!(!accepted(true, out.as_bytes(), b"", "DONE=PASS"));
        }
        assert!(!accepted(false, b"DONE=PASS", b"", "DONE=PASS"));
        assert!(!accepted(
            true,
            b"DONE=PASS",
            b"DEGRADED_REFERENCE_SERIAL_FALLBACK",
            "DONE=PASS"
        ));
        assert!(!accepted(true, b"", b"DONE=PASS", "DONE=PASS"));
    }
}
