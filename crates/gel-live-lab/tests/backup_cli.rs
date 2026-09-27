//! gel-backup: separate-word arguments (paths with spaces), one result line on
//! stdout, and exit codes 0 (done), 3 (not restorable), 2 (refused/failed).
use gel_source::{collection::Collection, hex, Hash};
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
        let p = std::env::temp_dir().join(format!(
            "gel-backup-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn tool(args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_gel-backup"))
        .args(args)
        .output()
        .unwrap();
    (
        o.status.code().unwrap(),
        String::from_utf8(o.stdout).unwrap(),
        String::from_utf8(o.stderr).unwrap(),
    )
}

fn snapshot(dir: &Path) -> (PathBuf, Hash) {
    let mut c = Collection::new();
    c.add("notes.txt", "RAM is volatile.\n").unwrap();
    let path = dir.join("my bank.gelset");
    let pin = c.save_new(&path).unwrap();
    (path, pin)
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn full_cycle_with_spaces_in_paths_and_distinct_exit_codes() {
    let t = Scratch::new();
    let (snap, pin) = snapshot(&t.0);
    let pin = hex(&pin);
    let dir = t.0.join("backup one");
    let target = t.0.join("restored copy.gelset");

    let (code, out, err) = tool(&["create", &pin, s(&snap), s(&dir)]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("BACKUP=CREATED COMPLETE bytes="), "{out}");

    let (code, out, _) = tool(&["inspect", &pin, s(&dir)]);
    assert_eq!(code, 0);
    assert!(out.contains("withdrawn=false temporaries=0"), "{out}");

    let (code, out, err) = tool(&["restore", &pin, s(&dir), s(&target)]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("RESTORE=PASS"), "{out}");
    assert_eq!(fs::read(&target).unwrap(), fs::read(&snap).unwrap());

    let (code, out, _) = tool(&["restore", &pin, s(&dir), s(&target)]);
    assert_eq!(code, 2);
    assert_eq!(out.trim(), "RESTORE=REFUSED reason=TARGET_EXISTS");

    let (code, _, err) = tool(&["withdraw", s(&dir), "superseded by a later collection"]);
    assert_eq!(code, 0, "{err}");
    let (code, out, _) = tool(&["inspect", &pin, s(&dir)]);
    assert_eq!(code, 3);
    assert!(out.contains("withdrawn=true"), "{out}");
    let (code, out, _) = tool(&["restore", &pin, s(&dir), s(&t.0.join("again"))]);
    assert_eq!(code, 2);
    assert_eq!(out.trim(), "RESTORE=REFUSED reason=WITHDRAWN");

    let (code, out, _) = tool(&["delete", &"0".repeat(64), s(&snap)]);
    assert_eq!(code, 2);
    assert_eq!(out.trim(), "DELETE=REFUSED reason=PIN_MISMATCH");
    let (code, out, _) = tool(&["delete", &pin, s(&snap)]);
    assert_eq!(code, 0);
    assert!(out.contains("not a secure erase"), "{out}");
    assert!(!snap.exists() && target.exists());
}

#[test]
fn an_unfinished_backup_is_not_restorable_and_usage_errors_write_no_result() {
    let t = Scratch::new();
    let (snap, pin) = snapshot(&t.0);
    let dir = t.0.join("unfinished");
    fs::create_dir(&dir).unwrap();
    fs::copy(&snap, dir.join("collection.gelset")).unwrap();
    let (code, out, _) = tool(&["inspect", &hex(&pin), s(&dir)]);
    assert_eq!(code, 3);
    assert!(out.starts_with("BACKUP=INCOMPLETE"), "{out}");

    let (code, out, err) = tool(&["restore", "not-a-pin"]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.starts_with("usage:"), "{err}");
}
