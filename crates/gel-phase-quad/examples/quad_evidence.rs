//! Bounded numeric fixture + canonical single-read baseline. No semantic or hardware-PUF claims.
#![forbid(unsafe_code)]
#[path = "../src/reference.rs"]
#[allow(dead_code)]
mod reference;
use gel_phase_quad::{
    grid,
    grid::{Grid, DIM},
    Policy, Reader, Record,
};
use std::{
    fs,
    hint::black_box,
    io::{Read, Write},
    time::Instant,
};

const MAGIC: &[u8; 8] = b"Q8DEMO01";
const RECORD_BYTES: usize = 1152;
const MAX_RECORDS: usize = 8192;
const MAX_BYTES: usize = 12 + MAX_RECORDS * RECORD_BYTES;

fn decode(raw: &[u8]) -> Result<Vec<Record>, String> {
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

fn synthetic(count: usize) -> Result<Vec<u8>, String> {
    if !(1..=MAX_RECORDS).contains(&count) {
        return Err("records must be 1..8192".into());
    }
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(count as u32).to_le_bytes());
    let mut seed = 100_u64;
    for n in 0..count {
        for j in 0..DIM {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            let code = match n % 4 {
                0 => 0,
                1 => 255,
                2 => j as u8,
                _ => (z ^ (z >> 31)) as u8,
            };
            out.push(code);
        }
        out.extend(std::iter::repeat(if n % 2 == 0 { 0xff } else { 0x55 }).take(128));
    }
    Ok(out)
}

// Independent scalar histogram over the SAME packed Record layout, not four Frames.
// Deliberately simple baseline, not a claim to be the fastest possible single reader.
fn canonical(query: &Record, body: &Record, masked: bool, cos: &[f64; DIM / 4]) -> Option<f64> {
    let mut hist = [0u32; 256];
    let mut count = 0;
    for j in 0..DIM {
        if query.active(j).unwrap() {
            count += 1;
            if !masked || body.active(j).unwrap() {
                hist[query.phase()[j].wrapping_sub(body.phase()[j]) as usize] += 1;
            }
        }
    }
    (count != 0).then(|| {
        hist.iter()
            .zip(cos)
            .map(|(&n, &c)| n as f64 * c)
            .sum::<f64>()
            / count as f64
    })
}

