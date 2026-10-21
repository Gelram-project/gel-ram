//! `speed-convention`: file names and output grammar of the timing examples
//! (`crates/*/examples/speed_*.rs`) and identity programs
//! (`crates/*/examples/identity_*.rs`), as described in `docs/SPEED.md`.
//!
//! The check reads source text only; it builds and runs nothing. It proves
//! that every such example prints through the shared helpers in `tools/`
//! (whose format strings it also checks) and never prints the grammar lines
//! itself. It does not prove that an example measures what it says.
use std::{collections::BTreeMap, fs, path::Path};

const SPEED_SUPPORT: &str = "tools/speed_support.rs";
const IDENTITY_SUPPORT: &str = "tools/identity_support.rs";
const SPEED_DOC: &str = "docs/SPEED.md";
const SPEED_INCLUDE: &str = "#[path = \"../../../tools/speed_support.rs\"]";
const IDENTITY_INCLUDE: &str = "#[path = \"../../../tools/identity_support.rs\"]";

/// Format strings that make up the grammar; each must appear in its helper.
const SPEED_FORMATS: &[&str] = &[
    "\"SPEED_SETUP example={example} runs={runs}\"",
    "\"SPEED name={name} run={run} ns={ns}\"",
    "\"SPEED_CHECK sha256={}\"",
    "\"SPEED_FAIL reason={reason}\"",
    "\"usage: {stem} {usage}\"",
];
const IDENTITY_FORMATS: &[&str] = &[
    "\"IDENTITY name={} cases={} equal={}\"",
    "\"IDENTITY_VERDICT={}\"",
];

/// Text an example must not contain: grammar lines printed around the helper,
/// a private digest copy, or argument parsing outside the helper.
const SPEED_FORBIDDEN: &[&str] = &[
    "\"SPEED ",
    "\"SPEED_SETUP",
    "\"SPEED_INPUT",
    "\"SPEED_CHECK",
    "\"SPEED_FAIL",
    "\"INPUT ",
    "struct Sha256",
    "fn sha256(",
    "std::env::args()",
];
const IDENTITY_FORBIDDEN: &[&str] = &[
    "\"IDENTITY ",
    "\"IDENTITY name=",
    "\"IDENTITY_VERDICT",
    "struct Tally",
    "struct Sha256",
];

/// `<prefix><topic>` with `<topic>` in `[a-z0-9_]+`, no leading, trailing or
/// doubled `_`, and not starting with a digit (no series numbers).
fn valid_stem(stem: &str, prefix: &str) -> bool {
    let Some(topic) = stem.strip_prefix(prefix) else {
        return false;
    };
    !topic.is_empty()
        && topic
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && !topic.starts_with('_')
        && !topic.ends_with('_')
        && !topic.contains("__")
        && !topic.starts_with(|c: char| c.is_ascii_digit())
}

/// The string literal right after `call(` (white space allowed), if any.
fn first_literal<'a>(text: &'a str, call: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(call) {
        rest = &rest[at + call.len()..];
        let trimmed = rest.trim_start();
        if let Some(body) = trimmed.strip_prefix('"') {
            if let Some(end) = body.find('"') {
                out.push(&body[..end]);
            }
        }
    }
    out
}

fn included(text: &str, include: &str, module: &str) -> bool {
    text.match_indices(include).any(|(at, _)| {
        let after = text[at + include.len()..].trim_start();
        after.starts_with(&format!("mod {module};"))
            || after.starts_with(&format!("pub mod {module};"))
    })
}

fn check_speed(rel: &str, stem: &str, text: &str, bad: &mut Vec<String>) {
    if !valid_stem(stem, "speed_") {
        bad.push(format!("{rel}: name is not speed_<topic>[_paired]"));
    }
    if !included(text, SPEED_INCLUDE, "speed") {
        bad.push(format!(
            "{rel}: does not include {SPEED_SUPPORT} as mod speed"
        ));
    }
    if first_literal(text, "speed::setup(") != [stem] {
        bad.push(format!(
            "{rel}: needs exactly one speed::setup(\"{stem}\", ..)"
        ));
    }
    if first_literal(text, "speed::args(") != [stem] {
        bad.push(format!(
            "{rel}: needs exactly one speed::args(\"{stem}\", ..)"
        ));
    }
    if !text.contains("speed::runs(&args)") {
        bad.push(format!("{rel}: RUNS is not read with speed::runs"));
    }
    if !text.contains("speed::line(") {
        bad.push(format!("{rel}: no speed::line call"));
    }
    let checks =
        text.matches("speed::check(").count() + text.matches("speed::check_digest(").count();
    if checks != 1 {
        bad.push(format!(
            "{rel}: needs exactly one speed::check or speed::check_digest call, found {checks}"
        ));
    }
    for forbidden in SPEED_FORBIDDEN {
        if text.contains(forbidden) {
            bad.push(format!("{rel}: contains {forbidden}"));
        }
    }
}

fn check_identity(rel: &str, stem: &str, text: &str, bad: &mut Vec<String>) {
    if !valid_stem(stem, "identity_") {
        bad.push(format!("{rel}: name is not identity_<topic>"));
    }
    if !included(text, IDENTITY_INCLUDE, "identity") {
        bad.push(format!(
            "{rel}: does not include {IDENTITY_SUPPORT} as mod identity"
        ));
    }
    if !text.contains("pub fn run(") {
        bad.push(format!("{rel}: no pub fn run(scale: Scale) -> Vec<Tally>"));
    }
    if text.matches("identity::report(&run(Scale::Full))").count() != 1 {
        bad.push(format!(
            "{rel}: main must be identity::report(&run(Scale::Full))"
        ));
    }
    for forbidden in IDENTITY_FORBIDDEN {
        if text.contains(forbidden) {
            bad.push(format!("{rel}: contains {forbidden}"));
        }
    }
}

