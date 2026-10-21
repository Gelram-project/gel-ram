//! Identity check for the whole-tree gates of `xtask` (rust-only, license
//! metadata, project contact, docs-refs, disclosure) and for recorder-lint:
//! one walk that keeps the metadata of every entry, per-file work on scoped
//! threads merged in file order, the IPv4 test without a list of octets, and
//! concurrent recorder-lint probes. Every gate is compared with an exact copy
//! of the earlier sequential code (module `reference`): the printed lines, the
//! returned error and, for the walk, the entry list.
//!
//! Inputs: the repository tree (read only), a scratch copy of it with planted
//! findings, seeded generated trees (nesting, skipped `target` and `.git`
//! directories, links, executable and unreadable entries, every finding kind,
//! reviewed asset and evidence paths, Markdown references, invalid UTF-8),
//! and scratch copies of the recorder tool with planted defects.
//!
//! Usage: cargo run --release -p gel-source --example identity_tree_gates -- [DIR]
//! DIR (default: the system temporary directory) receives one fresh scratch
//! directory, removed at the end. The recorder-lint cases need clippy-driver.
//!
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL when any output
//! differs, when a case ran zero times, or when no case ran); exit 0 on PASS,
//! 1 on FAIL. Output grammar: `docs/SPEED.md`.

#![forbid(unsafe_code)]

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

#[allow(dead_code)]
#[path = "../../../xtask/src/disclosure.rs"]
mod disclosure;
#[allow(dead_code)]
#[path = "../../../xtask/src/license_metadata.rs"]
mod license_metadata;
#[allow(dead_code)]
#[path = "../../../xtask/src/recorder_lint.rs"]
mod recorder_lint;
#[allow(dead_code)]
#[path = "../../../xtask/src/tree_scan.rs"]
mod tree_scan;

use std::fs;
use std::path::{Path, PathBuf};

/// Exact copies of the gate code before the shared walk, with each printed
/// line captured instead of printed.
mod reference {
    use super::{disclosure, license_metadata, recorder_lint, tree_scan};
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[derive(Default)]
    pub struct Printed {
        pub stdout: Vec<String>,
        pub stderr: Vec<String>,
    }

    pub fn walk(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if path
                .file_name()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x == "target" || x == ".git")
            {
                continue;
            }
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() {
                out.push(path);
            } else if metadata.is_dir() {
                walk(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }

    pub fn rust_only_at(
        root: &Path,
        reviewed_assets: &[(&str, &str)],
        printed: &mut Printed,
    ) -> Result<(), String> {
        let mut files = Vec::new();
        walk(root, &mut files).map_err(|e| e.to_string())?;
        let mut bad = Vec::new();
        for path in files {
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if metadata.file_type().is_symlink() {
                bad.push(format!("symlink is not allowed: {}", path.display()));
                continue;
            }
            #[cfg(unix)]
            if metadata.permissions().mode() & 0o111 != 0 {
                bad.push(format!(
                    "executable file is not allowed in the release tree: {}",
                    path.display()
                ));
                continue;
            }
            let documentation_image = [
                "docs/images/public-readout.svg",
                "docs/images/q8-r2-results.svg",
            ]
            .iter()
            .any(|p| path == root.join(p));
            let approved_png = reviewed_assets
                .iter()
                .find(|(name, _)| path == root.join(name));
            if let Some((_, expected)) = approved_png {
                if !metadata.is_file() || metadata.len() > 4 * 1024 * 1024 {
                    bad.push(format!(
                        "invalid reviewed asset size/type: {}",
                        path.display()
                    ));
                    continue;
                }
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                if gel_source::hex(&gel_source::digest(&bytes)) != *expected {
                    bad.push(format!(
                        "reviewed asset fingerprint mismatch: {}",
                        path.display()
                    ));
                    continue;
                }
            }
            let collection_evidence = [
                "docs/evidence-collection/MEASURED-SOURCES.sha256",
                "docs/evidence-collection/r1/raw.csv",
                "docs/evidence-collection/r1/summary.csv",
                "docs/evidence-gel-components/run-1.csv",
                "docs/evidence-gel-components/run-2.csv",
                "docs/evidence-gel-components/run-3.csv",
            ]
            .iter()
            .any(|p| path == root.join(p));
            if collection_evidence {
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                if bytes.len() > 1024 * 1024
                    || std::str::from_utf8(&bytes).is_err()
                    || bytes.iter().any(|b| *b == 0 || *b == 27)
                {
                    bad.push(format!(
                        "invalid collection text evidence: {}",
                        path.display()
                    ));
                    continue;
                }
            }
            let allowed = collection_evidence
                || documentation_image
                || approved_png.is_some()
                || path
                    .extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|ext| tree_scan::ALLOWED_EXTENSIONS.contains(&ext))
                || path
                    .file_name()
                    .and_then(|x| x.to_str())
                    .is_some_and(|name| tree_scan::ALLOWED_EXTENSIONLESS.contains(&name));
            if !allowed {
                bad.push(format!(
                    "file type is outside the release allow-list: {}",
                    path.display()
                ));
            }
        }
        bad.sort();
        bad.dedup();
        if bad.is_empty() {
            printed.stdout.push("RUST_ONLY_GATE=PASS".into());
            Ok(())
        } else {
            for message in bad {
                printed.stderr.push(message);
            }
            Err("RUST_ONLY_GATE=FAIL".into())
        }
    }

