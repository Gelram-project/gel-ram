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
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(diagnostic), "{case}: {stderr}");
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
