//! The SHA-256 digests a load needs, computed on scoped threads: the digest of
//! the whole file on the calling thread while one scoped thread checks the
//! structure and hashes the parts, with further scoped threads for the parts.
//! Every digest is the one [`crate::digest`] returns; only the threads that
//! compute them differ.
use crate::{digest, Hash};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Mutex, OnceLock, PoisonError,
};

/// Inputs at least this long are hashed on scoped threads; shorter inputs
/// keep the one-thread path.
const PARALLEL_MIN: usize = 1 << 20;

/// Upper bound on the threads one load uses: the calling thread (whole-file
/// digest), one thread for the structure and up to two more for the parts.
const MAX_THREADS: usize = 4;

/// A part is hashed in pieces of this many bytes (a multiple of the 64-byte
/// SHA-256 block), and a stop request is seen between two pieces.
const PIECE: usize = 256 << 10;

/// The threads a load of `len` bytes uses: 1 below [`PARALLEL_MIN`], else
/// `available_parallelism` clamped to `1..=MAX_THREADS`.
pub(crate) fn threads_for(len: usize) -> usize {
    static AVAILABLE: OnceLock<usize> = OnceLock::new();
    if len < PARALLEL_MIN {
        return 1;
    }
    *AVAILABLE.get_or_init(|| {
        std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, MAX_THREADS)
    })
}

/// `Some(rest(..))` when `digest(whole) == trusted_pin`, else `None`.
///
/// `digest(whole)` is computed on the calling thread while `rest` runs on one
/// scoped thread with the number of threads left for it (at least 1) and a
/// stop flag. The flag is set only after the digest differs from the pin, so a
/// `rest` result computed under a set flag is always discarded; `rest` may
/// leave work undone once it sees the flag. If the thread cannot be started,
/// the digest is compared first and `rest` runs on the calling thread only
/// when it matches. `threads` must be at least 2.
pub(crate) fn pinned<R: Send>(
    whole: &[u8],
    trusted_pin: Hash,
    threads: usize,
    rest: impl FnOnce(usize, &AtomicBool) -> R + Send,
) -> Option<R> {
    let stop = AtomicBool::new(false);
    // `rest` waits here so the calling thread still has it if no thread starts.
    let slot = Mutex::new(Some(rest));
    let take = || slot.lock().unwrap_or_else(PoisonError::into_inner).take();
    std::thread::scope(|scope| {
        let left = threads.saturating_sub(1).max(1);
        let (stop, take) = (&stop, &take);
        let spawned = std::thread::Builder::new()
            .spawn_scoped(scope, move || take().map(|rest| rest(left, stop)));
        let matches = digest(whole) == trusted_pin;
        match spawned {
            Ok(handle) => {
                if !matches {
                    stop.store(true, Ordering::Relaxed);
                }
                // `rest` cannot panic short of a bug; propagate it if it does.
                let value = handle
                    .join()
                    .unwrap_or_else(|e| std::panic::resume_unwind(e));
                value.filter(|_| matches)
            }
            Err(_) if matches => take().map(|rest| rest(threads.max(1), stop)),
            Err(_) => None,
        }
    })
}

/// `digest(part)`, or `None` once `stop` is set (checked between pieces).
fn digest_unless_stopped(part: &[u8], stop: &AtomicBool) -> Option<Hash> {
    let mut hash = crate::sha256::Sha256::new();
    for piece in part.chunks(PIECE) {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        hash.update(piece);
    }
    Some(hash.finalize())
}

