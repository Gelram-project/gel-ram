//! Silicon witness: does this machine compute the same score bits?
//!
//! Every public Q8 score is a sum of 256 cosine values weighted by a histogram
//! of Q8 symbol differences (docs/Q8-QUAD.md). The cosines are a table frozen
//! in gel-phase-quad (`COS256_BITS`), not the platform's maths library, whose
//! last bit differs from it on some levels on other platforms. The witness reads
//! all 256 values through the public `Reader`, one active position per level, so
//! each score is exactly one cosine, then reads 1,048,576 scores of a synthetic
//! bank, and compares both with SHA-256 pins recorded on the author's Linux host
//! before any other run. It also reports, for information only, where this
//! platform's own cosine differs from the table.
use gel_phase_quad::{
    grid::{COS256_BITS, DIM},
    Policy, Reader, Record,
};
use gel_source::{digest, hex};

/// The reader seed; `read` does not depend on it.
pub const SEED: u64 = 510_051;
pub const QUERIES: usize = 1024;
pub const BODIES: usize = 1024;

/// SHA-256 of the 256 cosine bits, little-endian, level by level.
const TABLE_SHA256: &str = "738d15af93756c68a256af78e4d5fe8bed9628c3b1e5c25038c0c64f6b36ad97";
/// SHA-256 of the 1,048,576 score bits, little-endian, query by query.
const SCORES_SHA256: &str = "3c09bb543d6e55eccd18a59b1c5e0c453d62c430ede8f7fee29cf300a2670cf0";

/// The score the public reader gives for a symbol difference of `level` at one
/// active position: that level's cosine, bit for bit.
pub fn table(reader: &Reader) -> [u64; 256] {
    let mut active = [false; DIM];
    active[0] = true;
    let body = Record::new([0; DIM], &[true; DIM]);
    std::array::from_fn(|level| {
        let mut phase = [0; DIM];
        phase[0] = level as u8;
        let query = reader.prepare(Record::new(phase, &active));
        reader
            .read(&query, &body, Policy::BodyActivity)
            .map_or(u64::MAX, |score| score.value().to_bits())
    })
}

/// This platform's own cosine for the same 256 arguments; scores do not use it.
pub fn platform_cosines() -> [u64; 256] {
    std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos().to_bits())
}

fn sha256(bits: &[u64; 256]) -> String {
    let bytes: Vec<u8> = bits.iter().flat_map(|b| b.to_le_bytes()).collect();
    hex(&digest(&bytes))
}

