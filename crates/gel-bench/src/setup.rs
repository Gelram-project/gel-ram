//! Work outside the timed region: bank generation and the independent Top-K
//! reference. Every result is a pure function of slot indices, so it is the
//! same for any worker count; the threads only shorten the run.

use gel_core::{splitmix64, ORB_WORDS};
use gel_orb::Orb1024;

/// Orbs per worker below which bank generation stays on one thread.
pub const BANK_ORBS_PER_WORKER: usize = 16_384;
/// Orbs per worker below which the reference scoring stays on one thread.
pub const REFERENCE_ORBS_PER_WORKER: usize = 1_024;

pub fn orb(index: usize, seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    words[0] = seed ^ index as u64;
    for (i, w) in words.iter_mut().enumerate().skip(1) {
        *w = splitmix64(seed ^ index as u64 ^ (i as u64).rotate_left(17));
    }
    Orb1024::from_words(words)
}

/// Worker count for `items` with at least `per_worker` items each, capped
/// by `available_parallelism`; never zero.
pub fn workers_for(items: usize, per_worker: usize) -> usize {
    std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(items / per_worker.max(1))
        .max(1)
}

/// Calls `fill(start, part)` once for each contiguous part of `data`, where
/// `start` is the index of `part[0]` in `data`. Parts run on scoped threads;
/// a part whose thread cannot start is filled on the calling thread after
/// the others finish. The parts are disjoint, so the result depends only on
/// what `fill` writes for each index.
pub fn fill_chunks<T, F>(data: &mut [T], workers: usize, fill: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync,
{
    let len = data.len();
    let workers = workers.clamp(1, len.max(1));
    if workers == 1 {
        fill(0, data);
        return;
    }
    let chunk = len.div_ceil(workers);
    let mut not_started = Vec::new();
    std::thread::scope(|scope| {
        let fill = &fill;
        let mut parts = data.chunks_mut(chunk).enumerate();
        let first = parts.next();
        for (i, part) in parts {
            let start = i * chunk;
            let spawned =
                std::thread::Builder::new().spawn_scoped(scope, move || fill(start, part));
            if spawned.is_err() {
                not_started.push(start);
            }
        }
        if let Some((_, part)) = first {
            fill(0, part);
        }
    });
    for start in not_started {
        let end = (start + chunk).min(len);
        fill(start, &mut data[start..end]);
    }
}

/// Writes `orb(index, seed)` into every slot of `bank`.
pub fn fill_bank(bank: &mut [Orb1024], seed: u64, workers: usize) {
    fill_chunks(bank, workers, |start, part| {
        for (offset, slot) in part.iter_mut().enumerate() {
            *slot = orb(start + offset, seed);
        }
    });
}

/// Matching bits counted one bit at a time, without the popcount kernel.
fn per_bit_score(orb: &Orb1024, query: &Orb1024) -> u16 {
    (0..1024)
        .filter(|&bit| {
            ((orb.words()[bit / 64] >> (bit % 64)) & 1)
                == ((query.words()[bit / 64] >> (bit % 64)) & 1)
        })
        .count() as u16
}

/// Score descending, then index ascending. Indices are unique, so this is a
/// strict total order and the first `k` entries are fully determined.
fn rank(a: &(usize, u16), b: &(usize, u16)) -> std::cmp::Ordering {
    b.1.cmp(&a.1).then(a.0.cmp(&b.0))
}

/// Independent per-bit scoring of every slot, then the first `k` entries in
/// full-sort order: an exact selection of the first `k` followed by a sort
/// of those `k` returns the same entries as sorting all of them.
pub fn reference_top_k_with(
    bank: &[Orb1024],
    query: &Orb1024,
    k: usize,
    workers: usize,
) -> Vec<(usize, u16)> {
    let mut scores = vec![(0usize, 0u16); bank.len()];
    fill_chunks(&mut scores, workers, |start, part| {
        for (offset, slot) in part.iter_mut().enumerate() {
            let index = start + offset;
            *slot = (index, per_bit_score(&bank[index], query));
        }
    });
    if k == 0 {
        scores.clear();
    } else if k < scores.len() {
        scores.select_nth_unstable_by(k - 1, rank);
        scores.truncate(k);
        scores.sort_unstable_by(rank);
    } else {
        scores.sort_unstable_by(rank);
    }
    scores
}

/// `reference_top_k_with` on as many workers as the bank size warrants.
pub fn reference_top_k(bank: &[Orb1024], query: &Orb1024, k: usize) -> Vec<(usize, u16)> {
    let workers = workers_for(bank.len(), REFERENCE_ORBS_PER_WORKER);
    reference_top_k_with(bank, query, k, workers)
}