/// `parts.iter().map(|p| digest(p))`, with the parts taken from one shared
/// queue by the calling thread and at most `threads - 1` scoped threads. The
/// digest of part `k` is stored at index `k` whichever thread computes it.
/// Once `stop` is set no further part is started and the part in progress is
/// abandoned; the returned digests are then incomplete and must be discarded.
pub(crate) fn digest_parts(parts: &[&[u8]], threads: usize, stop: &AtomicBool) -> Vec<Hash> {
    let threads = threads.max(1).min(parts.len());
    let next = AtomicUsize::new(0);
    let work = || {
        let mut done = Vec::new();
        while !stop.load(Ordering::Relaxed) {
            let k = next.fetch_add(1, Ordering::Relaxed);
            let Some(part) = parts.get(k) else {
                break;
            };
            match digest_unless_stopped(part, stop) {
                Some(hash) => done.push((k, hash)),
                None => break,
            }
        }
        done
    };
    let mut out = vec![[0u8; 32]; parts.len()];
    if threads <= 1 {
        for (k, hash) in work() {
            out[k] = hash;
        }
        return out;
    }
    std::thread::scope(|scope| {
        let workers: Vec<_> = (1..threads)
            .filter_map(|_| std::thread::Builder::new().spawn_scoped(scope, work).ok())
            .collect();
        // The calling thread takes parts too, so the queue is emptied even if
        // no thread could be started.
        for (k, hash) in work() {
            out[k] = hash;
        }
        for worker in workers {
            // A digest cannot panic short of a bug; propagate it if it does.
            for (k, hash) in worker
                .join()
                .unwrap_or_else(|e| std::panic::resume_unwind(e))
            {
                out[k] = hash;
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_parts_matches_serial_for_every_thread_count() {
        let data: Vec<u8> = (0..(3 * PIECE + 20_000) as u32)
            .map(|i| (i * 31 % 251) as u8)
            .collect();
        let cuts = [
            0usize,
            0,
            1,
            64,
            65,
            1000,
            4096,
            4097,
            12_345,
            20_000,
            20_000 + PIECE - 1,
            20_000 + 2 * PIECE - 1,
            20_000 + 3 * PIECE,
            3 * PIECE + 20_000,
        ];
        let parts: Vec<&[u8]> = cuts.windows(2).map(|w| &data[w[0]..w[1]]).collect();
        let lengths = [PIECE - 1, PIECE, PIECE + 1, 2 * PIECE + 1, 3 * PIECE];
        let mut sized: Vec<&[u8]> = lengths.iter().map(|&n| &data[..n]).collect();
        sized.extend(&parts);
        let serial: Vec<Hash> = sized.iter().map(|p| digest(p)).collect();
        let go = AtomicBool::new(false);
        for threads in 0..=12 {
            assert_eq!(
                digest_parts(&sized, threads, &go),
                serial,
                "threads={threads}"
            );
        }
        assert!(digest_parts(&[], 4, &go).is_empty());
    }

    #[test]
    fn digest_parts_starts_nothing_after_a_stop() {
        let data = vec![7u8; 2 * PIECE + 3];
        let parts: Vec<&[u8]> = vec![&data, &data[..5], &data[1..]];
        let stopped = AtomicBool::new(true);
        for threads in 0..=6 {
            assert_eq!(digest_parts(&parts, threads, &stopped), vec![[0u8; 32]; 3]);
        }
        assert_eq!(digest_unless_stopped(&data, &stopped), None);
        assert_eq!(digest_unless_stopped(&[], &stopped), Some(digest(&[])));
    }

    #[test]
    fn a_matching_pin_returns_rest_and_a_wrong_pin_stops_it() {
        let data: Vec<u8> = (0..5000u32).map(|i| (i * 7 % 256) as u8).collect();
        let pin = digest(&data);
        for threads in 2..=9 {
            let got = pinned(&data, pin, threads, |left, stop| {
                (left, stop.load(Ordering::Relaxed))
            });
            assert_eq!(got, Some((threads - 1, false)));
            let mut wrong = pin;
            wrong[31] ^= 1;
            // This `rest` ends only when it sees the stop flag.
            let got = pinned(&data, wrong, threads, |left, stop| {
                while !stop.load(Ordering::Relaxed) {
                    std::hint::spin_loop();
                }
                left
            });
            assert_eq!(got, None);
        }
    }

    #[test]
    fn short_inputs_stay_on_one_thread() {
        assert_eq!(threads_for(0), 1);
        assert_eq!(threads_for(PARALLEL_MIN - 1), 1);
        assert!((1..=MAX_THREADS).contains(&threads_for(PARALLEL_MIN)));
    }
}
