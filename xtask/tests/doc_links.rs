// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Every relative Markdown link of the roadmap and the status file leads to a
//! file or directory of the tree, and every `#anchor` to a heading of its
//! target; the historical test count of the status file equals the pinned
//! platform record. (`xtask docs-refs` reads only paths in code spans.)
use std::path::Path;

/// The files whose links are checked, by their path from the root.
const FILES: &[&str] = &["docs/ROADMAP.md", "CANDIDATE-STATUS.md"];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn read_file(file: &str) -> Option<String> {
    std::fs::read_to_string(root().join(file)).ok()
}

fn exists(path: &str) -> bool {
    root().join(path).exists()
}

/// A line without its code spans.
fn without_code_spans(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        out.push_str(&rest[..start]);
        let run = rest[start..].len() - rest[start..].trim_start_matches('`').len();
        let after = &rest[start + run..];
        let fence = &rest[start..start + run];
        match after.find(fence) {
            Some(end) => rest = &after[end + run..],
            None => {
                out.push_str(&rest[start..]);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The lines of `text` outside fenced code blocks, 1-based.
fn prose_lines(text: &str) -> Vec<(usize, &str)> {
    let mut fence: Option<&str> = None;
    let mut lines = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| trimmed.starts_with(m));
        match (fence, marker) {
            (None, Some(m)) => fence = Some(m),
            (Some(open), Some(m)) if open == m => fence = None,
            (None, None) => lines.push((i + 1, line)),
            _ => {}
        }
    }
    lines
}

/// Link targets `(line, target)` of Markdown `[text](target)` and `![alt](target)`.
fn links(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (number, line) in prose_lines(text) {
        let line = without_code_spans(line);
        let mut rest = line.as_str();
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            let Some(end) = after.find(')') else { break };
            let target = after[..end].split_whitespace().next().unwrap_or("");
            found.push((number, target.to_owned()));
            rest = &after[end..];
        }
    }
    found
}

/// The GitHub anchor of a heading: lower case, letters, digits, `-` and `_`
/// kept, each space a `-`, everything else dropped.
fn slug(heading: &str) -> String {
    heading
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

/// The anchors of a Markdown text: its headings, a repeated one with `-1`,
/// `-2` ..., and every HTML `id` or `name` attribute.
fn anchors(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (_, line) in prose_lines(text) {
        let hashes = line.len() - line.trim_start_matches('#').len();
        if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
            let base = slug(line[hashes..].trim_end_matches('#'));
            let mut anchor = base.clone();
            let mut n = 0;
            while found.contains(&anchor) {
                n += 1;
                anchor = format!("{base}-{n}");
            }
            found.push(anchor);
        }
        for attribute in [" id=\"", " name=\""] {
            for part in line.split(attribute).skip(1) {
                if let Some(end) = part.find('"') {
                    found.push(part[..end].to_owned());
                }
            }
        }
    }
    found
}

