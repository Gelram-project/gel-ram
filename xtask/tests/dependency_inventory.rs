//! Lock/inventory text consistency, not legal validation or vulnerability scanning.
use std::collections::BTreeSet;

const LOCK: &str = include_str!("../../Cargo.lock");
const INVENTORY: &str = include_str!("../../docs/DEPENDENCY-INVENTORY.md");

fn field<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| {
        line.strip_prefix(key)?
            .strip_prefix(" = \"")?
            .strip_suffix('"')
    })
}

fn check(lock: &str, inventory: &str) -> Result<(), &'static str> {
    let mut expected = BTreeSet::new();
    for block in lock.split("[[package]]").skip(1) {
        if field(block, "source").is_none() {
            continue;
        }
        let name = field(block, "name").ok_or("missing name")?;
        let version = field(block, "version").ok_or("missing version")?;
        let hash = field(block, "checksum").ok_or("missing checksum")?;
        expected.insert((name, version));
        let archive = format!("{hash}  {name}-{version}.crate");
        if !inventory.lines().any(|line| line == archive) {
            return Err("missing or mismatched archive checksum");
        }
    }
    let mut actual = BTreeSet::new();
    for line in inventory.lines().filter(|line| line.starts_with("| ")) {
        let cells: Vec<_> = line.split('|').map(str::trim).collect();
        if cells.get(1) == Some(&"Package") {
            continue;
        }
        if cells.len() != 6 || !actual.insert((cells[1], cells[2])) {
            return Err("malformed or duplicate inventory row");
        }
    }
    if expected.is_empty() || actual != expected {
        return Err("dependency list differs from lock");
    }
    Ok(())
}

#[test]
fn inventory_matches_every_locked_external_package() {
    check(LOCK, INVENTORY).unwrap();
}

#[test]
fn rejects_stale_extra_duplicate_or_missing_inventory_entries() {
    for edited in [
        INVENTORY.replace("| sha2 | 0.10.9 |", "| sha2 | 0.0.0 |"),
        INVENTORY.replace(
            "a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283",
            "bad",
        ),
        format!("{INVENTORY}\n| extra | 1 | MIT | upstream |\n"),
        format!("{INVENTORY}\n| sha2 | 0.10.9 | MIT | upstream |\n"),
        INVENTORY
            .lines()
            .filter(|line| !line.starts_with("| sha2 |"))
            .collect::<Vec<_>>()
            .join("\n"),
    ] {
        assert!(check(LOCK, &edited).is_err());
    }
}
