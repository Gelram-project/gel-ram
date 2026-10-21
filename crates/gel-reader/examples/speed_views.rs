#![forbid(unsafe_code)]

//! Repeatable timing of `apply_view` and `invert_view` on 65,536
//! deterministic ORBs (8 MiB) for the affine geometries, with and without the
//! seeded mask. The rotate, reverse and XOR-mask cases run code outside this
//! change and serve as the unchanged comparison.
//!
//! Usage: `cargo run --release -p gel-reader --example speed_views -- [RUNS]`
//! (1..=999, default 9). Prints `SPEED name=<case> run=<i> ns=<n>` per run
//! and, at the end, `SPEED_CHECK sha256=<hex>` over every output ORB of every
//! run. Uses only API that predates this change, so the same file builds on
//! the earlier tree. Output grammar: `docs/SPEED.md`.

use gel_core::splitmix64;
use gel_orb::Orb1024;
use gel_reader::{apply_view, invert_view, ViewSpec};
use std::hint::black_box;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

const ORBS: usize = 65_536;

fn main() {
    let args = speed::args("speed_views", "[RUNS]", 1);
    let runs = speed::runs(&args);
    let mut state = 0x7669_6577_7332_3032u64;
    let inputs: Vec<Orb1024> = (0..ORBS)
        .map(|_| {
            Orb1024::from_words(std::array::from_fn(|_| {
                state = splitmix64(state);
                state
            }))
        })
        .collect();
    let masked = ViewSpec::affine_masked(5, 37, 0x4745_4c02).expect("odd multiplier");
    let perm = ViewSpec::affine(3, 11).expect("odd multiplier");
    let masked_views: Vec<Orb1024> = inputs
        .iter()
        .map(|x| apply_view(x, masked).expect("valid view"))
        .collect();
    let perm_views: Vec<Orb1024> = inputs
        .iter()
        .map(|x| apply_view(x, perm).expect("valid view"))
        .collect();
    let cases: [(&str, &[Orb1024], ViewSpec, bool); 7] = [
        ("apply_affine_masked_65536", &inputs, masked, false),
        ("invert_affine_masked_65536", &masked_views, masked, true),
        ("apply_affine_perm_65536", &inputs, perm, false),
        ("invert_affine_perm_65536", &perm_views, perm, true),
        ("apply_rotate_65536", &inputs, ViewSpec::rotate(257), false),
        ("apply_reverse_65536", &inputs, ViewSpec::reverse(), false),
        (
            "apply_xor_mask_65536",
            &inputs,
            ViewSpec::xor_mask(0x4745_4c01),
            false,
        ),
    ];
    speed::setup(
        "speed_views",
        runs,
        &[
            ("orbs", ORBS.to_string()),
            ("bytes", (ORBS * 128).to_string()),
        ],
    );
    let mut hash = speed::Sha256::new();
    let mut out: Vec<Orb1024> = Vec::with_capacity(ORBS);
    for (name, source, spec, invert) in cases {
        for run in 0..runs {
            out.clear();
            let start = Instant::now();
            for x in black_box(source) {
                let y = if invert {
                    invert_view(x, spec)
                } else {
                    apply_view(x, spec)
                };
                out.push(y.expect("valid view"));
            }
            black_box(&out);
            let ns = start.elapsed().as_nanos();
            speed::line(name, run, ns);
            for orb in &out {
                for w in orb.words() {
                    hash.update(&w.to_le_bytes());
                }
            }
        }
    }
    speed::check(hash);
}
