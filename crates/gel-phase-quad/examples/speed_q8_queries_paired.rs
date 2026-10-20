//! Same-binary timing of 16 single-query `Reader::scan_into` calls
//! (`queries16_t<W>_ref`) against one `Reader::scan_many_into` call for the
//! same 16 queries (`queries16_t<W>_many`), on the deterministic
//! 32,768-record bank of `speed_q8_queries` (37,748,736 bytes of records),
//! with one worker and with THREADS workers. Every row of the multi-query
//! output must have the same bits as the single-query output; a difference
//! ends the run with `SPEED_FAIL reason=many_differs`.
//!
//! Usage: `cargo run --release -p gel-phase-quad --example speed_q8_queries_paired -- [RUNS] [THREADS]`
//! (RUNS 1..=999, default 9; THREADS 1..=999, default 8). Prints
//! `SPEED name=<case> run=<i> ns=<n>` per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over every score of every run. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_phase_quad::{Policy, Query, Reader, Record, SharedScore};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const BANK: usize = 32_768;
const QUERIES: usize = 16;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn record(&mut self, active_per_256: u64) -> Record {
        let phase = std::array::from_fn(|_| self.next() as u8);
        let active = std::array::from_fn(|_| self.next() % 256 < active_per_256);
        Record::new(phase, &active)
    }
}

/// The input of `speed_q8_queries`, generated the same way.
fn input(reader: &Reader) -> (Vec<Record>, Vec<Query>) {
    let mut rng = Rng(0x2026_1020_5138_0016);
    let bank = (0..BANK)
        .map(|n| match n % 16 {
            0 => rng.record(0),
            8 => rng.record(256),
            _ => rng.record(171),
        })
        .collect();
    let queries = (0..QUERIES)
        .map(|_| reader.prepare(rng.record(171)))
        .collect();
    (bank, queries)
}

fn main() {
    let args = speed::args("speed_q8_queries_paired", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let workers = speed::threads(&args, 1);
    let reader = Reader::new(210_021);
    let (bank, queries) = input(&reader);
    speed::setup(
        "speed_q8_queries_paired",
        runs,
        &[
            ("bank", BANK.to_string()),
            ("record_bytes", (BANK * 1152).to_string()),
            ("queries", QUERIES.to_string()),
            ("threads", format!("1,{workers}")),
            ("policy", "active".to_string()),
        ],
    );
    let mut hash = speed::Sha256::new();
    let mut out: Vec<SharedScore> = Vec::with_capacity(BANK);
    let mut single = vec![SharedScore::default(); QUERIES * BANK];
    let mut many: Vec<SharedScore> = Vec::with_capacity(QUERIES * BANK);
    for w in [1, workers] {
        for run in 0..runs {
            let start = Instant::now();
            for (q, slot) in queries.iter().zip(single.chunks_exact_mut(BANK)) {
                let scored = reader
                    .scan_into(
                        black_box(q),
                        black_box(&bank),
                        Policy::BodyActivity,
                        w,
                        &mut out,
                    )
                    .expect("valid worker budget");
                assert!(scored, "every query has an active position");
                slot.copy_from_slice(&out);
            }
            black_box(&single);
            let single_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            let report = reader
                .scan_many_into(
                    black_box(&queries),
                    black_box(&bank),
                    Policy::BodyActivity,
                    w,
                    &mut many,
                )
                .expect("valid worker budget");
            black_box(&many);
            let many_ns = start.elapsed().as_nanos();
            if !report.scored.iter().all(|&s| s) {
                speed::fail("many_not_scored");
            }
            if !(many.len() == single.len()
                && many
                    .iter()
                    .zip(&single)
                    .all(|(a, b)| a.value().to_bits() == b.value().to_bits()))
            {
                speed::fail("many_differs");
            }
            speed::line(&format!("queries16_t{w}_ref"), run, single_ns);
            speed::line(&format!("queries16_t{w}_many"), run, many_ns);
            for s in &many {
                hash.update(&s.value().to_bits().to_le_bytes());
            }
        }
    }
    speed::check(hash);
}
