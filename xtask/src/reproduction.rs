//! One-command reproduction for outside testers (docs/REPRODUCE.md).
//!
//! Runs, in order, the network-isolation check, the full report (verify and
//! runtime campaigns), the format mutation matrix and the grep comparison into
//! one new directory, and writes a single REPRODUCTION.txt with the revision,
//! host, load and every step's result. A failed step stops the run; later steps
//! are marked NOT_RUN. COMPLETE.txt is written only when every step passed or
//! was skipped for a stated reason.
use gel_source::{digest, hex};
use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub(crate) fn host_lines() -> String {
    let mut s = format!(
        "os={} arch={} available_parallelism={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(1, usize::from)
    );
    let cpu = fs::read_to_string("/proc/cpuinfo").ok().and_then(|c| {
        c.lines()
            .find(|l| l.starts_with("model name"))
            .map(str::to_string)
    });
    let mem = fs::read_to_string("/proc/meminfo").ok().and_then(|m| {
        m.lines()
            .find(|l| l.starts_with("MemTotal:"))
            .map(str::to_string)
    });
    let _ = writeln!(s, "{}", cpu.unwrap_or_else(|| "cpu=NOT_MEASURED".into()));
    let _ = writeln!(s, "{}", mem.unwrap_or_else(|| "memory=NOT_MEASURED".into()));
    s
}

pub(crate) fn load(when: &str) -> String {
    match fs::read_to_string("/proc/loadavg") {
        Ok(l) => format!("loadavg_{when}={}\n", l.trim()),
        Err(_) => format!("loadavg_{when}=NOT_MEASURED\n"),
    }
}

/// SHA-256 of every produced file, relative to the output directory.
pub(crate) fn hashes(
    dir: &Path,
    base: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let entries = super::audit_io::paths(dir)?;
    for path in entries {
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.is_dir() {
            hashes(&path, base, out)?;
        } else if meta.is_file() {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let relative = path
                .strip_prefix(base)
                .map_err(|_| "report path outside inventory root")?;
            for component in relative.components() {
                let part = component
                    .as_os_str()
                    .to_str()
                    .ok_or("non-UTF8 report path")?;
                if part.contains('\\') {
                    return Err("ambiguous backslash in report filename".into());
                }
            }
            let name = relative
                .to_str()
                .ok_or("non-UTF8 report path")?
                .replace('\\', "/");
            if name.chars().any(char::is_control) {
                return Err("control character in report path".into());
            }
            out.push((hex(&digest(&bytes)), name));
        } else {
            return Err("report inventory contains a symlink or non-regular entry".into());
        }
    }
    Ok(())
}

