//! A digest that commits to the exact contents of a directory without
//! publishing them. The manifest stays with its owner and only its SHA-256 is
//! published; anyone later given the manifest and the files can check that
//! they are the ones committed to (docs/COMMITMENTS.md).
use std::fs;
use std::io::Read;
use std::path::Path;

const HEADER: &str = "GEL_COMMITMENT_1\n";

#[derive(Debug, Default, PartialEq, Eq)]
struct Stats {
    files: u64,
    links: u64,
    bytes: u64,
}

/// Counts the bytes actually read, so the recorded length always matches the
/// hashed bytes even if the file changes while it is read.
struct Counting<R> {
    inner: R,
    count: u64,
}
impl<R: Read> Read for Counting<R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(out)?;
        self.count += n as u64;
        Ok(n)
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', '\n', '\r'])
}

fn collect(
    dir: &Path,
    prefix: &str,
    excludes: &[String],
    out: &mut Vec<(String, String)>,
    stats: &mut Stats,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("name is not UTF-8: {}", path.display()))?;
        if !valid_name(&name) {
            return Err(format!("unsupported name: {}", path.display()));
        }
        if excludes.contains(&name) {
            continue;
        }
        let rel = format!("{prefix}{name}");
        let kind = fs::symlink_metadata(&path)
            .map_err(|e| format!("{rel}: {e}"))?
            .file_type();
        if kind.is_symlink() {
            let target = fs::read_link(&path).map_err(|e| format!("{rel}: {e}"))?;
            let target = target
                .to_str()
                .ok_or_else(|| format!("link target is not UTF-8: {rel}"))?;
            let hash = gel_source::hex(&gel_source::digest(target.as_bytes()));
            out.push((rel.clone(), format!("link {hash} {rel}\n")));
            stats.links += 1;
        } else if kind.is_dir() {
            collect(&path, &format!("{rel}/"), excludes, out, stats)?;
        } else if kind.is_file() {
            let file = fs::File::open(&path).map_err(|e| format!("{rel}: {e}"))?;
            let mut reader = Counting {
                inner: file,
                count: 0,
            };
            let hash = gel_source::digest_reader(&mut reader).map_err(|e| format!("{rel}: {e}"))?;
            let len = reader.count;
            out.push((
                rel.clone(),
                format!("file {} {len} {rel}\n", gel_source::hex(&hash)),
            ));
            stats.files += 1;
            stats.bytes += len;
        } else {
            return Err(format!("unsupported file type: {rel}"));
        }
    }
    Ok(())
}

/// The manifest of `dir`: the header, one `exclude` line per excluded name,
/// then one line per file or link in byte order of the relative path.
fn manifest(dir: &Path, excludes: &[String]) -> Result<(Vec<u8>, Stats), String> {
    let mut excludes = excludes.to_vec();
    excludes.sort();
    excludes.dedup();
    if let Some(bad) = excludes.iter().find(|x| !valid_name(x)) {
        return Err(format!("invalid --exclude name: {bad:?}"));
    }
    let mut entries = Vec::new();
    let mut stats = Stats::default();
    collect(dir, "", &excludes, &mut entries, &mut stats)?;
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut text = String::from(HEADER);
    for name in &excludes {
        text.push_str(&format!("exclude {name}\n"));
    }
    for (_, line) in entries {
        text.push_str(&line);
    }
    Ok((text.into_bytes(), stats))
}

/// The excluded names recorded in a manifest.
fn recorded_excludes(manifest: &str) -> Result<Vec<String>, String> {
    let body = manifest
        .strip_prefix(HEADER)
        .ok_or("not a GEL_COMMITMENT_1 manifest")?;
    Ok(body
        .lines()
        .map_while(|line| line.strip_prefix("exclude "))
        .map(String::from)
        .collect())
}

/// The path of the first line where two manifests differ.
fn first_difference(expected: &str, actual: &str) -> String {
    let path = |line: &str| match line.split(' ').next() {
        Some("file") => line.splitn(4, ' ').nth(3).unwrap_or(line).to_string(),
        Some("link") => line.splitn(3, ' ').nth(2).unwrap_or(line).to_string(),
        _ => line.to_string(),
    };
    let (mut a, mut b) = (expected.lines(), actual.lines());
    loop {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) if x == y => {}
            (Some(x), Some(y)) => {
                let (px, py) = (path(x), path(y));
                return if px.as_bytes() <= py.as_bytes() {
                    px
                } else {
                    py
                };
            }
            (Some(x), None) | (None, Some(x)) => return path(x),
            (None, None) => return "(none)".into(),
        }
    }
}

