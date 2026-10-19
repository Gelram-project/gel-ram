//! `rebuild-check NEW_DIR [--report-only]`: builds the release package twice,
//! from two copies of the committed tree at different paths, and compares the
//! binaries byte for byte.
//!
//! `git archive HEAD` writes the committed tree (not uncommitted changes) to
//! NEW_DIR/source.tar; it is unpacked into two directories whose paths differ in
//! name and length, and `package-binaries` of each copy builds its package with
//! the release recipe (path remapping, its own target directory). The result is
//! `REBUILD=IDENTICAL` or `REBUILD=DIFFERENT`, with the SHA-256 of each binary of
//! both builds. Without `--report-only` a difference is an error.
use gel_source::{digest, hex};
use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const BINARIES: &[&str] = &["gel-evidence", "gel-backup", "gel-live-lab"];

fn run_in(dir: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "{program} {}: {}",
            args.join(" "),
            err.lines().last().unwrap_or("failed")
        ));
    }
    Ok(())
}

/// The one package directory a `package-binaries` run created.
fn package_dir(parent: &Path) -> Result<PathBuf, String> {
    let mut found: Vec<PathBuf> = fs::read_dir(parent)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && !p.to_string_lossy().ends_with("-smoke"))
        .collect();
    match found.len() {
        1 => Ok(found.remove(0)),
        n => Err(format!("expected one package directory, found {n}")),
    }
}

/// (name, SHA-256 of the first build, of the second) for every binary; a
/// missing or empty binary is an error, never a match.
fn compare(a: &Path, b: &Path) -> Result<Vec<(String, String, String)>, String> {
    let mut rows = Vec::new();
    for name in BINARIES {
        let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
        let read = |dir: &Path| -> Result<String, String> {
            let bytes = fs::read(dir.join(&file)).map_err(|e| format!("{file}: {e}"))?;
            if bytes.is_empty() {
                return Err(format!("{file}: empty"));
            }
            Ok(hex(&digest(&bytes)))
        };
        rows.push((file.clone(), read(a)?, read(b)?));
    }
    Ok(rows)
}

/// Printable runs of at least eight ASCII characters.
fn strings(bytes: &[u8]) -> std::collections::BTreeSet<&[u8]> {
    bytes
        .split(|b| !(0x20..0x7f).contains(b))
        .filter(|s| s.len() >= 8)
        .collect()
}

