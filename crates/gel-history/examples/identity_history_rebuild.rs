//! Identity check: the streaming sparse decoder of `Residual::apply` and
//! `Residual::popcount`, and the rolled rebuild in `RecordHistory::from_bytes`,
//! `exact_history`, `reconstruct` and `append`, against an exact copy of the
//! collecting decoder and the per-index rebuild they replace.
//!
//! Every result is compared bit for bit, errors included. Prints one line
//! `IDENTITY name=<case> cases=<n> equal=<n>` per case and then exactly one
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL`; FAIL when any result
//! differs or when a case ran no comparison; exit status 0 on PASS, 1 on
//! FAIL. Nothing is written to disk. Output grammar: `docs/SPEED.md`.
//!
//! Usage: cargo run --release -p gel-history --example identity_history_rebuild

use gel_core::{splitmix64, GelError, ORB_BITS, ORB_WORDS};
use gel_history::{HistoryEntry, RecordHistory};
use gel_orb::Orb1024;
use gel_structural::Residual;
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
pub use identity::{verdict, Scale, Tally};

/// How much of each case runs.
pub struct Sizes {
    /// Seeded histories built, saved and reopened.
    pub histories: usize,
    /// Longest seeded history.
    pub max_len: usize,
    /// States of the history whose bytes are mutated one by one.
    pub mutated_len: usize,
    /// Random sparse encodings.
    pub random_residuals: usize,
    /// One extra long history.
    pub long_len: usize,
    /// Stride over sparse counts 0..=1024 (the boundaries always run).
    pub count_step: usize,
}

pub const FULL: Sizes = Sizes {
    histories: 600,
    max_len: 1_000,
    mutated_len: 200,
    random_residuals: 200_000,
    long_len: 3_000,
    count_step: 1,
};

pub const SMALL: Sizes = Sizes {
    histories: 24,
    max_len: 60,
    mutated_len: 24,
    random_residuals: 3_000,
    long_len: 120,
    count_step: 29,
};

/// The decoder and the history rebuild as they were before this change,
/// copied from the library and written against its public types.
mod old {
    use gel_core::{crc64_ecma, GelError, ORB_BITS, ORB_BYTES, ORB_WORDS};
    use gel_history::{HistoryEntry, MAX_RESIDUAL_DEPTH};
    use gel_orb::Orb1024;
    use gel_structural::Residual;

    const SPARSE_INDEX_BITS: usize = 10;
    const MAGIC: [u8; 8] = *b"GELHIS01";
    const VERSION: u32 = 1;
    const HEADER_BYTES: usize = 48;
    const LITERAL_TAG: u8 = 0;
    const RESIDUAL_TAG: u8 = 1;
    const SPARSE_FORM: u8 = 0;
    const DENSE_FORM: u8 = 1;
    const LITERAL_ENTRY_BYTES: usize = 1 + ORB_BYTES;
    const RESIDUAL_ENTRY_EXTRA_BYTES: usize = 10;
    const MIN_ENTRY_BYTES: u64 = 13;

    pub fn unpack_positions(count: u16, packed: &[u8]) -> Result<Vec<u16>, GelError> {
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

    pub fn popcount(residual: &Residual) -> Result<u16, GelError> {
        match residual {
            Residual::Sparse {
                count,
                packed_positions,
            } => {
                unpack_positions(*count, packed_positions)?;
                Ok(*count)
            }
            Residual::Dense(delta) => Ok(delta.words().iter().map(|x| x.count_ones() as u16).sum()),
        }
    }

    pub fn apply(residual: &Residual, predictor: &Orb1024) -> Result<Orb1024, GelError> {
        match residual {
            Residual::Dense(delta) => {
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
            Residual::Sparse {
                count,
                packed_positions,
            } => {
                let positions = unpack_positions(*count, packed_positions)?;
                let mut out = *predictor;
                for position in positions {
                    let word = position as usize >> 6;
                    let bit = position as usize & 63;
                    out.words_mut()[word] ^= 1u64 << bit;
                }
                Ok(out)
            }
        }
    }

    pub fn reconstruct(entries: &[HistoryEntry], index: usize) -> Result<Orb1024, GelError> {
        let mut chain: Vec<&Residual> = Vec::with_capacity(MAX_RESIDUAL_DEPTH as usize);
        let mut cursor = index;
        let mut state = loop {
            let entry = entries
                .get(cursor)
                .ok_or(GelError::InvalidResidual("index outside history"))?;
            match entry {
                HistoryEntry::Literal(state) => break *state,
                HistoryEntry::Residual {
                    parent, residual, ..
                } => {
                    let parent = *parent as usize;
                    if parent >= cursor {
                        return Err(GelError::InvalidResidual("parent must precede child"));
                    }
                    if chain.len() == MAX_RESIDUAL_DEPTH as usize {
                        return Err(GelError::InvalidResidual(
                            "residual chain longer than MAX_RESIDUAL_DEPTH",
                        ));
                    }
                    chain.push(residual);
                    cursor = parent;
                }
            }
        };
        for residual in chain.iter().rev() {
            state = apply(residual, &state)?;
        }
        Ok(state)
    }

    pub fn exact_history(entries: &[HistoryEntry]) -> Result<Vec<Orb1024>, GelError> {
        (0..entries.len())
            .map(|index| reconstruct(entries, index))
            .collect()
    }

    pub fn latest(entries: &[HistoryEntry]) -> Result<Option<Orb1024>, GelError> {
        entries
            .len()
            .checked_sub(1)
            .map(|index| reconstruct(entries, index))
            .transpose()
    }

    fn entry_depth(entry: &HistoryEntry) -> u8 {
        match entry {
            HistoryEntry::Literal(_) => 0,
            HistoryEntry::Residual { depth, .. } => *depth,
        }
    }

    fn residual_entry_len(residual: &Residual) -> usize {
        RESIDUAL_ENTRY_EXTRA_BYTES + residual.serialized_len()
    }

    pub fn next_entry(
        entries: &[HistoryEntry],
        upto: usize,
        state: Orb1024,
    ) -> Result<HistoryEntry, GelError> {
        let Some(parent) = upto.checked_sub(1) else {
            return Ok(HistoryEntry::Literal(state));
        };
        let parent_depth = entries
            .get(parent)
            .map(entry_depth)
            .ok_or(GelError::InvalidResidual("index outside history"))?;
        let residual = Residual::from_exact_xor(&state, &reconstruct(entries, parent)?);
        let depth = parent_depth + 1;
        if depth > MAX_RESIDUAL_DEPTH || residual_entry_len(&residual) >= LITERAL_ENTRY_BYTES {
            return Ok(HistoryEntry::Literal(state));
        }
        Ok(HistoryEntry::Residual {
            parent: u32::try_from(parent)
                .map_err(|_| GelError::LimitExceeded("history parent index exceeds u32"))?,
            depth,
            residual,
        })
    }

    pub fn append(entries: &mut Vec<HistoryEntry>, state: Orb1024) -> Result<usize, GelError> {
        let entry = next_entry(entries, entries.len(), state)?;
        entries.push(entry);
        Ok(entries.len() - 1)
    }

    pub fn from_bytes(
        bytes: &[u8],
        max_records: u64,
        max_payload_bytes: u64,
    ) -> Result<Vec<HistoryEntry>, GelError> {
        let header: &[u8; HEADER_BYTES] = bytes
            .get(..HEADER_BYTES)
            .and_then(|header| header.try_into().ok())
            .ok_or(GelError::InvalidHeader("history file shorter than header"))?;
        let (record_count, payload_len, payload_crc) =
            check_header(header, max_records, max_payload_bytes)?;
        let payload = &bytes[HEADER_BYTES..];
        if payload.len() as u64 != payload_len {
            return Err(GelError::InvalidLength {
                expected: usize::try_from(payload_len)
                    .map_or(usize::MAX, |len| len.saturating_add(HEADER_BYTES)),
                actual: bytes.len(),
            });
        }
        if crc64_ecma(payload) != payload_crc {
            return Err(GelError::CorruptStore);
        }
        let entries = decode_entries(payload, record_count)?;
        for index in 0..entries.len() {
            let state = reconstruct(&entries, index)?;
            if entries[index] != next_entry(&entries, index, state)? {
                return Err(GelError::InvalidResidual(
                    "history file is not in canonical form",
                ));
            }
        }
        Ok(entries)
    }

    fn check_header(
        header: &[u8; HEADER_BYTES],
        max_records: u64,
        max_payload_bytes: u64,
    ) -> Result<(u64, u64, u64), GelError> {
        let (record_count, payload_len, payload_crc) = decode_header(header)?;
        if record_count > max_records {
            return Err(GelError::LimitExceeded("history record count"));
        }
        if payload_len > max_payload_bytes {
            return Err(GelError::LimitExceeded("history payload bytes"));
        }
        if record_count > payload_len / MIN_ENTRY_BYTES {
            return Err(GelError::InvalidHeader("history count/payload mismatch"));
        }
        Ok((record_count, payload_len, payload_crc))
    }

    fn decode_entries(payload: &[u8], record_count: u64) -> Result<Vec<HistoryEntry>, GelError> {
        let count = usize::try_from(record_count)
            .map_err(|_| GelError::LimitExceeded("history record address space"))?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| GelError::AllocationFailed)?;
        let mut cursor = 0usize;
        while entries.len() < count {
            let entry = match take_array::<1>(payload, &mut cursor)?[0] {
                LITERAL_TAG => HistoryEntry::Literal(Orb1024::from_le_bytes(take(
                    payload,
                    &mut cursor,
                    ORB_BYTES,
                )?)?),
                RESIDUAL_TAG => decode_residual(payload, &mut cursor, &entries)?,
                _ => return Err(GelError::InvalidResidual("unknown history entry tag")),
            };
            entries.push(entry);
        }
        if cursor != payload.len() {
            return Err(GelError::InvalidLength {
                expected: cursor,
                actual: payload.len(),
            });
        }
        Ok(entries)
    }

    fn decode_residual(
        payload: &[u8],
        cursor: &mut usize,
        earlier: &[HistoryEntry],
    ) -> Result<HistoryEntry, GelError> {
        let parent = u32::from_le_bytes(take_array(payload, cursor)?);
        let [depth] = take_array::<1>(payload, cursor)?;
        let parent_depth = earlier
            .get(parent as usize)
            .map(entry_depth)
            .ok_or(GelError::InvalidResidual("parent must precede child"))?;
        if depth != parent_depth + 1 {
            return Err(GelError::InvalidResidual(
                "residual depth does not continue its parent chain",
            ));
        }
        if depth > MAX_RESIDUAL_DEPTH {
            return Err(GelError::InvalidResidual(
                "residual chain longer than MAX_RESIDUAL_DEPTH",
            ));
        }
        let residual = match take_array::<1>(payload, cursor)?[0] {
            SPARSE_FORM => {
                let count = u16::from_le_bytes(take_array(payload, cursor)?);
                let len = u32::from_le_bytes(take_array(payload, cursor)?) as usize;
                Residual::Sparse {
                    count,
                    packed_positions: take(payload, cursor, len)?.to_vec(),
                }
            }
            DENSE_FORM => {
                return Err(GelError::InvalidResidual(
                    "dense residual is not written by this format",
                ))
            }
            _ => return Err(GelError::InvalidResidual("unknown residual form")),
        };
        Ok(HistoryEntry::Residual {
            parent,
            depth,
            residual,
        })
    }

    fn decode_header(header: &[u8; HEADER_BYTES]) -> Result<(u64, u64, u64), GelError> {
        if header[..8] != MAGIC {
            return Err(GelError::InvalidMagic);
        }
        if crc64_ecma(&header[..40]) != le_u64(header, 40) {
            return Err(GelError::CorruptHeader);
        }
        let version = le_u32(header, 8);
        if version != VERSION {
            return Err(GelError::UnsupportedVersion(version));
        }
        if le_u32(header, 12) != 0 {
            return Err(GelError::InvalidHeader(
                "history reserved field must be zero",
            ));
        }
        Ok((le_u64(header, 16), le_u64(header, 24), le_u64(header, 32)))
    }

    fn le_u32(header: &[u8; HEADER_BYTES], at: usize) -> u32 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&header[at..at + 4]);
        u32::from_le_bytes(word)
    }

    fn le_u64(header: &[u8; HEADER_BYTES], at: usize) -> u64 {
        let mut word = [0u8; 8];
        word.copy_from_slice(&header[at..at + 8]);
        u64::from_le_bytes(word)
    }

    fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], GelError> {
        let out = cursor
            .checked_add(len)
            .and_then(|end| bytes.get(*cursor..end))
            .ok_or(GelError::InvalidResidual("truncated history payload"))?;
        *cursor += len;
        Ok(out)
    }

    fn take_array<const N: usize>(bytes: &[u8], cursor: &mut usize) -> Result<[u8; N], GelError> {
        let mut out = [0u8; N];
        out.copy_from_slice(take(bytes, cursor, N)?);
        Ok(out)
    }
}

const NO_LIMIT: u64 = u64::MAX;

fn sample(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix64(self.0)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// `k` distinct positions: an odd step is coprime with 1024.
fn distinct_positions(k: usize, rng: &mut Rng) -> Vec<u16> {
    let start = rng.below(ORB_BITS as u64) as usize;
    let step = (rng.below(512) * 2 + 1) as usize;
    let mut out: Vec<u16> = (0..k)
        .map(|j| ((start + step * j) % ORB_BITS) as u16)
        .collect();
    out.sort_unstable();
    out
}

fn flip_positions(mut state: Orb1024, positions: &[u16]) -> Orb1024 {
    for &p in positions {
        state.words_mut()[p as usize >> 6] ^= 1u64 << (p & 63);
    }
    state
}

/// The library's packing, copied: 10 bits per position, low bits first.
fn pack(positions: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity((positions.len() * 10).div_ceil(8));
    let mut acc = 0u64;
    let mut bits = 0usize;
    for &position in positions {
        acc |= (position as u64) << bits;
        bits += 10;
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

/// A next state: unrelated, unchanged, at the 92/93 and 100/101 bit
/// boundaries, or a few to about a hundred changed bits.
fn next_state(state: Orb1024, rng: &mut Rng) -> Orb1024 {
    let k = match rng.below(12) {
        0 => return sample(rng.next()),
        1 => 0,
        2 => 92,
        3 => 93,
        4 => 1,
        5 | 6 => 3,
        7 => 40,
        8 => rng.below(120) as usize,
        9 => 100 + rng.below(2) as usize,
        10 => 512,
        _ => 2,
    };
    flip_positions(state, &distinct_positions(k, rng))
}

fn states(len: usize, rng: &mut Rng) -> Vec<Orb1024> {
    let mut out = Vec::with_capacity(len);
    let mut state = sample(rng.next());
    for _ in 0..len {
        out.push(state);
        state = next_state(state, rng);
    }
    out
}

fn new_open(bytes: &[u8], max_records: u64, max_payload: u64) -> Result<RecordHistory, GelError> {
    RecordHistory::from_bytes(bytes, max_records, max_payload)
}

/// Compares the new and the old reader on one file; when both accept it, the
/// rebuilt states and the latest state are compared too.
fn same_open(bytes: &[u8], max_records: u64, max_payload: u64) -> bool {
    let new = new_open(bytes, max_records, max_payload);
    let old = old::from_bytes(bytes, max_records, max_payload);
    match (&new, &old) {
        (Ok(history), Ok(entries)) => {
            history.entries() == entries.as_slice()
                && history.exact_history() == old::exact_history(entries)
                && history.latest() == old::latest(entries)
        }
        (Err(a), Err(b)) => a == b,
        _ => false,
    }
}

/// Recomputes the header CRC64 (bytes 40..48) over bytes 0..40.
fn reseal_header(bytes: &mut [u8]) {
    let crc = gel_core::crc64_ecma(&bytes[..40]);
    bytes[40..48].copy_from_slice(&crc.to_le_bytes());
}

/// Recomputes the payload CRC64 (bytes 32..40) and then the header CRC64.
fn reseal_payload(bytes: &mut [u8]) {
    let crc = gel_core::crc64_ecma(&bytes[48..]);
    bytes[32..40].copy_from_slice(&crc.to_le_bytes());
    reseal_header(bytes);
}

/// Byte offset in the file and kind of every entry, read from the format.
fn entry_offsets(bytes: &[u8]) -> Vec<(usize, bool)> {
    let mut out = Vec::new();
    let mut at = 48;
    while at < bytes.len() {
        match bytes[at] {
            0 => {
                out.push((at, false));
                at += 129;
            }
            _ => {
                out.push((at, true));
                let len = u32::from_le_bytes(bytes[at + 9..at + 13].try_into().unwrap());
                at += 13 + len as usize;
            }
        }
    }
    out
}

fn sparse_cases(scale: &Sizes, valid: &mut Tally, invalid: &mut Tally, dense: &mut Tally) {
    let mut rng = Rng(0x5350_4152_5345_0001);
    let predictors = [
        Orb1024::ZERO,
        sample(1),
        sample(2),
        Orb1024::from_words([!0u64; ORB_WORDS]),
    ];
    let compare = |residual: &Residual, predictor: &Orb1024| {
        residual.apply(predictor) == old::apply(residual, predictor)
            && residual.popcount() == old::popcount(residual)
    };
    let boundaries = [0usize, 1, 2, 3, 92, 93, 100, 101, 1023, 1024];
    for count in (0..=ORB_BITS).step_by(scale.count_step).chain(boundaries) {
        let prefix: Vec<u16> = (0..count as u16).collect();
        let spread: Vec<u16> = (0..count)
            .map(|j| (j * ORB_BITS / count.max(1)) as u16)
            .collect();
        let tail: Vec<u16> = ((ORB_BITS - count) as u16..ORB_BITS as u16).collect();
        let random = distinct_positions(count, &mut rng);
        for positions in [prefix, spread, tail, random] {
            let packed = pack(&positions);
            let residual = Residual::Sparse {
                count: count as u16,
                packed_positions: packed.clone(),
            };
            for predictor in &predictors {
                valid.check(compare(&residual, predictor));
            }
            // Bit flips of the valid packing: order, padding and spill errors.
            let flips = packed.len() * 8;
            let stride = if flips <= 96 { 1 } else { flips / 48 };
            for bit in (0..flips).step_by(stride.max(1)) {
                let mut changed = packed.clone();
                changed[bit / 8] ^= 1u8 << (bit % 8);
                let residual = Residual::Sparse {
                    count: count as u16,
                    packed_positions: changed,
                };
                invalid.check(compare(&residual, &predictors[1]));
            }
            // Count one off and packing one byte off in both directions.
            for (c, extra) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let c = count as i32 + c;
                if !(0..=u16::MAX as i32).contains(&c) {
                    continue;
                }
                let mut p = packed.clone();
                if extra > 0 {
                    p.push(rng.next() as u8);
                } else if extra < 0 && p.pop().is_none() {
                    continue;
                }
                let residual = Residual::Sparse {
                    count: c as u16,
                    packed_positions: p,
                };
                invalid.check(compare(&residual, &predictors[2]));
            }
        }
    }
    // Duplicates and descending pairs.
    for _ in 0..512 {
        let a = rng.below(ORB_BITS as u64) as u16;
        let b = rng.below(ORB_BITS as u64) as u16;
        let mut positions = distinct_positions(rng.below(20) as usize, &mut rng);
        let at = rng.below(positions.len() as u64 + 1) as usize;
        positions.insert(at, a.max(b));
        positions.insert(at + 1, a.min(b));
        let residual = Residual::Sparse {
            count: positions.len() as u16,
            packed_positions: pack(&positions),
        };
        invalid.check(compare(&residual, &predictors[1]));
    }
    // Random bytes at and near the expected length, counts up to and past 1024.
    for _ in 0..scale.random_residuals {
        let count = match rng.below(8) {
            0 => 1024 + rng.below(4) as u16,
            1 => u16::MAX - rng.below(2) as u16,
            _ => rng.below(1100) as u16,
        };
        let expected = (count as usize * 10).div_ceil(8);
        let len = (expected + rng.below(5) as usize).saturating_sub(2);
        let packed: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        let residual = Residual::Sparse {
            count,
            packed_positions: packed,
        };
        invalid.check(compare(&residual, &predictors[3]));
    }
    for _ in 0..256 {
        let residual = Residual::Dense(sample(rng.next()));
        dense.check(compare(&residual, &sample(rng.next())));
    }
}

fn history_cases(
    scale: &Sizes,
    append: &mut Tally,
    reopen: &mut Tally,
    rebuild: &mut Tally,
    limits: &mut Tally,
) {
    let mut rng = Rng(0x4849_5354_4f52_5900);
    let mut lengths: Vec<usize> = (0..=6).collect();
    lengths.push(scale.long_len);
    while lengths.len() < scale.histories {
        lengths.push(rng.below(scale.max_len as u64 + 1) as usize);
    }
    for len in lengths {
        let states = states(len, &mut rng);
        let mut history = RecordHistory::new();
        let mut entries = Vec::new();
        let mut same = true;
        for state in &states {
            same &= history.append(*state) == old::append(&mut entries, *state);
        }
        same &= history.entries() == entries.as_slice();
        append.check(same);

        let bytes = history.to_bytes().unwrap();
        reopen.check(same_open(&bytes, NO_LIMIT, NO_LIMIT));

        let mut same = history.exact_history() == old::exact_history(&entries)
            && history.latest() == old::latest(&entries);
        for index in 0..len + 2 {
            same &= history.reconstruct(index) == old::reconstruct(&entries, index);
        }
        rebuild.check(same);

        let n = len as u64;
        let p = (bytes.len() - 48) as u64;
        for (records, payload) in [(n, p), (n.wrapping_sub(1), p), (n, p.wrapping_sub(1))] {
            limits.check(same_open(&bytes, records, payload));
        }
    }
}

fn mutation_cases(scale: &Sizes, resealed: &mut Tally, rewired: &mut Tally, raw: &mut Tally) {
    let mut rng = Rng(0x4d55_5441_5445_0001);
    let states = states(scale.mutated_len, &mut rng);
    let mut history = RecordHistory::new();
    for state in &states {
        history.append(*state).unwrap();
    }
    let bytes = history.to_bytes().unwrap();

    // Every byte with several masks; the CRCs are recomputed so the file
    // reaches the entry decoder and the rebuild.
    for at in 0..bytes.len() {
        if (40..48).contains(&at) {
            continue;
        }
        let masks = [0x01u8, 0x80, 0xFF, (rng.below(255) + 1) as u8];
        for mask in masks {
            let mut changed = bytes.clone();
            changed[at] ^= mask;
            if at < 40 {
                reseal_header(&mut changed);
            } else {
                reseal_payload(&mut changed);
            }
            resealed.check(same_open(&changed, NO_LIMIT, NO_LIMIT));
        }
    }

    // Every residual entry pointed at other parents with chain-consistent and
    // other depths: files the decoder accepts but the encoder never writes.
    let offsets = entry_offsets(&bytes);
    let depths: Vec<u8> = history
        .entries()
        .iter()
        .map(|e| match e {
            HistoryEntry::Literal(_) => 0,
            HistoryEntry::Residual { depth, .. } => *depth,
        })
        .collect();
    for (index, &(at, is_residual)) in offsets.iter().enumerate() {
        if !is_residual {
            continue;
        }
        let mut parents = vec![0u32, index as u32, u32::MAX];
        for back in [1usize, 2, 3] {
            if let Some(p) = index.checked_sub(back) {
                parents.push(p as u32);
            }
        }
        parents.push((index / 2) as u32);
        parents.push(rng.below(index.max(1) as u64) as u32);
        for parent in parents {
            let consistent = depths.get(parent as usize).map_or(1, |d| d.wrapping_add(1));
            for depth in [consistent, 1, 2] {
                let mut changed = bytes.clone();
                changed[at + 1..at + 5].copy_from_slice(&parent.to_le_bytes());
                changed[at + 5] = depth;
                reseal_payload(&mut changed);
                rewired.check(same_open(&changed, NO_LIMIT, NO_LIMIT));
            }
        }
    }

    // Raw bit flips and truncations of a short file, no CRC recomputed.
    let short = {
        let mut h = RecordHistory::new();
        for state in &states[..states.len().min(30)] {
            h.append(*state).unwrap();
        }
        h.to_bytes().unwrap()
    };
    for bit in 0..short.len() * 8 {
        let mut changed = short.clone();
        changed[bit / 8] ^= 1u8 << (bit % 8);
        raw.check(same_open(&changed, NO_LIMIT, NO_LIMIT));
    }
    for len in 0..=short.len() {
        raw.check(same_open(&short[..len], NO_LIMIT, NO_LIMIT));
    }
    let mut longer = short.clone();
    longer.push(0);
    raw.check(same_open(&longer, NO_LIMIT, NO_LIMIT));
}

/// Runs every case at `scale` (`FULL` or `SMALL` sizes) and returns one tally
/// per case.
pub fn run(scale: Scale) -> Vec<Tally> {
    run_sizes(match scale {
        Scale::Full => &FULL,
        Scale::Small => &SMALL,
    })
}

/// Runs every case at the given sizes and returns one tally per case.
pub fn run_sizes(scale: &Sizes) -> Vec<Tally> {
    let mut sparse_valid = Tally::new("sparse_apply_popcount_valid");
    let mut sparse_invalid = Tally::new("sparse_apply_popcount_invalid");
    let mut dense = Tally::new("dense_apply_popcount");
    sparse_cases(scale, &mut sparse_valid, &mut sparse_invalid, &mut dense);
    let mut append = Tally::new("history_append_entries");
    let mut reopen = Tally::new("history_from_bytes_valid");
    let mut rebuild = Tally::new("history_exact_latest_reconstruct");
    let mut limits = Tally::new("history_from_bytes_limits");
    history_cases(scale, &mut append, &mut reopen, &mut rebuild, &mut limits);
    let mut resealed = Tally::new("history_from_bytes_byte_mutations_resealed");
    let mut rewired = Tally::new("history_from_bytes_parent_rewired");
    let mut raw = Tally::new("history_from_bytes_raw_flips_truncations");
    mutation_cases(scale, &mut resealed, &mut rewired, &mut raw);
    vec![
        sparse_valid,
        sparse_invalid,
        dense,
        append,
        reopen,
        rebuild,
        limits,
        resealed,
        rewired,
        raw,
    ]
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
