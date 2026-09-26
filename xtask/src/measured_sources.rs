//! Historical campaign provenance is distinct from the current source manifest.
use std::{fs, path::Path};

const MANIFEST: &str = "docs/evidence-collection/MEASURED-SOURCES.sha256";
const PIN: &str = "f01bf31d8074d7d89a2c41ba2d6475dcb38696583d9d2ddabe1909f6eb280a52";
const LOCK: &str = "docs/evidence-collection/Cargo.lock.measured.txt";
fn measured_path(name: &str) -> &str {
    match name {
        "Cargo.lock" => LOCK,
        "crates/gel-source/src/collection.rs" => {
            "docs/evidence-collection/collection.measured.rs.txt"
        }
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
    #[test]
    fn rejects_changed_manifest_and_current_lock_substitution() {
        let root = std::env::temp_dir().join(format!(
            "gel-measured-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = crate::workspace_root().unwrap();
        fs::create_dir_all(root.join("docs/evidence-collection")).unwrap();
        fs::write(root.join(MANIFEST), b"forged").unwrap();
        assert!(verify(&root).is_err());
        let manifest = fs::read_to_string(source.join(MANIFEST)).unwrap();
        fs::write(root.join(MANIFEST), &manifest).unwrap();
        for line in manifest.lines() {
            let (_, name) = line.split_once("  ").unwrap();
            let target = measured_path(name);
            fs::create_dir_all(root.join(target).parent().unwrap()).unwrap();
            fs::copy(source.join(target), root.join(target)).unwrap();
        }
        verify(&root).unwrap();
        fs::copy(source.join("Cargo.lock"), root.join(LOCK)).unwrap();
        assert!(verify(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