    pub fn project_contact_privacy(root: &Path, printed: &mut Printed) -> Result<(), String> {
        let mut files = Vec::new();
        walk(root, &mut files).map_err(|e| e.to_string())?;
        for path in files {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if let Ok(text) = std::str::from_utf8(&bytes) {
                if tree_scan::has_nonproject_gmail(text) {
                    return Err(format!("non-project Gmail address in {}", path.display()));
                }
            }
        }
        printed.stdout.push("PROJECT_CONTACT_GATE=PASS".into());
        Ok(())
    }

    pub fn license_metadata_check(root: &Path) -> Result<(), String> {
        let mut files = Vec::new();
        walk(root, &mut files).map_err(|e| e.to_string())?;
        let mut count = 0;
        for file in files
            .into_iter()
            .filter(|p| p.file_name().is_some_and(|n| n == "Cargo.toml"))
        {
            if fs::symlink_metadata(&file)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("symlinked Cargo manifest".into());
            }
            let is_root = file == root.join("Cargo.toml");
            license_metadata::validate(
                &fs::read_to_string(&file).map_err(|e| e.to_string())?,
                is_root,
            )
            .map_err(|e| format!("{}: {e}", file.strip_prefix(root).unwrap().display()))?;
            count += 1;
        }
        if count != 15 {
            return Err(format!(
                "unexpected manifest count {count}; review workspace changes"
            ));
        }
        Ok(())
    }

    pub fn docs_refs(root: &Path, printed: &mut Printed) -> Result<(), String> {
        let mut files = Vec::new();
        walk(root, &mut files).map_err(|e| e.to_string())?;
        files.sort();
        let mut checked = 0usize;
        let mut missing = Vec::new();
        for path in files
            .iter()
            .filter(|path| path.extension().and_then(OsStr::to_str) == Some("md"))
        {
            let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let citing = path.strip_prefix(root).unwrap_or(path.as_path());
            for (line, token) in tree_scan::reference_tokens(&text) {
                checked += 1;
                if !root.join(token).exists() {
                    missing.push(format!(
                        "{}:{line}: missing repository reference: {token}",
                        citing.display()
                    ));
                }
            }
        }
        if checked == 0 {
            return Err(
                "DOCS_REFS_GATE=FAIL: no repository references found in any .md file".into(),
            );
        }
        if missing.is_empty() {
            printed.stdout.push("DOCS_REFS_GATE=PASS".into());
            printed.stdout.push(format!("DOCS_REFS_CHECKED={checked}"));
            Ok(())
        } else {
            for line in &missing {
                printed.stderr.push(line.clone());
            }
            Err("DOCS_REFS_GATE=FAIL".into())
        }
    }

    /// The earlier `disclosure()`: its own walk, then the sequential check.
    pub fn disclosure(root: &Path, printed: &mut Printed) -> Result<(), String> {
        let mut files = Vec::new();
        walk(root, &mut files).map_err(|e| e.to_string())?;
        disclosure_check(root, &files, printed)
    }

    pub fn disclosure_check(
        root: &Path,
        files: &[PathBuf],
        printed: &mut Printed,
    ) -> Result<(), String> {
        let (mut scanned, mut bad) = (0usize, Vec::new());
        for path in files {
            let Ok(bytes) = fs::read(path) else { continue };
            if bytes.contains(&0) {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            scanned += 1;
            let name = path
                .strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string();
            for finding in disclosure::polish_prose(&name, &text) {
                bad.push(format!("{name}: {finding}"));
            }
            for finding in scan(&text) {
                if disclosure::quoted_only_address(&name, &text, &finding) {
                    continue;
                }
                bad.push(format!("{name}: {finding}"));
            }
        }
        if !bad.is_empty() {
            return Err(format!(
                "PUBLIC_DISCLOSURE_GATE=FAIL findings={}\n{}",
                bad.len(),
                bad.join("\n")
            ));
        }
        printed
            .stdout
            .push(format!("PUBLIC_DISCLOSURE_GATE=PASS text_files={scanned}"));
        Ok(())
    }

    const CONTACTS: &[&str] = &["gelram.licensing@gmail.com", "noreply@anthropic.com"];
    const CONTACT_DOMAINS: &[&str] = &[
        "users.noreply.github.com",
        "example.com",
        "example.org",
        "example.net",
    ];
    const USER_DIRS: &[&str] = &["/home/", "/media/", "/Users/", "\\Users\\", "/run/media/"];
    const NEUTRAL_USERS: &[&str] = &[
        "runner",
        "runneradmin",
        "private",
        "user",
        "username",
        "example",
    ];
    const TOKEN_PREFIXES: &[(&str, usize)] = &[
        ("ghp_", 30),
        ("gho_", 30),
        ("ghu_", 30),
        ("ghs_", 30),
        ("ghr_", 30),
        ("github_pat_", 20),
        ("gsk_", 20),
        ("sk-", 20),
        ("xoxb-", 10),
        ("xoxp-", 10),
        ("AKIA", 16),
    ];

    /// The earlier `disclosure::scan`, with a list of octets per token.
    fn scan(text: &str) -> Vec<String> {
        let mut found = Vec::new();
        for dir in USER_DIRS {
            for (at, _) in text.match_indices(dir) {
                let absolute = text[..at]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !(c.is_ascii_alphanumeric() || "._-/".contains(c)));
                if !absolute {
                    continue;
                }
                let name: String = text[at + dir.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || "._-".contains(*c))
                    .collect();
                if !name.is_empty() && !NEUTRAL_USERS.contains(&name.to_lowercase().as_str()) {
                    found.push(format!("user directory path after {dir}"));
                }
            }
        }
        for part in text.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
            let octets: Vec<&str> = part.split('.').collect();
            let ip = octets.len() == 4
                && octets.iter().all(|o| {
                    (1..=3).contains(&o.len()) && o.parse::<u16>().is_ok_and(|v| v <= 255)
                });
            let allowed = part.starts_with("127.")
                || part == "0.0.0.0"
                || part == "255.255.255.255"
                || ["192.0.2.", "198.51.100.", "203.0.113."]
                    .iter()
                    .any(|p| part.starts_with(p));
            if ip && !allowed {
                found.push(format!("network address {part}"));
            }
        }
        for (prefix, least) in TOKEN_PREFIXES {
            for (at, _) in text.match_indices(prefix) {
                let boundary = text[..at]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !c.is_ascii_alphanumeric());
                let key = text[at + prefix.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .count();
                if boundary && key >= *least {
                    found.push(format!("credential-like string with prefix {prefix}"));
                }
            }
        }
        let (begin, end) = (
            ["-----", "BEGIN "].concat(),
            [" PRIVATE", " KEY-----"].concat(),
        );
        if text.contains(&begin) && text.contains(&end) {
            found.push("private key block".into());
        }
        for (at, _) in text.match_indices('@') {
            let local: String = text[..at]
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_alphanumeric() || "._%+-".contains(*c))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let domain: String = text[at + 1..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || ".-".contains(*c))
                .collect();
            let domain = domain.trim_end_matches('.');
            let tld = domain.rsplit('.').next().unwrap_or("");
            if local.is_empty()
                || !domain.contains('.')
                || tld.len() < 2
                || !tld.chars().all(|c| c.is_ascii_alphabetic())
            {
                continue;
            }
            let address = format!("{local}@{domain}").to_lowercase();
            if !CONTACTS.contains(&address.as_str())
                && !CONTACT_DOMAINS.contains(&domain.to_lowercase().as_str())
            {
                found.push(format!("e-mail address at {domain}"));
            }
        }
        found
    }

    /// The earlier sequential `recorder_lint::check` with its fixed work directory.
    pub fn recorder_check(root: &Path, printed: &mut Printed) -> Result<(), String> {
        use recorder_lint::{clippy_driver, io, lint, FORBIDDEN, MAIN, PROBES, SUPPORT, TOOL};
        let driver = clippy_driver(root)?;
        let tools = root.join("tools");
        let work = root.join("target").join("recorder-lint");
        if work.exists() {
            fs::remove_dir_all(&work).map_err(|e| io(&work, e))?;
        }
        fs::create_dir_all(&work).map_err(|e| io(&work, e))?;

        let clean = lint(root, &driver, &tools.join(TOOL), &work)?;
        if !clean.status.success() {
            return Err(format!(
                "RECORDER_LINT=FAIL\n{}",
                String::from_utf8_lossy(&clean.stderr)
            ));
        }

        let path = tools.join(TOOL);
        let source = fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        if source.matches(MAIN).count() != 1 {
            return Err("RECORDER_LINT=FAIL: expected exactly one `fn main() {`".into());
        }
        for (name, statement, expected) in PROBES {
            let dir = work.join(name);
            fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
            let probe_path = dir.join(TOOL);
            let probed = source.replacen(MAIN, &format!("{MAIN}\n    {statement}"), 1);
            fs::write(&probe_path, probed).map_err(|e| io(&probe_path, e))?;
            let support = dir.join(SUPPORT);
            fs::copy(tools.join(SUPPORT), &support).map_err(|e| io(&support, e))?;
            let probe = lint(root, &driver, &probe_path, &dir)?;
            if probe.status.success() || !String::from_utf8_lossy(&probe.stderr).contains(expected)
            {
                return Err(format!(
                    "RECORDER_LINT=FAIL: probe {name} was not rejected by {expected}"
                ));
            }
        }
        printed.stdout.push(format!(
            "RECORDER_LINT=PASS clean_tool=accepted forbidden_lints={} rejected_probes={}",
            FORBIDDEN.len(),
            PROBES.len()
        ));
        Ok(())
    }
}