/// Where two builds of one binary differ: sizes, how many bytes differ (equal
/// sizes only), the first differing offset, and up to three printable strings
/// that only one of them contains.
fn difference(name: &str, a: &[u8], b: &[u8]) -> String {
    let first = a.iter().zip(b).position(|(x, y)| x != y);
    let count = if a.len() == b.len() {
        a.iter().zip(b).filter(|(x, y)| x != y).count().to_string()
    } else {
        "sizes differ".into()
    };
    let (sa, sb) = (strings(a), strings(b));
    let only = |x: &std::collections::BTreeSet<&[u8]>, y: &std::collections::BTreeSet<&[u8]>| {
        x.difference(y)
            .take(3)
            .map(|s| String::from_utf8_lossy(&s[..s.len().min(120)]).into_owned())
            .collect::<Vec<_>>()
            .join(" | ")
    };
    format!(
        "REBUILD_DIFF {name} sizes={}/{} differing_bytes={count} first_offset={} only_first=[{}] only_second=[{}]\n",
        a.len(),
        b.len(),
        first.map_or("-".into(), |o| o.to_string()),
        only(&sa, &sb),
        only(&sb, &sa)
    )
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage = "usage: cargo run --locked --offline -p xtask -- rebuild-check NEW_DIR_OUTSIDE_CHECKOUT [--report-only]";
    let (dir, report_only) = match args {
        [dir] => (dir, false),
        [dir, flag] if flag == "--report-only" => (dir, true),
        _ => return Err(usage.into()),
    };
    // Absolute, not canonical: on Windows a canonical path is a verbatim
    // `\\?\` path, which git and tar do not open.
    let root = std::path::absolute(super::workspace_root()?).map_err(|e| e.to_string())?;
    let out = std::path::absolute(dir).map_err(|e| e.to_string())?;
    if out.starts_with(&root) {
        return Err("output directory must be outside the source checkout".into());
    }
    fs::create_dir(&out).map_err(|e| format!("new output directory required: {e}"))?;
    let tar = out.join("source.tar");
    run_in(
        &root,
        "git",
        &[
            "archive",
            "--format=tar",
            "-o",
            &tar.to_string_lossy(),
            "HEAD",
        ],
    )?;
    let copies = [
        out.join("a"),
        out.join("a-second-copy-with-a-longer-path").join("src"),
    ];
    let mut packages = Vec::new();
    for copy in &copies {
        fs::create_dir_all(copy).map_err(|e| e.to_string())?;
        run_in(copy, "tar", &["-xf", &tar.to_string_lossy()])?;
        let parent = copy.with_extension("packages");
        run_in(
            copy,
            "cargo",
            &[
                "run",
                "--locked",
                "--offline",
                "-q",
                "-p",
                "xtask",
                "--",
                "package-binaries",
                &parent.to_string_lossy(),
            ],
        )?;
        packages.push(package_dir(&parent)?);
    }
    let rows = compare(&packages[0], &packages[1])?;
    let mut text = String::new();
    let differing = rows.iter().filter(|(_, a, b)| a != b).count();
    for (name, a, b) in &rows {
        let _ = writeln!(
            text,
            "REBUILD_FILE {name} first={a} second={b} same={}",
            if a == b { "yes" } else { "no" }
        );
        if a != b {
            let read = |p: &PathBuf| fs::read(p.join(name)).map_err(|e| e.to_string());
            text.push_str(&difference(
                name,
                &read(&packages[0])?,
                &read(&packages[1])?,
            ));
        }
    }
    let verdict = if differing == 0 {
        "IDENTICAL"
    } else {
        "DIFFERENT"
    };
    let _ = writeln!(
        text,
        "REBUILD={verdict} files={} differing={differing} target={}-{}",
        rows.len(),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    fs::write(out.join("REBUILD.txt"), &text).map_err(|e| e.to_string())?;
    print!("{text}");
    if differing > 0 && !report_only {
        return Err(format!("REBUILD=DIFFERENT differing={differing}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_empty_binaries_are_errors_and_equal_bytes_match() {
        let base = std::env::temp_dir().join(format!(
            "gel-rebuild-check-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let (a, b) = (base.join("a"), base.join("b"));
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        // Nothing built: an error, not "identical".
        assert!(compare(&a, &b).is_err());
        for name in BINARIES {
            let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
            fs::write(a.join(&file), b"same bytes").unwrap();
            fs::write(b.join(&file), b"same bytes").unwrap();
        }
        let rows = compare(&a, &b).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|(_, x, y)| x == y));
        // One changed byte is a difference.
        let last = format!("gel-live-lab{}", std::env::consts::EXE_SUFFIX);
        fs::write(b.join(&last), b"same bytez").unwrap();
        assert_eq!(
            compare(&a, &b)
                .unwrap()
                .iter()
                .filter(|(_, x, y)| x != y)
                .count(),
            1
        );
        // An empty binary is an error.
        fs::write(b.join(&last), b"").unwrap();
        assert!(compare(&a, &b).is_err());
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_difference_names_its_offset_and_the_strings_only_one_build_has() {
        let a = b"\x7fELF\x00/build/one/target/x\x00same text here\x00";
        let b = b"\x7fELF\x00/build/two/target/x\x00same text here\x00";
        let note = difference("bin", a, b);
        assert!(
            note.contains("sizes=40/40 differing_bytes=3 first_offset=12"),
            "{note}"
        );
        assert!(note.contains("only_first=[/build/one/target/x]"), "{note}");
        assert!(note.contains("only_second=[/build/two/target/x]"), "{note}");
        assert!(difference("bin", b"short", b"longer").contains("differing_bytes=sizes differ"));
    }
}
