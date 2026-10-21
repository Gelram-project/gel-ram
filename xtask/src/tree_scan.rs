//! One inventory of the source tree for the gates that read every file:
//! rust-only, license metadata, project contact, docs-refs and disclosure.
//!
//! The inventory is the same depth-first walk the gates used before: entries in
//! directory-read order, `target` and `.git` skipped at every depth, and one
//! `symlink_metadata` call per entry, kept next to its path so no gate has to
//! ask again. Per-file work runs on scoped threads and is merged back in file
//! order, so every gate prints the same lines, in the same order, and returns
//! the same first error as one sequential pass over the same inventory.
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub const ALLOWED_EXTENSIONS: &[&str] = &["rs", "md", "toml", "yml", "txt", "gel", "cff"];
pub const ALLOWED_EXTENSIONLESS: &[&str] = &[
    "Cargo.lock",
    "LICENSE",
    "NOTICE",
    ".gitignore",
    ".gitattributes",
];

/// A backtick-quoted token with one of these extensions is a repository path citation.
pub const REFERENCE_EXTENSIONS: &[&str] = &["md", "toml", "txt", "yml", "rs", "lock", "cff"];
/// A backtick-quoted token starting with one of these prefixes is a repository path citation.
pub const REFERENCE_PREFIXES: &[&str] = &["docs/", ".github/", "crates/", "xtask/"];

pub const PROJECT_EMAIL: &str = "gelram.licensing@gmail.com";

/// Upper bound on the threads that share per-file work.
const MAX_THREADS: usize = 8;

/// Every non-directory entry of a walk, with the metadata read for it.
pub struct Tree {
    pub paths: Vec<PathBuf>,
    pub metadata: Vec<fs::Metadata>,
}

/// Lines a gate prints (standard output, then standard error) and its result.
pub struct Report {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub result: Result<(), String>,
}

impl Report {
    fn pass(stdout: Vec<String>) -> Self {
        Self {
            stdout,
            stderr: Vec::new(),
            result: Ok(()),
        }
    }

    fn fail(stderr: Vec<String>, error: &str) -> Self {
        Self {
            stdout: Vec::new(),
            stderr,
            result: Err(error.into()),
        }
    }

    fn error(error: String) -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
            result: Err(error),
        }
    }
}

/// Walks `root`; the first error ends the walk, as it did for every gate.
pub fn walk(root: &Path) -> io::Result<Tree> {
    let mut tree = Tree {
        paths: Vec::new(),
        metadata: Vec::new(),
    };
    walk_into(root, &mut tree)?;
    Ok(tree)
}

fn walk_into(dir: &Path, tree: &mut Tree) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
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
            tree.paths.push(path);
            tree.metadata.push(metadata);
        } else if metadata.is_dir() {
            walk_into(&path, tree)?;
        } else {
            tree.paths.push(path);
            tree.metadata.push(metadata);
        }
    }
    Ok(())
}

