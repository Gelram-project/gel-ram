//! A/A2/B timing comparison of one example between source trees (docs/SPEED-AB.md).
//!
//! A is the baseline, A2 a second copy of the baseline (the A/A control) and B
//! the candidate. The example file is taken from B, with its support directory
//! and every file it includes through `#[path = "..."]`, and placed into A and
//! A2 for the duration of the run, then removed again. Every tree is built in
//! release with its own build directory, the runs rotate their order from
//! round to round, and the report gives per-case medians, the A2/A and A/B
//! ratios and whether the three trees printed the same `SPEED_CHECK` values.
//! The tool states numbers and output identity only; it never states which
//! build is faster.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

const USAGE: &str = "usage: cargo run -p xtask -- speed-ab --a DIR --a2 DIR --b DIR --package PKG --example NAME --target-root DIR --out NEW_DIR [--rounds R] [--with-bins PKG]... [--replace-helpers] [--before-run PROGRAM [--before-run-arg ARG]...] [-- EXAMPLE_ARG...]\n   or: cargo run -p xtask -- speed-ab --candidate-only --b DIR --package PKG --example NAME --target-root DIR --out NEW_DIR [--rounds R] [--with-bins PKG]... [--before-run PROGRAM [--before-run-arg ARG]...] [-- EXAMPLE_ARG...]";
/// The only directory whose files `--replace-helpers` may replace in A and A2.
const HELPER_DIR: &str = "tools";
const DEFAULT_ROUNDS: usize = 3;
const MAX_ROUNDS: usize = 999;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tree {
    A,
    A2,
    B,
}

impl Tree {
    fn label(self) -> &'static str {
        match self {
            Tree::A => "a",
            Tree::A2 => "a2",
            Tree::B => "b",
        }
    }
}

/// Run order of round `round` (1-based): 1 -> B,A,A2; 2 -> A2,B,A; 0 -> A,A2,B (mod 3).
fn order(round: usize) -> [Tree; 3] {
    match round % 3 {
        1 => [Tree::B, Tree::A, Tree::A2],
        2 => [Tree::A2, Tree::B, Tree::A],
        _ => [Tree::A, Tree::A2, Tree::B],
    }
}

#[derive(Debug, PartialEq)]
struct Options {
    a: Option<PathBuf>,
    a2: Option<PathBuf>,
    b: PathBuf,
    package: String,
    example: String,
    target_root: PathBuf,
    out: PathBuf,
    rounds: usize,
    with_bins: Vec<String>,
    before_run: Vec<String>,
    example_args: Vec<String>,
    candidate_only: bool,
    replace_helpers: bool,
}

/// Package and example names become path components and cargo arguments.
fn plain_name(kind: &str, name: &str) -> Result<String, String> {
    if !name.is_empty()
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Ok(name.to_string())
    } else {
        Err(format!("{kind} must be a plain name: {name:?}"))
    }
}

fn options(args: &[String]) -> Result<Options, String> {
    let mut a = None;
    let mut a2 = None;
    let mut b = None;
    let mut package = None;
    let mut example = None;
    let mut target_root = None;
    let mut out = None;
    let mut rounds = None;
    let mut with_bins = Vec::new();
    let mut before_program = None;
    let mut before_args = Vec::new();
    let mut example_args = Vec::new();
    let mut candidate_only = false;
    let mut replace_helpers = false;
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        i += 1;
        if flag == "--" {
            example_args.extend_from_slice(&args[i..]);
            break;
        }
        if flag == "--candidate-only" {
            candidate_only = true;
            continue;
        }
        if flag == "--replace-helpers" {
            if replace_helpers {
                return Err("--replace-helpers given twice".into());
            }
            replace_helpers = true;
            continue;
        }
        let value = args
            .get(i)
            .ok_or_else(|| format!("{flag} needs a value"))?
            .clone();
        i += 1;
        let slot = match flag {
            "--a" => &mut a,
            "--a2" => &mut a2,
            "--b" => &mut b,
            "--package" => &mut package,
            "--example" => &mut example,
            "--target-root" => &mut target_root,
            "--out" => &mut out,
            "--rounds" => &mut rounds,
            "--before-run" => &mut before_program,
            "--with-bins" => {
                with_bins.push(plain_name("--with-bins package", &value)?);
                continue;
            }
            "--before-run-arg" => {
                before_args.push(value);
                continue;
            }
            _ => return Err(format!("unknown option {flag}")),
        };
        if slot.replace(value).is_some() {
            return Err(format!("{flag} given twice"));
        }
    }
    let rounds = match rounds {
        None => DEFAULT_ROUNDS,
        Some(raw) => match raw.parse::<usize>() {
            Ok(n) if (1..=MAX_ROUNDS).contains(&n) => n,
            _ => return Err(format!("--rounds must be 1..={MAX_ROUNDS}")),
        },
    };
    if candidate_only {
        if a.is_some() || a2.is_some() {
            return Err("--candidate-only takes --b only, not --a or --a2".into());
        }
        if replace_helpers {
            return Err("--replace-helpers needs --a and --a2, not --candidate-only".into());
        }
    } else if a.is_none() || a2.is_none() {
        return Err("--a and --a2 are required (or use --candidate-only)".into());
    }
    let before_run = match before_program {
        Some(program) => std::iter::once(program).chain(before_args).collect(),
        None if before_args.is_empty() => Vec::new(),
        None => return Err("--before-run-arg needs --before-run".into()),
    };
    let need =
        |value: Option<String>, flag: &str| value.ok_or_else(|| format!("{flag} is required"));
    Ok(Options {
        a: a.map(PathBuf::from),
        a2: a2.map(PathBuf::from),
        b: PathBuf::from(need(b, "--b")?),
        package: plain_name("--package", &need(package, "--package")?)?,
        example: plain_name("--example", &need(example, "--example")?)?,
        target_root: PathBuf::from(need(target_root, "--target-root")?),
        out: PathBuf::from(need(out, "--out")?),
        rounds,
        with_bins,
        before_run,
        example_args,
        candidate_only,
        replace_helpers,
    })
}

/// `SPEED` samples and `SPEED_CHECK` values of one run's standard output.
#[derive(Debug, Default, PartialEq)]
struct RunOutput {
    samples: Vec<(String, u128)>,
    checks: Vec<String>,
}

fn digits(value: &str) -> Option<u128> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

