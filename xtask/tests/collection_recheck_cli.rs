//! Execute the unchanged, measured R1 checker against corrupted copies of real evidence.
use std::{fs, path::PathBuf, process::Command};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn changed_field(csv: &str, row: usize, col: usize, replacement: &str) -> String {
    let mut lines: Vec<String> = csv.lines().map(str::to_owned).collect();
    let mut fields: Vec<String> = lines[row].split(',').map(str::to_owned).collect();
    assert_ne!(
        fields[col], replacement,
        "negative fixture must actually change"
    );
    fields[col] = replacement.to_owned();
    lines[row] = fields.join(",");
    lines.join("\n") + "\n"
}

#[test]
fn real_r1_process_rejects_listed_evidence_corruptions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let dir = std::env::temp_dir().join(format!(
        "gel-r1-negative-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let scratch = Scratch(dir);
    let binary = scratch
        .0
        .join(format!("recheck{}", std::env::consts::EXE_SUFFIX));
    let build = Command::new("rustc")
        .arg("--edition=2021")
        .arg(root.join("crates/gel-source/examples/collection_recheck.rs"))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let raw = fs::read_to_string(root.join("docs/evidence-collection/r1/raw.csv")).unwrap();
    let summary = fs::read_to_string(root.join("docs/evidence-collection/r1/summary.csv")).unwrap();
    let run = |label: &str, r: &str, s: &str, expected: Option<&str>| {
        let case = scratch.0.join(label);
        fs::create_dir(&case).unwrap();
        fs::write(case.join("raw.csv"), r).unwrap();
        fs::write(case.join("summary.csv"), s).unwrap();
        let result = Command::new(&binary).arg(&case).output().unwrap();
        let stdout = String::from_utf8(result.stdout).unwrap();
        let stderr = String::from_utf8(result.stderr).unwrap();
        match expected {
            None => {
                assert!(result.status.success(), "{label}: {stderr}");
                assert_eq!(stdout.trim(), "COLLECTION_RECHECK=PASS");
            }
            Some(reason) => {
                assert!(!result.status.success(), "{label}: falsely accepted");
                assert!(!stdout.contains("PASS"), "{label}: misleading stdout");
                assert_eq!(
                    stderr.trim(),
                    format!("COLLECTION_RECHECK=FAIL {reason}"),
                    "{label}"
                );
            }
        }
    };
    run("unchanged", &raw, &summary, None);
    for (label, col, value, reason) in [
        ("failed-correctness", 5, "0", "RAW_ROW"),
        ("invalid-correctness", 5, "true", "RAW_ROW"),
        ("rep-outside", 0, "3", "RAW_IDENTITY"),
        ("wrong-size", 1, "9", "RAW_IDENTITY"),
        ("unknown-operation", 2, "invented", "OP"),
        ("sample-outside", 3, "50", "RAW_IDENTITY"),
        ("negative-time", 4, "-1", "TIME"),
        ("time-overflow", 4, "18446744073709551616", "TIME"),
        (
            "changed-time",
            4,
            "18446744073709551615",
            "SUMMARY_MISMATCH",
        ),
    ] {
        run(
            label,
            &changed_field(&raw, 1, col, value),
            &summary,
            Some(reason),
        );
    }
    for col in 3..10 {
        run(
            &format!("summary-field-{col}"),
            &raw,
            &changed_field(&summary, 1, col, "0"),
            Some("SUMMARY_MISMATCH"),
        );
    }
    run(
        "raw-duplicate",
        &(raw.clone() + raw.lines().nth(1).unwrap() + "\n"),
        &summary,
        Some("RAW_IDENTITY"),
    );
    let omit = |csv: &str| {
        csv.lines()
            .enumerate()
            .filter(|(i, _)| *i != 1)
            .map(|(_, l)| l)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };
    run(
        "raw-missing",
        &omit(&raw),
        &summary,
        Some("INCOMPLETE_MATRIX"),
    );
    run(
        "summary-missing",
        &raw,
        &omit(&summary),
        Some("MISSING_SUMMARY"),
    );
    run(
        "summary-duplicate",
        &raw,
        &(summary.clone() + summary.lines().nth(1).unwrap() + "\n"),
        Some("SUMMARY_IDENTITY"),
    );
    run(
        "summary-wrong-operation",
        &raw,
        &changed_field(&summary, 1, 2, "invented"),
        Some("SUMMARY_IDENTITY"),
    );
    run(
        "summary-invalid-number",
        &raw,
        &changed_field(&summary, 1, 3, "NaN"),
        Some("SUMMARY_NUMBER"),
    );
    run(
        "raw-header",
        &raw.replacen("latency_ns", "latency_ms", 1),
        &summary,
        Some("RAW_HEADER"),
    );
    run(
        "summary-header",
        &raw,
        &summary.replacen("p99_ns", "p99_ms", 1),
        Some("SUMMARY_HEADER"),
    );
    // A coherent reordering is legal: IDs, not file positions, define samples.
    let reverse = |csv: &str| {
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let mut rest: Vec<_> = lines.collect();
        rest.reverse();
        format!("{header}\n{}\n", rest.join("\n"))
    };
    run("reordered", &reverse(&raw), &reverse(&summary), None);
}
