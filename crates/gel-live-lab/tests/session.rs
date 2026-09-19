use gel_live_lab::Lab;
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
                "gel-lab-test-{}-{}",
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
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(input: &str) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_gel-live-lab"))
        .arg("--plain")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    c.wait_with_output().unwrap()
}
#[test]
fn fresh_process_reopens_own_saved_document() {
    let s = Scratch::new();
    let input = s.0.join("my notes.txt");
    let bundle = s.0.join("my bank.gelsrc");
    fs::write(&input, "UNIQUE-TEST-ONLY Zażółć 🦀\r\n").unwrap();
    let first = run(&format!(
        "open {}\nsave {}\nexit\n",
        input.display(),
        bundle.display()
    ));
    assert!(first.status.success());
    let stdout = String::from_utf8(first.stdout).unwrap();
    let pin = stdout
        .lines()
        .map(str::trim)
        .find(|l| {
            l.len() == 64
                && l.bytes().all(|b| b.is_ascii_hexdigit())
                && *l == gel_source::hex(&gel_source::digest(&fs::read(&bundle).unwrap()))
        })
        .unwrap();
    let second = run(&format!("load {pin} {}\nexit\n", bundle.display()));
    assert!(second.status.success());
    let text = String::from_utf8(second.stdout).unwrap();
    assert!(text.contains("REOPEN=PASS"));
    assert!(text.contains("UNIQUE-TEST-ONLY Zażółć 🦀"));
}
#[test]
fn failed_load_preserves_session() {
    let s = Scratch::new();
    let bundle = s.0.join("bank");
    let mut a = Lab::demo().unwrap();
    a.save(&bundle).unwrap();
    let pin = a.last_pin.unwrap();
    let mut raw = fs::read(&bundle).unwrap();
    raw[0] ^= 1;
    fs::write(&bundle, raw).unwrap();
    a.select(3).unwrap();
    let before = a.quote.clone();
    assert!(a.load(&bundle, pin).is_err());
    assert_eq!(a.quote, before);
}
#[test]
fn save_never_replaces_existing_file() {
    let s = Scratch::new();
    let p = s.0.join("keep");
    fs::write(&p, b"KEEP").unwrap();
    assert!(Lab::demo().unwrap().save(&p).is_err());
    assert_eq!(fs::read(p).unwrap(), b"KEEP");
}
#[test]
fn malformed_input_does_not_replace_demo() {
    let s = Scratch::new();
    let p = s.0.join("bad");
    fs::write(&p, [255]).unwrap();
    let mut l = Lab::demo().unwrap();
    let before = l.quote.clone();
    assert!(l.command(&format!("open {}", p.display())).is_err());
    assert_eq!(l.quote, before);
}
#[test]
fn long_command_is_rejected() {
    let out = run(&format!("{}\n", "x".repeat(4097)));
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr)
        .unwrap()
        .contains("4096-byte limit"));
}
#[test]
fn scripted_demo_and_no_terminal_controls() {
    let out = run("read 2\nphase 128\nmask 0\nview 3\ntamper\nexit\n");
    assert!(out.status.success());
    assert!(!out.stdout.contains(&27));
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("TAMPER=REJECTED"));
    assert!(s.contains("Shared score: +0.000000"));
}
#[test]
fn paging_cannot_hide_unreachable_unicode() {
    let s = Scratch::new();
    let p = s.0.join("long");
    let text = "Zażółć🦀 ".repeat(200);
    fs::write(&p, &text).unwrap();
    let mut l = Lab::demo().unwrap();
    l.command(&format!("open {}", p.display())).unwrap();
    assert!(l.pages() > 1);
    for n in 1..=l.pages() {
        l.command(&format!("page {n}")).unwrap();
        assert!(l
            .render(false)
            .unwrap()
            .contains(&format!("Quote page {n}/")));
    }
    assert_eq!(l.quote, text);
}

