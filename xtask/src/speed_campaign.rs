// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Comparison-protocol records of `speed-ab` and the campaigns that re-run a
//! historical timing set under that protocol (docs/EVIDENCE-CAMPAIGN.md).
//!
//! `speed-ab-check DIR` accepts one `speed-ab` output directory only when its
//! manifest states every protocol field (input, profile, clock boundary,
//! warm-up, order, A/A control, host metadata), names no path on the host,
//! the rounds rotate completely, every raw run file is present and the
//! summary recomputes byte for byte from the measured runs. A record without
//! an A/A control is refused. It checks a record, not the honesty of a clock.
//!
//! `speed-campaign` runs every example of a plan file through `speed-ab` in
//! the same three trees, checks each record and writes `CAMPAIGN.txt`.
use super::speed_ab::{self, Tree, TreeData};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const CHECK_USAGE: &str = "usage: cargo run -p xtask -- speed-ab-check DIR";
const CAMPAIGN_USAGE: &str = "usage: cargo run -p xtask -- speed-campaign --plan FILE --a DIR --a2 DIR --b DIR --target-root DIR --out NEW_DIR [--invalid-for-timing [--runs N] [--only EXAMPLE]...] [--before-run PROGRAM [--before-run-arg ARG]...]";
const PLAN_FORMAT: &str = "PLAN=GEL_SPEED_CAMPAIGN_1";
const CAMPAIGN_FORMAT: &str = "FORMAT=GEL_SPEED_CAMPAIGN_1";
/// The example argument a plan writes for a new, empty directory of that
/// example, made by the campaign under `OUT/work/`.
const WORK_DIR: &str = "WORK_DIR";

/// Keys a protocol manifest may contain, each at the start of a line.
const KEYS: &[&str] = &[
    "FORMAT",
    "protocol",
    "timing_validity",
    "mode",
    "package",
    "input_args",
    "with_bins",
    "before_run",
    "profile",
    "rustflags",
    "cargo_env",
    "clock_boundary",
    "warmup_rounds",
    "order",
    "aa_control",
    "host_os",
    "host_arch",
    "host_logical_cpus",
    "host_cpu_model",
    "host_mem_total_kib",
    "host_kernel",
    "host_load_start",
    "host_load_end",
    "replace_helpers",
    "source_sha256",
    "tree",
    "placed_dir",
    "placed",
    "replaced",
    "binary_sha256",
    "aa_same_binary",
    "warmup_round",
    "round",
    "restored",
];

/// Keys that must appear exactly once.
const ONCE: &[&str] = &[
    "FORMAT",
    "protocol",
    "timing_validity",
    "mode",
    "package",
    "input_args",
    "with_bins",
    "before_run",
    "profile",
    "rustflags",
    "cargo_env",
    "clock_boundary",
    "warmup_rounds",
    "order",
    "aa_control",
    "host_os",
    "host_arch",
    "host_logical_cpus",
    "host_cpu_model",
    "host_mem_total_kib",
    "host_kernel",
    "host_load_start",
    "host_load_end",
    "aa_same_binary",
    "restored",
];

fn key(line: &str) -> &str {
    let end = line.find(['=', ' ']).unwrap_or(line.len());
    &line[..end]
}

fn value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)?.strip_prefix('=')
}

fn number(s: &str) -> Option<usize> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// What a checked record states.
#[derive(Debug, PartialEq)]
pub(crate) struct Checked {
    pub(crate) cases: usize,
    pub(crate) rounds: usize,
    pub(crate) warmup: usize,
    pub(crate) verdict: String,
    pub(crate) validity: String,
}

