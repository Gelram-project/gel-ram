//! Bounded source-only snapshots; never licenses, signs or publishes a release.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::Path,
};

const MANIFEST: &str = "SOURCE-SHA256SUMS.txt";
const MAX_FILE: usize = 8 * 1024 * 1024;
const MAX_TOTAL: usize = 64 * 1024 * 1024;
const MAX_FILES: usize = 4096;
type Snapshot = BTreeMap<String, Vec<u8>>;

fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 240
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-/".contains(&b))
        && s.split('/').all(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_lowercase();
            !part.is_empty()
                && !matches!(part, "." | "..")
                && !part.ends_with('.')
                && !matches!(part.to_ascii_lowercase().as_str(), ".git" | "target")
                && !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
                && !(stem.len() == 4
                    && (stem.starts_with("com") || stem.starts_with("lpt"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}

fn parse_manifest(bytes: &[u8]) -> Result<BTreeMap<String, String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "manifest is not UTF-8")?;
    if !text.ends_with('\n') || text.contains('\r') {
        return Err("manifest requires LF-terminated records".into());
    }
    let mut entries = BTreeMap::new();
    let mut portable_names = BTreeSet::new();
    for line in text.lines() {
        let (hash, name) = line.split_once("  ").ok_or("invalid manifest record")?;
        if !valid_hash(hash) || !valid_path(name) || name.eq_ignore_ascii_case(MANIFEST) {
            return Err("invalid manifest hash or path".into());
        }
        if !portable_names.insert(name.to_ascii_lowercase()) {
            return Err("duplicate or case-alias manifest path".into());
        }
        entries.insert(name.to_owned(), hash.to_owned());
        if entries.len() >= MAX_FILES {
            return Err("too many source files".into());
        }
    }
    if entries.is_empty() {
        return Err("empty manifest".into());
    }
    Ok(entries)
}

fn inventory(
    root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut BTreeSet<String>,
) -> Result<(), String> {
    if depth > 16 {
        return Err("source tree is too deep".into());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        // Build products and Git metadata are NEVER copied, even if tracked.
        if dir == root
            && matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some(".git" | "target")
            )
        {
            continue;
        }
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() || !(meta.is_file() || meta.is_dir()) {
            return Err("symlink or special file in source tree".into());
        }
        if meta.is_dir() {
            inventory(root, &path, depth + 1, out)?;
        } else {
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("non-UTF-8 source path")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            if !valid_path(&name) {
                return Err("nonportable source path".into());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if meta.permissions().mode() & 0o111 != 0 {
                    return Err("executable permission on source file".into());
                }
            }
            out.insert(name);
            if out.len() > MAX_FILES {
                return Err("too many source files".into());
            }
        }
    }
    Ok(())
}

fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit as u64 {
        return Err("invalid source file type or size".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("source file grew beyond limit".into());
    }
    Ok(bytes)
}

fn snapshot(root: &Path, expected: &str) -> Result<Snapshot, String> {
    if !valid_hash(expected) {
        return Err("expected a reviewed SHA-256 manifest pin".into());
    }
    let manifest = bounded_read(&root.join(MANIFEST), 1024 * 1024)?;
    if gel_source::hex(&gel_source::digest(&manifest)) != expected {
        return Err("manifest pin mismatch".into());
    }
    let entries = parse_manifest(&manifest)?;
    let mut actual = BTreeSet::new();
    inventory(root, root, 0, &mut actual)?;
    let mut expected_names: BTreeSet<_> = entries.keys().cloned().collect();
    expected_names.insert(MANIFEST.into());
    if actual != expected_names {
        // No unknown contents or potentially private filenames in logs.
        return Err("source inventory differs from manifest: missing or extra files".into());
    }
    let mut total = manifest.len();
    let mut files = BTreeMap::new();
    files.insert(MANIFEST.into(), manifest);
    for (name, hash) in entries {
        let bytes = bounded_read(&root.join(&name), MAX_FILE.min(MAX_TOTAL - total))?;
        if gel_source::hex(&gel_source::digest(&bytes)) != hash {
            return Err(format!("source digest mismatch: {name}"));
        }
        total += bytes.len();
        files.insert(name, bytes);
    }
    Ok(files)
}

pub(super) fn validated_identity(root: &Path, pin: &str) -> Result<(usize, usize), String> {
    let files = snapshot(root, pin)?;
    Ok((files.len(), files.values().map(Vec::len).sum()))
}

pub fn audit(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: source-audit REVIEWED_MANIFEST_SHA256".into());
    }
    let files = snapshot(super::workspace_root()?, &args[0])?;
    super::rust_only_at(super::workspace_root()?)?;
    println!(
        "SOURCE_AUDIT=PASS files={} bytes={} manifest_sha256={}",
        files.len(),
        files.values().map(Vec::len).sum::<usize>(),
        args[0]
    );
    println!("PUBLICATION_APPROVED=NO; source integrity is not legal or security approval");
    Ok(())
}

