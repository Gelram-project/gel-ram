use gel_live_lab::literal;
use gel_source::{digest, hex};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let p = std::env::temp_dir().join(format!(
                "gel-literal-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
    }
    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, bytes).unwrap();
        p
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn cli(args: &[&Path]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gel-live-lab"))
        .args(args)
        .output()
        .unwrap()
}
fn text(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr)
}

#[test]
fn saved_record_reopens_under_its_pin_and_is_never_replaced() {
    let s = Scratch::new();
    let source = s.file(
        "source.txt",
        "Łódź, café, cafe\u{301}, 🦀\n".repeat(60).as_bytes(),
    );
    let lit = literal::read(&source).unwrap();
    let saved = s.0.join("record.q8");
    let pin = literal::save(&saved, lit.record()).unwrap();
    let raw = fs::read(&saved).unwrap();
    assert_eq!(raw.len(), literal::SAVED_BYTES);
    assert_eq!(digest(&raw), pin);
    let back = literal::reopen(&saved, &pin).unwrap();
    assert_eq!(back.record().phase(), lit.record().phase());
    assert_eq!(back.record().active_mask(), lit.record().active_mask());
    assert!(literal::save(&saved, lit.record()).is_err());
    assert_eq!(fs::read(&saved).unwrap(), raw);
    let mut wrong = pin;
    wrong[0] ^= 1;
    assert!(literal::reopen(&saved, &wrong).is_err());
    let mut changed = raw.clone();
    changed[12 + 500] ^= 1;
    let changed_path = s.file("changed.q8", &changed);
    assert!(literal::reopen(&changed_path, &pin).is_err());
}

#[test]
fn files_that_are_not_one_regular_nonempty_file_are_refused() {
    let s = Scratch::new();
    assert!(literal::read(&s.file("empty.txt", b"")).is_err());
    assert!(literal::read(&s.0).is_err());
    #[cfg(unix)]
    {
        let target = s.file("target.txt", b"bytes");
        let link = s.0.join("link.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(literal::read(&link).is_err());
    }
}

#[test]
fn command_line_save_and_reopen() {
    let s = Scratch::new();
    let source = s.file("source.txt", b"GEL RAM keeps the bytes it was given.\n");
    let saved = s.0.join("record.q8");
    let out = cli(&[Path::new("--literal"), &source, Path::new("--save"), &saved]);
    let shown = text(&out);
    assert!(out.status.success(), "{shown}");
    for want in [
        "ROUNDTRIP=4/4 DIFFERENT_BITS=0",
        "TAMPER=REJECTED",
        "\"GEL RAM keeps the bytes it was given.\\n\"",
        "SAVED without replacing any file",
        "GEL_LIVE_LAB_LITERAL=PASS",
    ] {
        assert!(shown.contains(want), "{want}\n{shown}");
    }
    let plain = text(&cli(&[Path::new("--literal"), &source]));
    assert!(
        plain.contains("Not written.") && !plain.contains("REOPEN=PASS"),
        "{plain}"
    );
    let pin = hex(&digest(&fs::read(&saved).unwrap()));
    assert!(shown.contains(&pin), "{shown}");
    let reopen = cli(&[Path::new("--reopen"), Path::new(&pin), &saved]);
    let shown = text(&reopen);
    assert!(reopen.status.success(), "{shown}");
    assert!(shown.contains("REOPEN=PASS"), "{shown}");
    let again = cli(&[Path::new("--literal"), &source, Path::new("--save"), &saved]);
    assert!(!again.status.success());
}

#[test]
fn command_line_refuses_a_misplaced_flag_and_accepts_a_non_utf8_path() {
    let out = cli(&[Path::new("--literal"), Path::new("--save")]);
    assert!(text(&out).contains("usage"), "{}", text(&out));
    // Linux file systems accept a name that is not UTF-8; APFS on macOS does not.
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::ffi::OsStrExt;
        let s = Scratch::new();
        let odd = s.0.join(std::ffi::OsStr::from_bytes(b"name-\xff.txt"));
        fs::write(&odd, b"bytes under a name that is not UTF-8").unwrap();
        let out = cli(&[Path::new("--literal"), &odd]);
        assert!(out.status.success(), "{}", text(&out));
    }
}

#[test]
fn command_line_refuses_a_second_file_and_a_wrong_pin() {
    let s = Scratch::new();
    let a = s.file("a.txt", b"first");
    let b = s.file("b.txt", b"second");
    let out = cli(&[Path::new("--literal"), &a, &b]);
    assert!(!out.status.success(), "{}", text(&out));
    let saved = s.0.join("a.q8");
    let lit = literal::read(&a).unwrap();
    literal::save(&saved, lit.record()).unwrap();
    let out = cli(&[Path::new("--reopen"), Path::new(&"0".repeat(64)), &saved]);
    assert!(!out.status.success());
    assert!(text(&out).contains("REJECTED"), "{}", text(&out));
}
