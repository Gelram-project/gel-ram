// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Source-header gate (`xtask source-headers`, part of `xtask verify`) and its
//! writer (`xtask source-headers --write`).
//!
//! Every GEL RAM-owned `.rs` file of the tree begins with the two lines of
//! [`HEADER`]: the license name and the Required Notice. The header carries
//! them with the file and should stay with it. The gate reads the same walk as
//! the other whole-tree gates and checks the first two lines byte for byte. A
//! file that starts with a UTF-8 byte order mark, or whose first line ends with
//! CR LF, is refused: the header is written at byte 0 with LF line endings, so
//! it would land after the mark or mix line endings. Lines that look like a
//! header line are found by `gel_core::source_header_like`, the rule
//! `gel-license check` uses too.
//!
//! The writer inserts the header in front of the existing first line where it
//! is missing (no blank line is added), and writes all or nothing. It first
//! refuses every file whose first two lines are a different, partial or
//! altered header, and then writes nothing. Otherwise it writes and syncs a
//! temporary file beside every file to change; if any of them cannot be
//! written, it removes them all and no file has changed. Only then are the
//! temporary files renamed over the originals; should a rename fail, the error
//! says how many files were already renamed, and the remaining temporary files
//! are removed. A second run changes nothing.
use std::{
    fs,
    io::Write as _,
    path::{Component, Path, PathBuf},
};

use crate::tree_scan::{self, Report, Tree};

/// Line 1 of the source header: the license of GEL RAM-owned material, by the
/// name on the first line of LICENSE (`gel_core::SOURCE_HEADER_LINE_1`).
pub const HEADER_LINE_1: &str = gel_core::SOURCE_HEADER_LINE_1;
/// Line 2 of the source header: the `Required Notice:` line of LICENSE and
/// NOTICE (`gel_core::SOURCE_HEADER_LINE_2`).
pub const HEADER_LINE_2: &str = gel_core::SOURCE_HEADER_LINE_2;
/// Both header lines, each ended by a line feed: the first bytes of every
/// GEL RAM-owned `.rs` file (`gel_core::SOURCE_HEADER`).
pub const HEADER: &str = gel_core::SOURCE_HEADER;

/// `.rs` files of the tree that carry no header, as (path, reason). Only
/// material that is not GEL RAM-owned, or whose exact bytes must not change,
/// belongs here; the third-party fixtures and Unicode data files of the tree
/// are text, not Rust source. An entry that is no longer in the tree is a
/// finding.
pub const EXEMPT: &[(&str, &str)] = &[(
    "crates/gel-source/src/nfc_tables.rs",
    "Unicode Character Database data, modified into Rust tables by `xtask ucd-tables`; \
     under the Unicode License v3, not GEL RAM NCRL 1.0 (LICENSING.md section 8, \
     THIRD-PARTY-NOTICES.md section 2a, NOTICE), so a GEL RAM header would contradict them",
)];

/// What the first two lines of one file are.
#[derive(Debug, PartialEq, Eq)]
enum Status {
    /// Both header lines, exact, each ended by a line feed.
    Intact,
    /// No header at all: the writer may insert one.
    Missing,
    /// A different, partial or altered header: the writer refuses the file.
    Altered(&'static str),
}

fn status(bytes: &[u8]) -> Status {
    if bytes.starts_with(gel_core::UTF8_BOM) {
        return Status::Altered("starts with a byte order mark");
    }
    if bytes.starts_with(HEADER.as_bytes()) {
        return Status::Intact;
    }
    let mut lines = bytes.split(|b| *b == b'\n');
    let first = lines.next().unwrap_or_default();
    let second = lines.next().unwrap_or_default();
    let (line_1, line_2) = (HEADER_LINE_1.as_bytes(), HEADER_LINE_2.as_bytes());
    let header_like = gel_core::source_header_like;
    if first == line_1 && second == line_2 {
        Status::Altered("the source header is not ended by a line feed")
    } else if first == line_1 {
        Status::Altered("line 2 is not the Required Notice line of the source header")
    } else if second == line_2 {
        Status::Altered("line 1 is not the license line of the source header")
    } else if first.ends_with(b"\r") {
        Status::Altered("line 1 ends with CR LF; the source header is written with LF only")
    } else if header_like(first) || header_like(second) {
        Status::Altered("the first two lines are a different, partial or altered source header")
    } else {
        Status::Missing
    }
}

/// `path` relative to `root`, with `/` between components on every platform.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|c| match c {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// One `.rs` file of the walk that must carry the header.
struct Source<'a> {
    rel: String,
    path: &'a Path,
    metadata: &'a fs::Metadata,
}

