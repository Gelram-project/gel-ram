//! Exact copy of the earlier inline chain builder of `pointer_chase`, and
//! the comparison that checks `chain` against it. Used by the unit tests and
//! by `gel-bench/examples/identity_bench_setup.rs`.

use crate::chain;
use gel_core::splitmix64;

/// The earlier builder: (order, next, first cursor, steps) for one round.
pub fn old_chain(bytes: usize) -> (Vec<usize>, Vec<usize>, usize, usize) {
    let slots = (bytes / core::mem::size_of::<usize>()).max(2);
    let mut order = (0..slots).collect::<Vec<_>>();
    let mut state = 0x4745_4c00_5048_5953u64 ^ bytes as u64;
    for i in (1..slots).rev() {
        state = splitmix64(state);
        let j = (state as usize) % (i + 1);
        order.swap(i, j);
    }
    let mut next = vec![0usize; slots];
    for pair in order.windows(2) {
        next[pair[0]] = pair[1];
    }
    next[*order.last().expect("nonempty")] = order[0];
    let steps = slots.saturating_mul(4).max(32_768);
    let cursor = order[0];
    (order, next, cursor, steps)
}

/// For each size: the order built once, and the table built twice from it
/// (as two timed rounds do), against the earlier per-round builder. Returns
/// (cases, equal).
pub fn compare_chain(sizes: &[usize]) -> (usize, usize) {
    let mut cases = 0;
    let mut equal = 0;
    for &bytes in sizes {
        let (old_order, old_next, old_cursor, old_steps) = old_chain(bytes);
        let order = chain::chain_order(bytes);
        let steps = order.len().saturating_mul(4).max(32_768);
        cases += 1;
        equal += usize::from(order == old_order && order[0] == old_cursor && steps == old_steps);
        for _round in 0..2 {
            cases += 1;
            equal += usize::from(chain::build_next(&order) == old_next);
        }
    }
    (cases, equal)
}
