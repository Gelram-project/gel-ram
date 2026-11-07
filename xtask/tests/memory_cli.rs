// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! The real command: the control operation in child processes, on Linux.
#![cfg(target_os = "linux")]
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const XTASK: &str = env!("CARGO_BIN_EXE_xtask");
const CONTROL_KIB: u64 = 64 * 1024;
/// Processes of the control. In any other order than rising, a number that
/// varies by chance cannot look like a running total; seven rise by chance
/// once in 5,040 runs.
const REPS: usize = 7;
/// Even in huge pages, 64 MiB take at least this many faults of the child alone.
const FAULT_FLOOR: u64 = 32;

/// The directory of one test; it goes when the test ends, a failed one too.
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("gel-memory-cli-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The first number of `key=min,median,max` in the record line of the control.
fn lowest(record: &str, key: &str) -> u64 {
    let prefix = format!("{key}=");
    let value = record
        .split(' ')
        .find_map(|word| word.strip_prefix(&prefix))
        .expect("the key is in the record line");
    value.split(',').next().unwrap().parse().unwrap()
}

/// `key` in each sample line, in the order of the processes.
fn per_process(text: &str, key: &str) -> Vec<u64> {
    let prefix = format!("{key}=");
    text.lines()
        .filter(|line| line.starts_with("sample "))
        .map(|line| {
            let word = line.split(' ').find_map(|w| w.strip_prefix(&prefix));
            word.expect("the key is in the sample line")
                .parse()
                .unwrap()
        })
        .collect()
}

fn check(file: &Path) -> (bool, String) {
    let output = Command::new(XTASK)
        .args(["memory-check"])
        .arg(file)
        .output()
        .expect("xtask starts");
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.success(), text)
}

fn memory_run(out: &Path, work: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(XTASK);
    command
        .arg("memory-run")
        .arg("--out")
        .arg(out)
        .arg("--work")
        .arg(work)
        .args(args);
    command
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn program(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A run whose every process exits with 1 ends in a record the check refuses,
/// without a build: `cargo` and the operation are two small programs here.
fn refused_run(root: &Path, out: &Path) -> Output {
    let tools = root.join("tools");
    let target = root.join("target");
    program(&tools.join("cargo"), "#!/bin/sh\nexit 0\n");
    program(
        &target.join("release/examples/speed_q8_read"),
        "#!/bin/sh\nexit 1\n",
    );
    let mut path = OsString::from(&tools);
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap());
    let work = root.join("work");
    fs::create_dir(&work).unwrap();
    memory_run(out, &work, &["--only", "q8_read_8192", "--target-root"])
        .arg(&target)
        .env("PATH", path)
        .output()
        .expect("xtask starts")
}

#[test]
fn the_control_shows_its_known_peak_and_a_changed_record_is_refused() {
    let scratch = Scratch::new();
    let root = scratch.0.as_path();
    let out = root.join("out");

    // One sample is not a result: nothing runs and no directory appears.
    let single = memory_run(
        &out,
        root,
        &["--only", "control_touch_64mib", "--reps", "1"],
    )
    .output()
    .expect("xtask starts");
    assert!(!single.status.success());
    assert!(!out.exists());

    // A build directory that cannot be resolved stops the run before anything
    // exists.
    let lost = memory_run(
        &out,
        root,
        &["--only", "control_touch_64mib", "--target-root"],
    )
    .arg(root.join("no-such-directory"))
    .output()
    .expect("xtask starts");
    assert!(!lost.status.success());
    assert_eq!(names(root), [""; 0], "{}", stderr(&lost));

    // A run the check refuses leaves neither a record nor a directory.
    let refused = refused_run(root, &out);
    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains("GEL_MEMORY_RUN=REFUSED"),
        "{}",
        stderr(&refused)
    );
    assert!(stderr(&refused).contains("exit is not 0"));
    assert!(!out.exists());
    assert_eq!(names(&root.join("work")), [""; 0]);

    let reps = REPS.to_string();
    let run = memory_run(
        &out,
        root,
        &["--only", "control_touch_64mib", "--reps", &reps],
    )
    .output()
    .expect("xtask starts");
    assert!(run.status.success(), "{}", stderr(&run));
    let file = out.join("MEMORY.txt");
    let text = fs::read_to_string(&file).unwrap();
    let (accepted, said) = check(&file);
    assert!(accepted, "{said}");
    assert!(
        said.contains(&format!("records=1 samples={REPS}")),
        "{said}"
    );
    let faults = per_process(&text, "minor_faults");
    assert_eq!(faults.len(), REPS);
    assert!(
        !text.contains(root.to_str().unwrap()),
        "a path was recorded"
    );

    let record = text.lines().find(|l| l.starts_with("record ")).unwrap();
    // The block was resident at its peak and gone before the end: the peak is
    // the high-water mark, not the last resident set. A stalled host can miss
    // the pause of one process, so one process has to show it.
    assert!(lowest(record, "peak_rss_kib") >= CONTROL_KIB, "{record}");
    assert!(lowest(record, "rss_last_kib") < CONTROL_KIB / 2, "{record}");
    assert!(lowest(record, "mapped_peak_kib") >= CONTROL_KIB, "{record}");
    assert!(lowest(record, "minor_faults") >= FAULT_FLOOR, "{record}");
    // Each process is counted alone. A running total rises by a whole process
    // at every step; huge pages may spread the counts, but not in that order.
    assert!(
        faults.windows(2).any(|w| w[1] < w[0] + FAULT_FLOOR),
        "{faults:?}"
    );
    assert!(lowest(record, "threads_peak") >= 1, "{record}");
    assert_eq!(lowest(record, "payload_bytes"), CONTROL_KIB * 1024);

    let changed = root.join("changed.txt");
    let cases = [
        (" exit=0", " exit=1", "exit is not 0"),
        (" polls=", " pollz=", "unknown field pollz"),
        (" peak_rss_kib=", " peak_rss_kib=x", "not a whole number"),
        (" resident=yes", " resident=no", "field resident differs"),
        (" kind=control", " kind=bin", "field kind differs"),
    ];
    for (from, to, reason) in cases {
        fs::write(&changed, text.replacen(from, to, 1)).unwrap();
        let (accepted, said) = check(&changed);
        assert!(!accepted, "{from}");
        assert!(said.contains("GEL_MEMORY_CHECK=REFUSED"), "{said}");
        assert!(said.contains(reason), "{said}");
    }
    fs::write(&changed, text.replace("op=control_touch_64mib", "op=other")).unwrap();
    let (accepted, said) = check(&changed);
    assert!(!accepted);
    assert!(said.contains("op is not an operation"), "{said}");
}
