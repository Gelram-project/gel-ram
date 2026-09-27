//! Binary packages of the Evidence Lab tools, built in CI (docs/BINARIES.md).
//!
//! Builds gel-evidence, gel-backup and gel-live-lab for the host target with
//! build paths remapped and assembles one package directory. The package is
//! refused unless every third-party crate in the target's normal dependency
//! graph is in the reviewed inventory and each of its license files matches the
//! recorded SHA-256, the toolchain's own license files are present, no build
//! path appears in a binary, and the packaged binaries pass a smoke run on this
//! platform. Archiving and provenance attestation are left to the workflow.
use gel_source::{digest, hex};
use std::{
    fmt::Write as _,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const BINARIES: &[&str] = &["gel-evidence", "gel-backup", "gel-live-lab"];
const INVENTORY: &str = "docs/DEPENDENCY-INVENTORY.md";
/// Project terms shipped at the package root under their base names.
const PROJECT_FILES: &[&str] = &[
    "LICENSE",
    "NOTICE",
    "LICENSE-MODE.txt",
    "LICENSING.md",
    "COMMERCIAL-LICENSE.md",
    "THIRD-PARTY-NOTICES.md",
    INVENTORY,
];
/// License files of the statically linked Rust standard library, as shipped
/// with the toolchain in `share/doc/rust`.
const TOOLCHAIN_LICENSES: &[&str] = &["COPYRIGHT", "LICENSE-APACHE", "LICENSE-MIT"];

fn output(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd:?}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|l| l.strip_prefix(key))
        .map(str::trim)
}

fn workspace_version(root: &Path) -> Result<String, String> {
    let toml = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    toml.split("[workspace.package]")
        .nth(1)
        .and_then(|s| field(s, "version = "))
        .map(|v| v.trim_matches('"').to_string())
        .ok_or_else(|| "Cargo.toml: no [workspace.package] version".into())
}

/// `name-version` with its reviewed (SHA-256, file) pairs.
type Reviewed = (String, Vec<(String, String)>);

/// Reviewed license files per `name-version`, from the inventory's hash blocks.
fn inventory(text: &str) -> Result<Vec<Reviewed>, String> {
    let mut out: Vec<Reviewed> = Vec::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            let (name, version) = heading
                .split_once(' ')
                .ok_or_else(|| format!("{INVENTORY}: bad heading {heading}"))?;
            out.push((format!("{name}-{version}"), Vec::new()));
        } else if let (Some((key, files)), Some((hash, path))) =
            (out.last_mut(), line.split_once("  "))
        {
            if let Some(file) = path.strip_prefix(&format!("{key}/")) {
                if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(format!("{INVENTORY}: bad hash for {path}"));
                }
                files.push((hash.to_ascii_lowercase(), file.to_string()));
            }
        }
    }
    if let Some((key, _)) = out.iter().find(|(_, files)| files.is_empty()) {
        return Err(format!("{INVENTORY}: no license file recorded for {key}"));
    }
    Ok(out)
}

/// Third-party `name-version` entries of `cargo tree --prefix none -f {p}`;
/// workspace packages carry their path in parentheses and are skipped.
fn third_party(tree: &str) -> Vec<String> {
    let mut out: Vec<String> = tree
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(" (*)");
            let mut words = line.splitn(3, ' ');
            let name = words.next()?;
            let version = words.next()?.strip_prefix('v')?;
            words.next().is_none().then(|| format!("{name}-{version}"))
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// Build-machine paths that must not survive in a shipped binary, as byte
/// strings in both separator spellings, with a label that is safe to print.
fn leak_needles(paths: &[(&str, &Path)]) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for (label, path) in paths {
        let text = path.to_string_lossy();
        let text = text.trim_start_matches(r"\\?\");
        if text.len() < 4 {
            continue;
        }
        for spelling in [text.replace('\\', "/"), text.replace('/', "\\")] {
            out.push(((*label).to_string(), spelling.into_bytes()));
        }
    }
    for generic in ["/home/", "/Users/", "\\Users\\"] {
        out.push(("user directory".into(), generic.as_bytes().to_vec()));
    }
    out
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn cargo_home() -> Result<PathBuf, String> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".cargo")))
        .ok_or_else(|| "cannot locate CARGO_HOME".into())
}

/// The unpacked registry source of one locked crate.
fn registry_dir(cargo_home: &Path, key: &str) -> Result<PathBuf, String> {
    let src = cargo_home.join("registry").join("src");
    let mut found = Vec::new();
    for index in fs::read_dir(&src).map_err(|e| format!("registry sources: {e}"))? {
        let dir = index.map_err(|e| e.to_string())?.path().join(key);
        if dir.is_dir() {
            found.push(dir);
        }
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!(
            "{key}: not in the local registry; run cargo fetch --locked"
        )),
        _ => Err(format!("{key}: found in several registry indexes")),
    }
}

