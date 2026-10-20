//! Timing harness for the mask paths of the Q8 record: fixture decode and
//! encode, `Record::new`, `Reader::prepare` and `Reader::view`, over a
//! deterministic bank of 8192 records (the largest Q8DEMO01 fixture: a
//! 9,437,196-byte file). `grid_view_4poles` times `Grid::view` alone, the
//! phase part of a view, as a case whose code is not part of this change.
//!
//! Usage: `cargo run --release -p gel-phase-quad --example speed_q8_masks -- [RUNS]`
//! with RUNS repetitions per case (1..=999, default 9). Prints one
//! `SPEED name=<case> run=<i> ns=<n>` line per run and, at the end,
//! `SPEED_CHECK sha256=<hex>` over every output of every run, so two builds
//! can be compared for identical output. Uses only API that predates this
//! change, so the same file builds on the earlier tree. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_phase_quad::fixture;
use gel_phase_quad::grid::{Carrier, Grid, DIM};
use gel_phase_quad::{Policy, Reader, Record};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;
use speed::Sha256;

const BANK: usize = 8192;
const SEED: u64 = 210_021;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn phase(&mut self) -> Carrier {
        std::array::from_fn(|_| self.next() as u8)
    }
    fn mask(&mut self, per_256: u64) -> [bool; DIM] {
        std::array::from_fn(|_| self.next() % 256 < per_256)
    }
}

fn hash_bools(hash: &mut Sha256, bits: &[bool; DIM]) {
    let bytes: [u8; DIM] = bits.map(u8::from);
    hash.update(&bytes);
}

fn hash_records(hash: &mut Sha256, records: &[Record]) {
    for r in records {
        hash.update(r.phase());
        hash_bools(hash, &r.active_mask());
    }
}

fn main() {
    let args = speed::args("speed_q8_masks", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let reader = Reader::new(SEED);
    let grid = Grid::new(256, SEED).expect("fixed valid grid");
    let mut rng = Rng(0x0062_6f6f_6c31_3032);
    // About two thirds of positions active, every eighth record fully active
    // and every sixteenth silent.
    let inputs: Vec<(Carrier, [bool; DIM])> = (0..BANK)
        .map(|n| {
            let per_256 = match n % 16 {
                0 => 0,
                8 => 256,
                _ => 171,
            };
            (rng.phase(), rng.mask(per_256))
        })
        .collect();
    let bank: Vec<Record> = inputs.iter().map(|(p, a)| Record::new(*p, a)).collect();
    let file = fixture::encode(&bank).expect("valid record count");
    let probe = bank[1].clone();
    speed::setup(
        "speed_q8_masks",
        runs,
        &[
            ("records", BANK.to_string()),
            ("file_bytes", file.len().to_string()),
        ],
    );
    let mut hash = Sha256::new();

    for run in 0..runs {
        let start = Instant::now();
        let decoded = fixture::decode(black_box(&file)).expect("valid file");
        black_box(&decoded);
        let ns = start.elapsed().as_nanos();
        speed::line("decode_8192", run, ns);
        hash_records(&mut hash, &decoded);
    }
    for run in 0..runs {
        let start = Instant::now();
        let encoded = fixture::encode(black_box(&bank)).expect("valid record count");
        black_box(&encoded);
        let ns = start.elapsed().as_nanos();
        speed::line("encode_8192", run, ns);
        hash.update(&encoded);
    }
    for run in 0..runs {
        let start = Instant::now();
        let built: Vec<Record> = black_box(&inputs)
            .iter()
            .map(|(p, a)| Record::new(*p, a))
            .collect();
        black_box(&built);
        let ns = start.elapsed().as_nanos();
        speed::line("record_new_8192", run, ns);
        hash_records(&mut hash, &built);
    }
    for run in 0..runs {
        let start = Instant::now();
        let queries: Vec<_> = black_box(&bank)
            .iter()
            .map(|r| reader.prepare(r.clone()))
            .collect();
        black_box(&queries);
        let ns = start.elapsed().as_nanos();
        speed::line("prepare_8192", run, ns);
        for q in &queries {
            let bits = reader
                .read(q, &probe, Policy::Archive)
                .map_or(u64::MAX, |s| s.value().to_bits());
            hash.update(&bits.to_le_bytes());
        }
    }
    // The two view cases refill one buffer each, allocated before timing.
    let mut views = Vec::with_capacity(4 * BANK);
    let mut phases: Vec<Carrier> = Vec::with_capacity(4 * BANK);
    for run in 0..runs {
        views.clear();
        let start = Instant::now();
        views.extend(
            black_box(&bank)
                .iter()
                .flat_map(|r| (0..4u8).map(|pole| reader.view(r, pole).expect("valid pole"))),
        );
        black_box(&views);
        let ns = start.elapsed().as_nanos();
        speed::line("view_4poles_8192", run, ns);
        for v in &views {
            hash.update(&v.phase);
            hash_bools(&mut hash, &v.active);
        }
    }
    for run in 0..runs {
        phases.clear();
        let start = Instant::now();
        phases.extend(
            black_box(&bank)
                .iter()
                .flat_map(|r| (0..4u8).map(|pole| grid.view(r.phase(), pole).expect("valid pole"))),
        );
        black_box(&phases);
        let ns = start.elapsed().as_nanos();
        speed::line("grid_view_4poles_8192", run, ns);
        for p in &phases {
            hash.update(p);
        }
    }
    speed::check(hash);
}
