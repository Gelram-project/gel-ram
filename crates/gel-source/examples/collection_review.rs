//! Post-freeze source-contract assessment, not a blind semantic benchmark.
use gel_source::{collection::Collection, digest, hex};
use std::{collections::BTreeSet, time::Instant};
const DOCS: [(&str, &str); 4] = [
    (
        "Rust Book",
        include_str!("../fixtures/collection-review/ownership.txt"),
    ),
    (
        "Source builder",
        include_str!("../fixtures/collection-review/source-builder.txt"),
    ),
    (
        "Ocean scope",
        include_str!("../fixtures/collection-review/ocean-scale.txt"),
    ),
    (
        "Synthetic Unicode",
        include_str!("../fixtures/collection-review/unicode.txt"),
    ),
];
const CASES: &str = include_str!("../fixtures/collection-review/cases.txt");
fn assess() -> Result<(), String> {
    if hex(&digest(CASES.as_bytes()))
        != "182b4b9a9f439c0fbee347c9131364e81572fff5a6b1d7de28ee7464f858eaa5"
    {
        return Err("FROZEN_CASES_CHANGED".into());
    }
    let mut bank = Collection::new();
    for (title, text) in DOCS {
        bank.add(title, text)?;
        println!(
            "DOCUMENT title={title} bytes={} sha256={}",
            text.len(),
            hex(&digest(text.as_bytes()))
        );
    }
    println!(
        "ASSESSMENT=post-freeze, developer-authored; NOT blind or independent semantic validation"
    );
    println!("CASES_SHA256={}", hex(&digest(CASES.as_bytes())));
    let saved = bank.to_bytes();
    let reopened = Collection::from_bytes(&saved, bank.root())?;
    if reopened.to_bytes() != saved {
        return Err("RECONSTRUCTION".into());
    }
    let mut count = 0;
    let mut failures = 0;
    for line in CASES.lines().skip(1) {
        let p: Vec<_> = line.split('\t').collect();
        if p.len() != 4 || p[0].parse::<usize>().ok() != Some(count + 1) {
            return Err("CASE_FORMAT".into());
        }
        let expected: BTreeSet<u64> = if p[3] == "-" {
            BTreeSet::new()
        } else {
            p[3].split(',')
                .map(|id| id.parse().map_err(|_| "CASE_ID".to_string()))
                .collect::<Result<_, _>>()?
        };
        let start = Instant::now();
        let result = reopened.search(p[2])?;
        let elapsed = start.elapsed().as_nanos();
        let actual: BTreeSet<u64> = result.hits.iter().map(|h| h.document_id()).collect();
        let mut valid = result.skipped_long_lines == 0 && actual == expected;
        for hit in &result.hits {
            valid &= reopened.validate(hit).is_ok();
            let original = DOCS[(hit.document_id() - 1) as usize].1;
            valid &= original.get(hit.span()) == Some(hit.quote());
        }
        let expected_status = if expected.is_empty() {
            "UNKNOWN"
        } else {
            "HIT"
        };
        valid &= result.status() == expected_status;
        println!("case={} kind={} query={:?} expected={expected:?} actual={actual:?} status={} search_ns={elapsed} pass={valid}", p[0], p[1], p[2], result.status());
        failures += usize::from(!valid);
        count += 1;
    }
    let mut changed = saved;
    *changed.last_mut().ok_or("EMPTY")? ^= 1;
    if Collection::from_bytes(&changed, bank.root()).is_ok() {
        return Err("CORRUPTION_ACCEPTED".into());
    }
    println!("CASES={count} FAILURES={failures} REOPEN_BIT_EQUAL=PASS CORRUPTION_REJECTED=PASS");
    if count != 24 || failures != 0 {
        return Err("ASSESSMENT_MISMATCH".into());
    }
    Ok(())
}
fn main() {
    if let Err(e) = assess() {
        eprintln!("COLLECTION_REVIEW=FAIL {e}");
        std::process::exit(1);
    }
    println!("COLLECTION_REVIEW=PASS");
}
#[test]
fn frozen_real_document_cases() {
    assess().unwrap();
}
