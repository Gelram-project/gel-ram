//! Reproducible supplemental mutants of bounded public fixtures, not exhaustive fuzzing.
#![forbid(unsafe_code)]
use gel_phase_quad::{fixture, Record};
use gel_source::{collection::Collection, digest};

const CASES: usize = 20_000;
fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
fn mutated(base: &[u8], state: &mut u64, iteration: usize) -> Vec<u8> {
    let mut bytes = base.to_vec();
    let offset = (random(state) % bytes.len() as u64) as usize;
    match iteration % 6 {
        0 => bytes[offset] ^= 1 << (random(state) % 8),
        1 => bytes[offset] = random(state) as u8,
        2 => bytes.truncate(offset),
        3 => bytes.push(random(state) as u8),
        4 => {
            bytes.remove(offset);
        }
        _ => {
            let end = (offset + 8).min(bytes.len());
            bytes[offset..end].fill(if random(state) & 1 == 0 { 0 } else { 255 });
        }
    }
    bytes
}

#[test]
fn collection_seeded_mutants_preserve_pins_and_canonical_roundtrips() {
    let mut collection = Collection::new();
    collection
        .add("Public sample", "Zażółć 🦀\r\nSample pressure is 2 bar.\n")
        .unwrap();
    collection.add("Second", "café\ncafe\u{301}\n").unwrap();
    let base = collection.to_bytes();
    let pin = digest(&base);
    let mut seed = 0x8e03_772b_4156_0001;
    let (mut accepted, mut rejected) = (0usize, 0usize);
    for iteration in 0..CASES {
        let bytes = mutated(&base, &mut seed, iteration);
        if bytes != base {
            assert!(
                Collection::from_bytes(&bytes, pin).is_err(),
                "case {iteration}"
            );
        }
        match Collection::from_bytes(&bytes, digest(&bytes)) {
            Ok(decoded) => {
                accepted += 1;
                assert_eq!(decoded.to_bytes(), bytes, "case {iteration}");
            }
            Err(_) => rejected += 1,
        }
    }
    assert_eq!(accepted + rejected, CASES);
    assert!(accepted > 0 && rejected > 0);
}

#[test]
fn q8_seeded_mutants_are_rejected_or_preserve_every_decoded_byte() {
    let base = fixture::encode(&[Record::new([7; 1024], &[true; 1024])]).unwrap();
    let mut seed = 0x8e03_772b_4156_0002;
    let (mut accepted, mut rejected) = (0usize, 0usize);
    for iteration in 0..CASES {
        let bytes = mutated(&base, &mut seed, iteration);
        match fixture::decode(&bytes) {
            Ok(decoded) => {
                accepted += 1;
                assert_eq!(
                    fixture::encode(&decoded).unwrap(),
                    bytes,
                    "case {iteration}"
                );
            }
            Err(_) => rejected += 1,
        }
    }
    assert_eq!(accepted + rejected, CASES);
    assert!(accepted > 0 && rejected > 0);
}