#[test]
fn find_after_restart_uses_exact_text_across_storage_parts() {
    let s = Scratch::new();
    let input = s.0.join("crossing.txt");
    let bundle = s.0.join("crossing.gelsrc");
    // A short line straddles the 32768-byte storage boundary; it must not
    // disappear because the source writer uses byte-bounded parts.
    let text = format!(
        "{}cat dog\rZażo\u{301}łć gęślą\r\nΟΣ\n",
        "\n".repeat(gel_source::MAX_PASSAGE - 3)
    );
    fs::write(&input, &text).unwrap();
    let mut lab = Lab::demo().unwrap();
    lab.command(&format!("open {}", input.display())).unwrap();
    lab.find("cat dog").unwrap();
    assert_eq!(lab.quote, "cat dog");
    lab.save(&bundle).unwrap();
    let pin = gel_source::hex(&lab.last_pin.unwrap());
    let out = run(&format!(
        "load {pin} {}\nfind cat dog\nfind zażółć gęślą\nfind οσ\nexit\n",
        bundle.display()
    ));
    assert!(out.status.success());
    let output = String::from_utf8(out.stdout).unwrap();
    assert!(output.contains("REOPEN=PASS"));
    assert!(output.contains("cat dog"));
    assert!(output.contains("ΟΣ"));
    assert_eq!(output.matches("FIND=HIT").count(), 3);
    assert!(!output.contains("REFUSED"));
    assert_eq!(fs::read(&input).unwrap(), text.as_bytes());
}

#[test]
fn find_miss_never_displays_previous_quote_as_a_result() {
    let mut l = Lab::demo().unwrap();
    l.find("cryptographic digest").unwrap();
    assert!(!l.quote.is_empty());
    l.find("definitely absent phrase").unwrap();
    assert!(l.quote.is_empty());
    assert!(l.notice.contains("FIND=UNKNOWN"));
    assert!(l.tamper().is_err());
    assert!(l.select_match(1).is_err());
}

#[test]
fn matching_lines_and_cr_boundaries_survive_import_and_reopen() {
    let s = Scratch::new();
    let p = s.0.join("multiple");
    fs::write(&p, "Alpha one\rAlpha two\r\nAlpha three\nnot\rapproved").unwrap();
    let mut l = Lab::demo().unwrap();
    l.command(&format!("open {}", p.display())).unwrap();
    l.find("alpha").unwrap();
    assert!(l.notice.contains("matches=3 shown=3"));
    l.command("match 2").unwrap();
    assert_eq!(l.quote, "Alpha two");
    l.command("match 3").unwrap();
    assert_eq!(l.quote, "Alpha three");
    l.command("read 1").unwrap();
    assert!(l.command("match 1").is_err());
    l.find("not approved").unwrap();
    assert!(l.notice.contains("FIND=UNKNOWN"));
}

#[test]
fn skipped_lines_and_search_limits_are_not_reported_as_absence() {
    let s = Scratch::new();
    let p = s.0.join("long-line");
    fs::write(&p, "x".repeat(gel_source::document::MAX_LINE + 1)).unwrap();
    let mut l = Lab::demo().unwrap();
    l.command(&format!("open {}", p.display())).unwrap();
    l.find("x").unwrap();
    assert!(l.notice.contains("FIND=INCOMPLETE"));
    assert!(l.find(&"q".repeat(513)).is_err());
    assert!(l.find("").is_err());
}

#[test]
fn multi_node_catalog_search_cannot_join_unrelated_documents() {
    let s = Scratch::new();
    let p = s.0.join("multiple-documents");
    let mut b = gel_source::CorpusBuilder::new();
    for (node, text) in [(1, "not "), (2, "approved")] {
        b.push(
            gel_source::Address { node, role: 1 },
            node,
            "Document",
            "Lead",
            text,
        )
        .unwrap();
    }
    let pin = gel_source::write_bundle_new(&p, &b.finish().unwrap()).unwrap();
    let mut l = Lab::demo().unwrap();
    l.load(&p, pin).unwrap();
    assert!(l.find("not approved").is_err());
    assert_eq!(l.quote, "not ");
}