/// Parse `SPEED name=<case> run=<i> ns=<n>` and `SPEED_CHECK <value>` lines.
/// Other lines (including `SPEED_INPUT`, `SPEED_SETUP`) are ignored; a
/// malformed `SPEED` or `SPEED_CHECK` line is an error, never a silent skip.
fn parse_output(text: &str) -> Result<RunOutput, String> {
    let mut out = RunOutput::default();
    for (index, raw) in text.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let bad = || format!("line {}: malformed line: {line:?}", index + 1);
        let mut tokens = line.split(' ');
        match tokens.next() {
            Some("SPEED") => {
                let fields: Vec<&str> = tokens.collect();
                let [name, run, ns] = fields.as_slice() else {
                    return Err(bad());
                };
                let name = name.strip_prefix("name=").ok_or_else(bad)?;
                let run = run.strip_prefix("run=").ok_or_else(bad)?;
                let ns = ns.strip_prefix("ns=").ok_or_else(bad)?;
                if name.is_empty() || name.contains('=') || digits(run).is_none() {
                    return Err(bad());
                }
                out.samples
                    .push((name.to_string(), digits(ns).ok_or_else(bad)?));
            }
            Some("SPEED_CHECK") => {
                let value = line["SPEED_CHECK".len()..].trim();
                if value.is_empty() {
                    return Err(bad());
                }
                out.checks.push(value.to_string());
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Lower median: the ((n + 1) / 2)-th smallest value, as `sort -n | sed -n "$(((n+1)/2))p"`.
fn median(values: &[u128]) -> Option<u128> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Some(sorted[(sorted.len() - 1) / 2])
}

/// `num / den` with exactly three decimals, rounded half up, integers only.
fn ratio(num: u128, den: u128) -> String {
    if den == 0 {
        return "NA".into();
    }
    let scaled = num
        .checked_mul(2000)
        .and_then(|n| n.checked_add(den))
        .and_then(|n| den.checked_mul(2).map(|d| n / d));
    match scaled {
        Some(q) => format!("{}.{:03}", q / 1000, q % 1000),
        None => "NA".into(),
    }
}

#[derive(Debug, Default)]
struct TreeData {
    samples: BTreeMap<String, Vec<u128>>,
    checks: BTreeSet<String>,
    speed_lines: usize,
}

impl TreeData {
    fn add(&mut self, run: RunOutput) {
        self.speed_lines += run.samples.len();
        for (name, ns) in run.samples {
            self.samples.entry(name).or_default().push(ns);
        }
        self.checks.extend(run.checks);
    }

    fn median(&self, case: &str) -> Option<u128> {
        self.samples.get(case).and_then(|v| median(v))
    }

    fn count(&self, case: &str) -> usize {
        self.samples.get(case).map_or(0, Vec::len)
    }

    fn has_data(&self) -> bool {
        self.speed_lines > 0 && !self.checks.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    SameOutput,
    DifferentOutput,
    NoData,
    CandidateOnly,
}

impl Verdict {
    fn word(self) -> &'static str {
        match self {
            Verdict::SameOutput => "SAME_OUTPUT",
            Verdict::DifferentOutput => "DIFFERENT_OUTPUT",
            Verdict::NoData => "NO_DATA",
            Verdict::CandidateOnly => "CANDIDATE_ONLY",
        }
    }
}

/// SAME_OUTPUT only when every tree printed at least one SPEED line and the
/// same non-empty set of SPEED_CHECK values.
fn verdict(trees: &[&TreeData]) -> Verdict {
    if trees.is_empty() || trees.iter().any(|t| !t.has_data()) {
        Verdict::NoData
    } else if trees.windows(2).all(|w| w[0].checks == w[1].checks) {
        Verdict::SameOutput
    } else {
        Verdict::DifferentOutput
    }
}

fn ns(value: Option<u128>) -> String {
    value.map_or_else(|| "NA".into(), |v| v.to_string())
}

fn check_lines(report: &mut String, tree: Tree, data: &TreeData) {
    for value in &data.checks {
        let _ = writeln!(report, "SPEED_AB_CHECK tree={} {value}", tree.label());
    }
}

fn report_ab(a: &TreeData, a2: &TreeData, b: &TreeData) -> (String, Verdict) {
    let mut report = String::new();
    let cases: BTreeSet<&String> = a
        .samples
        .keys()
        .chain(a2.samples.keys())
        .chain(b.samples.keys())
        .collect();
    for case in cases {
        let (ma, ma2, mb) = (a.median(case), a2.median(case), b.median(case));
        let pair = |n: Option<u128>, d: Option<u128>| match (n, d) {
            (Some(n), Some(d)) => ratio(n, d),
            _ => "NA".into(),
        };
        let counts = [a.count(case), a2.count(case), b.count(case)];
        let runs = if counts[0] == counts[1] && counts[1] == counts[2] {
            counts[0].to_string()
        } else {
            format!("{}/{}/{}", counts[0], counts[1], counts[2])
        };
        let _ = writeln!(
            report,
            "SPEED_AB case={case} a_ns={} a2_ns={} b_ns={} aa={} gain={} runs={runs}",
            ns(ma),
            ns(ma2),
            ns(mb),
            pair(ma2, ma),
            pair(ma, mb),
        );
    }
    check_lines(&mut report, Tree::A, a);
    check_lines(&mut report, Tree::A2, a2);
    check_lines(&mut report, Tree::B, b);
    let verdict = verdict(&[a, a2, b]);
    let _ = writeln!(report, "SPEED_AB_VERDICT={}", verdict.word());
    (report, verdict)
}

fn report_candidate(b: &TreeData) -> (String, Verdict) {
    let mut report = String::new();
    for case in b.samples.keys() {
        let _ = writeln!(
            report,
            "SPEED_AB case={case} b_ns={} runs={}",
            ns(b.median(case)),
            b.count(case)
        );
    }
    check_lines(&mut report, Tree::B, b);
    let verdict = if b.has_data() {
        Verdict::CandidateOnly
    } else {
        Verdict::NoData
    };
    let _ = writeln!(report, "SPEED_AB_VERDICT={}", verdict.word());
    (report, verdict)
}

/// Why a file is carried from B into A and A2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Origin {
    /// The example file itself.
    Example,
    /// A file of the example's `examples/support/` directory.
    Support,
    /// A file included through `#[path = "..."]`, directly or through another
    /// included file.
    Included,
}

/// One file taken from B: its path relative to the tree root and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Carried {
    rel: PathBuf,
    bytes: Vec<u8>,
    origin: Origin,
}

impl Carried {
    /// Only an included helper under `tools/` may be replaced, and only on request.
    fn replaceable(&self, replace_helpers: bool) -> bool {
        replace_helpers && self.origin == Origin::Included && self.rel.starts_with(HELPER_DIR)
    }
}

/// What placing one carried file into a tree does.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    /// The destination is missing: create it, remove it afterwards.
    Create,
    /// The destination holds exactly these bytes: leave it alone.
    Keep,
    /// A helper under `tools/` holds other bytes: replace it, and write these
    /// original bytes back afterwards.
    Replace(Vec<u8>),
}

/// Files placed into a tree by this tool, removed again on `restore` or drop;
/// files it replaced get their original bytes back.
#[derive(Debug, Default)]
struct Placed {
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
    replaced: Vec<(PathBuf, Vec<u8>)>,
}

