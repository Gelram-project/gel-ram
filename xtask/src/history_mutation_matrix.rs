//! GELHIS01 record-history files in the format mutation matrix
//! (docs/MUTATION-MATRIX.md). A history file carries two CRC64 fields of its
//! own, so a mutant whose CRC64 no longer fits would only test the checksum.
//! Every mutant therefore has its CRC64 fields recomputed, except a CRC field
//! it changes itself and one deliberately unsealed mutant, and the structural
//! rules decide. The expected result is fixed by a small reference reader
//! written from the format description in docs/RECORD-HISTORY.md, with its own
//! CRC-64/ECMA-182, independent of the gel-history decoder. Like Q8DEMO01 the
//! file has no pin of its own: the caller applies its retained SHA-256 first.
//! The report is kept apart from matrix-r1.txt, which stays byte for byte.
use crate::mutation_matrix::{generic, region, row, Int, Mutant, Reading, Region};
use gel_core::GelError;
use gel_history::RecordHistory;
use gel_orb::Orb1024;
use gel_source::{digest, hex, Hash};
use std::{fs, path::Path};

const REPORT: &str = "docs/evidence-mutation/history-matrix-r1.txt";
const HEADER: usize = 48;
const STATE: usize = 128;
const MAX_ENTRIES: u64 = 1024;
const MAX_PAYLOAD: u64 = 1 << 20;

/// s0 literal; s1 three bits on (a residual), s2 one more bit (a residual two
/// from the literal), s3 two more bits (a literal, the depth limit reached).
fn states() -> [Orb1024; 4] {
    let flip = |mut state: Orb1024, bits: &[usize]| {
        for &bit in bits {
            state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
        }
        state
    };
    let s0 = Orb1024::from_words(std::array::from_fn(|i| {
        0x0123_4567_89ab_cdef_u64.wrapping_mul(i as u64 + 1)
    }));
    let s1 = flip(s0, &[5, 300, 777]);
    let s2 = flip(s1, &[41]);
    let s3 = flip(s2, &[100, 900]);
    [s0, s1, s2, s3]
}

fn base() -> Result<Vec<u8>, String> {
    let mut history = RecordHistory::new();
    for state in states() {
        history.append(state).map_err(|e| e.to_string())?;
    }
    history.to_bytes().map_err(|e| e.to_string())
}

/// The header fields and every field of the four entries, in file order.
fn regions() -> Vec<Region> {
    let mut regions = vec![
        region("magic", 0, 8, None),
        region("version", 8, 12, Some(Int::U32)),
        region("reserved", 12, 16, Some(Int::U32)),
        region("count", 16, 24, Some(Int::U64)),
        region("payload_len", 24, 32, Some(Int::U64)),
        region("payload_crc", 32, 40, None),
        region("header_crc", 40, 48, None),
    ];
    let mut at = HEADER;
    for (entry, positions) in [(0, None), (1, Some(4)), (2, Some(2)), (3, None)] {
        let name = |field: &str| format!("entry{entry}.{field}");
        regions.push(region(&name("tag"), at, at + 1, None));
        match positions {
            None => {
                regions.push(region(&name("state"), at + 1, at + 1 + STATE, None));
                at += 1 + STATE;
            }
            Some(packed) => {
                regions.push(region(&name("parent"), at + 1, at + 5, Some(Int::U32)));
                regions.push(region(&name("depth"), at + 5, at + 6, None));
                regions.push(region(&name("form"), at + 6, at + 7, None));
                regions.push(region(&name("count"), at + 7, at + 9, None));
                regions.push(region(&name("len"), at + 9, at + 13, Some(Int::U32)));
                regions.push(region(&name("positions"), at + 13, at + 13 + packed, None));
                at += 13 + packed;
            }
        }
    }
    regions
}

