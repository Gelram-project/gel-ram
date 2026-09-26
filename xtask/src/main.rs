#![forbid(unsafe_code)]
mod license_metadata;
#[cfg(test)]
mod publication_status_tests;
mod reproduce;
mod source_bundle;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const ALLOWED_EXTENSIONS: &[&str] = &["rs", "md", "toml", "yml", "txt", "gel", "cff"];
// Exact reviewed media and source archive only; no general binary exception.
// Pins detect changed bytes; they do not prove decoding safety or semantic truth.
const REVIEWED_ASSETS: &[(&str, &str)] = &[
    (
        "media/GEL-EVIDENCE-LAB-EN.mp4",
        "a7b4e85d5db1274c61a49d8814908321130dad6c3eec9324de0500071d9c9835",
    ),
    (
        "media/evidence-lab/01-source-17s.png",
        "9c73954fe6b732f56140d7d541e2ba4e04768ced5f3f4ea90036de93b2e240c0",
    ),
    (
        "media/evidence-lab/02-reopened-56s.png",
        "3d385ca2c4862ac873c089be1502770550ea810d5af51d16a15656335e08a82a",
    ),
    (
        "research/ocean-scale-r3.tar.gz",
        "b712a6c6c4afb241e02d560d673eafb6bfce78049b498a8478f785508b8dcf48",
    ),
    (
        "research/ocean-scale-r2.tar.gz",
        "f3f5bdcc7a9177b76f14d0a0acc90521bd6cba9893587807a1bbd2d8352b8fde",
    ),
    (
        "media/GEL-RAM-CONTINUOUS-CHAT-EN-60s.mp4",
        "3b568bebdef34d34931970ce1f86ae0c521dba9e9ef6f7d050c5363357b63064",
    ),
    (
        "media/GEL-RAM-HARDWARE-EN-CENTERED-90s.mp4",
        "b2d6cccc2beca58b73705a1bb6bcf156c4d54a5304fb8d8a770ba628c7cf63cf",
    ),
    (
        "media/continuous-chat-preview.png",
        "7d71a0e0b21ef0772a14f7ba9aa1f163d4d179122307392ff0de404f6c280a65",
    ),
    (
        "media/screenshots/01-hardware-00m10s.png",
        "27bde081109e4fd347fd34d5248a103503a3de8258e2f520af4a5708c6faaeae",
    ),
    (
        "media/screenshots/02-chat-00m25s.png",
        "63dc404334d6332217924af8fb591cbe2b39b911d09ca7397d98de70f0300ebe",
    ),
    (
        "media/screenshots/03-followup-00m45s.png",
        "5068151833b636ef46499fe4cee99c6a84b9d5c1e52b51470799a263e0c67817",
    ),
    (
        "media/screenshots/04-unknown-01m10s.png",
        "d1b797a24fd2f717e52c5d75f015648acbcf87f2821dd531ec74930fb708b77c",
    ),
    (
        "media/screenshots/05-summaries-00m37s.png",
        "7d71a0e0b21ef0772a14f7ba9aa1f163d4d179122307392ff0de404f6c280a65",
    ),
    (
        "media/screenshots/06-unknown-00m49s.png",
        "caef327d44a01ce25dfcae5c875f97b9ea14ffaeafadd8e1eccd046a69c4510d",
    ),
    (
        "media/terminal-preview.png",
        "d1b797a24fd2f717e52c5d75f015648acbcf87f2821dd531ec74930fb708b77c",
    ),
    (
        "docs/images/q8-four-views-en.png",
        "061a5e99f1fa5ba8380a5b8ed6c60d672904b2b8069382971a60c0192a95c104",
    ),
    (
        "docs/images/evidence-limits-en.png",
        "0e675a58bb6955a16bde7b413e82094faabda951d60dff078735e30e052c58c2",
    ),
    (
        "docs/images/q8-four-views-pl.png",
        "22928431fc598e080a755e25a7acb75b29feacf011e554a5d2d3c8c28ee02a7f",
    ),
    (
        "docs/images/evidence-limits-pl.png",
        "040cf00b6577d7bb7f399e8f7aa112db2dd678119cab919629d3ec0531cc7d53",
    ),
];
const ALLOWED_EXTENSIONLESS: &[&str] = &[
    "Cargo.lock",
    "LICENSE",
    "NOTICE",
    ".gitignore",
    ".gitattributes",
];

