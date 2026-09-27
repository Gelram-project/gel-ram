//! Property-to-test map (docs/PROPERTY-TESTS.md, roadmap A07).
//!
//! `verify` checks that the map is well formed. The CI evidence collector
//! checks, on every platform, that each named test ran and passed exactly once
//! in the debug workspace pass, or is a declared Unix-only test absent on
//! another platform. That shows the tests ran, not that they cover everything.
use std::{fs, path::Path};

pub const MAP: &str = "docs/PROPERTY-TESTS.md";

pub struct Row {
    pub id: String,
    pub tests: Vec<String>,
}

fn valid_test(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':')
}

/// Table rows `| Pnn | property | source | `test` ... |`, IDs strictly ascending.
pub fn parse(text: &str) -> Result<Vec<Row>, String> {
    let mut rows: Vec<Row> = Vec::new();
    for line in text.lines().filter(|l| l.starts_with("| P")) {
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() != 6 || !cells[5].is_empty() {
            return Err(format!("{MAP}: a row needs four cells: {line}"));
        }
        let id = cells[1].trim();
        let number: u32 = id
            .strip_prefix('P')
            .filter(|n| n.len() == 2)
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("{MAP}: bad property id {id}"))?;
        if let Some(last) = rows.last() {
            if last.id[1..].parse::<u32>().unwrap_or(u32::MAX) >= number {
                return Err(format!("{MAP}: {id} is not after {}", last.id));
            }
        }
        if cells[2].trim().is_empty() || cells[3].trim().is_empty() {
            return Err(format!("{MAP}: {id} needs a property and a source"));
        }
        let tests: Vec<String> = super::code_spans(cells[4])
            .into_iter()
            .map(str::to_string)
            .collect();
        if tests.is_empty() || !tests.iter().all(|t| valid_test(t)) {
            return Err(format!("{MAP}: {id} needs test names in code spans"));
        }
        rows.push(Row {
            id: id.to_string(),
            tests,
        });
    }
    if rows.is_empty() {
        return Err(format!("{MAP}: no property rows"));
    }
    Ok(rows)
}

pub fn read(root: &Path) -> Result<Vec<Row>, String> {
    parse(&fs::read_to_string(root.join(MAP)).map_err(|e| format!("{MAP}: {e}"))?)
}

/// Report lines for a TESTS.txt: `counts(name)` gives (passed, listed) in the
/// debug workspace pass; `excluded(name)` says the test is compiled only on
/// Unix and this platform is not Unix.
pub fn check(
    rows: &[Row],
    counts: impl Fn(&str) -> (usize, usize),
    excluded: impl Fn(&str) -> bool,
) -> Result<String, String> {
    let mut lines = String::new();
    let mut total = 0;
    for row in rows {
        let (mut passed, mut skipped) = (0, 0);
        for test in &row.tests {
            match counts(test) {
                (1, 1) => passed += 1,
                (0, 0) if excluded(test) => skipped += 1,
                (p, l) => {
                    return Err(format!(
                        "{} {test}: passed={p} listed={l}, expected exactly one pass",
                        row.id
                    ))
                }
            }
        }
        total += row.tests.len();
        lines.push_str(&format!(
            "PROPERTY id={} tests={} passed={passed} excluded_here={skipped}\n",
            row.id,
            row.tests.len()
        ));
    }
    lines.push_str(&format!(
        "PROPERTY_MAP=CHECKED rows={} tests={total}\n",
        rows.len()
    ));
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "| ID | P | S | T |\n|---|---|---|---|\n| P01 | a | [s](S.md) | `x::one` `two` |\n| P02 | b | [s](S.md) | `three` |\n";

    #[test]
    fn rows_are_parsed_in_order_with_their_tests() {
        let rows = parse(GOOD).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tests, ["x::one", "two"]);
        assert!(parse(&GOOD.replace("P02", "P01")).is_err());
        assert!(parse(&GOOD.replace("`three`", "three")).is_err());
        assert!(parse(&GOOD.replace("`three`", "`a b`")).is_err());
        assert!(parse("| ID | P |\n").is_err());
    }

    #[test]
    fn every_test_must_pass_exactly_once_unless_excluded_here() {
        let rows = parse(GOOD).unwrap();
        let ok = check(&rows, |_| (1, 1), |_| false).unwrap();
        assert!(ok.contains("PROPERTY id=P01 tests=2 passed=2 excluded_here=0"));
        assert!(ok.ends_with("PROPERTY_MAP=CHECKED rows=2 tests=3\n"));
        assert!(check(
            &rows,
            |t| if t == "two" { (1, 2) } else { (1, 1) },
            |_| false
        )
        .is_err());
        assert!(check(
            &rows,
            |t| if t == "two" { (0, 1) } else { (1, 1) },
            |_| false
        )
        .is_err());
        assert!(check(
            &rows,
            |t| if t == "two" { (0, 0) } else { (1, 1) },
            |_| false
        )
        .is_err());
        let skipped = check(
            &rows,
            |t| if t == "two" { (0, 0) } else { (1, 1) },
            |t| t == "two",
        );
        assert!(skipped
            .unwrap()
            .contains("P01 tests=2 passed=1 excluded_here=1"));
    }
}