/// `commitment DIR MANIFEST_OUT [--exclude NAME]...`
pub fn create(args: &[String]) -> Result<(), String> {
    let usage = "usage: commitment DIR MANIFEST_OUT [--exclude NAME]...";
    let (dir, out) = match args {
        [dir, out, ..] => (Path::new(dir), Path::new(out)),
        _ => return Err(usage.into()),
    };
    let mut excludes = Vec::new();
    let mut rest = args[2..].iter();
    while let Some(flag) = rest.next() {
        match (flag.as_str(), rest.next()) {
            ("--exclude", Some(name)) => excludes.push(name.clone()),
            _ => return Err(usage.into()),
        }
    }
    let root = fs::canonicalize(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let parent = match out.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let parent = fs::canonicalize(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    if parent.starts_with(&root) {
        return Err("the manifest must be written outside the directory it describes".into());
    }
    let (bytes, stats) = manifest(&root, &excludes)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    std::io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    println!(
        "COMMITMENT files={} links={} bytes={} sha256={}",
        stats.files,
        stats.links,
        stats.bytes,
        gel_source::hex(&gel_source::digest(&bytes))
    );
    Ok(())
}

/// Compares a manifest with a fresh one of `dir`; the manifest's SHA-256 on success.
fn check_manifest(recorded: &[u8], dir: &Path) -> Result<String, String> {
    let text = std::str::from_utf8(recorded).map_err(|_| "manifest is not UTF-8")?;
    let excludes = recorded_excludes(text)?;
    let (fresh, _) = manifest(dir, &excludes)?;
    if fresh == recorded {
        Ok(gel_source::hex(&gel_source::digest(recorded)))
    } else {
        let fresh = String::from_utf8_lossy(&fresh);
        Err(format!("first={}", first_difference(text, &fresh)))
    }
}

/// `commitment-check MANIFEST DIR`
pub fn check(args: &[String]) -> Result<(), String> {
    let [manifest_path, dir] = args else {
        return Err("usage: commitment-check MANIFEST DIR".into());
    };
    let recorded = fs::read(manifest_path).map_err(|e| format!("{manifest_path}: {e}"))?;
    match check_manifest(&recorded, Path::new(dir)) {
        Ok(hash) => {
            println!("COMMITMENT_CHECK=PASS sha256={hash}");
            Ok(())
        }
        Err(why) => {
            println!("COMMITMENT_CHECK=FAIL {why}");
            Err("commitment check failed".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn scratch(tag: &str) -> std::path::PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "gel-commitment-{}-{tag}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(root: &Path, rel: &str, bytes: &[u8]) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn a_small_tree_gives_the_independently_written_manifest() {
        let root = scratch("known");
        put(&root, "b.txt", b"abc");
        put(&root, "a/empty", b"");
        put(&root, "a/z", b"z\n");
        let (bytes, stats) = manifest(&root, &[]).unwrap();
        // File digests from GNU coreutils sha256sum, not from this crate.
        let expected = "GEL_COMMITMENT_1\n\
            file e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 0 a/empty\n\
            file c865f6c5ab8d1b0bcd383a5e1e3879d22681c96bf462c269b7581d523fbe70ab 2 a/z\n\
            file ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad 3 b.txt\n";
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
        assert_eq!(
            stats,
            Stats {
                files: 3,
                links: 0,
                bytes: 5
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn creation_order_does_not_change_the_manifest() {
        let names = ["x/1", "x/2", "y", "Z", "x/10", "ż"];
        let a = scratch("order-a");
        let b = scratch("order-b");
        for n in names {
            put(&a, n, n.as_bytes());
        }
        for n in names.iter().rev() {
            put(&b, n, n.as_bytes());
        }
        assert_eq!(manifest(&a, &[]).unwrap(), manifest(&b, &[]).unwrap());
        fs::remove_dir_all(a).unwrap();
        fs::remove_dir_all(b).unwrap();
    }

    #[test]
    fn every_change_fails_the_check_and_none_passes_it() {
        let root = scratch("change");
        put(&root, "keep/a", b"one");
        put(&root, "keep/b", &vec![7; 200_000]);
        put(&root, "target/build", b"ignored");
        let excludes = ["target".to_string()];
        let (recorded, _) = manifest(&root, &excludes).unwrap();
        assert!(check_manifest(&recorded, &root).is_ok());
        put(&root, "target/build", b"changed inside an excluded name");
        assert!(check_manifest(&recorded, &root).is_ok());

        let mut flipped = vec![7; 200_000];
        flipped[123_456] = 8;
        put(&root, "keep/b", &flipped);
        assert_eq!(check_manifest(&recorded, &root), Err("first=keep/b".into()));
        put(&root, "keep/b", &vec![7; 200_000]);
        put(&root, "keep/c", b"");
        assert_eq!(check_manifest(&recorded, &root), Err("first=keep/c".into()));
        fs::remove_file(root.join("keep/c")).unwrap();
        fs::remove_file(root.join("keep/a")).unwrap();
        assert_eq!(check_manifest(&recorded, &root), Err("first=keep/a".into()));
        put(&root, "keep/a", b"one");
        assert!(check_manifest(&recorded, &root).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_link_is_recorded_by_its_target_and_not_followed() {
        let root = scratch("link");
        put(&root, "real", b"data");
        std::os::unix::fs::symlink("real", root.join("alias")).unwrap();
        let (recorded, stats) = manifest(&root, &[]).unwrap();
        assert_eq!((stats.files, stats.links), (1, 1));
        fs::remove_file(root.join("alias")).unwrap();
        std::os::unix::fs::symlink("elsewhere", root.join("alias")).unwrap();
        assert_eq!(check_manifest(&recorded, &root), Err("first=alias".into()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bad_input_is_refused() {
        let root = scratch("bad");
        assert!(manifest(&root, &["a/b".into()]).is_err());
        assert!(manifest(&root, &["".into()]).is_err());
        assert!(check_manifest(b"OTHER\n", &root).is_err());
        let inside = root.join("m.txt");
        let args = [root.display().to_string(), inside.display().to_string()];
        assert!(create(&args).is_err());
        assert!(!inside.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