/// CRC-64/ECMA-182: polynomial 0x42F0E1EBA9EA3693, initial value 0, no reflection.
fn crc64(bytes: &[u8]) -> u64 {
    let mut crc = 0u64;
    for &byte in bytes {
        crc ^= u64::from(byte) << 56;
        for _ in 0..8 {
            crc = if crc >> 63 == 1 {
                (crc << 1) ^ 0x42f0_e1eb_a9ea_3693
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// Recomputes the payload CRC64 and then the header CRC64 of a file of at least
/// 48 bytes, leaving alone the CRC field the mutant itself changed.
fn reseal(bytes: &mut [u8], changed: &str) {
    if bytes.len() < HEADER {
        return;
    }
    if changed != "payload_crc" && changed != "header_crc" {
        let crc = crc64(&bytes[HEADER..]);
        bytes[32..40].copy_from_slice(&crc.to_le_bytes());
    }
    if changed != "header_crc" {
        let crc = crc64(&bytes[..40]);
        bytes[40..48].copy_from_slice(&crc.to_le_bytes());
    }
}

fn case() -> Result<(Vec<u8>, Vec<Mutant>), String> {
    let base = base()?;
    let mut mutants = generic(&base, &regions());
    for m in &mut mutants {
        reseal(&mut m.bytes, &m.region);
    }
    let mut push = |region: &str, operator: &str, bytes: Vec<u8>| {
        mutants.push(Mutant {
            region: region.into(),
            operator: operator.into(),
            bytes,
        })
    };
    // A changed state byte under the old CRC64.
    let mut unsealed = base.clone();
    unsealed[HEADER + 1 + 3] ^= 0x10;
    push("entry0.state", "unsealed-flip", unsealed);
    // entry1 stored as a literal of s1 although its residual fits; entry2's
    // depth byte continues the new chain, so only canonical form is broken.
    let mut literal = base[..177].to_vec();
    literal.push(0);
    literal.extend_from_slice(&states()[1].to_le_bytes());
    literal.extend_from_slice(&base[194..]);
    literal[177 + 1 + STATE + 5] = 1;
    let payload = (literal.len() - HEADER) as u64;
    literal[24..32].copy_from_slice(&payload.to_le_bytes());
    reseal(&mut literal, "entry1");
    push("entry1", "literal-where-residual-fits", literal);
    // entry2 names entry0 as its parent with a depth that fits that chain.
    let mut skip = base.clone();
    skip[195..199].copy_from_slice(&0u32.to_le_bytes());
    skip[199] = 1;
    reseal(&mut skip, "entry2");
    push("entry2", "parent-skips-previous", skip);
    Ok((base, mutants))
}

// ---------- reference reader from docs/RECORD-HISTORY.md ----------

enum Stored {
    Literal([u8; STATE]),
    Residual {
        parent: usize,
        depth: u8,
        count: u16,
        packed: Vec<u8>,
    },
}

fn le(bytes: &[u8], at: usize, n: usize) -> u64 {
    (0..n).fold(0, |acc, i| acc | (u64::from(bytes[at + i]) << (8 * i)))
}

fn take<'a>(payload: &'a [u8], at: &mut usize, n: usize) -> Result<&'a [u8], &'static str> {
    let bytes = at
        .checked_add(n)
        .and_then(|end| payload.get(*at..end))
        .ok_or("ENTRY")?;
    *at += n;
    Ok(bytes)
}

fn bit(bytes: &[u8], i: usize) -> bool {
    (bytes[i / 8] >> (i % 8)) & 1 == 1
}

/// k 10-bit positions packed little-endian, strictly increasing, zero padding.
fn unpack(count: u16, packed: &[u8]) -> Result<Vec<usize>, &'static str> {
    let k = usize::from(count);
    if k > 1024 {
        return Err("ENTRY");
    }
    if packed.len() != (10 * k).div_ceil(8) {
        return Err("LENGTH");
    }
    let mut out: Vec<usize> = Vec::with_capacity(k);
    for j in 0..k {
        let position = (0..10).fold(0, |acc, b| {
            acc | (usize::from(bit(packed, 10 * j + b)) << b)
        });
        if out.last().is_some_and(|&last| position <= last) {
            return Err("ENTRY");
        }
        out.push(position);
    }
    if (10 * k..8 * packed.len()).any(|i| bit(packed, i)) {
        return Err("ENTRY");
    }
    Ok(out)
}

fn pack(positions: &[usize]) -> Vec<u8> {
    let mut out = vec![0u8; (10 * positions.len()).div_ceil(8)];
    for (j, &position) in positions.iter().enumerate() {
        for b in (0..10).filter(|b| (position >> b) & 1 == 1) {
            out[(10 * j + b) / 8] |= 1 << ((10 * j + b) % 8);
        }
    }
    out
}

/// Entry `i` is the one the encoder writes: a literal first, after two
/// residuals, or when 13 + ceil(10k/8) bytes would not be smaller than 129;
/// otherwise the residual from entry i - 1.
fn is_canonical(
    i: usize,
    entry: &Stored,
    state: &[u8; STATE],
    earlier: &[[u8; STATE]],
    depths: &[u8],
) -> bool {
    let changed: Vec<usize> = match i.checked_sub(1) {
        None => Vec::new(),
        Some(p) => (0..8 * STATE)
            .filter(|&q| bit(state, q) != bit(&earlier[p], q))
            .collect(),
    };
    let depth = i.checked_sub(1).map_or(0, |p| depths[p] + 1);
    let literal = i == 0 || depth > 2 || 13 + (10 * changed.len()).div_ceil(8) >= 1 + STATE;
    match entry {
        Stored::Literal(_) => literal,
        Stored::Residual {
            parent,
            depth: stored,
            count,
            packed,
        } => {
            !literal
                && *parent == i - 1
                && *stored == depth
                && usize::from(*count) == changed.len()
                && *packed == pack(&changed)
        }
    }
}

/// Reads in file order; the first rule a file breaks names its class.
/// `canonical: false` is the deliberately lenient reader of the negative control.
fn read_states(b: &[u8], canonical: bool) -> Result<Vec<[u8; STATE]>, &'static str> {
    if b.len() < HEADER {
        return Err("HEADER");
    }
    if &b[..8] != b"GELHIS01" {
        return Err("MAGIC");
    }
    if crc64(&b[..40]) != le(b, 40, 8) {
        return Err("HEADER_CRC");
    }
    if le(b, 8, 4) != 1 {
        return Err("VERSION");
    }
    if le(b, 12, 4) != 0 {
        return Err("HEADER");
    }
    let (count, len, crc) = (le(b, 16, 8), le(b, 24, 8), le(b, 32, 8));
    if count > MAX_ENTRIES || len > MAX_PAYLOAD {
        return Err("LIMIT");
    }
    // The smallest entry, a residual with no changed bit, takes 13 bytes.
    if count > len / 13 {
        return Err("HEADER");
    }
    let payload = &b[HEADER..];
    if payload.len() as u64 != len {
        return Err("LENGTH");
    }
    if crc64(payload) != crc {
        return Err("PAYLOAD_CRC");
    }
    let (mut at, mut stored, mut depths) = (0, Vec::new(), Vec::new());
    while stored.len() < count as usize {
        let entry = match take(payload, &mut at, 1)?[0] {
            0 => {
                let mut state = [0u8; STATE];
                state.copy_from_slice(take(payload, &mut at, STATE)?);
                Stored::Literal(state)
            }
            1 => {
                let parent = le(take(payload, &mut at, 4)?, 0, 4) as usize;
                let depth = take(payload, &mut at, 1)?[0];
                let parent_depth = *depths.get(parent).ok_or("ENTRY")?;
                if depth != parent_depth + 1 || depth > 2 {
                    return Err("ENTRY");
                }
                if take(payload, &mut at, 1)?[0] != 0 {
                    return Err("ENTRY");
                }
                let count = le(take(payload, &mut at, 2)?, 0, 2) as u16;
                let n = le(take(payload, &mut at, 4)?, 0, 4) as usize;
                let packed = take(payload, &mut at, n)?.to_vec();
                Stored::Residual {
                    parent,
                    depth,
                    count,
                    packed,
                }
            }
            _ => return Err("ENTRY"),
        };
        depths.push(match entry {
            Stored::Literal(_) => 0,
            Stored::Residual { depth, .. } => depth,
        });
        stored.push(entry);
    }
    if at != payload.len() {
        return Err("LENGTH");
    }
    let mut states: Vec<[u8; STATE]> = Vec::with_capacity(stored.len());
    for (i, entry) in stored.iter().enumerate() {
        let state = match entry {
            Stored::Literal(state) => *state,
            Stored::Residual {
                parent,
                count,
                packed,
                ..
            } => {
                let mut state = states[*parent];
                for q in unpack(*count, packed)? {
                    state[q / 8] ^= 1 << (q % 8);
                }
                state
            }
        };
        if canonical && !is_canonical(i, entry, &state, &states, &depths) {
            return Err("CANONICAL");
        }
        states.push(state);
    }
    Ok(states)
}

fn summary<'a>(states: impl Iterator<Item = &'a [u8]>) -> String {
    states
        .map(|state| hex(&digest(state)))
        .collect::<Vec<_>>()
        .join(";")
}

