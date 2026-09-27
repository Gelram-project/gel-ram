//! Paired comparison of Evidence Lab phrase search with a standard tool on the
//! same public corpus (docs/BENCHMARK-GREP.md).
//!
//! The two tools follow different rules: GEL matches whole words after Unicode
//! normalization and treats punctuation and underscores as separators; grep -w
//! -F matches a literal string at word boundaries. Answers are therefore
//! compared first, per query, and times are compared only for queries whose
//! answers agree. Every timing sample is kept; nothing is filtered.
use gel_source::{collection::Collection, digest, hex};
use std::{
    fmt::Write as _,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Instant,
};

const WARMUP: usize = 3;
const DEFAULT_REPS: usize = 30;
const LOCALE: &str = "C.UTF-8";

/// Fixed queries: common, rare, absent, multi-word, punctuation-bridging and Unicode.
const QUERIES: &[&str] = &[
    "the",
    "source",
    "release",
    "evidence lab",
    "exact source",
    "public main",
    "no model",
    "fresh process",
    "stale citation",
    "full disk",
    "plaintext",
    "not encrypted",
    "does not establish",
    "garbage collection",
    "ownership",
    "the stack",
    "move semantics",
    "invented answer",
    "quantum banana",
    "purple elephant",
    "rust 1 85 0",
    "sha 256",
    "gel evidence",
    "source sha256sums txt",
    "xtask verify",
    "cargo run",
    "locked offline",
    "q8",
    "four views",
    "zażółć gęślą",
    "ZAŻÓŁĆ GĘŚLĄ",
    "jaźń",
    "łódź",
    "λόγος",
    "ΛΌΓΟΣ",
    "café",
];

/// Synthetic Unicode document: precomposed and decomposed forms, final sigma.
const UNICODE: &str = "Zażółć gęślą jaźń.\nŁódź is a city name.\nλόγος and ΛΌΓΟΣ in one line.\nλόγοσ written without final sigma.\ncafe\u{301} with a combining accent.\ncafé precomposed.\n";

struct Doc {
    title: String,
    path: PathBuf,
    text: String,
}

#[derive(Clone)]
struct Answer {
    count: usize,
    skipped: usize,
    lines: Vec<(usize, usize)>,
}

fn line_of(text: &str, start: usize) -> usize {
    text[..start].matches('\n').count() + 1
}

fn gel_answer(bank: &Collection, docs: &[Doc], query: &str) -> Result<Answer, String> {
    let found = bank.search(query)?;
    let mut lines = Vec::new();
    for h in &found.hits {
        let index = (h.document_id() - 1) as usize;
        lines.push((index, line_of(&docs[index].text, h.span().start)));
    }
    Ok(Answer {
        count: found.matching_lines,
        skipped: found.skipped_long_lines,
        lines,
    })
}

fn grep_command(grep: &str, docs: &[Doc], query: &str) -> Command {
    let mut c = Command::new(grep);
    c.env("LC_ALL", LOCALE)
        .args(["-n", "-i", "-w", "-F", "-e", query, "--"])
        .args(docs.iter().map(|d| &d.path))
        .stdin(Stdio::null());
    c
}

fn grep_answer(docs: &[Doc], out: &Output) -> Result<Answer, String> {
    // Exit 1 means no match; 2 is an error.
    if !matches!(out.status.code(), Some(0 | 1)) {
        return Err(format!(
            "grep failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = Vec::new();
    for row in text.lines() {
        let (path, rest) = docs
            .iter()
            .enumerate()
            .find_map(|(i, d)| {
                row.strip_prefix(&format!("{}:", d.path.display()))
                    .map(|r| (i, r))
            })
            .ok_or_else(|| format!("unexpected grep line: {row}"))?;
        let number = rest
            .split(':')
            .next()
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("no line number: {row}"))?;
        lines.push((path, number));
    }
    Ok(Answer {
        count: lines.len(),
        skipped: 0,
        lines,
    })
}

/// Same answer: equal line counts, nothing skipped, and every line GEL shows
/// is also listed by grep. Otherwise the most likely reason is named.
fn compare(query: &str, gel: &Answer, grep: &Answer) -> (bool, &'static str) {
    if gel.skipped > 0 {
        return (false, "gel-skipped-long-line");
    }
    let contained = gel.lines.iter().all(|l| grep.lines.contains(l));
    if gel.count == grep.count && contained {
        return (true, "same");
    }
    if !query.is_ascii() {
        (false, "unicode-normalization-or-case")
    } else if gel.count > grep.count {
        (false, "gel-splits-words-at-punctuation-and-underscore")
    } else if gel.count < grep.count {
        (false, "grep-word-boundary-differs")
    } else {
        (false, "different-lines")
    }
}

fn target_binary(root: &Path, name: &str) -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    target
        .join("release")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// The tested revision and whether the working tree differed from it.
pub(crate) fn revision(root: &Path) -> String {
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    match (git(&["rev-parse", "HEAD"]), git(&["status", "--porcelain"])) {
        (Some(head), Some(status)) => format!(
            "revision={head} working_tree={}",
            if status.is_empty() { "clean" } else { "dirty" }
        ),
        _ => "revision=NOT_A_GIT_CHECKOUT".into(),
    }
}

fn first_line(program: &str, arg: &str) -> String {
    Command::new(program)
        .arg(arg)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(str::to_string))
        .unwrap_or_else(|| "UNAVAILABLE".into())
}

/// Value of `"key":` in one of our own JSON Lines records.
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let at = line.find(&format!("\"{key}\":"))? + key.len() + 3;
    let rest = &line[at..];
    Some(&rest[..rest.find([',', '}'])?])
}

