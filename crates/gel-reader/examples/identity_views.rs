//! Identity check for the word-level `AffinePermutation` and `AffineMasked`
//! paths of `apply_view` and `invert_view` against an exact copy of the
//! earlier per-bit algorithm (one mask word derivation and one branchy bit
//! store per position), on seeded ORBs, raw spec fields and edge cases,
//! compared bit for bit.
//!
//! Usage: `cargo run --release -p gel-reader --example identity_views`.
//! Prints `IDENTITY name=<case> cases=<n> equal=<n>` per case and one final
//! `IDENTITY_VERDICT=PASS` or `IDENTITY_VERDICT=FAIL` (FAIL also when no case
//! ran); exit status 0 on PASS, 1 on FAIL. The integration test
//! `tests/identity_views.rs` runs the same comparison. Output grammar:
//! `docs/SPEED.md`.
#![forbid(unsafe_code)]
use gel_core::{splitmix64, GelError, ORB_BITS, ORB_WORDS};
use gel_orb::Orb1024;
use gel_reader::{apply_view, invert_view, ViewKind, ViewSpec};
use std::process::ExitCode;

#[path = "../../../tools/identity_support.rs"]
pub mod identity;
use identity::{Scale, Tally};

// ---- Exact copy of the earlier algorithm (reference) ----

fn validate_odd(a: u16) -> Result<(), GelError> {
    if a == 0 || (a & 1) == 0 {
        Err(GelError::InvalidView(
            "affine multiplier must be odd modulo 1024",
        ))
    } else {
        Ok(())
    }
}

fn get(words: &[u64; ORB_WORDS], bit: usize) -> bool {
    ((words[bit >> 6] >> (bit & 63)) & 1) != 0
}

fn set(words: &mut [u64; ORB_WORDS], bit: usize, value: bool) {
    let mask = 1u64 << (bit & 63);
    if value {
        words[bit >> 6] |= mask;
    } else {
        words[bit >> 6] &= !mask;
    }
}

fn mask_word(seed: u64, word: usize) -> u64 {
    splitmix64(seed ^ (word as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93))
}

fn reference_apply(orb: &Orb1024, spec: ViewSpec) -> Result<Orb1024, GelError> {
    validate_odd(spec.a)?;
    let mut out = [0u64; ORB_WORDS];
    for dst in 0..ORB_BITS {
        let src = (spec.a as usize * dst + spec.b as usize) & 1023;
        let mut value = get(orb.words(), src);
        if matches!(spec.kind, ViewKind::AffineMasked) {
            value ^= ((mask_word(spec.seed, dst >> 6) >> (dst & 63)) & 1) != 0;
        }
        set(&mut out, dst, value);
    }
    Ok(Orb1024::from_words(out))
}

fn reference_invert(view: &Orb1024, spec: ViewSpec) -> Result<Orb1024, GelError> {
    validate_odd(spec.a)?;
    let mut base = [0u64; ORB_WORDS];
    for dst in 0..ORB_BITS {
        let mut value = get(view.words(), dst);
        if matches!(spec.kind, ViewKind::AffineMasked) {
            value ^= ((mask_word(spec.seed, dst >> 6) >> (dst & 63)) & 1) != 0;
        }
        let src = (spec.a as usize * dst + spec.b as usize) & 1023;
        set(&mut base, src, value);
    }
    Ok(Orb1024::from_words(base))
}

// ---- Comparison ----

/// Counts one case: one comparison per item of `results`.
fn case(tallies: &mut Vec<Tally>, name: &str, results: impl IntoIterator<Item = bool>) {
    let mut tally = Tally::new(name);
    tally.check_all(results);
    tallies.push(tally);
}

fn next(state: &mut u64) -> u64 {
    *state = splitmix64(*state);
    *state
}

fn orbs(state: &mut u64) -> Vec<Orb1024> {
    let mut out = vec![
        Orb1024::from_words([0; ORB_WORDS]),
        Orb1024::from_words([u64::MAX; ORB_WORDS]),
        Orb1024::from_words([0x5555_5555_5555_5555; ORB_WORDS]),
        Orb1024::from_words(std::array::from_fn(
            |w| if w % 2 == 0 { u64::MAX } else { 0 },
        )),
    ];
    for bit in [0usize, 1, 63, 64, 511, 512, 1022, 1023] {
        let mut words = [0u64; ORB_WORDS];
        words[bit >> 6] = 1u64 << (bit & 63);
        out.push(Orb1024::from_words(words));
        out.push(Orb1024::from_words(words.map(|w| !w)));
    }
    while out.len() < 64 {
        out.push(Orb1024::from_words(std::array::from_fn(|_| next(state))));
    }
    out
}

/// Raw spec fields, bypassing the constructors as public fields allow.
fn specs(state: &mut u64, kind: ViewKind) -> Vec<ViewSpec> {
    let mut a_values = vec![1u16, 3, 5, 511, 513, 1023, 1025, 2047, 0xffff];
    for _ in 0..5 {
        a_values.push((next(state) as u16) | 1);
    }
    let mut b_values = vec![0u16, 1, 37, 511, 1023, 1024, 1025, 0xffff];
    b_values.push(next(state) as u16);
    let seeds: Vec<u64> = if matches!(kind, ViewKind::AffineMasked) {
        vec![0, u64::MAX, 0x4745_4c02, next(state)]
    } else {
        vec![0, next(state)]
    };
    let mut out = Vec::new();
    for &a in &a_values {
        for &b in &b_values {
            for &seed in &seeds {
                out.push(ViewSpec { kind, a, b, seed });
            }
        }
    }
    out
}

/// Runs every comparison and returns one tally per case. There is no smaller
/// scale: `scale` is accepted for the common interface only.
pub fn run(_scale: Scale) -> Vec<Tally> {
    let mut tallies = Vec::new();
    let mut state = 0x6765_6c5f_7669_6577u64;
    let inputs = orbs(&mut state);
    for (kind, label) in [
        (ViewKind::AffinePermutation, "affine_perm"),
        (ViewKind::AffineMasked, "affine_masked"),
    ] {
        let specs = specs(&mut state, kind);
        let (mut apply, mut invert, mut roundtrip) = (Vec::new(), Vec::new(), Vec::new());
        for spec in &specs {
            for x in &inputs {
                let y = apply_view(x, *spec);
                apply.push(y == reference_apply(x, *spec));
                invert.push(invert_view(x, *spec) == reference_invert(x, *spec));
                roundtrip.push(y.and_then(|y| invert_view(&y, *spec)).as_ref() == Ok(x));
            }
        }
        case(&mut tallies, &format!("apply_{label}"), apply);
        case(&mut tallies, &format!("invert_{label}"), invert);
        case(&mut tallies, &format!("roundtrip_{label}"), roundtrip);
        // Even or zero multipliers fail with the same error on both paths.
        let x = inputs[inputs.len() - 1];
        case(
            &mut tallies,
            &format!("invalid_multiplier_{label}"),
            [0u16, 2, 4, 1024, 0xfffe].into_iter().flat_map(|a| {
                let spec = ViewSpec {
                    kind,
                    a,
                    b: 7,
                    seed: 9,
                };
                let applied = apply_view(&x, spec);
                let inverted = invert_view(&x, spec);
                [
                    applied.is_err() && applied == reference_apply(&x, spec),
                    inverted.is_err() && inverted == reference_invert(&x, spec),
                ]
            }),
        );
    }
    tallies
}

fn main() -> ExitCode {
    identity::report(&run(Scale::Full))
}