/// Checks the manifest text alone and returns (rounds, warm-up rounds,
/// whether replaced helpers are kept, timing validity).
fn check_manifest(text: &str) -> Result<(usize, usize, bool, String), String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&speed_ab::FORMAT) {
        return Err(format!(
            "manifest does not begin with {}; earlier records are historical, not protocol records",
            speed_ab::FORMAT
        ));
    }
    for line in &lines {
        if !KEYS.contains(&key(line)) {
            return Err(format!("manifest line with an unknown key: {line:?}"));
        }
        if let Some(word) = line
            .split(|c: char| c.is_whitespace() || "=\"[],".contains(c))
            .find(|w| speed_ab::names_a_path(w))
        {
            return Err(format!("manifest names a path on the host: {word:?}"));
        }
    }
    let one = |k: &str| -> Result<&str, String> {
        let found: Vec<&&str> = lines.iter().filter(|l| key(l) == k).collect();
        match found.as_slice() {
            [line] => Ok(line),
            [] => Err(format!("manifest has no {k} line (protocol field missing)")),
            _ => Err(format!("manifest has more than one {k} line")),
        }
    };
    for k in ONCE {
        let line = one(k)?;
        if line.len() <= k.len() + 1 {
            return Err(format!("manifest line {k} has no value"));
        }
    }
    let exact = |k: &str, wanted: &str| -> Result<(), String> {
        if one(k)? == wanted {
            Ok(())
        } else {
            Err(format!("manifest line {k} differs from the protocol"))
        }
    };
    exact("protocol", speed_ab::PROTOCOL)?;
    exact("clock_boundary", speed_ab::CLOCK_BOUNDARY)?;
    exact("order", speed_ab::ORDER)?;
    exact("aa_control", speed_ab::AA_CONTROL)?;
    exact("restored", "restored=yes")?;
    match value(one("mode")?, "mode") {
        Some("ab") => {}
        Some("candidate_only") => {
            return Err("the record has no A/A control (candidate-only run)".into())
        }
        _ => return Err("manifest mode is neither ab nor candidate_only".into()),
    }
    let validity = value(one("timing_validity")?, "timing_validity").unwrap_or("");
    if !["AS_MEASURED", "INVALID_FOR_TIMING"].contains(&validity) {
        return Err(format!("unknown timing_validity {validity:?}"));
    }
    if !matches!(
        value(one("aa_same_binary")?, "aa_same_binary"),
        Some("yes" | "no")
    ) {
        return Err("aa_same_binary must be yes or no".into());
    }
    let package = one("package")?;
    let rounds = package
        .split(' ')
        .find_map(|w| value(w, "rounds"))
        .and_then(number)
        .ok_or("package line has no rounds=")?;
    if rounds < 3 || rounds % 3 != 0 {
        return Err(format!(
            "rounds={rounds}: the rotation of three trees needs a multiple of 3"
        ));
    }
    let warmup = value(one("warmup_rounds")?, "warmup_rounds")
        .and_then(number)
        .filter(|w| *w <= speed_ab::MAX_WARMUP)
        .ok_or("warmup_rounds is not a number in range")?;
    for tree in Tree::ALL {
        let label = tree.label();
        let trees: Vec<&&str> = lines
            .iter()
            .filter(|l| key(l) == "tree" && l.starts_with(&format!("tree={label} ")))
            .collect();
        if trees.len() != 1 || !trees[0].contains(" revision=") {
            return Err(format!(
                "manifest needs one tree={label} line with its revision"
            ));
        }
        let hashes: Vec<&&str> = lines
            .iter()
            .filter(|l| l.starts_with(&format!("binary_sha256 tree={label} ")))
            .collect();
        match hashes.as_slice() {
            [line] if is_hex64(line.rsplit(' ').next().unwrap_or("")) => {}
            _ => return Err(format!("manifest needs one binary_sha256 for tree {label}")),
        }
    }
    if !lines.iter().any(|l| key(l) == "source_sha256") {
        return Err("manifest has no source_sha256 line".into());
    }
    // Warm-up rounds come first, then the measured rounds, each in its rotation.
    let sequence: Vec<&&str> = lines
        .iter()
        .filter(|l| matches!(key(l), "warmup_round" | "round"))
        .collect();
    let wanted: Vec<String> = (1..=warmup)
        .map(|k| ("warmup_round", k))
        .chain((1..=rounds).map(|r| ("round", r)))
        .map(|(k, r)| {
            let labels: Vec<&str> = speed_ab::order(r).iter().map(|t| t.label()).collect();
            format!("{k}={r} order={}", labels.join(","))
        })
        .collect();
    if sequence.len() != wanted.len() || sequence.iter().zip(&wanted).any(|(l, w)| **l != w) {
        return Err("round lines do not follow the warm-up count and the rotated order".into());
    }
    let replace = lines.iter().any(|l| key(l) == "replace_helpers");
    Ok((rounds, warmup, replace, validity.to_string()))
}

