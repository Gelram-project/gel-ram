//! Wall-time harness for whole `gel-bench` and `gel-physics` runs. It starts
//! the release binaries as child processes, prints the wall time of each run
//! in nanoseconds, and a SHA-256 over the run-independent output lines (the
//! headers, bank CRC, Top-K and quality results, row keys), so two builds
//! can be compared for both speed and output. Timing values printed by the
//! programs themselves are not part of the check.
//!
//! Build the binaries together with this example:
//!   cargo build --release -p gel-bench -p gel-physics --bins --examples
//! Usage: speed_bench_physics [RUNS] [PHYSICS_REPEATS] [BIN_DIR]
//! RUNS 1..=999 (default 9), PHYSICS_REPEATS 3..=999 (default 3, the
//! smallest the program accepts), BIN_DIR defaults to the directory above
//! this example's own directory.
//! Inputs: gel-bench with 1,310,720 orbs (160 MiB bank) and 16 rounds on 1
//! and 4 threads, gel-bench with 8,192 orbs (1 MiB) and 3 rounds, and
//! gel-physics over its fixed sizes from 48 KiB to 256 MiB. A program that
//! does not start or fails ends the run with `SPEED_FAIL`. Output grammar:
//! `docs/SPEED.md`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

/// `gel-bench` keys whose values do not depend on timing or scheduling.
const BENCH_KEYS: &[&str] = &[
    "GEL_BENCH_V5",
    "profile",
    "generator",
    "generator_seed",
    "bank_crc64_ecma",
    "orbs",
    "bank_bytes",
    "orb_bytes",
    "rounds",
    "warmup_rounds",
    "query_schedule",
    "requested_workers",
    "execution_scope",
    "thread_scan_exact",
    "backend",
    "target_arch",
    "scanned_orbs",
    "top1_exact",
    "progressive_top8_exact",
    "progressive_candidates",
    "progressive_reject_32b",
    "progressive_reject_64b",
    "progressive_full_128b",
    "null_top1_index",
    "null_top1_score",
];

/// `gel-physics` header keys whose values do not depend on timing.
const PHYSICS_KEYS: &[&str] = &[
    "GEL_PHYSICS_F0_V3",
    "timing_authority",
    "cycles",
    "profile",
    "target_arch",
    "repeats",
    "sequential_probe",
    "probe_buffer_alignment",
];

fn bin_dir(args: &[String]) -> PathBuf {
    if let Some(dir) = args.get(2) {
        return PathBuf::from(dir);
    }
    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("cannot locate this example: {e}");
        std::process::exit(2);
    });
    exe.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn key_of(line: &str) -> &str {
    line.split_once('=').map_or(line, |(key, _)| key)
}

/// The run-independent part of one program's output, one line each.
fn stable_lines(program: &str, stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        if program == "gel-bench" {
            if BENCH_KEYS.contains(&key_of(line)) || line.starts_with("quality_flips_") {
                out.push(line.to_owned());
            }
        } else if PHYSICS_KEYS.contains(&key_of(line)) || line.starts_with("--- ") {
            out.push(line.to_owned());
        } else if line.starts_with("bytes=") || line.starts_with("working_set_bytes=") {
            // Row keys only: the sizes, not the measured values.
            let keys: Vec<&str> = line
                .split(' ')
                .filter(|token| {
                    let key = key_of(token);
                    key == "bytes" || key == "working_set_bytes" || key == "fetch_bytes"
                })
                .collect();
            out.push(keys.join(" "));
        }
    }
    out
}

fn main() {
    let args = speed::args(
        "speed_bench_physics",
        "[RUNS] [PHYSICS_REPEATS] [BIN_DIR]",
        3,
    );
    let runs = speed::runs(&args);
    let physics_repeats = speed::number(&args, 1, "PHYSICS_REPEATS", 3, 3..=999);
    let dir = bin_dir(&args);
    let repeats = physics_repeats.to_string();
    let physics_name = format!("gel_physics_r{physics_repeats}");
    let cases: [(&str, &str, Vec<&str>); 4] = [
        (
            "gel_bench_1310720_t1",
            "gel-bench",
            vec!["1310720", "16", "1"],
        ),
        (
            "gel_bench_1310720_t4",
            "gel-bench",
            vec!["1310720", "16", "4"],
        ),
        ("gel_bench_8192_t1", "gel-bench", vec!["8192", "3", "1"]),
        (physics_name.as_str(), "gel-physics", vec![repeats.as_str()]),
    ];
    for (_, program, _) in &cases {
        let path = dir.join(program);
        if !path.is_file() {
            eprintln!(
                "missing {}: build with `cargo build --release -p gel-bench -p gel-physics --bins --examples` or pass BIN_DIR",
                path.display()
            );
            std::process::exit(2);
        }
    }
    speed::setup(
        "speed_bench_physics",
        runs,
        &[
            ("physics_repeats", physics_repeats.to_string()),
            ("bench_orbs", "1310720,8192".to_string()),
        ],
    );
    let mut outputs = Vec::new();
    for (name, program, args) in &cases {
        for run in 0..runs {
            let start = Instant::now();
            let result = Command::new(dir.join(program)).args(args).output();
            let ns = start.elapsed().as_nanos();
            let output = result.unwrap_or_else(|e| {
                eprintln!("{program} did not start: {e}");
                speed::fail("program_not_started");
            });
            if !output.status.success() {
                eprintln!(
                    "{program} failed: {}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                speed::fail("program_failed");
            }
            speed::line(name, run, ns);
            let stdout = String::from_utf8_lossy(&output.stdout);
            outputs.extend_from_slice(name.as_bytes());
            outputs.push(b'\n');
            for line in stable_lines(program, &stdout) {
                outputs.extend_from_slice(line.as_bytes());
                outputs.push(b'\n');
            }
        }
    }
    speed::check_digest(&speed::sha256(&outputs));
}

#[cfg(test)]
mod tests {
    #[test]
    fn stable_lines_drop_measured_values() {
        let bench = "orbs=8\nelapsed_ns=17\nquality_flips_1=target:4,hit:4\nquery_p50_ns=9\n";
        assert_eq!(
            super::stable_lines("gel-bench", bench),
            ["orbs=8", "quality_flips_1=target:4,hit:4"]
        );
        let physics = "--- pointer_chase ---\nbytes=49152 min_ns_per_step=1.000 execution_order=1.000\nworking_set_bytes=67108864 fetch_bytes=32 min_ns=2.000\nthp_policy=x\n";
        assert_eq!(
            super::stable_lines("gel-physics", physics),
            [
                "--- pointer_chase ---",
                "bytes=49152",
                "working_set_bytes=67108864 fetch_bytes=32"
            ]
        );
    }
}
