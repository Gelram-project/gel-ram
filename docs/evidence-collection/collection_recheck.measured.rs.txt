//! Independently recompute all advertised percentiles from raw observations.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
fn check(raw: &str, summary: &str) -> Result<(), String> {
    let mut groups: BTreeMap<(usize, usize, String), Vec<u128>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    if raw.lines().next() != Some("rep,documents,operation,sample,latency_ns,correct") {
        return Err("RAW_HEADER".into());
    }
    for l in raw.lines().skip(1) {
        let p: Vec<_> = l.split(',').collect();
        if p.len() != 6 || p[5] != "1" {
            return Err("RAW_ROW".into());
        }
        let rep = p[0].parse::<usize>().map_err(|_| "REP")?;
        let n = p[1].parse::<usize>().map_err(|_| "N")?;
        let op = p[2].to_string();
        let sample = p[3].parse::<usize>().map_err(|_| "SAMPLE")?;
        let expected = match op.as_str() {
            "exact" | "near" | "miss" | "unicode" => 50,
            "serialize" | "deserialize" | "insert" | "update" | "delete" | "save" | "load" => 5,
            _ => return Err("OP".into()),
        };
        if rep >= 3
            || ![8, 64, 256].contains(&n)
            || sample >= expected
            || !seen.insert((rep, n, op.clone(), sample))
        {
            return Err("RAW_IDENTITY".into());
        }
        groups
            .entry((rep, n, op))
            .or_default()
            .push(u128::from(p[4].parse::<u64>().map_err(|_| "TIME")?));
    }
    if groups.len() != 99 || seen.len() != 2115 {
        return Err("INCOMPLETE_MATRIX".into());
    }
    if summary.lines().next()
        != Some("rep,documents,operation,n,min_ns,p50_ns,p95_ns,p99_ns,max_ns,total_ns")
    {
        return Err("SUMMARY_HEADER".into());
    }
    for l in summary.lines().skip(1) {
        let p: Vec<_> = l.split(',').collect();
        if p.len() != 10 {
            return Err("SUMMARY_ROW".into());
        }
        let key = (
            p[0].parse().map_err(|_| "REP")?,
            p[1].parse().map_err(|_| "N")?,
            p[2].to_string(),
        );
        let mut v = groups.remove(&key).ok_or("SUMMARY_IDENTITY")?;
        v.sort_unstable();
        let n = v.len();
        let percentile = |q: usize| v[(n * q).div_ceil(100) - 1];
        let expected = [
            n as u128,
            v[0],
            percentile(50),
            percentile(95),
            percentile(99),
            v[n - 1],
            v.iter().sum(),
        ];
        for (s, e) in p[3..].iter().zip(expected) {
            if s.parse::<u128>().map_err(|_| "SUMMARY_NUMBER")? != e {
                return Err("SUMMARY_MISMATCH".into());
            }
        }
    }
    if !groups.is_empty() {
        return Err("MISSING_SUMMARY".into());
    }
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| {
        if args.len() != 1 {
            return Err("collection_recheck CAMPAIGN_DIRECTORY".into());
        }
        let p = Path::new(&args[0]);
        check(
            &fs::read_to_string(p.join("raw.csv")).map_err(|e| e.to_string())?,
            &fs::read_to_string(p.join("summary.csv")).map_err(|e| e.to_string())?,
        )
    })();
    if let Err(e) = result {
        eprintln!("COLLECTION_RECHECK=FAIL {e}");
        std::process::exit(1);
    }
    println!("COLLECTION_RECHECK=PASS");
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (String, String) {
        let mut r = "rep,documents,operation,sample,latency_ns,correct\n".to_string();
        let mut s =
            "rep,documents,operation,n,min_ns,p50_ns,p95_ns,p99_ns,max_ns,total_ns\n".to_string();
        for rep in 0..3 {
            for n in [8, 64, 256] {
                for op in [
                    "exact",
                    "near",
                    "miss",
                    "unicode",
                    "serialize",
                    "deserialize",
                    "insert",
                    "update",
                    "delete",
                    "save",
                    "load",
                ] {
                    let count = if ["exact", "near", "miss", "unicode"].contains(&op) {
                        50
                    } else {
                        5
                    };
                    for i in 0..count {
                        r += &format!("{rep},{n},{op},{i},10,1\n");
                    }
                    s += &format!("{rep},{n},{op},{count},10,10,10,10,10,{}\n", count * 10);
                }
            }
        }
        (r, s)
    }
    #[test]
    fn accepts_complete_matrix() {
        let (r, s) = fixture();
        check(&r, &s).unwrap();
    }
    #[test]
    fn rejects_forged_timing_or_summary() {
        let (r, s) = fixture();
        assert!(check(&r.replacen(",0,10,1", ",0,999,1", 1), &s).is_err());
        assert!(check(&r, &s.replacen(",50,10,", ",50,0,", 1)).is_err());
    }
    #[test]
    fn rejects_duplicate_missing_or_wrong_config() {
        let (r, s) = fixture();
        assert!(check(&(r.clone() + r.lines().nth(1).unwrap() + "\n"), &s).is_err());
        assert!(check(&r.lines().take(10).collect::<Vec<_>>().join("\n"), &s).is_err());
        assert!(check(&r.replacen("0,8,exact", "0,9,exact", 1), &s).is_err());
    }
}