// `Scale::Small`: generated trees only (no repository reads beyond the asset
// pin list, no clippy-driver). `Scale::Full`: also the repository tree, the
// planted copy and the recorder-lint copies.

/// One case kind; `cases` and `equal` grow as comparisons run.
struct Tallies(Vec<Tally>);

impl Tallies {
    fn new(names: Vec<String>) -> Self {
        Self(names.iter().map(|name| Tally::new(name)).collect())
    }

    /// Counts one comparison of two transcripts; on a difference the first
    /// differing lines go to standard error.
    fn compare(&mut self, name: &str, old: &str, new: &str, condition: bool) {
        if old != new {
            let (a, b) = old
                .lines()
                .zip(new.lines())
                .find(|(a, b)| a != b)
                .unwrap_or(("<end>", "<end>"));
            eprintln!("DIFF name={name} old={a:?} new={b:?}");
        } else if !condition {
            let first = old.lines().next().unwrap_or("");
            eprintln!("CONDITION name={name} reference={first:?}");
        }
        self.check(name, condition && old == new);
    }

    fn check(&mut self, name: &str, equal: bool) {
        match self.0.iter_mut().find(|t| t.name == name) {
            Some(tally) => tally.check(equal),
            // An unknown name is a defect of this program: count it as unequal.
            None => self.0.push(Tally::counted(name, 1, 0)),
        }
    }
}

