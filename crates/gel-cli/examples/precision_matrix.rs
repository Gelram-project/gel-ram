#![forbid(unsafe_code)]
//! Public precision reference: F32, IEEE binary16 and affine min/max Q1–Q16.
//! Not the private GEL codec. The private Q2.5 mechanism is not implemented and
//! cannot be expressed: widths are whole bits 1–16. Timing is not measured here;
//! the historical timed campaign is `quantization_matrix`.
//!
//! Container GPMX v1, all integers and floats little-endian, 16-byte header:
//! `"GPMX"` | version u16 = 1 | kind u8 | bits u8 | count u32 | reserved u32 = 0.
//! kind 0 = F32 (bits 32): count × f32. kind 1 = F16 (bits 16): count × binary16.
//! kind 2 = affine (bits 1–16): count/32 blocks × (lo f32, hi f32, payload).
//! Affine payload: value i of a block occupies bits [i·bits, (i+1)·bits) of an
//! LSB-first bitstream; 32·bits bits are exactly 4·bits bytes, so padding is 0.
//! Rounding: code = round_ties_even((x − lo)/(hi − lo)·(2^bits − 1)) in f64,
//! code 0 when lo == hi; value = lo + (hi − lo)·code/(2^bits − 1) in f64, then
//! rounded to f32; a constant block (lo == hi) reconstructs lo exactly and
//! must carry only zero codes (canonical form).
//! F16: IEEE 754 round-to-nearest-even from f32; values rounding beyond ±65504
//! are rejected. Inputs must be finite. Limits: 1 ≤ count ≤ 2^24; affine count
//! is a multiple of 32. Sign of zero: exact in F32/F16 and in all-(−0) affine
//! blocks, not preserved in affine blocks mixing −0 and +0.
//! Error budget per value: affine (hi − lo)/(2·(2^bits − 1)) + max(|lo|,|hi|)·ε32;
//! F16 half an F16 ulp of the value's binade (2^-25 in the subnormal range).
use std::{fs, path::Path};

const MAGIC: &[u8; 4] = b"GPMX";
const VERSION: u16 = 1;
const HEADER: usize = 16;
const BLOCK: usize = 32;
const MAX_VALUES: usize = 1 << 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    F32,
    F16,
    Affine(u32),
}

impl Format {
    fn all() -> Vec<Self> {
        let mut formats = vec![Self::F32, Self::F16];
        formats.extend((1..=16).map(Self::Affine));
        formats
    }
    fn parse(kind: u8, bits: u8) -> Result<Self, &'static str> {
        match (kind, bits) {
            (0, 32) => Ok(Self::F32),
            (1, 16) => Ok(Self::F16),
            (2, 1..=16) => Ok(Self::Affine(u32::from(bits))),
            _ => Err("unsupported kind/bits"),
        }
    }
    fn header_fields(self) -> (u8, u8) {
        match self {
            Self::F32 => (0, 32),
            Self::F16 => (1, 16),
            Self::Affine(bits) => (2, bits as u8),
        }
    }
    fn label(self) -> String {
        match self {
            Self::F32 => "F32".into(),
            Self::F16 => "F16".into(),
            Self::Affine(bits) => format!("Q{bits}"),
        }
    }
    fn metadata_len(self, count: usize) -> usize {
        match self {
            Self::Affine(_) => count / BLOCK * 8,
            _ => 0,
        }
    }
    fn body_len(self, count: usize) -> usize {
        match self {
            Self::F32 => count * 4,
            Self::F16 => count * 2,
            Self::Affine(bits) => count / BLOCK * (8 + 4 * bits as usize),
        }
    }
}

