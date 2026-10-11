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

/// The bits of every score of `QUERIES` queries against `BODIES` bodies, query by query.
pub fn scores(reader: &Reader) -> Vec<u8> {
    let bodies: Vec<Record> = (0..BODIES as u64).map(|b| record(2 * b + 1)).collect();
    let mut out = Vec::with_capacity(8 * QUERIES * BODIES);
    for q in 0..QUERIES as u64 {
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