/// Checks one `speed-ab` output directory against the protocol.
pub(crate) fn check_dir(dir: &Path) -> Result<Checked, String> {
    let read = |name: &str| fs::read_to_string(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    let (rounds, warmup, replace, validity) = check_manifest(&read("manifest.txt")?)?;
    let mut expected: BTreeSet<String> = ["manifest.txt", "summary.txt"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for tree in Tree::ALL {
        for r in 1..=rounds {
            expected.insert(speed_ab::run_file(false, tree, r));
        }
        for k in 1..=warmup {
            expected.insert(speed_ab::run_file(true, tree, k));
        }
    }
    let mut present = BTreeSet::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if replace && kind.is_dir() && (name == "replaced-a" || name == "replaced-a2") {
            continue;
        }
        if !kind.is_file() || !expected.contains(&name) {
            return Err(format!("unexpected entry in the record: {name}"));
        }
        present.insert(name);
    }
    if let Some(missing) = expected.difference(&present).next() {
        return Err(format!("raw run file missing: {missing}"));
    }
    let mut data = [
        TreeData::default(),
        TreeData::default(),
        TreeData::default(),
    ];
    for (i, tree) in Tree::ALL.iter().enumerate() {
        for k in 1..=warmup {
            let name = speed_ab::run_file(true, *tree, k);
            speed_ab::parse_output(&read(&name)?).map_err(|e| format!("{name}: {e}"))?;
        }
        for r in 1..=rounds {
            let name = speed_ab::run_file(false, *tree, r);
            let run = speed_ab::parse_output(&read(&name)?).map_err(|e| format!("{name}: {e}"))?;
            data[i].add(run);
        }
    }
    let (report, verdict) = speed_ab::report_ab(&data[0], &data[1], &data[2]);
    if read("summary.txt")? != report {
        return Err("summary.txt does not recompute from the raw run files".into());
    }
    if verdict == speed_ab::Verdict::NoData {
        return Err("a tree printed no SPEED line or no SPEED_CHECK value".into());
    }
    let mut cases = 0;
    for line in report.lines().filter(|l| l.starts_with("SPEED_AB case=")) {
        cases += 1;
        let field = |k: &str| line.split(' ').find_map(|w| value(w, k));
        if [field("a_ns"), field("a2_ns"), field("b_ns")]
            .iter()
            .any(|v| v.and_then(number).is_none())
        {
            return Err(format!("a case lacks a sample in A, A2 or B: {line}"));
        }
    }
    Ok(Checked {
        cases,
        rounds,
        warmup,
        verdict: verdict.word().to_string(),
        validity,
    })
}

/// `xtask speed-ab-check DIR`.
pub fn check_cmd(args: &[String]) -> Result<(), String> {
    let [dir] = args else {
        return Err(CHECK_USAGE.into());
    };
    let c = check_dir(Path::new(dir)).map_err(|e| format!("SPEED_AB_CHECK=FAIL {e}"))?;
    println!(
        "SPEED_AB_CHECK=PASS cases={} rounds={} warmup={} verdict={} timing_validity={}",
        c.cases, c.rounds, c.warmup, c.verdict, c.validity
    );
    Ok(())
}

/// One example line of a plan.
#[derive(Debug, PartialEq)]
struct Entry {
    package: String,
    example: String,
    expect: String,
    with_bins: Vec<String>,
    args: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct Plan {
    name: String,
    reruns: String,
    baseline: String,
    candidate: String,
    rounds: usize,
    warmup: usize,
    entries: Vec<Entry>,
    skips: Vec<(String, String)>,
}

fn plain(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

fn list(s: &str) -> Vec<String> {
    s.split(',').map(str::to_string).collect()
}

fn parse_plan(text: &str) -> Result<Plan, String> {
    let mut lines = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    if lines.next() != Some(PLAN_FORMAT) {
        return Err(format!("plan must begin with {PLAN_FORMAT}"));
    }
    let mut header: Vec<(String, String)> = Vec::new();
    let mut entries = Vec::new();
    let mut skips = Vec::new();
    let mut names = BTreeSet::new();
    for line in lines {
        let words: Vec<&str> = line.split(' ').collect();
        match words[0] {
            "example" | "skip" => {
                let [_, package, example, rest @ ..] = words.as_slice() else {
                    return Err(format!("plan line needs a package and an example: {line}"));
                };
                if !plain(package) || !plain(example) || !names.insert(example.to_string()) {
                    return Err(format!("plan names a bad or repeated example: {line}"));
                }
                if words[0] == "skip" {
                    let reason = rest
                        .first()
                        .and_then(|w| w.strip_prefix("reason="))
                        .filter(|r| !r.is_empty())
                        .ok_or_else(|| format!("a skip needs reason=: {line}"))?;
                    let tail: Vec<&str> = std::iter::once(reason)
                        .chain(rest[1..].iter().copied())
                        .collect();
                    skips.push((example.to_string(), tail.join(" ")));
                    continue;
                }
                let mut entry = Entry {
                    package: package.to_string(),
                    example: example.to_string(),
                    expect: "same_output".into(),
                    with_bins: Vec::new(),
                    args: Vec::new(),
                };
                for word in rest {
                    if let Some(v) = word.strip_prefix("expect=") {
                        if !["same_output", "different_output"].contains(&v) {
                            return Err(format!("unknown expect={v}"));
                        }
                        entry.expect = v.to_string();
                    } else if let Some(v) = word.strip_prefix("with_bins=") {
                        entry.with_bins = list(v);
                        if !entry.with_bins.iter().all(|p| plain(p)) {
                            return Err(format!("bad with_bins: {line}"));
                        }
                    } else if let Some(v) = word.strip_prefix("args=") {
                        entry.args = list(v);
                        if !entry
                            .args
                            .iter()
                            .all(|a| a == WORK_DIR || number(a).is_some())
                        {
                            return Err(format!(
                                "plan arguments are numbers or {WORK_DIR}: {line}"
                            ));
                        }
                    } else {
                        return Err(format!("unknown plan word {word:?}"));
                    }
                }
                entries.push(entry);
            }
            _ => {
                let (k, v) = line
                    .split_once('=')
                    .ok_or_else(|| format!("plan line without key=value: {line}"))?;
                if header.iter().any(|(h, _)| h == k) {
                    return Err(format!("plan key {k} given twice"));
                }
                header.push((k.to_string(), v.to_string()));
            }
        }
    }
    let get = |k: &str| {
        header
            .iter()
            .find(|(h, _)| h == k)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| format!("plan has no {k}="))
    };
    if let Some((k, _)) = header.iter().find(|(k, _)| {
        ![
            "name",
            "reruns",
            "baseline",
            "candidate",
            "rounds",
            "warmup",
        ]
        .contains(&k.as_str())
    }) {
        return Err(format!("unknown plan key {k}"));
    }
    let revision = |k: &str| -> Result<String, String> {
        let v = get(k)?;
        if v.len() == 40
            && v.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            Ok(v)
        } else {
            Err(format!("plan {k} must be a full lowercase commit id"))
        }
    };
    let rounds = number(&get("rounds")?)
        .filter(|r| *r >= 3 && r % 3 == 0)
        .ok_or("plan rounds must be a multiple of 3")?;
    let warmup = number(&get("warmup")?)
        .filter(|w| (1..=speed_ab::MAX_WARMUP).contains(w))
        .ok_or("plan warmup must be 1 or more")?;
    let name = get("name")?;
    if !plain(&name) {
        return Err("plan name must be a plain name".into());
    }
    if entries.is_empty() {
        return Err("plan has no example".into());
    }
    Ok(Plan {
        name,
        reruns: get("reruns")?,
        baseline: revision("baseline")?,
        candidate: revision("candidate")?,
        rounds,
        warmup,
        entries,
        skips,
    })
}

#[derive(Debug, PartialEq)]
struct CampaignOptions {
    plan: PathBuf,
    a: PathBuf,
    a2: PathBuf,
    b: PathBuf,
    target_root: PathBuf,
    out: PathBuf,
    invalid_for_timing: bool,
    runs: Option<usize>,
    only: Vec<String>,
    before_run: Vec<String>,
}

fn campaign_options(args: &[String]) -> Result<CampaignOptions, String> {
    let mut slots: [(&str, Option<String>); 7] = [
        ("--plan", None),
        ("--a", None),
        ("--a2", None),
        ("--b", None),
        ("--target-root", None),
        ("--out", None),
        ("--runs", None),
    ];
    let mut invalid_for_timing = false;
    let mut only = Vec::new();
    let mut before_program = None;
    let mut before_args = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        i += 1;
        if flag == "--invalid-for-timing" {
            if invalid_for_timing {
                return Err("--invalid-for-timing given twice".into());
            }
            invalid_for_timing = true;
            continue;
        }
        let v = args
            .get(i)
            .ok_or_else(|| format!("{flag} needs a value"))?
            .clone();
        i += 1;
        match flag {
            "--only" => only.push(v),
            "--before-run-arg" => before_args.push(v),
            "--before-run" => {
                if before_program.replace(v).is_some() {
                    return Err("--before-run given twice".into());
                }
            }
            _ => {
                let slot = slots
                    .iter_mut()
                    .find(|(name, _)| *name == flag)
                    .ok_or_else(|| format!("unknown option {flag}"))?;
                if slot.1.replace(v).is_some() {
                    return Err(format!("{flag} given twice"));
                }
            }
        }
    }
    let [plan, a, a2, b, target_root, out, runs] = slots.map(|(_, v)| v);
    let need = |v: Option<String>, f: &str| v.map(PathBuf::from).ok_or(format!("{f} is required"));
    let runs = match runs {
        None => None,
        Some(r) => Some(
            number(&r)
                .filter(|n| (1..=999).contains(n))
                .ok_or("--runs must be 1..=999")?,
        ),
    };
    if (runs.is_some() || !only.is_empty()) && !invalid_for_timing {
        return Err(
            "--runs and --only change the planned campaign; they need --invalid-for-timing".into(),
        );
    }
    let before_run = match before_program {
        Some(p) => std::iter::once(p).chain(before_args).collect(),
        None if before_args.is_empty() => Vec::new(),
        None => return Err("--before-run-arg needs --before-run".into()),
    };
    Ok(CampaignOptions {
        plan: need(plan, "--plan")?,
        a: need(a, "--a")?,
        a2: need(a2, "--a2")?,
        b: need(b, "--b")?,
        target_root: need(target_root, "--target-root")?,
        out: need(out, "--out")?,
        invalid_for_timing,
        runs,
        only,
        before_run,
    })
}