/// IEEE 754 binary16, round-to-nearest-even; overflow is an error.
fn f16_from_f32(x: f32) -> Result<u16, &'static str> {
    if !x.is_finite() {
        return Err("non-finite input");
    }
    let bits = x.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127;
    let mantissa = bits & 0x7f_ffff;
    if bits & 0x7fff_ffff == 0 || exponent < -26 {
        // Zero, f32 subnormals and anything below half the smallest F16 subnormal.
        return Ok(sign);
    }
    if exponent > 15 {
        return Err("F16 overflow");
    }
    let code = if exponent >= -14 {
        let mut code = (((exponent + 15) as u32) << 10) | (mantissa >> 13);
        let rest = mantissa & 0x1fff;
        if rest > 0x1000 || (rest == 0x1000 && code & 1 == 1) {
            code += 1;
        }
        code
    } else {
        // Subnormal F16: units of 2^-24; full significand shifted right.
        let significand = mantissa | 0x80_0000;
        let shift = (-(exponent + 1)) as u32;
        let mut code = significand >> shift;
        let rest = significand & ((1 << shift) - 1);
        let half = 1 << (shift - 1);
        if rest > half || (rest == half && code & 1 == 1) {
            code += 1;
        }
        code
    };
    if code >= 0x7c00 {
        return Err("F16 overflow");
    }
    Ok(sign | code as u16)
}

fn f32_from_f16(code: u16) -> Result<f32, &'static str> {
    let sign = if code & 0x8000 == 0 { 1.0f32 } else { -1.0 };
    let exponent = i32::from((code >> 10) & 0x1f);
    let mantissa = f32::from(code & 0x3ff);
    match exponent {
        0x1f => Err("non-finite F16"),
        0 => Ok(sign * mantissa * 2f32.powi(-24)),
        e => Ok(sign * (1.0 + mantissa / 1024.0) * 2f32.powi(e - 15)),
    }
}

fn affine_code(v: f32, lo: f32, hi: f32, levels: u32) -> u32 {
    if lo == hi {
        0
    } else {
        (((v as f64 - lo as f64) / (hi as f64 - lo as f64) * levels as f64).round_ties_even()
            as u32)
            .min(levels)
    }
}

fn affine_value(code: u32, lo: f32, hi: f32, levels: u32) -> f32 {
    (lo as f64 + (hi as f64 - lo as f64) * code as f64 / levels as f64) as f32
}

fn encode(x: &[f32], format: Format) -> Result<Vec<u8>, &'static str> {
    if x.is_empty() || x.len() > MAX_VALUES {
        return Err("value count outside 1..=2^24");
    }
    if x.iter().any(|v| !v.is_finite()) {
        return Err("non-finite input");
    }
    let (kind, bits) = format.header_fields();
    let mut out = Vec::with_capacity(HEADER + format.body_len(x.len()));
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&[kind, bits]);
    out.extend_from_slice(&(x.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    match format {
        Format::F32 => x
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes())),
        Format::F16 => {
            for &v in x {
                out.extend_from_slice(&f16_from_f32(v)?.to_le_bytes());
            }
        }
        Format::Affine(bits) => {
            if x.len() % BLOCK != 0 {
                return Err("affine requires whole 32-value blocks");
            }
            let levels = (1u32 << bits) - 1;
            for block in x.chunks_exact(BLOCK) {
                let lo = block.iter().copied().fold(f32::INFINITY, f32::min);
                let hi = block.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                out.extend_from_slice(&lo.to_le_bytes());
                out.extend_from_slice(&hi.to_le_bytes());
                let (mut acc, mut filled) = (0u64, 0u32);
                for &v in block {
                    acc |= u64::from(affine_code(v, lo, hi, levels)) << filled;
                    filled += bits;
                    while filled >= 8 {
                        out.push(acc as u8);
                        acc >>= 8;
                        filled -= 8;
                    }
                }
            }
        }
    }
    Ok(out)
}

