//! Contract of `gel-evidence --batch` (schema gel-evidence/1): JSON Lines on
//! stdout only, diagnostics on stderr, HIT/UNKNOWN/INCOMPLETE/ERROR kept apart
//! in the records and in the exit code, and a stop at the first ERROR.
use gel_source::{digest, hex};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let p = std::env::temp_dir().join(format!(
                "gel-batch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("{e}"),
            }
        }
    }
    fn file(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

struct Run {
    code: i32,
    stdout: Vec<String>,
    stderr: String,
}

fn batch(input: &[u8]) -> Run {
    let mut c = Command::new(env!("CARGO_BIN_EXE_gel-evidence"))
        .arg("--batch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input).unwrap();
    let o = c.wait_with_output().unwrap();
    let stdout = String::from_utf8(o.stdout).unwrap();
    let lines: Vec<String> = stdout.lines().map(str::to_string).collect();
    for line in &lines {
        assert!(
            line.starts_with("{\"schema\":\"gel-evidence/1\",\"record\":\"") && line.ends_with('}'),
            "stdout must carry only schema records: {line}"
        );
    }
    Run {
        code: o.status.code().unwrap(),
        stdout: lines,
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn summary(run: &Run) -> &str {
    let last = run.stdout.last().unwrap();
    assert!(last.contains("\"record\":\"summary\""), "{last}");
    assert!(
        last.contains(&format!("\"exit_code\":{}", run.code)),
        "{last}"
    );
    last
}

#[test]
fn hits_and_unknown_are_data_with_exact_quotes_and_exit_zero() {
    let s = Scratch::new();
    let a = s.file("a.txt", "Say \"hi\" \\ back\tnow\nZażółć gęślą jaźń.\n");
    let run = batch(
        format!(
            "add {}\nlist\nfind say hi back\nproof 1\nfind zażółć gęślą\nfind invented answer\n",
            a.display()
        )
        .as_bytes(),
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty(), "{}", run.stderr);
    assert!(run.stdout[0].contains("\"record\":\"session\""));
    let find = &run.stdout[3];
    assert!(find.contains("\"status\":\"HIT\""), "{find}");
    // The quote is the whole matching line. Quote characters, backslash and tab
    // are escaped, not altered or dropped, and the pin covers the exact bytes.
    assert!(
        find.contains(r#""quote":"Say \"hi\" \\ back\tnow","#),
        "{find}"
    );
    let quote_pin = hex(&digest("Say \"hi\" \\ back\tnow".as_bytes()));
    assert!(find.contains(&quote_pin), "{find}");
    assert!(run.stdout[4].contains("\"correspondence_not_truth\":true"));
    assert!(
        run.stdout[5].contains("\"quote\":\"Zażółć gęślą jaźń.\""),
        "{}",
        run.stdout[5]
    );
    assert!(run.stdout[6].contains("\"status\":\"UNKNOWN\""));
    let end = summary(&run);
    assert!(end.contains("\"executed\":6,\"not_run\":0"), "{end}");
    assert!(
        end.contains("\"HIT\":2,\"UNKNOWN\":1,\"INCOMPLETE\":0,\"ERROR\":0"),
        "{end}"
    );
}

#[test]
fn a_skipped_long_line_is_incomplete_with_exit_three() {
    let s = Scratch::new();
    let long = format!("{}\nshort line\n", "x ".repeat(3000));
    let a = s.file("long.txt", &long);
    let run = batch(format!("add {}\nfind x\n", a.display()).as_bytes());
    assert_eq!(run.code, 3, "{}", run.stderr);
    assert!(
        run.stdout[2].contains("\"status\":\"INCOMPLETE\""),
        "{}",
        run.stdout[2]
    );
    assert!(
        run.stdout[2].contains("\"skipped_long_lines\":1"),
        "{}",
        run.stdout[2]
    );
    assert!(summary(&run).contains("\"INCOMPLETE\":1,\"ERROR\":0"));
}

#[test]
fn the_first_error_stops_the_batch_and_the_rest_is_counted() {
    let run = batch(b"proof 1\nlist\n\nfind anything\n");
    assert_eq!(run.code, 2);
    assert_eq!(run.stdout.len(), 3, "{:?}", run.stdout);
    assert!(run.stdout[1].contains("\"status\":\"ERROR\",\"error\":\"NO_CURRENT_RESULT\""));
    assert!(
        run.stderr.starts_with("ERROR seq=1 command=proof"),
        "{}",
        run.stderr
    );
    let end = summary(&run);
    assert!(end.contains("\"executed\":1,\"not_run\":2"), "{end}");
}

#[test]
fn over_long_and_non_utf8_commands_are_errors_not_silent_skips() {
    let mut long = b"find ".to_vec();
    long.extend(std::iter::repeat_n(b'y', 5000));
    long.extend(b"\nlist\n");
    let run = batch(&long);
    assert_eq!(run.code, 2);
    assert!(run.stdout[1].contains("\"error\":\"COMMAND_LIMIT\""));
    assert!(summary(&run).contains("\"executed\":1,\"not_run\":1"));

    let run = batch(b"find \xff\xfe\n");
    assert_eq!(run.code, 2);
    assert!(run.stdout[1].contains("\"error\":\"UTF8_REQUIRED\""));
}

#[test]
fn saved_collections_reopen_by_pin_in_batch_mode() {
    let s = Scratch::new();
    let a = s.file("a.txt", "RAM is volatile.\n");
    let bank = s.0.join("bank");
    let run =
        batch(format!("add {}\nsave {}\nexit\nlist\n", a.display(), bank.display()).as_bytes());
    assert_eq!(run.code, 0, "{}", run.stderr);
    let pin = hex(&digest(&fs::read(&bank).unwrap()));
    assert!(run.stdout[2].contains(&format!("\"bundle_sha256\":\"{pin}\"")));
    assert!(summary(&run).contains("\"executed\":3,\"not_run\":1"));
    let run = batch(format!("load {pin} {}\nfind ram is volatile\n", bank.display()).as_bytes());
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout[1].contains("\"command\":\"load\",\"status\":\"OK\""));
    assert!(run.stdout[2].contains("\"status\":\"HIT\""));
}