impl Placed {
    /// Refuse before anything is written: a destination that exists must be a
    /// regular file with exactly the source bytes, or a replaceable helper.
    fn precheck(dest: &Path, file: &Carried, replace_helpers: bool) -> Result<Step, String> {
        match fs::symlink_metadata(dest) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Step::Create),
            Err(e) => Err(format!("{}: {e}", dest.display())),
            Ok(meta) if !meta.file_type().is_file() => Err(format!(
                "{} exists and is not a regular file",
                dest.display()
            )),
            Ok(_) => {
                let existing = fs::read(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
                if existing == file.bytes {
                    Ok(Step::Keep)
                } else if file.replaceable(replace_helpers) {
                    Ok(Step::Replace(existing))
                } else if file.origin == Origin::Included {
                    Err(format!(
                        "{} already exists with different content; refusing to replace it: the trees would build different included code (a file under {HELPER_DIR}/ may be replaced with --replace-helpers)",
                        dest.display()
                    ))
                } else {
                    Err(format!(
                        "{} already exists with different content; refusing to replace it",
                        dest.display()
                    ))
                }
            }
        }
    }

    /// The step for every carried file in `root`, in order; nothing is written.
    fn plan(root: &Path, files: &[Carried], replace_helpers: bool) -> Result<Vec<Step>, String> {
        files
            .iter()
            .map(|file| Self::precheck(&root.join(&file.rel), file, replace_helpers))
            .collect()
    }

    /// Place every carried file into `root`; all destinations are checked
    /// before the first byte is written.
    #[cfg(test)]
    fn place_all(
        &mut self,
        root: &Path,
        files: &[Carried],
        replace_helpers: bool,
    ) -> Result<(), String> {
        let plan = Self::plan(root, files, replace_helpers)?;
        self.apply(root, files, &plan)
    }

    /// Carry out `plan` (from `Self::plan` for the same `root` and `files`).
    /// Every file is recorded before it is written, so `restore` undoes a
    /// partial write as well.
    fn apply(&mut self, root: &Path, files: &[Carried], plan: &[Step]) -> Result<(), String> {
        if plan.len() != files.len() {
            return Err("placement plan does not match the carried files".into());
        }
        for (file, step) in files.iter().zip(plan) {
            let dest = root.join(&file.rel);
            match step {
                Step::Keep => {}
                Step::Replace(original) => {
                    self.replaced.push((dest.clone(), original.clone()));
                    write_existing(&dest, &file.bytes)?;
                }
                Step::Create => self.create(root, &dest, &file.bytes)?,
            }
        }
        Ok(())
    }

    /// Create `dest` and the directories it needs; each is recorded for
    /// `restore` as soon as it exists.
    fn create(&mut self, root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), String> {
        let parent = dest.parent().ok_or("destination without a parent")?;
        let mut missing = Vec::new();
        let mut dir = parent;
        while !dir.exists() {
            missing.push(dir.to_path_buf());
            dir = dir
                .parent()
                .ok_or("destination without an existing ancestor")?;
        }
        if !dir.starts_with(root) {
            return Err(format!("{} is outside the tree", dest.display()));
        }
        for d in missing.into_iter().rev() {
            fs::create_dir(&d).map_err(|e| format!("{}: {e}", d.display()))?;
            self.dirs.push(d);
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest)
            .map_err(|e| format!("{}: {e}", dest.display()))?;
        self.files.push(dest.to_path_buf());
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| format!("{}: {e}", dest.display()))
    }

    /// Write the original bytes of replaced files back, remove placed files,
    /// then the directories this tool created (newest first).
    fn restore(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        for (file, original) in self.replaced.drain(..).rev() {
            if let Err(e) = write_existing(&file, &original) {
                errors.push(e);
            }
        }
        for file in self.files.drain(..).rev() {
            if let Err(e) = fs::remove_file(&file) {
                errors.push(format!("{}: {e}", file.display()));
            }
        }
        for dir in self.dirs.drain(..).rev() {
            if let Err(e) = fs::remove_dir(&dir) {
                errors.push(format!("{}: {e}", dir.display()));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "could not restore the trees: {}",
                errors.join("; ")
            ))
        }
    }
}

impl Drop for Placed {
    fn drop(&mut self) {
        if let Err(e) = self.restore() {
            eprintln!("speed-ab: {e}");
        }
    }
}

fn canonical(path: &Path, what: &str) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("{what} {}: {e}", path.display()))
}

/// The build directory name of one tree: its label and the first 16 hex
/// digits of the SHA-256 of its canonical path. Cargo treats a path package
/// as fresh when the sources named in its last build are unchanged, so two
/// trees sharing one build directory could run the other tree's library code;
/// a directory per tree path rules that out.
fn target_name(tree: Tree, root: &Path) -> String {
    let path = root.to_string_lossy();
    let hash = gel_source::hex(&gel_source::digest(path.as_bytes()));
    format!("{}-{}", tree.label(), &hash[..16])
}