fn timed(mut c: Command) -> Result<(u128, Output), String> {
    let start = Instant::now();
    let out = c.output().map_err(|e| e.to_string())?;
    Ok((start.elapsed().as_nanos(), out))
}

fn gel_batch(binary: &Path, input: &str) -> Result<(u128, Output), String> {
    let start = Instant::now();
    let mut child = Command::new(binary)
        .arg("--batch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(input.as_bytes())
        .map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let ns = start.elapsed().as_nanos();
    if out.status.code() != Some(0) {
        return Err(format!(
            "gel-evidence batch failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok((ns, out))
}

fn corpus(root: &Path, out: &Path) -> Result<Vec<Doc>, String> {
    let mut paths = vec![root.join("README.md")];
    let mut docs_dir: Vec<PathBuf> = fs::read_dir(root.join("docs"))
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    // The page reporting this comparison is not part of its own corpus.
    docs_dir.retain(|p| !p.ends_with("BENCHMARK-GREP.md"));
    docs_dir.sort();
    paths.extend(docs_dir);
    paths.push(root.join("crates/gel-source/fixtures/rust-book/ownership.txt"));
    let unicode = out.join("unicode.txt");
    fs::write(&unicode, UNICODE).map_err(|e| e.to_string())?;
    paths.push(unicode);
    paths
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let title = path
                .strip_prefix(root)
                .unwrap_or_else(|_| Path::new("synthetic/unicode.txt"))
                .to_string_lossy()
                .replace('\\', "/");
            Ok(Doc { title, path, text })
        })
        .collect()
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    let rank = ((p * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

struct Samples(Vec<(String, String, usize, bool, String, u128)>);
impl Samples {
    fn add(&mut self, phase: &str, tool: &str, rep: usize, query: &str, value: u128) {
        self.0.push((
            phase.into(),
            tool.into(),
            rep,
            rep < WARMUP,
            query.into(),
            value,
        ));
    }
    fn measured(&self, phase: &str, tool: &str) -> Vec<u128> {
        let mut v: Vec<u128> = self
            .0
            .iter()
            .filter(|s| s.0 == phase && s.1 == tool && !s.3)
            .map(|s| s.5)
            .collect();
        v.sort_unstable();
        v
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage = "usage: cargo run -p xtask -- bench-compare NEW_DIR_OUTSIDE_CHECKOUT [--answers-only | --reps N]";
    let (dir, mode) = match args {
        [dir] => (dir, None),
        [dir, flag] if flag == "--answers-only" => (dir, Some(0)),
        [dir, flag, n] if flag == "--reps" => (dir, Some(n.parse::<usize>().map_err(|_| usage)?)),
        _ => return Err(usage.into()),
    };
    let answers_only = mode == Some(0);
    let reps = mode.filter(|n| *n > 0).unwrap_or(DEFAULT_REPS);
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
    fs::create_dir(out).map_err(|e| format!("new output directory required: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;
    let load_start = fs::read_to_string("/proc/loadavg").unwrap_or_else(|_| "NOT_MEASURED".into());

    super::run(
        "cargo",
        &[
            "build",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
        ],
    )?;
    let binary = target_binary(&root, "gel-evidence");
    let docs = corpus(&root, &out)?;
    let mut bank = Collection::new();
    for d in &docs {
        bank.add(&d.title, &d.text)?;
    }
    let snapshot = out.join("corpus.gelset");
    let pin = hex(&bank.save_new(&snapshot)?);

    let mut manifest = String::new();
    let _ = writeln!(manifest, "FORMAT=GEL_BENCH_COMPARE_1");
    let _ = writeln!(manifest, "{}", revision(&root));
    let _ = writeln!(manifest, "grep={}", first_line("grep", "--version"));
    let _ = writeln!(
        manifest,
        "sha256sum={}",
        first_line("sha256sum", "--version")
    );
    let _ = writeln!(manifest, "locale={LOCALE}");
    let _ = writeln!(manifest, "gel_grep_semantics=GEL: whole words after NFC+lowercase, punctuation and underscore separate words, lines over 4096 bytes skipped; grep -n -i -w -F: literal string at word boundaries");
    let _ = writeln!(manifest, "{}", super::isolation::status());
    let _ = writeln!(manifest, "loadavg_start={}", load_start.trim());
    let _ = writeln!(manifest, "snapshot_sha256={pin} documents={}", docs.len());
    for d in &docs {
        let _ = writeln!(
            manifest,
            "{}  {}  bytes={}",
            hex(&digest(d.text.as_bytes())),
            d.title,
            d.text.len()
        );
    }

    let mut answers =
        String::from("id\tquery\tgel_lines\tgrep_lines\tgel_skipped\tverdict\treason\n");
    let mut same = Vec::new();
    for (id, query) in QUERIES.iter().enumerate() {
        let gel = gel_answer(&bank, &docs, query)?;
        let (_, output) = timed(grep_command("grep", &docs, query))?;
        let grep = grep_answer(&docs, &output)?;
        let (ok, reason) = compare(query, &gel, &grep);
        if ok {
            same.push(id);
        }
        let _ = writeln!(
            answers,
            "{id}\t{query}\t{}\t{}\t{}\t{}\t{reason}",
            gel.count,
            grep.count,
            gel.skipped,
            if ok { "SAME" } else { "DIFFERENT" }
        );
    }
    fs::write(out.join("answers.tsv"), &answers).map_err(|e| e.to_string())?;
    let agreement = format!(
        "same_answer={} different_answer={} queries={}",
        same.len(),
        QUERIES.len() - same.len(),
        QUERIES.len()
    );
    if answers_only {
        fs::write(out.join("manifest.txt"), &manifest).map_err(|e| e.to_string())?;
        println!("BENCH_COMPARE_ANSWERS {agreement} dir={}", out.display());
        return Ok(());
    }

    let snapshot_arg = snapshot.display().to_string();
    let mut workload = format!("load {pin} {snapshot_arg}\n");
    for id in &same {
        let _ = writeln!(workload, "find {}", QUERIES[*id]);
    }
    workload.push_str("exit\n");
    let load_only = format!("load {pin} {snapshot_arg}\nexit\n");
    let mut samples = Samples(Vec::new());
    let mut rss = Vec::new();
    for rep in 0..WARMUP + reps {
        // Alternate which tool runs first in each repetition.
        for turn in 0..2 {
            if (rep + turn) % 2 == 0 {
                let (ns, output) = gel_batch(&binary, &workload)?;
                samples.add("process-workload", "gel-evidence", rep, "*", ns);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let finds = stdout
                    .lines()
                    .filter(|l| l.contains("\"command\":\"find\""));
                for (id, line) in same.iter().zip(finds) {
                    let ns = field(line, "search_ns")
                        .and_then(|v| v.parse().ok())
                        .ok_or("missing search_ns")?;
                    samples.add("in-memory-search", "gel-evidence", rep, QUERIES[*id], ns);
                }
                if let Some(kb) = stdout.lines().last().and_then(|l| field(l, "peak_rss_kb")) {
                    rss.push(kb.to_string());
                }
                let (ns, _) = gel_batch(&binary, &load_only)?;
                samples.add("integrity-process", "gel-evidence", rep, "*", ns);
            } else {
                let mut total = 0;
                for id in &same {
                    let (ns, output) = timed(grep_command("grep", &docs, QUERIES[*id]))?;
                    grep_answer(&docs, &output)?;
                    samples.add("process-per-query", "grep", rep, QUERIES[*id], ns);
                    total += ns;
                }
                samples.add("process-workload", "grep", rep, "*", total);
                let mut c = Command::new("sha256sum");
                c.arg(&snapshot);
                let (ns, output) = timed(c)?;
                if !String::from_utf8_lossy(&output.stdout).starts_with(&pin) {
                    return Err("sha256sum disagrees with the snapshot pin".into());
                }
                samples.add("integrity-process", "sha256sum", rep, "*", ns);
            }
        }
    }

    let mut csv = String::from("phase,tool,rep,warmup,query,value_ns\n");
    for s in &samples.0 {
        let _ = writeln!(
            csv,
            "{},{},{},{},\"{}\",{}",
            s.0,
            s.1,
            s.2,
            s.3,
            s.4.replace('"', "\"\""),
            s.5
        );
    }
    fs::write(out.join("samples.csv"), csv).map_err(|e| e.to_string())?;
    let mut summary = format!("{agreement} warmup={WARMUP} reps={reps}\n");
    for (phase, tools) in [
        ("process-workload", ["gel-evidence", "grep"]),
        ("integrity-process", ["gel-evidence", "sha256sum"]),
    ] {
        let a = samples.measured(phase, tools[0]);
        let b = samples.measured(phase, tools[1]);
        let _ = writeln!(
            summary,
            "{phase}: {} p50={} p95={} | {} p50={} p95={} | median_ratio={:.3} (small sample; p95 is nearest rank)",
            tools[0],
            percentile(&a, 0.5),
            percentile(&a, 0.95),
            tools[1],
            percentile(&b, 0.5),
            percentile(&b, 0.95),
            percentile(&a, 0.5) as f64 / percentile(&b, 0.5) as f64
        );
    }
    let search = samples.measured("in-memory-search", "gel-evidence");
    let _ = writeln!(
        summary,
        "in-memory-search: gel-evidence per query p50={} p95={} ns (excludes process start and load; no grep counterpart)",
        percentile(&search, 0.5),
        percentile(&search, 0.95)
    );
    let _ = writeln!(
        summary,
        "gel-evidence peak_rss_kb per workload run: {}",
        rss.join(",")
    );
    let _ = writeln!(
        summary,
        "loadavg_end={}",
        fs::read_to_string("/proc/loadavg")
            .unwrap_or_else(|_| "NOT_MEASURED".into())
            .trim()
    );
    fs::write(out.join("manifest.txt"), &manifest).map_err(|e| e.to_string())?;
    fs::write(out.join("summary.txt"), &summary).map_err(|e| e.to_string())?;
    fs::write(out.join("COMPLETE.txt"), "BENCH_COMPARE_COMPLETE=PASS\n")
        .map_err(|e| e.to_string())?;
    print!("{summary}");
    println!("BENCH_COMPARE=PASS dir={}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(count: usize, skipped: usize, lines: &[(usize, usize)]) -> Answer {
        Answer {
            count,
            skipped,
            lines: lines.to_vec(),
        }
    }

    #[test]
    fn only_equal_counts_with_contained_lines_are_the_same_answer() {
        let grep = answer(2, 0, &[(0, 3), (1, 7)]);
        assert_eq!(
            compare("a", &answer(2, 0, &[(0, 3), (1, 7)]), &grep),
            (true, "same")
        );
        assert_eq!(
            compare("a", &answer(2, 0, &[(0, 4)]), &grep).1,
            "different-lines"
        );
        assert_eq!(
            compare("a", &answer(3, 0, &[]), &grep).1,
            "gel-splits-words-at-punctuation-and-underscore"
        );
        assert_eq!(
            compare("a", &answer(1, 0, &[]), &grep).1,
            "grep-word-boundary-differs"
        );
        assert_eq!(
            compare("a", &answer(2, 1, &[]), &grep).1,
            "gel-skipped-long-line"
        );
        assert_eq!(
            compare("łódź", &answer(1, 0, &[]), &grep).1,
            "unicode-normalization-or-case"
        );
    }

    #[test]
    fn nearest_rank_percentiles() {
        let v: Vec<u128> = (1..=20).collect();
        assert_eq!(percentile(&v, 0.5), 10);
        assert_eq!(percentile(&v, 0.95), 19);
        assert_eq!(percentile(&[7], 0.95), 7);
    }

    #[test]
    fn fields_are_read_from_our_own_records() {
        let line = r#"{"schema":"gel-evidence/1","search_ns":123,"peak_rss_kb":null}"#;
        assert_eq!(field(line, "search_ns"), Some("123"));
        assert_eq!(field(line, "peak_rss_kb"), Some("null"));
        assert_eq!(field(line, "missing"), None);
    }
}
