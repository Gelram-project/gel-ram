//! Identity check for the word-level mask paths: `fixture::decode`,
//! `fixture::encode`, `Record::new`, `Reader::prepare` and `Reader::view`
//! against exact copies of the earlier per-position algorithms, on seeded
//! inputs and edge cases, compared bit for bit.
//!
//! Usage: `cargo run --release -p gel-phase-quad --example identity_q8_masks`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when no case
//! ran); exit status 0 on PASS, 1 on FAIL. The integration test
//! `tests/identity_q8_masks.rs` runs the same comparison. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_phase_quad::fixture::{self, MAGIC, MAX_RECORDS, RECORD_BYTES};
use gel_phase_quad::grid::{self, Carrier, Grid, DIM};
use gel_phase_quad::{Policy, Reader, Record, View};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn phase(&mut self) -> Carrier {
        std::array::from_fn(|_| self.next() as u8)
    }
    fn mask(&mut self, per_256: u64) -> [bool; DIM] {
        std::array::from_fn(|_| self.next() % 256 < per_256)
    }
    fn bytes(&mut self, out: &mut [u8]) {
        for b in out {
            *b = self.next() as u8;
        }
    }
}

// ---- Exact copies of the earlier algorithms (reference) ----

fn reference_decode(raw: &[u8]) -> Result<Vec<Record>, String> {
    if raw.len() < 12 || &raw[..8] != MAGIC {
        return Err("invalid Q8DEMO01 header".into());
    }
    let count = u32::from_le_bytes(raw[8..12].try_into().unwrap()) as usize;
    if !(1..=MAX_RECORDS).contains(&count) || raw.len() != 12 + count * RECORD_BYTES {
        return Err("invalid record count or exact file length".into());
    }
    Ok(raw[12..]
        .chunks_exact(RECORD_BYTES)
        .map(|r| {
            let phase = r[..DIM].try_into().unwrap();
            let active = std::array::from_fn(|j| r[DIM + j / 8] & (1 << (j % 8)) != 0);
            Record::new(phase, &active)
        })
        .collect())
}

fn reference_encode(records: &[Record]) -> Result<Vec<u8>, String> {
    if !(1..=MAX_RECORDS).contains(&records.len()) {
        return Err("records must be 1..8192".into());
    }
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for record in records {
        out.extend_from_slice(record.phase());
        let mask = record.active_mask();
        out.extend((0..DIM / 8).map(|byte| {
            (0..8).fold(0u8, |bits, bit| {
                bits | (u8::from(mask[byte * 8 + bit]) << bit)
            })
        }));
    }
    Ok(out)
}

/// The earlier `Record::new` packing loop, as sixteen words.
fn reference_pack(active: &[bool; DIM]) -> [u64; DIM / 64] {
    let mut mask = [0; DIM / 64];
    for (j, &yes) in active.iter().enumerate() {
        if yes {
            mask[j / 64] |= 1u64 << (j % 64);
        }
    }
    mask
}

/// The words a record carries, read back through the unchanged public readout.
fn words_of(record: &Record) -> [u64; DIM / 64] {
    reference_pack(&record.active_mask())
}

/// Earlier `prepare` (indices by `active(j)`) followed by the earlier
/// per-position score, written from public API only.
fn reference_score(cos: &[f64; 256], query: &Record, body: &Record, policy: Policy) -> Option<u64> {
    let active: Vec<usize> = (0..DIM).filter(|&j| query.active(j).unwrap()).collect();
    if active.is_empty() {
        return None;
    }
    let mut hist = [0u32; 256];
    for &j in &active {
        if matches!(policy, Policy::Archive) || body.active(j).unwrap() {
            hist[query.phase()[j].wrapping_sub(body.phase()[j]) as usize] += 1;
        }
    }
    let s = hist
        .iter()
        .zip(cos)
        .map(|(&n, &c)| n as f64 * c)
        .sum::<f64>()
        / active.len() as f64;
    Some(s.to_bits())
}

fn reference_view(grid: &Grid, record: &Record, pole: u8) -> Result<View, String> {
    Ok(View {
        phase: grid.view(record.phase(), pole)?,
        active: Grid::support_view(&record.active_mask(), pole)?,
    })
}

// ---- Comparison ----

/// Counts one case: one comparison per item of `results`.
fn case(tallies: &mut Vec<Tally>, name: &str, results: impl IntoIterator<Item = bool>) {
    let mut tally = Tally::new(name);
    tally.check_all(results);
    tallies.push(tally);
}

fn same_records(a: &[Record], b: &[Record]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.phase() == y.phase() && words_of(x) == words_of(y))
}

fn edge_masks(rng: &mut Rng) -> Vec<[bool; DIM]> {
    let mut masks = vec![[false; DIM], [true; DIM]];
    for bits in [
        &[0][..],
        &[63],
        &[64],
        &[127],
        &[1023],
        &[0, 1023],
        &[63, 64, 511, 512, 960],
    ] {
        masks.push(std::array::from_fn(|j| bits.contains(&j)));
    }
    masks.push(std::array::from_fn(|j| j >= DIM - 64));
    masks.push(std::array::from_fn(|j| j < 64));
    masks.push(std::array::from_fn(|j| j % 2 == 0));
    masks.push(std::array::from_fn(|j| j % 3 == 0));
    for per_256 in [1, 4, 64, 128, 171, 252, 255] {
        for _ in 0..8 {
            masks.push(rng.mask(per_256));
        }
    }
    masks
}

