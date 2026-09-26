//! Standalone recorder tool: clippy restriction lints forced from the command
//! line. `cargo clippy --workspace` never sees tools/record_evidence.rs, so
//! this gate runs clippy-driver on it directly. `-F` (forbid) cannot be relaxed
//! by an `#[allow]` inside the file. The gate also proves it can fail: one
//! copy per probe injects a panicking construct that must be rejected.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const TOOL: &str = "record_evidence.rs";
const SUPPORT: &str = "record_support.rs";
const MAIN: &str = "fn main() {";
const FORBIDDEN: &[&str] = &[
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::unreachable",
    "clippy::todo",
    "clippy::unimplemented",
    "clippy::indexing_slicing",
    "clippy::string_slice",
    "clippy::arithmetic_side_effects",
    "clippy::print_stdout",
    "clippy::print_stderr",
    "clippy::disallowed_macros",
    "clippy::disallowed_methods",
    "clippy::dbg_macro",
];
/// (name, statement injected at the start of main, lint that must reject it)
const PROBES: &[(&str, &str, &str)] = &[
    (
        "unwrap",
        r#"let _p: u8 = "7".parse().unwrap();"#,
        "unwrap_used",
    ),
    (
        "allow-bypass",
        r#"#[allow(clippy::unwrap_used)] let _p: u8 = "7".parse().unwrap();"#,
        "unwrap_used",
    ),
    (
        "assert",
        "assert!(std::env::args_os().len() < 99);",
        "disallowed_macros",
    ),
    (
        "index",
        "let _p = [1u8, 2][std::env::args_os().len()];",
        "indexing_slicing",
    ),
    (
        "str-slice",
        r#"let _p = &String::from("ab")[..1];"#,
        "string_slice",
    ),
    (
        "arithmetic",
        "let _p = std::env::args_os().len() - 1;",
        "arithmetic_side_effects",
    ),
    (
        "spawn",
        "let _p = std::thread::spawn(|| ());",
        "disallowed_methods",
    ),
    ("print", r#"println!("probe");"#, "print_stdout"),
    ("dbg", "let _p = dbg!(1u8);", "dbg_macro"),
    (
        "env-args",
        "let _p = std::env::args().count();",
        "disallowed_methods",
    ),
];

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

fn lint(root: &Path, driver: &Path, source: &Path, out_dir: &Path) -> Result<Output, String> {
    let mut command = Command::new(driver);
    command
        .env("CLIPPY_CONF_DIR", root.join("tools").join("recorder-lint"))
        .args([
            "--edition=2021",
            "--crate-type=bin",
            "--emit=metadata",
            "-D",
            "warnings",
        ]);
    for lint in FORBIDDEN {
        command.args(["-F", lint]);
    }
    command
        .arg("--out-dir")
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
    // Fresh probe copies on every run; an earlier probe is never reused.
    if work.exists() {
        fs::remove_dir_all(&work).map_err(|e| io(&work, e))?;
    }
    fs::create_dir_all(&work).map_err(|e| io(&work, e))?;

    let clean = lint(root, &driver, &tools.join(TOOL), &work)?;
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
    for (name, statement, expected) in PROBES {
        let dir = work.join(name);
        fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
        let probe_path = dir.join(TOOL);
        let probed = source.replacen(MAIN, &format!("{MAIN}\n    {statement}"), 1);
        fs::write(&probe_path, probed).map_err(|e| io(&probe_path, e))?;
        let support = dir.join(SUPPORT);
        fs::copy(tools.join(SUPPORT), &support).map_err(|e| io(&support, e))?;
        let probe = lint(root, &driver, &probe_path, &dir)?;
        if probe.status.success() || !String::from_utf8_lossy(&probe.stderr).contains(expected) {
            return Err(format!(
                "RECORDER_LINT=FAIL: probe {name} was not rejected by {expected}"
            ));
        }
    }
    println!(
        "RECORDER_LINT=PASS clean_tool=accepted forbidden_lints={} rejected_probes={}",
        FORBIDDEN.len(),
        PROBES.len()
    );
    Ok(())
}