/// A backtick-quoted token with one of these extensions is a repository path citation.
const REFERENCE_EXTENSIONS: &[&str] = &["md", "toml", "txt", "yml", "rs", "lock", "cff"];
/// A backtick-quoted token starting with one of these prefixes is a repository path citation.
const REFERENCE_PREFIXES: &[&str] = &["docs/", ".github/", "crates/", "xtask/"];

/// The ticked CLA acknowledgement line from `.github/PULL_REQUEST_TEMPLATE.md`.
const CLA_ACK_TICKED: &[&str] = &[
    "[x] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
    "[X] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
];

const USAGE: &str =
    "verify|report|source-audit|source-bundle|rust-only|licensing|ci-policy|docs-refs|cla-ack|fmt|clippy|test|bench|physics";
const CHECKOUT_SHA: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const PROJECT_EMAIL: &str = "gelram.licensing@gmail.com";

// Exact active-license text pin; not evidence of legal approval or authorship.
fn canonical_license(bytes: &[u8]) -> bool {
    gel_source::hex(&gel_source::digest(bytes))
        == "c0b560ffc53ad735c4a6236356ff0cd47d92cf671181e799917975b071e8cd40"
}

// Narrow accidental-contact check, NOT a general secret or email scanner.
// A bare domain suffix in source code is not an email address.
fn has_nonproject_gmail(text: &str) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || ".+-_@".contains(c)))
        .any(|token| {
            let token = token.trim_matches('.');
            let Some((local, domain)) = token.rsplit_once('@') else {
                return false;
            };
            !local.is_empty()
                && domain.eq_ignore_ascii_case("gmail.com")
                && !token.eq_ignore_ascii_case(PROJECT_EMAIL)
        })
}

fn project_contact_privacy(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|e| e.to_string())?;
    for path in files {
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Ok(text) = std::str::from_utf8(&bytes) {
            if has_nonproject_gmail(text) {
                // Do not echo a potentially private address into public CI logs.
                return Err(format!("non-project Gmail address in {}", path.display()));
            }
        }
    }
    println!("PROJECT_CONTACT_GATE=PASS");
    Ok(())
}
const FMT_CHECK_ARGS: &[&str] = &["fmt", "--all", "--", "--check"];
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
];
const TEST_ARGS: &[&str] = &[
    "test",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
];

fn walk(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
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

fn workspace_root() -> Result<&'static Path, String> {
    // Resolve the source checkout embedded by this build, not the caller's cwd.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "missing workspace root".into())
}

fn rust_only_at(root: &Path) -> Result<(), String> {
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
        // Exactly two reviewed, static vector illustrations; not a general
        // SVG or executable-asset exception. Implementation remains Rust-only.
        let documentation_image = [
            "docs/images/public-readout.svg",
            "docs/images/q8-r2-results.svg",
        ]
        .iter()
        .any(|p| path == root.join(p));
        let approved_png = REVIEWED_ASSETS
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
        // Only these reviewed text evidence paths, not a general CSV/sha256 exception.
        // Their exact bytes are pinned by the enclosing source manifest.
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
                .is_some_and(|ext| ALLOWED_EXTENSIONS.contains(&ext))
            || path
                .file_name()
                .and_then(|x| x.to_str())
                .is_some_and(|name| ALLOWED_EXTENSIONLESS.contains(&name));
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
        println!("RUST_ONLY_GATE=PASS");
        Ok(())
    } else {
        for message in bad {
            eprintln!("{message}");
        }
        Err("RUST_ONLY_GATE=FAIL".into())
    }
}

