//! Stored bytes of the record history on synthetic walks of 3,000 states.
//!
//! Each walk changes exactly k bits per step, chosen by splitmix64; the last
//! row uses unrelated states. Every row is checked against the format,
//! 48 + sum(129 for a literal | 13 + ceil(10k/8) for a residual), every state
//! is rebuilt and reopened from the bytes, and the process exits nonzero on
//! any difference. Nothing is written to disk.

use gel_core::{splitmix64, GelError, ORB_BITS, ORB_WORDS};
use gel_history::{HistoryEntry, RecordHistory, MAX_RESIDUAL_DEPTH};
use gel_orb::Orb1024;
use std::process::ExitCode;

const STATES: usize = 3_000;
const STATE_BYTES: usize = 128;

fn sample(seed: u64) -> Orb1024 {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(seed.wrapping_add(i as u64));
    }
    Orb1024::from_words(words)
}

/// The next state: exactly `k` distinct bits changed, chosen by splitmix64.
fn step(mut state: Orb1024, k: usize, seed: &mut u64) -> Orb1024 {
    let mut changed = [false; ORB_BITS];
    let mut left = k;
    while left > 0 {
        *seed = seed.wrapping_add(1);
        let bit = (splitmix64(*seed) % ORB_BITS as u64) as usize;
        if !changed[bit] {
            changed[bit] = true;
            state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
            left -= 1;
        }
    }
    state
}

fn walk(k: Option<usize>, seed: u64) -> Vec<Orb1024> {
    let mut cursor = seed;
    let mut states = vec![sample(seed)];
    while states.len() < STATES {
        let next = match k {
            Some(k) => step(states[states.len() - 1], k, &mut cursor),
            None => sample(seed.wrapping_add(1_000_003 * states.len() as u64)),
        };
        states.push(next);
    }
    states
}

/// The format's own count: a residual of k bits takes 13 + ceil(10k/8) bytes
/// and is kept when smaller than the 129-byte literal; at most two residuals
/// follow a literal. Unrelated states are all literals.
fn expected(k: Option<usize>) -> (usize, usize, usize) {
    let residual = k.map(|k| 13 + (10 * k).div_ceil(8)).filter(|&b| b < 129);
    match residual {
        Some(bytes) => {
            let literals = STATES.div_ceil(3);
            let residuals = STATES - literals;
            (literals, residuals, 48 + literals * 129 + residuals * bytes)
        }
        None => (STATES, 0, 48 + STATES * 129),
    }
}

struct Row {
    literals: usize,
    residuals: usize,
    stored: usize,
    exact: bool,
}

fn measure(states: &[Orb1024]) -> Result<Row, GelError> {
    let mut history = RecordHistory::new();
    for state in states {
        history.append(*state)?;
    }
    let bytes = history.to_bytes()?;
    let reopened = RecordHistory::from_bytes(&bytes, STATES as u64, bytes.len() as u64)?;
    let literals = history
        .entries()
        .iter()
        .filter(|entry| matches!(entry, HistoryEntry::Literal(_)))
        .count();
    let depth_ok = history.entries().iter().all(|entry| match entry {
        HistoryEntry::Literal(_) => true,
        HistoryEntry::Residual { depth, .. } => *depth <= MAX_RESIDUAL_DEPTH,
    });
    Ok(Row {
        literals,
        residuals: history.len() - literals,
        stored: bytes.len(),
        exact: depth_ok
            && history.encoded_len() == bytes.len()
            && history.exact_history()? == states
            && reopened.exact_history()? == states,
    })
}

fn main() -> ExitCode {
    let rows: [(Option<usize>, &str); 8] = [
        (Some(0), "0"),
        (Some(1), "1"),
        (Some(8), "8"),
        (Some(32), "32"),
        (Some(64), "64"),
        (Some(92), "92"),
        (Some(93), "93"),
        (None, "unrelated"),
    ];
    println!(
        "RECORD_HISTORY states={STATES} full_copy_bytes={}",
        STATES * STATE_BYTES
    );
    println!("k\tliterals\tresiduals\tstored_bytes\texpected_bytes\trebuilt");
    let mut failed = 0;
    for (i, (k, label)) in rows.iter().enumerate() {
        let states = walk(*k, 0x5EED_0000 + i as u64);
        let (literals, residuals, bytes) = expected(*k);
        let row = match measure(&states) {
            Ok(row) => row,
            Err(error) => {
                eprintln!("RECORD_HISTORY=FAIL k={label}: {error}");
                return ExitCode::FAILURE;
            }
        };
        let ok = row.exact
            && row.literals == literals
            && row.residuals == residuals
            && row.stored == bytes;
        failed += usize::from(!ok);
        println!(
            "{label}\t{}\t{}\t{}\t{bytes}\t{}",
            row.literals,
            row.residuals,
            row.stored,
            if row.exact { "exact" } else { "DIFFERENT" }
        );
    }
    if failed != 0 {
        println!("RECORD_HISTORY=FAIL rows_different={failed}");
        return ExitCode::FAILURE;
    }
    println!(
        "RECORD_HISTORY=PASS rows={} states_rebuilt={} synthetic walks only",
        rows.len(),
        rows.len() * STATES
    );
    ExitCode::SUCCESS
}
