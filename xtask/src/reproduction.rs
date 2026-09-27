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

fn host_lines() -> String {
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

fn load(when: &str) -> String {
    match fs::read_to_string("/proc/loadavg") {
        Ok(l) => format!("loadavg_{when}={}\n", l.trim()),
        Err(_) => format!("loadavg_{when}=NOT_MEASURED\n"),
    }
}

/// SHA-256 of every produced file, relative to the output directory.
fn hashes(dir: &Path, base: &Path, out: &mut Vec<(String, String)>) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.is_dir() {
            hashes(&path, base, out)?;
        } else if meta.is_file() {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let name = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((hex(&digest(&bytes)), name));
        }
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage =
        "usage: cargo run -p xtask -- reproduce NEW_DIR_OUTSIDE_CHECKOUT [--require-isolation]";
    let (dir, require) = match args {
        [dir] => (dir, false),
        [dir, flag] if flag == "--require-isolation" => (dir, true),
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
    let mut failed = false;
    for (name, step) in &steps {
        if failed {
            let _ = writeln!(text, "step={name} result=NOT_RUN");
            continue;
        }
        match step() {
            Ok(None) => {
                let _ = writeln!(text, "step={name} result=PASS");
            }
            Ok(Some(why)) => {
                let _ = writeln!(text, "step={name} result=SKIPPED reason={why}");
            }
            Err(e) => {
                failed = true;
                let first = e.lines().next().unwrap_or("").to_string();
                let _ = writeln!(text, "step={name} result=FAIL error={first}");
            }
        }
        write(&text)?;
    }
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
    let _ = writeln!(text, "REPRODUCTION={verdict}");
    write(&text)?;
    if failed {
        return Err(format!(
            "REPRODUCTION=FAIL see {}",
            out.join("REPRODUCTION.txt").display()
        ));
    }
    fs::write(out.join("COMPLETE.txt"), "REPRODUCTION_COMPLETE=PASS\n")
        .map_err(|e| e.to_string())?;
    println!("REPRODUCTION=PASS {}", out.display());
    Ok(())
}
