// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Every file that names the current version names the version of the
//! workspace, and the release notes in the root are those of that version only.
use std::path::Path;

/// (file, the text it must contain), `{v}` standing for the version.
const MENTIONS: &[(&str, &str)] = &[
    ("CITATION.cff", "version: \"{v}\""),
    (
        "docs/EVIDENCE-BATCH.md",
        "\"tool\":\"gel-evidence\",\"version\":\"{v}\"",
    ),
    ("CANDIDATE-STATUS.md", "# GEL RAM {v} — "),
    ("CANDIDATE-STATUS.md", "\nVERSION_LINE={v}\n"),
    ("CANDIDATE-STATUS.md", "\nTARGET_RELEASE={v}\n"),
    ("SECURITY.md", "(version line {v})"),
    (".github/ISSUE_TEMPLATE/bug.yml", "placeholder: v{v} or"),
    ("docs/REPRODUCE.md", "\ngit checkout v{v}\n"),
    ("docs/TRY-IT.md", "\ngit checkout v{v}\n"),
    ("docs/TRY-IT.md", "at the v{v} tag."),
    (
        "docs/CANDIDATE-QUICKCHECK.md",
        "# Verify Evidence Lab ({v}) yourself",
    ),
    (
        "tools/readme_presentation.rs",
        "const RELEASE: &str = \"v{v}\";",
    ),
    ("README.md", "(RELEASE-NOTES-v{v}.md)"),
];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn workspace_version(manifest: &str) -> Option<&str> {
    let package = manifest.split("\n[workspace.package]\n").nth(1)?;
    let line = package.lines().find(|l| l.starts_with("version = "))?;
    line.strip_prefix("version = \"")?.strip_suffix('"')
}

/// `read` gives a file's text by its path from the root; `notes` lists the
/// release-notes files in the root.
fn check(
    version: &str,
    read: &dyn Fn(&str) -> Option<String>,
    notes: &[String],
) -> Result<(), String> {
    for (file, pattern) in MENTIONS {
        let wanted = pattern.replace("{v}", version);
        let text = read(file).ok_or(format!("{file} is missing"))?;
        if !text.contains(&wanted) {
            return Err(format!("{file} does not say {wanted:?}"));
        }
    }
    let current = format!("RELEASE-NOTES-v{version}.md");
    if notes != [current.clone()] {
        return Err(format!(
            "the root must hold only {current}, found {notes:?}"
        ));
    }
    Ok(())
}

fn read_file(file: &str) -> Option<String> {
    std::fs::read_to_string(root().join(file)).ok()
}

fn root_notes() -> Vec<String> {
    let mut notes: Vec<String> = std::fs::read_dir(root())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.starts_with("RELEASE-NOTES-v") && n.ends_with(".md"))
        .collect();
    notes.sort();
    notes
}

#[test]
fn every_version_mention_follows_the_workspace_version() {
    let manifest = read_file("Cargo.toml").unwrap();
    let version = workspace_version(&manifest).unwrap();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    check(version, &read_file, &root_notes()).unwrap();
}

#[test]
fn each_stale_mention_and_each_wrong_notes_file_is_refused() {
    let version = env!("CARGO_PKG_VERSION");
    let notes = root_notes();
    for (i, (file, pattern)) in MENTIONS.iter().enumerate() {
        let current = pattern.replace("{v}", version);
        let stale = pattern.replace("{v}", "0.0.1");
        let read = |f: &str| {
            let text = read_file(f)?;
            Some(if f == *file {
                text.replace(&current, &stale)
            } else {
                text
            })
        };
        let refused = check(version, &read, &notes).expect_err(&format!("mention {i} kept"));
        assert!(refused.contains(file), "{refused}");
    }
    let mut two = notes.clone();
    two.push("RELEASE-NOTES-v0.0.1.md".to_owned());
    assert!(check(version, &read_file, &two).is_err());
    assert!(check(version, &read_file, &[]).is_err());
    assert!(check(version, &|_| None, &notes).is_err());
}