/// The example arguments of one entry: RUNS replaced or put first when
/// `--runs` is given, WORK_DIR replaced by the example's new, empty
/// directory under `work`, in which the example makes its own scratch directory.
fn entry_args(entry: &Entry, runs: Option<usize>, work: &Path) -> Vec<String> {
    let mut args: Vec<String> = entry
        .args
        .iter()
        .map(|a| {
            if a == WORK_DIR {
                work.join(&entry.example).display().to_string()
            } else {
                a.clone()
            }
        })
        .collect();
    if let Some(n) = runs {
        match args.first().map(|a| number(a).is_some()) {
            Some(true) => args[0] = n.to_string(),
            _ => args.insert(0, n.to_string()),
        }
    }
    args
}

/// `xtask speed-campaign`.
pub fn campaign_cmd(args: &[String]) -> Result<(), String> {
    let o = campaign_options(args).map_err(|e| format!("{CAMPAIGN_USAGE}\nspeed-campaign: {e}"))?;
    let plan_bytes = fs::read(&o.plan).map_err(|e| format!("--plan: {e}"))?;
    let plan = parse_plan(&String::from_utf8_lossy(&plan_bytes))?;
    if let Some(name) = o
        .only
        .iter()
        .find(|n| !plan.entries.iter().any(|e| &e.example == *n))
    {
        return Err(format!("--only {name}: not an example line of the plan"));
    }
    // The trees must be clean checkouts of the planned revisions.
    let mut tree_lines = String::new();
    for (label, root, rev) in [
        ("a", &o.a, &plan.baseline),
        ("a2", &o.a2, &plan.baseline),
        ("b", &o.b, &plan.candidate),
    ] {
        let found = super::bench_compare::revision(root);
        if found != format!("revision={rev} working_tree=clean") {
            return Err(format!(
                "tree {label} must be a clean checkout of {rev}; found {found}"
            ));
        }
        let _ = writeln!(tree_lines, "tree={label} {found}");
    }
    fs::create_dir(&o.out).map_err(|e| format!("new --out directory required: {e}"))?;
    let out = o.out.canonicalize().map_err(|e| e.to_string())?;
    let work = out.join("work");
    fs::create_dir(&work).map_err(|e| e.to_string())?;
    let mut log = String::new();
    let _ = writeln!(log, "{CAMPAIGN_FORMAT}");
    let _ = writeln!(
        log,
        "plan={} plan_sha256={}",
        plan.name,
        gel_source::hex(&gel_source::digest(&plan_bytes))
    );
    let _ = writeln!(log, "reruns={}", plan.reruns);
    let _ = writeln!(log, "rounds={} warmup={}", plan.rounds, plan.warmup);
    let _ = writeln!(
        log,
        "runs={}",
        o.runs
            .map_or_else(|| "as_planned".to_string(), |n| n.to_string())
    );
    let _ = writeln!(
        log,
        "timing_validity={}",
        if o.invalid_for_timing {
            "INVALID_FOR_TIMING"
        } else {
            "AS_MEASURED"
        }
    );
    log.push_str(&tree_lines);
    for (example, reason) in &plan.skips {
        let _ = writeln!(log, "skip={example} reason={reason}");
    }
    let write_log = |log: &str| fs::write(out.join("CAMPAIGN.txt"), log).map_err(|e| e.to_string());
    write_log(&log)?;
    let mut all_met = true;
    let selected: Vec<&Entry> = plan
        .entries
        .iter()
        .filter(|e| o.only.is_empty() || o.only.contains(&e.example))
        .collect();
    for entry in &selected {
        let record = out.join(&entry.example);
        if entry.args.iter().any(|a| a == WORK_DIR) {
            // The examples create their own directory inside this one.
            let dir = work.join(&entry.example);
            fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut ab: Vec<String> = Vec::new();
        for (flag, v) in [
            ("--a", o.a.display().to_string()),
            ("--a2", o.a2.display().to_string()),
            ("--b", o.b.display().to_string()),
            ("--package", entry.package.clone()),
            ("--example", entry.example.clone()),
            ("--target-root", o.target_root.display().to_string()),
            ("--out", record.display().to_string()),
            ("--rounds", plan.rounds.to_string()),
            ("--warmup", plan.warmup.to_string()),
        ] {
            ab.push(flag.into());
            ab.push(v);
        }
        if o.invalid_for_timing {
            ab.push("--invalid-for-timing".into());
        }
        for p in &entry.with_bins {
            ab.push("--with-bins".into());
            ab.push(p.clone());
        }
        if let Some((program, rest)) = o.before_run.split_first() {
            ab.push("--before-run".into());
            ab.push(program.clone());
            for x in rest {
                ab.push("--before-run-arg".into());
                ab.push(x.clone());
            }
        }
        ab.push("--".into());
        ab.extend(entry_args(entry, o.runs, &work));
        let run = speed_ab::run(&ab);
        let verdict = fs::read_to_string(record.join("summary.txt"))
            .ok()
            .and_then(|s| {
                s.lines()
                    .find_map(|l| l.strip_prefix("SPEED_AB_VERDICT=").map(str::to_string))
            })
            .unwrap_or_else(|| "NONE".into());
        let check = match check_dir(&record) {
            Ok(_) => "PASS".to_string(),
            Err(e) => format!("FAIL {e}"),
        };
        let wanted = entry.expect.to_ascii_uppercase();
        let met = verdict == wanted && check == "PASS" && (run.is_ok() || wanted != "SAME_OUTPUT");
        all_met &= met;
        let _ = writeln!(
            log,
            "example={} expect={} verdict={verdict} speed_ab={} check={check} met={}",
            entry.example,
            entry.expect,
            if run.is_ok() { "ok" } else { "error" },
            if met { "yes" } else { "no" }
        );
        if let Err(e) = run {
            eprintln!("speed-campaign: {}: {e}", entry.example);
        }
        write_log(&log)?;
    }
    let end = if !all_met {
        "INCOMPLETE"
    } else if selected.len() < plan.entries.len() {
        "PARTIAL"
    } else {
        "COMPLETE"
    };
    let _ = writeln!(log, "SPEED_CAMPAIGN={end}");
    write_log(&log)?;
    print!("{log}");
    if all_met {
        Ok(())
    } else {
        Err("speed-campaign: an example did not meet its planned result".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "source_sha256=0000000000000000000000000000000000000000000000000000000000000000  crates/x/examples/speed_x.rs";
    const HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    fn manifest(rounds: usize, warmup: usize) -> String {
        let mut m = String::new();
        for line in [
            speed_ab::FORMAT,
            speed_ab::PROTOCOL,
            "timing_validity=INVALID_FOR_TIMING",
            "mode=ab",
        ] {
            m.push_str(line);
            m.push('\n');
        }
        let _ = writeln!(m, "package=gel-core example=speed_x rounds={rounds}");
        for line in [
            "input_args=[\"1\"]",
            "with_bins=[]",
            "before_run=none",
            "profile=release: cargo build --locked --offline --release with each tree's own Cargo.toml",
            "rustflags=none",
            "cargo_env=none",
            speed_ab::CLOCK_BOUNDARY,
        ] {
            m.push_str(line);
            m.push('\n');
        }
        let _ = writeln!(m, "warmup_rounds={warmup}");
        for line in [
            speed_ab::ORDER,
            speed_ab::AA_CONTROL,
            "host_os=linux",
            "host_arch=x86_64",
            "host_logical_cpus=4",
            "host_cpu_model=Example CPU w/ graphics",
            "host_mem_total_kib=1000",
            "host_kernel=6.0.0",
            "host_load_start=0.10 0.20 0.30",
            SOURCE,
            "tree=a revision=aaaa working_tree=clean rustc=rustc 1.85.0",
            "tree=a2 revision=aaaa working_tree=clean rustc=rustc 1.85.0",
            "tree=b revision=bbbb working_tree=clean rustc=rustc 1.85.0",
            "placed tree=a crates/x/examples/speed_x.rs",
            "placed tree=a2 crates/x/examples/speed_x.rs",
        ] {
            m.push_str(line);
            m.push('\n');
        }
        for t in Tree::ALL {
            let _ = writeln!(m, "binary_sha256 tree={} {HASH}", t.label());
        }
        m.push_str("aa_same_binary=yes\n");
        for (k, n) in [("warmup_round", warmup), ("round", rounds)] {
            for r in 1..=n {
                let labels: Vec<&str> = speed_ab::order(r).iter().map(|t| t.label()).collect();
                let _ = writeln!(m, "{k}={r} order={}", labels.join(","));
            }
        }
        m.push_str("host_load_end=0.10 0.20 0.30\nrestored=yes\n");
        m
    }

    fn run_text(ns: u128) -> String {
        format!("SPEED_SETUP example=speed_x runs=1\nSPEED name=c run=0 ns={ns}\nSPEED_CHECK sha256={HASH}\n")
    }

    /// A complete record in a new directory; `ns` gives each tree's sample.
    fn record(tag: &str, rounds: usize, warmup: usize) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "gel-speed-campaign-{tag}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.txt"), manifest(rounds, warmup)).unwrap();
        let mut data = [
            TreeData::default(),
            TreeData::default(),
            TreeData::default(),
        ];
        for (i, t) in Tree::ALL.iter().enumerate() {
            for k in 1..=warmup {
                fs::write(dir.join(speed_ab::run_file(true, *t, k)), run_text(999)).unwrap();
            }
            for r in 1..=rounds {
                let text = run_text(100 + (i as u128) * 10 + r as u128);
                fs::write(dir.join(speed_ab::run_file(false, *t, r)), &text).unwrap();
                data[i].add(speed_ab::parse_output(&text).unwrap());
            }
        }
        let (report, _) = speed_ab::report_ab(&data[0], &data[1], &data[2]);
        fs::write(dir.join("summary.txt"), report).unwrap();
        dir
    }

    fn edit_manifest(dir: &Path, f: impl Fn(String) -> String) {
        let path = dir.join("manifest.txt");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, f(text)).unwrap();
    }

    fn without(text: String, prefix: &str) -> String {
        let mut kept = String::new();
        for line in text.lines().filter(|l| !l.starts_with(prefix)) {
            kept.push_str(line);
            kept.push('\n');
        }
        kept
    }

    #[test]
    fn an_intact_record_passes() {
        let dir = record("intact", 3, 1);
        let c = check_dir(&dir).unwrap();
        assert_eq!(
            c,
            Checked {
                cases: 1,
                rounds: 3,
                warmup: 1,
                verdict: "SAME_OUTPUT".into(),
                validity: "INVALID_FOR_TIMING".into()
            }
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn every_protocol_field_is_required() {
        for prefix in [
            "protocol=",
            "timing_validity=",
            "input_args=",
            "profile=",
            "rustflags=",
            "cargo_env=",
            "clock_boundary=",
            "warmup_rounds=",
            "order=",
            "aa_control=",
            "host_os=",
            "host_arch=",
            "host_logical_cpus=",
            "host_cpu_model=",
            "host_mem_total_kib=",
            "host_kernel=",
            "host_load_start=",
            "host_load_end=",
            "aa_same_binary=",
            "restored=",
            "source_sha256=",
            "binary_sha256 tree=a2 ",
            "tree=a2 ",
        ] {
            let dir = record("field", 3, 1);
            edit_manifest(&dir, |t| without(t, prefix));
            assert!(check_dir(&dir).is_err(), "{prefix} missing was accepted");
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn a_record_without_an_aa_control_is_refused() {
        let dir = record("candidate", 3, 1);
        edit_manifest(&dir, |t| t.replace("mode=ab\n", "mode=candidate_only\n"));
        let e = check_dir(&dir).unwrap_err();
        assert!(e.contains("A/A"), "{e}");
        fs::remove_dir_all(&dir).unwrap();
        let dir = record("no-a2", 3, 1);
        fs::remove_file(dir.join("a2-r2.txt")).unwrap();
        assert!(check_dir(&dir).unwrap_err().contains("a2-r2.txt"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rotation_warmup_summary_paths_and_extra_files_are_checked() {
        type Mutant = fn(&Path);
        let mutants: [(&str, Mutant); 9] = [
            ("quoted path argument", |d| {
                edit_manifest(d, |t| {
                    t.replace("input_args=[\"1\"]", "input_args=[\"1\", \"/x\"]")
                })
            }),
            ("order swapped", |d| {
                edit_manifest(d, |t| {
                    t.replace("round=2 order=a2,b,a", "round=2 order=a,b,a2")
                })
            }),
            ("warm-up count", |d| {
                edit_manifest(d, |t| t.replace("warmup_rounds=1", "warmup_rounds=2"))
            }),
            ("summary median", |d| {
                let p = d.join("summary.txt");
                let s = fs::read_to_string(&p)
                    .unwrap()
                    .replacen("a_ns=10", "a_ns=90", 1);
                fs::write(p, s).unwrap();
            }),
            ("absolute path", |d| {
                edit_manifest(d, |t| {
                    t.replace("placed tree=a crates", "placed tree=a /x/crates")
                })
            }),
            ("path in a value", |d| {
                edit_manifest(d, |t| {
                    t.replace("input_args=[\"1\"]", "input_args=[\"1\"] dir=/x")
                })
            }),
            ("unknown key", |d| {
                edit_manifest(d, |t| format!("{t}hostname=x\n"))
            }),
            ("extra file", |d| {
                fs::write(d.join("notes.txt"), "x").unwrap()
            }),
            ("warm-up file", |d| {
                fs::remove_file(d.join("w-b-r1.txt")).unwrap()
            }),
        ];
        for (name, mutate) in mutants {
            let dir = record("mutant", 3, 1);
            mutate(&dir);
            assert!(check_dir(&dir).is_err(), "mutant {name:?} was accepted");
            fs::remove_dir_all(dir).unwrap();
        }
        let dir = record("rounds", 2, 1);
        assert!(check_dir(&dir).unwrap_err().contains("multiple of 3"));
        fs::remove_dir_all(dir).unwrap();
        let dir = record("v1", 3, 1);
        edit_manifest(&dir, |t| {
            t.replace(speed_ab::FORMAT, "FORMAT=GEL_SPEED_AB_1")
        });
        assert!(check_dir(&dir).unwrap_err().contains("historical"));
        fs::remove_dir_all(dir).unwrap();
    }

    const PLAN: &str = "# comment\nPLAN=GEL_SPEED_CAMPAIGN_1\nname=demo\nreruns=docs/x\nbaseline=db9cee36bca3c589fe4f6e0395aaaf89ca8b2b1d\ncandidate=876247f7ba4d9b602e4d3b83a5cb0799852d7925\nrounds=3\nwarmup=1\nexample gel-core speed_crc\nexample gel-bench speed_bench_physics with_bins=gel-bench,gel-physics args=3\nexample gel-source speed_tree_gates expect=different_output args=9,WORK_DIR\nskip gel-reader speed_top1_many reason=no API in the baseline\n";

    #[test]
    fn plans_are_parsed_strictly() {
        let p = parse_plan(PLAN).unwrap();
        assert_eq!(p.name, "demo");
        assert_eq!((p.rounds, p.warmup), (3, 1));
        assert_eq!(p.entries.len(), 3);
        assert_eq!(p.entries[1].with_bins, list("gel-bench,gel-physics"));
        assert_eq!(p.entries[2].expect, "different_output");
        assert_eq!(
            p.skips,
            vec![("speed_top1_many".into(), "no API in the baseline".into())]
        );
        for (from, to) in [
            ("rounds=3", "rounds=4"),
            ("warmup=1", "warmup=0"),
            ("baseline=db9cee36", "baseline=DB9CEE36"),
            (
                "example gel-core speed_crc",
                "example gel-core speed_crc args=/x",
            ),
            (
                "example gel-core speed_crc",
                "example gel-core speed_crc expect=faster",
            ),
            (
                "example gel-core speed_crc",
                "example gel-core speed_tree_gates",
            ),
            ("reason=no API", "because=no API"),
            ("name=demo", "name=demo\nname=again"),
            ("name=demo", "name=demo\nhost=x"),
            ("PLAN=GEL_SPEED_CAMPAIGN_1", "PLAN=OTHER"),
        ] {
            let bad = PLAN.replacen(from, to, 1);
            assert_ne!(bad, PLAN, "{from}");
            assert!(parse_plan(&bad).is_err(), "{to:?} was accepted");
        }
    }

    /// Every committed plan parses, names examples of this checkout and only
    /// examples of the recorded set it re-runs.
    #[test]
    fn committed_plans_match_their_recorded_sets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut plans = 0;
        for entry in fs::read_dir(root.join("docs/speed-campaign")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|x| x.to_str()) != Some("txt") {
                continue;
            }
            plans += 1;
            let plan = parse_plan(&fs::read_to_string(&path).unwrap()).unwrap();
            let recorded = root.join(&plan.reruns);
            assert!(recorded.is_dir(), "{}", plan.reruns);
            for (example, _) in &plan.skips {
                assert!(
                    recorded.join(example).join("manifest.txt").is_file(),
                    "{example}"
                );
            }
            for e in &plan.entries {
                let file = format!("crates/{}/examples/{}.rs", e.package, e.example);
                assert!(root.join(&file).is_file(), "{file}");
                let manifest =
                    fs::read_to_string(recorded.join(&e.example).join("manifest.txt")).unwrap();
                for wanted in [
                    format!(
                        "package={} example={} rounds={}\n",
                        e.package, e.example, plan.rounds
                    ),
                    format!("\nexample_args={:?}\n", e.args),
                    format!("\nwith_bins={:?}\n", e.with_bins),
                ] {
                    assert!(manifest.contains(&wanted), "{}: {wanted:?}", e.example);
                }
            }
        }
        assert_eq!(plans, 4);
    }

    #[test]
    fn a_shrunk_campaign_must_say_it_is_invalid_for_timing() {
        let base: Vec<String> = "--plan p --a a --a2 a2 --b b --target-root t --out o"
            .split(' ')
            .map(str::to_string)
            .collect();
        assert!(campaign_options(&base).is_ok());
        for extra in ["--runs 1", "--only speed_crc"] {
            let mut args = base.clone();
            args.extend(extra.split(' ').map(str::to_string));
            assert!(campaign_options(&args).is_err(), "{extra}");
            args.push("--invalid-for-timing".into());
            assert!(campaign_options(&args).is_ok(), "{extra}");
        }
        let entry = parse_plan(PLAN).unwrap().entries.remove(2);
        let work = Path::new("w");
        assert_eq!(
            entry_args(&entry, Some(1), work),
            vec![
                "1".to_string(),
                work.join("speed_tree_gates").display().to_string()
            ]
        );
        let crc = parse_plan(PLAN).unwrap().entries.remove(0);
        assert_eq!(entry_args(&crc, Some(2), work), vec!["2".to_string()]);
        assert!(entry_args(&crc, None, work).is_empty());
    }
}
