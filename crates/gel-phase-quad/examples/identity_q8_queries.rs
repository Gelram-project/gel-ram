//! Identity check for `Reader::scan_many_into`: every row must have the same
//! bits as `Reader::scan_into` for that query alone, and every score the same
//! bits as `Reader::read` for that pair. Seeded queries (random densities,
//! single bits, full and empty masks), bank lengths 0..=9, 129, 1031 and the
//! 8192-record fixture maximum, query counts 0..=16, worker counts 1..=MAX,
//! both policies. The 1024 x 1024 grid of the silicon witness is also scored
//! through `scan_many_into` and must hash to the witness's pinned SHA-256.
//!
//! Usage: `cargo run --release -p gel-phase-quad --example identity_q8_queries`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (also FAIL if no case ran).
//! Exit status 0 on PASS, 1 on FAIL. Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_phase_quad::{fixture::MAX_RECORDS, grid::DIM, Policy, Query, Reader, Record, SharedScore};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
mod identity;
#[path = "../../../tools/speed_support.rs"]
mod speed;
use identity::{Scale, Tally};

/// SHA-256 of the witness's 1,048,576 score bits (crates/gel-live-lab/src/witness.rs).
const WITNESS_SCORES_SHA256: &str =
    "3c09bb543d6e55eccd18a59b1c5e0c453d62c430ede8f7fee29cf300a2670cf0";

struct SplitMix(u64);
impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn mask(&mut self, per_256: u64) -> [bool; DIM] {
        std::array::from_fn(|_| self.next() % 256 < per_256)
    }
    fn phase(&mut self) -> [u8; DIM] {
        std::array::from_fn(|_| self.next() as u8)
    }
}

fn single(bits: &[usize]) -> [bool; DIM] {
    std::array::from_fn(|j| bits.contains(&j))
}

/// The witness's synthetic record, same generator and same order of draws.
fn witness_record(seed: u64) -> Record {
    let mut state = seed;
    let mut next = move || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let phase = std::array::from_fn(|_| next() as u8);
    let active = std::array::from_fn(|_| next() % 3 != 0);
    Record::new(phase, &active)
}

fn bits(scores: &[SharedScore]) -> Vec<u64> {
    scores.iter().map(|s| s.value().to_bits()).collect()
}

struct Tallies {
    rows: Tally,
    flags: Tally,
    unknown_rows: Tally,
    pairs: Tally,
}

/// Compares one `scan_many_into` call with single-query scans and pair reads.
#[allow(clippy::too_many_arguments)]
fn compare(
    reader: &Reader,
    queries: &[Query],
    bank: &[Record],
    policy: Policy,
    workers: usize,
    with_pairs: bool,
    many: &mut Vec<SharedScore>,
    one: &mut Vec<SharedScore>,
    t: &mut Tallies,
) {
    let n = bank.len();
    let report = match reader.scan_many_into(queries, bank, policy, workers, many) {
        Ok(report) => report,
        Err(_) => {
            t.rows.check(false);
            return;
        }
    };
    t.flags
        .check(report.scored.len() == queries.len() && !report.serial_fallback);
    t.rows.check(many.len() == queries.len() * n);
    if many.len() != queries.len() * n || report.scored.len() != queries.len() {
        return;
    }
    for (i, q) in queries.iter().enumerate() {
        let row = &many[i * n..(i + 1) * n];
        let scored = reader
            .scan_into(q, bank, policy, workers, one)
            .expect("valid worker count");
        t.flags.check(report.scored[i] == scored);
        if scored {
            t.rows.check(bits(row) == bits(one));
            if with_pairs {
                for (body, score) in bank.iter().zip(row) {
                    let read = reader.read(q, body, policy).map(|s| s.value().to_bits());
                    t.pairs.check(read == Some(score.value().to_bits()));
                }
            }
        } else {
            t.unknown_rows
                .check(row.iter().all(|s| s.value().to_bits() == 0));
        }
    }
}

