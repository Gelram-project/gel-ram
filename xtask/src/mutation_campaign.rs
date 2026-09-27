//! Fresh paired timing and fresh-process memory of one collection mutation
//! (docs/MUTATION-COMPARISON.md, roadmap A09).
//!
//! Builds the mutation_compare harness once and runs its paired
//! timing mode into a new directory. Then, on Linux, runs its memory mode REPS
//! times for every variant, size and operation, each sample in a fresh process,
//! alternating which variant of a pair runs first. The harness resets VmHWM just
//! before the mutation, so the peak above the RSS at that moment belongs to the
//! mutation, not to building the bank. Both variants of a pair must
//! report the same result length and root. Records revision, host and load and
//! writes COMPLETE.txt only when every sample passed. A busy host is refused.
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const SIZES: [usize; 3] = [8, 64, 256];
const OPERATIONS: [&str; 3] = ["add", "replace", "remove"];
const VARIANTS: [&str; 2] = ["stream", "historical_vec"];
const DEFAULT_REPS: usize = 10;

#[derive(Debug, PartialEq)]
struct Sample {
    rss_before: i64,
    hwm_before: i64,
    rss_reset: i64,
    hwm_reset: i64,
    rss_after: i64,
    hwm_after: i64,
    latency_ns: i64,
    result_bytes: i64,
    root: String,
}

/// The data row of one memory-mode run; the run must end in MEMORY_SAMPLE=PASS.
fn parse_memory(stdout: &str) -> Result<(Sample, String), String> {
    if !stdout.lines().any(|l| l == "MEMORY_SAMPLE=PASS") {
        return Err("memory sample did not pass".into());
    }
    let mut lines = stdout.lines();
    lines
        .find(|l| l.starts_with("variant,documents,operation,"))
        .ok_or("memory sample has no header")?;
    let row = lines.next().ok_or("memory sample has no row")?;
    let f: Vec<&str> = row.split(',').collect();
    if f.len() != 12 {
        return Err(format!("unexpected memory row: {row}"));
    }
    let num = |i: usize| f[i].parse::<i64>().map_err(|e| format!("field {i}: {e}"));
    let harness = stdout
        .lines()
        .find_map(|l| l.strip_prefix("HARNESS_SHA256="))
        .ok_or("memory sample has no harness hash")?
        .to_string();
    Ok((
        Sample {
            rss_before: num(3)?,
            hwm_before: num(4)?,
            rss_reset: num(5)?,
            hwm_reset: num(6)?,
            rss_after: num(7)?,
            hwm_after: num(8)?,
            latency_ns: num(9)?,
            result_bytes: num(10)?,
            root: f[11].to_string(),
        },
        harness,
    ))
}

/// Nearest-rank percentile of sorted values (p in 1..=100).
fn rank(sorted: &[i64], p: usize) -> i64 {
    let k = (sorted.len() * p).div_ceil(100).max(1);
    sorted[k - 1]
}

fn stats(mut values: Vec<i64>) -> String {
    values.sort_unstable();
    format!(
        "{},{},{}",
        values[0],
        rank(&values, 50),
        values[values.len() - 1]
    )
}

/// Per size and operation: the median over pairs of historical_vec / stream
/// time, from the harness's own raw.csv (both results already oracle-checked).
fn paired_ratios(raw: &str) -> Result<String, String> {
    let mut pairs: BTreeMap<(usize, String, usize), [Option<f64>; 2]> = BTreeMap::new();
    for line in raw.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 9 || f[8] != "1" {
            return Err(format!("unexpected timing row: {line}"));
        }
        let n: usize = f[0].parse().map_err(|_| "documents")?;
        let sample: usize = f[1].parse().map_err(|_| "sample")?;
        let ns: f64 = f[6].parse().map_err(|_| "latency")?;
        let slot = match f[4] {
            "stream" => 0,
            "historical_vec" => 1,
            other => return Err(format!("unknown variant {other}")),
        };
        pairs.entry((n, f[3].to_string(), sample)).or_default()[slot] = Some(ns);
    }
    let mut ratios: BTreeMap<(usize, String), Vec<f64>> = BTreeMap::new();
    for ((n, op, _), pair) in pairs {
        let (Some(stream), Some(historical)) = (pair[0], pair[1]) else {
            return Err("unpaired timing row".into());
        };
        if stream <= 0.0 {
            return Err("zero stream time".into());
        }
        ratios.entry((n, op)).or_default().push(historical / stream);
    }
    let mut out = String::from("documents,operation,pairs,min_ratio,median_ratio,max_ratio\n");
    for ((n, op), mut r) in ratios {
        r.sort_by(f64::total_cmp);
        let k = (r.len() * 50).div_ceil(100).max(1);
        let _ = writeln!(
            out,
            "{n},{op},{},{:.2},{:.2},{:.2}",
            r.len(),
            r[0],
            r[k - 1],
            r[r.len() - 1]
        );
    }
    Ok(out)
}

