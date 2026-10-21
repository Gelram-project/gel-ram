//! Differential check for the popcount-first encoder paths.
//!
//! The `reference_*` functions are the earlier `from_exact_xor`,
//! `pack_positions`, `encode_against` and `encode_best_prototype`, kept
//! verbatim apart from their names so the current paths can be compared with
//! them bit for bit. This file is compiled into the crate's unit tests and,
//! through `#[path]`, into the `identity_structural_encode` example; in both
//! places the parent module provides the crate's public types.

use super::{encode_best_prototype, EncodedOrb, ParentRef, Residual};
use gel_core::{splitmix64, GelError, ORB_BITS, ORB_BYTES, ORB_WORDS};
use gel_orb::Orb1024;

const SPARSE_INDEX_BITS: usize = 10;
const MAX_DELTA_DEPTH: u8 = 2;

fn reference_from_exact_xor(target: &Orb1024, predictor: &Orb1024) -> Residual {
    let mut delta = [0u64; ORB_WORDS];
    let mut positions = Vec::new();
    for (i, ((slot, &target_word), &predictor_word)) in delta
        .iter_mut()
        .zip(target.words().iter())
        .zip(predictor.words().iter())
        .enumerate()
    {
        *slot = target_word ^ predictor_word;
        let mut word = *slot;
        while word != 0 {
            let bit = word.trailing_zeros() as usize;
            positions.push((i * 64 + bit) as u16);
            word &= word - 1;
        }
    }
    let sparse = reference_pack_positions(&positions);
    let sparse_len = 1 + 2 + sparse.len();
    let dense_len = 1 + ORB_BYTES;
    if sparse_len < dense_len {
        Residual::Sparse {
            count: positions.len() as u16,
            packed_positions: sparse,
        }
    } else {
        Residual::Dense(Orb1024::from_words(delta))
    }
}

fn reference_pack_positions(positions: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity((positions.len() * SPARSE_INDEX_BITS).div_ceil(8));
    let mut acc = 0u64;
    let mut bits = 0usize;
    for &position in positions {
        acc |= (position as u64) << bits;
        bits += SPARSE_INDEX_BITS;
        while bits >= 8 {
            out.push(acc as u8);
            acc >>= 8;
            bits -= 8;
        }
    }
    if bits != 0 {
        out.push(acc as u8);
    }
    out
}

fn reference_encode_against(
    target: &Orb1024,
    predictor: &Orb1024,
    parent: ParentRef,
    parent_depth: u8,
) -> Result<EncodedOrb, GelError> {
    if parent_depth >= MAX_DELTA_DEPTH {
        return Err(GelError::InvalidResidual(
            "delta parent depth would exceed 2",
        ));
    }
    let child_depth = parent_depth + 1;
    let residual = reference_from_exact_xor(target, predictor);
    let delta_len = 1 + 1 + parent.serialized_len() + residual.serialized_len();
    let literal_len = 1 + ORB_BYTES;
    if delta_len < literal_len {
        Ok(EncodedOrb::Delta {
            parent,
            parent_depth: child_depth,
            residual,
        })
    } else {
        Ok(EncodedOrb::Literal(*target))
    }
}

fn reference_encode_best_prototype(
    target: &Orb1024,
    prototypes: &[Orb1024],
) -> Result<(EncodedOrb, Option<usize>), GelError> {
    let mut best = EncodedOrb::Literal(*target);
    let mut best_index = None;
    for (index, prototype) in prototypes.iter().enumerate() {
        let prototype_id = u32::try_from(index)
            .map_err(|_| GelError::InvalidResidual("prototype index exceeds u32"))?;
        let candidate =
            reference_encode_against(target, prototype, ParentRef::Prototype(prototype_id), 0)?;
        if candidate.serialized_len() < best.serialized_len() {
            best = candidate;
            best_index = Some(index);
        }
    }
    Ok((best, best_index))
}

/// One named group of comparisons.
pub struct IdentityCase {
    pub name: &'static str,
    pub cases: usize,
    pub equal: usize,
}

struct Tally {
    cases: usize,
    equal: usize,
}