fn copy_checked(from: &Path, to: &Path, expected: &str) -> Result<(), String> {
    let bytes = fs::read(from).map_err(|e| format!("{}: {e}", to.display()))?;
    if hex(&digest(&bytes)) != expected {
        return Err(format!(
            "{}: SHA-256 differs from the reviewed inventory",
            to.display()
        ));
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(to, bytes).map_err(|e| e.to_string())
}

fn run_with(bin: &Path, args: &[&str], stdin: &str) -> Result<Output, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{args:?}: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(stdin.as_bytes())
        .map_err(|e| e.to_string())?;
    child.wait_with_output().map_err(|e| e.to_string())
}

fn expect(name: &str, out: &Output, code: i32, needle: &str) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(&out.stdout);
    if out.status.code() != Some(code) || !stdout.contains(needle) {
        return Err(format!(
            "smoke {name}: exit {:?}, expected {code} and {needle:?}; stderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

/// Runs the packaged binaries, not the build outputs, in a scratch directory.
fn smoke(pkg: &Path, work: &Path, log: &mut String) -> Result<(), String> {
    let bin = |name: &str| pkg.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let path = |name: &str| work.join(name).to_string_lossy().into_owned();
    let (evidence, backup) = (bin("gel-evidence"), bin("gel-backup"));
    let mut pass = |name: &str| {
        let _ = writeln!(log, "smoke={name} PASS");
    };

    let out = run_with(&evidence, &["--help"], "")?;
    expect("evidence-help", &out, 0, "gel-evidence [--demo | --batch]")?;
    pass("evidence-help");
    let out = run_with(&evidence, &["--demo"], "")?;
    expect("evidence-demo", &out, 0, "GEL_EVIDENCE_DEMO=PASS")?;
    pass("evidence-demo");
    let out = run_with(&bin("gel-live-lab"), &["--demo"], "")?;
    expect("live-lab-demo", &out, 0, "GEL_LIVE_LAB_DEMO=PASS")?;
    pass("live-lab-demo");

    fs::create_dir(work).map_err(|e| format!("smoke directory: {e}"))?;
    fs::write(
        work.join("notes.txt"),
        "RAM is volatile.\nŁódź is a city name.\n",
    )
    .map_err(|e| e.to_string())?;
    let snap = path("notes.gelset");
    let script = format!(
        "add {}\nfind ram is volatile\nfind łódź\nfind invented answer\nsave {snap}\nexit\n",
        path("notes.txt")
    );
    let out = run_with(&evidence, &["--batch"], &script)?;
    expect("batch-find-save", &out, 0, "\"record\":\"summary\"")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let statuses: Vec<&str> = stdout
        .lines()
        .filter_map(|l| l.split("\"status\":\"").nth(1)?.split('"').next())
        .collect();
    if statuses != ["OK", "HIT", "HIT", "UNKNOWN", "OK", "OK"] {
        return Err(format!("smoke batch-find-save: statuses {statuses:?}"));
    }
    let pin = stdout
        .split("\"bundle_sha256\":\"")
        .nth(1)
        .and_then(|s| s.get(..64))
        .ok_or("smoke batch-find-save: no bundle_sha256")?
        .to_string();
    pass("batch-find-save");

    let dir = path("backup");
    let out = run_with(&backup, &["create", &pin, &snap, &dir], "")?;
    expect("backup-create", &out, 0, "BACKUP=CREATED COMPLETE")?;
    let out = run_with(&backup, &["inspect", &pin, &dir], "")?;
    expect("backup-inspect", &out, 0, "BACKUP=COMPLETE")?;
    let restored = path("restored.gelset");
    let out = run_with(&backup, &["restore", &pin, &dir, &restored], "")?;
    expect("backup-restore", &out, 0, "RESTORE=PASS")?;
    let same = fs::read(&snap).map_err(|e| e.to_string())?
        == fs::read(&restored).map_err(|e| e.to_string())?;
    if !same {
        return Err("smoke backup-restore: restored bytes differ".into());
    }
    let wrong = "0".repeat(64);
    let out = run_with(
        &backup,
        &["restore", &wrong, &dir, &path("refused.gelset")],
        "",
    )?;
    expect("backup-wrong-pin", &out, 2, "RESTORE=REFUSED")?;
    if work.join("refused.gelset").exists() {
        return Err("smoke backup-wrong-pin: a refused restore wrote a file".into());
    }
    pass("backup-create-inspect-restore-refuse");
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let [parent] = args else {
        return Err(
            "usage: cargo run -p xtask -- package-binaries NEW_PARENT_DIR_OUTSIDE_CHECKOUT".into(),
        );
    };
    let root = super::workspace_root()?;
    let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
    let parent = Path::new(parent);
    fs::create_dir(parent).map_err(|e| format!("new output directory required: {e}"))?;
    let parent = parent.canonicalize().map_err(|e| e.to_string())?;
    if parent.starts_with(&canonical_root) {
        return Err("output directory must be outside the source checkout".into());
    }

    let rustc = output(Command::new("rustc").arg("-vV"))?;
    let target = field(&rustc, "host: ")
        .ok_or("rustc -vV: no host")?
        .to_string();
    let release = field(&rustc, "release: ")
        .ok_or("rustc -vV: no release")?
        .to_string();
    let name = format!("gel-{}-{target}", workspace_version(root)?);
    let pkg = parent.join(&name);
    fs::create_dir(&pkg).map_err(|e| e.to_string())?;

    let cargo_home = cargo_home()?;
    let home = home().unwrap_or_default();
    // Later mappings take precedence, so the more specific prefixes come last.
    let mut flags = Vec::new();
    for (from, to) in [
        (&home, "~"),
        (&cargo_home, "cargo"),
        (&root.to_path_buf(), "gel"),
    ] {
        if !from.as_os_str().is_empty() {
            flags.push(format!("--remap-path-prefix={}={to}", from.display()));
        }
    }
    let target_dir = root.join("target").join("package-build");
    let status = Command::new("cargo")
        .current_dir(root)
        .args([
            "build",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bins",
            "--target",
            &target,
        ])
        .arg("--target-dir")
        .arg(&target_dir)
        .env_remove("RUSTFLAGS")
        .env("CARGO_ENCODED_RUSTFLAGS", flags.join("\x1f"))
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("release build failed".into());
    }

    let mut info = String::from("FORMAT=GEL_BINARY_PACKAGE_1\n");
    let _ = writeln!(info, "package={name}");
    let _ = writeln!(info, "{}", super::bench_compare::revision(root));
    let _ = writeln!(info, "target={target}");
    let _ = writeln!(info, "rustc={}", rustc.lines().next().unwrap_or(""));
    let _ = writeln!(
        info,
        "cargo={}",
        output(Command::new("cargo").arg("-V"))?.trim()
    );
    info.push_str("profile=release lto=thin codegen-units=1 panic=abort strip=symbols\n");
    info.push_str("path_remapping=home,cargo_home,workspace\n");
    let ci = std::env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true");
    let env = |k: &str| std::env::var(k).unwrap_or_else(|_| "NOT_SET".into());
    if ci {
        let _ = writeln!(
            info,
            "build_context=github-actions runner_os={} image={} image_version={} run_id={}",
            env("RUNNER_OS"),
            env("ImageOS"),
            env("ImageVersion"),
            env("GITHUB_RUN_ID")
        );
    } else {
        info.push_str("build_context=local\n");
    }

    let tree = output(Command::new("cargo").current_dir(root).args([
        "tree",
        "--locked",
        "--offline",
        "-p",
        "gel-live-lab",
        "-e",
        "normal",
        "--target",
        &target,
        "--prefix",
        "none",
        "-f",
        "{p}",
    ]))?;
    let reviewed = inventory(
        &fs::read_to_string(root.join(INVENTORY)).map_err(|e| format!("{INVENTORY}: {e}"))?,
    )?;
    for key in third_party(&tree) {
        let files = reviewed
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, files)| files)
            .ok_or_else(|| format!("{key}: linked but not in the reviewed inventory"))?;
        let src = registry_dir(&cargo_home, &key)?;
        for (hash, file) in files {
            copy_checked(
                &src.join(file),
                &pkg.join("licenses").join(&key).join(file),
                hash,
            )?;
        }
        let _ = writeln!(
            info,
            "crate={key} license_files={} checked_against={INVENTORY}",
            files.len()
        );
    }
    let sysroot = output(Command::new("rustc").args(["--print", "sysroot"]))?;
    let docs = Path::new(sysroot.trim())
        .join("share")
        .join("doc")
        .join("rust");
    for file in TOOLCHAIN_LICENSES {
        let bytes = fs::read(docs.join(file)).map_err(|e| format!("toolchain {file}: {e}"))?;
        let to = pkg.join("licenses").join(format!("rust-{release}"));
        fs::create_dir_all(&to).map_err(|e| e.to_string())?;
        fs::write(to.join(file), bytes).map_err(|e| e.to_string())?;
    }
    let _ = writeln!(info, "statically_linked=Rust standard library {release}, license files in licenses/rust-{release}");
    info.push_str("not_shipped=the target's system C runtime, linked dynamically\n");
    for file in PROJECT_FILES {
        let base = Path::new(file).file_name().ok_or("project file name")?;
        fs::copy(root.join(file), pkg.join(base)).map_err(|e| format!("{file}: {e}"))?;
    }

    let needles = leak_needles(&[
        ("workspace", root),
        ("workspace", &canonical_root),
        ("cargo home", &cargo_home),
        ("home directory", &home),
    ]);
    for bin in BINARIES {
        let file = format!("{bin}{}", std::env::consts::EXE_SUFFIX);
        let built = target_dir.join(&target).join("release").join(&file);
        let bytes = fs::read(&built).map_err(|e| format!("{file}: {e}"))?;
        if let Some((label, _)) = needles.iter().find(|(_, n)| contains(&bytes, n)) {
            return Err(format!("{file}: contains a build-machine path ({label})"));
        }
        fs::write(pkg.join(&file), &bytes).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(pkg.join(&file), fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
        let _ = writeln!(
            info,
            "binary={file} bytes={} sha256={}",
            bytes.len(),
            hex(&digest(&bytes))
        );
    }
    let _ = writeln!(info, "path_scan=PASS patterns={}", needles.len());
    smoke(&pkg, &parent.join(format!("{name}-smoke")), &mut info)?;

    let clean = info.contains("working_tree=clean");
    let eligible = match (ci, clean) {
        (true, true) => "yes".to_string(),
        (false, _) => "no (built outside CI)".to_string(),
        (true, false) => "no (working tree not clean)".to_string(),
    };
    let _ = writeln!(info, "release_eligible={eligible}");

    // SHA256SUMS.txt lists BUILD-INFO.txt from its in-memory bytes and is
    // written first; BUILD-INFO.txt, the release-eligibility marker, is written
    // last, so no failure can leave the marker without a checksum manifest.
    let mut files = Vec::new();
    super::reproduction::hashes(&pkg, &pkg, &mut files)?;
    fs::write(
        pkg.join("SHA256SUMS.txt"),
        checksum_manifest(files, info.as_bytes()),
    )
    .map_err(|e| e.to_string())?;
    fs::write(pkg.join("BUILD-INFO.txt"), &info).map_err(|e| e.to_string())?;
    if let Some(gh) = std::env::var_os("GITHUB_OUTPUT") {
        fs::OpenOptions::new()
            .append(true)
            .open(gh)
            .and_then(|mut f| writeln!(f, "name={name}"))
            .map_err(|e| e.to_string())?;
    }
    println!("PACKAGE=PASS name={name} release_eligible={eligible}");
    Ok(())
}

/// `sha256sum -c` lines for the package files plus BUILD-INFO.txt, which is
/// not on disk yet; sorted by path.
fn checksum_manifest(mut files: Vec<(String, String)>, build_info: &[u8]) -> String {
    files.push((hex(&digest(build_info)), "BUILD-INFO.txt".into()));
    files.sort_by(|a, b| a.1.cmp(&b.1));
    let mut sums = String::new();
    for (hash, file) in &files {
        let _ = writeln!(sums, "{hash}  {file}");
    }
    sums
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_checksum_manifest_covers_build_info_before_it_is_written() {
        let sums = checksum_manifest(vec![("a".repeat(64), "LICENSE".into())], b"x");
        let expected = format!(
            "{}  BUILD-INFO.txt\n{}  LICENSE\n",
            hex(&digest(b"x")),
            "a".repeat(64)
        );
        assert_eq!(sums, expected);
    }

    #[test]
    fn inventory_reads_license_files_and_skips_the_archive() {
        let text = "### a 1.0\n\n```text\n".to_string()
            + &"1".repeat(64)
            + "  a-1.0.crate\n"
            + &"2".repeat(64)
            + "  a-1.0/LICENSE-MIT\n```\n";
        let parsed = inventory(&text).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "a-1.0");
        assert_eq!(parsed[0].1, [("2".repeat(64), "LICENSE-MIT".to_string())]);
        assert!(inventory("### b 2.0\n").is_err());
    }

    #[test]
    fn tree_lines_keep_only_registry_crates() {
        let tree = "gel-live-lab v0.4.0 (/w/crates/gel-live-lab)\nsha2 v0.10.9\ngeneric-array v0.14.7\ngeneric-array v0.14.7 (*)\n";
        assert_eq!(third_party(tree), ["generic-array-0.14.7", "sha2-0.10.9"]);
    }

    #[test]
    fn needles_cover_both_separators_and_ignore_short_paths() {
        let needles = leak_needles(&[("w", Path::new("/a/bc/d")), ("h", Path::new("/"))]);
        assert!(needles.iter().any(|(_, n)| n == b"/a/bc/d"));
        assert!(needles.iter().any(|(_, n)| n == b"\\a\\bc\\d"));
        assert!(!needles.iter().any(|(l, _)| l == "h"));
        assert!(contains(b"xx/a/bc/dyy", b"/a/bc/d"));
        assert!(!contains(b"anything", b""));
    }
}