fn decode(bytes: &[u8]) -> Result<(Format, Vec<f32>), &'static str> {
    let header = bytes.get(..HEADER).ok_or("truncated header")?;
    let field = |range: std::ops::Range<usize>| &header[range];
    if field(0..4) != MAGIC {
        return Err("bad magic");
    }
    if u16::from_le_bytes([header[4], header[5]]) != VERSION {
        return Err("unsupported version");
    }
    let format = Format::parse(header[6], header[7])?;
    let count = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;
    if u32::from_le_bytes([header[12], header[13], header[14], header[15]]) != 0 {
        return Err("reserved field not zero");
    }
    if count == 0 || count > MAX_VALUES {
        return Err("value count outside 1..=2^24");
    }
    if matches!(format, Format::Affine(_)) && count % BLOCK != 0 {
        return Err("affine requires whole 32-value blocks");
    }
    let body = &bytes[HEADER..];
    if body.len() != format.body_len(count) {
        return Err("body length does not match header");
    }
    let mut x = Vec::with_capacity(count);
    match format {
        Format::F32 => {
            for chunk in body.chunks_exact(4) {
                let v = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                if !v.is_finite() {
                    return Err("non-finite F32");
                }
                x.push(v);
            }
        }
        Format::F16 => {
            for chunk in body.chunks_exact(2) {
                x.push(f32_from_f16(u16::from_le_bytes([chunk[0], chunk[1]]))?);
            }
        }
        Format::Affine(bits) => {
            let levels = (1u32 << bits) - 1;
            for block in body.chunks_exact(8 + 4 * bits as usize) {
                let lo = f32::from_le_bytes([block[0], block[1], block[2], block[3]]);
                let hi = f32::from_le_bytes([block[4], block[5], block[6], block[7]]);
                if !lo.is_finite() || !hi.is_finite() || lo > hi {
                    return Err("bad block range");
                }
                let mut payload = block[8..].iter();
                let (mut acc, mut available) = (0u64, 0u32);
                for _ in 0..BLOCK {
                    while available < bits {
                        acc |= u64::from(*payload.next().ok_or("short payload")?) << available;
                        available += 8;
                    }
                    let code = (acc & u64::from(levels)) as u32;
                    acc >>= bits;
                    available -= bits;
                    if lo == hi && code != 0 {
                        return Err("non-canonical constant block");
                    }
                    // A constant block reconstructs lo exactly (keeps the sign of -0).
                    let v = if lo == hi {
                        lo
                    } else {
                        affine_value(code, lo, hi, levels)
                    };
                    if !v.is_finite() {
                        return Err("non-finite reconstruction");
                    }
                    x.push(v);
                }
            }
        }
    }
    Ok((format, x))
}

fn datasets() -> Vec<(&'static str, Vec<f32>)> {
    // Same synthetic inputs as the historical quantization_matrix campaign.
    let n = 32768;
    vec![
        ("silence", vec![0.; n]),
        ("constant", vec![0.25; n]),
        (
            "image_gradient",
            (0..n).map(|i| (i % 256) as f32 / 255.).collect(),
        ),
        (
            "audio_tones",
            (0..n)
                .map(|i| {
                    let t = i as f64 / 48000.;
                    (0.6 * (std::f64::consts::TAU * 440. * t).sin()
                        + 0.02 * (std::f64::consts::TAU * 7000. * t).sin())
                        as f32
                })
                .collect(),
        ),
        (
            "small_signal_outliers",
            (0..n)
                .map(|i| {
                    if i % BLOCK == 0 {
                        1000.
                    } else {
                        ((i % 31) as f32 - 15.) * 0.0001
                    }
                })
                .collect(),
        ),
        (
            "alternating_extremes",
            (0..n).map(|i| if i % 2 == 0 { -1. } else { 1. }).collect(),
        ),
    ]
}

struct Row {
    rmse: f64,
    max: f64,
    exact: usize,
}

fn measure(x: &[f32], decoded: &[f32]) -> Row {
    let mut error2 = 0.;
    let mut max = 0f64;
    let mut exact = 0;
    for (&a, &b) in x.iter().zip(decoded) {
        let e = (a as f64 - b as f64).abs();
        error2 += e * e;
        max = max.max(e);
        exact += usize::from(a.to_bits() == b.to_bits());
    }
    Row {
        rmse: (error2 / x.len() as f64).sqrt(),
        max,
        exact,
    }
}

