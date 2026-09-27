//! Maintained review inventory: counts are checked against the actual 24 rows.
use std::{collections::BTreeSet, fs, path::Path};

pub fn roadmap(text: &str) -> Result<(), String> {
    let states = ["DONE_SCOPED", "PARTIAL", "OPEN", "OWNER"];
    let mut counts = [0usize; 4];
    let mut seen = BTreeSet::new();
    for line in text.lines().filter(|line| line.starts_with("| A")) {
        let fields: Vec<_> = line.split('|').map(str::trim).collect();
        let id = fields.get(1).ok_or("missing roadmap identifier")?;
        let n: usize = id
            .strip_prefix('A')
            .ok_or("invalid roadmap identifier")?
            .parse()
            .map_err(|_| "invalid roadmap number")?;
        if !(1..=24).contains(&n) || *id != format!("A{n:02}") || !seen.insert(n) {
            return Err("duplicate or invalid roadmap identifier".into());
        }
        let state = fields.get(2).ok_or("missing roadmap state")?;
        let index = states
            .iter()
            .position(|s| s == state)
            .ok_or("invalid roadmap state")?;
        counts[index] += 1;
    }
    if seen.len() != 24 {
        return Err("roadmap must contain A01 through A24 exactly once".into());
    }
    let expected = format!(
        "Summary: {} DONE_SCOPED · {} PARTIAL · {} OPEN · {} OWNER.",
        counts[0], counts[1], counts[2], counts[3]
    );
    let summaries: Vec<_> = text.lines().filter(|l| l.starts_with("Summary:")).collect();
    if summaries != [expected.as_str()] {
        return Err(format!("roadmap summary mismatch; expected {expected}"));
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let text = fs::read_to_string(root.join("docs/ROADMAP.md")).map_err(|e| e.to_string())?;
    roadmap(&text)?;
    println!("ROADMAP_CONSISTENCY=PASS rows=24");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const ROADMAP: &str = include_str!("../../docs/ROADMAP.md");
    #[test]
    fn actual_roadmap_counts_are_exact() {
        roadmap(ROADMAP).unwrap();
    }
    #[test]
    fn missing_duplicate_and_unknown_rows_fail() {
        for bad in [
            ROADMAP.replace("| A01 |", "| A02 |"),
            ROADMAP.replace("| A01 |", "| A25 |"),
            ROADMAP.replace("| A01 | DONE_SCOPED", "| A01 | PASS"),
            ROADMAP.replace("Summary:", "Missing:"),
            format!("{ROADMAP}\nSummary: 0 DONE_SCOPED · 0 PARTIAL · 0 OPEN · 0 OWNER.\n"),
        ] {
            assert!(roadmap(&bad).is_err());
        }
        let bad = ROADMAP
            .lines()
            .filter(|l| !l.starts_with("| A01 |"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(roadmap(&bad).is_err());
    }
    #[test]
    fn stale_summary_fails_even_when_the_table_is_valid() {
        let bad = ROADMAP.replace(
            "Summary: 14 DONE_SCOPED · 6 PARTIAL",
            "Summary: 12 DONE_SCOPED · 8 PARTIAL",
        );
        assert_ne!(bad, ROADMAP);
        assert!(roadmap(&bad).is_err());
    }
}