/// The canonical form `path` will have once created: its nearest existing
/// ancestor canonicalized, with the missing components appended.
fn planned_canonical(path: &Path) -> Result<PathBuf, String> {
    let whole = std::path::absolute(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut base = whole.as_path();
    let mut missing = Vec::new();
    while !base.exists() {
        let name = base
            .file_name()
            .ok_or_else(|| format!("{}: no existing ancestor", path.display()))?;
        missing.push(name.to_owned());
        base = base
            .parent()
            .ok_or_else(|| format!("{}: no existing ancestor", path.display()))?;
    }
    let mut out = canonical(base, "--target-root ancestor")?;
    for name in missing.iter().rev() {
        out.push(name);
    }
    Ok(out)
}

/// Relative directory of PKG inside a tree: `crates/PKG` or `PKG`.
fn package_dir(tree: &Path, package: &str) -> Result<PathBuf, String> {
    [Path::new("crates").join(package), PathBuf::from(package)]
        .into_iter()
        .find(|rel| tree.join(rel).join("Cargo.toml").is_file())
        .ok_or_else(|| format!("package {package} not found in {}", tree.display()))
}

fn uses_support(source: &str) -> bool {
    source.contains("\"support/") || source.contains("mod support")
}

fn collect_files(root: &Path, rel: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String> {
    let dir = root.join(rel);
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|entry| entry.map(|e| e.file_name()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    entries.sort();
    for name in entries {
        let rel = rel.join(name);
        let path = root.join(&rel);
        let meta = fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if meta.file_type().is_dir() {
            collect_files(root, &rel, out)?;
        } else if meta.file_type().is_file() {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.push((rel, bytes));
        } else {
            return Err(format!("{} is not a regular file", path.display()));
        }
    }
    Ok(())
}

/// Overwrite (or recreate) an existing file with `bytes`, then sync it.
fn write_existing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Index just after the string literal whose opening quote is at `at`.
fn skip_string(s: &[u8], at: usize) -> Result<usize, String> {
    let mut i = at + 1;
    while i < s.len() {
        match s[i] {
            b'\\' => i += 2,
            b'"' => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err("unterminated string literal".into())
}

/// When a raw string literal (`r"`, `r#"`, `br#"`, ...) starts at `at`, the
/// index just after it.
fn skip_raw_string(s: &[u8], at: usize) -> Result<Option<usize>, String> {
    let start = if s[at] == b'b' { at + 1 } else { at };
    if s.get(start) != Some(&b'r') || (at > 0 && is_ident(s[at - 1])) {
        return Ok(None);
    }
    let mut i = start + 1;
    let mut hashes = 0;
    while s.get(i) == Some(&b'#') {
        hashes += 1;
        i += 1;
    }
    if s.get(i) != Some(&b'"') {
        return Ok(None);
    }
    i += 1;
    while i < s.len() {
        let closes = |at: usize| {
            s.get(at + 1..at + 1 + hashes)
                .is_some_and(|tail| tail.iter().all(|b| *b == b'#'))
        };
        if s[i] == b'"' && closes(i) {
            return Ok(Some(i + 1 + hashes));
        }
        i += 1;
    }
    Err("unterminated raw string literal".into())
}

/// Index just after a comment starting at `at`, if one starts there.
fn skip_comment(s: &[u8], at: usize) -> Result<Option<usize>, String> {
    match (s[at], s.get(at + 1).copied()) {
        (b'/', Some(b'/')) => Ok(Some(
            s[at..]
                .iter()
                .position(|b| *b == b'\n')
                .map_or(s.len(), |p| at + p + 1),
        )),
        (b'/', Some(b'*')) => {
            let (mut i, mut depth) = (at + 2, 1);
            while i < s.len() {
                if s[i] == b'/' && s.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    i += 2;
                } else if s[i] == b'*' && s.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        return Ok(Some(i));
                    }
                } else {
                    i += 1;
                }
            }
            Err("unterminated block comment".into())
        }
        _ => Ok(None),
    }
}

/// Index just after a character literal starting at `at`; `at + 1` for a
/// lifetime or label.
fn skip_char(s: &[u8], at: usize) -> usize {
    if s.get(at + 1) == Some(&b'\\') {
        // The escaped character itself may be a quote: search after it.
        return s
            .get(at + 3..)
            .and_then(|rest| rest.iter().position(|b| *b == b'\''))
            .map_or(s.len(), |p| at + 3 + p + 1);
    }
    let width = match s.get(at + 1) {
        Some(b) if *b < 0x80 => 1,
        Some(b) if *b >= 0xf0 => 4,
        Some(b) if *b >= 0xe0 => 3,
        Some(_) => 2,
        None => return s.len(),
    };
    if s.get(at + 1 + width) == Some(&b'\'') {
        at + 2 + width
    } else {
        at + 1
    }
}

/// The body of the attribute whose `[` is at `open`, and the index after its `]`.
fn attribute_body(s: &[u8], open: usize) -> Result<(&[u8], usize), String> {
    let mut depth = 0usize;
    let mut i = open;
    while i < s.len() {
        match s[i] {
            b'"' => {
                i = skip_string(s, i)?;
                continue;
            }
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&s[open + 1..i], i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err("unterminated attribute".into())
}

/// The literal of a `path = "..."` attribute body, if it is one.
fn path_literal(body: &[u8]) -> Result<Option<String>, String> {
    let text = String::from_utf8_lossy(body);
    let text = text.trim();
    let Some(rest) = text.strip_prefix("path") else {
        if text.starts_with("cfg_attr") && text.contains("path") {
            return Err(format!(
                "#[{text}]: a conditional path attribute is not supported"
            ));
        }
        return Ok(None);
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix('=') else {
        return Ok(None);
    };
    let rest = rest.trim();
    let literal = rest
        .strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .filter(|l| !l.is_empty() && !l.contains('"') && !l.contains('\\'))
        .ok_or_else(|| format!("#[{text}]: only a plain string literal is supported"))?;
    Ok(Some(literal.to_string()))
}

/// Every `#[path = "..."]` literal of a Rust source, in order. Comments,
/// string and character literals are skipped, so a path quoted in
/// documentation is not an include.
fn path_includes(source: &str) -> Result<Vec<String>, String> {
    let s = source.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if let Some(next) = skip_comment(s, i)? {
            i = next;
            continue;
        }
        match s[i] {
            b'"' => i = skip_string(s, i)?,
            b'r' | b'b' => match skip_raw_string(s, i)? {
                Some(next) => i = next,
                None => i += 1,
            },
            b'\'' => i = skip_char(s, i),
            b'#' => {
                let mut j = i + 1;
                while j < s.len() && s[j].is_ascii_whitespace() {
                    j += 1;
                }
                if s.get(j) == Some(&b'!') {
                    j += 1;
                    while j < s.len() && s[j].is_ascii_whitespace() {
                        j += 1;
                    }
                }
                if s.get(j) == Some(&b'[') {
                    let (body, next) = attribute_body(s, j)?;
                    if let Some(literal) = path_literal(body)? {
                        found.push(literal);
                    }
                    i = next;
                } else {
                    i = j;
                }
            }
            _ => i += 1,
        }
    }
    Ok(found)
}

/// `literal` resolved against the directory of `including` (both relative to
/// the tree root), without leaving the tree.
fn resolve_include(including: &Path, literal: &str) -> Result<PathBuf, String> {
    let joined = including.parent().unwrap_or(Path::new("")).join(literal);
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Normal(name) => out.push(name),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(format!(
                        "{}: #[path = {literal:?}] leaves the tree",
                        including.display()
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "{}: #[path = {literal:?}] is not a relative path",
                    including.display()
                ))
            }
        }
    }
    Ok(out)
}

/// The bytes of the regular file `rel` of tree `root`.
fn read_regular(root: &Path, rel: &Path, what: &str) -> Result<Vec<u8>, String> {
    let path = root.join(rel);
    let meta =
        fs::symlink_metadata(&path).map_err(|e| format!("{what} {}: {e}", path.display()))?;
    if !meta.file_type().is_file() {
        return Err(format!("{what} {} is not a regular file", path.display()));
    }
    fs::read(&path).map_err(|e| format!("{what} {}: {e}", path.display()))
}

/// The example file, its `examples/support/` directory when it uses one, and
/// every file included through `#[path = "..."]` (resolved relative to the
/// including file, followed through included files), from B.
fn example_files(b: &Path, package: &str, example: &str) -> Result<Vec<Carried>, String> {
    let examples = package_dir(b, package)?.join("examples");
    let rel = examples.join(format!("{example}.rs"));
    let path = b.join(&rel);
    let bytes = fs::read(&path).map_err(|e| format!("example {}: {e}", path.display()))?;
    let support = uses_support(&String::from_utf8_lossy(&bytes));
    let mut files = vec![Carried {
        rel,
        bytes,
        origin: Origin::Example,
    }];
    if support {
        let mut found = Vec::new();
        collect_files(b, &examples.join("support"), &mut found)?;
        files.extend(found.into_iter().map(|(rel, bytes)| Carried {
            rel,
            bytes,
            origin: Origin::Support,
        }));
    }
    let mut seen: BTreeSet<PathBuf> = files.iter().map(|f| f.rel.clone()).collect();
    let mut next = 0;
    while next < files.len() {
        let including = files[next].rel.clone();
        let source = String::from_utf8_lossy(&files[next].bytes).into_owned();
        next += 1;
        let literals =
            path_includes(&source).map_err(|e| format!("{}: {e}", including.display()))?;
        for literal in literals {
            let rel = resolve_include(&including, &literal)?;
            if seen.insert(rel.clone()) {
                let bytes = read_regular(b, &rel, "included file")?;
                files.push(Carried {
                    rel,
                    bytes,
                    origin: Origin::Included,
                });
            }
        }
    }
    Ok(files)
}

fn hook(before_run: &[String]) -> Result<(), String> {
    let Some((program, args)) = before_run.split_first() else {
        return Ok(());
    };
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::io::stderr()))
        .status()
        .map_err(|e| format!("--before-run {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("--before-run {program} failed: {status}"))
    }
}

fn build_args<'a>(package: &'a str, example: &'a str, with_bins: &'a [String]) -> Vec<&'a str> {
    let mut args = vec![
        "build",
        "--locked",
        "--offline",
        "--release",
        "-p",
        package,
        "--example",
        example,
    ];
    for bin_package in with_bins {
        args.extend(["-p", bin_package.as_str()]);
    }
    if !with_bins.is_empty() {
        args.push("--bins");
    }
    args
}

fn first_line(program: &str, arg: &str, dir: &Path) -> String {
    Command::new(program)
        .arg(arg)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(str::to_string))
        .unwrap_or_else(|| "UNAVAILABLE".into())
}

struct Site {
    tree: Tree,
    root: PathBuf,
    target: PathBuf,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let o = options(args).map_err(|e| format!("{USAGE}\nspeed-ab: {e}"))?;
    let mut roots = vec![(Tree::B, canonical(&o.b, "--b")?)];
    if let (Some(a), Some(a2)) = (&o.a, &o.a2) {
        roots.insert(0, (Tree::A2, canonical(a2, "--a2")?));
        roots.insert(0, (Tree::A, canonical(a, "--a")?));
    }
    for (i, (_, x)) in roots.iter().enumerate() {
        if roots[..i].iter().any(|(_, y)| y == x) {
            return Err("--a, --a2 and --b must be three different trees".into());
        }
    }
    let inside_a_tree = |p: &Path| roots.iter().any(|(_, r)| p.starts_with(r));
    let out_parent = o
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if inside_a_tree(&canonical(out_parent, "--out parent")?) {
        return Err("--out must be outside every tree".into());
    }
    // Refuse before creating anything, so a refused run leaves no directory behind.
    if inside_a_tree(&planned_canonical(&o.target_root)?) {
        return Err("--target-root must be outside every tree".into());
    }
    fs::create_dir_all(&o.target_root)
        .map_err(|e| format!("--target-root {}: {e}", o.target_root.display()))?;
    let target_root = canonical(&o.target_root, "--target-root")?;
    if inside_a_tree(&target_root) {
        return Err("--target-root must be outside every tree".into());
    }
    let (_, b_root) = roots.last().ok_or("missing --b")?;
    let files = example_files(b_root, &o.package, &o.example)?;
    fs::create_dir(&o.out).map_err(|e| format!("new --out directory required: {e}"))?;
    let out = canonical(&o.out, "--out")?;