/// `path` relative to the directory of `file`, as a path from the root, or
/// `None` when it leaves the root.
fn resolve(file: &str, path: &str) -> Option<String> {
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Checks the links of `files`; `read` gives a file's text and `exists` says
/// whether a path from the root exists. Returns the number of links checked.
fn check(
    files: &[&str],
    read: &dyn Fn(&str) -> Option<String>,
    exists: &dyn Fn(&str) -> bool,
) -> Result<usize, Vec<String>> {
    let mut checked = 0;
    let mut broken = Vec::new();
    for file in files {
        let Some(text) = read(file) else {
            broken.push(format!("{file} is missing"));
            continue;
        };
        for (line, target) in links(&text) {
            if target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            checked += 1;
            let (path, anchor) = match target.split_once('#') {
                Some((path, anchor)) => (path, Some(anchor)),
                None => (target.as_str(), None),
            };
            let resolved = if path.is_empty() {
                Some((*file).to_owned())
            } else {
                resolve(file, path)
            };
            let Some(resolved) = resolved.filter(|r| !r.is_empty() && exists(r)) else {
                broken.push(format!("{file}:{line}: no such file: {target}"));
                continue;
            };
            let Some(anchor) = anchor else { continue };
            let known = resolved.ends_with(".md")
                && read(&resolved).is_some_and(|t| anchors(&t).iter().any(|a| a == anchor));
            if !known {
                broken.push(format!("{file}:{line}: no such anchor: {target}"));
            }
        }
    }
    if broken.is_empty() {
        Ok(checked)
    } else {
        Err(broken)
    }
}

/// The historical Linux test count of the status file and of the pinned
/// platform record must be the same number.
fn historical_count(status: &str, platform: &str) -> Result<(), String> {
    let stated = status
        .lines()
        .find_map(|l| l.strip_prefix("LINUX_WORKSPACE_TESTS_AT_71142A2="))
        .and_then(|v| v.strip_suffix("_PASS"))
        .ok_or("CANDIDATE-STATUS.md has no LINUX_WORKSPACE_TESTS_AT_71142A2=N_PASS")?;
    let recorded = platform
        .lines()
        .find_map(|l| l.strip_prefix("| Linux | "))
        .and_then(|v| v.split(" PASS |").next())
        .ok_or("docs/PLATFORM-REVIEW.md has no Linux row")?;
    if !platform.contains("merged main `71142a2`") {
        return Err("docs/PLATFORM-REVIEW.md no longer records the revision of 71142a2".into());
    }
    if stated == recorded {
        Ok(())
    } else {
        Err(format!(
            "CANDIDATE-STATUS.md says {stated} tests at 71142a2, the platform record {recorded}"
        ))
    }
}

#[test]
fn every_link_of_the_roadmap_and_the_status_file_resolves() {
    let checked = check(FILES, &read_file, &exists).unwrap_or_else(|e| panic!("{e:#?}"));
    assert!(checked > 50, "only {checked} links checked");
}

#[test]
fn the_historical_test_count_matches_the_platform_record() {
    historical_count(
        &read_file("CANDIDATE-STATUS.md").unwrap(),
        &read_file("docs/PLATFORM-REVIEW.md").unwrap(),
    )
    .unwrap();
}

#[test]
fn broken_links_and_anchors_are_refused() {
    let roadmap = read_file("docs/ROADMAP.md").unwrap();
    let with = |extra: &str| {
        let changed = format!("{roadmap}\n{extra}\n");
        move |f: &str| {
            if f == "docs/ROADMAP.md" {
                Some(changed.clone())
            } else {
                read_file(f)
            }
        }
    };
    let only = &["docs/ROADMAP.md"][..];
    for extra in [
        "[gone](NO-SUCH-FILE.md)",
        "[gone](../NO-SUCH-FILE.md)",
        "[outside](../../outside.md)",
        "[anchor](LIVE-LAB.md#no-such-heading)",
        "[here](#no-such-heading)",
        "[not markdown](../Cargo.toml#package)",
    ] {
        let refused = check(only, &with(extra), &exists).expect_err(extra);
        assert_eq!(refused.len(), 1, "{refused:?}");
    }
    for kept in [
        "```text\n[gone](NO-SUCH-FILE.md)\n```",
        "`[gone](NO-SUCH-FILE.md)`",
        "[web](https://example.org/NO-SUCH-FILE.md)",
    ] {
        check(only, &with(kept), &exists).expect(kept);
    }
    let renamed = roadmap.replacen("\n## After v", "\n## Since v", 1);
    let read = |f: &str| {
        if f == "docs/ROADMAP.md" {
            Some(renamed.clone())
        } else {
            read_file(f)
        }
    };
    assert!(check(only, &read, &exists).is_err());
    assert!(check(only, &|_: &str| None, &exists).is_err());
}

#[test]
fn headings_give_github_anchors() {
    let text = "# A b\n## Public main — already delivered\n## v0.8.0\n## v0.8.0\n```\n## Not a heading\n```\n<a id=\"own\"></a>\n";
    assert_eq!(
        anchors(text),
        [
            "a-b",
            "public-main--already-delivered",
            "v080",
            "v080-1",
            "own"
        ]
    );
}

#[test]
fn a_changed_historical_count_is_refused() {
    let status = read_file("CANDIDATE-STATUS.md").unwrap();
    let platform = read_file("docs/PLATFORM-REVIEW.md").unwrap();
    let line = status
        .lines()
        .find(|l| l.starts_with("LINUX_WORKSPACE_TESTS_AT_71142A2="))
        .unwrap();
    let changed = status.replace(line, "LINUX_WORKSPACE_TESTS_AT_71142A2=306_PASS");
    assert!(historical_count(&changed, &platform).is_err());
    let gone = status.replace(line, "");
    assert!(historical_count(&gone, &platform).is_err());
    let moved = platform.replace("merged main `71142a2`", "merged main");
    assert!(historical_count(&status, &moved).is_err());
}
