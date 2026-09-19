//! Small deterministic text-kernel probe, not encrypted storage or an AI benchmark.
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1000".into())
        .parse()?;
    if !(1..=100_000).contains(&n) {
        return Err("iterations must be 1..100000".into());
    }
    let source=format!("{}\nAda did not approve Bob's proposal.\nGate is open.\nGate is not open.\nZażółć gęślą jaźń.\n", "Source preface.\n".repeat(1000));
    eprintln!("SYNTHETIC_TEXT_KERNEL source_bytes={} source_sha256={:x} N={n}; no storage, LLM, P2P or semantic accuracy",source.len(),Sha256::digest(source.as_bytes()));
    println!("query_id,class,latency_ns,matching_lines");
    for i in 0..n {
        let (phrase, class, expected) = match i % 4 {
            0 => ("Ada did not approve", "HIT", 1),
            1 => ("Ada approved Bob", "MISS", 0),
            2 => ("Gate is", "CONFLICTING_QUOTES", 2),
            _ => ("zażółć gęślą", "UNICODE", 1),
        };
        let at = Instant::now();
        let result = gel_source::document::search(black_box(&source), black_box(phrase))?;
        let elapsed = at.elapsed().as_nanos();
        assert_eq!(result.matching_lines, expected);
        for range in &result.passages {
            assert!(source.get(range.clone()).is_some());
        }
        println!("{i},{class},{elapsed},{}", result.matching_lines);
    }
    Ok(())
}