fn write_snapshot(files: &Snapshot, output: &Path) -> Result<(), String> {
    // New directory only. On failure retain partial output WITHOUT completion status.
    fs::create_dir(output).map_err(|e| format!("new output directory required: {e}"))?;
    let source = output.join("source");
    fs::create_dir(&source).map_err(|e| e.to_string())?;
    for (name, bytes) in files {
        let dest = source.join(name);
        fs::create_dir_all(dest.parent().ok_or("missing parent")?).map_err(|e| e.to_string())?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)
            .map_err(|e| e.to_string())?;
        f.write_all(bytes).map_err(|e| e.to_string())?;
    }
    let pin = gel_source::hex(&gel_source::digest(
        files.get(MANIFEST).ok_or("missing manifest")?,
    ));
    snapshot(&source, &pin)?;
    let mut status = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("STATUS.txt"))
        .map_err(|e| e.to_string())?;
    write!(status, "SOURCE_BUNDLE=CHECKED\nPUBLICATION_APPROVED=NO\nMANIFEST_SHA256={pin}\nReview-only source snapshot; existing licensing unchanged.\nNot a signed release, legal approval, benchmark certification or secret scan.\n").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn bundle(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err(
            "usage: source-bundle REVIEWED_MANIFEST_SHA256 NEW_DIRECTORY_OUTSIDE_CHECKOUT".into(),
        );
    }
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let requested = Path::new(&args[1]);
    let name = requested
        .file_name()
        .ok_or("missing output directory name")?;
    let parent = requested
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if parent.starts_with(&root) {
        return Err("output must be outside checkout".into());
    }
    let files = snapshot(&root, &args[0])?;
    super::rust_only_at(&root)?;
    write_snapshot(&files, &parent.join(name))?;
    println!("SOURCE_BUNDLE=CHECKED; PUBLICATION_APPROVED=NO");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let p =
                std::env::temp_dir().join(format!("gel-source-audit-{}-{n}", std::process::id()));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn seed(&self) -> String {
            fs::write(self.0.join("README.md"), b"synthetic fixture\n").unwrap();
            let hash = gel_source::hex(&gel_source::digest(b"synthetic fixture\n"));
            let manifest = format!("{hash}  README.md\n");
            fs::write(self.0.join(MANIFEST), &manifest).unwrap();
            gel_source::hex(&gel_source::digest(manifest.as_bytes()))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn only_portable_relative_paths() {
        for s in [
            "../secret",
            "/absolute",
            "a//b",
            "a/./b",
            "a/../b",
            "C:/x",
            "a\\b",
            ".git/config",
            "x/target/x",
            "NUL.txt",
            "com1.rs",
            "a.",
            "a b",
            "ż.txt",
        ] {
            assert!(!valid_path(s), "{s}");
        }
        for s in [".cargo/config.toml", "docs/README.md", "LICENSE"] {
            assert!(valid_path(s));
        }
    }
    #[test]
    fn strict_manifest_format_and_duplicates() {
        let hash = "0".repeat(64);
        for bad in [
            String::new(),
            format!("{hash}  ../x\n"),
            format!("{hash}  README.md"),
            format!("{hash}  README.md\r\n"),
            format!("{hash}  a\n{hash}  A\n"),
            format!("{hash}  {MANIFEST}\n"),
            format!("{hash}  a\n{hash}  a\n"),
            format!("{}  a\n", "g".repeat(64)),
        ] {
            assert!(parse_manifest(bad.as_bytes()).is_err());
        }
    }
    #[test]
    fn changed_missing_extra_files_and_wrong_pin_fail() {
        let f = Fixture::new();
        let pin = f.seed();
        assert!(snapshot(&f.0, &pin).is_ok());
        assert!(snapshot(&f.0, &"0".repeat(64)).is_err());
        fs::write(f.0.join("extra.txt"), "not approved").unwrap();
        assert!(snapshot(&f.0, &pin).is_err());
        fs::remove_file(f.0.join("extra.txt")).unwrap();
        fs::write(f.0.join("README.md"), "changed").unwrap();
        assert!(snapshot(&f.0, &pin).is_err());
        fs::remove_file(f.0.join("README.md")).unwrap();
        assert!(snapshot(&f.0, &pin).is_err());
    }
    #[test]
    fn build_and_git_metadata_never_copied() {
        let f = Fixture::new();
        let pin = f.seed();
        fs::create_dir(f.0.join("target")).unwrap();
        fs::write(f.0.join("target/private.txt"), "synthetic excluded fixture").unwrap();
        fs::write(f.0.join(".git"), "synthetic metadata").unwrap();
        let files = snapshot(&f.0, &pin).unwrap();
        assert_eq!(files.len(), 2);
        let out = f.0.join("output");
        write_snapshot(&files, &out).unwrap();
        assert!(out.join("source/README.md").exists());
        assert!(!out.join("source/target").exists());
        assert!(!out.join("source/.git").exists());
        assert!(fs::read_to_string(out.join("STATUS.txt"))
            .unwrap()
            .contains("PUBLICATION_APPROVED=NO"));
        assert!(write_snapshot(&files, &out).is_err());
    }
    #[test]
    fn bounded_read_rejects_oversize_file() {
        let f = Fixture::new();
        f.seed();
        assert!(bounded_read(&f.0.join("README.md"), 2).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_file_and_directory_rejected() {
        let f = Fixture::new();
        let pin = f.seed();
        std::os::unix::fs::symlink(f.0.join("README.md"), f.0.join("link.txt")).unwrap();
        assert!(snapshot(&f.0, &pin).is_err());
        fs::remove_file(f.0.join("link.txt")).unwrap();
        std::os::unix::fs::symlink(&f.0, f.0.join("loop")).unwrap();
        assert!(snapshot(&f.0, &pin).is_err());
    }
}