/// `work(0..count)` on up to `MAX_THREADS` threads (never more than the
/// available parallelism), results in index order. The calling thread works
/// too, so a refused thread start only means fewer helpers.
pub fn ordered<R: Send>(count: usize, work: impl Fn(usize) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(MAX_THREADS)
        .min(count);
    if threads <= 1 {
        return (0..count).map(work).collect();
    }
    let next = AtomicUsize::new(0);
    let run = || {
        let mut done = Vec::new();
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= count {
                break done;
            }
            done.push((index, work(index)));
        }
    };
    let mut slots: Vec<Option<R>> = Vec::with_capacity(count);
    slots.resize_with(count, || None);
    std::thread::scope(|scope| {
        let helpers: Vec<_> = (1..threads)
            .filter_map(|_| std::thread::Builder::new().spawn_scoped(scope, run).ok())
            .collect();
        let mut parts = vec![run()];
        for helper in helpers {
            match helper.join() {
                Ok(part) => parts.push(part),
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        for (index, result) in parts.into_iter().flatten() {
            slots[index] = Some(result);
        }
    });
    slots
        .into_iter()
        .map(|slot| slot.expect("every index is claimed exactly once"))
        .collect()
}

/// The rust-only gate over a walked tree. `reviewed_assets` are the exact
/// reviewed media paths and their SHA-256 pins.
pub fn rust_only(root: &Path, tree: &Tree, reviewed_assets: &[(&str, &str)]) -> Report {
    // Exactly two reviewed, static vector illustrations; not a general
    // SVG or executable-asset exception. Implementation remains Rust-only.
    let documentation_images = [
        "docs/images/public-readout.svg",
        "docs/images/q8-r2-results.svg",
    ]
    .map(|p| root.join(p));
    let reviewed: Vec<(PathBuf, &str)> = reviewed_assets
        .iter()
        .map(|(name, expected)| (root.join(name), *expected))
        .collect();
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
    .map(|p| root.join(p));
    let found = ordered(tree.paths.len(), |i| {
        rust_only_file(
            &tree.paths[i],
            &tree.metadata[i],
            &documentation_images,
            &reviewed,
            &collection_evidence,
        )
    });
    let mut bad = Vec::new();
    for finding in found {
        match finding {
            Ok(Some(message)) => bad.push(message),
            Ok(None) => {}
            Err(error) => return Report::error(error),
        }
    }
    bad.sort();
    bad.dedup();
    if bad.is_empty() {
        Report::pass(vec!["RUST_ONLY_GATE=PASS".into()])
    } else {
        Report::fail(bad, "RUST_ONLY_GATE=FAIL")
    }
}

/// The rust-only finding for one file, if any.
fn rust_only_file(
    path: &Path,
    metadata: &fs::Metadata,
    documentation_images: &[PathBuf],
    reviewed: &[(PathBuf, &str)],
    collection_evidence: &[PathBuf],
) -> Result<Option<String>, String> {
    if metadata.file_type().is_symlink() {
        return Ok(Some(format!("symlink is not allowed: {}", path.display())));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o111 != 0 {
        return Ok(Some(format!(
            "executable file is not allowed in the release tree: {}",
            path.display()
        )));
    }
    let documentation_image = documentation_images.iter().any(|p| path == p);
    let approved_png = reviewed.iter().find(|(name, _)| path == name);
    if let Some((_, expected)) = approved_png {
        if !metadata.is_file() || metadata.len() > 4 * 1024 * 1024 {
            return Ok(Some(format!(
                "invalid reviewed asset size/type: {}",
                path.display()
            )));
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if gel_source::hex(&gel_source::digest(&bytes)) != *expected {
            return Ok(Some(format!(
                "reviewed asset fingerprint mismatch: {}",
                path.display()
            )));
        }
    }
    let collection_evidence = collection_evidence.iter().any(|p| path == p);
    if collection_evidence {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if bytes.len() > 1024 * 1024
            || std::str::from_utf8(&bytes).is_err()
            || bytes.iter().any(|b| *b == 0 || *b == 27)
        {
            return Ok(Some(format!(
                "invalid collection text evidence: {}",
                path.display()
            )));
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
    if allowed {
        Ok(None)
    } else {
        Ok(Some(format!(
            "file type is outside the release allow-list: {}",
            path.display()
        )))
    }
}

// Narrow accidental-contact check, NOT a general secret or email scanner.
// A bare domain suffix in source code is not an email address.
pub fn has_nonproject_gmail(text: &str) -> bool {
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

/// The project-contact gate over a walked tree; the first file in walk order
/// that cannot be read or holds a non-project Gmail address is the error.
pub fn project_contact(tree: &Tree) -> Report {
    for outcome in ordered(tree.paths.len(), |i| contact_file(&tree.paths[i])) {
        if let Err(error) = outcome {
            return Report::error(error);
        }
    }
    Report::pass(vec!["PROJECT_CONTACT_GATE=PASS".into()])
}

fn contact_file(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if let Ok(text) = std::str::from_utf8(&bytes) {
        if has_nonproject_gmail(text) {
            // Do not echo a potentially private address into public CI logs.
            return Err(format!("non-project Gmail address in {}", path.display()));
        }
    }
    Ok(())
}

/// Width of a code-fence line (three or more leading backticks), if `line` is one.
pub fn fence_width(line: &str) -> Option<usize> {
    let width = line.trim_start().bytes().take_while(|b| *b == b'`').count();
    (width >= 3).then_some(width)
}

/// Contents of the inline code spans in one Markdown line.
///
/// A span opens with a run of backticks and closes with the next run of the
/// same width; a run without a closer is literal text and scanning resumes
/// after it. Backticks are ASCII, so every slice boundary is a char boundary.
pub fn code_spans(line: &str) -> Vec<&str> {
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
pub fn strip_span_padding(span: &str) -> &str {
    if span.len() >= 2 && span.starts_with(' ') && span.ends_with(' ') && !span.trim().is_empty() {
        &span[1..span.len() - 1]
    } else {
        span
    }
}

/// Whether a code-span token is shaped like a repository path that must exist.
pub fn is_reference(token: &str) -> bool {
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
pub fn reference_tokens(text: &str) -> Vec<(usize, &str)> {
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

/// The docs-refs gate over a walked tree: Markdown files in sorted path order.
pub fn docs_refs(root: &Path, tree: &Tree) -> Report {
    let mut files: Vec<&PathBuf> = tree
        .paths
        .iter()
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("md"))
        .collect();
    files.sort();
    let mut checked = 0usize;
    let mut missing = Vec::new();
    for outcome in ordered(files.len(), |i| docs_refs_file(root, files[i])) {
        match outcome {
            Ok((count, lines)) => {
                checked += count;
                missing.extend(lines);
            }
            Err(error) => return Report::error(error),
        }
    }
    if checked == 0 {
        return Report::error(
            "DOCS_REFS_GATE=FAIL: no repository references found in any .md file".into(),
        );
    }
    if missing.is_empty() {
        Report::pass(vec![
            "DOCS_REFS_GATE=PASS".into(),
            format!("DOCS_REFS_CHECKED={checked}"),
        ])
    } else {
        Report::fail(missing, "DOCS_REFS_GATE=FAIL")
    }
}

/// References checked in one Markdown file and the lines for the missing ones.
fn docs_refs_file(root: &Path, path: &Path) -> Result<(usize, Vec<String>), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let citing = path.strip_prefix(root).unwrap_or(path);
    let mut checked = 0usize;
    let mut missing = Vec::new();
    for (line, token) in reference_tokens(&text) {
        checked += 1;
        if !root.join(token).exists() {
            missing.push(format!(
                "{}:{line}: missing repository reference: {token}",
                citing.display()
            ));
        }
    }
    Ok((checked, missing))
}
