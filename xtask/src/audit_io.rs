//! Deterministic inventories must not silently discard filesystem errors.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

fn sorted(entries: impl IntoIterator<Item = io::Result<PathBuf>>) -> io::Result<Vec<PathBuf>> {
    let mut paths = entries.into_iter().collect::<io::Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

pub fn paths(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    sorted(entries.map(|e| e.map(|e| e.path())))
        .map_err(|e| format!("incomplete directory inventory {}: {e}", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_middle_and_last_enumeration_errors_fail_closed() {
        for position in 0..3 {
            let entries = (0..3).map(|i| {
                if i == position {
                    Err(io::Error::from(io::ErrorKind::PermissionDenied))
                } else {
                    Ok(PathBuf::from(format!("file-{i}")))
                }
            });
            assert_eq!(
                sorted(entries).unwrap_err().kind(),
                io::ErrorKind::PermissionDenied
            );
        }
        assert_eq!(
            sorted([Ok(PathBuf::from("b")), Ok(PathBuf::from("a"))]).unwrap(),
            [PathBuf::from("a"), PathBuf::from("b")]
        );
    }
}