/// What one gate returned and printed, as one comparable text: the result
/// first (`ok` or `error <message>`), then the printed lines.
fn transcript(stdout: &[String], stderr: &[String], result: &Result<(), String>) -> String {
    let mut text = match result {
        Ok(()) => "ok\n".to_string(),
        Err(error) => format!("error {error}\n"),
    };
    for line in stdout {
        text.push_str("out ");
        text.push_str(line);
        text.push('\n');
    }
    for line in stderr {
        text.push_str("err ");
        text.push_str(line);
        text.push('\n');
    }
    text
}

fn old_transcript(run: impl FnOnce(&mut reference::Printed) -> Result<(), String>) -> String {
    let mut printed = reference::Printed::default();
    let result = run(&mut printed);
    transcript(&printed.stdout, &printed.stderr, &result)
}

fn report_transcript(report: tree_scan::Report) -> String {
    transcript(&report.stdout, &report.stderr, &report.result)
}

/// A new-side gate: the shared walk (its error as text), then the gate.
fn new_transcript(root: &Path, gate: impl FnOnce(&tree_scan::Tree) -> String) -> String {
    match tree_scan::walk(root) {
        Ok(tree) => gate(&tree),
        Err(e) => transcript(&[], &[], &Err(e.to_string())),
    }
}

fn line_result(result: Result<String, String>) -> String {
    match result {
        Ok(line) => transcript(&[line], &[], &Ok(())),
        Err(error) => transcript(&[], &[], &Err(error)),
    }
}

/// Old and new transcripts of every whole-tree gate on `root`, in gate order.
fn gate_pairs(root: &Path, reviewed: &[(&str, &str)]) -> Vec<(&'static str, String, String)> {
    let walk_old = {
        let mut files = Vec::new();
        match reference::walk(root, &mut files) {
            Ok(()) => {
                let mut text = String::new();
                for path in &files {
                    text.push_str("path ");
                    text.push_str(&path.display().to_string());
                    text.push('\n');
                }
                text
            }
            Err(e) => format!("error {e}\n"),
        }
    };
    let walk_new = match tree_scan::walk(root) {
        Ok(tree) => {
            let mut text = String::new();
            for (path, metadata) in tree.paths.iter().zip(&tree.metadata) {
                // The kept metadata must be what a fresh lookup reads.
                let fresh = fs::symlink_metadata(path).ok();
                let same = fresh.is_some_and(|m| {
                    m.file_type() == metadata.file_type()
                        && m.len() == metadata.len()
                        && m.permissions() == metadata.permissions()
                });
                text.push_str(&format!("path {}\n", path.display()));
                if !same {
                    text.push_str("metadata differs\n");
                }
            }
            text
        }
        Err(e) => format!("error {e}\n"),
    };
    vec![
        ("walk", walk_old, walk_new),
        (
            "rust_only",
            old_transcript(|p| reference::rust_only_at(root, reviewed, p)),
            new_transcript(root, |tree| {
                report_transcript(tree_scan::rust_only(root, tree, reviewed))
            }),
        ),
        (
            "contact",
            old_transcript(|p| reference::project_contact_privacy(root, p)),
            new_transcript(root, |tree| {
                report_transcript(tree_scan::project_contact(tree))
            }),
        ),
        (
            "license_metadata",
            old_transcript(|_| reference::license_metadata_check(root)),
            new_transcript(root, |tree| {
                transcript(&[], &[], &license_metadata::check(root, tree))
            }),
        ),
        (
            "docs_refs",
            old_transcript(|p| reference::docs_refs(root, p)),
            new_transcript(root, |tree| {
                report_transcript(tree_scan::docs_refs(root, tree))
            }),
        ),
        (
            "disclosure",
            old_transcript(|p| reference::disclosure(root, p)),
            new_transcript(root, |tree| {
                line_result(disclosure::outcome(root, &tree.paths))
            }),
        ),
    ]
}

/// Whether a transcript (or a walk listing) records an error.
fn failed(text: &str) -> bool {
    text.starts_with("error ")
}

