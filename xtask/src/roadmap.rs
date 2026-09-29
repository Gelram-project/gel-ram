//! Recounts the review work list in docs/ROADMAP.md: the items must be A01 to
//! A24 in order, each in a known state, and the one summary line must state the
//! counts of the table beneath it.
use std::{fs, path::Path};

const STATES: [&str; 4] = ["DONE_SCOPED", "PARTIAL", "OPEN", "OWNER"];
const REVIEW_ITEMS: usize = 24;

fn summary(counts: &[usize; 4]) -> String {
    format!(
        "Summary: {} DONE_SCOPED · {} PARTIAL · {} OPEN · {} OWNER.",
        counts[0], counts[1], counts[2], counts[3]
    )
}

fn is_item_id(cell: &str) -> bool {
    cell.len() == 3 && cell.starts_with('A') && cell[1..].bytes().all(|b| b.is_ascii_digit())
}

fn validate(doc: &str) -> Result<[usize; 4], String> {
    let mut counts = [0; 4];
    let mut ids = Vec::new();
    for line in doc.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 4 || !cells[0].is_empty() || !is_item_id(cells[1]) {
            continue;
        }
        let state = STATES
            .iter()
            .position(|s| *s == cells[2])
            .ok_or_else(|| format!("roadmap item {} has unknown state {}", cells[1], cells[2]))?;
        counts[state] += 1;
        ids.push(cells[1].to_string());
    }
    let expected: Vec<String> = (1..=REVIEW_ITEMS).map(|n| format!("A{n:02}")).collect();
    if ids != expected {
        return Err(format!(
            "roadmap review items must be A01 to A{REVIEW_ITEMS} in order, each once"
        ));
    }
    let want = summary(&counts);
    let lines: Vec<&str> = doc.lines().filter(|l| l.starts_with("Summary:")).collect();
    if lines != [want.as_str()] {
        return Err(format!("roadmap needs exactly one summary line: `{want}`"));
    }
    Ok(counts)
}

pub fn check(root: &Path) -> Result<(), String> {
    let doc = fs::read_to_string(root.join("docs/ROADMAP.md")).map_err(|e| e.to_string())?;
    let c = validate(&doc)?;
    println!(
        "ROADMAP_SUMMARY=PASS done_scoped={} partial={} open={} owner={}",
        c[0], c[1], c[2], c[3]
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(states: &[&str], summary_line: &str) -> String {
        use std::fmt::Write;
        let mut d =
            format!("{summary_line}\n\n| ID | State | Work | Evidence |\n|---|---|---|---|\n");
        for (i, s) in states.iter().enumerate() {
            writeln!(d, "| A{:02} | {s} | work | link |", i + 1).unwrap();
        }
        d
    }

    fn states() -> Vec<&'static str> {
        let mut v = vec!["DONE_SCOPED"; 14];
        v.extend(["PARTIAL"; 6]);
        v.extend(["OPEN"; 3]);
        v.push("OWNER");
        v
    }

    #[test]
    fn accepts_matching_summary_and_rejects_drift() {
        let good = "Summary: 14 DONE_SCOPED · 6 PARTIAL · 3 OPEN · 1 OWNER.";
        assert_eq!(validate(&doc(&states(), good)).unwrap(), [14, 6, 3, 1]);
        let stale = "Summary: 12 DONE_SCOPED · 8 PARTIAL · 3 OPEN · 1 OWNER.";
        assert!(validate(&doc(&states(), stale)).is_err());
        assert!(validate(&format!("{}\n{good}\n", doc(&states(), good))).is_err());
        assert!(validate(&doc(&states(), "")).is_err());
    }

    #[test]
    fn rejects_unknown_state_and_wrong_item_list() {
        let good = "Summary: 14 DONE_SCOPED · 6 PARTIAL · 3 OPEN · 1 OWNER.";
        let mut s = states();
        s[0] = "PASS";
        assert!(validate(&doc(&s, good)).is_err());
        let d = doc(&states(), good);
        assert!(validate(&d.replacen("| A05 |", "| A04 |", 1)).is_err());
        assert!(validate(&d.replacen("| A24 | OWNER | work | link |\n", "", 1)).is_err());
    }

    #[test]
    fn published_roadmap_agrees_with_its_table() {
        check(crate::workspace_root().unwrap()).unwrap();
    }
}
