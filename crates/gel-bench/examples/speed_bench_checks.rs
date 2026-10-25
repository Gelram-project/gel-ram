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

/// The bytes one `gel-bench` run adds to the check: the case name and then
/// every stable output line, each followed by `\n`.
fn check_bytes(name: &str, stdout: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(name.len() + 1 + stdout.len());
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(b'\n');
    for line in stable_lines(stdout) {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    bytes
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
            outputs.extend_from_slice(&check_bytes(name, &stdout));
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
        // Every key of the full sample: the ten timing lines go, every other
        // line stays, in order. Dropping any other key family fails here.
        assert_eq!(
            super::stable_lines(FULL_OUTPUT),
            STABLE_OUTPUT.lines().collect::<Vec<_>>()
        );
        assert_eq!(
            super::stable_lines(STABLE_OUTPUT),
            STABLE_OUTPUT.lines().collect::<Vec<_>>()
        );
        for key in super::TIMING_KEYS {
            let lines = FULL_OUTPUT
                .lines()
                .filter(|line| super::key_of(line) == *key)
                .count();
            assert_eq!(lines, 1, "{key} must appear once in the sample");
        }
        assert_eq!(
            FULL_OUTPUT.lines().count(),
            STABLE_OUTPUT.lines().count() + super::TIMING_KEYS.len()
        );
    }

    /// One line for every key `gel-bench` prints, in its output order, from
    /// a two-worker run with one serial fallback, so the conditional
    /// `TIMING_COMPARISON` line is present too.
    const FULL_OUTPUT: &str = "\
GEL_BENCH_V5
profile=release
cpu_model=example cpu
generator=splitmix64_with_unique_word0
generator_seed=5135594514081467971
bank_crc64_ecma=6f038d6db5b8839f
orbs=8192
bank_bytes=1048576
orb_bytes=128
rounds=3
warmup_rounds=1
query_schedule=varying-exact-self-stride-65537
requested_workers=2
worker_budget=96
effective_workers_min=1
effective_workers_max=2
spawned_workers_total=4
execution_reported_scans=5
execution_scope=warmup+timed+threaded_null_check_if_requested
execution_reports=[Top1Execution{requested_workers:2,worker_budget:96,effective_workers:1,spawned_workers:0,fallback:Some(ThreadStartFailed)}]
fallback_scans=1
fallback_bank_too_small=0
fallback_budget_exceeded=0
fallback_thread_start_failed=1
fallback_worker_panicked=0
TIMING_COMPARISON=DEGRADED_SERIAL_FALLBACK
thread_scan=mixed_or_serial_fallback
thread_scan_exact=PASS
backend=count_ones(popcnt=false,avx2=false)
target_arch=x86_64
observed_cpu_start=8
scanned_orbs=24576
elapsed_ns=464545
query_p50_ns=150884
query_p95_ns=174899
query_p99_ns=174899
query_samples=3 percentile_method=sorted[ceil((n-1)*q)] empirical_not_a_tail_bound
query_p99_is_max=true
query_ns_execution_order=174899,150884,138231
orbs_per_sec=52903378.575
effective_gib_per_sec=6.306574
top1_exact=3/3
progressive_top8_exact=PASS
progressive_candidates=8192
progressive_reject_32b=0
progressive_reject_64b=0
progressive_full_128b=8192
quality_flips_1=target:4096,hit:4096,score:1023,survived:true
quality_flips_8=target:4096,hit:4096,score:1016,survived:true
quality_flips_32=target:4096,hit:4096,score:992,survived:true
quality_flips_64=target:4096,hit:4096,score:960,survived:true
quality_flips_128=target:4096,hit:4096,score:896,survived:true
quality_flips_256=target:4096,hit:4096,score:768,survived:true
quality_flips_384=target:4096,hit:4096,score:640,survived:true
null_top1_index=3916
null_top1_score=565
observed_cpu_end=8
";

    /// `FULL_OUTPUT` without its ten timing and scheduling lines.
    const STABLE_OUTPUT: &str = "\
GEL_BENCH_V5
profile=release
cpu_model=example cpu
generator=splitmix64_with_unique_word0
generator_seed=5135594514081467971
bank_crc64_ecma=6f038d6db5b8839f
orbs=8192
bank_bytes=1048576
orb_bytes=128
rounds=3
warmup_rounds=1
query_schedule=varying-exact-self-stride-65537
requested_workers=2
worker_budget=96
effective_workers_min=1
effective_workers_max=2
spawned_workers_total=4
execution_reported_scans=5
execution_scope=warmup+timed+threaded_null_check_if_requested
execution_reports=[Top1Execution{requested_workers:2,worker_budget:96,effective_workers:1,spawned_workers:0,fallback:Some(ThreadStartFailed)}]
fallback_scans=1
fallback_bank_too_small=0
fallback_budget_exceeded=0
fallback_thread_start_failed=1
fallback_worker_panicked=0
TIMING_COMPARISON=DEGRADED_SERIAL_FALLBACK
thread_scan=mixed_or_serial_fallback
thread_scan_exact=PASS
backend=count_ones(popcnt=false,avx2=false)
target_arch=x86_64
scanned_orbs=24576
query_samples=3 percentile_method=sorted[ceil((n-1)*q)] empirical_not_a_tail_bound
top1_exact=3/3
progressive_top8_exact=PASS
progressive_candidates=8192
progressive_reject_32b=0
progressive_reject_64b=0
progressive_full_128b=8192
quality_flips_1=target:4096,hit:4096,score:1023,survived:true
quality_flips_8=target:4096,hit:4096,score:1016,survived:true
quality_flips_32=target:4096,hit:4096,score:992,survived:true
quality_flips_64=target:4096,hit:4096,score:960,survived:true
quality_flips_128=target:4096,hit:4096,score:896,survived:true
quality_flips_256=target:4096,hit:4096,score:768,survived:true
quality_flips_384=target:4096,hit:4096,score:640,survived:true
null_top1_index=3916
null_top1_score=565
";

    /// The `gel-bench` source; its `run()` prints every output line.
    const BENCH_SOURCE: &str = include_str!("../src/main.rs");

    /// The key of every `println!` in `run()` of `source`, in source order.
    /// Each such call must start with a string literal; `{name}` templates
    /// are returned as written.
    fn printed_keys(source: &str) -> Vec<String> {
        let start = source.find("\nfn run()").expect("gel-bench has fn run()");
        let len = source[start..]
            .find("\nfn main()")
            .expect("fn main() follows fn run()");
        let mut rest = &source[start..start + len];
        let mut keys = Vec::new();
        while let Some(at) = rest.find("println!(") {
            rest = rest[at + "println!(".len()..].trim_start();
            let body = rest
                .strip_prefix('"')
                .expect("every println! in run() starts with a string literal");
            let end = body.find('"').expect("closed string literal");
            keys.push(super::key_of(&body[..end]).to_owned());
            rest = &body[end..];
        }
        keys
    }

    /// The sample is the full list: its keys, in order, are the keys the
    /// `gel-bench` source prints, with the two loop templates expanded. A
    /// new, removed or moved output line in `gel-bench` fails here until
    /// the sample follows it.
    #[test]
    fn sample_has_one_line_for_every_key_gel_bench_prints() {
        let printed: Vec<String> = printed_keys(BENCH_SOURCE)
            .into_iter()
            .flat_map(|key| match key.as_str() {
                "fallback_{label}" => [
                    "bank_too_small",
                    "budget_exceeded",
                    "thread_start_failed",
                    "worker_panicked",
                ]
                .iter()
                .map(|label| format!("fallback_{label}"))
                .collect::<Vec<_>>(),
                "quality_flips_{flips}" => [1, 8, 32, 64, 128, 256, 384]
                    .iter()
                    .map(|flips| format!("quality_flips_{flips}"))
                    .collect(),
                _ => {
                    assert!(!key.contains('{'), "unexpanded key template {key}");
                    vec![key]
                }
            })
            .collect();
        let sample: Vec<&str> = FULL_OUTPUT.lines().map(super::key_of).collect();
        assert_eq!(printed, sample);
        assert_eq!(sample.len(), 57);
        assert_eq!(
            printed_keys("\nfn run() {\n}\nfn main() {}\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn check_bytes_are_the_name_and_every_stable_line() {
        assert_eq!(
            super::check_bytes("case", "a=1\nelapsed_ns=5\nb=2\n"),
            b"case\na=1\nb=2\n"
        );
        assert_eq!(super::check_bytes("case", ""), b"case\n");
        assert_eq!(super::check_bytes("", "observed_cpu_end=1"), b"\n");
        // A timing key is dropped only when it is the whole key, not a prefix.
        assert_eq!(
            super::check_bytes(
                "c",
                "elapsed_ns_extra=1\nxelapsed_ns=2\nelapsed_ns =3\nelapsed_ns=4\n",
            ),
            b"c\nelapsed_ns_extra=1\nxelapsed_ns=2\nelapsed_ns =3\n"
        );
        // Leading and trailing white space is kept as printed.
        assert_eq!(
            super::check_bytes("c", " a=1 \n\tb=2\t\n"),
            b"c\n a=1 \n\tb=2\t\n"
        );
        let mut expected = b"gel_bench_8192_t1\n".to_vec();
        expected.extend_from_slice(STABLE_OUTPUT.as_bytes());
        assert_eq!(
            super::check_bytes("gel_bench_8192_t1", FULL_OUTPUT),
            expected
        );
    }

    /// Every line outside the timing list changes the check when it is
    /// changed, removed or added; every timing line leaves it as it is.
    #[test]
    fn check_changes_with_checked_lines_and_only_those() {
        let digest =
            |name: &str, stdout: &str| super::speed::sha256(&super::check_bytes(name, stdout));
        let base = digest("case", FULL_OUTPUT);
        let lines: Vec<&str> = FULL_OUTPUT.lines().collect();
        // The sample with line `target` replaced by the lines in `with`.
        let edited = |target: usize, with: &[String]| {
            let mut text = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i == target {
                    for new in with {
                        text.push_str(new);
                        text.push('\n');
                    }
                } else {
                    text.push_str(line);
                    text.push('\n');
                }
            }
            text
        };
        assert_eq!(edited(lines.len(), &[]), FULL_OUTPUT);
        let mut timing = 0;
        for (target, line) in lines.iter().enumerate() {
            let is_timing = super::TIMING_KEYS.contains(&super::key_of(line));
            timing += usize::from(is_timing);
            let variants = [
                ("changed", edited(target, &[format!("{line}9")])),
                ("removed", edited(target, &[])),
                (
                    "doubled",
                    edited(target, &[line.to_string(), line.to_string()]),
                ),
            ];
            for (what, text) in variants {
                assert_eq!(
                    digest("case", &text) == base,
                    is_timing,
                    "line {target} {what}: {line}"
                );
            }
        }
        assert_eq!(timing, super::TIMING_KEYS.len());
        assert_ne!(digest("case", &format!("{FULL_OUTPUT}extra_key=1\n")), base);
        assert_ne!(digest("case", &format!("extra_line\n{FULL_OUTPUT}")), base);
        for key in super::TIMING_KEYS {
            assert_eq!(
                digest("case", &format!("{FULL_OUTPUT}{key}=1\n")),
                base,
                "{key}"
            );
        }
        assert_ne!(digest("case2", FULL_OUTPUT), base);
    }

    /// The earlier assembly, inline in `main()`, for one run.
    fn old_check_bytes(outputs: &mut Vec<u8>, name: &str, stdout: &str) {
        outputs.extend_from_slice(name.as_bytes());
        outputs.push(b'\n');
        for line in super::stable_lines(stdout) {
            outputs.extend_from_slice(line.as_bytes());
            outputs.push(b'\n');
        }
    }

    #[test]
    fn check_bytes_match_the_earlier_inline_assembly() {
        let stdouts = [
            FULL_OUTPUT,
            STABLE_OUTPUT,
            "",
            "\n",
            "\n\n",
            "a=1",
            "a=1\r\nelapsed_ns=2\r\nb=3\r\n",
            "no_equals_line\nelapsed_ns\n",
            "key=\n=value\n==\n",
            "elapsed_ns_extra=1\nxelapsed_ns=2\nelapsed_ns =3\n",
            "elapsed_ns_extra=1\nxelapsed_ns=2\nelapsed_ns =3\nelapsed_ns=4\n",
            " a=1 \n\tb=2\t\n",
            "orbs_per_sec=1=2\nquery_p99_is_max\n",
            "bad \u{fffd} byte=1\n",
        ];
        let names = ["gel_bench_1310720_t1", "gel_bench_8192_t1", ""];
        let mut old = Vec::new();
        let mut new = Vec::new();
        for name in names {
            for stdout in stdouts {
                let mut one = Vec::new();
                old_check_bytes(&mut one, name, stdout);
                assert_eq!(super::check_bytes(name, stdout), one, "{name}: {stdout:?}");
                old_check_bytes(&mut old, name, stdout);
                new.extend_from_slice(&super::check_bytes(name, stdout));
            }
        }
        assert_eq!(new, old);
        assert_eq!(super::speed::sha256(&new), super::speed::sha256(&old));
    }
}