    let sites: Vec<Site> = roots
        .iter()
        .map(|(tree, root)| Site {
            tree: *tree,
            root: root.clone(),
            target: target_root.join(target_name(*tree, root)),
        })
        .collect();

    let mut manifest = String::new();
    let _ = writeln!(manifest, "FORMAT=GEL_SPEED_AB_1");
    let mode = if o.candidate_only {
        "candidate_only"
    } else {
        "ab"
    };
    let _ = writeln!(manifest, "mode={mode}");
    let _ = writeln!(
        manifest,
        "package={} example={} rounds={}",
        o.package, o.example, o.rounds
    );
    let _ = writeln!(manifest, "example_args={:?}", o.example_args);
    let _ = writeln!(manifest, "with_bins={:?}", o.with_bins);
    let _ = writeln!(manifest, "before_run={:?}", o.before_run);
    if o.replace_helpers {
        let _ = writeln!(manifest, "replace_helpers=yes");
    }
    for file in &files {
        let _ = writeln!(
            manifest,
            "source_sha256={}  {}",
            gel_source::hex(&gel_source::digest(&file.bytes)),
            file.rel.display()
        );
    }

    // Tree state is recorded before anything is placed into a tree.
    for site in &sites {
        let _ = writeln!(
            manifest,
            "tree={} root={} {} rustc={}",
            site.tree.label(),
            site.root.display(),
            super::bench_compare::revision(&site.root),
            first_line("rustc", "--version", &site.root)
        );
    }
    // Every destination in A and A2 is checked before anything is written to
    // either tree, so a refusal leaves both trees untouched.
    let mut plans = Vec::new();
    for site in sites.iter().filter(|s| s.tree != Tree::B) {
        plans.push((site, Placed::plan(&site.root, &files, o.replace_helpers)?));
    }
    // The original bytes of every helper to be replaced are kept in --out
    // before the first replacement.
    for (site, plan) in &plans {
        for (file, step) in files.iter().zip(plan) {
            if let Step::Replace(original) = step {
                let backup = out
                    .join(format!("replaced-{}", site.tree.label()))
                    .join(&file.rel);
                let parent = backup.parent().ok_or("backup path without a parent")?;
                fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
                fs::write(&backup, original).map_err(|e| format!("{}: {e}", backup.display()))?;
            }
        }
    }
    // Place the candidate's files into A and A2; the guard removes them (and
    // writes replaced helpers back) on every exit path, including errors.
    let mut placed = Placed::default();
    for (site, plan) in &plans {
        let (files_before, dirs_before, replaced_before) =
            (placed.files.len(), placed.dirs.len(), placed.replaced.len());
        placed.apply(&site.root, &files, plan)?;
        let label = site.tree.label();
        for dir in &placed.dirs[dirs_before..] {
            let _ = writeln!(manifest, "placed_dir tree={label} {}", dir.display());
        }
        for file in &placed.files[files_before..] {
            let _ = writeln!(manifest, "placed tree={label} {}", file.display());
        }
        for (file, original) in &placed.replaced[replaced_before..] {
            let _ = writeln!(
                manifest,
                "replaced tree={label} {} original_sha256={}",
                file.display(),
                gel_source::hex(&gel_source::digest(original))
            );
        }
    }
    fs::write(out.join("manifest.txt"), &manifest).map_err(|e| e.to_string())?;

    for site in &sites {
        hook(&o.before_run)?;
        let status = Command::new("cargo")
            .args(build_args(&o.package, &o.example, &o.with_bins))
            .env("CARGO_TARGET_DIR", &site.target)
            .current_dir(&site.root)
            .stdin(Stdio::null())
            .status()
            .map_err(|e| format!("cargo build could not start: {e}"))?;
        if !status.success() {
            return Err(format!(
                "cargo build failed in tree {}: {status}",
                site.tree.label()
            ));
        }
    }

    let mut data: BTreeMap<Tree, TreeData> = BTreeMap::new();
    for round in 1..=o.rounds {
        let trees: Vec<Tree> = if o.candidate_only {
            vec![Tree::B]
        } else {
            order(round).to_vec()
        };
        let labels: Vec<&str> = trees.iter().map(|t| t.label()).collect();
        let _ = writeln!(manifest, "round={round} order={}", labels.join(","));
        for tree in trees {
            let site = sites
                .iter()
                .find(|s| s.tree == tree)
                .ok_or("missing tree")?;
            hook(&o.before_run)?;
            let name = format!("{}{}", o.example, std::env::consts::EXE_SUFFIX);
            let binary = site.target.join("release").join("examples").join(name);
            let output = Command::new(&binary)
                .args(&o.example_args)
                .current_dir(&site.root)
                .stdin(Stdio::null())
                .stderr(Stdio::inherit())
                .output()
                .map_err(|e| format!("{}: {e}", binary.display()))?;
            let file = out.join(format!("{}-r{round}.txt", tree.label()));
            fs::write(&file, &output.stdout).map_err(|e| format!("{}: {e}", file.display()))?;
            if !output.status.success() {
                return Err(format!(
                    "example run failed in tree {} round {round}: {}",
                    tree.label(),
                    output.status
                ));
            }
            let parsed = parse_output(&String::from_utf8_lossy(&output.stdout))
                .map_err(|e| format!("{}: {e}", file.display()))?;
            data.entry(tree).or_default().add(parsed);
        }
    }
    placed.restore()?;
    let _ = writeln!(manifest, "restored=yes");
    fs::write(out.join("manifest.txt"), &manifest).map_err(|e| e.to_string())?;