fn rust_only() -> Result<(), String> {
    rust_only_at(workspace_root()?)
}

fn require(text: &str, needle: &str, file: &str) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(format!("{file} is missing required text: {needle}"))
    }
}

// Technical tests must run before publication approval; PASS is not permission.
fn publication_status(text: &str) -> Result<bool, String> {
    let flag = |key: &str| -> Result<bool, String> {
        let values: Vec<_> = text
            .lines()
            .filter_map(|line| {
                let (name, value) = line.trim().split_once('=')?;
                (name == key).then_some(value)
            })
            .collect();
        match values.as_slice() {
            ["YES"] => Ok(true),
            ["NO"] => Ok(false),
            _ => Err(format!("missing, duplicated or invalid {key}")),
        }
    };
    let review = flag("REVIEW_PUBLICATION_APPROVED")?;
    let publication = flag("PUBLICATION_APPROVED")?;
    if publication && !review {
        return Err("publication cannot precede scope review".into());
    }
    Ok(publication)
}

fn licensing() -> Result<(), String> {
    let root = workspace_root()?;
    let read = |name: &str| fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"));
    let mode = read("LICENSE-MODE.txt")?;
    let license = read("LICENSE")?;
    let notice = read("NOTICE")?;
    let commercial = read("COMMERCIAL-LICENSE.md")?;
    let cla = read("CLA.md")?;
    licensing_scope(&read("LICENSING.md")?, &notice)?;
    let fixture_license = fs::read(root.join("crates/gel-source/fixtures/rust-book/LICENSE.txt"))
        .map_err(|e| e.to_string())?;
    if gel_source::hex(&gel_source::digest(&fixture_license))
        != "0621878e61f0d0fda054bcbe02df75192c28bde1ecc8289cbd86aeba2dd72720"
    {
        return Err("reviewed third-party fixture license is missing or changed".into());
    }
    license_metadata::check(root)?;
    if !canonical_license(license.as_bytes()) {
        return Err("LICENSE differs from the pinned active NCRL bytes".into());
    }
    match mode.trim() {
        "GEL-RAM-NCRL-1.0 + Commercial + CLA-2.0" => {
            require(
                &license,
                "GEL RAM Noncommercial Reciprocal License 1.0",
                "LICENSE",
            )?;
            require(
                &notice,
                "Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)",
                "NOTICE",
            )?;
        }
        other => return Err(format!("unsupported LICENSE-MODE.txt value: {other}")),
    }
    require(
        &commercial,
        "not itself a commercial license grant",
        "COMMERCIAL-LICENSE.md",
    )?;
    for text in [
        "Contributor License Agreement 2.0",
        "right to license or relicense",
        "Patent license",
        "legal authority",
        "third-party material",
        "AI system",
        "completed Agreement is on file",
        "Pin the exact accepted agreement text",
        "public pseudonym",
        "CLA-PRIVACY.md",
    ] {
        require(&cla, text, "CLA.md")?;
    }
    require(
        &read("CLA-PRIVACY.md")?,
        "RR is a public pseudonym",
        "CLA-PRIVACY.md",
    )?;
    let publication = publication_status(&read("CANDIDATE-STATUS.md")?)?;
    println!(
        "PUBLICATION_APPROVED={}",
        if publication { "YES" } else { "NO" }
    );
    require(
        &read("CANDIDATE-STATUS.md")?,
        "LEGAL_APPROVED=NO",
        "CANDIDATE-STATUS.md",
    )?;
    project_contact_privacy(root)?;
    println!("LICENSING_GATE=PASS");
    println!("LICENSE_MODE={}", mode.trim());
    println!("LICENSE_CHECK_SCOPE=ACTIVE_LICENSE_CONSISTENCY_NOT_LEGAL_CERTIFICATION");
    Ok(())
}