/// Runs every case. There is no smaller scale: `scale` is accepted for the
/// common interface only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    speed::self_check();
    let mut t = Tallies {
        rows: Tally::new("rows_equal_scan_into"),
        flags: Tally::new("scored_flags_equal_scan_into"),
        unknown_rows: Tally::new("unknown_rows_are_zero"),
        pairs: Tally::new("scores_equal_read"),
    };
    let mut errors = Tally::new("zero_workers_error_and_clear");
    let mut witness = Tally::new("witness_grid_sha256_pin");
    let (mut many, mut one) = (vec![SharedScore::default(); 7], Vec::new());
    for seed in 0..3u64 {
        let reader = Reader::new(seed * 7919 + 1);
        let mut rng = SplitMix(seed ^ 0x51ed_270b_d1c3_a4f1);
        let base = rng.phase();
        let mut records = vec![
            Record::new(rng.phase(), &rng.mask(171)),
            Record::new(rng.phase(), &[false; DIM]),
            Record::new(base, &[true; DIM]),
        ];
        for per_256 in [1, 4, 64, 128, 252, 256] {
            records.push(Record::new(rng.phase(), &rng.mask(per_256)));
        }
        for set in [&[0][..], &[63], &[64], &[1023], &[63, 64, 127, 960]] {
            records.push(Record::new(rng.phase(), &single(set)));
        }
        records.push(Record::new(rng.phase(), &[false; DIM]));
        records.push(Record::new(base, &std::array::from_fn(|j| j >= DIM - 64)));
        assert_eq!(records.len(), 16);
        let queries: Vec<Query> = records.into_iter().map(|r| reader.prepare(r)).collect();
        let bank: Vec<Record> = (0..MAX_RECORDS)
            .map(|n| {
                let phase = match n % 11 {
                    0 => base,
                    1 => base.map(|p| p.wrapping_add(128)),
                    _ => rng.phase(),
                };
                let active = match n % 7 {
                    0 => [false; DIM],
                    1 => [true; DIM],
                    2 => single(&[n % DIM]),
                    k => rng.mask([64, 171, 255, 128][k - 3]),
                };
                Record::new(phase, &active)
            })
            .collect();
        for count in [0, 1, 2, 3, 5, 16] {
            for len in [0, 1, 2, 3, 4, 5, 7, 8, 9, 129, 1031] {
                for policy in [Policy::Archive, Policy::BodyActivity] {
                    for workers in [1, 2, 3, 7, 24, usize::MAX] {
                        compare(
                            &reader,
                            &queries[..count],
                            &bank[..len],
                            policy,
                            workers,
                            true,
                            &mut many,
                            &mut one,
                            &mut t,
                        );
                    }
                }
            }
        }
        for policy in [Policy::Archive, Policy::BodyActivity] {
            for workers in [1, 7, 24] {
                compare(
                    &reader,
                    &queries,
                    &bank,
                    policy,
                    workers,
                    seed == 0,
                    &mut many,
                    &mut one,
                    &mut t,
                );
            }
        }
        many.push(SharedScore::default());
        let refused = reader.scan_many_into(&queries, &bank[..9], Policy::Archive, 0, &mut many);
        let single_refused =
            reader.scan_into(&queries[0], &bank[..9], Policy::Archive, 0, &mut one);
        errors.check(refused.is_err() && refused.err() == single_refused.err() && many.is_empty());
    }
    let reader = Reader::new(510_051);
    let bodies: Vec<Record> = (0..1024u64).map(|b| witness_record(2 * b + 1)).collect();
    let queries: Vec<Query> = (0..1024u64)
        .map(|q| reader.prepare(witness_record(2 * q + 2)))
        .collect();
    let workers = std::thread::available_parallelism().map_or(1, usize::from);
    for workers in [1, workers] {
        let report = reader
            .scan_many_into(&queries, &bodies, Policy::BodyActivity, workers, &mut many)
            .expect("valid worker count");
        let mut hash = speed::Sha256::new();
        for (row, &scored) in many.chunks_exact(bodies.len()).zip(&report.scored) {
            for score in row {
                let cell = if scored {
                    score.value().to_bits()
                } else {
                    u64::MAX
                };
                hash.update(&cell.to_le_bytes());
            }
        }
        witness.check(hash.finish_hex() == WITNESS_SCORES_SHA256);
    }
    vec![t.rows, t.flags, t.unknown_rows, t.pairs, errors, witness]
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
