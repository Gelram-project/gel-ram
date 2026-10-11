//! Silicon witness: does this machine compute the same bits?
//!
//! Every public Q8 score is a sum of 256 cosine values weighted by a histogram
//! of Q8 symbol differences (docs/Q8-QUAD.md). The cosines come from the
//! platform's maths library at run time. The witness reads all 256 of them through the public `Reader`, one
//! active position per level, so each score is exactly one cosine, and then
//! reads 1,048,576 scores of a synthetic bank. Both are compared with bits
//! frozen in this file, recorded on the author's Linux host before any other run.
//! A difference is a result, not a failure of the witness: it is reported with
//! the first differing level.
use gel_phase_quad::{grid::DIM, Policy, Reader, Record};
use gel_source::{digest, hex};

/// The reader seed; `read` does not depend on it.
pub const SEED: u64 = 510_051;
pub const QUERIES: usize = 1024;
pub const BODIES: usize = 1024;

/// The 256 cosine values as IEEE-754 bits, frozen on the author's Linux host.
#[rustfmt::skip]
const TABLE: [u64; 256] = [
    0x3ff0000000000000, 0x3feffd886084cd0d, 0x3feff621e3796d7e, 0x3fefe9cdad01883a,
    0x3fefd88da3d12526, 0x3fefc26470e19fd3, 0x3fefa7557f08a517, 0x3fef8764fa714ba9,
    0x3fef6297cff75cb0, 0x3fef38f3ac64e589, 0x3fef0a7efb9230d7, 0x3feed740e7684963,
    0x3fee9f4156c62dda, 0x3fee6288ec48e112, 0x3fee212104f686e5, 0x3feddb13b6ccc23d,
    0x3fed906bcf328d46, 0x3fed4134d14dc93a, 0x3feced7af43cc773, 0x3fec954b213411f5,
    0x3fec38b2f180bdb1, 0x3febd7c0ac6f952a, 0x3feb728345196e3e, 0x3feb090a58150200,
    0x3fea9b66290ea1a3, 0x3fea29a7a0462782, 0x3fe9b3e047f38741, 0x3fe93a22499263fc,
    0x3fe8bc806b151741, 0x3fe83b0e0bff976e, 0x3fe7b5df226aafaf, 0x3fe72d0837efff97,
    0x3fe6a09e667f3bcd, 0x3fe610b7551d2cdf, 0x3fe57d69348cec9f, 0x3fe4e6cabbe3e5e9,
    0x3fe44cf325091dd6, 0x3fe3affa292050b9, 0x3fe30ff7fce17036, 0x3fe26d054cdd12df,
    0x3fe1c73b39ae68c9, 0x3fe11eb3541b4b23, 0x3fe073879922ffed, 0x3fdf8ba4dbf89abb,
    0x3fde2b5d3806f63e, 0x3fdcc66e9931c45e, 0x3fdb5d1009e15cc2, 0x3fd9ef7943a8ed8a,
    0x3fd87de2a6aea964, 0x3fd7088530fa45a1, 0x3fd58f9a75ab1fdd, 0x3fd4135c94176603,
    0x3fd294062ed59f05, 0x3fd111d262b1f678, 0x3fcf19f97b215f1e, 0x3fcc0b826a7e4f62,
    0x3fc8f8b83c69a60d, 0x3fc5e214448b3fcb, 0x3fc2c8106e8e613a, 0x3fbf564e56a97314,
    0x3fb917a6bc29b438, 0x3fb2d52092ce19f8, 0x3fa91f65f10dd824, 0x3f992155f7a36677,
    0x3c91a62633145c07, 0xbf992155f7a36654, 0xbfa91f65f10dd813, 0xbfb2d52092ce19ef,
    0xbfb917a6bc29b42f, 0xbfbf564e56a9730b, 0xbfc2c8106e8e6136, 0xbfc5e214448b3fc7,
    0xbfc8f8b83c69a608, 0xbfcc0b826a7e4f5e, 0xbfcf19f97b215f1a, 0xbfd111d262b1f676,
    0xbfd294062ed59f02, 0xbfd4135c94176600, 0xbfd58f9a75ab1fdb, 0xbfd7088530fa459f,
    0xbfd87de2a6aea962, 0xbfd9ef7943a8ed88, 0xbfdb5d1009e15cbc, 0xbfdcc66e9931c460,
    0xbfde2b5d3806f63c, 0xbfdf8ba4dbf89ab9, 0xbfe073879922ffed, 0xbfe11eb3541b4b21,
    0xbfe1c73b39ae68c6, 0xbfe26d054cdd12df, 0xbfe30ff7fce17035, 0xbfe3affa292050b8,
    0xbfe44cf325091dd5, 0xbfe4e6cabbe3e5e7, 0xbfe57d69348ceca0, 0xbfe610b7551d2cdf,
    0xbfe6a09e667f3bcc, 0xbfe72d0837efff95, 0xbfe7b5df226aafad, 0xbfe83b0e0bff976e,
    0xbfe8bc806b151741, 0xbfe93a22499263fb, 0xbfe9b3e047f38740, 0xbfea29a7a0462781,
    0xbfea9b66290ea1a4, 0xbfeb090a58150200, 0xbfeb728345196e3d, 0xbfebd7c0ac6f9529,
    0xbfec38b2f180bdb0, 0xbfec954b213411f4, 0xbfeced7af43cc773, 0xbfed4134d14dc93a,
    0xbfed906bcf328d46, 0xbfeddb13b6ccc23c, 0xbfee212104f686e4, 0xbfee6288ec48e112,
    0xbfee9f4156c62dda, 0xbfeed740e7684963, 0xbfef0a7efb9230d7, 0xbfef38f3ac64e588,
    0xbfef6297cff75cb0, 0xbfef8764fa714ba9, 0xbfefa7557f08a517, 0xbfefc26470e19fd3,
    0xbfefd88da3d12525, 0xbfefe9cdad01883a, 0xbfeff621e3796d7e, 0xbfeffd886084cd0d,
    0xbff0000000000000, 0xbfeffd886084cd0d, 0xbfeff621e3796d7e, 0xbfefe9cdad01883a,
    0xbfefd88da3d12526, 0xbfefc26470e19fd3, 0xbfefa7557f08a517, 0xbfef8764fa714baa,
    0xbfef6297cff75cb0, 0xbfef38f3ac64e589, 0xbfef0a7efb9230d7, 0xbfeed740e7684964,
    0xbfee9f4156c62ddb, 0xbfee6288ec48e113, 0xbfee212104f686e5, 0xbfeddb13b6ccc23d,
    0xbfed906bcf328d47, 0xbfed4134d14dc93a, 0xbfeced7af43cc774, 0xbfec954b213411f5,
    0xbfec38b2f180bdb1, 0xbfebd7c0ac6f952a, 0xbfeb728345196e3e, 0xbfeb090a58150201,
    0xbfea9b66290ea1a5, 0xbfea29a7a0462782, 0xbfe9b3e047f38741, 0xbfe93a22499263fc,
    0xbfe8bc806b151742, 0xbfe83b0e0bff9770, 0xbfe7b5df226aafaf, 0xbfe72d0837efff97,
    0xbfe6a09e667f3bce, 0xbfe610b7551d2ce0, 0xbfe57d69348ceca2, 0xbfe4e6cabbe3e5ec,
    0xbfe44cf325091dda, 0xbfe3affa292050bd, 0xbfe30ff7fce17033, 0xbfe26d054cdd12de,
    0xbfe1c73b39ae68c8, 0xbfe11eb3541b4b23, 0xbfe073879922ffee, 0xbfdf8ba4dbf89abd,
    0xbfde2b5d3806f63f, 0xbfdcc66e9931c464, 0xbfdb5d1009e15cc7, 0xbfd9ef7943a8ed93,
    0xbfd87de2a6aea96d, 0xbfd7088530fa459b, 0xbfd58f9a75ab1fdb, 0xbfd4135c94176601,
    0xbfd294062ed59f07, 0xbfd111d262b1f67a, 0xbfcf19f97b215f23, 0xbfcc0b826a7e4f6e,
    0xbfc8f8b83c69a619, 0xbfc5e214448b3fd7, 0xbfc2c8106e8e614e, 0xbfbf564e56a972fd,
    0xbfb917a6bc29b421, 0xbfb2d52092ce19f1, 0xbfa91f65f10dd816, 0xbf992155f7a3669b,
    0xbcaa79394c9e8a0a, 0x3f992155f7a36631, 0x3fa91f65f10dd7e1, 0x3fb2d52092ce19d6,
    0x3fb917a6bc29b407, 0x3fbf564e56a972e3, 0x3fc2c8106e8e6141, 0x3fc5e214448b3fca,
    0x3fc8f8b83c69a60c, 0x3fcc0b826a7e4f61, 0x3fcf19f97b215f16, 0x3fd111d262b1f674,
    0x3fd294062ed59f00, 0x3fd4135c941765fb, 0x3fd58f9a75ab1fd5, 0x3fd7088530fa4595,
    0x3fd87de2a6aea967, 0x3fd9ef7943a8ed8d, 0x3fdb5d1009e15cc1, 0x3fdcc66e9931c45e,
    0x3fde2b5d3806f63a, 0x3fdf8ba4dbf89ab7, 0x3fe073879922ffec, 0x3fe11eb3541b4b20,
    0x3fe1c73b39ae68c5, 0x3fe26d054cdd12db, 0x3fe30ff7fce17031, 0x3fe3affa292050bb,
    0x3fe44cf325091dd7, 0x3fe4e6cabbe3e5e9, 0x3fe57d69348cec9f, 0x3fe610b7551d2cde,
    0x3fe6a09e667f3bcb, 0x3fe72d0837efff94, 0x3fe7b5df226aafad, 0x3fe83b0e0bff976b,
    0x3fe8bc806b15173e, 0x3fe93a22499263f8, 0x3fe9b3e047f38742, 0x3fea29a7a0462782,
    0x3fea9b66290ea1a3, 0x3feb090a581501ff, 0x3feb728345196e3d, 0x3febd7c0ac6f9529,
    0x3fec38b2f180bdaf, 0x3fec954b213411f3, 0x3feced7af43cc771, 0x3fed4134d14dc938,
    0x3fed906bcf328d44, 0x3feddb13b6ccc23d, 0x3fee212104f686e5, 0x3fee6288ec48e112,
    0x3fee9f4156c62dda, 0x3feed740e7684963, 0x3fef0a7efb9230d7, 0x3fef38f3ac64e588,
    0x3fef6297cff75caf, 0x3fef8764fa714ba8, 0x3fefa7557f08a516, 0x3fefc26470e19fd3,
    0x3fefd88da3d12526, 0x3fefe9cdad01883a, 0x3feff621e3796d7e, 0x3feffd886084cd0d,
];
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