fn licensing_scope(guide: &str, notice: &str) -> Result<(), String> {
    require(
        guide,
        "Identified third-party material remains under its own license terms",
        "LICENSING.md",
    )?;
    require(guide, "remains MIT-licensed", "LICENSING.md")?;
    require(notice, "The Rust Book excerpt", "NOTICE")?;
    require(notice, "distributed under MIT", "NOTICE")?;
    let normalized = guide
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    for obsolete in [
        "everything in this distribution is",
        "no other public license is granted for this tree",
        "this tree is offered under polyform noncommercial 1.0.0 only",
    ] {
        if normalized.contains(obsolete) {
            return Err(
                "overbroad whole-tree licensing claim conflicts with third-party material".into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod licensing_scope_tests {
    const GUIDE: &str = include_str!("../../LICENSING.md");
    const NOTICE: &str = include_str!("../../NOTICE");
    #[test]
    fn root_third_party_scope_and_notices() {
        super::licensing_scope(GUIDE, NOTICE).unwrap();
        assert!(
            super::licensing_scope(&GUIDE.replace("remains MIT-licensed", "omitted"), NOTICE)
                .is_err()
        );
        assert!(
            super::licensing_scope(GUIDE, &NOTICE.replace("distributed under MIT", "omitted"))
                .is_err()
        );
    }
    #[test]
    fn appended_overbroad_claim_is_rejected() {
        for claim in [
            "Everything in this distribution is ours.",
            "No other public license is granted for this tree.",
            "This tree is offered under PolyForm Noncommercial 1.0.0 only.",
        ] {
            assert!(super::licensing_scope(&format!("{GUIDE}\n{claim}"), NOTICE).is_err());
        }
    }
}

fn ci_policy() -> Result<(), String> {
    let root = workspace_root()?;
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .map_err(|e| format!(".github/workflows/ci.yml: {e}"))?;
    let cla = fs::read_to_string(root.join(".github/workflows/cla.yml"))
        .map_err(|e| format!(".github/workflows/cla.yml: {e}"))?;
    require(&ci, CHECKOUT_SHA, ".github/workflows/ci.yml")?;
    require(
        &ci,
        "persist-credentials: false",
        ".github/workflows/ci.yml",
    )?;
    require(&ci, "contents: read", ".github/workflows/ci.yml")?;
    require(&cla, "pull_request_target:", ".github/workflows/cla.yml")?;
    require(&cla, "permissions: {}", ".github/workflows/cla.yml")?;
    require(
        &cla,
        "github.event.pull_request.author_association",
        ".github/workflows/cla.yml",
    )?;
    require(
        &cla,
        "repository owner is the Project Licensor",
        ".github/workflows/cla.yml",
    )?;
    require(&cla, "grep -Fqx", ".github/workflows/cla.yml")?;
    if cla.contains("actions/checkout") || cla.contains("cargo run") {
        return Err(
            ".github/workflows/cla.yml must not check out or execute pull-request code".into(),
        );
    }
    println!("CI_POLICY_GATE=PASS");
    Ok(())
}

/// Width of a code-fence line (three or more leading backticks), if `line` is one.
fn fence_width(line: &str) -> Option<usize> {
    let width = line.trim_start().bytes().take_while(|b| *b == b'`').count();
    (width >= 3).then_some(width)
}

/// Contents of the inline code spans in one Markdown line.
///
/// A span opens with a run of backticks and closes with the next run of the
/// same width; a run without a closer is literal text and scanning resumes
/// after it. Backticks are ASCII, so every slice boundary is a char boundary.
fn code_spans(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let open = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let width = i - open;
        let start = i;
        let mut close = None;
        while i < bytes.len() {
            if bytes[i] != b'`' {
                i += 1;
                continue;
            }
            let run = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            if i - run == width {
                close = Some(run);
                break;
            }
        }
        match close {
            Some(end) => spans.push(&line[start..end]),
            None => i = start,
        }
    }
    spans
}

/// One space of padding on both sides of a span is not part of its content.
fn strip_span_padding(span: &str) -> &str {
    if span.len() >= 2 && span.starts_with(' ') && span.ends_with(' ') && !span.trim().is_empty() {
        &span[1..span.len() - 1]
    } else {
        span
    }
}

/// Whether a code-span token is shaped like a repository path that must exist.
fn is_reference(token: &str) -> bool {
    if token.is_empty()
        || token
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '*' | '{' | '$'))
    {
        return false;
    }
    if ALLOWED_EXTENSIONLESS.contains(&token) {
        return true;
    }
    if REFERENCE_PREFIXES
        .iter()
        .any(|prefix| token.starts_with(prefix))
    {
        return true;
    }
    if !token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
    {
        return false;
    }
    token
        .rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && REFERENCE_EXTENSIONS.contains(&ext))
}