fn read(root: &Path, rel: &str) -> Result<String, String> {
    fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))
}

/// Every finding, sorted; empty means PASS. Also returns the counts of
/// timing examples and identity programs seen.
pub fn findings(root: &Path) -> Result<(Vec<String>, usize, usize), String> {
    let mut bad = Vec::new();
    let speed_support = read(root, SPEED_SUPPORT)?;
    for format in SPEED_FORMATS {
        if !speed_support.contains(format) {
            bad.push(format!("{SPEED_SUPPORT}: missing format {format}"));
        }
    }
    let identity_support = read(root, IDENTITY_SUPPORT)?;
    for format in IDENTITY_FORMATS {
        if !identity_support.contains(format) {
            bad.push(format!("{IDENTITY_SUPPORT}: missing format {format}"));
        }
    }
    let doc = read(root, SPEED_DOC)?;

    let mut crates: Vec<_> = fs::read_dir(root.join("crates"))
        .map_err(|e| format!("crates: {e}"))?
        .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    crates.sort();
    let mut stems: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let (mut speed, mut identity) = (0usize, 0usize);
    for krate in crates {
        let package = krate
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("crate directory name is not UTF-8")?
            .to_owned();
        for (dir, is_test) in [("examples", false), ("tests", true)] {
            let Ok(entries) = fs::read_dir(krate.join(dir)) else {
                continue;
            };
            let mut paths: Vec<_> = entries
                .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
                .collect::<Result<_, _>>()?;
            paths.sort();
            for path in paths {
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                let rel = format!("crates/{package}/{dir}/{name}");
                if path.is_dir() {
                    if !is_test && (name.starts_with("speed") || name.starts_with("identity")) {
                        bad.push(format!(
                            "{rel}: timing and identity examples are single files"
                        ));
                    }
                    if !is_test && name == "support" && path.join("sha256.rs").exists() {
                        bad.push(format!("{rel}/sha256.rs: private digest copy"));
                    }
                    continue;
                }
                let Some(stem) = name.strip_suffix(".rs") else {
                    continue;
                };
                let is_speed = stem.starts_with("speed");
                let is_identity = stem.starts_with("identity");
                if !is_speed && !is_identity {
                    continue;
                }
                let text = read(root, &rel)?;
                if is_test {
                    if is_speed {
                        bad.push(format!("{rel}: timing examples have no test wrapper"));
                        continue;
                    }
                    let target = format!("#[path = \"../examples/{stem}.rs\"]");
                    if !text.contains(&target)
                        || !root
                            .join(format!("crates/{package}/examples/{stem}.rs"))
                            .is_file()
                    {
                        bad.push(format!(
                            "{rel}: a wrapper must include examples/{stem}.rs of the same crate"
                        ));
                    }
                    continue;
                }
                stems.entry(stem.to_owned()).or_default().push(rel.clone());
                if !doc.contains(&format!("`{rel}`")) {
                    bad.push(format!("{rel}: not listed in {SPEED_DOC}"));
                }
                if is_speed {
                    speed += 1;
                    check_speed(&rel, stem, &text, &mut bad);
                } else {
                    identity += 1;
                    check_identity(&rel, stem, &text, &mut bad);
                }
            }
        }
    }
    for (stem, places) in &stems {
        if places.len() > 1 {
            bad.push(format!("{stem}: example name used {} times", places.len()));
        }
    }
    if speed == 0 || identity == 0 {
        bad.push(format!(
            "no timing examples or identity programs found ({speed}, {identity})"
        ));
    }
    bad.sort();
    bad.dedup();
    Ok((bad, speed, identity))
}

pub fn check(root: &Path) -> Result<(), String> {
    let (bad, speed, identity) = findings(root)?;
    if bad.is_empty() {
        println!("SPEED_CONVENTION=PASS speed_examples={speed} identity_programs={identity}");
        Ok(())
    } else {
        for message in &bad {
            eprintln!("{message}");
        }
        Err(format!("SPEED_CONVENTION=FAIL findings={}", bad.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_follow_the_convention() {
        for good in [
            "speed_crc",
            "speed_crc_threads_paired",
            "speed_q8_read",
            "speed_sha256",
        ] {
            assert!(valid_stem(good, "speed_"), "{good}");
        }
        for bad in [
            "speed2_crc",
            "speed_",
            "speed__x",
            "speed_x_",
            "speed_2x",
            "speed_Crc",
            "speed-crc",
            "speedcrc",
        ] {
            assert!(!valid_stem(bad, "speed_"), "{bad}");
        }
        assert!(valid_stem("identity_views", "identity_"));
        assert!(!valid_stem("identity_bez-x", "identity_"));
    }

    #[test]
    fn the_workspace_passes() {
        let (bad, speed, identity) = findings(crate::workspace_root().unwrap()).unwrap();
        assert!(bad.is_empty(), "{bad:?}");
        assert!(speed > 0 && identity > 0);
    }

    #[test]
    fn literal_after_call() {
        let text = "speed::setup(\n  \"speed_x\",\n 1, &[]); speed::setup(\"y\"";
        assert_eq!(first_literal(text, "speed::setup("), ["speed_x", "y"]);
    }
}
