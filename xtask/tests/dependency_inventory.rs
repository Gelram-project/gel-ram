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
    if actual != expected {
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
    // A registry package that is not in the tree, with the row and archive line
    // the inventory would need for it.
    let block = "\n[[package]]\nname = \"extra\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"00ff\"\n";
    let row = "| extra | 1.0.0 | MIT | upstream |";
    let archive = "00ff  extra-1.0.0.crate";
    let lock = format!("{LOCK}{block}");
    // The check accepts a lock and an inventory that agree, so the refusals below
    // come from the edit, not from a check that refuses everything.
    check(&lock, &format!("{INVENTORY}\n{row}\n{archive}\n")).unwrap();
    for (lock, inventory) in [
        (LOCK.to_string(), format!("{INVENTORY}\n{row}\n")),
        (LOCK.to_string(), format!("{INVENTORY}\n{row}\n{row}\n")),
        (lock.clone(), INVENTORY.to_string()),
        (lock.clone(), format!("{INVENTORY}\n{row}\n")),
        (lock.clone(), format!("{INVENTORY}\n{archive}\n")),
        (
            lock.clone(),
            format!(
                "{INVENTORY}\n{}\n{archive}\n",
                row.replace("1.0.0", "0.9.0")
            ),
        ),
    ] {
        assert!(check(&lock, &inventory).is_err(), "{inventory}");
    }
}