/// Repository path citations in a Markdown document as `(line, token)`, lines 1-based.
/// Fenced code blocks are not scanned.
fn reference_tokens(text: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut fence = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(open) = fence {
            let closes = fence_width(line).is_some_and(|width| width >= open)
                && line.trim().trim_start_matches('`').trim().is_empty();
            if closes {
                fence = None;
            }
            continue;
        }
        if let Some(width) = fence_width(line) {
            fence = Some(width);
            continue;
        }
        for span in code_spans(line) {
            let token = strip_span_padding(span);
            if is_reference(token) {
                found.push((index + 1, token));
            }
        }
    }
    found
}

fn docs_refs() -> Result<(), String> {
    let root = workspace_root()?;
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
        for (line, token) in reference_tokens(&text) {
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
        return Err("DOCS_REFS_GATE=FAIL: no repository references found in any .md file".into());
    }
    if missing.is_empty() {
        println!("DOCS_REFS_GATE=PASS");
        println!("DOCS_REFS_CHECKED={checked}");
        Ok(())
    } else {
        for line in &missing {
            eprintln!("{line}");
        }
        Err("DOCS_REFS_GATE=FAIL".into())
    }
}

fn cla_ack() -> Result<(), String> {
    let body = std::env::var_os("PR_BODY")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    if CLA_ACK_TICKED.iter().any(|line| body.contains(line)) {
        println!("CLA_ACK_GATE=PASS");
        Ok(())
    } else {
        eprintln!(
            "PR_BODY is unset or does not contain the ticked CLA acknowledgement line from .github/PULL_REQUEST_TEMPLATE.md."
        );
        eprintln!(
            "A completed CLA must be on file with the project before a pull request is opened; see CLA.md and CONTRIBUTING.md."
        );
        Err("CLA_ACK_GATE=FAIL".into())
    }
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} failed: {status}"))
    }
}

fn run_docs() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["doc", "--locked", "--offline", "--workspace", "--no-deps"])
        .env("RUSTDOCFLAGS", "-D warnings")
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo doc failed: {status}"))
    }
}

/// `cargo run --release -p <package> -- <args>` inside the workspace.
fn run_release_binary<S: AsRef<OsStr>>(package: &str, args: &[S]) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args([
        "run",
        "--locked",
        "--offline",
        "--release",
        "-p",
        package,
        "--",
    ]);
    command.args(args);
    let status = command
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{package} failed: {status}"))
    }
}