fn reference(b: &[u8], canonical: bool) -> Reading {
    match read_states(b, canonical) {
        Ok(states) => Reading::Accept(summary(states.iter().map(|s| &s[..]))),
        Err(class) => Reading::Reject(class.into()),
    }
}

fn class(error: &GelError) -> String {
    match error {
        GelError::InvalidMagic => "MAGIC".into(),
        GelError::CorruptHeader => "HEADER_CRC".into(),
        GelError::UnsupportedVersion(_) => "VERSION".into(),
        GelError::InvalidHeader(_) => "HEADER".into(),
        GelError::LimitExceeded(_) => "LIMIT".into(),
        GelError::InvalidLength { .. } => "LENGTH".into(),
        GelError::CorruptStore => "PAYLOAD_CRC".into(),
        GelError::InvalidResidual("history file is not in canonical form") => "CANONICAL".into(),
        GelError::InvalidResidual(_) => "ENTRY".into(),
        other => format!("OTHER({other})"),
    }
}

fn library(b: &[u8], pin: Hash) -> Reading {
    // GELHIS01 has no pin of its own: the caller applies its retained pin first.
    if digest(b) != pin {
        return Reading::Reject("PIN(caller)".into());
    }
    match RecordHistory::from_bytes(b, MAX_ENTRIES, MAX_PAYLOAD).and_then(|h| h.exact_history()) {
        Ok(states) => {
            let bytes: Vec<_> = states.iter().map(|s| s.to_le_bytes()).collect();
            Reading::Accept(summary(bytes.iter().map(|s| &s[..])))
        }
        Err(e) => Reading::Reject(class(&e)),
    }
}