    let empty = TreeData::default();
    let get = |t: Tree| data.get(&t).unwrap_or(&empty);
    let (report, verdict) = if o.candidate_only {
        report_candidate(get(Tree::B))
    } else {
        report_ab(get(Tree::A), get(Tree::A2), get(Tree::B))
    };
    print!("{report}");
    fs::write(out.join("summary.txt"), &report).map_err(|e| e.to_string())?;
    match verdict {
        Verdict::SameOutput | Verdict::CandidateOnly => Ok(()),
        Verdict::DifferentOutput => {
            Err("speed-ab: the trees printed different SPEED_CHECK values".into())
        }
        Verdict::NoData => {
            Err("speed-ab: a tree printed no SPEED line or no SPEED_CHECK value".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    fn data(text: &str) -> TreeData {
        let mut d = TreeData::default();
        d.add(parse_output(text).unwrap());
        d
    }

    fn scratch(tag: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("gel-speed-ab-{tag}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_tree_path_gets_its_own_build_directory() {
        let one = target_name(Tree::B, Path::new("/w/one"));
        let two = target_name(Tree::B, Path::new("/w/two"));
        assert_ne!(one, two);
        assert_eq!(one, target_name(Tree::B, Path::new("/w/one")));
        assert!(one.starts_with(&format!("{}-", Tree::B.label())));
        assert_ne!(
            target_name(Tree::A, Path::new("/w/one")),
            target_name(Tree::A2, Path::new("/w/one"))
        );
    }

    #[test]
    fn a_planned_target_root_is_judged_without_being_created() {
        let root = scratch("planned");
        let planned = root.join("a").join("b");
        assert_eq!(
            planned_canonical(&planned).unwrap(),
            root.canonicalize().unwrap().join("a").join("b")
        );
        assert!(!root.join("a").exists());
        assert_eq!(
            planned_canonical(&root).unwrap(),
            root.canonicalize().unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn listing(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        collect_files(root, Path::new(""), &mut out).unwrap();
        out
    }

    #[test]
    fn speed_and_check_lines_are_parsed_and_other_lines_ignored() {
        let text = "SPEED_INPUT bytes=8 runs=2\nSPEED name=crc_1 run=0 ns=30\nnoise\nSPEED name=crc_1 run=1 ns=10\r\nSPEEDY name=x\nSPEED_CHECK sha256=ab12\n";
        let out = parse_output(text).unwrap();
        assert_eq!(
            out.samples,
            vec![("crc_1".to_string(), 30), ("crc_1".to_string(), 10)]
        );
        assert_eq!(out.checks, vec!["sha256=ab12".to_string()]);
        let paired = parse_output("SPEED_CHECK equal=true crc=00ff\n").unwrap();
        assert_eq!(paired.checks, vec!["equal=true crc=00ff".to_string()]);
        assert_eq!(parse_output("").unwrap(), RunOutput::default());
    }

    #[test]
    fn malformed_speed_lines_are_errors() {
        for bad in [
            "SPEED",
            "SPEED name=x run=0",
            "SPEED name=x run=0 ns=1 extra=2",
            "SPEED name= run=0 ns=1",
            "SPEED name=a=b run=0 ns=1",
            "SPEED name=x run=a ns=1",
            "SPEED name=x run=0 ns=",
            "SPEED name=x run=0 ns=-1",
            "SPEED name=x run=0 ns=1.5",
            "SPEED run=0 name=x ns=1",
            "SPEED  name=x run=0 ns=1",
            "SPEED_CHECK",
            "SPEED_CHECK   ",
        ] {
            assert!(parse_output(bad).is_err(), "{bad:?} was accepted");
        }
    }

    #[test]
    fn median_is_the_lower_middle_value() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[7]), Some(7));
        assert_eq!(median(&[5, 1, 3]), Some(3));
        assert_eq!(median(&[4, 1, 3, 2]), Some(2));
        assert_eq!(median(&[10, 20]), Some(10));
        // Same index as `sort -n | sed -n "$(((n+1)/2))p"` for every n.
        for n in 1..=12u128 {
            let values: Vec<u128> = (1..=n).rev().collect();
            let line = (n + 1) / 2;
            assert_eq!(median(&values), Some(line));
        }
    }

    #[test]
    fn rounds_rotate_the_run_order() {
        assert_eq!(order(1), [Tree::B, Tree::A, Tree::A2]);
        assert_eq!(order(2), [Tree::A2, Tree::B, Tree::A]);
        assert_eq!(order(3), [Tree::A, Tree::A2, Tree::B]);
        assert_eq!(order(4), order(1));
        for r in 1..=6 {
            let mut trees = order(r).to_vec();
            trees.sort();
            assert_eq!(trees, vec![Tree::A, Tree::A2, Tree::B]);
        }
        // Each tree takes each position once in three consecutive rounds.
        for position in 0..3 {
            let mut seen: Vec<Tree> = (1..=3).map(|r| order(r)[position]).collect();
            seen.sort();
            assert_eq!(seen, vec![Tree::A, Tree::A2, Tree::B]);
        }
    }

    #[test]
    fn ratios_have_three_decimals_from_integers() {
        assert_eq!(ratio(1, 1), "1.000");
        assert_eq!(ratio(1, 2), "0.500");
        assert_eq!(ratio(2, 1), "2.000");
        assert_eq!(ratio(1, 3), "0.333");
        assert_eq!(ratio(2, 3), "0.667");
        assert_eq!(ratio(1, 2000), "0.001");
        assert_eq!(ratio(1, 2001), "0.000");
        assert_eq!(ratio(0, 5), "0.000");
        assert_eq!(ratio(123_456_789, 100_000_000), "1.235");
        assert_eq!(ratio(5, 0), "NA");
        assert_eq!(ratio(u128::MAX, 1), "NA");
    }

    #[test]
    fn verdict_needs_data_in_every_tree_and_equal_check_sets() {
        let one = "SPEED name=c run=0 ns=5\nSPEED_CHECK sha256=aa\n";
        let other = "SPEED name=c run=0 ns=5\nSPEED_CHECK sha256=bb\n";
        let (a, a2, b) = (data(one), data(one), data(one));
        assert_eq!(verdict(&[&a, &a2, &b]), Verdict::SameOutput);
        assert_eq!(verdict(&[&a, &a2, &data(other)]), Verdict::DifferentOutput);
        assert_eq!(verdict(&[&data(other), &a2, &b]), Verdict::DifferentOutput);
        let no_speed = data("SPEED_CHECK sha256=aa\n");
        assert_eq!(verdict(&[&a, &a2, &no_speed]), Verdict::NoData);
        let no_check = data("SPEED name=c run=0 ns=5\n");
        assert_eq!(verdict(&[&no_check, &no_check, &no_check]), Verdict::NoData);
        assert_eq!(verdict(&[&a, &TreeData::default(), &b]), Verdict::NoData);
        assert_eq!(verdict(&[]), Verdict::NoData);
        // Sets, not sequences: order and repetition do not matter, an extra value does.
        let mut twice = data(one);
        twice.add(parse_output(other).unwrap());
        twice.add(parse_output(one).unwrap());
        let mut swapped = data(other);
        swapped.add(parse_output(one).unwrap());
        assert_eq!(verdict(&[&twice, &swapped, &swapped]), Verdict::SameOutput);
        assert_eq!(verdict(&[&twice, &a2, &swapped]), Verdict::DifferentOutput);
    }

    #[test]
    fn report_lines_are_exact() {
        let a = data("SPEED name=x run=0 ns=100\nSPEED name=x run=1 ns=300\nSPEED name=x run=2 ns=200\nSPEED name=y run=0 ns=9\nSPEED_CHECK sha256=aa\n");
        let a2 = data("SPEED name=x run=0 ns=210\nSPEED name=x run=1 ns=220\nSPEED name=x run=2 ns=230\nSPEED name=y run=0 ns=9\nSPEED_CHECK sha256=aa\n");
        let b = data("SPEED name=x run=0 ns=50\nSPEED name=x run=1 ns=400\nSPEED name=x run=2 ns=100\nSPEED_CHECK sha256=aa\n");
        let (report, v) = report_ab(&a, &a2, &b);
        assert_eq!(v, Verdict::SameOutput);
        assert_eq!(
            report,
            "SPEED_AB case=x a_ns=200 a2_ns=220 b_ns=100 aa=1.100 gain=2.000 runs=3\n\
             SPEED_AB case=y a_ns=9 a2_ns=9 b_ns=NA aa=1.000 gain=NA runs=1/1/0\n\
             SPEED_AB_CHECK tree=a sha256=aa\n\
             SPEED_AB_CHECK tree=a2 sha256=aa\n\
             SPEED_AB_CHECK tree=b sha256=aa\n\
             SPEED_AB_VERDICT=SAME_OUTPUT\n"
        );
        let (report, v) = report_candidate(&b);
        assert_eq!(v, Verdict::CandidateOnly);
        assert_eq!(
            report,
            "SPEED_AB case=x b_ns=100 runs=3\nSPEED_AB_CHECK tree=b sha256=aa\nSPEED_AB_VERDICT=CANDIDATE_ONLY\n"
        );
        let (report, v) = report_candidate(&TreeData::default());
        assert_eq!(v, Verdict::NoData);
        assert_eq!(report, "SPEED_AB_VERDICT=NO_DATA\n");
        let changed = data("SPEED name=x run=0 ns=1\nSPEED_CHECK sha256=bb\n");
        let (report, v) = report_ab(&a, &a2, &changed);
        assert_eq!(v, Verdict::DifferentOutput);
        let tail = "SPEED_AB_CHECK tree=b sha256=bb\nSPEED_AB_VERDICT=DIFFERENT_OUTPUT\n";
        assert!(report.ends_with(tail), "{report}");
    }

    #[test]
    fn options_are_strict() {
        let base = words(
            "--a /a --a2 /a2 --b /b --package gel-core --example speed_crc --target-root /t --out /o",
        );
        let o = options(&base).unwrap();
        assert_eq!(o.rounds, 3);
        assert!(o.before_run.is_empty() && o.example_args.is_empty() && !o.candidate_only);
        let mut args = base.clone();
        args.extend(words(
            "--rounds 5 --before-run-arg --x --with-bins gel-bench --before-run wait-quiet -- 1 --y",
        ));
        let o = options(&args).unwrap();
        assert_eq!(o.rounds, 5);
        assert_eq!(o.with_bins, words("gel-bench"));
        assert_eq!(o.before_run, words("wait-quiet --x"));
        assert_eq!(o.example_args, words("1 --y"));
        for extra in [
            "--rounds 0",
            "--rounds 1000",
            "--rounds x",
            "--a /again",
            "--before-run",
            "--before-run a --before-run b",
            "--before-run-arg x",
            "--bogus 1",
            "--candidate-only",
            "--with-bins ../x",
        ] {
            let mut args = base.clone();
            args.extend(words(extra));
            assert!(options(&args).is_err(), "{extra:?} was accepted");
        }
        let candidate =
            words("--candidate-only --b /b --package p --example e --target-root /t --out /o");
        assert!(options(&candidate).unwrap().candidate_only);
        assert!(options(&candidate[1..]).is_err());
        let mut bad = candidate.clone();
        bad[6] = "../e".into();
        assert!(options(&bad).is_err());
    }

    #[test]
    fn build_command_matches_the_method() {
        assert_eq!(
            build_args("gel-core", "speed_crc", &[]).join(" "),
            "build --locked --offline --release -p gel-core --example speed_crc"
        );
        let bins = words("gel-bench gel-physics");
        assert_eq!(
            build_args("gel-bench", "speed_x", &bins).join(" "),
            "build --locked --offline --release -p gel-bench --example speed_x -p gel-bench -p gel-physics --bins"
        );
    }

    #[test]
    fn placed_files_and_created_directories_are_removed_on_error() {
        let src = scratch("src");
        let dest = scratch("dest");
        fs::create_dir_all(src.join("crates/p/examples/support/deep")).unwrap();
        fs::write(src.join("crates/p/Cargo.toml"), "[package]\n").unwrap();
        let x_source = "#[path = \"support/s.rs\"]\nmod s;\n";
        fs::write(src.join("crates/p/examples/x.rs"), x_source).unwrap();
        fs::write(src.join("crates/p/examples/support/s.rs"), "// s\n").unwrap();
        fs::write(src.join("crates/p/examples/support/deep/t.rs"), "// t\n").unwrap();
        fs::create_dir_all(dest.join("crates/p/examples")).unwrap();
        fs::write(dest.join("crates/p/Cargo.toml"), "[package]\n").unwrap();
        fs::write(dest.join("crates/p/examples/other.rs"), "fn main() {}\n").unwrap();
        let before = listing(&dest);

        let files = example_files(&src, "p", "x").unwrap();
        let names: Vec<&Path> = files.iter().map(|f| f.rel.as_path()).collect();
        assert_eq!(
            names,
            vec![
                Path::new("crates/p/examples/x.rs"),
                Path::new("crates/p/examples/support/deep/t.rs"),
                Path::new("crates/p/examples/support/s.rs"),
            ]
        );
        let failing_run = |files: &[Carried]| -> Result<(), String> {
            let mut placed = Placed::default();
            placed.place_all(&dest, files, false)?;
            assert_eq!(placed.files.len(), 3);
            assert_eq!(placed.dirs.len(), 2);
            let placed_t = fs::read(dest.join("crates/p/examples/support/deep/t.rs")).unwrap();
            assert_eq!(placed_t, b"// t\n");
            Err("example run failed".into())
        };
        assert!(failing_run(&files).is_err());
        assert_eq!(listing(&dest), before);
        assert!(!dest.join("crates/p/examples/support").exists());

        // An identical existing file is left in place and not removed afterwards.
        fs::write(dest.join("crates/p/examples/x.rs"), &files[0].bytes).unwrap();
        let before = listing(&dest);
        let mut placed = Placed::default();
        placed.place_all(&dest, &files, false).unwrap();
        assert_eq!(placed.files.len(), 2);
        placed.restore().unwrap();
        assert_eq!(listing(&dest), before);

        // Different content is refused before anything is written, also with
        // --replace-helpers: the example is not a helper.
        fs::write(dest.join("crates/p/examples/x.rs"), "fn main() {}\n").unwrap();
        let before = listing(&dest);
        let mut placed = Placed::default();
        let err = placed.place_all(&dest, &files, true).unwrap_err();
        assert!(err.contains("different content"), "{err}");
        assert!(placed.files.is_empty() && placed.dirs.is_empty());
        drop(placed);
        assert_eq!(listing(&dest), before);
        assert!(!dest.join("crates/p/examples/support").exists());

        fs::remove_dir_all(&src).unwrap();
        fs::remove_dir_all(&dest).unwrap();
    }

    #[test]
    fn support_directory_is_copied_only_when_used() {
        let with_path = "#[path = \"support/v020_topk.rs\"]\nmod v020;";
        assert!(uses_support(with_path));
        assert!(uses_support("mod support;"));
        assert!(!uses_support("// unsupported input\nfn main() {}"));
        let src = scratch("plain");
        fs::create_dir_all(src.join("crates/p/examples/support")).unwrap();
        fs::write(src.join("crates/p/Cargo.toml"), "[package]\n").unwrap();
        fs::write(src.join("crates/p/examples/x.rs"), "fn main() {}\n").unwrap();
        fs::write(src.join("crates/p/examples/support/s.rs"), "// s\n").unwrap();
        assert_eq!(example_files(&src, "p", "x").unwrap().len(), 1);
        assert!(example_files(&src, "p", "missing").is_err());
        assert!(example_files(&src, "q", "x").is_err());
        fs::remove_dir_all(&src).unwrap();
    }

    #[test]
    fn path_includes_are_found_in_code_only() {
        let source = r###"//! Included with `#[path = "doc/in_comment.rs"] mod x;`.
/* #[path = "block.rs"] /* nested #[path = "nested.rs"] */ */
#[path = "../../../tools/speed_support.rs"]
mod speed;
#[allow(dead_code)]
#[path="support/tight.rs"]
mod tight;
# [ path = "spaced.rs" ]
mod spaced;
const A: &str = "#[path = \"in_string.rs\"]";
const B: &str = r#"#[path = "in_raw.rs"]"#;
const B2: &[u8] = br##"a"# #[path = "in_raw_two.rs"] "##;
const C: char = '"';
const D: char = '#';
const E: char = '\'';
fn f<'a>(x: &'a str) -> &'a str { x }
#[cfg(not(test))]
#[path = "../src/last.rs"]
mod last;
"###;
        assert_eq!(
            path_includes(source).unwrap(),
            vec![
                "../../../tools/speed_support.rs",
                "support/tight.rs",
                "spaced.rs",
                "../src/last.rs",
            ]
        );
        assert_eq!(path_includes("fn main() {}").unwrap(), Vec::<String>::new());
        for bad in [
            "#[cfg_attr(unix, path = \"u.rs\")]\nmod u;",
            "#[path = \"a\\\\b.rs\"]\nmod e;",
            "#[path = concat!(\"a\", \".rs\")]\nmod m;",
            "#[path = \"open.rs\"\nmod o;",
            "/* unterminated",
        ] {
            assert!(path_includes(bad).is_err(), "{bad:?} was accepted");
        }
    }

    #[test]
    fn includes_resolve_against_the_including_file_inside_the_tree() {
        let ex = Path::new("crates/p/examples/x.rs");
        assert_eq!(
            resolve_include(ex, "../../../tools/h.rs").unwrap(),
            Path::new("tools/h.rs")
        );
        assert_eq!(
            resolve_include(ex, "./support/./s.rs").unwrap(),
            Path::new("crates/p/examples/support/s.rs")
        );
        assert_eq!(
            resolve_include(Path::new("tools/h.rs"), "inner.rs").unwrap(),
            Path::new("tools/inner.rs")
        );
        assert!(resolve_include(ex, "../../../../outside.rs").is_err());
        assert!(resolve_include(ex, "/etc/hostname").is_err());
    }

    /// A tree with crate `p`, example `x` including a helper under `tools/`
    /// (which includes a second helper) and a source file of the crate.
    fn tree_with_includes(tag: &str) -> PathBuf {
        let src = scratch(tag);
        fs::create_dir_all(src.join("crates/p/examples")).unwrap();
        fs::create_dir_all(src.join("crates/p/src")).unwrap();
        fs::create_dir_all(src.join("tools")).unwrap();
        fs::write(src.join("crates/p/Cargo.toml"), "[package]\n").unwrap();
        fs::write(
            src.join("crates/p/examples/x.rs"),
            "#[path = \"../../../tools/h.rs\"]\nmod h;\n#[path = \"../src/r.rs\"]\nmod r;\n#[path = \"../../../tools/h.rs\"]\nmod again;\nfn main() {}\n",
        )
        .unwrap();
        fs::write(
            src.join("tools/h.rs"),
            "//! `#[path = \"ignored.rs\"]`\n#[path = \"inner.rs\"]\nmod inner;\n#[path = \"h.rs\"]\nmod cycle;\n",
        )
        .unwrap();
        fs::write(src.join("tools/inner.rs"), "// inner\n").unwrap();
        fs::write(src.join("crates/p/src/r.rs"), "// r\n").unwrap();
        src
    }

    #[test]
    fn included_files_are_carried_recursively_once() {
        let src = tree_with_includes("inc-src");
        let files = example_files(&src, "p", "x").unwrap();
        let carried: Vec<(&Path, Origin)> =
            files.iter().map(|f| (f.rel.as_path(), f.origin)).collect();
        assert_eq!(
            carried,
            vec![
                (Path::new("crates/p/examples/x.rs"), Origin::Example),
                (Path::new("tools/h.rs"), Origin::Included),
                (Path::new("crates/p/src/r.rs"), Origin::Included),
                (Path::new("tools/inner.rs"), Origin::Included),
            ]
        );
        assert_eq!(files[3].bytes, b"// inner\n");
        // An include that does not exist in B is an error, not a silent skip.
        fs::remove_file(src.join("tools/inner.rs")).unwrap();
        let err = example_files(&src, "p", "x").unwrap_err();
        let inner = Path::new("tools").join("inner.rs");
        assert!(err.contains(&inner.display().to_string()), "{err}");
        fs::remove_dir_all(&src).unwrap();
    }

    #[test]
    fn included_files_are_placed_refused_or_replaced_and_restored() {
        let src = tree_with_includes("inc-from");
        let files = example_files(&src, "p", "x").unwrap();
        let a = scratch("inc-a");
        let a2 = scratch("inc-a2");
        for root in [&a, &a2] {
            fs::create_dir_all(root.join("crates/p/src")).unwrap();
            fs::create_dir_all(root.join("crates/p/examples")).unwrap();
            fs::write(root.join("crates/p/Cargo.toml"), "[package]\n").unwrap();
            fs::write(root.join("crates/p/src/r.rs"), "// r\n").unwrap();
        }

        // Missing files are created (with their directory) and removed again;
        // the identical r.rs is kept.
        let before = listing(&a);
        let mut placed = Placed::default();
        placed.place_all(&a, &files, false).unwrap();
        assert_eq!(placed.files.len(), 3);
        assert_eq!(placed.dirs, vec![a.join("tools")]);
        assert_eq!(fs::read(a.join("tools/inner.rs")).unwrap(), b"// inner\n");
        placed.restore().unwrap();
        assert_eq!(listing(&a), before);
        assert!(!a.join("tools").exists());

        // A helper under tools/ with other bytes in A2 and no flag: refused
        // before either tree is written.
        fs::create_dir_all(a2.join("tools")).unwrap();
        fs::write(a2.join("tools/h.rs"), "// other helper\n").unwrap();
        let (before_a, before_a2) = (listing(&a), listing(&a2));
        let plan_a = Placed::plan(&a, &files, false).unwrap();
        let err = Placed::plan(&a2, &files, false).unwrap_err();
        assert!(err.contains("different content"), "{err}");
        assert!(err.contains("--replace-helpers"), "{err}");
        assert!(plan_a.contains(&Step::Create));
        assert_eq!((listing(&a), listing(&a2)), (before_a, before_a2.clone()));

        // With the flag the helper is replaced and its bytes come back, both
        // on restore and when a later step fails (the guard is dropped).
        for fail_later in [false, true] {
            let mut placed = Placed::default();
            let plan = Placed::plan(&a2, &files, true).unwrap();
            assert_eq!(plan[1], Step::Replace(b"// other helper\n".to_vec()));
            placed.apply(&a2, &files, &plan).unwrap();
            assert_eq!(fs::read(a2.join("tools/h.rs")).unwrap(), files[1].bytes);
            if fail_later {
                drop(placed);
            } else {
                placed.restore().unwrap();
            }
            assert_eq!(listing(&a2), before_a2);
            assert_eq!(
                fs::read(a2.join("tools/h.rs")).unwrap(),
                b"// other helper\n"
            );
        }

        // A different included file outside tools/ is refused even with the flag.
        fs::write(a2.join("crates/p/src/r.rs"), "// other r\n").unwrap();
        let before_a2 = listing(&a2);
        let err = Placed::plan(&a2, &files, true).unwrap_err();
        let r = Path::new("crates").join("p").join("src").join("r.rs");
        assert!(err.contains(&r.display().to_string()), "{err}");
        let mut placed = Placed::default();
        assert!(placed.place_all(&a2, &files, true).is_err());
        drop(placed);
        assert_eq!(listing(&a2), before_a2);

        for root in [&src, &a, &a2] {
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn replace_helpers_is_a_flag_of_the_ab_mode() {
        let base = words(
            "--a /a --a2 /a2 --b /b --package gel-core --example speed_crc --target-root /t --out /o",
        );
        assert!(!options(&base).unwrap().replace_helpers);
        let mut args = base.clone();
        args.push("--replace-helpers".into());
        assert!(options(&args).unwrap().replace_helpers);
        args.push("--replace-helpers".into());
        assert!(options(&args).is_err());
        let candidate = words(
            "--candidate-only --replace-helpers --b /b --package p --example e --target-root /t --out /o",
        );
        assert!(options(&candidate).is_err());
    }
}
