//! Timing example for the witness score grid: `QUERIES` x `BODIES` = 1,048,576
//! Q8 reads through the public `witness::scores`, repeated R times (argument,
//! default 9). Prints one `SPEED` line per run and the SHA-256 of the scores,
//! which must be the same on every run.
#![forbid(unsafe_code)]
use gel_live_lab::witness::{scores, BODIES, QUERIES, SEED};
use gel_phase_quad::Reader;
use gel_source::{digest, hex};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let rounds = match std::env::args().nth(1) {
        None => 9,
        Some(arg) => match arg.parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => {
                eprintln!("usage: speed_witness_watki [ROUNDS>0]");
                std::process::exit(2);
            }
        },
    };
    let reader = Reader::new(SEED);
    println!(
        "SPEED_INPUT queries={QUERIES} bodies={BODIES} bytes={}",
        8 * QUERIES * BODIES
    );
    let mut first: Option<String> = None;
    for run in 0..rounds {
        let start = Instant::now();
        let bytes = black_box(scores(black_box(&reader)));
        let ns = start.elapsed().as_nanos();
        println!("SPEED name=witness_scores run={run} ns={ns}");
        let sha = hex(&digest(&bytes));
        match &first {
            None => first = Some(sha),
            Some(f) if *f == sha => {}
            Some(f) => {
                eprintln!("SPEED_CHECK mismatch run={run} first={f} now={sha}");
                std::process::exit(1);
            }
        }
    }
    println!("SPEED_CHECK sha256={}", first.unwrap_or_default());
}