/// The report, the number of mutants and the number of failed rows.
fn build() -> Result<(String, usize, usize), String> {
    let (base, mutants) = case()?;
    let pin = digest(&base);
    if reference(&base, true) != library(&base, pin)
        || !matches!(library(&base, pin), Reading::Accept(_))
    {
        return Err(
            "the GELHIS01 base file is not read alike by the reference and the library".into(),
        );
    }
    let mut out =
        String::from("format\tcase\tregion\toperator\tpin\texpected\tobserved\tverdict\n");
    let mut failed = 0;
    for (n, m) in mutants.iter().enumerate() {
        for (mode, used, expected) in [
            ("original", pin, Reading::Reject("PIN(caller)".into())),
            ("recomputed", digest(&m.bytes), reference(&m.bytes, true)),
        ] {
            let observed = library(&m.bytes, used);
            let ok = expected == observed;
            failed += usize::from(!ok);
            row(
                &mut out,
                [
                    "GELHIS01",
                    &n.to_string(),
                    &m.region,
                    &m.operator,
                    mode,
                    &expected.label(),
                    &observed.label(),
                    if ok { "PASS" } else { "FAIL" },
                ],
            );
        }
    }
    Ok((out, mutants.len(), failed))
}

/// Negative control: mutants that tell the reference apart from a reader
/// without the canonical-form rule. Zero would mean the matrix cannot catch
/// such a reader, so the gate fails.
fn negative_control() -> Result<usize, String> {
    let (_, mutants) = case()?;
    Ok(mutants
        .iter()
        .filter(|m| reference(&m.bytes, true) != reference(&m.bytes, false))
        .count())
}

