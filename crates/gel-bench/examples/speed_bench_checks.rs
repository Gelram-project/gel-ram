//! Wall-time harness for whole `gel-bench` runs, aimed at the untimed work
//! around the program's own timers (bank CRC, null and warm-up references,
//! quality probes). It starts the release binary as a child process, prints
//! the wall time of each run in nanoseconds, and a SHA-256 over every output
//! line except the ones that depend on timing or scheduling, so two builds
//! can be compared for both speed and output. Any added, removed or changed
//! line outside that short list changes the check.
//!
//! Build the binary together with this example:
//!   cargo build --release -p gel-bench --bins --examples
//! Usage: speed_bench_checks [RUNS] [BIN_DIR]
//! RUNS 1..=999 (default 9); BIN_DIR defaults to the directory above this
//! example's own directory.
//! Inputs: gel-bench with 1,310,720 orbs (160 MiB bank) and 16 rounds on 1
//! and 4 threads, with 131,072 orbs (16 MiB) and 16 rounds on 1 thread, and
//! with 8,192 orbs (1 MiB) and 3 rounds on 1 thread. A run that does not
//! start or fails ends with `SPEED_FAIL`. Output grammar: `docs/SPEED.md`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[path = "../../../tools/speed_support.rs"]
mod speed;

/// `gel-bench` keys whose values depend on timing or on where the process
/// was scheduled. Every other line is part of the check.
const TIMING_KEYS: &[&str] = &[
    "elapsed_ns",
    "query_p50_ns",
    "query_p95_ns",
    "query_p99_ns",
    "query_p99_is_max",
    "query_ns_execution_order",
    "orbs_per_sec",
    "effective_gib_per_sec",
    "observed_cpu_start",
    "observed_cpu_end",
];

fn bin_dir(args: &[String]) -> PathBuf {
    if let Some(dir) = args.get(1) {
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

/// Every output line except the timing and scheduling ones, in order.
fn stable_lines(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| !TIMING_KEYS.contains(&key_of(line)))
        .collect()
}

fn main() {
    let args = speed::args("speed_bench_checks", "[RUNS] [BIN_DIR]", 2);
    let runs = speed::runs(&args);
    let program = bin_dir(&args).join("gel-bench");
    if !program.is_file() {
        eprintln!(
            "missing {}: build with `cargo build --release -p gel-bench --bins --examples` or pass BIN_DIR",
            program.display()
        );
        std::process::exit(2);
    }
    let cases: [(&str, [&str; 3]); 4] = [
        ("gel_bench_1310720_t1", ["1310720", "16", "1"]),
        ("gel_bench_1310720_t4", ["1310720", "16", "4"]),
        ("gel_bench_131072_t1", ["131072", "16", "1"]),
        ("gel_bench_8192_t1", ["8192", "3", "1"]),
    ];
    speed::setup(
        "speed_bench_checks",
        runs,
        &[("bench_orbs", "1310720,131072,8192".to_string())],
    );
    let mut outputs = Vec::new();
    for (name, bench_args) in &cases {
        for run in 0..runs {
            let start = Instant::now();
            let result = Command::new(&program).args(bench_args).output();
            let ns = start.elapsed().as_nanos();
            let output = result.unwrap_or_else(|e| {
                eprintln!("gel-bench did not start: {e}");
                speed::fail("program_not_started");
            });
            if !output.status.success() {
                eprintln!(
                    "gel-bench failed: {}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                speed::fail("program_failed");
            }
            speed::line(name, run, ns);
            let stdout = String::from_utf8_lossy(&output.stdout);
            outputs.extend_from_slice(name.as_bytes());
            outputs.push(b'\n');
            for line in stable_lines(&stdout) {
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
    fn stable_lines_drop_only_timing_values() {
        let bench = "orbs=8\nelapsed_ns=17\nquality_flips_1=target:4,hit:4\nquery_p50_ns=9\nexecution_reported_scans=18\nobserved_cpu_end=3\n";
        assert_eq!(
            super::stable_lines(bench),
            [
                "orbs=8",
                "quality_flips_1=target:4,hit:4",
                "execution_reported_scans=18"
            ]
        );
    }

    /// The excluded keys are pinned: adding, removing or reordering a key
    /// must be a visible change to this test, not only to review.
    #[test]
    fn timing_keys_are_exactly_the_timing_and_scheduling_ones() {
        assert_eq!(
            super::TIMING_KEYS,
            [
                "elapsed_ns",
                "query_p50_ns",
                "query_p95_ns",
                "query_p99_ns",
                "query_p99_is_max",
                "query_ns_execution_order",
                "orbs_per_sec",
                "effective_gib_per_sec",
                "observed_cpu_start",
                "observed_cpu_end",
            ]
        );
        let checked = "bank_crc64_ecma=6f038d6db5b8839f\nexecution_reports=t1:1\nexecution_reported_scans=5\nquality_flips_1=target:4,hit:4\nnull_top1_index=0\nnull_top1_score=512\ntop1_exact=PASS\nthread_scan_exact=PASS\n";
        assert_eq!(
            super::stable_lines(checked),
            checked.lines().collect::<Vec<_>>()
        );
    }
}
