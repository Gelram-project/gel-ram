//! Transparent reference experiment, not the private GEL encoder.
//! Q1/Q2/Q4/Q8: affine min/max per 32 samples, packed LSB first, 8 metadata bytes.
//! Finite input only. No claim that quantization removes only noise.
use std::{hint::black_box, time::Instant};
const BLOCK: usize = 32;

fn encode(x: &[f32], bits: usize) -> Result<Vec<u8>, &'static str> {
    if ![1, 2, 4, 8].contains(&bits)
        || x.is_empty()
        || x.len() % BLOCK != 0
        || x.iter().any(|v| !v.is_finite())
    {
        return Err("finite whole blocks and Q1/Q2/Q4/Q8 required");
    }
    let levels = (1usize << bits) - 1;
    let mut output = Vec::new();
    for block in x.chunks_exact(BLOCK) {
        let lo = block.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = block.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        output.extend_from_slice(&lo.to_le_bytes());
        output.extend_from_slice(&hi.to_le_bytes());
        let base = output.len();
        output.resize(base + BLOCK * bits / 8, 0);
        for (i, &v) in block.iter().enumerate() {
            let q = if lo == hi {
                0
            } else {
                (((v as f64 - lo as f64) / (hi as f64 - lo as f64) * levels as f64)
                    .round_ties_even() as usize)
                    .min(levels)
            };
            output[base + i * bits / 8] |= (q << (i * bits % 8)) as u8;
        }
    }
    Ok(output)
}
fn decode(bytes: &[u8], bits: usize) -> Result<Vec<f32>, &'static str> {
    if ![1, 2, 4, 8].contains(&bits) {
        return Err("bad bits");
    }
    let size = 8 + BLOCK * bits / 8;
    let levels = (1usize << bits) - 1;
    if bytes.is_empty() || bytes.len() % size != 0 {
        return Err("bad length");
    }
    let mut x = Vec::new();
    for b in bytes.chunks_exact(size) {
        let lo = f32::from_le_bytes(b[..4].try_into().unwrap());
        let hi = f32::from_le_bytes(b[4..8].try_into().unwrap());
        if !lo.is_finite() || !hi.is_finite() || lo > hi {
            return Err("bad range");
        }
        for i in 0..BLOCK {
            let q = ((b[8 + i * bits / 8] as usize) >> (i * bits % 8)) & levels;
            let v = (lo as f64 + (hi as f64 - lo as f64) * q as f64 / levels as f64) as f32;
            if !v.is_finite() {
                return Err("nonfinite reconstruction");
            }
            x.push(v);
        }
    }
    Ok(x)
}
fn percentile(values: &mut [u128], p: usize) -> u128 {
    values.sort_unstable();
    values[(values.len() * p).div_ceil(100).saturating_sub(1)]
}
fn datasets() -> Vec<(&'static str, Vec<f32>)> {
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
fn main() {
    println!("REFERENCE=affine_minmax32 LSB_packed metadata_per_block=8 rounds=21 warmup=1\nSCOPE=synthetic decoded sample quality; not semantic accuracy, media container compression or private GEL codec\nTIMING=encode_and_decode_with_allocations warm_buffers not cold_DRAM; no ORB/s claim");
    println!("dataset,q,values,payload_bytes,metadata_bytes,total_bytes,rmse,max_abs_error,bit_exact,median_ns,p95_ns");
    for (name, x) in datasets() {
        println!(
            "{name},F32,{}, {},0,{},0,0,{},NA,NA",
            x.len(),
            x.len() * 4,
            x.len() * 4,
            x.len()
        );
        for bits in [1, 2, 4, 8] {
            let encoded = encode(&x, bits).unwrap();
            let decoded = decode(&encoded, bits).unwrap();
            let mut error2 = 0.;
            let mut max = 0f64;
            let mut exact = 0;
            for (&a, &b) in x.iter().zip(&decoded) {
                let e = (a as f64 - b as f64).abs();
                error2 += e * e;
                max = max.max(e);
                exact += usize::from(a.to_bits() == b.to_bits());
            }
            let mut times = Vec::new();
            for _ in 0..21 {
                let t = Instant::now();
                let b = encode(black_box(&x), bits).unwrap();
                black_box(decode(black_box(&b), bits).unwrap());
                times.push(t.elapsed().as_nanos());
            }
            // Keep every raw timing, including slower samples, outside the CSV rows.
            println!("RAW_NS dataset={name} q=Q{bits} values={times:?}");
            let median = percentile(&mut times, 50);
            let p95 = percentile(&mut times, 95);
            println!(
                "{name},Q{bits},{},{},{},{},{:.9e},{:.9e},{},{median},{p95}",
                x.len(),
                x.len() * bits / 8,
                x.len() / BLOCK * 8,
                encoded.len(),
                (error2 / x.len() as f64).sqrt(),
                max,
                exact
            );
        }
    }
    println!("QUANTIZATION_MATRIX=PASS F16=SEE_EXISTING_DATA_INTEGRITY_REFERENCE");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_endpoints_and_constant_all_q() {
        for bits in [1, 2, 4, 8] {
            for x in [
                vec![0.25; 32],
                (0..32).map(|i| if i % 2 == 0 { -1. } else { 1. }).collect(),
            ] {
                let e = encode(&x, bits).unwrap();
                assert_eq!(e.len(), 8 + 4 * bits);
                assert_eq!(decode(&e, bits).unwrap(), x);
            }
        }
    }
    #[test]
    fn packed_codes_are_independently_known() {
        let x: Vec<_> = (0..32).map(|i| (i % 4) as f32).collect();
        let b = encode(&x, 2).unwrap();
        assert_eq!(&b[8..], &[0b11100100; 8]);
        assert_eq!(decode(&b, 2).unwrap(), x);
    }
    #[test]
    fn halfway_ties_are_even() {
        let mut x = vec![0.; 32];
        x[0] = 3.;
        x[1] = 0.5;
        x[2] = 1.5;
        x[3] = 2.5;
        let d = decode(&encode(&x, 2).unwrap(), 2).unwrap();
        assert_eq!(&d[..4], &[3., 0., 2., 2.]);
    }
    #[test]
    fn invalid_inputs_and_ranges_rejected() {
        assert!(encode(&[0.; 31], 8).is_err());
        assert!(encode(&[f32::NAN; 32], 8).is_err());
        assert!(encode(&[0.; 32], 3).is_err());
        let b = encode(&[0.; 32], 8).unwrap();
        assert!(decode(&b[..39], 8).is_err());
        assert!(decode(&b, 0).is_err());
        let mut b = b;
        b[..4].copy_from_slice(&f32::INFINITY.to_le_bytes());
        assert!(decode(&b, 8).is_err());
    }
    #[test]
    fn reconstruction_bound_all_datasets() {
        for (_, x) in datasets() {
            for bits in [1, 2, 4, 8] {
                let d = decode(&encode(&x, bits).unwrap(), bits).unwrap();
                for (a, b) in x.chunks_exact(32).zip(d.chunks_exact(32)) {
                    let lo = a.iter().copied().fold(f32::INFINITY, f32::min) as f64;
                    let hi = a.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
                    let bound = (hi - lo) / ((1usize << bits) - 1) as f64 / 2.
                        + hi.abs().max(lo.abs()) * f32::EPSILON as f64;
                    for (&v, &w) in a.iter().zip(b) {
                        assert!((v as f64 - w as f64).abs() <= bound);
                    }
                }
            }
        }
    }
    #[test]
    fn weak_signal_is_lost_not_just_noise() {
        let mut x = vec![0.001; 32];
        x[0] = 0.;
        x[1] = 1000.;
        let d = decode(&encode(&x, 8).unwrap(), 8).unwrap();
        assert_eq!(d[2], 0.);
        assert_ne!(d[2], x[2]);
    }
    #[test]
    fn extremes_are_finite() {
        let mut x = vec![0.; 32];
        x[0] = -f32::MAX;
        x[1] = f32::MAX;
        for q in [1, 2, 4, 8] {
            assert!(decode(&encode(&x, q).unwrap(), q)
                .unwrap()
                .iter()
                .all(|v| v.is_finite()));
        }
    }
}