fn benchmark(bank: &[Record], rounds: usize, masked: bool) -> Result<(), String> {
    let reader = Reader::new(510051);
    let grid = Grid::new(256, 510051)?;
    let eligible: Vec<_> = bank
        .iter()
        .enumerate()
        .filter(|(_, r)| r.active_mask().iter().any(|&x| x))
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err("no nonempty query masks; no successful benchmark".into());
    }
    let mut inverses = 0;
    for record in bank {
        for pole in 0..4 {
            let restored = reader.restore_bound_view(&reader.bound_view(record, pole)?)?;
            if restored.phase() != record.phase() || restored.active_mask() != record.active_mask()
            {
                return Err("bound view inverse mismatch".into());
            }
            inverses += 1;
        }
    }
    let views: Vec<_> = bank
        .iter()
        .map(|r| reference::frames(r.phase(), &r.active_mask(), &grid))
        .collect();
    let cos = std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos());
    let mut one = vec![0.0; bank.len()];
    let mut four = vec![[0.0; 4]; bank.len()];
    let mut shared = Vec::with_capacity(bank.len());
    let mut comparisons = 0;
    let mut total = [0u128; 3];
    println!("Q8_EVIDENCE_V1 records={} rounds={rounds} policy={} workers=1 view_inverses={inverses} timed_query=bank_record_with_one_code_perturbed semantic_accuracy=NOT_MEASURED", bank.len(), if masked {"active"} else {"archive"});
    println!("record_bytes={} materialized_four_bytes={} simultaneous_banks=true query_preparation_timed=false", std::mem::size_of::<Record>(), std::mem::size_of::<[reference::Frame; 4]>());
    println!("round,canonical_single_ns,four_materialized_ns,shared_ns");
    for round in 0..rounds + 2 {
        let i = eligible[(round * 65537) % eligible.len()];
        let mut phase = *bank[i].phase();
        phase[round % DIM] = phase[round % DIM].wrapping_add(17);
        let query = Record::new(phase, &bank[i].active_mask());
        let frames = reference::frames(query.phase(), &query.active_mask(), &grid);
        let indices: [Vec<usize>; 4] = std::array::from_fn(|p| reference::indices(&frames[p]));
        let prepared = reader.prepare(query.clone());
        let mut elapsed = [0; 3];
        for offset in 0..3 {
            let method = (round + offset) % 3;
            let start = Instant::now();
            match method {
                0 => {
                    for (body, out) in black_box(bank).iter().zip(&mut one) {
                        *out = canonical(black_box(&query), body, masked, &cos).unwrap();
                    }
                    black_box(&one);
                }
                1 => {
                    for (body, out) in black_box(&views).iter().zip(&mut four) {
                        for p in 0..4 {
                            out[p] =
                                reference::score(&frames[p], &body[p], &indices[p], masked, &cos)
                                    .unwrap();
                        }
                    }
                    black_box(&four);
                }
                _ => {
                    if !reader.scan_into(
                        black_box(&prepared),
                        black_box(bank),
                        if masked {
                            Policy::BodyActivity
                        } else {
                            Policy::Archive
                        },
                        1,
                        &mut shared,
                    )? {
                        return Err("unexpected empty query".into());
                    }
                    black_box(&shared);
                }
            }
            elapsed[method] = start.elapsed().as_nanos();
        }
        for j in 0..bank.len() {
            if one[j].to_bits() != shared[j].value().to_bits()
                || four[j].iter().any(|x| x.to_bits() != one[j].to_bits())
            {
                return Err("score parity failed".into());
            }
            comparisons += 4;
        }
        if round >= 2 {
            println!("{},{},{},{}", round - 1, elapsed[0], elapsed[1], elapsed[2]);
            for m in 0..3 {
                total[m] += elapsed[m];
            }
        }
    }
    println!(
        "canonical_over_shared={:.6} four_over_shared={:.6} view_comparisons={comparisons}",
        total[0] as f64 / total[2] as f64,
        total[1] as f64 / total[2] as f64
    );
    println!("Q8_EVIDENCE=PASS");
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    if args == ["--demo"] {
        return benchmark(&decode(&synthetic(32)?)?, 9, true);
    }
    if args.first().map(String::as_str) == Some("--generate") && args.len() == 3 {
        let count = args[2]
            .parse::<usize>()
            .map_err(|_| "invalid record count")?;
        let data = synthetic(count)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[1])
            .map_err(|e| e.to_string())?;
        file.write_all(&data).map_err(|e| e.to_string())?;
        println!(
            "Q8_SYNTHETIC_FIXTURE_CREATED records={count} bytes={}",
            data.len()
        );
        return Ok(());
    }
    if args.len() != 3 {
        return Err(
            "usage: quad_evidence --generate NEW_PATH RECORDS | PATH ROUNDS active|archive".into(),
        );
    }
    let rounds = args[1].parse::<usize>().map_err(|_| "invalid rounds")?;
    if !(1..=99).contains(&rounds) {
        return Err("rounds must be 1..99".into());
    }
    let masked = match args[2].as_str() {
        "active" => true,
        "archive" => false,
        _ => return Err("invalid policy".into()),
    };
    // Reject stable special files before open (a FIFO can block during open itself).
    // This CLI expects a caller-controlled directory: the pathname check is not
    // protection against another process replacing the file between these calls.
    if !fs::symlink_metadata(&args[0])
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("fixture must be a regular file, not a symlink or special file".into());
    }
    // Read at most cap+1 even when the file grows; reject oversize before parsing/allocating records.
    let file = fs::File::open(&args[0]).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("fixture must be a regular file".into());
    }
    let mut raw = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > MAX_BYTES {
        return Err("fixture exceeds byte budget".into());
    }
    benchmark(&decode(&raw)?, rounds, masked)
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("quad_evidence --demo | --generate NEW_PATH RECORDS | PATH ROUNDS active|archive\nNumeric compatibility/scan test, not semantic retrieval. Input limit: 8192 records.");
        return std::process::ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Q8_EVIDENCE=ERROR {e}");
            std::process::ExitCode::from(2)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_header_bit_and_every_truncation_rejected() {
        let raw = synthetic(1).unwrap();
        for length in 0..raw.len() {
            assert!(decode(&raw[..length]).is_err(), "length={length}");
        }
        for byte in 0..12 {
            for bit in 0..8 {
                let mut bad = raw.clone();
                bad[byte] ^= 1 << bit;
                assert!(decode(&bad).is_err(), "header byte={byte} bit={bit}");
            }
        }
    }
    #[test]
    fn upper_bound_and_payload_bits_roundtrip() {
        assert_eq!(
            decode(&synthetic(MAX_RECORDS).unwrap()).unwrap().len(),
            MAX_RECORDS
        );
        let mut raw = synthetic(1).unwrap();
        // Payload changes are valid data, not detectable corruption: this format
        // has no checksum or signature. Check all phase bytes and mask bits.
        for (j, byte) in raw[12..12 + DIM].iter_mut().enumerate() {
            *byte = j.wrapping_mul(137) as u8;
        }
        for j in 0..DIM {
            raw[12 + DIM..].fill(0);
            raw[12 + DIM + j / 8] = 1 << (j % 8);
            let bank = decode(&raw).unwrap();
            assert_eq!(bank[0].phase().as_slice(), &raw[12..12 + DIM]);
            for k in 0..DIM {
                assert_eq!(bank[0].active(k), Some(k == j));
            }
        }
    }
    #[test]
    fn malformed_cli_is_rejected_before_io() {
        for args in [
            vec![],
            vec!["missing"],
            vec!["missing", "0", "active"],
            vec!["missing", "100", "active"],
            vec!["missing", "-1", "active"],
            vec!["missing", "1", "unknown"],
            vec!["--generate", "missing", "0"],
            vec!["--generate", "missing", "8193"],
        ] {
            assert!(run(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
        }
        assert!(run(&[".".into(), "1".into(), "active".into()])
            .unwrap_err()
            .contains("regular file"));
    }
    #[test]
    fn fixture_bounds_and_trailing_bytes() {
        let raw = synthetic(3).unwrap();
        assert_eq!(decode(&raw).unwrap().len(), 3);
        for length in [0, 8, 11, 12, raw.len() - 1] {
            assert!(decode(&raw[..length]).is_err());
        }
        let mut extra = raw.clone();
        extra.push(0);
        assert!(decode(&extra).is_err());
        let mut bad = raw;
        bad[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&bad).is_err());
        assert!(synthetic(0).is_err());
        assert!(synthetic(MAX_RECORDS + 1).is_err());
    }
    #[test]
    fn active_mask_bit_order() {
        let mut raw = synthetic(1).unwrap();
        raw[12 + DIM..].fill(0);
        raw[12 + DIM] = 0x81;
        let bank = decode(&raw).unwrap();
        for j in 0..DIM {
            assert_eq!(bank[0].active(j), Some(j == 0 || j == 7));
        }
    }
    #[test]
    fn single_oracle_and_empty_masks() {
        let bank = decode(&synthetic(4).unwrap()).unwrap();
        let empty = Record::new([0; DIM], &[false; DIM]);
        let cos = std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos());
        assert!(canonical(&empty, &bank[0], true, &cos).is_none());
        assert!(benchmark(&[empty], 1, true).is_err());
        benchmark(&bank, 1, true).unwrap();
        benchmark(&bank, 1, false).unwrap();
    }
    #[test]
    fn generator_refuses_to_overwrite() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("gel-q8-fixture-{}-{stamp}", std::process::id()));
        let args = vec![
            "--generate".into(),
            path.to_str().unwrap().into(),
            "2".into(),
        ];
        run(&args).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(run(&args).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }
}