/// The `.rs` files of the walk outside `exempt`, in walk order, and one
/// finding per exemption that names no `.rs` file of the walk.
fn sources<'a>(
    root: &Path,
    tree: &'a Tree,
    exempt: &[(&str, &str)],
) -> (Vec<Source<'a>>, Vec<String>) {
    let mut found = Vec::new();
    let mut seen = vec![false; exempt.len()];
    for (path, metadata) in tree.paths.iter().zip(&tree.metadata) {
        if path.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }
        let rel = relative(root, path);
        if let Some(at) = exempt.iter().position(|(name, _)| *name == rel) {
            seen[at] = true;
            continue;
        }
        found.push(Source {
            rel,
            path,
            metadata,
        });
    }
    let stale = exempt
        .iter()
        .zip(seen)
        .filter(|(_, seen)| !seen)
        .map(|((name, _), _)| format!("{name}: exempt file is not in the tree"))
        .collect();
    (found, stale)
}

/// The status of one source file; a symbolic link is never a source file.
fn read_status(source: &Source) -> Result<Status, String> {
    if source.metadata.file_type().is_symlink() {
        return Ok(Status::Altered("symbolic link, not a source file"));
    }
    let bytes = fs::read(source.path).map_err(|e| format!("{}: {e}", source.rel))?;
    Ok(status(&bytes))
}

fn failure(stderr: Vec<String>, error: String) -> Report {
    Report {
        stdout: Vec::new(),
        stderr,
        result: Err(error),
    }
}

/// The source-header gate over a walked tree.
pub fn check(root: &Path, tree: &Tree) -> Report {
    check_with(root, tree, EXEMPT)
}

/// The `files=<n>` of a passing gate report, read from its one PASS line;
/// `None` for a failing report.
pub fn passed_files(report: &Report) -> Option<usize> {
    match (report.result.is_ok(), report.stdout.as_slice()) {
        (true, [line]) => line
            .strip_prefix("SOURCE_HEADERS_GATE=PASS files=")?
            .parse()
            .ok(),
        _ => None,
    }
}

fn check_with(root: &Path, tree: &Tree, exempt: &[(&str, &str)]) -> Report {
    let (files, stale) = sources(root, tree, exempt);
    let mut bad: Vec<String> = stale;
    for (source, outcome) in files
        .iter()
        .zip(tree_scan::ordered(files.len(), |i| read_status(&files[i])))
    {
        match outcome {
            Ok(Status::Intact) => {}
            Ok(Status::Missing) => bad.push(format!("{}: missing source header", source.rel)),
            Ok(Status::Altered(reason)) => bad.push(format!("{}: {reason}", source.rel)),
            Err(error) => return failure(Vec::new(), error),
        }
    }
    if bad.is_empty() {
        return Report {
            stdout: vec![format!("SOURCE_HEADERS_GATE=PASS files={}", files.len())],
            stderr: Vec::new(),
            result: Ok(()),
        };
    }
    bad.sort();
    let count = bad.len();
    failure(
        bad.into_iter()
            .map(|line| format!("SOURCE_HEADERS_GATE=FAIL {line}"))
            .collect(),
        format!("source headers: {count} finding(s)"),
    )
}

/// A new file beside `path` holding `HEADER` and then `bytes`, written,
/// synced and given the permissions of `path`. A temporary file this call did
/// not create is never removed.
fn prepare(path: &Path, bytes: &[u8], metadata: &fs::Metadata) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("{}: no file name", path.display()))?
        .to_string_lossy();
    let temporary = path.with_file_name(format!(".{name}.source-header.tmp"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| format!("{}: {e}", temporary.display()))?;
    let written = file
        .write_all(HEADER.as_bytes())
        .and_then(|()| file.write_all(bytes))
        .and_then(|()| file.sync_all())
        .and_then(|()| fs::set_permissions(&temporary, metadata.permissions()));
    drop(file);
    match written {
        Ok(()) => Ok(temporary),
        Err(e) => {
            let _ = fs::remove_file(&temporary);
            Err(format!("{}: {e}", path.display()))
        }
    }
}

/// Removes the temporary files of `prepared`; the number removed.
fn remove_all(prepared: &[(PathBuf, &Source)]) -> usize {
    prepared
        .iter()
        .filter(|(temporary, _)| fs::remove_file(temporary).is_ok())
        .count()
}