fn verify() -> Result<(), String> {
    rust_only()?;
    licensing()?;
    ci_policy()?;
    docs_refs()?;
    run("cargo", FMT_CHECK_ARGS)?;
    run("cargo", CLIPPY_ARGS)?;
    run(
        "cargo",
        &["build", "--locked", "--offline", "--release", "--workspace"],
    )?;
    run_docs()?;
    run("cargo", TEST_ARGS)?;
    run_release_binary("gel-cli", &["selftest"])?;
    run_release_binary("gel-bench", &["8192", "3", "2"])?;
    run_release_binary("gel-bench", &["8192", "3", "1"])?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-phase-quad",
            "--example",
            "quad_compare",
            "--",
            "--orbs",
            "32",
            "--rounds",
            "3",
            "--workers",
            "2",
            "--sparse",
            "1",
        ],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-phase-quad",
            "--example",
            "quad_evidence",
            "--",
            "--demo",
        ],
    )?;
    for (package, example, args) in [
        ("gel-phase-quad", "quad_playground", vec!["--demo"]),
        ("gel-source", "source_readout", vec![]),
        ("gel-source", "source_parts", vec![]),
        ("gel-source", "source_real", vec![]),
        ("gel-source", "source_find", vec![]),
        ("gel-source", "source_build", vec![]),
        ("gel-source", "collection_review", vec![]),
        ("gel-cli", "quantization_matrix", vec![]),
    ] {
        let mut command = vec![
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            package,
            "--example",
            example,
            "--",
        ];
        command.extend(args);
        run("cargo", &command)?;
    }
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--",
            "--demo",
        ],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
    )?;
    println!("GEL_VERIFY_ALL=PASS");
    Ok(())
}

fn collect_args() -> Result<Vec<String>, String> {
    std::env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|bad| format!("argument is not valid UTF-8: {}", bad.to_string_lossy()))
        })
        .collect()
}

fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None | Some("verify") => verify(),
        Some("report") => reproduce::report(&args[1..]),
        Some("source-audit") => source_bundle::audit(&args[1..]),
        Some("source-bundle") => source_bundle::bundle(&args[1..]),
        Some("rust-only") => rust_only(),
        Some("licensing") => licensing(),
        Some("ci-policy") => ci_policy(),
        Some("docs-refs") => docs_refs(),
        Some("cla-ack") => cla_ack(),
        Some("fmt") => run("cargo", FMT_CHECK_ARGS),
        Some("clippy") => run("cargo", CLIPPY_ARGS),
        Some("test") => run("cargo", TEST_ARGS),
        Some("bench") => run_release_binary("gel-bench", &args[1..]),
        Some("physics") => run_release_binary("gel-physics", &args[1..]),
        Some(x) => Err(format!("unknown xtask: {x}; use {USAGE}")),
    }
}