/// Write, sync, read back and decode; the file bytes must equal memory bytes.
fn file_roundtrip(dir: &Path, name: &str, bytes: &[u8]) -> Result<Vec<f32>, String> {
    use std::io::Write;
    let path = dir.join(name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("{name}: {e}"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{name}: {e}"))?;
    let read = fs::read(&path).map_err(|e| format!("{name}: {e}"))?;
    if read != bytes {
        return Err(format!("{name}: file bytes differ"));
    }
    Ok(decode(&read).map_err(|e| format!("{name}: {e}"))?.1)
}

fn main() -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!(
        "gel-precision-matrix-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ));
    fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    println!("REFERENCE=GPMX_v1 formats=F32,F16,affine_minmax32_Q1..Q16 header_bytes={HEADER}\nSCOPE=public reference precisions on synthetic samples; not semantic accuracy or the private GEL codec; private Q2.5 NOT_IMPLEMENTED\nTIMING=NOT_MEASURED (historical timed campaign: quantization_matrix)");
    println!("dataset,format,values,payload_bytes,metadata_bytes,body_bytes,container_bytes,rmse,max_abs_error,bit_exact,file_roundtrip");
    let mut rows = 0;
    for (name, x) in datasets() {
        for format in Format::all() {
            let bytes = encode(&x, format).map_err(|e| format!("{name} {format:?}: {e}"))?;
            let (_, decoded) = decode(&bytes).map_err(|e| format!("{name} {format:?}: {e}"))?;
            let from_file =
                file_roundtrip(&dir, &format!("{name}-{}.gpmx", format.label()), &bytes)?;
            if from_file
                .iter()
                .map(|v| v.to_bits())
                .ne(decoded.iter().map(|v| v.to_bits()))
            {
                return Err(format!("{name} {format:?}: file decode differs"));
            }
            let row = measure(&x, &decoded);
            let body = format.body_len(x.len());
            let metadata = format.metadata_len(x.len());
            println!(
                "{name},{},{},{},{metadata},{body},{},{:.9e},{:.9e},{},PASS",
                format.label(),
                x.len(),
                body - metadata,
                bytes.len(),
                row.rmse,
                row.max,
                row.exact
            );
            rows += 1;
        }
    }
    fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    println!("PRECISION_MATRIX=PASS formats=18 datasets=6 rows={rows} file_roundtrips={rows}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const F16_MAX: f32 = 65504.0;

    /// Deterministic 64-bit LCG; test inputs only.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
    }

    /// Adjacent f32 values for positive finite inputs (Rust 1.85: no next_up).
    fn up(x: f32) -> f32 {
        f32::from_bits(x.to_bits() + 1)
    }
    fn down(x: f32) -> f32 {
        f32::from_bits(x.to_bits() - 1)
    }

    fn body(bytes: &[u8]) -> &[u8] {
        &bytes[HEADER..]
    }

    /// Independent packer: one bool per bit, LSB-first, then bytes bit by bit.
    fn oracle_pack(codes: &[u32], bits: u32) -> Vec<u8> {
        let stream: Vec<bool> = codes
            .iter()
            .flat_map(|&c| (0..bits).map(move |b| (c >> b) & 1 == 1))
            .collect();
        stream
            .chunks(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .fold(0u8, |acc, (i, &bit)| acc | (u8::from(bit) << i))
            })
            .collect()
    }

    #[test]
    fn codes_known_by_construction_every_width() {
        let mut rng = Lcg(7);
        for bits in 1..=16u32 {
            let levels = (1u32 << bits) - 1;
            let mut x = Vec::new();
            let mut expected = Vec::new();
            for block in 0..8 {
                let mut codes: Vec<u32> = (0..BLOCK)
                    .map(|_| (rng.next() % (u64::from(levels) + 1)) as u32)
                    .collect();
                // Endpoints make lo = 0 and hi = levels, so value == code.
                codes[block % BLOCK] = 0;
                codes[(block + 5) % BLOCK] = levels;
                x.extend(codes.iter().map(|&c| c as f32));
                expected.push(codes);
            }
            let bytes = encode(&x, Format::Affine(bits)).unwrap();
            assert_eq!(bytes.len(), HEADER + 8 * (8 + 4 * bits as usize));
            for (block, codes) in body(&bytes)
                .chunks_exact(8 + 4 * bits as usize)
                .zip(&expected)
            {
                assert_eq!(&block[..4], &0f32.to_le_bytes());
                assert_eq!(&block[4..8], &(levels as f32).to_le_bytes());
                assert_eq!(&block[8..], oracle_pack(codes, bits).as_slice(), "Q{bits}");
            }
            let (format, decoded) = decode(&bytes).unwrap();
            assert_eq!(format, Format::Affine(bits));
            assert!(decoded
                .iter()
                .zip(&x)
                .all(|(a, b)| a.to_bits() == b.to_bits()));
        }
    }

    #[test]
    fn halfway_values_round_to_even_code_every_width() {
        for bits in 1..=16u32 {
            let levels = (1u32 << bits) - 1;
            let mut x = vec![0f32; BLOCK];
            x[1] = levels as f32;
            let mut expected = vec![0u32; BLOCK];
            expected[1] = levels;
            for (slot, k) in (2..BLOCK).zip((0..levels).cycle()) {
                x[slot] = k as f32 + 0.5;
                // The tie must really occur in the encoder's f64 arithmetic.
                let t = (x[slot] as f64 - 0.0) / (levels as f64 - 0.0) * levels as f64;
                assert_eq!(t, k as f64 + 0.5, "Q{bits} k={k}: no exact tie");
                expected[slot] = if k % 2 == 0 { k } else { k + 1 };
            }
            let bytes = encode(&x, Format::Affine(bits)).unwrap();
            assert_eq!(&body(&bytes)[8..], oracle_pack(&expected, bits).as_slice());
        }
    }

    #[test]
    fn chosen_code_is_a_nearest_level() {
        let mut rng = Lcg(11);
        for bits in 1..=16u32 {
            let levels = (1u32 << bits) - 1;
            for scale in [1e-6f32, 1.0, 1e6] {
                let x: Vec<f32> = (0..BLOCK * 4)
                    .map(|_| (rng.next() % 2_000_001) as f32 / 1e6 * scale - scale)
                    .collect();
                let bytes = encode(&x, Format::Affine(bits)).unwrap();
                let (_, decoded) = decode(&bytes).unwrap();
                for (block, out) in x.chunks_exact(BLOCK).zip(decoded.chunks_exact(BLOCK)) {
                    let lo = block.iter().copied().fold(f32::INFINITY, f32::min);
                    let hi = block.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    let slack = hi.abs().max(lo.abs()) as f64 * f32::EPSILON as f64;
                    for (&v, &w) in block.iter().zip(out) {
                        let chosen = (v as f64 - w as f64).abs();
                        let code = affine_code(v, lo, hi, levels);
                        for other in [code.wrapping_sub(1), code + 1] {
                            if other <= levels {
                                let alt = affine_value(other, lo, hi, levels) as f64;
                                assert!(chosen <= (v as f64 - alt).abs() + slack, "Q{bits}");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn affine_error_budget_all_widths_all_datasets() {
        for (name, x) in datasets() {
            for bits in 1..=16u32 {
                let levels = ((1u32 << bits) - 1) as f64;
                let (_, d) = decode(&encode(&x, Format::Affine(bits)).unwrap()).unwrap();
                for (a, b) in x.chunks_exact(BLOCK).zip(d.chunks_exact(BLOCK)) {
                    let lo = a.iter().copied().fold(f32::INFINITY, f32::min) as f64;
                    let hi = a.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
                    let bound =
                        (hi - lo) / levels / 2. + hi.abs().max(lo.abs()) * f32::EPSILON as f64;
                    for (&v, &w) in a.iter().zip(b) {
                        assert!((v as f64 - w as f64).abs() <= bound, "{name} Q{bits}");
                    }
                }
            }
        }
    }

    #[test]
    fn f16_exhaustive_roundtrip_ties_and_overflow() {
        for code in (0u16..0x7c00).chain(0x8000..0xfc00) {
            let value = f32_from_f16(code).unwrap();
            assert_eq!(f16_from_f32(value).unwrap(), code, "{code:#06x}");
        }
        for code in 0u16..0x7bff {
            let (a, b) = (f32_from_f16(code).unwrap(), f32_from_f16(code + 1).unwrap());
            let mid = ((a as f64 + b as f64) / 2.) as f32;
            assert_eq!(
                mid as f64,
                (a as f64 + b as f64) / 2.,
                "midpoint representable"
            );
            let even = if code % 2 == 0 { code } else { code + 1 };
            assert_eq!(f16_from_f32(mid).unwrap(), even, "{code:#06x}");
            assert_eq!(f16_from_f32(down(mid)).unwrap(), code);
            assert_eq!(f16_from_f32(up(mid)).unwrap(), code + 1);
        }
        assert_eq!(f16_from_f32(65519.996).unwrap(), 0x7bff);
        assert!(f16_from_f32(65520.0).is_err());
        assert!(f16_from_f32(-65520.0).is_err());
        assert_eq!(f16_from_f32(2f32.powi(-25)).unwrap(), 0); // tie to even zero
        assert_eq!(f16_from_f32(up(2f32.powi(-25))).unwrap(), 1);
        assert_eq!(f16_from_f32(-0.0).unwrap(), 0x8000);
        assert_eq!(f16_from_f32(f32::from_bits(1)).unwrap(), 0);
        assert!(f32_from_f16(0x7c00).is_err());
        assert_eq!(f32_from_f16(0x7bff).unwrap(), F16_MAX);
        assert_eq!(f32_from_f16(0xfbff).unwrap(), -F16_MAX);
        assert!(f32_from_f16(0x7e00).is_err());
    }

    #[test]
    fn f16_matches_table_search_oracle() {
        // Independent method: nearest value in the sorted table, ties to even code.
        let table: Vec<f32> = (0u16..0x7c00).map(|c| f32_from_f16(c).unwrap()).collect();
        let mut rng = Lcg(3);
        for _ in 0..20000 {
            let magnitude = f32::from_bits((rng.next() as u32) % 0x477f_e000);
            for v in [magnitude, -magnitude] {
                let target = v.abs();
                let upper = table.partition_point(|&t| t < target).min(table.len() - 1);
                let lower = upper.saturating_sub(1);
                let (dl, du) = (target - table[lower], table[upper] - target);
                let code = if du < dl || (du == dl && upper % 2 == 0) {
                    upper
                } else {
                    lower
                } as u16;
                let sign = if v.is_sign_negative() { 0x8000 } else { 0 };
                assert_eq!(f16_from_f32(v).unwrap(), sign | code, "{v:e}");
            }
        }
    }

    #[test]
    fn container_save_load_every_format() {
        let dir = std::env::temp_dir().join(format!(
            "gel-precision-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let x: Vec<f32> = (0..BLOCK * 3).map(|i| (i as f32 - 40.) / 7.).collect();
        for format in Format::all() {
            let bytes = encode(&x, format).unwrap();
            assert_eq!(bytes.len(), HEADER + format.body_len(x.len()));
            let memory = decode(&bytes).unwrap().1;
            let file = file_roundtrip(&dir, &format.label(), &bytes).unwrap();
            assert!(memory
                .iter()
                .zip(&file)
                .all(|(a, b)| a.to_bits() == b.to_bits()));
            assert!(
                file_roundtrip(&dir, &format.label(), &bytes).is_err(),
                "create_new"
            );
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn malformed_containers_and_inputs_rejected() {
        let x: Vec<f32> = (0..BLOCK).map(|i| i as f32).collect();
        let good = encode(&x, Format::Affine(3)).unwrap();
        let mutate = |at: usize, value: u8| {
            let mut b = good.clone();
            b[at] = value;
            decode(&b)
        };
        assert_eq!(mutate(0, b'X').unwrap_err(), "bad magic");
        assert_eq!(mutate(4, 2).unwrap_err(), "unsupported version");
        assert_eq!(mutate(7, 0).unwrap_err(), "unsupported kind/bits");
        assert_eq!(mutate(7, 17).unwrap_err(), "unsupported kind/bits");
        assert_eq!(mutate(6, 3).unwrap_err(), "unsupported kind/bits");
        assert_eq!(mutate(12, 1).unwrap_err(), "reserved field not zero");
        assert_eq!(
            mutate(8, 31).unwrap_err(),
            "affine requires whole 32-value blocks"
        );
        assert_eq!(decode(&good[..HEADER - 1]).unwrap_err(), "truncated header");
        assert_eq!(
            decode(&good[..good.len() - 1]).unwrap_err(),
            "body length does not match header"
        );
        let mut long = good.clone();
        long.push(0);
        assert_eq!(
            decode(&long).unwrap_err(),
            "body length does not match header"
        );
        let mut zero = good.clone();
        zero[8..12].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(decode(&zero).unwrap_err(), "value count outside 1..=2^24");
        let mut range = good.clone();
        range[HEADER..HEADER + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(decode(&range).unwrap_err(), "bad block range");
        let mut inverted = good.clone();
        inverted[HEADER..HEADER + 4].copy_from_slice(&1e9f32.to_le_bytes());
        assert_eq!(decode(&inverted).unwrap_err(), "bad block range");
        let mut constant = encode(&[5.0; BLOCK], Format::Affine(4)).unwrap();
        constant[HEADER + 8] = 1;
        assert_eq!(
            decode(&constant).unwrap_err(),
            "non-canonical constant block"
        );
        let mut half = encode(&[1.0; 2], Format::F16).unwrap();
        half[HEADER..HEADER + 2].copy_from_slice(&0x7c00u16.to_le_bytes());
        assert_eq!(decode(&half).unwrap_err(), "non-finite F16");
        let mut single = encode(&[1.0; 2], Format::F32).unwrap();
        single[HEADER..HEADER + 4].copy_from_slice(&f32::INFINITY.to_le_bytes());
        assert_eq!(decode(&single).unwrap_err(), "non-finite F32");
        for format in Format::all() {
            assert!(encode(&[], format).is_err());
            assert!(encode(&[f32::NAN; BLOCK], format).is_err());
            assert!(encode(&[f32::INFINITY; BLOCK], format).is_err());
        }
        assert!(encode(&[1.0; 31], Format::Affine(8)).is_err());
        assert!(encode(&[1e5; 2], Format::F16).is_err());
        assert!(Format::parse(2, 0).is_err() && Format::parse(1, 32).is_err());
    }

    #[test]
    fn sign_of_zero_boundaries() {
        let negative = encode(&[-0.0; BLOCK], Format::Affine(8)).unwrap();
        assert!(decode(&negative)
            .unwrap()
            .1
            .iter()
            .all(|v| v.to_bits() == (-0f32).to_bits()));
        let f16 = encode(&[-0.0, 0.0], Format::F16).unwrap();
        let d = decode(&f16).unwrap().1;
        assert_eq!((d[0].to_bits(), d[1].to_bits()), ((-0f32).to_bits(), 0));
    }

    #[test]
    fn historical_q1_q2_q4_q8_rows_reproduced_exactly() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/evidence-collection/quantization-matrix.txt");
        let text = fs::read_to_string(path).unwrap();
        let data = datasets();
        let mut compared = 0;
        for line in text.lines() {
            let f: Vec<&str> = line.split(',').collect();
            let Some(bits) = f.get(1).and_then(|q| q.strip_prefix('Q')) else {
                continue;
            };
            let bits: u32 = bits.parse().unwrap();
            let x = &data.iter().find(|(name, _)| *name == f[0]).unwrap().1;
            let bytes = encode(x, Format::Affine(bits)).unwrap();
            let row = measure(x, &decode(&bytes).unwrap().1);
            let body = Format::Affine(bits).body_len(x.len());
            assert_eq!(f[5], body.to_string(), "{line}");
            assert_eq!(f[6], format!("{:.9e}", row.rmse), "{line}");
            assert_eq!(f[7], format!("{:.9e}", row.max), "{line}");
            assert_eq!(f[8], row.exact.to_string(), "{line}");
            compared += 1;
        }
        assert_eq!(compared, 24);
    }
}
