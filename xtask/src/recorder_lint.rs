//! Standalone recorder tool: clippy restriction lints declared in the file.
//! `cargo clippy --workspace` never sees tools/record_evidence.rs, so this gate
//! runs clippy-driver on it directly. It also proves it can fail: a copy with
//! one injected `unwrap()` must be rejected by clippy::unwrap_used.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const TOOL: &str = "record_evidence.rs";
const SUPPORT: &str = "record_support.rs";
const MAIN: &str = "fn main() {";
const PROBE: &str = "fn main() {\n    let _probe: u8 = \"7\".parse().unwrap();";

fn clippy_driver(root: &Path) -> Result<PathBuf, String> {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("rustc --print sysroot: {e}"))?;
    if !output.status.success() {
        return Err(format!("rustc --print sysroot: {}", output.status));
    }
    let sysroot = String::from_utf8(output.stdout).map_err(|e| format!("sysroot: {e}"))?;
    let driver = Path::new(sysroot.trim())
        .join("bin")
        .join(format!("clippy-driver{}", std::env::consts::EXE_SUFFIX));
    if driver.is_file() {
        Ok(driver)
    } else {
        Err(format!("missing {}", driver.display()))
    }
}

fn lint(driver: &Path, source: &Path, out_dir: &Path) -> Result<Output, String> {
    Command::new(driver)
        .args([
            "--edition=2021",
            "--crate-type=bin",
            "--emit=metadata",
            "-D",
            "warnings",
            "--out-dir",
        ])
        .arg(out_dir)
        .arg(source)
        .output()
        .map_err(|e| format!("{}: {e}", driver.display()))
}

fn io(path: &Path, e: std::io::Error) -> String {
    format!("{}: {e}", path.display())
}

pub fn check(root: &Path) -> Result<(), String> {
    let driver = clippy_driver(root)?;
    let tools = root.join("tools");
    let work = root.join("target").join("recorder-lint");
    let negative = work.join("negative");
    // A fresh probe copy on every run; an earlier probe is never reused.
    if negative.exists() {
        fs::remove_dir_all(&negative).map_err(|e| io(&negative, e))?;
    }
    fs::create_dir_all(&negative).map_err(|e| io(&negative, e))?;

    let clean = lint(&driver, &tools.join(TOOL), &work)?;
    if !clean.status.success() {
        return Err(format!(
            "RECORDER_LINT=FAIL\n{}",
            String::from_utf8_lossy(&clean.stderr)
        ));
    }

    let path = tools.join(TOOL);
    let source = fs::read_to_string(&path).map_err(|e| io(&path, e))?;
    if source.matches(MAIN).count() != 1 {
        return Err("RECORDER_LINT=FAIL: expected exactly one `fn main() {`".into());
    }
    let probe_path = negative.join(TOOL);
    fs::write(&probe_path, source.replacen(MAIN, PROBE, 1)).map_err(|e| io(&probe_path, e))?;
    let support = negative.join(SUPPORT);
    fs::copy(tools.join(SUPPORT), &support).map_err(|e| io(&support, e))?;
    let probe = lint(&driver, &probe_path, &negative)?;
    if probe.status.success()
        || !String::from_utf8_lossy(&probe.stderr).contains("clippy::unwrap_used")
    {
        return Err("RECORDER_LINT=FAIL: injected unwrap() was not rejected".into());
    }
    println!("RECORDER_LINT=PASS clean_tool=accepted injected_unwrap=rejected");
    Ok(())
}