impl Tally {
    fn new() -> Self {
        Self { cases: 0, equal: 0 }
    }

    fn check<T: PartialEq>(&mut self, current: T, reference: T) {
        self.cases += 1;
        if current == reference {
            self.equal += 1;
        }
    }

    fn finish(self, name: &'static str) -> IdentityCase {
        IdentityCase {
            name,
            cases: self.cases,
            equal: self.equal,
        }
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix64(self.0)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn orb(&mut self) -> Orb1024 {
        let mut words = [0u64; ORB_WORDS];
        for word in &mut words {
            *word = self.next();
        }
        Orb1024::from_words(words)
    }
}

#[derive(Clone, Copy)]
enum Layout {
    Spread,
    Cluster,
    Low,
    High,
}

const LAYOUTS: [Layout; 4] = [Layout::Spread, Layout::Cluster, Layout::Low, Layout::High];

/// A mask with exactly `k` distinct set bits placed according to `layout`.
fn mask(k: usize, layout: Layout, rng: &mut Rng) -> [u64; ORB_WORDS] {
    let mut words = [0u64; ORB_WORDS];
    let mut set = |position: usize| words[position >> 6] |= 1u64 << (position & 63);
    match layout {
        Layout::Spread => {
            let mut order: Vec<usize> = (0..ORB_BITS).collect();
            for i in 0..k {
                let j = i + rng.below(ORB_BITS - i);
                order.swap(i, j);
                set(order[i]);
            }
        }
        Layout::Cluster => {
            let start = rng.below(ORB_BITS);
            for offset in 0..k {
                set((start + offset) % ORB_BITS);
            }
        }
        Layout::Low => (0..k).for_each(&mut set),
        Layout::High => (ORB_BITS - k..ORB_BITS).for_each(&mut set),
    }
    words
}

fn xor(orb: &Orb1024, mask: &[u64; ORB_WORDS]) -> Orb1024 {
    let mut out = *orb;
    for (word, &m) in out.words_mut().iter_mut().zip(mask.iter()) {
        *word ^= m;
    }
    out
}

fn near(orb: &Orb1024, k: usize, rng: &mut Rng) -> Orb1024 {
    let layout = LAYOUTS[rng.below(LAYOUTS.len())];
    xor(orb, &mask(k, layout, rng))
}

fn residual_sweep() -> IdentityCase {
    let mut tally = Tally::new();
    let mut rng = Rng(0x4745_4C5F_5352_5731);
    for seed in 0..6 {
        let predictor = if seed == 0 { Orb1024::ZERO } else { rng.orb() };
        for k in 0..=ORB_BITS {
            for layout in LAYOUTS {
                let target = xor(&predictor, &mask(k, layout, &mut rng));
                tally.check(
                    Residual::from_exact_xor(&target, &predictor),
                    reference_from_exact_xor(&target, &predictor),
                );
            }
        }
    }
    tally.finish("from_exact_xor_k_sweep")
}

fn residual_random_pairs() -> IdentityCase {
    let mut tally = Tally::new();
    let mut rng = Rng(0x4745_4C5F_5241_4E44);
    for _ in 0..8192 {
        let target = rng.orb();
        let predictor = rng.orb();
        tally.check(
            Residual::from_exact_xor(&target, &predictor),
            reference_from_exact_xor(&target, &predictor),
        );
    }
    tally.finish("from_exact_xor_random_pairs")
}

fn residual_edges() -> IdentityCase {
    let mut tally = Tally::new();
    let ones = Orb1024::from_words([u64::MAX; ORB_WORDS]);
    let mut pairs = vec![
        (Orb1024::ZERO, Orb1024::ZERO),
        (ones, Orb1024::ZERO),
        (Orb1024::ZERO, ones),
        (ones, ones),
    ];
    for position in 0..ORB_BITS {
        let mut single = Orb1024::ZERO;
        single.words_mut()[position >> 6] = 1u64 << (position & 63);
        pairs.push((single, Orb1024::ZERO));
        pairs.push((Orb1024::ZERO, xor(&ones, single.words())));
    }
    // Break-even counts placed so they start, end or straddle word boundaries.
    for k in [92usize, 93, 94, 95, 96, 97, 99, 100, 101, 102] {
        for start in [0usize, 1, 63, 64, 127, 960, ORB_BITS - k, ORB_BITS - 1] {
            let mut words = [0u64; ORB_WORDS];
            for offset in 0..k {
                let position = (start + offset) % ORB_BITS;
                words[position >> 6] |= 1u64 << (position & 63);
            }
            pairs.push((Orb1024::from_words(words), Orb1024::ZERO));
        }
    }
    for (target, predictor) in pairs {
        tally.check(
            Residual::from_exact_xor(&target, &predictor),
            reference_from_exact_xor(&target, &predictor),
        );
    }
    tally.finish("from_exact_xor_edges")
}

fn encode_against_sweep() -> IdentityCase {
    let mut tally = Tally::new();
    let mut rng = Rng(0x4745_4C5F_454E_4341);
    let parents = [
        ParentRef::Prototype(0),
        ParentRef::Prototype(u32::MAX),
        ParentRef::SegmentLocal(0),
        ParentRef::SegmentLocal(u16::MAX),
    ];
    for k in 0..=ORB_BITS {
        let predictor = rng.orb();
        let target = near(&predictor, k, &mut rng);
        for parent in parents {
            for depth in 0..=MAX_DELTA_DEPTH + 1 {
                tally.check(
                    EncodedOrb::encode_against(&target, &predictor, parent, depth),
                    reference_encode_against(&target, &predictor, parent, depth),
                );
            }
        }
    }
    tally.finish("encode_against_sweep")
}

fn best_prototype_pools() -> IdentityCase {
    let mut tally = Tally::new();
    let mut rng = Rng(0x4745_4C5F_504F_4F4C);
    for round in 0..3000 {
        let size = round % 65;
        let target = rng.orb();
        let mut pool = Vec::with_capacity(size);
        for _ in 0..size {
            let member = match rng.below(8) {
                0 | 1 => rng.orb(),
                2 => target,
                3 if !pool.is_empty() => pool[rng.below(pool.len())],
                4 => near(&target, 90 + rng.below(10), &mut rng),
                _ => near(&target, rng.below(130), &mut rng),
            };
            pool.push(member);
        }
        tally.check(
            encode_best_prototype(&target, &pool),
            reference_encode_best_prototype(&target, &pool),
        );
    }
    tally.finish("best_prototype_pools")
}

fn best_prototype_ties() -> IdentityCase {
    let mut tally = Tally::new();
    let mut rng = Rng(0x4745_4C5F_5449_4553);
    // Several different members at the same distance, mixed with farther
    // ones, for distances on both sides of the 94/95-bit break-even.
    for k in (0..=110).chain([200, 512, 1024]) {
        for round in 0..24 {
            let target = rng.orb();
            let size = 2 + round % 23;
            let mut pool = Vec::with_capacity(size);
            for _ in 0..size {
                let member = if rng.below(3) == 0 {
                    near(&target, (k + 1 + rng.below(40)).min(ORB_BITS), &mut rng)
                } else {
                    near(&target, k, &mut rng)
                };
                pool.push(member);
            }
            tally.check(
                encode_best_prototype(&target, &pool),
                reference_encode_best_prototype(&target, &pool),
            );
        }
    }
    let target = rng.orb();
    tally.check(
        encode_best_prototype(&target, &[]),
        reference_encode_best_prototype(&target, &[]),
    );
    let same = vec![near(&target, 50, &mut rng); 64];
    tally.check(
        encode_best_prototype(&target, &same),
        reference_encode_best_prototype(&target, &same),
    );
    tally.finish("best_prototype_ties")
}

/// Runs every comparison group, in a fixed order.
pub fn identity_cases() -> Vec<IdentityCase> {
    vec![
        residual_sweep(),
        residual_random_pairs(),
        residual_edges(),
        encode_against_sweep(),
        best_prototype_pools(),
        best_prototype_ties(),
    ]
}
