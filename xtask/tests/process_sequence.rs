#[path = "../src/process_sequence.rs"]
mod process_sequence;
use std::{fs, path::PathBuf, process::Command};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn first_and_middle_native_failure_stop_later_success() {
    let dir = std::env::temp_dir().join(format!(
        "gel-native-sequence-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let scratch = Scratch(dir);
    let source = scratch.0.join("child.rs");
    fs::write(
        &source,
        r#"
        use std::{fs::OpenOptions, io::Write};
        fn main() {
            let args: Vec<_> = std::env::args().collect();
            let mut trace = OpenOptions::new().create(true).append(true).open(&args[1]).unwrap();
            writeln!(trace, "{}", args[2]).unwrap();
            trace.sync_all().unwrap();
            // Deliberately misleading output must not override the exit status.
            println!("FIXTURE=PASS");
            std::process::exit(args[3].parse().unwrap());
        }
    "#,
    )
    .unwrap();
    let binary = scratch
        .0
        .join(format!("child{}", std::env::consts::EXE_SUFFIX));
    let built = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    for (case, exits, expected_trace, failing) in [
        ("first", [17, 0, 0], "0\n", Some(0)),
        ("middle", [0, 23, 0], "0\n1\n", Some(1)),
        ("success", [0, 0, 0], "0\n1\n2\n", None),
    ] {
        let trace = scratch.0.join(case);
        let mut commands: Vec<_> = exits
            .iter()
            .enumerate()
            .map(|(i, code)| {
                let mut command = Command::new(&binary);
                command.arg(&trace).arg(i.to_string()).arg(code.to_string());
                command
            })
            .collect();
        let result = process_sequence::sequence(&mut commands);
        match failing {
            Some(i) => assert!(result
                .unwrap_err()
                .starts_with(&format!("command {i} failed:"))),
            None => result.unwrap(),
        }
        assert_eq!(fs::read_to_string(trace).unwrap(), expected_trace);
    }
    let mut commands = [
        Command::new(scratch.0.join("missing-executable")),
        Command::new(&binary),
    ];
    assert!(process_sequence::sequence(&mut commands)
        .unwrap_err()
        .starts_with("command 0 could not start:"));
}
