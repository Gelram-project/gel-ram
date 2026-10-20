#![forbid(unsafe_code)]

//! F2 structural codec: exact XOR prediction plus compact residuals.
//! No floating-point prediction is used; decode is bit-exact by construction.

use gel_core::{GelError, ORB_BITS, ORB_BYTES, ORB_WORDS};
use gel_orb::Orb1024;

pub const MAX_DELTA_DEPTH: u8 = 2;
pub const SPARSE_INDEX_BITS: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParentRef {
    Prototype(u32),
    SegmentLocal(u16),
}

impl ParentRef {
    pub const fn serialized_len(self) -> usize {
        match self {
            Self::Prototype(_) => 1 + 4,
            Self::SegmentLocal(_) => 1 + 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Residual {
    Sparse {
        count: u16,
        packed_positions: Vec<u8>,
    },
    Dense(Orb1024),
}

impl Residual {
    pub fn from_exact_xor(target: &Orb1024, predictor: &Orb1024) -> Self {
        let mut delta = [0u64; ORB_WORDS];
        let mut count = 0usize;
        for ((slot, &target_word), &predictor_word) in delta
            .iter_mut()
            .zip(target.words().iter())
            .zip(predictor.words().iter())
        {
            *slot = target_word ^ predictor_word;
            count += slot.count_ones() as usize;
        }
        // The packed length depends only on the number of changed bits, so the
        // sparse/dense choice is made before any position is written.
        let packed_len = (count * SPARSE_INDEX_BITS).div_ceil(8);
        let sparse_len = 1 + 2 + packed_len;
        let dense_len = 1 + ORB_BYTES;
        if sparse_len < dense_len {
            Self::Sparse {
                count: count as u16,
                packed_positions: pack_delta_positions(&delta, packed_len),
            }
        } else {
            Self::Dense(Orb1024::from_words(delta))
        }
    }

    pub fn popcount(&self) -> Result<u16, GelError> {
        match self {
            Self::Sparse {
                count,
                packed_positions,
            } => {
                // Counts are meaningful only for a canonical decodable residual.
                for_each_position(*count, packed_positions, |_| {})?;
                Ok(*count)
            }
            Self::Dense(delta) => Ok(delta.words().iter().map(|x| x.count_ones() as u16).sum()),
        }
    }

    pub fn serialized_len(&self) -> usize {
        match self {
            Self::Sparse {
                packed_positions, ..
            } => 1 + 2 + packed_positions.len(),
            Self::Dense(_) => 1 + ORB_BYTES,
        }
    }

    pub fn apply(&self, predictor: &Orb1024) -> Result<Orb1024, GelError> {
        match self {
            Self::Dense(delta) => {
                let mut out = [0u64; ORB_WORDS];
                for ((slot, &predictor_word), &delta_word) in out
                    .iter_mut()
                    .zip(predictor.words().iter())
                    .zip(delta.words().iter())
                {
                    *slot = predictor_word ^ delta_word;
                }
                Ok(Orb1024::from_words(out))
            }
            Self::Sparse {
                count,
                packed_positions,
            } => {
                // Positions are applied as they are decoded; on an invalid
                // encoding the partly changed copy is dropped with the error.
                let mut out = *predictor;
                for_each_position(*count, packed_positions, |position| {
                    let word = position as usize >> 6;
                    let bit = position as usize & 63;
                    out.words_mut()[word] ^= 1u64 << bit;
                })?;
                Ok(out)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodedOrb {
    Literal(Orb1024),
    Delta {
        parent: ParentRef,
        parent_depth: u8,
        residual: Residual,
    },
}

impl EncodedOrb {
    pub fn encode_against(
        target: &Orb1024,
        predictor: &Orb1024,
        parent: ParentRef,
        parent_depth: u8,
    ) -> Result<Self, GelError> {
        if parent_depth >= MAX_DELTA_DEPTH {
            return Err(GelError::InvalidResidual(
                "delta parent depth would exceed 2",
            ));
        }
        let child_depth = parent_depth + 1;
        let residual = Residual::from_exact_xor(target, predictor);
        let delta_len = 1 + 1 + parent.serialized_len() + residual.serialized_len();
        let literal_len = 1 + ORB_BYTES;
        if delta_len < literal_len {
            Ok(Self::Delta {
                parent,
                parent_depth: child_depth,
                residual,
            })
        } else {
            Ok(Self::Literal(*target))
        }
    }

    pub fn literal(orb: Orb1024) -> Self {
        Self::Literal(orb)
    }

    pub fn decode(&self, predictor: Option<&Orb1024>) -> Result<Orb1024, GelError> {
        match self {
            Self::Literal(orb) => Ok(*orb),
            Self::Delta {
                parent_depth,
                residual,
                ..
            } => {
                if *parent_depth == 0 || *parent_depth > MAX_DELTA_DEPTH {
                    return Err(GelError::InvalidResidual("invalid stored delta depth"));
                }
                let predictor =
                    predictor.ok_or(GelError::InvalidResidual("delta requires predictor"))?;
                residual.apply(predictor)
            }
        }
    }

    pub fn serialized_len(&self) -> usize {
        match self {
            Self::Literal(_) => 1 + ORB_BYTES,
            Self::Delta {
                parent, residual, ..
            } => 1 + 1 + parent.serialized_len() + residual.serialized_len(),
        }
    }

    pub fn residual_popcount(&self) -> Result<u16, GelError> {
        match self {
            Self::Literal(_) => Ok(ORB_BITS as u16),
            Self::Delta { residual, .. } => residual.popcount(),
        }
    }

    pub fn local_der(&self) -> f64 {
        ORB_BYTES as f64 / self.serialized_len() as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StructuralMetrics {
    pub exact_bytes: usize,
    pub physical_bytes: usize,
    pub context_touch_bytes: usize,
    pub residual_popcount: u16,
    pub der_local: f64,
}

impl StructuralMetrics {
    pub fn from_encoded(encoded: &EncodedOrb, predictor_touched: bool) -> Result<Self, GelError> {
        let physical_bytes = encoded.serialized_len();
        let context_touch_bytes = if predictor_touched { ORB_BYTES } else { 0 };
        Ok(Self {
            exact_bytes: ORB_BYTES,
            physical_bytes,
            context_touch_bytes,
            residual_popcount: encoded.residual_popcount()?,
            der_local: ORB_BYTES as f64 / physical_bytes as f64,
        })
    }
}

/// Search a bounded prototype pool and choose the smallest exact encoding.
/// Ties are deterministic: the lowest prototype index wins.
pub fn encode_best_prototype(
    target: &Orb1024,
    prototypes: &[Orb1024],
) -> Result<(EncodedOrb, Option<usize>), GelError> {
    // A prototype delta is 1 + 1 + 5 + 1 + 2 + ceil(10k / 8) bytes for k changed
    // bits while its residual is sparse; that length grows strictly with k, and
    // a dense residual (k > 100) is never smaller than the literal. So the
    // smallest encoding comes from the first prototype with the fewest changed
    // bits, or is the literal when even that one does not beat it.
    let mut best: Option<(usize, u32, u16)> = None;
    for (index, prototype) in prototypes.iter().enumerate() {
        let prototype_id = u32::try_from(index)
            .map_err(|_| GelError::InvalidResidual("prototype index exceeds u32"))?;
        let changed = target.hamming_distance(prototype);
        if best.is_none_or(|(_, _, best_changed)| changed < best_changed) {
            best = Some((index, prototype_id, changed));
        }
    }
    let Some((index, prototype_id, _)) = best else {
        return Ok((EncodedOrb::Literal(*target), None));
    };
    let encoded = EncodedOrb::encode_against(
        target,
        &prototypes[index],
        ParentRef::Prototype(prototype_id),
        0,
    )?;
    let chosen = matches!(encoded, EncodedOrb::Delta { .. }).then_some(index);
    Ok((encoded, chosen))
}

/// Packs the set bits of `delta` in ascending order as 10-bit fields, the
/// same bytes `pack_positions` writes for that position list.
fn pack_delta_positions(delta: &[u64; ORB_WORDS], packed_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(packed_len);
    let mut acc = 0u64;
    let mut bits = 0usize;
    for (i, &delta_word) in delta.iter().enumerate() {
        let mut word = delta_word;
        while word != 0 {
            let position = (i * 64 + word.trailing_zeros() as usize) as u64;
            acc |= position << bits;
            bits += SPARSE_INDEX_BITS;
            while bits >= 8 {
                out.push(acc as u8);
                acc >>= 8;
                bits -= 8;
            }
            word &= word - 1;
        }
    }
    if bits != 0 {
        out.push(acc as u8);
    }
    out
}

#[cfg(test)]
fn pack_positions(positions: &[u16]) -> Vec<u8> {
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

/// Decodes the packed positions in order and passes each one to `visit`,
/// without collecting them. The checks and their order are those of the
/// reference `unpack_positions`: count and length first, then per position
/// (a missing byte, a value outside the ORB, an order that is not strictly
/// increasing), then the padding bits and trailing bytes at the end. On an
/// error some positions may already have been visited.
fn for_each_position(
    count: u16,
    packed: &[u8],
    mut visit: impl FnMut(u16),
) -> Result<(), GelError> {
    validate_sparse(count, packed)?;
    let mut acc = 0u64;
    let mut bits = 0usize;
    let mut input = packed.iter().copied();
    let mut last: Option<u16> = None;
    for _ in 0..count {
        while bits < SPARSE_INDEX_BITS {
            let byte = input
                .next()
                .ok_or(GelError::InvalidResidual("truncated sparse positions"))?;
            acc |= (byte as u64) << bits;
            bits += 8;
        }
        let position = (acc & ((1u64 << SPARSE_INDEX_BITS) - 1)) as u16;
        acc >>= SPARSE_INDEX_BITS;
        bits -= SPARSE_INDEX_BITS;
        if position as usize >= ORB_BITS {
            return Err(GelError::InvalidResidual("sparse position outside ORB"));
        }
        if last.is_some_and(|last| position <= last) {
            return Err(GelError::InvalidResidual(
                "sparse positions must be strictly increasing",
            ));
        }
        visit(position);
        last = Some(position);
    }
    // The encoder writes zero padding bits after the last position and no
    // trailing bytes; anything else is not a canonical encoding.
    if acc != 0 || input.next().is_some() {
        return Err(GelError::InvalidResidual(
            "sparse padding bits must be zero",
        ));
    }
    Ok(())
}

/// The collecting decoder `apply` and `popcount` used before
/// `for_each_position`; kept as the reference it is compared against.
#[cfg(test)]
fn unpack_positions(count: u16, packed: &[u8]) -> Result<Vec<u16>, GelError> {
    validate_sparse(count, packed)?;
    let mut out = Vec::with_capacity(count as usize);
    let mut acc = 0u64;
    let mut bits = 0usize;
    let mut input = packed.iter().copied();
    while out.len() < count as usize {
        while bits < SPARSE_INDEX_BITS {
            let byte = input
                .next()
                .ok_or(GelError::InvalidResidual("truncated sparse positions"))?;
            acc |= (byte as u64) << bits;
            bits += 8;
        }
        let position = (acc & ((1u64 << SPARSE_INDEX_BITS) - 1)) as u16;
        acc >>= SPARSE_INDEX_BITS;
        bits -= SPARSE_INDEX_BITS;
        if position as usize >= ORB_BITS {
            return Err(GelError::InvalidResidual("sparse position outside ORB"));
        }
        if out.last().is_some_and(|last| position <= *last) {
            return Err(GelError::InvalidResidual(
                "sparse positions must be strictly increasing",
            ));
        }
        out.push(position);
    }
    // The encoder writes zero padding bits after the last position and no
    // trailing bytes; anything else is not a canonical encoding.
    if acc != 0 || input.next().is_some() {
        return Err(GelError::InvalidResidual(
            "sparse padding bits must be zero",
        ));
    }
    Ok(out)
}

fn validate_sparse(count: u16, packed: &[u8]) -> Result<(), GelError> {
    if count as usize > ORB_BITS {
        return Err(GelError::InvalidResidual("sparse count exceeds 1024"));
    }
    let expected = (count as usize * SPARSE_INDEX_BITS).div_ceil(8);
    if packed.len() != expected {
        return Err(GelError::InvalidLength {
            expected,
            actual: packed.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod popcount_first_reference;

#[cfg(test)]
mod tests {
    use super::*;
    use gel_core::{splitmix64, ORB_WORDS};

    #[test]
    fn popcount_first_paths_match_the_earlier_encoder_bit_for_bit() {
        let groups = crate::popcount_first_reference::identity_cases();
        let total: usize = groups.iter().map(|group| group.cases).sum();
        assert!(total > 0, "no comparison ran");
        for group in groups {
            assert!(group.cases > 0, "{} ran no comparison", group.name);
            assert_eq!(group.equal, group.cases, "{}", group.name);
        }
    }

    fn sample(seed: u64) -> Orb1024 {
        let mut words = [0u64; ORB_WORDS];
        for (i, word) in words.iter_mut().enumerate() {
            *word = splitmix64(seed ^ i as u64);
        }
        Orb1024::from_words(words)
    }

    fn flipped(base: Orb1024, count: usize) -> Orb1024 {
        let mut out = base;
        for bit in 0..count {
            out.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
        }
        out
    }

    #[test]
    fn exact_xor_roundtrip_is_bit_identical() {
        for seed in 0..128 {
            let predictor = sample(seed);
            for flips in [0usize, 1, 17, 100, 101, 512, 1024] {
                let target = flipped(predictor, flips);
                let residual = Residual::from_exact_xor(&target, &predictor);
                assert_eq!(residual.apply(&predictor).unwrap(), target);
                assert_eq!(residual.popcount().unwrap(), flips as u16);
            }
        }
    }

    #[test]
    fn measured_sparse_dense_break_even_is_100_bits() {
        let predictor = Orb1024::ZERO;
        let r100 = Residual::from_exact_xor(&flipped(predictor, 100), &predictor);
        let r101 = Residual::from_exact_xor(&flipped(predictor, 101), &predictor);
        assert!(matches!(&r100, Residual::Sparse { .. }));
        assert!(matches!(&r101, Residual::Dense(_)));
        assert_eq!(r100.serialized_len(), 128);
        assert_eq!(r101.serialized_len(), 129);
    }

    fn sparse(count: u16, packed_positions: Vec<u8>) -> Residual {
        Residual::Sparse {
            count,
            packed_positions,
        }
    }

    /// A valid three-position residual: 30 index bits in 4 bytes, so the top
    /// two bits of the last byte are padding.
    fn three_position_sparse() -> (Orb1024, Orb1024, u16, Vec<u8>) {
        let predictor = sample(30);
        let target = flipped(predictor, 3);
        let residual = Residual::from_exact_xor(&target, &predictor);
        assert_eq!(residual.apply(&predictor).unwrap(), target);
        let Residual::Sparse {
            count,
            packed_positions,
        } = residual
        else {
            panic!("three flips must encode as Sparse");
        };
        assert_eq!((count, packed_positions.len()), (3, 4));
        (predictor, target, count, packed_positions)
    }

    #[test]
    fn sparse_padding_bit_set_is_rejected() {
        let (predictor, target, count, packed) = three_position_sparse();
        assert_eq!(
            sparse(count, packed.clone()).apply(&predictor).unwrap(),
            target
        );
        for mask in [0x40u8, 0x80, 0xC0] {
            let mut corrupted = packed.clone();
            *corrupted.last_mut().unwrap() ^= mask;
            assert_eq!(
                sparse(count, corrupted).apply(&predictor),
                Err(GelError::InvalidResidual(
                    "sparse padding bits must be zero"
                )),
                "mask={mask:#x}"
            );
        }
    }

    #[test]
    fn sparse_truncated_packed_positions_are_rejected() {
        let (predictor, _, count, packed) = three_position_sparse();
        for keep in 0..packed.len() {
            assert_eq!(
                sparse(count, packed[..keep].to_vec()).apply(&predictor),
                Err(GelError::InvalidLength {
                    expected: packed.len(),
                    actual: keep,
                }),
                "keep={keep}"
            );
        }
    }

    #[test]
    fn sparse_count_and_packed_length_mismatch_is_rejected() {
        let (predictor, _, count, packed) = three_position_sparse();
        let mut longer = packed.clone();
        longer.push(0);
        assert!(sparse(count, longer).apply(&predictor).is_err());
        assert!(sparse(count + 1, packed.clone()).apply(&predictor).is_err());
        assert!(sparse(count - 1, packed.clone()).apply(&predictor).is_err());
        assert!(sparse(count + 1, packed.clone()).popcount().is_err());
        assert!(sparse(1025, vec![0; 1282]).popcount().is_err());
    }

    #[test]
    fn sparse_popcount_and_apply_reject_the_same_invalid_encodings() {
        let predictor = sample(811);
        let invalid = [
            sparse(2, vec![0, 0, 0]),
            sparse(2, pack_positions(&[9, 3])),
            sparse(1, vec![0, 128]),
            sparse(0, vec![0]),
            sparse(2, vec![0, 0]),
            sparse(1025, vec![0; 1282]),
        ];
        for residual in invalid {
            assert!(residual.popcount().is_err(), "{residual:?}");
            assert!(residual.apply(&predictor).is_err(), "{residual:?}");
        }
        for count in 0..=1024u16 {
            let positions: Vec<_> = (0..count).collect();
            let residual = sparse(count, pack_positions(&positions));
            assert_eq!(residual.popcount().unwrap(), count);
            assert!(residual.apply(&predictor).is_ok());
        }
    }

    /// The streaming decoder must accept, refuse and order exactly as the
    /// collecting reference does, with the same error, on valid packings,
    /// bit flips of them, wrong lengths and random bytes.
    #[test]
    fn streaming_positions_equal_the_collecting_reference() {
        fn streamed(count: u16, packed: &[u8]) -> Result<Vec<u16>, GelError> {
            let mut out = Vec::new();
            for_each_position(count, packed, |position| out.push(position))?;
            Ok(out)
        }
        let predictor = sample(812);
        let mut checked = 0usize;
        let mut seed = 0x5354_5255_4354_0001u64;
        let mut cases: Vec<(u16, Vec<u8>)> = Vec::new();
        for count in (0..=1024u16)
            .step_by(7)
            .chain([1, 2, 92, 93, 100, 101, 1023, 1024])
        {
            let positions: Vec<u16> = (0..count).collect();
            let packed = pack_positions(&positions);
            for bit in (0..packed.len() * 8).step_by(13) {
                let mut flipped = packed.clone();
                flipped[bit / 8] ^= 1u8 << (bit % 8);
                cases.push((count, flipped));
            }
            cases.push((count, packed));
        }
        for _ in 0..4096 {
            seed = splitmix64(seed);
            let count = (seed % 1100) as u16;
            let expected = (count as usize * SPARSE_INDEX_BITS).div_ceil(8);
            let len = (expected + (seed >> 40) as usize % 5).saturating_sub(2);
            let bytes = (0..len)
                .map(|j| splitmix64(seed ^ j as u64) as u8)
                .collect::<Vec<u8>>();
            cases.push((count, bytes));
        }
        cases.push((1025, vec![0; 1282]));
        cases.push((u16::MAX, vec![0; 81_919]));
        for (count, packed) in cases {
            let reference = unpack_positions(count, &packed);
            assert_eq!(streamed(count, &packed), reference, "count={count}");
            let residual = sparse(count, packed);
            let expected_apply = reference.as_ref().map(|positions| {
                let mut out = predictor;
                for &position in positions {
                    out.words_mut()[position as usize >> 6] ^= 1u64 << (position & 63);
                }
                out
            });
            assert_eq!(
                residual.apply(&predictor),
                expected_apply.map_err(Clone::clone)
            );
            assert_eq!(residual.popcount(), reference.map(|_| count));
            checked += 1;
        }
        assert!(checked > 4096);
    }

    #[test]
    fn sparse_positions_must_be_strictly_increasing() {
        let predictor = sample(31);
        for positions in [[9u16, 9], [9, 3], [1023, 0]] {
            assert_eq!(
                sparse(2, pack_positions(&positions)).apply(&predictor),
                Err(GelError::InvalidResidual(
                    "sparse positions must be strictly increasing"
                )),
                "positions={positions:?}"
            );
        }
        assert!(sparse(2, pack_positions(&[3, 9])).apply(&predictor).is_ok());
    }

    /// A 10-bit field holds at most 1023. Packing 1024 spills its eleventh
    /// bit into the padding (single position) or into the next field (which
    /// then decodes as 0, below its predecessor); both spills are rejected.
    #[test]
    fn sparse_position_at_or_above_1024_is_rejected() {
        let predictor = sample(32);
        assert_eq!(
            sparse(1, pack_positions(&[1024])).apply(&predictor),
            Err(GelError::InvalidResidual(
                "sparse padding bits must be zero"
            ))
        );
        assert_eq!(
            sparse(2, pack_positions(&[1023, 1024])).apply(&predictor),
            Err(GelError::InvalidResidual(
                "sparse positions must be strictly increasing"
            ))
        );
        assert!(sparse(1, pack_positions(&[1023])).apply(&predictor).is_ok());
    }

    #[test]
    fn full_record_break_even_includes_parent_metadata() {
        let predictor = Orb1024::ZERO;
        let p94 = EncodedOrb::encode_against(
            &flipped(predictor, 94),
            &predictor,
            ParentRef::Prototype(0),
            0,
        )
        .unwrap();
        let p95 = EncodedOrb::encode_against(
            &flipped(predictor, 95),
            &predictor,
            ParentRef::Prototype(0),
            0,
        )
        .unwrap();
        let l96 = EncodedOrb::encode_against(
            &flipped(predictor, 96),
            &predictor,
            ParentRef::SegmentLocal(0),
            0,
        )
        .unwrap();
        let l97 = EncodedOrb::encode_against(
            &flipped(predictor, 97),
            &predictor,
            ParentRef::SegmentLocal(0),
            0,
        )
        .unwrap();
        assert!(matches!(&p94, EncodedOrb::Delta { .. }));
        assert!(matches!(&p95, EncodedOrb::Literal(_)));
        assert!(matches!(&l96, EncodedOrb::Delta { .. }));
        assert!(matches!(&l97, EncodedOrb::Literal(_)));
    }

    #[test]
    fn delta_depth_is_hard_bounded() {
        let p = sample(1);
        let t = flipped(p, 3);
        assert!(EncodedOrb::encode_against(&t, &p, ParentRef::Prototype(0), 0).is_ok());
        assert!(EncodedOrb::encode_against(&t, &p, ParentRef::Prototype(0), 1).is_ok());
        assert!(EncodedOrb::encode_against(&t, &p, ParentRef::Prototype(0), 2).is_err());
    }

    #[test]
    fn random_data_falls_back_to_literal_when_delta_is_not_smaller() {
        let a = sample(10);
        let b = sample(11);
        let encoded = EncodedOrb::encode_against(&a, &b, ParentRef::Prototype(0), 0).unwrap();
        assert!(matches!(&encoded, EncodedOrb::Literal(_)));
        assert_eq!(encoded.decode(None).unwrap(), a);
    }

    #[test]
    fn best_prototype_selects_correlated_parent_and_decodes_exactly() {
        let prototypes = [sample(20), sample(21), sample(22)];
        let target = flipped(prototypes[1], 12);
        let (encoded, index) = encode_best_prototype(&target, &prototypes).unwrap();
        assert_eq!(index, Some(1));
        assert_eq!(encoded.decode(Some(&prototypes[1])).unwrap(), target);
        assert!(encoded.local_der() > 1.0);
    }
}
