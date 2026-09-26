//! Real recorder failures: no completion marker, overwrite, or panic.
use std::{fs, path::Path, process::Command};

fn compile(source: &Path, target: &Path) {
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg(source)
        .arg("-o")
        .arg(target)
        .status()
        .expect("rustc starts");
    assert!(status.success());
}

#[test]
fn recorder_existing_log_missing_executable_and_eof_fail_closed() {
    let root = std::env::temp_dir().join(format!(
        "gel-recorder-process-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let recorder = root.join(format!("recorder{}", std::env::consts::EXE_SUFFIX));
    compile(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../tools/record_evidence.rs"),
        &recorder,
    );
    let empty_source = root.join("empty.rs");
    fs::write(&empty_source, "fn main() {}\n").unwrap();
    let empty_child = root.join(format!("empty{}", std::env::consts::EXE_SUFFIX));
    compile(&empty_source, &empty_child);
    let prompt_source = root.join("prompt_then_exit.rs");
    fs::write(
        &prompt_source,
        "use std::io::Write; fn main() { print!(\"gel> \" ); std::io::stdout().flush().unwrap(); }\n",
    ).unwrap();
    let prompt_child = root.join(format!("prompt_then_exit{}", std::env::consts::EXE_SUFFIX));
    compile(&prompt_source, &prompt_child);
    let log_failure_source = root.join("log_failure.rs");
    fs::write(
        &log_failure_source,
        r#"
use std::io::Write;
fn main() {
    std::fs::remove_file("commands.txt").unwrap();
    std::fs::create_dir("commands.txt").unwrap();
    print!("gel> "); std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    println!("ADDED id=1"); std::io::stdout().flush().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
}
"#,
    )
    .unwrap();
    let log_failure_child = root.join(format!("log_failure{}", std::env::consts::EXE_SUFFIX));
    compile(&log_failure_source, &log_failure_child);
    for (case, diagnostic) in [
        (
            "existing",
            "RECORDING_REFUSED: existing output commands.txt",
        ),
        ("missing", "RECORDING_FAILED: spawn:"),
        ("eof", "RECORDING_FAILED: EOF or timeout before marker"),
        ("closed_stdin", "RECORDING_FAILED: command I/O:"),
        ("command_log", "RECORDING_FAILED: command log I/O:"),
    ] {
        let dir = root.join(case);
        fs::create_dir(&dir).unwrap();
        if case == "existing" {
            fs::write(dir.join("commands.txt"), b"KEEP ORIGINAL\n").unwrap();
        }
        let binary = match case {
            "eof" => empty_child.clone(),
            "closed_stdin" => prompt_child.clone(),
            "command_log" => log_failure_child.clone(),
            _ => root.join("nonexistent"),
        };
        let output = Command::new(&recorder)
            .arg(binary)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{case}");
        if case == "existing" {
            assert_eq!(output.status.code(), Some(2), "{case}: refusal exit code");
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        // A child that exits right after its prompt fails either on the command
        // pipe or at the next marker, depending on scheduling; both fail closed.
        let alternative = case == "closed_stdin"
            && stderr.contains("RECORDING_FAILED: EOF or timeout before marker");
        assert!(
            stderr.contains(diagnostic) || alternative,
            "{case}: {stderr}"
        );
        assert!(!stderr.contains("panicked"), "{case}: {stderr}");
        assert!(!dir.join("COMPLETE.txt").exists(), "{case}");
        if case == "existing" {
            assert_eq!(
                fs::read(dir.join("commands.txt")).unwrap(),
                b"KEEP ORIGINAL\n"
            );
            assert!(!dir.join("process-1.txt").exists());
        } else {
            let status = fs::read_to_string(dir.join("process-1-status.txt")).unwrap();
            assert!(status.starts_with("FAILED"), "{case}: {status}");
        }
    }
    fs::remove_dir_all(&root).unwrap();
}

fn fresh_root(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "gel-recorder-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn compiled_recorder(root: &Path) -> std::path::PathBuf {
    let recorder = root.join(format!("recorder{}", std::env::consts::EXE_SUFFIX));
    compile(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../tools/record_evidence.rs"),
        &recorder,
    );
    recorder
}

fn compiled_child(root: &Path, name: &str, source: &str) -> std::path::PathBuf {
    let path = root.join(format!("{name}.rs"));
    fs::write(&path, source).unwrap();
    let binary = root.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    compile(&path, &binary);
    binary
}

#[test]
fn recorder_usage_and_closed_display_fail_closed() {
    use std::io::Read;
    use std::process::Stdio;
    let root = fresh_root("display");
    let recorder = compiled_recorder(&root);
    // A wrong command line is refused with exit 2 before any output exists.
    for (index, args) in [
        &[][..],
        &["a", "b"][..],
        &["a", "--other"][..],
        &["--update"][..],
        &["relative-binary"][..],
        &["relative-binary", "--update"][..],
    ]
    .iter()
    .enumerate()
    {
        let dir = root.join(format!("usage-{index}"));
        fs::create_dir(&dir).unwrap();
        let output = Command::new(&recorder)
            .args(*args)
            .current_dir(&dir)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains("RECORDING_REFUSED: usage"), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0, "{args:?}");
    }
    // The display disappears while the child is alive: controlled failure,
    // child killed, no completion marker and no command logged.
    let child = compiled_child(
        &root,
        "prompt_wait",
        "use std::io::{BufRead, Write}; fn main() { print!(\"gel> \"); \
         std::io::stdout().flush().unwrap(); \
         for line in std::io::stdin().lock().lines() { if line.is_err() { break; } } }\n",
    );
    let dir = root.join("closed-display");
    fs::create_dir(&dir).unwrap();
    let mut running = Command::new(&recorder)
        .arg(&child)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut display = running.stdout.take().unwrap();
    let mut seen = Vec::new();
    let mut buf = [0u8; 256];
    while !seen.windows(5).any(|w| w == b"gel> ") {
        let n = display.read(&mut buf).unwrap();
        assert!(n > 0, "recorder ended before the child prompt");
        seen.extend_from_slice(&buf[..n]);
    }
    drop(display);
    let output = running.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("RECORDING_FAILED: display I/O"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!dir.join("COMPLETE.txt").exists());
    assert_eq!(fs::read(dir.join("commands.txt")).unwrap(), b"");
    let status = fs::read_to_string(dir.join("process-1-status.txt")).unwrap();
    assert!(status.starts_with("FAILED display I/O"), "{status}");
    // Linux: a full display device fails the first banner write.
    #[cfg(target_os = "linux")]
    {
        let dir = root.join("full-display");
        fs::create_dir(&dir).unwrap();
        let output = Command::new(&recorder)
            .arg(root.join("never-started"))
            .current_dir(&dir)
            .stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            )
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(stderr.contains("RECORDING_FAILED: display I/O"), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
        assert!(!dir.join("COMPLETE.txt").exists());
        assert!(!dir.join("process-1.txt").exists());
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn recorder_invalid_emitted_pin_fails_closed() {
    let root = fresh_root("pin");
    let recorder = compiled_recorder(&root);
    // Answers every marker of the default walkthrough, then emits a bad pin.
    let child = compiled_child(
        &root,
        "bad_pin",
        r#"
use std::io::{BufRead, Write};
fn main() {
    let mut out = std::io::stdout();
    print!("gel> "); out.flush().unwrap();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let reply = if line.starts_with("add memory") { "ADDED id=1" }
            else if line.starts_with("add unicode") { "ADDED id=2" }
            else if line == "find imaginary evidence" { "FIND=UNKNOWN" }
            else if line.starts_with("find") { "FIND=HIT" }
            else if line.starts_with("proof") { "CITATION=PASS" }
            else if line.starts_with("save") { "BUNDLE_SHA256=not-a-64-hex-pin;" }
            else { "Closed." };
        println!("{reply}"); print!("gel> "); out.flush().unwrap();
    }
}
"#,
    );
    let dir = root.join("run");
    fs::create_dir(&dir).unwrap();
    let output = Command::new(&recorder)
        .arg(&child)
        .current_dir(&dir)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("RECORDING_FAILED: invalid emitted pin"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!dir.join("COMPLETE.txt").exists());
    assert!(!dir.join("process-2.txt").exists());
    let status = fs::read_to_string(dir.join("process-1-status.txt")).unwrap();
    assert!(status.starts_with("FAILED invalid emitted pin"), "{status}");
    let commands = fs::read_to_string(dir.join("commands.txt")).unwrap();
    assert!(commands.ends_with("save checkpoint.gelset\n"), "{commands}");
    fs::remove_dir_all(&root).unwrap();
}