fn file_with(rng: &mut Rng, count: usize) -> Vec<u8> {
    let mut raw = vec![0u8; 12 + count * RECORD_BYTES];
    raw[..8].copy_from_slice(MAGIC);
    raw[8..12].copy_from_slice(&(count as u32).to_le_bytes());
    rng.bytes(&mut raw[12..]);
    raw
}

/// Runs every comparison and returns one tally per case. There is no smaller
/// scale: `scale` is accepted for the common interface only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    let mut tallies = Vec::new();
    let mut rng = Rng(0x6765_6c5f_7138_6964);
    let masks = edge_masks(&mut rng);

    // Record::new packs exactly the earlier words.
    case(
        &mut tallies,
        "record_new",
        masks.iter().map(|a| {
            let p = rng.phase();
            let r = Record::new(p, a);
            *r.phase() == p && r.active_mask() == *a && words_of(&r) == reference_pack(a)
        }),
    );

    // decode on valid files: 1, 2, 7, 64, random and the maximum count.
    let mut files: Vec<Vec<u8>> = [1, 2, 7, 64, MAX_RECORDS]
        .into_iter()
        .map(|n| file_with(&mut rng, n))
        .collect();
    for _ in 0..16 {
        let n = 1 + (rng.next() % 300) as usize;
        files.push(file_with(&mut rng, n));
    }
    // Mask bytes all zero and all 0xff in one file.
    let mut extremes = file_with(&mut rng, 2);
    extremes[12 + DIM..12 + RECORD_BYTES].fill(0);
    extremes[12 + RECORD_BYTES + DIM..].fill(0xff);
    files.push(extremes);
    case(
        &mut tallies,
        "decode_valid",
        files
            .iter()
            .map(|raw| match (fixture::decode(raw), reference_decode(raw)) {
                (Ok(a), Ok(b)) => same_records(&a, &b),
                _ => false,
            }),
    );

    // decode errors keep their exact text.
    let mut bad: Vec<Vec<u8>> = vec![
        Vec::new(),
        MAGIC[..7].to_vec(),
        MAGIC.to_vec(),
        b"Q8DEMO02\x01\x00\x00\x00".to_vec(),
    ];
    let mut zero = MAGIC.to_vec();
    zero.extend_from_slice(&0u32.to_le_bytes());
    bad.push(zero);
    let mut over = file_with(&mut rng, 1);
    over[8..12].copy_from_slice(&((MAX_RECORDS + 1) as u32).to_le_bytes());
    bad.push(over);
    let mut short = file_with(&mut rng, 3);
    short.pop();
    bad.push(short);
    let mut long = file_with(&mut rng, 3);
    long.push(0);
    bad.push(long);
    let mut header_bad = file_with(&mut rng, 1);
    header_bad[0] ^= 1;
    bad.push(header_bad);
    case(
        &mut tallies,
        "decode_errors",
        bad.iter()
            .map(|raw| match (fixture::decode(raw), reference_decode(raw)) {
                (Err(a), Err(b)) => a == b,
                _ => false,
            }),
    );

    // encode: byte-equal with the earlier fold, and the inverse of decode.
    let mut sets: Vec<Vec<Record>> = files
        .iter()
        .map(|raw| reference_decode(raw).unwrap())
        .collect();
    sets.push(masks.iter().map(|a| Record::new(rng.phase(), a)).collect());
    case(
        &mut tallies,
        "encode_valid",
        sets.iter()
            .map(|set| match (fixture::encode(set), reference_encode(set)) {
                (Ok(a), Ok(b)) => a == b,
                _ => false,
            }),
    );
    case(
        &mut tallies,
        "encode_decode_roundtrip",
        files
            .iter()
            .map(|raw| fixture::encode(&fixture::decode(raw).unwrap()).as_ref() == Ok(raw)),
    );
    let too_many = vec![Record::new([0; DIM], &[true; DIM]); MAX_RECORDS + 1];
    case(
        &mut tallies,
        "encode_errors",
        [&[][..], &too_many[..]].into_iter().map(|set| {
            match (fixture::encode(set), reference_encode(set)) {
                (Err(a), Err(b)) => a == b,
                _ => false,
            }
        }),
    );

    // prepare, observed through read: support set and denominator.
    let cos = grid::cos256();
    let mut prepare_results = Vec::new();
    let mut view_results = Vec::new();
    for seed in [0u64, 1, 210_021, u64::MAX] {
        let reader = Reader::new(seed);
        let grid = Grid::new(256, seed).unwrap();
        let bodies: Vec<Record> = masks
            .iter()
            .step_by(7)
            .map(|a| Record::new(rng.phase(), a))
            .collect();
        for a in &masks {
            let record = Record::new(rng.phase(), a);
            let q = reader.prepare(record.clone());
            for body in &bodies {
                for policy in [Policy::Archive, Policy::BodyActivity] {
                    let got = reader.read(&q, body, policy).map(|s| s.value().to_bits());
                    prepare_results.push(got == reference_score(&cos, &record, body, policy));
                }
            }
            for pole in 0..=5u8 {
                let ok = match (
                    reader.view(&record, pole),
                    reference_view(&grid, &record, pole),
                ) {
                    (Ok(x), Ok(y)) => x.phase == y.phase && x.active == y.active,
                    (Err(x), Err(y)) => x == y,
                    _ => false,
                };
                view_results.push(ok);
            }
        }
    }
    case(&mut tallies, "prepare_read", prepare_results);
    case(&mut tallies, "view_poles", view_results);

    tallies
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
