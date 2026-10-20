//! Timing harness for several Q8 queries against one bank, using only API
//! that also exists on older trees: 16 calls of `Reader::scan_into`, one per
//! query, over a deterministic bank of 32,768 records (37,748,736 bytes of
//! records), with one worker (`queries16_t1`) and with THREADS workers
//! (`queries16_t<THREADS>`).
//!
//! Usage: `cargo run --release -p gel-phase-quad --example speed_q8_queries -- [RUNS] [THREADS]`
//! with RUNS repetitions per case (1..=999, default 9) and THREADS workers
//! (1..=999, default 8). Prints one `SPEED name=<case> run=<i> ns=<n>` line
//! per run and, at the end, `SPEED_CHECK sha256=<hex>` over the bits of every
//! score produced, so two builds can be compared for identical output.
//! Output grammar: `docs/SPEED.md`.
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

/// Bodies: about two thirds of positions active, every sixteenth body silent
/// and every sixteenth fully active. Queries: about two thirds active.
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
    let args = speed::args("speed_q8_queries", "[RUNS] [THREADS]", 2);
    let runs = speed::runs(&args);
    let workers = speed::threads(&args, 1);
    let reader = Reader::new(210_021);
    let (bank, queries) = input(&reader);
    speed::setup(
        "speed_q8_queries",
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
    let mut produced = vec![SharedScore::default(); QUERIES * BANK];
    for (name, w) in [
        ("queries16_t1".to_string(), 1),
        (format!("queries16_t{workers}"), workers),
    ] {
        for run in 0..runs {
            let start = Instant::now();
            for (q, slot) in queries.iter().zip(produced.chunks_exact_mut(BANK)) {
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
            black_box(&produced);
            let ns = start.elapsed().as_nanos();
            speed::line(&name, run, ns);
            for s in &produced {
                hash.update(&s.value().to_bits().to_le_bytes());
            }
        }
    }
    speed::check(hash);
}
