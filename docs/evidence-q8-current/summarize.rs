// Validates completeness, not authenticity. Logs remain caller-controlled.
// Tests: rustc --edition=2021 --test summarize.rs -o summary-tests
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
fn value(s: &str, key: &str) -> f64 {
    let prefix = format!("{key}=");
    let mut values = s.split_whitespace().filter_map(|w| w.strip_prefix(&prefix));
    let v: f64 = values
        .next()
        .expect("missing metric")
        .parse()
        .expect("invalid metric");
    assert!(values.next().is_none(), "duplicate metric");
    assert!(v.is_finite() && v >= 0., "non-finite or negative metric");
    v
}
fn field<'a>(s: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key}=");
    let mut values = s.split_whitespace().filter_map(|w| w.strip_prefix(&prefix));
    let v = values.next().expect("missing configuration");
    assert!(values.next().is_none(), "duplicate configuration");
    v
}
fn configuration(name: &str, text: &str, v2: bool) {
    let pieces: Vec<_> = name.strip_suffix(".txt").unwrap().split('-').collect();
    assert_eq!(pieces.len(), if v2 { 6 } else { 4 });
    let eq = |key: &str, expected: &str| {
        assert_eq!(
            field(text, key),
            expected,
            "filename/header mismatch: {key}"
        )
    };
    if v2 {
        eq("seed", "510051");
        eq("orbs", pieces[2]);
        eq(
            "sparse",
            match pieces[3] {
                "0" => "false",
                "1" => "true",
                _ => panic!("bad sparse setting"),
            },
        );
        eq("policy", pieces[4]);
        eq("requested_workers", pieces[5]);
    } else {
        if let Some(n) = pieces[2].strip_prefix("synthetic") {
            eq("records", n);
        } else {
            assert_eq!(pieces[2], "external");
        }
        eq("policy", pieces[3]);
        eq("workers", "1");
    }
}
fn recompute(text: &str, v2: bool) -> [f64; 3] {
    let mut sums = [0u128; 3];
    let mut shared = Vec::new();
    for line in text.lines() {
        let c: Vec<_> = line.split(',').collect();
        if c[0].parse::<usize>().is_err() {
            continue;
        }
        assert_eq!(c.len(), 4);
        assert_eq!(
            c[0].parse::<usize>().unwrap(),
            shared.len() + 1,
            "missing/duplicate round"
        );
        for j in 0..3 {
            let ns = c[j + 1].parse::<u64>().expect("invalid duration");
            assert!(ns > 0);
            sums[j] = sums[j].checked_add(u128::from(ns)).unwrap();
        }
        shared.push(c[3].parse::<u64>().unwrap());
    }
    assert_eq!(shared.len(), 9);
    let one = sums[0] as f64 / sums[2] as f64;
    let four = sums[1] as f64 / sums[2] as f64;
    for (key, actual) in [
        (
            if v2 {
                "four_over_shared_total"
            } else {
                "four_over_shared"
            },
            four,
        ),
        (
            if v2 {
                "reference_one_over_shared_total"
            } else {
                "canonical_over_shared"
            },
            one,
        ),
    ] {
        // Writers serialize exactly six fractional digits. Check the rounded
        // representation, but use the unrounded raw-duration ratio in the table.
        let expected: f64 = format!("{actual:.6}").parse().unwrap();
        assert_eq!(
            value(text, key),
            expected,
            "ratio disagrees with raw timing: {key}"
        );
    }
    shared.sort();
    [four, one, shared[4] as f64 / 1e6]
}
fn validate(text: &str, v2: bool) {
    let marker = if v2 {
        "Q8_QUAD_EXACT=PASS"
    } else {
        "Q8_EVIDENCE=PASS"
    };
    assert_eq!(
        text.split_whitespace().filter(|w| *w == marker).count(),
        1,
        "missing/duplicate PASS"
    );
    assert!(
        !text.contains("DEGRADED") && !text.contains("=ERROR") && !text.contains("=FAIL"),
        "failed log"
    );
    assert_eq!(value(text, "rounds"), 9.);
    let n = value(text, if v2 { "orbs" } else { "records" });
    assert!((1. ..=8192.).contains(&n) && n.fract() == 0.);
    let comparisons = value(
        text,
        if v2 {
            "score_view_comparisons"
        } else {
            "view_comparisons"
        },
    );
    assert_eq!(
        comparisons,
        n * 4. * if v2 { 10. } else { 11. },
        "wrong comparison count"
    );
    if v2 {
        assert_eq!(value(text, "reference_one_fallback_scans"), 0.);
        assert_eq!(value(text, "four_views_fallback_scans"), 0.);
        assert_eq!(
            value(text, "requested_workers"),
            value(text, "effective_workers")
        );
    }
}
fn expected_cells(external: bool) -> BTreeSet<String> {
    let mut cells = BTreeSet::new();
    for n in [512, 8192] {
        for s in [0, 1] {
            for p in ["archive", "active"] {
                for w in [1, 24] {
                    cells.insert(format!("V2 {n}-{s}-{p}-{w}"));
                }
            }
        }
    }
    for n in ["synthetic32", "synthetic512", "synthetic8192"] {
        for p in ["archive", "active"] {
            cells.insert(format!("Canonical {n}-{p}"));
        }
    }
    if external {
        for p in ["archive", "active"] {
            cells.insert(format!("Canonical external-{p}"));
        }
    }
    cells
}
fn range(v: &[f64]) -> String {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    format!("{:.3} ({:.3}–{:.3})", v[v.len() / 2], v[0], v[v.len() - 1])
}
fn main() {
    let dir = std::env::args().nth(1).expect("log directory");
    let mut groups = BTreeMap::<String, Vec<[f64; 3]>>::new();
    let mut repetitions = BTreeSet::new();
    let mut contents = BTreeSet::new();
    let mut v2_checks = 0.;
    let mut evidence_checks = 0.;
    for file in fs::read_dir(dir).unwrap() {
        let path = file.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        if !name.ends_with(".txt") || !(name.starts_with("v2-") || name.starts_with("evidence-")) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let v2 = name.starts_with("v2-");
        validate(&text, v2);
        configuration(name, &text, v2);
        assert!(contents.insert(text.clone()), "duplicate log content");
        let (_, rest) = name.split_once('-').unwrap();
        let (rep, cell) = rest.split_once('-').unwrap();
        let rep: usize = rep.parse().expect("invalid repetition");
        assert!(rep < 3, "unexpected repetition");
        let label = format!(
            "{} {}",
            if v2 { "V2" } else { "Canonical" },
            cell.trim_end_matches(".txt")
        );
        assert!(
            repetitions.insert((label.clone(), rep)),
            "duplicate repetition"
        );
        groups.entry(label).or_default().push(recompute(&text, v2));
        if v2 {
            v2_checks += value(&text, "score_view_comparisons");
        } else {
            evidence_checks += value(&text, "view_comparisons");
        }
    }
    let external = groups.keys().any(|k| k.starts_with("Canonical external-"));
    assert_eq!(
        groups.keys().cloned().collect::<BTreeSet<_>>(),
        expected_cells(external),
        "incomplete/unexpected campaign"
    );
    for rows in groups.values() {
        assert_eq!(rows.len(), 3, "incomplete repetitions");
    }
    println!("cell | Four/Shared median (min–max) | Single/Shared median (min–max) | Shared ms/query median (min–max)");
    for (label, rows) in groups {
        assert_eq!(rows.len(), 3);
        println!(
            "{label} | {} | {} | {}",
            range(&rows.iter().map(|r| r[0]).collect::<Vec<_>>()),
            range(&rows.iter().map(|r| r[1]).collect::<Vec<_>>()),
            range(&rows.iter().map(|r| r[2]).collect::<Vec<_>>())
        );
    }
    println!("V2_view_comparisons={v2_checks} EVIDENCE_view_comparisons={evidence_checks}");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swapped_logs_and_configuration_rejected() {
        let log = include_str!("v2-0-512-0-active-1.txt");
        configuration("v2-0-512-0-active-1.txt", log, true);
        for name in [
            "v2-0-8192-0-active-1.txt",
            "v2-0-512-1-active-1.txt",
            "v2-0-512-0-archive-1.txt",
            "v2-0-512-0-active-24.txt",
        ] {
            assert!(std::panic::catch_unwind(|| configuration(name, log, true)).is_err());
        }
        let log = include_str!("evidence-0-synthetic32-active.txt");
        configuration("evidence-0-synthetic32-active.txt", log, false);
        for name in [
            "evidence-0-synthetic512-active.txt",
            "evidence-0-synthetic32-archive.txt",
        ] {
            assert!(std::panic::catch_unwind(|| configuration(name, log, false)).is_err());
        }
    }
    #[test]
    fn ratios_recomputed_and_tampering_rejected() {
        for (log, v2, key) in [
            (
                include_str!("v2-0-512-0-active-1.txt"),
                true,
                "four_over_shared_total",
            ),
            (
                include_str!("evidence-0-external-active.txt"),
                false,
                "four_over_shared",
            ),
        ] {
            assert!(recompute(log, v2)[0] > 0.);
            let bad = log.replace(
                &format!("{key}={}", field(log, key)),
                &format!("{key}=99.000000"),
            );
            assert!(std::panic::catch_unwind(|| recompute(&bad, v2)).is_err());
            let bad = log
                .lines()
                .filter(|l| !l.starts_with("9,"))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(std::panic::catch_unwind(|| recompute(&bad, v2)).is_err());
        }
    }
    #[test]
    fn invalid_metrics_rejected() {
        for s in ["x=NaN", "x=inf", "x=-1", "x=2 x=3", "other=1"] {
            assert!(std::panic::catch_unwind(|| value(s, "x")).is_err());
        }
        assert_eq!(value("x=1.25", "x"), 1.25);
    }
    #[test]
    fn invalid_status_rejected() {
        let good = "Q8_EVIDENCE=PASS rounds=9 records=1 view_comparisons=44";
        validate(good, false);
        for bad in [
            good.replace("PASS", "FAIL"),
            good.replace("PASS", "ERROR"),
            good.replace("rounds=9", "rounds=8"),
            format!("{good} Q8_EVIDENCE=PASS"),
        ] {
            assert!(std::panic::catch_unwind(|| validate(&bad, false)).is_err());
        }
    }
    #[test]
    fn fallback_rejected() {
        let good = include_str!("v2-0-512-0-active-1.txt");
        validate(good, true);
        let bad = good.replace("four_views_fallback_scans=0", "four_views_fallback_scans=1");
        assert!(std::panic::catch_unwind(|| validate(&bad, true)).is_err());
    }
    #[test]
    fn campaign_shape() {
        assert_eq!(expected_cells(false).len(), 22);
        assert_eq!(expected_cells(true).len(), 24);
    }
}
