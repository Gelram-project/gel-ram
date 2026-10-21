//! Timing example for the witness score grid: `QUERIES` x `BODIES` = 1,048,576
//! Q8 reads through the public `witness::scores`, repeated RUNS times
//! (argument, 1..=999, default 9). Prints one `SPEED` line per run and the
//! SHA-256 of the scores, which must be the same on every run; a difference
//! ends the run with `SPEED_FAIL reason=run_differs`. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_live_lab::witness::{scores, BODIES, QUERIES, SEED};
use gel_phase_quad::Reader;
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

fn main() {
    let args = speed::args("speed_witness", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let reader = Reader::new(SEED);
    speed::setup(
        "speed_witness",
        runs,
        &[
            ("queries", QUERIES.to_string()),
            ("bodies", BODIES.to_string()),
            ("bytes", (8 * QUERIES * BODIES).to_string()),
        ],
    );
    let mut first: Option<[u8; 32]> = None;
    for run in 0..runs {
        let start = Instant::now();
        let bytes = black_box(scores(black_box(&reader)));
        let ns = start.elapsed().as_nanos();
        speed::line("witness_scores", run, ns);
        let sha = speed::sha256(&bytes);
        match &first {
            None => first = Some(sha),
            Some(f) if *f == sha => {}
            Some(_) => speed::fail("run_differs"),
        }
    }
    speed::check_digest(&first.expect("RUNS is at least 1"));
}