/// The witness report and whether this machine matched the frozen bits.
pub fn run() -> (String, bool) {
    let reader = Reader::new(SEED);
    let observed = table(&reader);
    let table_bytes: Vec<u8> = observed.iter().flat_map(|b| b.to_le_bytes()).collect();
    let (differing, first) = compare(&observed, &TABLE);
    let scores_sha = hex(&digest(&scores(&reader)));
    let table_ok = differing == 0;
    let scores_ok = scores_sha == SCORES_SHA256;
    let verdict = |ok: bool| if ok { "MATCH" } else { "MISMATCH" };
    let mut report = format!(
        "WITNESS_TABLE={} levels=256 differing={differing} sha256={}",
        verdict(table_ok),
        hex(&digest(&table_bytes))
    );
    if let Some(level) = first {
        report.push_str(&format!(
            " first_level={level} expected=0x{:016x} observed=0x{:016x} levels_ulps={}",
            TABLE[level],
            observed[level],
            differences(&observed, &TABLE)
        ));
    }
    report.push_str(&format!(
        "\nWITNESS_SCORES={} scores={} sha256={scores_sha}\n",
        verdict(scores_ok),
        QUERIES * BODIES
    ));
    report.push_str(&format!(
        "WITNESS={} platform={}-{} | bits frozen on the author's Linux host before any other run\n",
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
        let mut changed = TABLE;
        changed[200] ^= 1;
        changed[201] ^= 1;
        assert_eq!(compare(&TABLE, &TABLE), (0, None));
        assert_eq!(compare(&changed, &TABLE), (2, Some(200)));
        changed[200] = TABLE[200] + 1;
        changed[201] = TABLE[201] - 2;
        assert_eq!(differences(&changed, &TABLE), "200:+1,201:-2");
        assert_eq!(differences(&TABLE, &TABLE), "");
    }

    #[test]
    fn records_are_deterministic_and_every_query_has_an_active_position() {
        assert_eq!(record(7).phase(), record(7).phase());
        assert_ne!(record(7).phase(), record(9).phase());
        for q in 0..QUERIES as u64 {
            assert!(record(2 * q + 2).active_mask().iter().any(|&a| a));
        }
    }

    /// The witness itself, on every platform the tests run on: a different
    /// maths library shows up here with the first differing level.
    #[test]
    fn this_platform_gives_the_frozen_cosine_bits() {
        let observed = table(&Reader::new(SEED));
        let (differing, first) = compare(&observed, &TABLE);
        assert_eq!(
            differing,
            0,
            "{}-{}: first differing level {first:?}; level:ulps {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            differences(&observed, &TABLE)
        );
    }
}