/// Compares every gate on `root`; the case name is `<prefix><gate>` plus
/// `_pass` / `_fail` (from the reference side) when `split` is set.
fn compare_tree(
    tallies: &mut Tallies,
    root: &Path,
    reviewed: &[(&str, &str)],
    prefix: &str,
    split: bool,
) {
    for (gate, old, new) in gate_pairs(root, reviewed) {
        let name = if split {
            let outcome = if failed(&old) { "fail" } else { "pass" };
            format!("{prefix}{gate}_{outcome}")
        } else {
            format!("{prefix}{gate}")
        };
        tallies.compare(&name, &old, &new, true);
    }
}

/// SplitMix64.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }
}

/// Text snippets; every finding is assembled at run time, so this file holds none.
fn snippets() -> Vec<String> {
    let polish = ["za", "\u{17c}\u{f3}\u{142}\u{107}"].concat();
    vec![
        "plain text line".to_string(),
        "fn main() {}".to_string(),
        ["/", "home", "/", "jane", "doe", "/notes"].concat(),
        ["/", "home", "/", "runner", "/work"].concat(),
        "see ../media/gifs/a.gif".to_string(),
        ["host ", &["10", "20", "30", "40"].join("."), " up"].concat(),
        ["docs ", &["203", "0", "113", "9"].join(".")].concat(),
        "1.2.3.4.5 and v0.5.2 and 3.184029".to_string(),
        ["key ", "gh", "p_", &"a".repeat(36)].concat(),
        ["-----", "BEGIN", " PRIVATE", " KEY-----"].concat(),
        ["mail ", "someone", "@", "mail", ".test"].concat(),
        ["mail ", "fixture", "@", "gmail", ".com"].concat(),
        tree_scan::PROJECT_EMAIL.to_string(),
        ["a ", &polish, " b"].concat(),
        ["run `find ", &polish, "`"].concat(),
        "see `LICENSE` and `docs/a.md` here".to_string(),
        "see `src/lib.rs` and `missing/none.md`".to_string(),
        "```text\n`docs/inside.md`\n```".to_string(),
        "`` unmatched ` one.md `".to_string(),
        "`NOTICE` `Cargo.lock` `notes.txt`".to_string(),
    ]
}

const DIR_NAMES: &[&str] = &[
    "docs", "src", "media", "crates", "target", ".git", "a", "b c", "images",
];
const FILE_NAMES: &[&str] = &[
    "README.md",
    "a.md",
    "notes.txt",
    "lib.rs",
    "x.csv",
    "data.json",
    "image.svg",
    "page.html",
    "LICENSE",
    "NOTICE",
    "Cargo.lock",
    "run",
    "z.gel",
    "c.cff",
];

/// Fixed paths the gates treat specially.
const SPECIAL_TEXT: &[&str] = &[
    "docs/images/public-readout.svg",
    "docs/evidence-collection/MEASURED-SOURCES.sha256",
    "docs/evidence-collection/r1/raw.csv",
    "docs/evidence-gel-components/run-2.csv",
];

fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

fn content(rng: &mut Rng, snippets: &[String]) -> Vec<u8> {
    match rng.below(10) {
        0 => vec![0, 1, 2, 3],
        1 => vec![0xff, 0xfe, b'a'],
        2 => b"\x1b[2J escape".to_vec(),
        _ => {
            let count = 1 + rng.below(3);
            (0..count)
                .map(|_| snippets[rng.below(snippets.len())].as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes()
        }
    }
}

fn fill(
    dir: &Path,
    depth: usize,
    rng: &mut Rng,
    snippets: &[String],
    locked: &mut Vec<(PathBuf, u32)>,
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    for i in 0..rng.below(6) {
        let name = FILE_NAMES[rng.below(FILE_NAMES.len())];
        let name = if rng.chance(2) {
            name.to_string()
        } else {
            format!("{i}-{name}")
        };
        let path = dir.join(name);
        fs::write(&path, content(rng, snippets))?;
        unix_extras(&path, rng, locked)?;
    }
    if depth < 3 {
        for i in 0..rng.below(4) {
            let name = DIR_NAMES[rng.below(DIR_NAMES.len())];
            let sub = if rng.chance(2) {
                dir.join(name)
            } else {
                dir.join(format!("{name}{i}"))
            };
            fill(&sub, depth + 1, rng, snippets, locked)?;
        }
    }
    Ok(())
}

/// Links, executable bits and unreadable entries next to `path` (Unix only).
/// Modes that would block later writes are recorded in `locked` and applied
/// once the tree is complete. A link whose name already exists is skipped.
#[cfg(unix)]
fn unix_extras(
    path: &Path,
    rng: &mut Rng,
    locked: &mut Vec<(PathBuf, u32)>,
) -> std::io::Result<()> {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let dir = path.parent().unwrap_or(path);
    let name = path.file_name().unwrap_or_default();
    match rng.below(24) {
        0 => {
            let _ = symlink(name, dir.join(format!("link-{}", rng.below(99))));
        }
        1 => {
            let _ = symlink(".", dir.join(format!("linkdir{}.md", rng.below(99))));
        }
        2 => {
            let _ = symlink(
                "absent.rs",
                dir.join(format!("dangling{}.rs", rng.below(99))),
            );
        }
        3 | 4 => fs::set_permissions(path, fs::Permissions::from_mode(0o755))?,
        5 => locked.push((path.to_path_buf(), 0o000)),
        // Readable but not searchable: listing works, lookups fail.
        6 if rng.chance(3) => locked.push((dir.to_path_buf(), 0o600)),
        _ => {}
    }
    Ok(())
}

#[cfg(not(unix))]
fn unix_extras(_: &Path, _: &mut Rng, _: &mut Vec<(PathBuf, u32)>) -> std::io::Result<()> {
    Ok(())
}

/// Applies the recorded modes, deepest entries first, so no entry sits in a
/// directory that is already locked.
fn lock(locked: &mut [(PathBuf, u32)]) -> std::io::Result<()> {
    locked.sort_by_key(|(path, _)| std::cmp::Reverse(path.components().count()));
    #[cfg(unix)]
    for (path, mode) in locked.iter() {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(*mode))?;
    }
    Ok(())
}