fn evaluate() -> Result<(String, String), String> {
    let (report, mutants, failed) = build()?;
    let control = negative_control()?;
    let accepted = report
        .lines()
        .filter(|l| l.contains("\trecomputed\tACCEPT\tACCEPT\t"))
        .count();
    let line = format!(
        "HISTORY_MUTATION_MATRIX={} mutants={mutants} rows={} accepted_valid_changes={accepted} failed={failed} negative_control_distinguishing_mutants={control}",
        if failed == 0 && control > 0 { "PASS" } else { "FAIL" },
        2 * mutants,
    );
    if failed > 0 || control == 0 {
        for fail in report.lines().filter(|l| l.ends_with("\tFAIL")) {
            eprintln!("{fail}");
        }
        return Err(line);
    }
    Ok((report, line))
}

/// `history-mutation-matrix`: prints the summary; `--write NEW_FILE` also saves the report.
pub fn run(args: &[String]) -> Result<(), String> {
    let path = match args {
        [] => None,
        [flag, path] if flag == "--write" => Some(path),
        _ => {
            return Err(
                "usage: cargo run -p xtask -- history-mutation-matrix [--write NEW_FILE]".into(),
            )
        }
    };
    let (report, line) = evaluate()?;
    if let Some(path) = path {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, report.as_bytes()))
            .map_err(|e| format!("{path}: {e}"))?;
    }
    println!("{line}");
    Ok(())
}

/// verify: the matrix must pass and equal the committed report byte for byte.
pub fn check(root: impl AsRef<Path>) -> Result<(), String> {
    let (report, line) = evaluate()?;
    let committed =
        fs::read_to_string(root.as_ref().join(REPORT)).map_err(|e| format!("{REPORT}: {e}"))?;
    if committed != report {
        return Err(format!("{REPORT} differs from the regenerated matrix"));
    }
    println!("{line} report={REPORT}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_crc_has_the_ecma_182_check_value() {
        assert_eq!(crc64(b"123456789"), 0x6c40_df5f_0b49_7347);
        assert_eq!(crc64(b""), 0);
    }

    #[test]
    fn the_base_file_is_338_bytes_and_the_regions_cover_it_in_order() {
        let base = base().unwrap();
        assert_eq!(base.len(), 338);
        let regions = regions();
        assert_eq!(regions.len(), 25);
        assert_eq!(regions[0].start, 0);
        for pair in regions.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "{}", pair[1].name);
        }
        assert_eq!(regions.last().unwrap().end, base.len());
    }

    #[test]
    fn the_reference_rebuilds_the_four_states_of_the_base_file() {
        let states_read = read_states(&base().unwrap(), true).unwrap();
        let expected: Vec<[u8; STATE]> = states().iter().map(|s| s.to_le_bytes()).collect();
        assert_eq!(states_read, expected);
        assert_eq!(unpack(3, &pack(&[5, 300, 777])), Ok(vec![5, 300, 777]));
        assert_eq!(pack(&[41]), vec![41, 0]);
    }

    #[test]
    fn resealing_leaves_a_changed_crc_field_alone() {
        let base = base().unwrap();
        let mut changed = base.clone();
        changed[40] ^= 1;
        reseal(&mut changed, "header_crc");
        assert_eq!(read_states(&changed, true), Err("HEADER_CRC"));
        let mut changed = base.clone();
        changed[32] ^= 1;
        reseal(&mut changed, "payload_crc");
        assert_eq!(read_states(&changed, true), Err("PAYLOAD_CRC"));
        let mut changed = base;
        changed[8] = 2;
        reseal(&mut changed, "version");
        assert_eq!(read_states(&changed, true), Err("VERSION"));
    }
}