/// The writer over a walked tree: refuses before writing, inserts where the
/// header is missing (all files or none), then reads the tree again through
/// the gate.
pub fn write(root: &Path, tree: &Tree) -> Report {
    write_with(root, tree, EXEMPT)
}

fn write_with(root: &Path, tree: &Tree, exempt: &[(&str, &str)]) -> Report {
    let (files, stale) = sources(root, tree, exempt);
    let mut refused: Vec<String> = stale;
    let mut missing = Vec::new();
    for (source, outcome) in files
        .iter()
        .zip(tree_scan::ordered(files.len(), |i| read_status(&files[i])))
    {
        match outcome {
            Ok(Status::Intact) => {}
            Ok(Status::Missing) => missing.push(source),
            Ok(Status::Altered(reason)) => refused.push(format!("{}: {reason}", source.rel)),
            Err(error) => return failure(Vec::new(), error),
        }
    }
    if !refused.is_empty() {
        refused.sort();
        let count = refused.len();
        return failure(
            refused
                .into_iter()
                .map(|line| format!("SOURCE_HEADERS_WRITE=REFUSED {line}"))
                .collect(),
            format!("source headers: {count} file(s) refused; nothing written"),
        );
    }
    // One order on every platform, so a failure names the same file each time.
    missing.sort_by(|a, b| a.rel.cmp(&b.rel));
    let total = missing.len();
    // Phase 1: every new file is written and synced beside its original.
    let mut prepared: Vec<(PathBuf, &Source)> = Vec::with_capacity(total);
    for source in &missing {
        let outcome = fs::read(source.path)
            .map_err(|e| format!("{}: {e}", source.rel))
            .and_then(|bytes| {
                // Read again: the file must still be one without a header.
                if status(&bytes) == Status::Missing {
                    prepare(source.path, &bytes, source.metadata)
                } else {
                    Err(format!(
                        "{}: changed while the header was written",
                        source.rel
                    ))
                }
            });
        match outcome {
            Ok(temporary) => prepared.push((temporary, *source)),
            Err(error) => {
                let removed = remove_all(&prepared);
                return failure(
                    vec![format!(
                        "SOURCE_HEADERS_WRITE=FAILED written=0 of {total} temporary_removed={removed}"
                    )],
                    format!("{error}; nothing written"),
                );
            }
        }
    }
    // Phase 2: rename them all over the originals.
    for (done, (temporary, source)) in prepared.iter().enumerate() {
        if let Err(e) = fs::rename(temporary, source.path) {
            let removed = remove_all(&prepared[done..]);
            return failure(
                vec![format!(
                    "SOURCE_HEADERS_WRITE=FAILED written={done} of {total} temporary_removed={removed}"
                )],
                format!("{}: {e}; {done} of {total} file(s) already written", source.rel),
            );
        }
    }
    let mut report = check_with(root, tree, exempt);
    report.stdout.insert(
        0,
        format!(
            "SOURCE_HEADERS_WRITE=DONE written={} already={}",
            missing.len(),
            files.len() - missing.len()
        ),
    );
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const LICENSE: &str = include_str!("../../LICENSE");
    const NOTICE: &str = include_str!("../../NOTICE");

    fn scratch(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "gel-source-headers-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn put(root: &Path, rel: &str, bytes: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn gate(root: &Path, exempt: &[(&str, &str)]) -> Report {
        check_with(root, &tree_scan::walk(root).unwrap(), exempt)
    }

    fn writer(root: &Path, exempt: &[(&str, &str)]) -> Report {
        write_with(root, &tree_scan::walk(root).unwrap(), exempt)
    }

    /// The headed form of `body`.
    fn headed(body: &str) -> String {
        format!("{HEADER}{body}")
    }

    /// A tree whose `.rs` files all carry the header, beside files the gate
    /// does not read: other extensions and skipped directories.
    fn good_tree(name: &str) -> PathBuf {
        let root = scratch(name);
        put(&root, "src/lib.rs", &headed("#![forbid(unsafe_code)]\n"));
        put(&root, "src/deep/mod.rs", &headed("//! Doc.\nfn f() {}\n"));
        put(&root, "README.md", "no header here\n");
        put(&root, "target/debug/build.rs", "fn main() {}\n");
        put(&root, ".git/hooks/x.rs", "fn main() {}\n");
        root
    }

    #[test]
    fn the_header_is_the_license_name_and_the_required_notice() {
        assert_eq!(HEADER, format!("{HEADER_LINE_1}\n{HEADER_LINE_2}\n"));
        let name = LICENSE.lines().next().unwrap();
        assert_eq!(
            HEADER_LINE_1,
            format!("// GEL RAM — {name}; see LICENSE and NOTICE.")
        );
        for text in [LICENSE, NOTICE] {
            assert!(text.lines().any(|line| line.starts_with("Required Notice:")
                && format!("// {line}") == HEADER_LINE_2));
        }
    }

    #[test]
    fn every_exemption_is_a_headerless_file_of_the_workspace_with_a_reason() {
        let root = crate::workspace_root().unwrap();
        assert!(EXEMPT
            .iter()
            .any(|(name, _)| *name == "crates/gel-source/src/nfc_tables.rs"));
        for (name, reason) in EXEMPT {
            assert!(!reason.trim().is_empty(), "{name}");
            let bytes = fs::read(root.join(name)).unwrap();
            assert_eq!(status(&bytes), Status::Missing, "{name}");
        }
    }

    #[test]
    fn a_tree_with_every_header_passes() {
        let root = good_tree("pass");
        let report = gate(&root, &[]);
        assert_eq!(report.result, Ok(()));
        assert_eq!(report.stdout, ["SOURCE_HEADERS_GATE=PASS files=2"]);
        assert_eq!(passed_files(&report), Some(2));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_file_without_a_header_fails() {
        let root = good_tree("missing");
        put(&root, "src/plain.rs", "fn plain() {}\n");
        put(&root, "src/empty.rs", "");
        let report = gate(&root, &[]);
        assert!(report.result.is_err());
        assert_eq!(passed_files(&report), None);
        assert_eq!(
            report.stderr,
            [
                "SOURCE_HEADERS_GATE=FAIL src/empty.rs: missing source header",
                "SOURCE_HEADERS_GATE=FAIL src/plain.rs: missing source header",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_altered_first_line_fails() {
        let root = good_tree("line-1");
        let changed = HEADER.replacen("License 1.0", "License 2.0", 1);
        put(&root, "src/one.rs", &format!("{changed}fn f() {{}}\n"));
        let report = gate(&root, &[]);
        assert!(report.result.is_err());
        assert_eq!(
            report.stderr,
            ["SOURCE_HEADERS_GATE=FAIL src/one.rs: line 1 is not the license line of the source header"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_altered_second_line_fails() {
        let root = good_tree("line-2");
        let changed = HEADER.replacen("(C) 2026", "(C) 2027", 1);
        put(&root, "src/two.rs", &format!("{changed}fn f() {{}}\n"));
        // Line 1 alone, the original first line after it: a partial header.
        put(
            &root,
            "src/partial.rs",
            &format!("{HEADER_LINE_1}\nfn f() {{}}\n"),
        );
        // Both lines without the closing line feed.
        put(
            &root,
            "src/open.rs",
            &format!("{HEADER_LINE_1}\n{HEADER_LINE_2}"),
        );
        let report = gate(&root, &[]);
        assert!(report.result.is_err());
        assert_eq!(
            report.stderr,
            [
                "SOURCE_HEADERS_GATE=FAIL src/open.rs: the source header is not ended by a line feed",
                "SOURCE_HEADERS_GATE=FAIL src/partial.rs: line 2 is not the Required Notice line of the source header",
                "SOURCE_HEADERS_GATE=FAIL src/two.rs: line 2 is not the Required Notice line of the source header",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_listed_third_party_file_is_exempt_and_a_stale_listing_fails() {
        let root = good_tree("exempt");
        put(
            &root,
            "fixtures/upstream.rs",
            "// Upstream code.\nfn f() {}\n",
        );
        let listed = [("fixtures/upstream.rs", "third-party fixture, exact bytes")];
        let report = gate(&root, &listed);
        assert_eq!(report.result, Ok(()));
        assert_eq!(report.stdout, ["SOURCE_HEADERS_GATE=PASS files=2"]);
        // Unlisted, the same file fails.
        assert!(gate(&root, &[]).result.is_err());
        // A listing that names no file of the tree is a finding.
        let stale = [("fixtures/gone.rs", "removed upstream file")];
        let report = gate(&root, &[listed[0], stale[0]]);
        assert_eq!(
            report.stderr,
            ["SOURCE_HEADERS_GATE=FAIL fixtures/gone.rs: exempt file is not in the tree"]
        );
        // The writer leaves the listed file as it is.
        let report = writer(&root, &listed);
        assert_eq!(report.result, Ok(()));
        assert_eq!(
            fs::read_to_string(root.join("fixtures/upstream.rs")).unwrap(),
            "// Upstream code.\nfn f() {}\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_writer_inserts_the_header_once() {
        let root = good_tree("write");
        let body = "#![forbid(unsafe_code)]\n//! A doc comment that names GEL RAM.\nfn f() {}\n";
        put(&root, "src/new.rs", body);
        put(&root, "src/empty.rs", "");
        let report = writer(&root, &[]);
        assert_eq!(report.result, Ok(()));
        assert_eq!(
            report.stdout,
            [
                "SOURCE_HEADERS_WRITE=DONE written=2 already=2",
                "SOURCE_HEADERS_GATE=PASS files=4",
            ]
        );
        assert_eq!(
            fs::read_to_string(root.join("src/new.rs")).unwrap(),
            headed(body)
        );
        assert_eq!(
            fs::read_to_string(root.join("src/empty.rs")).unwrap(),
            HEADER
        );
        assert_eq!(
            fs::read_to_string(root.join("target/debug/build.rs")).unwrap(),
            "fn main() {}\n"
        );
        // A second run writes nothing and changes no byte.
        let before: Vec<Vec<u8>> = ["src/new.rs", "src/empty.rs", "src/lib.rs"]
            .iter()
            .map(|rel| fs::read(root.join(rel)).unwrap())
            .collect();
        let report = writer(&root, &[]);
        assert_eq!(report.result, Ok(()));
        assert_eq!(
            report.stdout[0],
            "SOURCE_HEADERS_WRITE=DONE written=0 already=4"
        );
        let after: Vec<Vec<u8>> = ["src/new.rs", "src/empty.rs", "src/lib.rs"]
            .iter()
            .map(|rel| fs::read(root.join(rel)).unwrap())
            .collect();
        assert_eq!(before, after);
        // No temporary file is left behind.
        let names: Vec<String> = fs::read_dir(root.join("src"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().all(|n| !n.ends_with(".tmp")), "{names:?}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_writer_refuses_an_altered_header_and_writes_nothing() {
        let root = good_tree("refuse");
        put(&root, "src/new.rs", "fn f() {}\n");
        let altered = format!("{HEADER_LINE_1}\n// Required Notice: someone else\nfn g() {{}}\n");
        put(&root, "src/altered.rs", &altered);
        let other = "// GEL RAM — another license\n// Required Notice: another\nfn h() {}\n";
        put(&root, "src/other.rs", other);
        let report = writer(&root, &[]);
        assert!(report.result.is_err());
        assert!(report.stdout.is_empty());
        assert_eq!(
            report.stderr,
            [
                "SOURCE_HEADERS_WRITE=REFUSED src/altered.rs: line 2 is not the Required Notice line of the source header",
                "SOURCE_HEADERS_WRITE=REFUSED src/other.rs: the first two lines are a different, partial or altered source header",
            ]
        );
        assert_eq!(
            fs::read_to_string(root.join("src/altered.rs")).unwrap(),
            altered
        );
        assert_eq!(
            fs::read_to_string(root.join("src/other.rs")).unwrap(),
            other
        );
        // The file without a header is not written either: refusal comes first.
        assert_eq!(
            fs::read_to_string(root.join("src/new.rs")).unwrap(),
            "fn f() {}\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn doc_comments_and_other_comments_are_not_headers() {
        assert_eq!(
            status(b"//! GEL RAM crate docs.\nfn f() {}\n"),
            Status::Missing
        );
        assert_eq!(status(b"/// GEL RAM item.\n"), Status::Missing);
        assert_eq!(
            status(b"// Frozen code.\n// Kept as is.\n"),
            Status::Missing
        );
        assert_eq!(
            status(b"//GEL RAM\n"),
            Status::Altered(
                "the first two lines are a different, partial or altered source header"
            )
        );
        assert_eq!(
            status(b"fn f() {}\n//  Required Notice: x\n"),
            Status::Altered(
                "the first two lines are a different, partial or altered source header"
            )
        );
        let crlf = HEADER.replace('\n', "\r\n");
        assert!(matches!(status(crlf.as_bytes()), Status::Altered(_)));
    }

    const BOM_REASON: &str = "starts with a byte order mark";
    const CRLF_REASON: &str = "line 1 ends with CR LF; the source header is written with LF only";

    #[test]
    fn a_byte_order_mark_is_refused_with_or_without_a_header() {
        assert_eq!(
            status("\u{feff}fn bom() {}\n".as_bytes()),
            Status::Altered(BOM_REASON)
        );
        assert_eq!(
            status(format!("\u{feff}{HEADER}fn f() {{}}\n").as_bytes()),
            Status::Altered(BOM_REASON)
        );
        let root = good_tree("bom");
        put(&root, "src/bom.rs", "\u{feff}fn bom() {}\n");
        assert_eq!(
            gate(&root, &[]).stderr,
            [format!("SOURCE_HEADERS_GATE=FAIL src/bom.rs: {BOM_REASON}")]
        );
        let report = writer(&root, &[]);
        assert!(report.result.is_err());
        assert_eq!(
            report.stderr,
            [format!(
                "SOURCE_HEADERS_WRITE=REFUSED src/bom.rs: {BOM_REASON}"
            )]
        );
        assert_eq!(
            fs::read_to_string(root.join("src/bom.rs")).unwrap(),
            "\u{feff}fn bom() {}\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn crlf_line_endings_are_refused_not_mixed() {
        // A CRLF header is altered, as in `gel-license check`.
        assert_eq!(
            status(HEADER.replace('\n', "\r\n").as_bytes()),
            Status::Altered(CRLF_REASON)
        );
        let root = good_tree("crlf");
        put(&root, "src/crlf.rs", "fn f() {}\r\nfn g() {}\r\n");
        let report = writer(&root, &[]);
        assert_eq!(
            report.stderr,
            [format!(
                "SOURCE_HEADERS_WRITE=REFUSED src/crlf.rs: {CRLF_REASON}"
            )]
        );
        assert_eq!(
            fs::read_to_string(root.join("src/crlf.rs")).unwrap(),
            "fn f() {}\r\nfn g() {}\r\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    /// Every file name below `dir`, recursively.
    fn names_below(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = tree_scan::walk(dir)
            .unwrap()
            .paths
            .iter()
            .map(|p| relative(dir, p))
            .collect();
        names.sort();
        names
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_write_changes_no_file_and_leaves_no_temporary_file() {
        use std::os::unix::fs::PermissionsExt;
        let root = good_tree("all-or-nothing");
        put(&root, "a/one.rs", "fn one() {}\n");
        put(&root, "a/two.rs", "fn two() {}\n");
        put(&root, "z/locked.rs", "fn locked() {}\n");
        let locked = root.join("z");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
        // A user who may write into a read-only directory cannot run this case.
        let probe = locked.join("probe");
        if fs::write(&probe, b"").is_ok() {
            fs::remove_file(&probe).unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
            fs::remove_dir_all(root).unwrap();
            return;
        }
        let before = names_below(&root);
        let report = writer(&root, &[]);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        let error = report.result.unwrap_err();
        assert!(error.contains("locked.rs"), "{error}");
        assert!(error.ends_with("; nothing written"), "{error}");
        assert_eq!(
            report.stderr,
            ["SOURCE_HEADERS_WRITE=FAILED written=0 of 3 temporary_removed=2"]
        );
        assert!(report.stdout.is_empty());
        // The files sorted before the locked one were not written either.
        for (rel, body) in [
            ("a/one.rs", "fn one() {}\n"),
            ("a/two.rs", "fn two() {}\n"),
            ("z/locked.rs", "fn locked() {}\n"),
        ] {
            assert_eq!(fs::read_to_string(root.join(rel)).unwrap(), body, "{rel}");
        }
        assert_eq!(names_below(&root), before);
        // Once the directory may be written, the same run writes all three.
        let report = writer(&root, &[]);
        assert_eq!(report.result, Ok(()));
        assert_eq!(
            report.stdout[0],
            "SOURCE_HEADERS_WRITE=DONE written=3 already=2"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_source_file_is_refused() {
        let root = good_tree("link");
        std::os::unix::fs::symlink(root.join("src/lib.rs"), root.join("src/link.rs")).unwrap();
        let report = gate(&root, &[]);
        assert_eq!(
            report.stderr,
            ["SOURCE_HEADERS_GATE=FAIL src/link.rs: symbolic link, not a source file"]
        );
        assert!(writer(&root, &[]).result.is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