/// The 15 manifests the license metadata gate accepts.
fn manifests(root: &Path) -> std::io::Result<()> {
    write(
        &root.join("Cargo.toml"),
        b"[workspace.package]\nlicense-file = \"LICENSE\"\n",
    )?;
    for i in 0..14 {
        write(
            &root.join(format!("crates/m{i}/Cargo.toml")),
            b"[package]\nlicense-file.workspace = true\n",
        )?;
    }
    Ok(())
}

/// A clean tree: allowed file types, existing references, no findings.
fn clean(root: &Path, rng: &mut Rng) -> std::io::Result<()> {
    write(&root.join("LICENSE"), b"license text\n")?;
    write(&root.join("src/lib.rs"), b"pub fn f() {}\n")?;
    write(
        &root.join("docs/a.md"),
        b"# Notes\n\nsee `LICENSE` and `src/lib.rs`\n",
    )?;
    for i in 0..rng.below(8) {
        write(
            &root.join(format!("docs/n{i}.md")),
            b"see `docs/a.md`\n\n```\n`docs/absent.md`\n```\n",
        )?;
        write(&root.join(format!("src/m{i}.rs")), b"fn g() {}\n")?;
    }
    Ok(())
}

/// One generated tree. Tree 0 holds the accepted manifests, trees 1, 11, 21,
/// ... are clean, the others are random.
fn generate(
    root: &Path,
    index: u64,
    snippets: &[String],
    assets: &[(String, Vec<u8>)],
    locked: &mut Vec<(PathBuf, u32)>,
) -> std::io::Result<()> {
    let mut rng = Rng(0x4745_4c5f_5452_4545 ^ index.wrapping_mul(0x9e37_79b9));
    fs::create_dir_all(root)?;
    match index {
        0 => return manifests(root),
        i if i % 10 == 1 => return clean(root, &mut rng),
        _ => {}
    }
    fill(root, 0, &mut rng, snippets, locked)?;
    if rng.chance(3) {
        manifests(root)?;
    }
    if rng.chance(2) && !assets.is_empty() {
        let (name, bytes) = &assets[rng.below(assets.len())];
        let body: Vec<u8> = if rng.chance(2) {
            bytes.clone()
        } else {
            b"not the reviewed bytes".to_vec()
        };
        write(&root.join(name), &body)?;
    }
    if rng.chance(2) {
        let name = SPECIAL_TEXT[rng.below(SPECIAL_TEXT.len())];
        let body: &[u8] = match rng.below(3) {
            0 => b"rep,n\n0,16384\n",
            1 => b"\x1b[2J",
            _ => b"binary\0",
        };
        write(&root.join(name), body)?;
    }
    if rng.chance(3) {
        let address = ["1", "15", "23", "6"].join(".");
        let body = format!("# header\n1\ten\tWho won?\tX\tx\tresults {address} X\n");
        write(
            &root.join("docs/answer-or-abstain-v4/with-answer-questions.txt"),
            body.as_bytes(),
        )?;
    }
    if rng.chance(4) {
        write(&root.join("docs/broken.md"), &[b'`', 0xff, b'`'])?;
    }
    lock(locked)
}

/// Makes every locked entry removable again (shallowest first), then removes the tree.
fn remove(root: &Path, locked: &[(PathBuf, u32)]) {
    #[cfg(unix)]
    for (path, _) in locked.iter().rev() {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
    }
    #[cfg(not(unix))]
    let _ = locked;
    let _ = fs::remove_dir_all(root);
}

/// The reviewed asset pins, read from the `REVIEWED_ASSETS` table in `xtask/src/main.rs`.
fn reviewed_assets(repo: &Path) -> Result<Vec<(String, String)>, String> {
    let main = fs::read_to_string(repo.join("xtask/src/main.rs")).map_err(|e| e.to_string())?;
    let marker = "const REVIEWED_ASSETS: &[(&str, &str)] = &[\n";
    let start = main.find(marker).ok_or("missing asset table")? + marker.len();
    let end = main[start..]
        .find("\n];")
        .ok_or("unterminated asset table")?
        + start;
    let literals: Vec<&str> = main[start..end].split('"').skip(1).step_by(2).collect();
    let pairs: Vec<(String, String)> = literals
        .chunks(2)
        .filter_map(|pair| match pair {
            [name, pin] => Some((name.to_string(), pin.to_string())),
            _ => None,
        })
        .collect();
    let valid = !pairs.is_empty()
        && literals.len() == pairs.len() * 2
        && pairs
            .iter()
            .all(|(_, pin)| pin.len() == 64 && pin.bytes().all(|b| b.is_ascii_hexdigit()));
    if valid {
        Ok(pairs)
    } else {
        Err("asset table does not parse as (path, pin) pairs".into())
    }
}

