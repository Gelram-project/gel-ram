//! Identity check for the blocked `top1_many` (and `top1_many_threads`, which
//! runs it per chunk) against an exact copy of the earlier per-ORB loop (every
//! ORB scored against all queries before the next ORB) and against
//! `top1(bank, &queries[q])`: index, score and the same ORB reference.
//! Seeded banks: random, a small repeated pool (equal scores in every block),
//! all equal, alternating all-zero and all-one ORBs, and low-weight ORBs;
//! lengths around the 128-ORB block edges; query counts 0..=33 including
//! queries taken from the bank; thread counts below, at and above the
//! parallel-path limits.
//!
//! Usage: `cargo run --release -p gel-reader --example identity_top1_many`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when no case
//! ran); exit status 0 on PASS, 1 on FAIL. The integration test
//! `tests/identity_top1_many.rs` runs the same comparison at a smaller scale.
//! Output grammar: `docs/SPEED.md`.
#![forbid(unsafe_code)]

use gel_core::{splitmix64, ORB_WORDS};
use gel_orb::Orb1024;
use gel_reader::{top1, top1_many, top1_many_threads};

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

type Hit<'a> = Option<(usize, u16, &'a Orb1024)>;

mod reference {
    use gel_orb::Orb1024;
    use gel_reader::score_xnor;

    /// Exact copy of the earlier `top1_many` (one pass, per ORB, all queries).
    pub fn top1_many<'a>(
        bank: &'a [Orb1024],
        queries: &[Orb1024],
    ) -> Vec<Option<(usize, u16, &'a Orb1024)>> {
        let mut best: Vec<Option<(usize, u16, &'a Orb1024)>> = vec![None; queries.len()];
        if queries.is_empty() {
            return best;
        }
        for (index, orb) in bank.iter().enumerate() {
            for (slot, query) in best.iter_mut().zip(queries) {
                let score = score_xnor(orb, query);
                if slot.as_ref().is_none_or(|x| score > x.1) {
                    *slot = Some((index, score, orb));
                }
            }
        }
        best
    }
}

// How many inputs the comparison covers: `Scale::Full` from `main`,
// `Scale::Small` from the test wrapper.

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn random_orb(state: &mut u64) -> Orb1024 {
    Orb1024::from_words(std::array::from_fn(|_| next(state)))
}

fn low_weight_orb(state: &mut u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for _ in 0..(next(state) % 6) {
        let bit = (next(state) % 1024) as usize;
        words[bit / 64] |= 1 << (bit % 64);
    }
    Orb1024::from_words(words)
}

fn banks(len: usize, seed: u64) -> Vec<(&'static str, Vec<Orb1024>)> {
    let mut state = seed;
    let random: Vec<Orb1024> = (0..len).map(|_| random_orb(&mut state)).collect();
    let pool: Vec<Orb1024> = (0..3).map(|_| random_orb(&mut state)).collect();
    let pooled = (0..len)
        .map(|_| pool[(next(&mut state) % 3) as usize])
        .collect();
    let equal = vec![random_orb(&mut state); len];
    let zero = Orb1024::from_words([0; ORB_WORDS]);
    let ones = Orb1024::from_words([u64::MAX; ORB_WORDS]);
    let alternating = (0..len)
        .map(|i| if i % 2 == 0 { zero } else { ones })
        .collect();
    let low = (0..len).map(|_| low_weight_orb(&mut state)).collect();
    vec![
        ("random", random),
        ("pooled", pooled),
        ("equal", equal),
        ("alternating", alternating),
        ("low_weight", low),
    ]
}

fn queries(bank: &[Orb1024], count: usize, seed: u64) -> Vec<Orb1024> {
    let mut state = seed;
    (0..count)
        .map(|q| match q % 4 {
            0 if !bank.is_empty() => bank[(next(&mut state) as usize) % bank.len()],
            1 => Orb1024::from_words([0; ORB_WORDS]),
            2 => Orb1024::from_words([u64::MAX; ORB_WORDS]),
            _ => random_orb(&mut state),
        })
        .collect()
}

fn same(a: Hit<'_>, b: Hit<'_>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some((ia, sa, oa)), Some((ib, sb, ob))) => ia == ib && sa == sb && std::ptr::eq(oa, ob),
        _ => false,
    }
}

fn compare(got: &[Hit<'_>], want: &[Hit<'_>], tally: &mut Tally, lengths: &mut Tally) {
    lengths.check(got.len() == want.len());
    for (q, want) in want.iter().enumerate() {
        tally.check(got.get(q).is_some_and(|got| same(*got, *want)));
    }
}

/// Runs every comparison and returns one tally per named case.
pub fn run(scale: Scale) -> Vec<Tally> {
    let lens: &[usize] = match scale {
        Scale::Small => &[0, 1, 2, 127, 128, 129, 257, 1000],
        Scale::Full => &[
            0, 1, 2, 3, 63, 127, 128, 129, 255, 256, 257, 383, 384, 385, 1000, 4097, 20_000,
        ],
    };
    let counts: &[usize] = match scale {
        Scale::Small => &[0, 1, 2, 3, 16],
        Scale::Full => &[0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 32, 33],
    };
    let threads: &[usize] = &[0, 1, 2, 3, 8, 257, usize::MAX];
    let mut vs_reference = Tally::new("top1_many_equals_reference");
    let mut vs_top1 = Tally::new("top1_many_equals_top1");
    let mut threads_vs_reference = Tally::new("top1_many_threads_equals_reference");
    let mut lengths = Tally::new("result_lengths");
    for (li, &len) in lens.iter().enumerate() {
        for (kind, bank) in banks(len, 0x2026_1020_5103_0000 ^ li as u64) {
            for (ci, &count) in counts.iter().enumerate() {
                let qs = queries(&bank, count, 0x7031_0016_0000_0000 ^ (li * 64 + ci) as u64);
                let want = reference::top1_many(&bank, &qs);
                let got = top1_many(&bank, &qs);
                compare(&got, &want, &mut vs_reference, &mut lengths);
                let single: Vec<Hit<'_>> = qs.iter().map(|q| top1(&bank, q)).collect();
                compare(&got, &single, &mut vs_top1, &mut lengths);
                for &t in threads {
                    let threaded = top1_many_threads(&bank, &qs, t);
                    compare(&threaded, &want, &mut threads_vs_reference, &mut lengths);
                }
                // Equal-score inputs are where block order matters most.
                if kind == "pooled" && count == 0 && len > 0 {
                    let edge: Vec<Orb1024> = vec![bank[len - 1]; 2];
                    let want = reference::top1_many(&bank, &edge);
                    compare(
                        &top1_many(&bank, &edge),
                        &want,
                        &mut vs_reference,
                        &mut lengths,
                    );
                }
            }
        }
    }
    vec![vs_reference, vs_top1, threads_vs_reference, lengths]
}

fn main() -> std::process::ExitCode {
    identity::report(&run(Scale::Full))
}