fn load1() -> Option<f64> {
    fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage =
        "usage: cargo run -p xtask -- mutation-campaign NEW_DIR_OUTSIDE_CHECKOUT [--reps N]";
    let (dir, reps) = match args {
        [dir] => (dir, DEFAULT_REPS),
        [dir, flag, n] if flag == "--reps" => (
            dir,
            n.parse::<usize>()
                .ok()
                .filter(|r| (2..=100).contains(r))
                .ok_or(usage)?,
        ),
        _ => return Err(usage.into()),
    };
    if !cfg!(target_os = "linux") {
        return Err("the memory part reads Linux /proc; run on Linux".into());
    }
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let out = Path::new(dir);
    if out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("output directory must be outside the source checkout".into());
    }
    // Timings from a loaded host are not evidence; wait and retry instead.
    let cpus = std::thread::available_parallelism().map_or(1, usize::from) as f64;
    let limit = (cpus / 6.0).max(1.0);
    match load1() {
        Some(l) if l <= limit => {}
        Some(l) => {
            return Err(format!(
                "HOST_BUSY load1={l} limit={limit}; retry when idle"
            ))
        }
        None => return Err("cannot read /proc/loadavg".into()),
    }
    fs::create_dir(out).map_err(|e| format!("new output directory required: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;

    let status = Command::new("cargo")
        .current_dir(&root)
        .args([
            "build",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-source",
            "--example",
            "mutation_compare",
        ])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("harness build failed".into());
    }
    let harness: PathBuf = root.join("target/release/examples/mutation_compare");

    let mut text = String::from("FORMAT=GEL_MUTATION_CAMPAIGN_1\n");
    let _ = writeln!(text, "{}", super::bench_compare::revision(&root));
    text.push_str(&super::reproduction::host_lines());
    text.push_str(&super::reproduction::load("start"));
    let _ = writeln!(text, "load_limit={limit} memory_reps={reps}");

    let timing = out.join("timing");
    let status = Command::new(&harness)
        .arg(&timing)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() || !timing.join("COMPLETE").is_file() {
        return Err("paired timing run failed".into());
    }
    let raw = fs::read_to_string(timing.join("raw.csv")).map_err(|e| e.to_string())?;
    fs::write(out.join("timing-paired-ratios.txt"), paired_ratios(&raw)?)
        .map_err(|e| e.to_string())?;

    let mut memory = String::from("rep,position,variant,documents,operation,rss_before_bytes,hwm_before_bytes,rss_reset_bytes,hwm_reset_bytes,rss_after_bytes,hwm_after_bytes,latency_ns,result_bytes,result_root\n");
    let mut groups: BTreeMap<(&str, usize, &str), Vec<Sample>> = BTreeMap::new();
    let mut harness_hash = None;
    for rep in 0..reps {
        for n in SIZES {
            for op in OPERATIONS {
                let order = if rep % 2 == 0 { [0, 1] } else { [1, 0] };
                let mut pair: Vec<(usize, Sample)> = Vec::new();
                for (position, v) in order.into_iter().enumerate() {
                    let output = Command::new(&harness)
                        .args(["--memory", VARIANTS[v], &n.to_string(), op])
                        .output()
                        .map_err(|e| e.to_string())?;
                    if !output.status.success() {
                        return Err(format!("memory run failed: {} {n} {op}", VARIANTS[v]));
                    }
                    let (sample, hash) = parse_memory(&String::from_utf8_lossy(&output.stdout))?;
                    if *harness_hash.get_or_insert_with(|| hash.clone()) != hash {
                        return Err("harness changed during the campaign".into());
                    }
                    let _ = writeln!(
                        memory,
                        "{rep},{position},{},{n},{op},{},{},{},{},{},{},{},{},{}",
                        VARIANTS[v],
                        sample.rss_before,
                        sample.hwm_before,
                        sample.rss_reset,
                        sample.hwm_reset,
                        sample.rss_after,
                        sample.hwm_after,
                        sample.latency_ns,
                        sample.result_bytes,
                        sample.root
                    );
                    pair.push((v, sample));
                }
                if pair[0].1.root != pair[1].1.root
                    || pair[0].1.result_bytes != pair[1].1.result_bytes
                {
                    return Err(format!("variants disagree: {n} {op} rep {rep}"));
                }
                for (v, sample) in pair {
                    groups.entry((VARIANTS[v], n, op)).or_default().push(sample);
                }
            }
        }
    }
    fs::write(out.join("memory-raw.txt"), &memory).map_err(|e| e.to_string())?;
    let mut summary = String::from("variant,documents,operation,n,rss_delta_min,rss_delta_p50,rss_delta_max,mutation_peak_min,mutation_peak_p50,mutation_peak_max,reset_residual_max,build_hwm_min,build_hwm_p50,build_hwm_max\n");
    for ((variant, n, op), samples) in &groups {
        let rss: Vec<i64> = samples.iter().map(|s| s.rss_after - s.rss_before).collect();
        // Peak from the HWM reset to the after sample, above the RSS at the reset.
        let peak: Vec<i64> = samples.iter().map(|s| s.hwm_after - s.rss_reset).collect();
        let residual = samples
            .iter()
            .map(|s| s.hwm_reset - s.rss_reset)
            .max()
            .unwrap_or(0);
        let build: Vec<i64> = samples.iter().map(|s| s.hwm_before).collect();
        let _ = writeln!(
            summary,
            "{variant},{n},{op},{},{},{},{residual},{}",
            samples.len(),
            stats(rss),
            stats(peak),
            stats(build)
        );
    }
    fs::write(out.join("memory-summary.txt"), &summary).map_err(|e| e.to_string())?;

    let _ = writeln!(text, "harness_sha256={}", harness_hash.unwrap_or_default());
    text.push_str(&super::reproduction::load("end"));
    let line = format!(
        "MUTATION_CAMPAIGN=PASS timing_pairs=270 memory_samples={}",
        groups.values().map(Vec::len).sum::<usize>()
    );
    let _ = writeln!(text, "{line}");
    fs::write(out.join("CAMPAIGN.txt"), &text).map_err(|e| e.to_string())?;
    fs::write(out.join("COMPLETE.txt"), format!("{line}\n")).map_err(|e| e.to_string())?;
    println!("{line} {}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_memory_row_is_read_only_after_a_passing_sample() {
        let ok = "variant,documents,operation,x\nstream,8,add,1,2,3,4,5,6,7,8,ab\nHARNESS_SHA256=h\nMEMORY_SAMPLE=PASS\n";
        let (sample, hash) = parse_memory(ok).unwrap();
        assert_eq!(
            (sample.rss_before, sample.root.as_str(), hash.as_str()),
            (1, "ab", "h")
        );
        assert!(parse_memory(&ok.replace("MEMORY_SAMPLE=PASS", "")).is_err());
        assert!(parse_memory(&ok.replace(",ab", "")).is_err());
    }

    #[test]
    fn nearest_rank_and_pairing() {
        assert_eq!(rank(&[1, 2, 3, 4], 50), 2);
        assert_eq!(rank(&[1, 2, 3], 50), 2);
        let raw = "h\n8,1,0,add,stream,1,100,r,1\n8,1,1,add,historical_vec,1,300,r,1\n";
        assert!(paired_ratios(raw)
            .unwrap()
            .contains("8,add,1,3.00,3.00,3.00"));
        assert!(paired_ratios("h\n8,1,0,add,stream,1,100,r,1\n").is_err());
    }
}