/// The recorded result of one step. In strict mode (a release gate) a step
/// skipped for a missing dependency is a failure, never a pass.
fn classify(outcome: Result<Option<String>, String>, strict: bool) -> (&'static str, String) {
    match outcome {
        Ok(None) => ("PASS", String::new()),
        Ok(Some(why)) if strict => ("FAIL", format!(" error=skipped in strict mode: {why}")),
        Ok(Some(why)) => ("SKIPPED", format!(" reason={why}")),
        Err(e) => ("FAIL", format!(" error={}", e.lines().next().unwrap_or(""))),
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage = "usage: cargo run -p xtask -- reproduce NEW_DIR_OUTSIDE_CHECKOUT [--require-isolation] [--strict]";
    let Some((dir, flags)) = args.split_first() else {
        return Err(usage.into());
    };
    let (mut require, mut strict) = (false, false);
    for flag in flags {
        match flag.as_str() {
            "--require-isolation" if !require => require = true,
            // Release gate: a skipped step is a failure, never a pass.
            "--strict" if !strict => strict = true,
            _ => return Err(usage.into()),
        }
    }
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
    // Root, including root mapped in a user namespace, bypasses file
    // permissions, so the permission-denial tests would fail for that reason alone.
    let effective_root = fs::read_to_string("/proc/self/status").is_ok_and(|s| {
        s.lines()
            .find_map(|l| l.strip_prefix("Uid:"))
            .and_then(|ids| ids.split_whitespace().nth(1))
            == Some("0")
    });
    if effective_root {
        return Err("REPRODUCTION=REFUSED running as root (also root mapped by unshare --map-root-user); run unprivileged, e.g. unshare --user --net or bwrap --unshare-net".into());
    }
    fs::create_dir(out).map_err(|e| format!("new output directory required: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;

    let isolation = super::isolation::status();
    let mut text = String::from("FORMAT=GEL_REPRODUCTION_1\n");
    let _ = writeln!(text, "{}", super::bench_compare::revision(&root));
    text.push_str(&host_lines());
    let _ = writeln!(text, "{isolation}");
    text.push_str(&load("start"));
    let write =
        |text: &str| fs::write(out.join("REPRODUCTION.txt"), text).map_err(|e| e.to_string());
    if require && !isolation.ends_with("=VERIFIED") {
        text.push_str("result=REFUSED network isolation was required but not verified\n");
        write(&text)?;
        return Err(format!("REPRODUCTION=REFUSED {isolation}"));
    }

    let as_arg = |p: PathBuf| p.to_string_lossy().into_owned();
    let tools = available("grep") && available("sha256sum");
    type Step<'a> = (
        &'a str,
        Box<dyn Fn() -> Result<Option<String>, String> + 'a>,
    );
    let steps: Vec<Step> = vec![
        (
            "report",
            Box::new(|| super::reproduce::report(&[as_arg(out.join("report"))]).map(|()| None)),
        ),
        (
            "mutation-matrix",
            Box::new(|| {
                super::mutation_matrix::run(&[
                    "--write".into(),
                    as_arg(out.join("mutation-matrix.txt")),
                ])
                .map(|()| None)
            }),
        ),
        (
            "bench-compare",
            Box::new(|| {
                if !tools {
                    return Ok(Some("grep or sha256sum not available".into()));
                }
                super::bench_compare::run(&[as_arg(out.join("bench"))]).map(|()| None)
            }),
        ),
    ];
    let (mut pass, mut fail, mut skipped, mut not_run) = (0, 0, 0, 0);
    for (name, step) in &steps {
        if fail > 0 {
            not_run += 1;
            let _ = writeln!(text, "step={name} result=NOT_RUN");
            continue;
        }
        let (result, detail) = classify(step(), strict);
        match result {
            "PASS" => pass += 1,
            "SKIPPED" => skipped += 1,
            _ => fail += 1,
        }
        let _ = writeln!(text, "step={name} result={result}{detail}");
        write(&text)?;
    }
    let failed = fail > 0;
    if let Ok(summary) = fs::read_to_string(out.join("bench/summary.txt")) {
        for line in summary.lines() {
            let _ = writeln!(text, "bench: {line}");
        }
    }
    text.push_str(&load("end"));
    let mut files = Vec::new();
    hashes(&out, &out, &mut files)?;
    for (hash, name) in files.iter().filter(|(_, n)| n != "REPRODUCTION.txt") {
        let _ = writeln!(text, "sha256 {hash}  {name}");
    }
    let verdict = if failed { "FAIL" } else { "PASS" };
    let isolated = if isolation.ends_with("=VERIFIED") {
        "VERIFIED"
    } else {
        "NOT_VERIFIED"
    };
    let line = format!(
        "REPRODUCTION={verdict} pass={pass} fail={fail} skipped={skipped} not_run={not_run} isolation={isolated} required_isolation={} strict={}",
        if require { "yes" } else { "no" },
        if strict { "yes" } else { "no" },
    );
    let _ = writeln!(text, "{line}");
    write(&text)?;
    if failed {
        return Err(format!(
            "{line} see {}",
            out.join("REPRODUCTION.txt").display()
        ));
    }
    // COMPLETE means a complete report of the declared run, not that every
    // possible check passed on every system.
    fs::write(
        out.join("COMPLETE.txt"),
        format!("REPRODUCTION_COMPLETE=PASS complete report of the declared run: {line}\n"),
    )
    .map_err(|e| e.to_string())?;
    println!("{line} {}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skip_passes_only_outside_strict_mode() {
        let skip = || Ok(Some("grep missing".to_string()));
        assert_eq!(classify(skip(), false).0, "SKIPPED");
        let strict = classify(skip(), true);
        assert_eq!(strict.0, "FAIL");
        assert!(strict.1.contains("strict mode"), "{}", strict.1);
        assert_eq!(classify(Ok(None), true), ("PASS", String::new()));
        assert_eq!(
            classify(Err("first\nsecond".into()), false),
            ("FAIL", " error=first".to_string())
        );
    }
    #[test]
    fn report_inventory_rejects_links_and_preserves_complete_hashes() {
        let dir = std::env::temp_dir().join(format!(
            "gel-report-inventory-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("one.txt"), "one").unwrap();
        let mut entries = Vec::new();
        hashes(&dir, &dir, &mut entries).unwrap();
        assert_eq!(entries, vec![(hex(&digest(b"one")), "one.txt".into())]);
        assert!(hashes(&dir, &dir.join("wrong-root"), &mut Vec::new()).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.join("one.txt"), dir.join("link.txt")).unwrap();
            assert!(hashes(&dir, &dir, &mut Vec::new()).is_err());
        }
        fs::remove_dir_all(&dir).unwrap();
    }
}
