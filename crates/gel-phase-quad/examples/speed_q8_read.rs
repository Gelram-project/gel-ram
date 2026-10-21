//! Timing harness for the Q8 scoring path: `Reader::read` and single-worker
//! `Reader::scan_into` over a deterministic synthetic bank of 8192 records
//! (9,437,184 bytes of records, the largest Q8DEMO01 fixture size).
//!
//! Usage: `cargo run --release -p gel-phase-quad --example speed_q8_read -- [RUNS]`
//! with RUNS repetitions per case (1..=999, default 9). Prints one
//! `SPEED name=<case> run=<i> ns=<n>` line per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over the bits of every score produced, so two
//! builds can be compared for identical output. Uses only the public API.
//! Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_phase_quad::{grid::DIM, Policy, Reader, Record, SharedScore};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;
use speed::Sha256;

const BANK: usize = 8192;
const QUERIES: usize = 8;

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
    /// Exactly `count` active positions, chosen at random.
    fn sparse(&mut self, count: usize) -> Record {
        let phase = std::array::from_fn(|_| self.next() as u8);
        let mut active = [false; DIM];
        let mut set = 0;
        while set < count {
            let j = (self.next() % DIM as u64) as usize;
            if !active[j] {
                active[j] = true;
                set += 1;
            }
        }
        Record::new(phase, &active)
    }
}

fn record_scores(hash: &mut Sha256, scores: &[SharedScore]) {
    for s in scores {
        hash.update(&s.value().to_bits().to_le_bytes());
    }
}

fn main() {
    let args = speed::args("speed_q8_read", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let reader = Reader::new(210_021);
    let mut rng = Rng(0x0051_ed27_0b0d_1c3a);
    // Bodies: about two thirds of positions active, as in the witness bank,
    // with every eighth body fully active and every sixteenth silent.
    let bank: Vec<Record> = (0..BANK)
        .map(|n| match n % 16 {
            0 => rng.record(0),
            8 => rng.record(256),
            _ => rng.record(171),
        })
        .collect();
    let dense: Vec<_> = (0..QUERIES)
        .map(|_| reader.prepare(rng.record(171)))
        .collect();
    let sparse: Vec<_> = (0..QUERIES)
        .map(|_| reader.prepare(rng.sparse(64)))
        .collect();
    speed::setup(
        "speed_q8_read",
        runs,
        &[
            ("bank", BANK.to_string()),
            ("record_bytes", (BANK * 1152).to_string()),
            ("queries", QUERIES.to_string()),
            ("threads", "1".to_string()),
        ],
    );
    let cases = [
        ("scan_active_dense", &dense, Policy::BodyActivity, true),
        ("scan_archive_dense", &dense, Policy::Archive, true),
        ("scan_active_sparse64", &sparse, Policy::BodyActivity, true),
        ("read_active_dense", &dense, Policy::BodyActivity, false),
    ];
    let mut hash = Sha256::new();
    let mut out: Vec<SharedScore> = Vec::with_capacity(BANK);
    let mut produced = vec![SharedScore::default(); QUERIES * BANK];
    for (name, queries, policy, scan) in cases {
        for run in 0..runs {
            let start = Instant::now();
            for (q, slot) in queries.iter().zip(produced.chunks_exact_mut(BANK)) {
                if scan {
                    let scored = reader
                        .scan_into(black_box(q), black_box(&bank), policy, 1, &mut out)
                        .expect("valid worker budget");
                    assert!(scored, "every query has an active position");
                    slot.copy_from_slice(&out);
                } else {
                    for (body, score) in black_box(&bank).iter().zip(slot.iter_mut()) {
                        *score = reader
                            .read(black_box(q), body, policy)
                            .expect("every query has an active position");
                    }
                }
            }
            black_box(&produced);
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            record_scores(&mut hash, &produced);
        }
    }
    speed::check(hash);
}
