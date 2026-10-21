//! Timing probe for `gel_source::digest` (SHA-256). Cases:
//! `large`: one digest of MAX_TEXT (64 MiB) of deterministic bytes;
//! `small`: 65,536 digests of messages of 0..=127 bytes (every padding case).
//! Prints `SPEED name=<case> run=<i> ns=<n>` per run and finally
//! `SPEED_CHECK sha256=<hex>`, the digest of every produced digest of run 0,
//! computed by the shared example helper rather than by the code being timed.
//! Every later run must produce the same digests; a difference ends the run
//! with `SPEED_FAIL reason=run_differs`. Usage: `speed_sha256 [RUNS]`
//! (1..=999, default 9). Output grammar: `docs/SPEED.md`.
use gel_source::{digest, MAX_TEXT};
use std::{hint::black_box, time::Instant};

#[path = "../../../tools/speed_support.rs"]
mod speed;

const SMALL: usize = 65_536;

fn bytes(len: usize) -> Vec<u8> {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

fn main() {
    let args = speed::args("speed_sha256", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let data = bytes(MAX_TEXT);
    speed::setup(
        "speed_sha256",
        runs,
        &[
            ("large_bytes", MAX_TEXT.to_string()),
            ("small_messages", SMALL.to_string()),
        ],
    );
    let mut first: Option<Vec<u8>> = None;
    for run in 0..runs {
        let mut produced = Vec::with_capacity(32 * (SMALL + 1));

        let at = Instant::now();
        let large = digest(black_box(&data));
        let ns = at.elapsed().as_nanos();
        black_box(&large);
        speed::line("large", run, ns);
        produced.extend_from_slice(&large);

        let mut small = vec![[0u8; 32]; SMALL];
        let at = Instant::now();
        for (i, out) in small.iter_mut().enumerate() {
            let start = (i * 61) % (MAX_TEXT - 128);
            *out = digest(black_box(&data[start..start + (i % 128)]));
        }
        let ns = at.elapsed().as_nanos();
        black_box(&small);
        speed::line("small", run, ns);
        for d in &small {
            produced.extend_from_slice(d);
        }

        match &first {
            None => first = Some(produced),
            Some(f) if *f != produced => speed::fail("run_differs"),
            Some(_) => {}
        }
    }
    let outputs = first.expect("RUNS is at least 1");
    speed::check_digest(&speed::sha256(&outputs));
}
