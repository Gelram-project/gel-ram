//! Each license name in docs/LICENSE-PINS.md refers to one exact file: its SHA-256 must match.
use std::{fs, path::Path};

#[test]
fn license_texts_match_their_pins() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let pins = fs::read_to_string(root.join("docs/LICENSE-PINS.md")).unwrap();
    let mut checked = 0;
    for line in pins.lines().filter(|l| l.starts_with("| ")) {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        let [_, file, pin] = cells[..] else { continue };
        if pin.len() != 64 || !pin.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let path = file
            .split_once("](../")
            .and_then(|(_, rest)| rest.strip_suffix(')'))
            .unwrap_or_else(|| panic!("pin row without a link to a root file: {line}"));
        let bytes = fs::read(root.join(path)).unwrap();
        let found = gel_source::hex(&gel_source::digest(&bytes));
        assert_eq!(found, pin, "{path} does not match its pinned SHA-256");
        checked += 1;
    }
    assert_eq!(checked, 4, "expected four pinned license texts");
}