/// `ordered` against a sequential map, with later items finishing first.
fn compare_ordered(tallies: &mut Tallies) {
    for count in [0usize, 1, 2, 3, 7, 8, 9, 17, 64, 1000] {
        let work = |i: usize| {
            let mut x = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            for _ in 0..(count - i) * 50 {
                x = x.rotate_left(7) ^ 0x5851_f42d;
            }
            (i, x)
        };
        let sequential: Vec<(usize, u64)> = (0..count).map(work).collect();
        tallies.check(
            "ordered_index_order",
            tree_scan::ordered(count, work) == sequential,
        );
    }
}

/// Copies every walked entry of the repository into `copy` (regular files only).
fn copy_tree(repo: &Path, copy: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    reference::walk(repo, &mut files).map_err(|e| e.to_string())?;
    for path in files {
        let relative = path.strip_prefix(repo).map_err(|e| e.to_string())?;
        let target = copy.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(&path, &target).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

fn append(path: &Path, text: &str) -> Result<(), String> {
    let mut body = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    body.push_str(text);
    fs::write(path, body).map_err(|e| format!("{}: {e}", path.display()))
}

/// Findings planted in a copy of the repository; each gate must fail on both
/// sides with the same text.
fn planted(tallies: &mut Tallies, repo: &Path, scratch: &Path, reviewed: &[(&str, &str)]) {
    let copy = scratch.join("planted");
    let planted = copy_tree(repo, &copy).and_then(|()| {
        let user_dir = ["/", "home", "/", "jane", "doe", "/x"].concat();
        let gmail = ["fixture", "@", "gmail", ".com"].concat();
        append(
            &copy.join("README.md"),
            &format!("\nPlanted {user_dir} and `docs/planted-missing.md`.\n"),
        )?;
        append(
            &copy.join("xtask/src/tree_scan.rs"),
            &format!("\n// planted {gmail}\n"),
        )?;
        fs::write(copy.join("docs/planted.bin"), b"planted").map_err(|e| e.to_string())?;
        fs::create_dir_all(copy.join("planted")).map_err(|e| e.to_string())?;
        fs::write(
            copy.join("planted/Cargo.toml"),
            b"[package]\nlicense-file.workspace = true\n",
        )
        .map_err(|e| e.to_string())
    });
    if planted.is_err() {
        tallies.check("planted_setup", false);
    }
    for (gate, old, new) in gate_pairs(&copy, reviewed) {
        if gate == "walk" {
            continue;
        }
        // Activity marker: equal only when both sides flag the plant.
        tallies.compare(&format!("planted_{gate}"), &old, &new, failed(&old));
    }
    let _ = fs::remove_dir_all(&copy);
}

/// Recorder-lint on copies of the recorder tool, earlier sequential gate
/// against the concurrent one.
fn recorder(tallies: &mut Tallies, repo: &Path, scratch: &Path) {
    let sources = [
        "tools/record_evidence.rs",
        "tools/record_support.rs",
        "tools/recorder-lint/clippy.toml",
        "rust-toolchain.toml",
    ];
    let config =
        fs::read_to_string(repo.join("tools/recorder-lint/clippy.toml")).unwrap_or_default();
    let variants: [(&str, &str, String); 5] = [
        ("clean", "", String::new()),
        (
            "unwrap",
            "tools/record_evidence.rs",
            "\nfn planted() -> u8 {\n    \"7\".parse().unwrap()\n}\n".to_string(),
        ),
        (
            "main_twice",
            "tools/record_evidence.rs",
            "\n// fn main() {\n".to_string(),
        ),
        (
            "no_spawn",
            "tools/recorder-lint/clippy.toml",
            config.replace("    \"std::thread::spawn\",\n", ""),
        ),
        (
            "empty_config",
            "tools/recorder-lint/clippy.toml",
            String::new(),
        ),
    ];
    for (variant, file, text) in variants {
        let root = scratch.join(format!("recorder-{variant}"));
        let mut ready = sources.iter().all(|name| {
            let target = root.join(name);
            target
                .parent()
                .is_some_and(|parent| fs::create_dir_all(parent).is_ok())
                && fs::copy(repo.join(name), &target).is_ok()
        });
        if variant == "unwrap" || variant == "main_twice" {
            ready &= append(&root.join(file), &text).is_ok();
        } else if !file.is_empty() {
            ready &= fs::write(root.join(file), &text).is_ok();
        }
        if !ready {
            tallies.check("recorder_setup", false);
            continue;
        }
        let old = old_transcript(|p| reference::recorder_check(&root, p));
        let new = line_result(recorder_lint::outcome_in(&root, &root.join("work-new")));
        let outcome = if failed(&old) { "fail" } else { "pass" };
        let first = old.lines().next().unwrap_or("");
        eprintln!("RECORDER variant={variant} reference={first:?}");
        tallies.compare(&format!("recorder_{outcome}"), &old, &new, true);
        if variant == "empty_config" {
            // Several probes fail here; the first in probe order is reported.
            tallies.compare(
                "recorder_first_probe",
                &old,
                &new,
                old.contains("probe assert was not rejected"),
            );
        }
        let _ = fs::remove_dir_all(&root);
    }
}

/// The work directory follows `CARGO_TARGET_DIR` and is unchanged without it;
/// a relative `CARGO_TARGET_DIR` is taken against the caller's directory.
fn compare_work_dir(tallies: &mut Tallies) {
    let root = Path::new("repo");
    let cwd = Path::new("caller");
    let old = root.join("target").join("recorder-lint");
    tallies.check(
        "recorder_work_dir",
        recorder_lint::work_dir(root, None, cwd) == old,
    );
    tallies.check(
        "recorder_work_dir",
        recorder_lint::work_dir(root, Some("".into()), cwd) == old,
    );
    tallies.check(
        "recorder_work_dir",
        recorder_lint::work_dir(root, Some("build".into()), cwd)
            == Path::new("caller/build/recorder-lint"),
    );
    tallies.check(
        "recorder_work_dir",
        recorder_lint::work_dir(root, Some("/abs/build".into()), cwd)
            == Path::new("/abs/build/recorder-lint"),
    );
}

/// Runs every comparison in a fresh scratch directory under the system
/// temporary directory, removed afterwards.
pub fn run(scale: Scale) -> Vec<Tally> {
    run_under(scale, &std::env::temp_dir())
}

/// Runs every comparison in a fresh scratch directory under `parent`, removed
/// afterwards.
pub fn run_under(scale: Scale, parent: &Path) -> Vec<Tally> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let scratch = parent.join(format!(
        "identity-tree-gates-{}-{nonce}",
        std::process::id()
    ));
    let tallies = run_in(scale, &scratch);
    let _ = fs::remove_dir_all(&scratch);
    tallies
}

/// Runs every comparison with `scratch` as the scratch directory.
pub fn run_in(scale: Scale, scratch: &Path) -> Vec<Tally> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or(Path::new("."));
    let mut names = vec![
        "ordered_index_order".to_string(),
        "recorder_work_dir".to_string(),
    ];
    for gate in [
        "walk",
        "rust_only",
        "contact",
        "license_metadata",
        "docs_refs",
        "disclosure",
    ] {
        names.push(format!("{gate}_pass"));
        names.push(format!("{gate}_fail"));
    }
    if scale == Scale::Full {
        for name in [
            "real_walk",
            "real_rust_only",
            "real_contact",
            "real_license_metadata",
            "real_docs_refs",
            "real_disclosure",
            "planted_rust_only",
            "planted_contact",
            "planted_license_metadata",
            "planted_docs_refs",
            "planted_disclosure",
            "recorder_pass",
            "recorder_fail",
            "recorder_first_probe",
        ] {
            names.push(name.to_string());
        }
    }
    let mut tallies = Tallies::new(names);
    let reviewed_owned = match reviewed_assets(repo) {
        Ok(pairs) => pairs,
        Err(_) => {
            tallies.check("asset_table", false);
            Vec::new()
        }
    };
    let reviewed: Vec<(&str, &str)> = reviewed_owned
        .iter()
        .map(|(name, pin)| (name.as_str(), pin.as_str()))
        .collect();
    if fs::create_dir_all(scratch).is_err() {
        tallies.check("scratch", false);
        return tallies.0;
    }

    compare_ordered(&mut tallies);
    compare_work_dir(&mut tallies);

    // A root that does not exist: every walk fails the same way.
    compare_tree(&mut tallies, &scratch.join("absent"), &reviewed, "", true);

    let assets: Vec<(String, Vec<u8>)> = if scale == Scale::Full {
        reviewed_owned
            .iter()
            .filter_map(|(name, _)| fs::read(repo.join(name)).ok().map(|b| (name.clone(), b)))
            .collect()
    } else {
        reviewed_owned
            .iter()
            .take(3)
            .map(|(name, _)| (name.clone(), b"small-scale stand-in".to_vec()))
            .collect()
    };
    let snippets = snippets();
    let trees = if scale == Scale::Full { 240 } else { 24 };
    for index in 0..trees {
        let root = scratch.join(format!("tree-{index}"));
        let mut locked = Vec::new();
        if generate(&root, index, &snippets, &assets, &mut locked).is_err() {
            tallies.check("generate", false);
        } else {
            compare_tree(&mut tallies, &root, &reviewed, "", true);
        }
        remove(&root, &locked);
    }

    if scale == Scale::Full {
        compare_tree(&mut tallies, repo, &reviewed, "real_", false);
        planted(&mut tallies, repo, scratch, &reviewed);
        if recorder_lint::clippy_driver(repo).is_ok() {
            recorder(&mut tallies, repo, scratch);
        }
    }
    tallies.0
}

fn main() -> std::process::ExitCode {
    match std::env::args_os().nth(1).map(PathBuf::from) {
        None => identity::report(&run(Scale::Full)),
        Some(parent) => identity::report(&run_under(Scale::Full, &parent)),
    }
}
