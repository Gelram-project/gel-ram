//! Value tests of the record history: byte counts, hand-written files and the
//! reason every refused file is refused.

use gel_core::{crc64_ecma, splitmix64, GelError, ORB_BITS, ORB_WORDS};
use gel_history::{HistoryEntry, RecordHistory, MAX_RESIDUAL_DEPTH};
use gel_orb::Orb1024;
use gel_structural::Residual;
use std::fs;
use std::path::PathBuf;

const NO_LIMIT: u64 = u64::MAX;

fn sample(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

fn flip(mut state: Orb1024, bits: &[usize]) -> Orb1024 {
    for &bit in bits {
        state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
    }
    state
}

/// `k` distinct bits from `start` in steps of 37 (coprime with 1024).
fn flip_k(state: Orb1024, k: usize, start: usize) -> Orb1024 {
    let bits: Vec<usize> = (0..k).map(|j| (start + 37 * j) % ORB_BITS).collect();
    flip(state, &bits)
}

/// `n` states, each differing from the one before in exactly `k` bits.
fn walk(n: usize, k: usize, seed: u64) -> Vec<Orb1024> {
    let mut states = vec![sample(seed)];
    for i in 1..n {
        states.push(flip_k(states[i - 1], k, i * 101));
    }
    states
}

fn history_of(states: &[Orb1024]) -> RecordHistory {
    let mut history = RecordHistory::new();
    for (i, state) in states.iter().enumerate() {
        assert_eq!(history.append(*state).unwrap(), i);
    }
    history
}

fn open(bytes: &[u8]) -> Result<RecordHistory, GelError> {
    RecordHistory::from_bytes(bytes, NO_LIMIT, NO_LIMIT)
}

/// A GELHIS01 file written by hand from the format description: magic,
/// version, reserved field, entry count, payload length, payload CRC64 and the
/// CRC64 of those 40 bytes, then the payload.
fn file_with(version: u32, reserved: u32, count: u64, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"GELHIS01");
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&reserved.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(&crc64_ecma(payload).to_le_bytes());
    let header_crc = crc64_ecma(&out);
    out.extend_from_slice(&header_crc.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

fn file(count: u64, payload: &[u8]) -> Vec<u8> {
    file_with(1, 0, count, payload)
}

fn literal(state: Orb1024) -> Vec<u8> {
    let mut out = vec![0u8];
    out.extend_from_slice(&state.to_le_bytes());
    out
}

fn residual_bytes(
    parent: u32,
    depth: u8,
    form: u8,
    count: u16,
    len: u32,
    packed: &[u8],
) -> Vec<u8> {
    let mut out = vec![1u8];
    out.extend_from_slice(&parent.to_le_bytes());
    out.push(depth);
    out.push(form);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(packed);
    out
}

/// A residual entry built with the public sparse residual of gel-structural.
fn sparse(parent: u32, depth: u8, from: Orb1024, to: Orb1024) -> Vec<u8> {
    match Residual::from_exact_xor(&to, &from) {
        Residual::Sparse {
            count,
            packed_positions,
        } => residual_bytes(
            parent,
            depth,
            0,
            count,
            packed_positions.len() as u32,
            &packed_positions,
        ),
        Residual::Dense(_) => panic!("more than 100 bits differ"),
    }
}

fn dense(parent: u32, depth: u8, from: Orb1024, to: Orb1024) -> Vec<u8> {
    let mut out = vec![1u8];
    out.extend_from_slice(&parent.to_le_bytes());
    out.push(depth);
    out.push(1);
    let mut xor = to;
    for (word, from_word) in xor.words_mut().iter_mut().zip(from.words()) {
        *word ^= from_word;
    }
    out.extend_from_slice(&xor.to_le_bytes());
    out
}

fn refused(reason: &'static str) -> Result<RecordHistory, GelError> {
    Err(GelError::InvalidResidual(reason))
}

#[test]
fn entries_follow_literal_residual_residual_and_the_length_is_exact() {
    let states = walk(7, 1, 10);
    let history = history_of(&states);
    let expected = [(0, 0), (0, 1), (1, 2), (3, 0), (3, 1), (4, 2), (6, 0)];
    for (i, (entry, &(parent, depth))) in history.entries().iter().zip(&expected).enumerate() {
        match entry {
            HistoryEntry::Literal(state) => {
                assert_eq!(depth, 0, "entry {i}");
                assert_eq!(*state, states[i]);
            }
            HistoryEntry::Residual {
                parent: p,
                depth: d,
                residual,
            } => {
                assert_eq!((*p, *d), (parent, depth), "entry {i}");
                assert_eq!(residual.popcount(), Ok(1));
                assert_eq!(residual.serialized_len(), 1 + 2 + 2);
            }
        }
    }
    assert_eq!(history.encoded_len(), 48 + 3 * 129 + 4 * 15);
    assert_eq!(history.encoded_len(), 495);
    assert_eq!(history.to_bytes().unwrap().len(), 495);
}

#[test]
fn ninety_two_changed_bits_are_a_residual_and_ninety_three_a_literal() {
    let base = sample(11);
    // (changed bits, stored as a residual, file bytes for two states)
    for (k, as_residual, bytes) in [
        (0, true, 48 + 129 + 13),
        (1, true, 48 + 129 + 15),
        (92, true, 48 + 129 + 128),
        (93, false, 48 + 129 + 129),
        (100, false, 48 + 129 + 129),
        (101, false, 48 + 129 + 129),
    ] {
        let next = flip_k(base, k, 3);
        let history = history_of(&[base, next]);
        match &history.entries()[1] {
            HistoryEntry::Residual { residual, .. } => {
                assert!(as_residual, "k={k}");
                assert_eq!(residual.popcount(), Ok(k as u16));
            }
            HistoryEntry::Literal(state) => {
                assert!(!as_residual, "k={k}");
                assert_eq!(*state, next);
            }
        }
        assert_eq!(history.encoded_len(), bytes, "k={k}");
        let saved = history.to_bytes().unwrap();
        assert_eq!(saved.len(), bytes, "k={k}");
        assert_eq!(open(&saved).unwrap().exact_history().unwrap(), [base, next]);
    }
}

#[test]
fn three_states_give_the_hand_built_file() {
    let base = sample(12);
    let second = flip(base, &[5]);
    let third = flip(second, &[0, 1023]);
    let history = history_of(&[base, second, third]);

    let mut payload = literal(base);
    // tag 1, parent 0, depth 1, sparse form, count 1, length 2, position 5 in 10 bits
    payload.extend_from_slice(&[1, 0, 0, 0, 0, 1, 0, 1, 0, 2, 0, 0, 0, 0x05, 0x00]);
    // tag 1, parent 1, depth 2, sparse form, count 2, length 3, positions 0 and 1023
    payload.extend_from_slice(&[1, 1, 0, 0, 0, 2, 0, 2, 0, 3, 0, 0, 0, 0x00, 0xFC, 0x0F]);
    assert_eq!(payload.len(), 160);
    let reference = file(3, &payload);
    assert_eq!(reference.len(), 208);

    assert_eq!(history.to_bytes().unwrap(), reference);
    assert_eq!(history.encoded_len(), 208);
    let reopened = open(&reference).unwrap();
    assert_eq!(reopened, history);
    assert_eq!(reopened.exact_history().unwrap(), [base, second, third]);
}

#[test]
fn an_empty_history_is_a_48_byte_file_without_a_latest_state() {
    let history = RecordHistory::new();
    assert_eq!(history.len(), 0);
    assert!(history.is_empty());
    assert!(history.entries().is_empty());
    assert_eq!(history.latest(), Ok(None));
    assert_eq!(history.exact_history(), Ok(Vec::new()));
    assert_eq!(history.encoded_len(), 48);
    let bytes = history.to_bytes().unwrap();
    assert_eq!(bytes, file(0, &[]));
    assert_eq!(open(&bytes), Ok(RecordHistory::new()));
}

#[test]
fn append_returns_each_index_and_latest_is_the_last_state() {
    let states = walk(10, 4, 13);
    let mut history = RecordHistory::new();
    for (i, state) in states.iter().enumerate() {
        assert_eq!(history.append(*state), Ok(i));
        assert_eq!(history.len(), i + 1);
        assert!(!history.is_empty());
        assert_eq!(history.latest(), Ok(Some(*state)));
    }
    for (i, state) in states.iter().enumerate() {
        assert_eq!(history.reconstruct(i), Ok(*state));
    }
    assert_eq!(
        history.reconstruct(10),
        Err(GelError::InvalidResidual("index outside history"))
    );
}

#[test]
fn long_history_is_exact_and_residual_depth_never_exceeds_two() {
    let mut history = RecordHistory::new();
    let mut state = sample(1);
    let mut expected = Vec::new();
    for i in 0..1000 {
        let start = i % 900;
        let bits: Vec<usize> = (start..start + (i % 7) + 1).collect();
        state = flip(state, &bits);
        expected.push(state);
        history.append(state).unwrap();
    }
    assert_eq!(history.exact_history().unwrap(), expected);
    let depths: Vec<u8> = history
        .entries()
        .iter()
        .map(|entry| match entry {
            HistoryEntry::Literal(_) => 0,
            HistoryEntry::Residual { depth, .. } => *depth,
        })
        .collect();
    // At most seven bits change per step, so every state that may be a residual is one.
    for (i, depth) in depths.iter().enumerate() {
        assert_eq!(*depth, (i % 3) as u8, "entry {i}");
        assert!(*depth <= MAX_RESIDUAL_DEPTH);
    }
    assert_eq!(depths.iter().filter(|&&d| d == 0).count(), 334);
    let reopened = open(&history.to_bytes().unwrap()).unwrap();
    assert_eq!(reopened.exact_history().unwrap(), expected);
}

#[test]
fn an_unrelated_state_is_stored_literally() {
    let first = sample(1);
    let second = sample(2);
    assert!(
        Residual::from_exact_xor(&second, &first)
            .popcount()
            .unwrap()
            > 92
    );
    let history = history_of(&[first, second]);
    assert_eq!(history.entries()[1], HistoryEntry::Literal(second));
    assert_eq!(history.encoded_len(), 48 + 2 * 129);
}

#[test]
fn every_bit_flip_and_truncation_of_a_30_state_file_is_refused() {
    let states = walk(30, 3, 14);
    let bytes = history_of(&states).to_bytes().unwrap();
    assert_eq!(bytes.len(), 1678);
    assert_eq!(open(&bytes).unwrap().exact_history().unwrap(), states);

    let mut refused_bits = 0;
    for bit in 0..bytes.len() * 8 {
        let mut changed = bytes.clone();
        changed[bit / 8] ^= 1u8 << (bit % 8);
        let expected = match bit / 8 {
            0..=7 => GelError::InvalidMagic,
            8..=47 => GelError::CorruptHeader,
            _ => GelError::CorruptStore,
        };
        assert_eq!(open(&changed), Err(expected), "bit {bit}");
        refused_bits += 1;
    }
    assert_eq!(refused_bits, 13_424);

    let mut refused_lengths = 0;
    for len in 0..bytes.len() {
        let expected = if len < 48 {
            GelError::InvalidHeader("history file shorter than header")
        } else {
            GelError::InvalidLength {
                expected: 1678,
                actual: len,
            }
        };
        assert_eq!(open(&bytes[..len]), Err(expected), "length {len}");
        refused_lengths += 1;
    }
    assert_eq!(refused_lengths, 1678);

    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(
        open(&longer),
        Err(GelError::InvalidLength {
            expected: 1678,
            actual: 1679
        })
    );
}

#[test]
fn a_depth_byte_that_lies_is_refused() {
    let s0 = sample(77);
    let s1 = flip(s0, &[3, 4]);
    let s2 = flip(s1, &[13, 14]);
    let s3 = flip(s2, &[23, 24]);

    // Positive control: an honest chain of depth two opens and is rebuilt exactly.
    let honest = [literal(s0), sparse(0, 1, s0, s1), sparse(1, 2, s1, s2)].concat();
    assert_eq!(
        open(&file(3, &honest)).unwrap().exact_history().unwrap(),
        [s0, s1, s2]
    );

    // Fails when the decoder trusts the depth byte instead of counting the chain:
    // three residuals in a row, each declaring depth 1.
    let lying = [
        literal(s0),
        sparse(0, 1, s0, s1),
        sparse(1, 1, s1, s2),
        sparse(2, 1, s2, s3),
    ]
    .concat();
    assert_eq!(
        open(&file(4, &lying)),
        refused("residual depth does not continue its parent chain")
    );

    // A residual declaring depth 0 does not continue its literal either.
    let zero = [literal(s0), sparse(0, 0, s0, s1)].concat();
    assert_eq!(
        open(&file(2, &zero)),
        refused("residual depth does not continue its parent chain")
    );

    // An honest chain of depth three is refused by the depth limit.
    let deep = [
        literal(s0),
        sparse(0, 1, s0, s1),
        sparse(1, 2, s1, s2),
        sparse(2, 3, s2, s3),
    ]
    .concat();
    assert_eq!(
        open(&file(4, &deep)),
        refused("residual chain longer than MAX_RESIDUAL_DEPTH")
    );
}

#[test]
fn files_the_encoder_does_not_write_are_refused_as_not_canonical() {
    let s0 = sample(15);
    let s1 = flip(s0, &[3]);
    let s2 = flip(s1, &[9]);
    let not_canonical = refused("history file is not in canonical form");

    // A literal where a residual fits.
    let two_literals = [literal(s0), literal(s1)].concat();
    assert_eq!(open(&file(2, &two_literals)), not_canonical);

    // A residual whose parent is not the entry before it.
    let s2_from_s0 = flip(s0, &[9]);
    let skipped_parent = [
        literal(s0),
        sparse(0, 1, s0, s1),
        sparse(0, 1, s0, s2_from_s0),
    ]
    .concat();
    assert_eq!(open(&file(3, &skipped_parent)), not_canonical);

    // A sparse residual larger than a literal (93 changed bits, 130 bytes).
    let far = flip_k(s0, 93, 1);
    let large = [literal(s0), sparse(0, 1, s0, far)].concat();
    assert_eq!(large.len(), 129 + 130);
    assert_eq!(open(&file(2, &large)), not_canonical);

    // Three states with dense residuals to the first: 447 bytes with valid CRCs.
    let dense_file = file(
        3,
        &[literal(s0), dense(0, 1, s0, s1), dense(0, 1, s0, s2)].concat(),
    );
    assert_eq!(dense_file.len(), 447);
    assert_eq!(
        open(&dense_file),
        refused("dense residual is not written by this format")
    );

    // The canonical file of the same states opens.
    let canonical = history_of(&[s0, s1, s2]).to_bytes().unwrap();
    assert_eq!(canonical.len(), 48 + 129 + 15 + 15);
    assert_eq!(
        open(&canonical).unwrap().exact_history().unwrap(),
        [s0, s1, s2]
    );
}

#[test]
fn structurally_broken_entries_are_refused_with_their_reason() {
    let s0 = sample(16);
    let s1 = flip(s0, &[5]);
    let one_bit = sparse(0, 1, s0, s1);
    let parent_first = refused("parent must precede child");

    // The first entry is a residual.
    assert_eq!(open(&file(1, &one_bit)), parent_first);
    // A residual that is its own parent, and one whose parent comes later.
    let own = [literal(s0), sparse(1, 1, s0, s1)].concat();
    assert_eq!(open(&file(2, &own)), parent_first);
    let later = [literal(s0), sparse(2, 1, s0, s1), literal(s1)].concat();
    assert_eq!(open(&file(3, &later)), parent_first);

    // Unknown entry tag and unknown residual form.
    let mut tag = literal(s0);
    tag[0] = 2;
    assert_eq!(open(&file(1, &tag)), refused("unknown history entry tag"));
    let form = [literal(s0), residual_bytes(0, 1, 2, 1, 2, &[5, 0])].concat();
    assert_eq!(open(&file(2, &form)), refused("unknown residual form"));
    let dense_form = [literal(s0), dense(0, 1, s0, s1)].concat();
    assert_eq!(
        open(&file(2, &dense_form)),
        refused("dense residual is not written by this format")
    );

    // A length field that does not match the count, and one past the payload.
    let long_len = [literal(s0), residual_bytes(0, 1, 0, 1, 3, &[5, 0, 0])].concat();
    assert_eq!(
        open(&file(2, &long_len)),
        Err(GelError::InvalidLength {
            expected: 2,
            actual: 3
        })
    );
    let past_end = [literal(s0), residual_bytes(0, 1, 0, 1, 1000, &[5, 0])].concat();
    assert_eq!(
        open(&file(2, &past_end)),
        refused("truncated history payload")
    );

    // Nonzero padding bits, positions out of order and a count above 1024.
    let padding = [literal(s0), residual_bytes(0, 1, 0, 1, 2, &[5, 4])].concat();
    assert_eq!(
        open(&file(2, &padding)),
        refused("sparse padding bits must be zero")
    );
    // Positions 7 then 5 in 10-bit fields: 7 | 5 << 10 = 0x01407.
    let unsorted = [
        literal(s0),
        residual_bytes(0, 1, 0, 2, 3, &[0x07, 0x14, 0x00]),
    ]
    .concat();
    assert_eq!(
        open(&file(2, &unsorted)),
        refused("sparse positions must be strictly increasing")
    );
    let too_many = [literal(s0), residual_bytes(0, 1, 0, 2000, 0, &[])].concat();
    assert_eq!(
        open(&file(2, &too_many)),
        refused("sparse count exceeds 1024")
    );
}

#[test]
fn the_header_count_and_fields_must_match_the_payload() {
    let s0 = sample(17);
    let s1 = flip(s0, &[5]);
    let mismatch = Err(GelError::InvalidHeader("history count/payload mismatch"));

    // At least 13 bytes per entry: 3 entries cannot fit in 38 bytes, 2 may fit in 26.
    assert_eq!(open(&file(3, &[0; 38])), mismatch);
    assert_eq!(
        open(&file(2, &[0; 26])),
        refused("truncated history payload")
    );
    assert_eq!(open(&file(1, &[])), mismatch);

    // A count above the entries present, and one below.
    assert_eq!(
        open(&file(2, &literal(s0))),
        refused("truncated history payload")
    );
    let two = [literal(s0), sparse(0, 1, s0, s1)].concat();
    assert_eq!(
        open(&file(1, &two)),
        Err(GelError::InvalidLength {
            expected: 129,
            actual: 144
        })
    );
    assert_eq!(
        open(&file(0, &[0])),
        Err(GelError::InvalidLength {
            expected: 0,
            actual: 1
        })
    );
    let trailing = [literal(s0), vec![0xAA]].concat();
    assert_eq!(
        open(&file(1, &trailing)),
        Err(GelError::InvalidLength {
            expected: 129,
            actual: 130
        })
    );

    // Fields behind the header CRC64, with the CRC64 recomputed.
    let one = literal(s0);
    assert_eq!(
        open(&file_with(2, 0, 1, &one)),
        Err(GelError::UnsupportedVersion(2))
    );
    assert_eq!(
        open(&file_with(1, 1, 1, &one)),
        Err(GelError::InvalidHeader(
            "history reserved field must be zero"
        ))
    );
    assert_eq!(
        open(&file_with(1, 0, 1, &one)).unwrap().latest(),
        Ok(Some(s0))
    );
}

#[test]
fn open_limits_hold_at_their_boundary_and_come_before_the_length() {
    let bytes = history_of(&walk(4, 2, 18)).to_bytes().unwrap();
    let payload = (bytes.len() - 48) as u64;
    // Two changed bits: 13 + ceil(20/8) = 16 bytes per residual.
    assert_eq!(payload, 129 + 16 + 16 + 129);
    assert_eq!(
        RecordHistory::from_bytes(&bytes, 4, payload).unwrap().len(),
        4
    );
    assert_eq!(
        RecordHistory::from_bytes(&bytes, 3, payload),
        Err(GelError::LimitExceeded("history record count"))
    );
    assert_eq!(
        RecordHistory::from_bytes(&bytes, 4, payload - 1),
        Err(GelError::LimitExceeded("history payload bytes"))
    );
    // A header alone that declares a terabyte of payload.
    let mut header = file(1, &[0; 129]);
    header[24..32].copy_from_slice(&(1u64 << 40).to_le_bytes());
    let crc = crc64_ecma(&header[..40]);
    header[40..48].copy_from_slice(&crc.to_le_bytes());
    header.truncate(48);
    assert_eq!(
        RecordHistory::from_bytes(&header, 10, 1 << 20),
        Err(GelError::LimitExceeded("history payload bytes"))
    );
}

#[test]
fn an_older_shorter_history_still_opens() {
    let states = walk(5, 6, 19);
    let older = history_of(&states[..3]).to_bytes().unwrap();
    let newer = history_of(&states).to_bytes().unwrap();
    assert!(older.len() < newer.len());
    // There is no generation: the older file is a valid history of three states.
    assert_eq!(open(&older).unwrap().exact_history().unwrap(), states[..3]);
    assert_eq!(open(&newer).unwrap().exact_history().unwrap(), states);
}

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The only test in this file that writes files: the first temporary name this
// process tries is `<path>.tmp-<pid>-0`.
#[test]
fn saved_history_reopens_exactly_and_rejects_a_changed_payload() {
    let dir = TempDir(std::env::temp_dir().join(format!(
        "gel-history-saved-{}-{}",
        std::process::id(),
        splitmix64(20)
    )));
    fs::create_dir_all(&dir.0).unwrap();
    let path = dir.0.join("record.gelhis");
    let leftover = dir
        .0
        .join(format!("record.gelhis.tmp-{}-0", std::process::id()));
    fs::write(&leftover, b"left by an earlier run").unwrap();

    let empty = RecordHistory::new();
    empty.write_atomic(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap().len(), 48);
    assert_eq!(RecordHistory::open_verified(&path, 0, 0), Ok(empty));
    assert_eq!(fs::read(&leftover).unwrap(), b"left by an earlier run");

    let states = walk(3, 9, 21);
    let history = history_of(&states);
    history.write_atomic(&path).unwrap();
    let saved = fs::read(&path).unwrap();
    assert_eq!(saved, history.to_bytes().unwrap());
    assert_eq!(saved.len(), 48 + 129 + 2 * (13 + 12));
    let reopened = RecordHistory::open_verified(&path, 3, (saved.len() - 48) as u64).unwrap();
    assert_eq!(reopened.exact_history().unwrap(), states);
    assert_eq!(
        RecordHistory::open_verified(&path, 2, NO_LIMIT),
        Err(GelError::LimitExceeded("history record count"))
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &PathBuf| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&path), 0o600);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let more = walk(4, 9, 21);
        history_of(&more).write_atomic(&path).unwrap();
        assert_eq!(mode(&path), 0o640);
        assert_eq!(
            RecordHistory::open_verified(&path, NO_LIMIT, NO_LIMIT)
                .unwrap()
                .exact_history()
                .unwrap(),
            more
        );
        history.write_atomic(&path).unwrap();
    }

    // Nothing is left behind except the file that was already there.
    let mut names: Vec<_> = fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names.sort();
    let leftover_name = leftover.file_name().unwrap().to_owned();
    assert_eq!(names, ["record.gelhis".into(), leftover_name]);
    assert_eq!(fs::read(&leftover).unwrap(), b"left by an earlier run");

    let mut changed = saved.clone();
    changed[48 + 1] ^= 0x01;
    fs::write(&path, &changed).unwrap();
    assert_eq!(
        RecordHistory::open_verified(&path, NO_LIMIT, NO_LIMIT),
        Err(GelError::CorruptStore)
    );

    let mut longer = saved.clone();
    longer.push(0);
    fs::write(&path, &longer).unwrap();
    assert_eq!(
        RecordHistory::open_verified(&path, NO_LIMIT, NO_LIMIT),
        Err(GelError::InvalidLength {
            expected: saved.len(),
            actual: saved.len() + 1
        })
    );

    fs::write(&path, &saved[..47]).unwrap();
    assert_eq!(
        RecordHistory::open_verified(&path, NO_LIMIT, NO_LIMIT),
        Err(GelError::InvalidHeader("history file shorter than header"))
    );

    // A header that declares a terabyte is refused before the payload is read.
    let mut huge = saved[..48].to_vec();
    huge[24..32].copy_from_slice(&(1u64 << 40).to_le_bytes());
    let crc = crc64_ecma(&huge[..40]);
    huge[40..48].copy_from_slice(&crc.to_le_bytes());
    fs::write(&path, &huge).unwrap();
    assert_eq!(
        RecordHistory::open_verified(&path, 10, 1 << 20),
        Err(GelError::LimitExceeded("history payload bytes"))
    );

    // A missing file, and a file in a missing directory.
    assert!(matches!(
        RecordHistory::open_verified(dir.0.join("missing"), NO_LIMIT, NO_LIMIT),
        Err(GelError::Io(_))
    ));
    match history.write_atomic(dir.0.join("missing").join("record.gelhis")) {
        Err(GelError::Io(message)) => {
            assert!(!message.contains("could not allocate"), "{message}")
        }
        other => panic!("{other:?}"),
    }
}
