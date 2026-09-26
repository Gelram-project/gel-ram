use gel_source::{collection::Collection, hex};
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
                "gel-evidence-{}-{}",
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
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(input: &str) -> String {
    let mut c = Command::new(env!("CARGO_BIN_EXE_gel-evidence"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let o = c.wait_with_output().unwrap();
    assert!(o.status.success(), "{:?}", o.stderr);
    String::from_utf8(o.stdout).unwrap()
}
#[test]
fn own_documents_save_restart_find_and_refuse_cross_source() {
    let s = Scratch::new();
    let a = s.0.join("a.txt");
    let b = s.0.join("b.txt");
    let out = s.0.join("bank");
    fs::write(&a, "Zażółć gęślą jaźń.\nnot ").unwrap();
    fs::write(&b, "approved\nRAM is volatile.").unwrap();
    let log = run(&format!(
        "add {}\nadd {}\nfind not approved\nsave {}\nexit\n",
        a.display(),
        b.display(),
        out.display()
    ));
    assert!(log.contains("FIND=UNKNOWN"));
    let pin = hex(&gel_source::digest(&fs::read(&out).unwrap()));
    assert!(log.contains(&pin));
    let log = run(&format!(
        "load {pin} {}\nfind zażółć gęślą\nproof 1\nfind ram is volatile\nproof 1\nexit\n",
        out.display()
    ));
    assert!(log.contains("REOPEN=PASS"));
    assert_eq!(log.matches("CITATION=PASS").count(), 2);
    assert!(log.contains("doc=1"));
    assert!(log.contains("doc=2"));
    assert!(!log.contains("REFUSED"));
}
#[test]
fn update_delete_clear_results_and_old_snapshot_stays() {
    let s = Scratch::new();
    let p = s.0.join("doc");
    let q = s.0.join("updated");
    let out = s.0.join("old");
    fs::write(&p, "old answer").unwrap();
    fs::write(&q, "new answer").unwrap();
    let log=run(&format!("add {}\nsave {}\nfind old answer\nreplace 1 {}\nproof 1\nfind old answer\nfind new answer\ndrop 1\nproof 1\nfind new answer\nexit\n",p.display(),out.display(),q.display()));
    assert_eq!(log.matches("NO_CURRENT_RESULT").count(), 2);
    assert_eq!(log.matches("FIND=UNKNOWN").count(), 2);
    let bytes = fs::read(&out).unwrap();
    let c = Collection::from_bytes(&bytes, gel_source::digest(&bytes)).unwrap();
    assert_eq!(c.get(1).unwrap().text(), "old answer");
}
#[test]
fn corrupt_load_refusal_retains_current_collection() {
    let s = Scratch::new();
    let p = s.0.join("doc");
    let out = s.0.join("bad");
    fs::write(&p, "retained text").unwrap();
    fs::write(&out, "broken").unwrap();
    let log = run(&format!(
        "add {}\nload {} {}\nfind retained text\nproof 1\nexit\n",
        p.display(),
        "0".repeat(64),
        out.display()
    ));
    assert!(log.contains("REFUSED"));
    assert!(log.contains("CITATION=PASS"));
}
#[test]
fn controls_in_document_never_become_terminal_escape() {
    let s = Scratch::new();
    let p = s.0.join("doc");
    fs::write(&p, "find me \x1b[2J\u{202e}hidden").unwrap();
    let log = run(&format!("add {}\nfind find me\nexit\n", p.display()));
    assert!(!log.contains('\x1b'));
    assert!(!log.contains('\u{202e}'));
    assert!(log.contains("FIND=HIT"));
}

#[test]
fn complete_update_restart_and_corruption_scenario() {
    let s = Scratch::new();
    let original = s.0.join("original.txt");
    let revised = s.0.join("revised.txt");
    let old = s.0.join("old.snapshot");
    let new = s.0.join("new.snapshot");
    let bad = s.0.join("corrupt.snapshot");
    fs::write(
        &original,
        "Safety condition:\nDo not\nopen the valve while pressure is high.",
    )
    .unwrap();
    fs::write(
        &revised,
        "Safety condition:\nKeep the valve closed until pressure is zero.",
    )
    .unwrap();
    let first = run(&format!(
        "add {}\nfind open the valve\nproof 1\nfind invented instruction\nsave {}\nreplace 1 {}\nproof 1\nfind open the valve\nfind keep the valve closed\nproof 1\nsave {}\nexit\n",
        original.display(), old.display(), revised.display(), new.display()
    ));
    assert_eq!(first.matches("REFUSED").count(), 1, "{first}");
    assert!(first.contains("Do not\\nopen the valve"), "{first}");
    assert_eq!(first.matches("NO_CURRENT_RESULT").count(), 1);
    assert_eq!(first.matches("FIND=UNKNOWN").count(), 2);
    assert_eq!(first.matches("CITATION=PASS").count(), 2);
    let old_bytes = fs::read(&old).unwrap();
    let new_bytes = fs::read(&new).unwrap();
    let old_pin = hex(&gel_source::digest(&old_bytes));
    let new_pin = hex(&gel_source::digest(&new_bytes));
    assert_ne!(old_pin, new_pin);
    let mut corrupt = old_bytes.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    fs::write(&bad, corrupt).unwrap();
    // A fresh process has no access to the first process's in-memory state.
    let second = run(&format!(
        "load {new_pin} {}\nfind keep the valve closed\nproof 1\nload {old_pin} {}\nfind open the valve\nproof 1\nload {old_pin} {}\nfind open the valve\nproof 1\nexit\n",
        new.display(), old.display(), bad.display()
    ));
    assert_eq!(second.matches("REOPEN=PASS").count(), 2);
    assert_eq!(second.matches("REFUSED").count(), 1);
    assert_eq!(second.matches("CITATION=PASS").count(), 3);
    assert!(!second.contains("FIND=UNKNOWN"));
    assert_eq!(fs::read(old).unwrap(), old_bytes);
    assert_eq!(fs::read(new).unwrap(), new_bytes);
}
#[test]
fn demo_really_executes_the_collection() {
    let o = Command::new(env!("CARGO_BIN_EXE_gel-evidence"))
        .arg("--demo")
        .output()
        .unwrap();
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("GEL_EVIDENCE_DEMO=PASS"));
    assert_eq!(s.matches("CITATION=PASS").count(), 2);
    assert_eq!(s.matches("FIND=UNKNOWN").count(), 2);
}
#[test]
#[cfg(unix)]
fn sigkill_after_save_ack_keeps_a_readable_snapshot() {
    use std::{
        io::{BufRead, BufReader},
        sync::mpsc,
        time::Duration,
    };
    let s = Scratch::new();
    let input = s.0.join("doc");
    let output = s.0.join("saved");
    fs::write(&input, "durable test-only phrase").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_gel-evidence"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            if line.contains("BUNDLE_SHA256=") {
                let _ = tx.send(line);
                break;
            }
        }
    });
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(format!("add {}\nsave {}\n", input.display(), output.display()).as_bytes())
        .unwrap();
    let ack = rx.recv_timeout(Duration::from_secs(20));
    let _ = child.kill();
    child.wait().unwrap();
    reader.join().unwrap();
    assert!(ack.is_ok(), "no acknowledged save");
    let pin = hex(&gel_source::digest(&fs::read(&output).unwrap()));
    assert!(ack.unwrap().contains(&pin));
    let log = run(&format!(
        "load {pin} {}\nfind durable test\nproof 1\nexit\n",
        output.display()
    ));
    assert!(log.contains("REOPEN=PASS"));
    assert!(log.contains("CITATION=PASS"));
}

