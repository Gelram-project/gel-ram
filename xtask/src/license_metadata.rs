//! Narrow, fail-closed check for this workspace's chosen Cargo spelling.
//! Not a general TOML parser or a legal-rights validator.
use std::{fs, path::Path};

fn validate(text: &str, workspace: bool) -> Result<(), String> {
    let section = if workspace {
        "[workspace.package]"
    } else {
        "[package]"
    };
    let expected = if workspace {
        "license-file=\"LICENSE\""
    } else {
        "license-file.workspace=true"
    };
    let mut inside = false;
    let mut found = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            inside = line == section;
        }
        if inside {
            if let Some((key, _)) = line.split_once('=') {
                let key: String = key.chars().filter(|c| !c.is_whitespace()).collect();
                if key.trim_matches(['\'', '"']).starts_with("license") {
                    found.push(
                        line.chars()
                            .filter(|c| !c.is_whitespace())
                            .collect::<String>(),
                    );
                }
            }
        }
    }
    if found != [expected] {
        return Err("expected exactly one reviewed license-file setting".into());
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    super::walk(root, &mut files).map_err(|e| e.to_string())?;
    let mut count = 0;
    for file in files
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "Cargo.toml"))
    {
        if fs::symlink_metadata(&file)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("symlinked Cargo manifest".into());
        }
        let is_root = file == root.join("Cargo.toml");
        validate(
            &fs::read_to_string(&file).map_err(|e| e.to_string())?,
            is_root,
        )
        .map_err(|e| format!("{}: {e}", file.strip_prefix(root).unwrap().display()))?;
        count += 1;
    }
    if count != 13 {
        return Err(format!(
            "unexpected manifest count {count}; review workspace changes"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;
    #[test]
    fn root_license_file() {
        assert!(validate("[workspace.package]\nlicense-file = \"LICENSE\"\n", true).is_ok());
    }
    #[test]
    fn member_inherits_license_file() {
        assert!(validate("[package]\nlicense-file.workspace = true\n", false).is_ok());
    }
    #[test]
    fn old_identifier_is_rejected() {
        assert!(validate(
            "[workspace.package]\nlicense = \"PolyForm-Noncommercial-1.0.0\"\n",
            true
        )
        .is_err());
    }
    #[test]
    fn additional_license_is_rejected() {
        assert!(validate(
            "[package]\nlicense-file.workspace=true\nlicense=\"MIT\"\n",
            false
        )
        .is_err());
    }
    #[test]
    fn wrong_location_is_rejected() {
        assert!(validate("[package]\nlicense-file = \"LICENSE\"\n", false).is_err());
    }
    #[test]
    fn duplicate_is_rejected() {
        assert!(validate(
            "[package]\nlicense-file.workspace=true\nlicense-file.workspace=true\n",
            false
        )
        .is_err());
    }
    #[test]
    fn comments_are_not_permissions() {
        assert!(validate("[package]\n# license-file.workspace=true\n", false).is_err());
    }
    #[test]
    fn wrong_section_is_rejected() {
        assert!(validate("[dependencies]\nlicense-file.workspace=true\n", false).is_err());
    }
}
