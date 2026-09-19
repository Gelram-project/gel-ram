//! Independent validator/statistics for document_bench CSV; never calls search.
use std::{collections::BTreeMap, io::Read};

const LIMIT: u64 = 8 * 1024 * 1024;
type Series = BTreeMap<String, Vec<u64>>;

fn parse(text: &str) -> Result<Series, &'static str> {
    if text.len() as u64 > LIMIT {
        return Err("file limit");
    }
    let mut lines = text.lines();
    if lines.next() != Some("query_id,class,latency_ns,matching_lines") {
        return Err("header");
    }
    let mut series: Series = BTreeMap::new();
    for (i, line) in lines.enumerate() {
        if i >= 100_000 {
            return Err("row limit");
        }
        let cells: Vec<_> = line.split(',').collect();
        if cells.len() != 4 {
            return Err("columns");
        }
        let (class, count) = [
            ("HIT", "1"),
            ("MISS", "0"),
            ("CONFLICTING_QUOTES", "2"),
            ("UNICODE", "1"),
        ][i % 4];
        if cells[0] != i.to_string() || cells[1] != class || cells[3] != count {
            return Err("query order, class or expected match count");
        }
        let ns: u64 = cells[2].parse().map_err(|_| "duration")?;
        if ns == 0 {
            return Err("zero duration");
        }
        series.entry(class.into()).or_default().push(ns);
    }
    if series.is_empty() {
        return Err("no observations");
    }
    Ok(series)
}

fn percentile(sorted: &[u64], percent: usize) -> u64 {
    sorted[(sorted.len() * percent).div_ceil(100) - 1]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [path] = args.as_slice() else {
        return Err("usage: document_stats CSV_FILE".into());
    };
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(LIMIT + 1)
        .read_to_string(&mut text)?;
    let mut series = parse(&text)?;
    println!("CSV_VALIDATION=PASS; expected diagnostic counts, not authenticated measurements");
    println!("PERCENTILE_METHOD=nearest-rank; units=ns; class,N,min,p50,p95,p99,max,mean");
    for (class, ns) in &mut series {
        ns.sort_unstable();
        let total: u128 = ns.iter().map(|&n| u128::from(n)).sum();
        println!(
            "{class},{},{},{},{},{},{},{}",
            ns.len(),
            ns[0],
            percentile(ns, 50),
            percentile(ns, 95),
            percentile(ns, 99),
            ns[ns.len() - 1],
            total / ns.len() as u128
        );
    }
    println!("Known repeated phrases; not semantic recall or evidence for extreme tail stability.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOOD: &str = "query_id,class,latency_ns,matching_lines\n0,HIT,10,1\n1,MISS,20,0\n2,CONFLICTING_QUOTES,30,2\n3,UNICODE,40,1\n";
    #[test]
    fn validates_four_known_classes() {
        let rows = parse(GOOD).unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows["HIT"], [10]);
    }
    #[test]
    fn rejects_malformed_or_inconsistent_rows() {
        for bad in [
            GOOD.replace("0,HIT", "1,HIT"),
            GOOD.replace("20,0", "0,0"),
            GOOD.replace("30,2", "30,1"),
            GOOD.replace("UNICODE", "OTHER"),
            GOOD.replace("10", "-1"),
            GOOD.replace("40", "18446744073709551616"),
            format!("{GOOD}3,UNICODE,40,1\n"),
            String::new(),
        ] {
            assert!(parse(&bad).is_err());
        }
    }
    #[test]
    fn nearest_rank_is_explicit_for_small_n() {
        assert_eq!(percentile(&[10, 20, 30, 40], 50), 20);
        assert_eq!(percentile(&[10, 20, 30, 40], 99), 40);
        assert_eq!(percentile(&[10], 95), 10);
    }
}
