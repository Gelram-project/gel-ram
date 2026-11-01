// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Historical campaign provenance is distinct from the current source manifest.
use std::{fs, path::Path};

const MANIFEST: &str = "docs/evidence-collection/MEASURED-SOURCES.sha256";
const PIN: &str = "f01bf31d8074d7d89a2c41ba2d6475dcb38696583d9d2ddabe1909f6eb280a52";
const LOCK: &str = "docs/evidence-collection/Cargo.lock.measured.txt";
/// Examples whose measured bytes are kept beside the manifest since the source
/// header was put in front of the current files: (current file, measured copy).
/// The current file must be exactly `gel_core::SOURCE_HEADER` followed by the
/// measured copy, so the example still builds the measured code.
const HEADED: &[(&str, &str)] = &[
    (
        "crates/gel-source/examples/collection_campaign.rs",
        "docs/evidence-collection/collection_campaign.measured.rs.txt",
    ),
    (
        "crates/gel-source/examples/collection_recheck.rs",
        "docs/evidence-collection/collection_recheck.measured.rs.txt",
    ),
    (
        "crates/gel-cli/examples/quantization_matrix.rs",
        "docs/evidence-collection/quantization_matrix.measured.rs.txt",
    ),
];
fn measured_path(name: &str) -> &str {
    if let Some((_, copy)) = HEADED.iter().find(|(current, _)| *current == name) {
        return copy;
    }
    match name {
        "Cargo.lock" => LOCK,
        "crates/gel-source/src/bundle.rs" => "docs/evidence-collection/bundle.measured.rs.txt",
        "crates/gel-source/src/collection.rs" => {
            "docs/evidence-collection/collection.measured.rs.txt"
        }
        "crates/gel-source/src/document.rs" => "docs/evidence-collection/document.measured.rs.txt",
        other => other,
    }
}

pub fn verify(root: &Path) -> Result<(), String> {
    let manifest = fs::read(root.join(MANIFEST)).map_err(|e| e.to_string())?;
    if gel_source::hex(&gel_source::digest(&manifest)) != PIN {
        return Err("historical measurement manifest changed".into());
    }
    for line in std::str::from_utf8(&manifest)
        .map_err(|e| e.to_string())?
        .lines()
    {
        let (expected, name) = line.split_once("  ").ok_or("invalid source pin")?;
        let path = root.join(measured_path(name));
        let bytes = fs::read(&path).map_err(|e| format!("{name}: {e}"))?;
        if gel_source::hex(&gel_source::digest(&bytes)) != expected {
            return Err(format!("historical measurement source mismatch: {name}"));
        }
    }
    for (current, copy) in HEADED {
        let measured = fs::read(root.join(copy)).map_err(|e| format!("{copy}: {e}"))?;
        let now = fs::read(root.join(current)).map_err(|e| format!("{current}: {e}"))?;
        if now != [gel_core::SOURCE_HEADER.as_bytes(), &measured].concat() {
            return Err(format!(
                "current example is not the source header and the measured bytes: {current}"
            ));
        }
    }
    println!("MEASURED_SOURCES_R1=PASS");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn published_sources_and_historical_lock_match() {
        verify(crate::workspace_root().unwrap()).unwrap();
    }

    /// A copy of every file `verify` reads, in a new directory.
    fn copy_of_the_read_files() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "gel-measured-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = crate::workspace_root().unwrap();
        let manifest = fs::read_to_string(source.join(MANIFEST)).unwrap();
        let mut files: Vec<&str> = vec![MANIFEST];
        for line in manifest.lines() {
            let (_, name) = line.split_once("  ").unwrap();
            files.push(measured_path(name));
        }
        files.extend(HEADED.iter().map(|(current, _)| *current));
        for file in files {
            fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
            fs::copy(source.join(file), root.join(file)).unwrap();
        }
        root
    }

    #[test]
    fn rejects_changed_manifest_and_current_lock_substitution() {
        let root = copy_of_the_read_files();
        verify(&root).unwrap();
        let source = crate::workspace_root().unwrap();
        fs::write(root.join(MANIFEST), b"forged").unwrap();
        assert!(verify(&root).is_err());
        fs::copy(source.join(MANIFEST), root.join(MANIFEST)).unwrap();
        verify(&root).unwrap();
        fs::copy(source.join("Cargo.lock"), root.join(LOCK)).unwrap();
        assert!(verify(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn one_changed_byte_of_a_current_example_fails() {
        let root = copy_of_the_read_files();
        verify(&root).unwrap();
        for (current, _) in HEADED {
            let path = root.join(current);
            let good = fs::read(&path).unwrap();
            // A byte in the header, one in the measured code, and the last byte.
            let header = gel_core::SOURCE_HEADER.len();
            for at in [3, header + good[header..].len() / 2, good.len() - 1] {
                let mut bad = good.clone();
                bad[at] ^= 0x01;
                fs::write(&path, &bad).unwrap();
                let error = verify(&root).unwrap_err();
                assert!(error.ends_with(current), "{current} byte {at}: {error}");
            }
            // One byte appended, and the measured bytes without the header.
            for bad in [[good.as_slice(), b"\n"].concat(), good[header..].to_vec()] {
                fs::write(&path, &bad).unwrap();
                assert!(verify(&root).is_err(), "{current}");
            }
            fs::write(&path, &good).unwrap();
            verify(&root).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
}
