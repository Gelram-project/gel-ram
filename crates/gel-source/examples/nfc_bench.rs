//! Latency of `document::search` on lines that exercise Unicode normalization:
//! the worst and best order of 2047 combining marks in one 4095-byte line,
//! Polish text, decomposed and composed Hangul. Prints p50/p95/p99 per line
//! and the time for a document of 4096 such lines. A timing probe of this
//! machine, not an accuracy measurement.
use gel_source::document::{search, MAX_LINE, MAX_TEXT};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "200".into())
        .parse()?;
    if !(1..=100_000).contains(&n) {
        return Err("iterations must be 1..100000".into());
    }
    let cases = [
        (
            "reversed-marks",
            format!("a{}{}", "\u{0301}".repeat(1023), "\u{0316}".repeat(1024)),
        ),
        (
            "ordered-marks",
            format!("a{}{}", "\u{0316}".repeat(1024), "\u{0301}".repeat(1023)),
        ),
        ("polish", "Zażółć gęślą jaźń, źdźbło i łódź. ".repeat(40)),
        ("hangul-jamo", "\u{1100}\u{1161}\u{11A8}".repeat(450)),
        (
            "hangul-syllables",
            "한국어 문장은 대부분 완성형 음절로 쓴다 ".repeat(40),
        ),
    ];
    println!("case,line_bytes,p50_ms,p95_ms,p99_ms,document_lines,document_s");
    for (name, line) in &cases {
        assert!(line.len() <= MAX_LINE, "{name}");
        let mut ns: Vec<u128> = (0..n)
            .map(|_| {
                let at = Instant::now();
                black_box(search(black_box(line), "a").map(|r| r.matching_lines)).ok();
                at.elapsed().as_nanos()
            })
            .collect();
        ns.sort_unstable();
        let p = |q: usize| ns[(ns.len() - 1) * q / 100] as f64 / 1e6;
        let lines = (MAX_TEXT / (line.len() + 1)).min(4096);
        let document = format!("{line}\n").repeat(lines);
        let at = Instant::now();
        search(&document, "a")?;
        let document_s = at.elapsed().as_secs_f64();
        println!(
            "{name},{},{:.3},{:.3},{:.3},{lines},{document_s:.3}",
            line.len(),
            p(50),
            p(95),
            p(99)
        );
    }
    Ok(())
}