fn main() -> ExitCode {
    match collect_args().and_then(|args| dispatch(&args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-component-gate-{}-{nonce}", std::process::id()));
        let dir = root.join("docs/evidence-gel-components");
        fs::create_dir_all(&dir).unwrap();
        for i in 1..=3 {
            let path = dir.join(format!("run-{i}.csv"));
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
            assert!(rust_only_at(&root).is_ok());
            for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
                fs::write(&path, bad).unwrap();
                assert!(rust_only_at(&root).is_err());
            }
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
        }
        fs::write(dir.join("run-4.csv"), b"unreviewed\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn collection_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-evidence-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/evidence-collection/r1")).unwrap();
        let raw = root.join("docs/evidence-collection/r1/raw.csv");
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        assert!(rust_only_at(&root).is_ok());
        for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
            fs::write(&raw, bad).unwrap();
            assert!(rust_only_at(&root).is_err());
        }
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        fs::write(root.join("unreviewed.csv"), b"text\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn documentation_png_gate_requires_exact_reviewed_bytes_and_path() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-image-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/images")).unwrap();
        for (name, _) in REVIEWED_ASSETS {
            let bytes = fs::read(workspace_root().unwrap().join(name)).unwrap();
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(rust_only_at(&root).is_ok());
            let mut changed = bytes.clone();
            changed[100] ^= 1;
            fs::write(&path, &changed).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::write(&path, &bytes).unwrap();
            let unexpected = root.join("docs/images/unreviewed.png");
            fs::write(&unexpected, &bytes).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(unexpected).unwrap();
            fs::write(&path, b"not a PNG").unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_license_rejects_changed_or_appended_bytes() {
        let original = include_bytes!("../../LICENSE");
        assert!(canonical_license(original));
        let mut changed = original.to_vec();
        changed[0] ^= 1;
        assert!(!canonical_license(&changed));
        changed = original.to_vec();
        changed.push(b'\n');
        assert!(!canonical_license(&changed));
    }

    #[test]
    fn contact_gate_distinguishes_source_suffix_from_an_address() {
        assert!(!has_nonproject_gmail("token.ends_with(\"@gmail.com\")"));
        assert!(!has_nonproject_gmail(PROJECT_EMAIL));
        assert!(!has_nonproject_gmail(&format!("<mailto:{PROJECT_EMAIL}>.")));
        assert!(!has_nonproject_gmail(&PROJECT_EMAIL.to_uppercase()));
        for local in ["fixture", "test.user", "test+tag", "test_user"] {
            assert!(has_nonproject_gmail(&format!("`{local}@{}.`", "gmail.com")));
        }
    }

    #[cfg(unix)]
    #[test]
    fn rust_only_gate_rejects_symlinks_and_executable_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-rust-only-gate-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("good.rs");
        fs::write(&source, "fn main() {}\n").unwrap();
        fs::write(
            root.join("CITATION.cff"),
            "cff-version: 1.2.0\ntitle: \"fixture\"\n",
        )
        .unwrap();
        assert!(rust_only_at(&root).is_ok());

        let link = root.join("link.rs");
        symlink("good.rs", &link).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_file(&link).unwrap();

        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn code_spans_follow_backtick_run_width() {
        assert_eq!(
            code_spans("see `a.md` and ``b `x` c`` here"),
            vec!["a.md", "b `x` c"]
        );
        assert_eq!(code_spans("`` unmatched ` one.md `"), vec![" one.md "]);
        assert_eq!(code_spans("no spans ` here"), Vec::<&str>::new());
        assert_eq!(strip_span_padding(" one.md "), "one.md");
        assert_eq!(strip_span_padding("  "), "  ");
    }

    #[test]
    fn fenced_blocks_are_not_scanned() {
        let text = "```text\n`docs/inside.md`\n```\n`docs/outside.md`\n````\n```\n`docs/still-inside.md`\n````\n`CLA.md`\n";
        assert_eq!(
            reference_tokens(text),
            vec![(4, "docs/outside.md"), (9, "CLA.md")]
        );
    }

    #[test]
    fn reference_shape_matches_repository_paths_only() {
        for yes in [
            "CLA.md",
            "docs/",
            ".github/workflows/ci.yml",
            "crates/gel-core/src/lib.rs",
            "xtask/Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "LICENSE-MODE.txt",
            "LICENSE",
            "NOTICE",
            ".gitignore",
        ] {
            assert!(is_reference(yes), "{yes}");
        }
        for no in [
            "",
            ".gel",
            "xtask",
            "gel-bench",
            "u64::count_ones()",
            "gelram.licensing@gmail.com",
            "<ORB_COUNT> <ROUNDS> [THREADS]",
            "docs/<name>.md",
            "record_count * 128",
            "GEL_BENCH_V3",
            "unsafe_code = \"forbid\"",
            "$HOME/x.md",
            "{root}/x.md",
            "x86_64-unknown-linux-gnu",
        ] {
            assert!(!is_reference(no), "{no}");
        }
    }

    #[test]
    fn cla_ack_needles_match_the_template_line() {
        for needle in CLA_ACK_TICKED {
            assert!(needle.starts_with("[x] ") || needle.starts_with("[X] "));
            assert!(needle.ends_with(" before opening this pull request."));
        }
        assert_eq!(CLA_ACK_TICKED[0][4..], CLA_ACK_TICKED[1][4..]);
    }
}