#[test]
#[cfg(unix)]
fn sigkill_at_observable_publication_never_installs_a_partial_snapshot() {
    use std::time::{Duration, Instant};
    let s = Scratch::new();
    let input = s.0.join("large.txt");
    let text = "source-only test line\n".repeat(200_000);
    fs::write(&input, &text).unwrap();
    let mut expected = Collection::new();
    expected.add("large.txt", &text).unwrap();
    let old = s.0.join("old");
    let old_pin = Collection::new().save_new(&old).unwrap();
    let old_bytes = fs::read(&old).unwrap();
    for trial in 0..3 {
        let directory = s.0.join(format!("trial{trial}"));
        fs::create_dir(&directory).unwrap();
        let output = directory.join("new");
        let mut child = Command::new(env!("CARGO_BIN_EXE_gel-evidence"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(format!("add {}\nsave {}\n", input.display(), output.display()).as_bytes())
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut observed = false;
        while Instant::now() < deadline {
            if fs::read_dir(&directory).unwrap().next().is_some() {
                observed = true;
                break;
            }
            if child.try_wait().unwrap().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let _ = child.kill();
        child.wait().unwrap();
        assert!(observed, "publication was not observed");
        // Scheduling may expose a temporary file or the fully installed destination.
        // No ACK was consumed: either absence or a complete validated snapshot is legal.
        if output.exists() {
            let c = Collection::load(&output, expected.root()).unwrap();
            assert_eq!(c.to_bytes(), expected.to_bytes());
        }
        assert_eq!(fs::read(&old).unwrap(), old_bytes);
        Collection::load(&old, old_pin).unwrap();
    }
}
