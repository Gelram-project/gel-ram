use gel_source::{
    import_text, load_bundle, read_regular, write_bundle_new, Address, BundleError, Error,
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier,
    },
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "gel-source-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("scratch: {e}"),
            }
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn data() -> gel_source::EncodedCorpus {
    import_text("Zażółć 🦀\r\nIt's exact.".as_bytes(), "Notes").unwrap()
}

#[test]
fn persisted_roundtrip_and_no_temporary_files() {
    let root = Scratch::new();
    let path = root.0.join("bank.gelsrc");
    let b = data();
    let pin = write_bundle_new(&path, &b).unwrap();
    drop(b);
    let c = load_bundle(&path, pin).unwrap();
    assert_eq!(
        c.quote(Address { node: 1, role: 1 }).unwrap().quote(),
        "Zażółć 🦀\r\nIt's exact."
    );
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
#[test]
fn existing_file_is_never_overwritten() {
    let root = Scratch::new();
    let path = root.0.join("existing");
    fs::write(&path, b"KEEP").unwrap();
    assert!(write_bundle_new(&path, &data()).is_err());
    assert_eq!(fs::read(path).unwrap(), b"KEEP");
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
}
#[test]
fn existing_directory_is_not_replaced() {
    let root = Scratch::new();
    let path = root.0.join("directory");
    fs::create_dir(&path).unwrap();
    assert!(write_bundle_new(&path, &data()).is_err());
    assert!(path.is_dir());
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
}
#[test]
fn competing_writers_have_exactly_one_winner() {
    let root = Scratch::new();
    let path = root.0.join("race");
    let barrier = Arc::new(Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let p = path.clone();
            let b = barrier.clone();
            std::thread::spawn(move || {
                let d = data();
                b.wait();
                write_bundle_new(&p, &d)
            })
        })
        .collect();
    let pins: Vec<_> = threads
        .into_iter()
        .filter_map(|t| t.join().unwrap().ok())
        .collect();
    assert_eq!(pins.len(), 1);
    load_bundle(&path, pins[0]).unwrap();
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
}
#[test]
fn corrupt_or_other_generation_rejected() {
    let root = Scratch::new();
    let path = root.0.join("bank");
    let pin = write_bundle_new(&path, &data()).unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes.pop();
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        load_bundle(&path, pin),
        Err(BundleError::Data(Error::Integrity))
    ));
    let other = root.0.join("other");
    write_bundle_new(&other, &import_text(b"other", "Notes").unwrap()).unwrap();
    assert!(matches!(
        load_bundle(&other, pin),
        Err(BundleError::Data(Error::Integrity))
    ));
}
#[test]
fn bounded_read_and_non_file_rejected() {
    let root = Scratch::new();
    let path = root.0.join("text");
    fs::write(&path, b"12345").unwrap();
    assert_eq!(read_regular(&path, 5).unwrap(), b"12345");
    assert_eq!(read_regular(&path, usize::MAX).unwrap(), b"12345");
    assert!(matches!(
        read_regular(&path, 4),
        Err(BundleError::Data(Error::Limit))
    ));
    assert!(matches!(
        read_regular(&root.0, 5),
        Err(BundleError::NotRegular)
    ));
}
#[cfg(unix)]
#[test]
fn symlink_input_and_output_rejected() {
    use std::os::unix::fs::symlink;
    let root = Scratch::new();
    let original = root.0.join("original");
    fs::write(&original, b"KEEP").unwrap();
    let link = root.0.join("link");
    symlink(&original, &link).unwrap();
    assert!(matches!(
        read_regular(&link, 10),
        Err(BundleError::NotRegular)
    ));
    assert!(write_bundle_new(&link, &data()).is_err());
    assert_eq!(fs::read(&original).unwrap(), b"KEEP");
    let dangling = root.0.join("dangling");
    symlink(root.0.join("missing"), &dangling).unwrap();
    assert!(write_bundle_new(&dangling, &data()).is_err());
    assert!(!root.0.join("missing").exists());
}
#[test]
fn cli_fresh_process_roundtrip_and_unknown_title() {
    let root = Scratch::new();
    let input = root.0.join("input.txt");
    let output = root.0.join("bank");
    fs::write(&input, "Łódź — exact text.\n").unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_gel-source"))
        .args([
            "build",
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "Notes",
        ])
        .output()
        .unwrap();
    assert!(build.status.success(), "{:?}", build);
    let stdout = String::from_utf8(build.stdout).unwrap();
    let pin = stdout
        .lines()
        .find_map(|l| l.strip_prefix("BUNDLE_SHA256="))
        .unwrap();
    let read = Command::new(env!("CARGO_BIN_EXE_gel-source"))
        .args(["read", output.to_str().unwrap(), pin, "Notes"])
        .output()
        .unwrap();
    assert!(read.status.success());
    let text = String::from_utf8(read.stdout).unwrap();
    assert!(text.contains("Łódź — exact text."));
    assert!(text.contains("READ=PASS"));
    let miss = Command::new(env!("CARGO_BIN_EXE_gel-source"))
        .args(["read", output.to_str().unwrap(), pin, "Other"])
        .output()
        .unwrap();
    assert!(!miss.status.success());
    assert!(miss.stdout.is_empty());
}
#[test]
fn cli_does_not_emit_terminal_escape_from_text() {
    let root = Scratch::new();
    let input = root.0.join("input");
    let output = root.0.join("output");
    fs::write(&input, b"hello\x1b[2Jworld").unwrap();
    let b = Command::new(env!("CARGO_BIN_EXE_gel-source"))
        .args([
            "build",
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "Notes",
        ])
        .output()
        .unwrap();
    assert!(b.status.success());
    let stdout = String::from_utf8(b.stdout).unwrap();
    let pin = stdout
        .lines()
        .find_map(|l| l.strip_prefix("BUNDLE_SHA256="))
        .unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_gel-source"))
        .args(["read", output.to_str().unwrap(), pin, "Notes"])
        .output()
        .unwrap();
    assert!(r.status.success());
    assert!(!r.stdout.contains(&27));
    assert!(String::from_utf8(r.stdout).unwrap().contains("\\u{1b}"));
}

#[cfg(unix)]
#[test]
fn kernel_permission_denial_keeps_previous_snapshot_and_leaves_no_file() {
    use std::os::unix::fs::PermissionsExt;
    let root = Scratch::new();
    let previous = root.0.join("previous.gelsrc");
    let pin = write_bundle_new(&previous, &data()).unwrap();
    let locked = root.0.join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o500)).unwrap();
    // The kernel itself must refuse; an unprivileged probe establishes that.
    let probe = fs::File::create(locked.join("probe"));
    let result = write_bundle_new(&locked.join("new.gelsrc"), &data());
    let entries = fs::read_dir(&locked).unwrap().count();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        matches!(&probe, Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied),
        "run unprivileged: the kernel did not refuse a write into a 0500 directory"
    );
    assert!(
        matches!(&result, Err(BundleError::Io(e)) if e.kind() == std::io::ErrorKind::PermissionDenied),
        "{result:?}"
    );
    assert_eq!(entries, 0, "no destination or temporary file may appear");
    let reopened = load_bundle(&previous, pin).unwrap();
    assert_eq!(
        reopened
            .quote(Address { node: 1, role: 1 })
            .unwrap()
            .quote(),
        "Zażółć 🦀\r\nIt's exact."
    );
}