/// A synthetic record: splitmix64 symbols, about two thirds of positions active.
fn record(seed: u64) -> Record {
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

/// Upper bound on the threads that compute the witness scores, the caller included.
const MAX_SCORE_WORKERS: usize = 64;

/// The bits of every score of `QUERIES` queries against `BODIES` bodies, query by query.
pub fn scores(reader: &Reader) -> Vec<u8> {
    let workers = std::thread::available_parallelism().map_or(1, usize::from);
    score_grid(reader, 0, QUERIES, BODIES, workers)
}

/// Scores of queries `first_query..first_query + queries` against bodies
/// `0..bodies`, query by query, little-endian. Contiguous query ranges go to
/// at most `workers` threads (the caller included); each range is written at
/// its own position, so the bytes do not depend on the thread count.
fn score_grid(
    reader: &Reader,
    first_query: u64,
    queries: usize,
    bodies: usize,
    workers: usize,
) -> Vec<u8> {
    let records: Vec<Record> = (0..bodies as u64).map(|b| record(2 * b + 1)).collect();
    let row = 8 * bodies;
    let mut out = vec![0u8; row * queries];
    let fill = |first: u64, rows: &mut [u8]| {
        if row == 0 {
            return;
        }
        for (q, cells) in (first..).zip(rows.chunks_exact_mut(row)) {
            let query = reader.prepare(record(2 * q + 2));
            for (body, cell) in records.iter().zip(cells.chunks_exact_mut(8)) {
                let bits = reader
                    .read(&query, body, Policy::BodyActivity)
                    .map_or(u64::MAX, |score| score.value().to_bits());
                cell.copy_from_slice(&bits.to_le_bytes());
            }
        }
    };
    let workers = workers.clamp(1, MAX_SCORE_WORKERS).min(queries.max(1));
    if workers == 1 || row == 0 {
        fill(first_query, &mut out);
        return out;
    }
    let per = queries.div_ceil(workers);
    let failed = std::thread::scope(|scope| {
        let (head, rest) = out.split_at_mut(row * per);
        let mut next = first_query + per as u64;
        for rows in rest.chunks_mut(row * per) {
            let first = next;
            next += per as u64;
            if std::thread::Builder::new()
                .spawn_scoped(scope, move || fill(first, rows))
                .is_err()
            {
                return true;
            }
        }
        fill(first_query, head);
        false
    });
    if failed {
        fill(first_query, &mut out);
    }
    out
}

/// Number of differing levels and the first of them.
pub fn compare(observed: &[u64; 256], expected: &[u64; 256]) -> (usize, Option<usize>) {
    let differing: Vec<usize> = (0..256).filter(|&i| observed[i] != expected[i]).collect();
    (differing.len(), differing.first().copied())
}

/// Every differing level as `level:ulps`, the signed distance in units in the
/// last place (both values have the same sign at every level).
pub fn differences(observed: &[u64; 256], expected: &[u64; 256]) -> String {
    (0..256)
        .filter(|&i| observed[i] != expected[i])
        .map(|i| format!("{i}:{:+}", observed[i] as i64 - expected[i] as i64))
        .collect::<Vec<_>>()
        .join(",")
}

/// The witness report and whether this machine matched the pinned bits.
pub fn run() -> (String, bool) {
    let reader = Reader::new(SEED);
    let observed = table(&reader);
    let (differing, first) = compare(&observed, &COS256_BITS);
    let table_sha = sha256(&observed);
    let scores_sha = hex(&digest(&scores(&reader)));
    let table_ok = differing == 0 && table_sha == TABLE_SHA256;
    let scores_ok = scores_sha == SCORES_SHA256;
    let verdict = |ok: bool| if ok { "MATCH" } else { "MISMATCH" };
    let mut report = format!(
        "WITNESS_TABLE={} levels=256 differing={differing} sha256={table_sha}",
        verdict(table_ok),
    );
    if let Some(level) = first {
        report.push_str(&format!(
            " first_level={level} expected=0x{:016x} observed=0x{:016x} levels_ulps={}",
            COS256_BITS[level],
            observed[level],
            differences(&observed, &COS256_BITS)
        ));
    }
    report.push_str(&format!(
        "\nWITNESS_SCORES={} scores={} sha256={scores_sha}\n",
        verdict(scores_ok),
        QUERIES * BODIES
    ));
    let libm = platform_cosines();
    let (libm_differing, _) = compare(&libm, &COS256_BITS);
    report.push_str(&format!(
        "WITNESS_LIBM levels=256 differing={libm_differing} levels_ulps={} | this platform's own cosine, which the scores do not use\n",
        differences(&libm, &COS256_BITS)
    ));
    report.push_str(&format!(
        "WITNESS={} platform={}-{} | bits pinned on the author's Linux host before any other run\n",
        verdict(table_ok && scores_ok),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    (report, table_ok && scores_ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The serial loop `scores` used before it was split by query, kept as the
    /// reference for the threaded grid.
    fn scores_reference(
        reader: &Reader,
        first_query: u64,
        queries: usize,
        bodies: usize,
    ) -> Vec<u8> {
        let bodies: Vec<Record> = (0..bodies as u64).map(|b| record(2 * b + 1)).collect();
        let mut out = Vec::with_capacity(8 * queries * bodies.len());
        for q in first_query..first_query + queries as u64 {
            let query = reader.prepare(record(2 * q + 2));
            for body in &bodies {
                let bits = reader
                    .read(&query, body, Policy::BodyActivity)
                    .map_or(u64::MAX, |score| score.value().to_bits());
                out.extend_from_slice(&bits.to_le_bytes());
            }
        }
        out
    }

    #[test]
    fn threaded_grid_matches_the_serial_loop_on_many_shapes() {
        let reader = Reader::new(SEED);
        let mut state = 0x5eed_u64;
        let mut firsts = vec![0u64];
        for _ in 0..5 {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            firsts.push(state % 1_000_000);
        }
        for &first in &firsts {
            for queries in [0, 1, 2, 3, 5, 7, 8, 9, 31, 64, 129] {
                for bodies in [0, 1, 2, 7, 64] {
                    let expected = scores_reference(&reader, first, queries, bodies);
                    for workers in [0, 1, 2, 3, 4, 7, 8, 24, 64, 1000, usize::MAX] {
                        assert!(
                            score_grid(&reader, first, queries, bodies, workers) == expected,
                            "first={first} queries={queries} bodies={bodies} workers={workers}"
                        );
                    }
                }
            }
        }
        for queries in [1, 3, 9] {
            let expected = scores_reference(&reader, 0, queries, BODIES);
            for workers in [1, 2, 3, 64] {
                assert!(score_grid(&reader, 0, queries, BODIES, workers) == expected);
            }
        }
    }

    /// The full witness grid: the threaded path gives the serial bytes and the pin.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "full 1024 x 1024 grid; run with --release")]
    fn full_grid_matches_the_serial_loop_and_the_pin() {
        let reader = Reader::new(SEED);
        let expected = scores_reference(&reader, 0, QUERIES, BODIES);
        assert_eq!(hex(&digest(&expected)), SCORES_SHA256);
        assert!(scores(&reader) == expected);
        for workers in [1, 3, 64] {
            assert!(score_grid(&reader, 0, QUERIES, BODIES, workers) == expected);
        }
    }

    #[test]
    fn a_changed_level_is_found_and_named() {
        let mut changed = COS256_BITS;
        changed[200] ^= 1;
        changed[201] ^= 1;
        assert_eq!(compare(&COS256_BITS, &COS256_BITS), (0, None));
        assert_eq!(compare(&changed, &COS256_BITS), (2, Some(200)));
        assert_ne!(sha256(&changed), TABLE_SHA256);
        changed[200] = COS256_BITS[200] + 1;
        changed[201] = COS256_BITS[201] - 2;
        assert_eq!(differences(&changed, &COS256_BITS), "200:+1,201:-2");
        assert_eq!(differences(&COS256_BITS, &COS256_BITS), "");
    }

    #[test]
    fn records_are_deterministic_and_every_query_has_an_active_position() {
        assert_eq!(record(7).phase(), record(7).phase());
        assert_ne!(record(7).phase(), record(9).phase());
        for q in 0..QUERIES as u64 {
            assert!(record(2 * q + 2).active_mask().iter().any(|&a| a));
        }
    }

    /// The witness itself, on every platform the tests run on: the reader must
    /// give the pinned table whatever this platform's own cosine says.
    #[test]
    fn this_platform_gives_the_frozen_cosine_bits() {
        let observed = table(&Reader::new(SEED));
        let (differing, first) = compare(&observed, &COS256_BITS);
        let libm = platform_cosines();
        assert_eq!(
            (differing, sha256(&observed).as_str()),
            (0, TABLE_SHA256),
            "{}-{}: first differing level {first:?}; level:ulps {}; own cosine level:ulps {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            differences(&observed, &COS256_BITS),
            differences(&libm, &COS256_BITS)
        );
        println!(
            "{}-{} own cosine level:ulps [{}]",
            std::env::consts::OS,
            std::env::consts::ARCH,
            differences(&libm, &COS256_BITS)
        );
    }
}
