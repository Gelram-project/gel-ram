//! Exact copies of the earlier serial bank generation and full-sort Top-K
//! reference, and the comparisons that check `setup` against them. Used by
//! the unit tests and by `examples/identity_bench_setup.rs`.

use crate::setup;
use gel_core::{splitmix64, Crc64Ecma, ORB_WORDS};
use gel_orb::Orb1024;

pub fn old_orb(index: usize, seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    words[0] = seed ^ index as u64;
    for (i, w) in words.iter_mut().enumerate().skip(1) {
        *w = splitmix64(seed ^ index as u64 ^ (i as u64).rotate_left(17));
    }
    Orb1024::from_words(words)
}

pub fn old_bank(count: usize, seed: u64) -> Vec<Orb1024> {
    let mut bank = Vec::new();
    bank.try_reserve_exact(count)
        .expect("reference bank fits in memory");
    bank.extend((0..count).map(|i| old_orb(i, seed)));
    bank
}

pub fn old_reference_top_k(bank: &[Orb1024], query: &Orb1024, k: usize) -> Vec<(usize, u16)> {
    let mut scores: Vec<_> = bank
        .iter()
        .enumerate()
        .map(|(index, orb)| {
            let score = (0..1024)
                .filter(|&bit| {
                    ((orb.words()[bit / 64] >> (bit % 64)) & 1)
                        == ((query.words()[bit / 64] >> (bit % 64)) & 1)
                })
                .count() as u16;
            (index, score)
        })
        .collect();
    scores.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scores.truncate(k);
    scores
}

pub fn bank_crc64(bank: &[Orb1024]) -> u64 {
    let mut crc = Crc64Ecma::new();
    for orb in bank {
        crc.update(&orb.to_le_bytes());
    }
    crc.finish()
}

/// The bank as `run` builds it: exact reservation, zeroed slots, then the
/// threaded fill.
pub fn new_bank(count: usize, seed: u64, workers: usize) -> Vec<Orb1024> {
    let mut bank = Vec::new();
    bank.try_reserve_exact(count)
        .expect("candidate bank fits in memory");
    bank.resize(count, Orb1024::ZERO);
    setup::fill_bank(&mut bank, seed, workers);
    bank
}

/// Cases and equal cases, as (cases, equal).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub cases: usize,
    pub equal: usize,
}

impl Tally {
    pub fn record(&mut self, same: bool) {
        self.cases += 1;
        self.equal += usize::from(same);
    }

    pub fn all_equal(self) -> bool {
        self.cases > 0 && self.cases == self.equal
    }
}

/// New bank vs old bank, slot for slot and by CRC, for each count and
/// worker count (0 stands for `workers_for` as `run` uses it).
pub fn compare_bank(counts: &[usize], workers: &[usize], seed: u64) -> Tally {
    let mut tally = Tally::default();
    for &count in counts {
        let old = old_bank(count, seed);
        let old_crc = bank_crc64(&old);
        for &w in workers {
            let w = if w == 0 {
                setup::workers_for(count, setup::BANK_ORBS_PER_WORKER)
            } else {
                w
            };
            let new = new_bank(count, seed, w);
            tally.record(new.len() == count && new == old && bank_crc64(&new) == old_crc);
        }
        // `setup::orb` itself, on the first, middle and last slot.
        for index in [0, count / 2, count.saturating_sub(1)] {
            tally.record(setup::orb(index, seed) == old_orb(index, seed));
        }
    }
    tally
}

/// The published bank CRC pins, built through the new path.
pub fn compare_crc_pins(seed: u64, counts_and_pins: &[(usize, u64)]) -> Tally {
    let mut tally = Tally::default();
    for &(count, pin) in counts_and_pins {
        let w = setup::workers_for(count, setup::BANK_ORBS_PER_WORKER);
        tally.record(bank_crc64(&new_bank(count, seed, w)) == pin);
    }
    tally
}

/// A bank of `len` slots drawn from a pool of `pool` distinct orbs, so
/// equal orbs (and equal scores) occur at many indices.
pub fn tie_bank(len: usize, pool: usize, salt: u64) -> Vec<Orb1024> {
    let pool = pool.max(1);
    (0..len)
        .map(|i| {
            let pick = splitmix64(salt ^ (i as u64).wrapping_mul(0x9E37_79B9)) as usize % pool;
            old_orb(pick, salt)
        })
        .collect()
}

/// New vs old reference on banks of every length in `lens`, with ties, for
/// k in {0, 1, 8, len-1, len, len+1} and each worker count.
pub fn compare_top_k_small(lens: std::ops::RangeInclusive<usize>, workers: &[usize]) -> Tally {
    let mut tally = Tally::default();
    for len in lens {
        let salt = 0x5eed_0000 ^ len as u64;
        // Alternate between heavy duplication and mostly distinct orbs.
        let pool = if len % 2 == 0 { 3 + len % 7 } else { len + 1 };
        let bank = tie_bank(len, pool, salt);
        let queries = [
            old_orb(0, salt),
            old_orb(1, salt ^ 0xA5A5),
            bank.first().copied().unwrap_or(Orb1024::ZERO),
        ];
        let ks = [0, 1, 8, len.saturating_sub(1), len, len + 1];
        for query in &queries {
            for &k in &ks {
                let old = old_reference_top_k(&bank, query, k);
                for &w in workers {
                    tally.record(setup::reference_top_k_with(&bank, query, k, w) == old);
                }
            }
        }
    }
    tally
}

/// New vs old reference on one bank for each (query index, k); the new side
/// runs both with `reference_top_k` (as `run` calls it) and with `workers`.
pub fn compare_top_k_bank(
    bank: &[Orb1024],
    query_indices: &[usize],
    ks: &[usize],
    workers: &[usize],
) -> Tally {
    let mut tally = Tally::default();
    for &qi in query_indices {
        let query = bank[qi];
        for &k in ks {
            let old = old_reference_top_k(bank, &query, k);
            tally.record(setup::reference_top_k(bank, &query, k) == old);
            for &w in workers {
                tally.record(setup::reference_top_k_with(bank, &query, k, w) == old);
            }
        }
    }
    tally
}
