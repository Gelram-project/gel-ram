//! The pointer-chase chain. `chain_order` is the seeded Fisher-Yates
//! permutation; it depends only on the working-set size, so it is built once
//! per size. `build_next` turns it into the successor table that a timed
//! round walks; each round builds its own table right before its timer.

use gel_core::splitmix64;

/// Visit order of the chain for a `bytes`-sized working set (at least two
/// slots).
pub fn chain_order(bytes: usize) -> Vec<usize> {
    let slots = (bytes / core::mem::size_of::<usize>()).max(2);
    let mut order = (0..slots).collect::<Vec<_>>();
    let mut state = 0x4745_4c00_5048_5953u64 ^ bytes as u64;
    for i in (1..slots).rev() {
        state = splitmix64(state);
        let j = (state as usize) % (i + 1);
        order.swap(i, j);
    }
    order
}

/// Fresh successor table: `next[order[w]] = order[w + 1]`, closing the cycle
/// at the last slot.
pub fn build_next(order: &[usize]) -> Vec<usize> {
    let slots = order.len();
    let mut next = vec![0usize; slots];
    for pair in order.windows(2) {
        next[pair[0]] = pair[1];
    }
    next[*order.last().expect("nonempty")] = order[0];
    next
}
