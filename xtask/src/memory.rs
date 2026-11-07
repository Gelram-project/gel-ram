// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! `memory-run` and `memory-check`: memory accounting of public operations
//! (docs/MEMORY-ACCOUNTING.md).
//!
//! `memory-run` starts each named operation as a child process at least five
//! times, in rounds, and records what the process cost: the payload the
//! operation states, the peak of its mapped address space, its peak and last
//! resident set, its page faults, its threads, its exit status, and the load and
//! available memory of the host just before and just after it. It records no
//! time, and it keeps no output of the operation.
//!
//! The workspace forbids unsafe code and the standard library has no safe call
//! for `wait4`, so the peak is not taken from the rusage of the child: this
//! program reads `/proc/<pid>/status` in a short loop until the child ends, and
//! the last VmHWM it could read is the peak. Pages touched after the last read
//! are not seen. Page faults do come from the kernel's account of the child:
//! `/proc/self/stat` of this process counts the faults of every child it has
//! waited for, so the difference over one wait is exactly that child and the
//! children it waited for.
//!
//! `memory-check FILE` accepts a record only when every field is present and
//! well formed, every process exited with 0, nothing mandatory is missing, every
//! operation has at least five samples, every summary recomputes from its
//! samples, every record line describes an operation the way this program
//! does, and the peak of a resident operation is not below its payload.
use gel_source::MAX_TEXT;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};

const VERSION: &str = "GEL_MEMORY_1";
const MIN_REPS: usize = 5;
const MAX_REPS: usize = 99;
/// The pause of the read loop; the real interval is longer on a loaded host.
const POLL_MICROS: u64 = 100;
const USAGE: &str = "usage: cargo run --locked --offline -p xtask -- memory-run --out NEW_DIR_OUTSIDE_CHECKOUT --work DIR_OUTSIDE_CHECKOUT [--target-root BUILD_DIR] [--reps N] [--only OPERATION]...\n   or: cargo run --locked --offline -p xtask -- memory-check FILE";

/// The header keys of a record, each exactly once, before the first sample.
const HEADER_KEYS: [&str; 12] = [
    "tool",
    "revision",
    "os",
    "os_version",
    "kernel",
    "cpu_model",
    "logical_cpus",
    "memory_total_kib",
    "rustc",
    "profile",
    "poll_pause_us",
    "note",
];

/// The numbers of one sample, in line order; a summary holds one triple for each.
const MEASURES: [&str; 12] = [
    "payload_bytes",
    "mapped_peak_kib",
    "rss_last_kib",
    "peak_rss_kib",
    "minor_faults",
    "major_faults",
    "threads_peak",
    "polls",
    "load1_centi_start",
    "load1_centi_end",
    "mem_available_kib_start",
    "mem_available_kib_end",
];
const PAYLOAD: usize = 0;
const PEAK: usize = 3;
const POLLS: usize = 7;

/// The keys of a record line before its triples.
const RECORD_KEYS: [&str; 9] = [
    "op",
    "kind",
    "package",
    "target",
    "args",
    "payload_source",
    "resident",
    "workers_requested",
    "reps",
];
const SAMPLE_KEYS: [&str; 3] = ["op", "rep", "exit"];

#[derive(Clone, Copy, PartialEq)]
enum Program {
    Example,
    Bin,
    /// This program itself, `memory-touch`: the control of the instrument.
    Control,
}

/// What an operation reads on standard input.
#[derive(Clone, Copy, PartialEq)]
enum Input {
    Nothing,
    /// The authored Evidence Lab fixtures, then three searches, many times.
    Fixtures,
    /// Eight generated documents, then a saved snapshot.
    Save,
    /// A saved snapshot of those documents, then one search.
    Load,
}

/// Where the payload of an operation is stated.
#[derive(Clone, Copy)]
enum Payload {
    /// A key of the operation's own `SPEED_SETUP` line.
    Output(&'static str),
    /// The sizes of the files it reads.
    Files,
}

struct Operation {
    name: &'static str,
    program: Program,
    package: &'static str,
    target: &'static str,
    /// `WORK_DIR` stands for the working directory of the run.
    args: &'static [&'static str],
    input: Input,
    payload: Payload,
    /// The payload must be in memory at once, so the peak cannot be below it.
    resident: bool,
    workers_requested: u64,
}

const CONTROL_BYTES: &str = "67108864";
const OPERATIONS: [Operation; 8] = [
    Operation {
        name: "control_touch_64mib",
        program: Program::Control,
        package: "xtask",
        target: "memory-touch",
        args: &[CONTROL_BYTES],
        input: Input::Nothing,
        payload: Payload::Output("touched_bytes"),
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "q8_read_8192",
        program: Program::Example,
        package: "gel-phase-quad",
        target: "speed_q8_read",
        args: &["1"],
        input: Input::Nothing,
        payload: Payload::Output("record_bytes"),
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "q8_queries_32768_t4",
        program: Program::Example,
        package: "gel-phase-quad",
        target: "speed_q8_queries",
        args: &["1", "4"],
        input: Input::Nothing,
        payload: Payload::Output("record_bytes"),
        resident: true,
        workers_requested: 4,
    },
    Operation {
        name: "evidence_fixtures",
        program: Program::Bin,
        package: "gel-live-lab",
        target: "gel-evidence",
        args: &["--batch"],
        input: Input::Fixtures,
        payload: Payload::Files,
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "evidence_save_64mib",
        program: Program::Bin,
        package: "gel-live-lab",
        target: "gel-evidence",
        args: &["--batch"],
        input: Input::Save,
        payload: Payload::Files,
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "evidence_load_64mib",
        program: Program::Bin,
        package: "gel-live-lab",
        target: "gel-evidence",
        args: &["--batch"],
        input: Input::Load,
        payload: Payload::Files,
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "snapshot_load_64mib",
        program: Program::Example,
        package: "gel-source",
        target: "speed_collection_memory",
        args: &["1", "WORK_DIR"],
        input: Input::Nothing,
        payload: Payload::Output("snapshot_bytes"),
        resident: true,
        workers_requested: 1,
    },
    Operation {
        name: "bundle_load_64mib",
        program: Program::Example,
        package: "gel-source",
        target: "speed_bundle_memory",
        args: &["1", "WORK_DIR"],
        input: Input::Nothing,
        payload: Payload::Output("bundle_bytes"),
        resident: true,
        workers_requested: 1,
    },
];

const FIXTURES: [&str; 4] = ["memory.txt", "original.txt", "revised.txt", "unicode.txt"];
const FIXTURE_SEARCHES: usize = 1000;
const DOCUMENTS: usize = 8;

/// Refuses every host but Linux: the three sources of the numbers are in `/proc`.
fn linux_only(os: &str) -> Result<(), String> {
    if os == "linux" {
        Ok(())
    } else {
        Err(format!(
            "memory-run reads /proc and needs Linux; nothing was measured on {os}"
        ))
    }
}

/// The value of one `Key:   123 kB` line of a status or meminfo text.
fn proc_value(text: &str, key: &str) -> Option<u64> {
    let line = text.lines().find_map(|l| l.strip_prefix(key))?;
    let value = line.strip_prefix(':')?.split_whitespace().next()?;
    whole_number(value)
}

/// One unsigned decimal number: digits only, without a sign, not overflowing.
fn whole_number(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

/// What the read loop keeps of a process's status text.
#[derive(Debug, Default, PartialEq)]
struct Polled {
    mapped_peak_kib: u64,
    rss_last_kib: u64,
    peak_rss_kib: u64,
    threads_peak: u64,
    polls: u64,
}

impl Polled {
    /// Counts one read. A process that has given up its memory has no `Vm`
    /// lines, and such a read counts for nothing.
    fn absorb(&mut self, status: &str) {
        let (Some(peak), Some(hwm), Some(rss)) = (
            proc_value(status, "VmPeak"),
            proc_value(status, "VmHWM"),
            proc_value(status, "VmRSS"),
        ) else {
            return;
        };
        self.mapped_peak_kib = self.mapped_peak_kib.max(peak);
        self.peak_rss_kib = self.peak_rss_kib.max(hwm);
        self.rss_last_kib = rss;
        if let Some(threads) = proc_value(status, "Threads") {
            self.threads_peak = self.threads_peak.max(threads);
        }
        self.polls += 1;
    }
}

/// Minor and major faults of the waited-for children, from `/proc/self/stat`.
/// The process name may hold spaces and brackets, so the count starts after
/// the last closing bracket: field 11 is cminflt and field 13 is cmajflt.
fn child_faults(stat: &str) -> Result<(u64, u64), String> {
    let (_, rest) = stat.rsplit_once(')').ok_or("stat has no process name")?;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let field = |i: usize| {
        fields
            .get(i)
            .and_then(|v| whole_number(v))
            .ok_or("stat has too few fields")
    };
    Ok((field(8)?, field(10)?))
}

/// The one-minute load in hundredths, from the text of `/proc/loadavg`.
fn load_centi(loadavg: &str) -> Option<u64> {
    let (whole, part) = loadavg.split_whitespace().next()?.split_once('.')?;
    if part.len() != 2 {
        return None;
    }
    whole_number(whole)?
        .checked_mul(100)?
        .checked_add(whole_number(part)?)
}

/// The conditions of the host: one-minute load and available memory.
fn host_now() -> Result<(u64, u64), String> {
    let loadavg = fs::read_to_string("/proc/loadavg").map_err(|e| e.to_string())?;
    let meminfo = fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;
    Ok((
        load_centi(&loadavg).ok_or("/proc/loadavg has no load")?,
        proc_value(&meminfo, "MemAvailable").ok_or("/proc/meminfo has no MemAvailable")?,
    ))
}

fn faults_now() -> Result<(u64, u64), String> {
    child_faults(&fs::read_to_string("/proc/self/stat").map_err(|e| e.to_string())?)
}

/// The value of `key` in the `SPEED_SETUP` line of an operation's output.
fn setup_value(stdout: &str, key: &str) -> Option<u64> {
    let line = stdout.lines().find(|l| l.starts_with("SPEED_SETUP "))?;
    line.split_whitespace()
        .find_map(|field| field.strip_prefix(key)?.strip_prefix('='))
        .and_then(whole_number)
}

/// One process of one operation.
#[derive(Clone, Debug, PartialEq)]
struct Sample {
    op: String,
    rep: u64,
    /// The exit status as text: a number or the signal that ended the process.
    exit: String,
    values: [u64; MEASURES.len()],
}

fn exit_text(status: &ExitStatus) -> String {
    if let Some(code) = status.code() {
        return code.to_string();
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal-{signal}");
        }
    }
    "unknown".into()
}

/// The directories of a run and the paths of its programs.
struct Context {
    root: PathBuf,
    work: PathBuf,
    target_root: Option<PathBuf>,
    /// The pin of the saved snapshot, once it exists.
    snapshot_pin: Option<String>,
}

impl Context {
    fn documents(&self) -> Vec<PathBuf> {
        (1..=DOCUMENTS)
            .map(|n| self.work.join(format!("document-{n}.txt")))
            .collect()
    }

    fn snapshot(&self) -> PathBuf {
        self.work.join("snapshot.gelset")
    }

    fn fixtures(&self) -> Vec<PathBuf> {
        let dir = self.root.join("crates/gel-source/fixtures/evidence-lab");
        FIXTURES.iter().map(|f| dir.join(f)).collect()
    }

    /// The files whose sizes are the payload of an operation.
    fn input_files(&self, input: Input) -> Vec<PathBuf> {
        match input {
            Input::Nothing => Vec::new(),
            Input::Fixtures => self.fixtures(),
            Input::Save => self.documents(),
            Input::Load => vec![self.snapshot()],
        }
    }

    fn program(&self, op: &Operation) -> Result<PathBuf, String> {
        if op.program == Program::Control {
            return std::env::current_exe().map_err(|e| e.to_string());
        }
        let release = self
            .target_root
            .as_ref()
            .ok_or_else(|| format!("{} needs --target-root", op.name))?
            .join("release");
        let name = format!("{}{}", op.target, std::env::consts::EXE_SUFFIX);
        Ok(match op.program {
            Program::Example => release.join("examples").join(name),
            _ => release.join(name),
        })
    }

    fn arguments(&self, op: &Operation) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        if op.program == Program::Control {
            words.push(op.target.to_string());
        }
        words.extend(op.args.iter().map(|a| {
            if *a == "WORK_DIR" {
                self.work.to_string_lossy().into_owned()
            } else {
                a.to_string()
            }
        }));
        words
    }

    /// The commands an operation reads on standard input.
    fn script(&self, input: Input) -> Result<String, String> {
        let mut s = String::new();
        let add = |s: &mut String, files: &[PathBuf]| {
            for file in files {
                let _ = writeln!(s, "add {}", file.display());
            }
        };
        match input {
            Input::Nothing => {}
            Input::Fixtures => {
                add(&mut s, &self.fixtures());
                // The repeats keep the process alive past its peak, so that a
                // read of its status file falls after the peak.
                for _ in 0..FIXTURE_SEARCHES {
                    s.push_str(
                        "find ram is volatile\nfind zażółć gęślą\nfind imaginary evidence\n",
                    );
                }
            }
            Input::Save => {
                add(&mut s, &self.documents());
                let _ = writeln!(s, "save {}", self.work.join("saved.gelset").display());
            }
            Input::Load => {
                let pin = self.snapshot_pin.as_ref().ok_or("no snapshot pin")?;
                let _ = writeln!(s, "load {pin} {}", self.snapshot().display());
                s.push_str("find exact quote\n");
            }
        }
        s.push_str("exit\n");
        Ok(s)
    }
}

/// What `try_wait` said of `child`. A child that can no longer be watched is
/// ended and waited for, so that it does not outlive the run.
fn watched(
    child: &mut Child,
    state: std::io::Result<Option<ExitStatus>>,
) -> Result<Option<ExitStatus>, String> {
    state.map_err(|e| {
        let _ = child.kill();
        let _ = child.wait();
        e.to_string()
    })
}

/// Runs `command` to its end, reading its status file all the while.
fn measure(mut command: Command) -> Result<(ExitStatus, Polled, u64, u64), String> {
    let before = faults_now()?;
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let status_file = format!("/proc/{}/status", child.id());
    let mut polled = Polled::default();
    // The file is read before the wait: once the child is waited for, the
    // number may belong to another process.
    let status = loop {
        if let Ok(text) = fs::read_to_string(&status_file) {
            polled.absorb(&text);
        }
        let state = child.try_wait();
        if let Some(status) = watched(&mut child, state)? {
            break status;
        }
        std::thread::sleep(Duration::from_micros(POLL_MICROS));
    };
    let after = faults_now()?;
    let minor = after.0.checked_sub(before.0).ok_or("fault count fell")?;
    let major = after.1.checked_sub(before.1).ok_or("fault count fell")?;
    Ok((status, polled, minor, major))
}

fn run_operation(op: &Operation, context: &Context, rep: u64) -> Result<Sample, String> {
    let out_path = context.work.join("operation.out");
    let err_path = context.work.join("operation.err");
    let in_path = context.work.join("operation.in");
    let mut command = Command::new(context.program(op)?);
    command.args(context.arguments(op));
    command.current_dir(&context.work);
    command.stdout(File::create(&out_path).map_err(|e| e.to_string())?);
    command.stderr(File::create(&err_path).map_err(|e| e.to_string())?);
    if op.input == Input::Nothing {
        command.stdin(Stdio::null());
    } else {
        fs::write(&in_path, context.script(op.input)?).map_err(|e| e.to_string())?;
        command.stdin(File::open(&in_path).map_err(|e| e.to_string())?);
    }
    let (load_start, available_start) = host_now()?;
    let (status, polled, minor, major) = measure(command)?;
    let (load_end, available_end) = host_now()?;
    let stdout = fs::read(&out_path).map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&stdout);
    let payload = match op.payload {
        Payload::Output(key) => setup_value(&stdout, key),
        Payload::Files => context
            .input_files(op.input)
            .iter()
            .map(|f| fs::metadata(f).map(|m| m.len()).ok())
            .sum(),
    };
    let payload = match (payload, status.success()) {
        (Some(bytes), _) => bytes,
        // The failed exit is what the record will be refused for.
        (None, false) => 0,
        (None, true) => return Err(format!("{}: no payload in its output", op.name)),
    };
    for path in [
        out_path,
        err_path,
        in_path,
        context.work.join("saved.gelset"),
    ] {
        let _ = fs::remove_file(path);
    }
    let values = [
        payload,
        polled.mapped_peak_kib,
        polled.rss_last_kib,
        polled.peak_rss_kib,
        minor,
        major,
        polled.threads_peak,
        polled.polls,
        load_start,
        load_end,
        available_start,
        available_end,
    ];
    Ok(Sample {
        op: op.name.to_string(),
        rep,
        exit: exit_text(&status),
        values,
    })
}

/// Deterministic ASCII text of exactly `len` bytes: short words and newlines.
fn document_text(seed: u64, len: usize) -> String {
    const WORDS: [&str; 8] = [
        "gel", "source", "exact", "quote", "bundle", "root", "pin", "line",
    ];
    let mut state = seed;
    let mut s = String::with_capacity(len + 16);
    while s.len() < len {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        s.push_str(WORDS[(z % 8) as usize]);
        s.push(if z % 13 == 0 { '\n' } else { ' ' });
    }
    s.truncate(len);
    s
}

/// The value of `"key":"…"` in the JSON Lines of a batch.
fn json_text<'a>(jsonl: &'a str, key: &str) -> Option<&'a str> {
    let start = jsonl.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let rest = &jsonl[start..];
    Some(&rest[..rest.find('"')?])
}

/// Writes the generated documents and, for the load, one saved snapshot. The
/// save here is not measured.
fn prepare(context: &mut Context, operations: &[&Operation]) -> Result<(), String> {
    let needs = |input: Input| operations.iter().any(|op| op.input == input);
    if !needs(Input::Save) && !needs(Input::Load) {
        return Ok(());
    }
    for (n, path) in context.documents().iter().enumerate() {
        let text = document_text(0x2026_1107 + n as u64, MAX_TEXT / DOCUMENTS);
        fs::write(path, text).map_err(|e| e.to_string())?;
    }
    if !needs(Input::Load) {
        return Ok(());
    }
    let mut script = String::new();
    for path in context.documents() {
        let _ = writeln!(script, "add {}", path.display());
    }
    let _ = writeln!(script, "save {}\nexit", context.snapshot().display());
    let in_path = context.work.join("prepare.in");
    fs::write(&in_path, script).map_err(|e| e.to_string())?;
    let save = OPERATIONS
        .iter()
        .find(|op| op.input == Input::Save)
        .ok_or("no save operation")?;
    let output = Command::new(context.program(save)?)
        .arg("--batch")
        .stdin(File::open(&in_path).map_err(|e| e.to_string())?)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("the snapshot of the load operation could not be saved".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pin = json_text(&stdout, "bundle_sha256").ok_or("the save stated no snapshot pin")?;
    context.snapshot_pin = Some(pin.to_string());
    fs::remove_file(in_path).map_err(|e| e.to_string())
}

/// Builds every program of a run once, in release.
fn build(operations: &[&Operation], target_root: Option<&Path>, root: &Path) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for op in operations {
        if op.program == Program::Control || !seen.insert((op.package, op.target)) {
            continue;
        }
        let target_root = target_root.ok_or_else(|| format!("{} needs --target-root", op.name))?;
        let flag = match op.program {
            Program::Example => "--example",
            _ => "--bin",
        };
        let status = Command::new("cargo")
            .args([
                "build",
                "--locked",
                "--offline",
                "--release",
                "--target-dir",
            ])
            .arg(target_root)
            .args(["-p", op.package, flag, op.target])
            .current_dir(root)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("{} did not build", op.target));
        }
    }
    Ok(())
}

/// The values of one measure over the samples, as `min,median,max`. The median
/// of an even count is the lower of the two middle values.
fn triple(samples: &[Sample], index: usize) -> String {
    let mut values: Vec<u64> = samples.iter().map(|s| s.values[index]).collect();
    values.sort_unstable();
    format!(
        "{},{},{}",
        values[0],
        values[(values.len() - 1) / 2],
        values[values.len() - 1]
    )
}

fn sample_line(sample: &Sample) -> String {
    let mut line = format!(
        "sample op={} rep={} exit={}",
        sample.op, sample.rep, sample.exit
    );
    for (key, value) in MEASURES.iter().zip(sample.values) {
        let _ = write!(line, " {key}={value}");
    }
    line
}

/// What a record says of an operation besides its numbers.
#[derive(Clone, Debug, PartialEq)]
struct Description {
    op: String,
    kind: String,
    package: String,
    target: String,
    args: String,
    payload_source: String,
    resident: bool,
    workers_requested: u64,
}

fn describe(op: &Operation) -> Description {
    Description {
        op: op.name.to_string(),
        kind: match op.program {
            Program::Example => "example",
            Program::Bin => "bin",
            Program::Control => "control",
        }
        .to_string(),
        package: op.package.to_string(),
        target: op.target.to_string(),
        args: op.args.join(","),
        payload_source: match op.payload {
            Payload::Output(key) => format!("output_{key}"),
            Payload::Files => "file_sizes".to_string(),
        },
        resident: op.resident,
        workers_requested: op.workers_requested,
    }
}

fn record_line(d: &Description, samples: &[Sample]) -> String {
    let mut line = format!(
        "record op={} kind={} package={} target={} args={} payload_source={} resident={} workers_requested={} reps={}",
        d.op,
        d.kind,
        d.package,
        d.target,
        d.args,
        d.payload_source,
        if d.resident { "yes" } else { "no" },
        d.workers_requested,
        samples.len()
    );
    for (index, key) in MEASURES.iter().enumerate() {
        let _ = write!(line, " {key}={}", triple(samples, index));
    }
    line
}

/// The sample and record lines of a run, grouped by operation.
fn body(runs: &[(Description, Vec<Sample>)]) -> String {
    let mut text = String::new();
    for (_, samples) in runs {
        for sample in samples {
            text.push_str(&sample_line(sample));
            text.push('\n');
        }
    }
    for (description, samples) in runs {
        text.push_str(&record_line(description, samples));
        text.push('\n');
    }
    text
}

fn end_line(records: usize, samples: usize, redactions: usize) -> String {
    format!("end records={records} samples={samples} redactions={redactions}\n")
}

/// The fields of a line after its first word, each named in `allowed` and
/// present once. A refusal names the line and the key, never the value.
fn fields<'a>(
    number: usize,
    rest: &'a str,
    allowed: &[&str],
) -> Result<BTreeMap<&'a str, &'a str>, String> {
    let mut map = BTreeMap::new();
    for word in rest.split_whitespace() {
        let (key, value) = word
            .split_once('=')
            .ok_or(format!("line {number}: a field has no value"))?;
        if !allowed.contains(&key) {
            return Err(format!("line {number}: unknown field {key}"));
        }
        if value.is_empty() {
            return Err(format!("line {number}: field {key} is empty"));
        }
        if map.insert(key, value).is_some() {
            return Err(format!("line {number}: field {key} is repeated"));
        }
    }
    for key in allowed {
        if !map.contains_key(key) {
            return Err(format!("line {number}: field {key} is missing"));
        }
    }
    Ok(map)
}

fn number_field(number: usize, map: &BTreeMap<&str, &str>, key: &str) -> Result<u64, String> {
    map.get(key)
        .copied()
        .and_then(whole_number)
        .ok_or(format!("line {number}: field {key} is not a whole number"))
}

fn parse_sample(number: usize, rest: &str) -> Result<Sample, String> {
    let allowed: Vec<&str> = SAMPLE_KEYS.iter().chain(&MEASURES).copied().collect();
    let map = fields(number, rest, &allowed)?;
    let mut values = [0; MEASURES.len()];
    for (value, key) in values.iter_mut().zip(MEASURES) {
        *value = number_field(number, &map, key)?;
    }
    if map["exit"] != "0" {
        return Err(format!("line {number}: exit is not 0"));
    }
    if values[POLLS] == 0 {
        return Err(format!(
            "line {number}: polls is 0; the process was never read"
        ));
    }
    if values[PEAK] == 0 {
        return Err(format!("line {number}: peak_rss_kib is 0"));
    }
    Ok(Sample {
        op: map["op"].to_string(),
        rep: number_field(number, &map, "rep")?,
        exit: map["exit"].to_string(),
        values,
    })
}

/// Checks what a record line says of its operation against `operations`. A
/// refusal names the field, never the value.
fn check_description(
    number: usize,
    map: &BTreeMap<&str, &str>,
    resident: bool,
    workers_requested: u64,
    operations: &[Operation],
) -> Result<(), String> {
    let known = operations
        .iter()
        .find(|op| op.name == map["op"])
        .ok_or(format!(
            "line {number}: op is not an operation of this tool"
        ))?;
    let expected = describe(known);
    let same = [
        ("kind", map["kind"] == expected.kind),
        ("package", map["package"] == expected.package),
        ("target", map["target"] == expected.target),
        ("args", map["args"] == expected.args),
        (
            "payload_source",
            map["payload_source"] == expected.payload_source,
        ),
        ("resident", resident == expected.resident),
        (
            "workers_requested",
            workers_requested == expected.workers_requested,
        ),
    ];
    match same.iter().find(|(_, same)| !same) {
        Some((key, _)) => Err(format!(
            "line {number}: field {key} differs from the operation"
        )),
        None => Ok(()),
    }
}

/// Checks one record line against the samples of its operation.
fn check_record(
    number: usize,
    rest: &str,
    samples: &BTreeMap<String, Vec<(usize, Sample)>>,
    operations: &[Operation],
) -> Result<String, String> {
    let allowed: Vec<&str> = RECORD_KEYS.iter().chain(&MEASURES).copied().collect();
    let map = fields(number, rest, &allowed)?;
    let op = map["op"];
    let resident = match map["resident"] {
        "yes" => true,
        "no" => false,
        _ => return Err(format!("line {number}: resident is neither yes nor no")),
    };
    let workers_requested = number_field(number, &map, "workers_requested")?;
    for key in MEASURES {
        let parts: Vec<&str> = map[key].split(',').collect();
        if parts.len() != 3 || parts.iter().any(|p| whole_number(p).is_none()) {
            return Err(format!(
                "line {number}: field {key} is not three whole numbers"
            ));
        }
    }
    let rows = samples
        .get(op)
        .ok_or(format!("line {number}: the operation has no samples"))?;
    check_description(number, &map, resident, workers_requested, operations)?;
    if rows.len() < MIN_REPS {
        return Err(format!(
            "line {number}: {} samples; at least {MIN_REPS} are required",
            rows.len()
        ));
    }
    if number_field(number, &map, "reps")? != rows.len() as u64 {
        return Err(format!("line {number}: reps is not the number of samples"));
    }
    for (expected, (row, sample)) in (1u64..).zip(rows) {
        if sample.rep != expected {
            return Err(format!("line {row}: rep is out of sequence"));
        }
        if sample.values[PAYLOAD] != rows[0].1.values[PAYLOAD] {
            return Err(format!("line {row}: payload_bytes differs between samples"));
        }
        if resident && u128::from(sample.values[PEAK]) * 1024 < u128::from(sample.values[PAYLOAD]) {
            return Err(format!(
                "line {row}: peak_rss_kib is below payload_bytes of a resident operation"
            ));
        }
    }
    let plain: Vec<Sample> = rows.iter().map(|(_, s)| s.clone()).collect();
    for (index, key) in MEASURES.iter().enumerate() {
        if map[key] != triple(&plain, index) {
            return Err(format!(
                "line {number}: {key} does not recompute from the samples"
            ));
        }
    }
    Ok(op.to_string())
}

/// Checks a whole record and returns the numbers of records and samples.
pub(crate) fn check_text(text: &str) -> Result<(usize, usize), String> {
    check_against(text, &OPERATIONS)
}

fn check_against(text: &str, operations: &[Operation]) -> Result<(usize, usize), String> {
    let mut lines = text.lines().enumerate().map(|(i, l)| (i + 1, l));
    if lines.next().map(|(_, l)| l) != Some(VERSION) {
        return Err(format!("line 1: the version line is not {VERSION}"));
    }
    let mut header: BTreeSet<&str> = BTreeSet::new();
    let mut samples: BTreeMap<String, Vec<(usize, Sample)>> = BTreeMap::new();
    let mut record_rows: Vec<(usize, &str)> = Vec::new();
    let mut counts = None;
    for (number, line) in lines {
        if counts.is_some() {
            return Err(format!("line {number}: text after the end line"));
        }
        if let Some(rest) = line.strip_prefix("sample ") {
            let sample = parse_sample(number, rest)?;
            samples
                .entry(sample.op.clone())
                .or_default()
                .push((number, sample));
        } else if let Some(rest) = line.strip_prefix("record ") {
            record_rows.push((number, rest));
        } else if let Some(rest) = line.strip_prefix("end ") {
            let map = fields(number, rest, &["records", "samples", "redactions"])?;
            number_field(number, &map, "redactions")?;
            counts = Some((
                number,
                number_field(number, &map, "records")?,
                number_field(number, &map, "samples")?,
            ));
        } else {
            let key = line.split('=').next().unwrap_or_default();
            if !samples.is_empty() || !HEADER_KEYS.contains(&key) {
                return Err(format!("line {number}: not a line of this format"));
            }
            if !header.insert(key) {
                return Err(format!("line {number}: header {key} is repeated"));
            }
        }
    }
    if let Some(key) = HEADER_KEYS.iter().find(|k| !header.contains(*k)) {
        return Err(format!("the header {key} is missing"));
    }
    let (end, records, sample_count) = counts.ok_or("the end line is missing")?;
    let mut named = BTreeSet::new();
    for (number, rest) in &record_rows {
        let op = check_record(*number, rest, &samples, operations)?;
        if !named.insert(op) {
            return Err(format!("line {number}: the operation is recorded twice"));
        }
    }
    if let Some(op) = samples.keys().find(|op| !named.contains(*op)) {
        let row = samples[op][0].0;
        return Err(format!("line {row}: no record names these samples"));
    }
    let total: usize = samples.values().map(Vec::len).sum();
    if records != record_rows.len() as u64 || sample_count != total as u64 {
        return Err(format!("line {end}: the end line does not count the lines"));
    }
    if record_rows.is_empty() {
        return Err(format!("line {end}: the record holds no operation"));
    }
    Ok((record_rows.len(), total))
}

struct Options {
    out: PathBuf,
    work: PathBuf,
    target_root: Option<PathBuf>,
    reps: usize,
    only: Vec<String>,
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let (mut out, mut work, mut target_root) = (None, None, None);
    let mut reps = MIN_REPS;
    let mut only = Vec::new();
    let mut words = args.iter();
    while let Some(word) = words.next() {
        let value = words.next().ok_or(USAGE)?;
        match word.as_str() {
            "--out" => out = Some(PathBuf::from(value)),
            "--work" => work = Some(PathBuf::from(value)),
            "--target-root" => target_root = Some(PathBuf::from(value)),
            "--reps" => reps = value.parse().map_err(|_| USAGE)?,
            "--only" => only.push(value.clone()),
            _ => return Err(USAGE.into()),
        }
    }
    if !(MIN_REPS..=MAX_REPS).contains(&reps) {
        return Err(format!(
            "--reps must be {MIN_REPS} to {MAX_REPS}: a single sample is not a result"
        ));
    }
    Ok(Options {
        out: out.ok_or(USAGE)?,
        work: work.ok_or(USAGE)?,
        target_root,
        reps,
        only,
    })
}

fn selected(only: &[String]) -> Result<Vec<&'static Operation>, String> {
    if let Some(name) = only
        .iter()
        .find(|name| !OPERATIONS.iter().any(|op| op.name == name.as_str()))
    {
        return Err(format!("unknown operation {name}"));
    }
    Ok(OPERATIONS
        .iter()
        .filter(|op| only.is_empty() || only.iter().any(|name| name == op.name))
        .collect())
}

/// Refuses a directory whose parent lies inside the source checkout.
fn outside(path: &Path, root: &Path, what: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| format!("{what}: {e}"))?;
    if parent.starts_with(root) {
        return Err(format!("{what} must be outside the source checkout"));
    }
    Ok(())
}

/// The record goes to `file` only once it has passed the check, so a refused
/// run leaves no file.
fn write_record(file: &Path, text: &str) -> Result<(usize, usize), String> {
    let counts = check_text(text).map_err(|e| format!("GEL_MEMORY_RUN=REFUSED {e}"))?;
    fs::write(file, text).map_err(|e| e.to_string())?;
    Ok(counts)
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    linux_only(std::env::consts::OS)?;
    let options = parse_options(args)?;
    let operations = selected(&options.only)?;
    let root = super::workspace_root()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    outside(&options.out, &root, "--out")?;
    outside(&options.work.join("x"), &root, "--work")?;
    let base = options.work.canonicalize().map_err(|e| e.to_string())?;
    build(&operations, options.target_root.as_deref(), &root)?;
    // The build creates the target root, so this comes after it; and before
    // the output directory, so that a refusal here leaves nothing behind.
    let target_root = match &options.target_root {
        Some(dir) => Some(dir.canonicalize().map_err(|e| e.to_string())?),
        None => None,
    };
    fs::create_dir(&options.out).map_err(|e| format!("new output directory required: {e}"))?;
    let finished = measure_all(&options, &operations, &root, &base, target_root.as_deref());
    if finished.is_err() {
        // Only what this run may have written; anything else keeps the directory.
        let _ = fs::remove_file(options.out.join("MEMORY.txt"));
        let _ = fs::remove_dir(&options.out);
    }
    let (records, samples) = finished?;
    println!("GEL_MEMORY_RUN=PASS records={records} samples={samples}");
    Ok(())
}

/// Measures every operation and writes the record into the new output
/// directory; the working directory is gone when this returns.
fn measure_all(
    options: &Options,
    operations: &[&Operation],
    root: &Path,
    base: &Path,
    target_root: Option<&Path>,
) -> Result<(usize, usize), String> {
    let out = options.out.canonicalize().map_err(|e| e.to_string())?;
    let work = base.join(format!("gel-memory-{}", std::process::id()));
    fs::create_dir(&work).map_err(|e| e.to_string())?;
    let mut context = Context {
        root: root.to_path_buf(),
        work: work.clone(),
        target_root: target_root.map(Path::to_path_buf),
        snapshot_pin: None,
    };
    let measured = prepare(&mut context, operations).and_then(|()| {
        let mut runs: Vec<(Description, Vec<Sample>)> = operations
            .iter()
            .map(|op| (describe(op), Vec::new()))
            .collect();
        // Rounds, so that a change of the host falls on every operation.
        for rep in 1..=options.reps {
            for (op, (_, samples)) in operations.iter().zip(&mut runs) {
                samples.push(run_operation(op, &context, rep as u64)?);
            }
        }
        Ok(runs)
    });
    let removed = fs::remove_dir_all(&work).map_err(|e| e.to_string());
    let runs = measured?;
    removed?;

    let rustc = Command::new("rustc")
        .arg("-V")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "NOT_MEASURED".into());
    let mut facts = format!("{VERSION}\n");
    let _ = writeln!(
        facts,
        "tool=xtask-memory version={}",
        env!("CARGO_PKG_VERSION")
    );
    let _ = writeln!(facts, "{}", super::bench_compare::revision(root));
    facts.push_str(&super::host_report::facts());
    let _ = writeln!(facts, "rustc={rustc}");
    let _ = writeln!(
        facts,
        "profile=release rustflags_set={}",
        if std::env::var_os("RUSTFLAGS").is_some() {
            "yes"
        } else {
            "no"
        }
    );
    let _ = writeln!(facts, "poll_pause_us={POLL_MICROS}");
    facts.push_str("note=process-wide numbers of one machine, not pinned; no time is recorded\n");
    let mut dirs = vec![root, out.as_path(), base];
    dirs.extend(target_root);
    let private = super::host_report::Private::of_this_host(&dirs);
    let (mut text, redactions) = super::host_report::finish(&facts, &body(&runs), &private)
        .map_err(|e| format!("GEL_MEMORY_RUN={e}"))?;
    let sample_count: usize = runs.iter().map(|(_, s)| s.len()).sum();
    text.push_str(&end_line(runs.len(), sample_count, redactions));
    write_record(&out.join("MEMORY.txt"), &text)
}

pub(crate) fn check_cmd(args: &[String]) -> Result<(), String> {
    let [file] = args else {
        return Err(USAGE.into());
    };
    let text = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let (records, samples) =
        check_text(&text).map_err(|e| format!("GEL_MEMORY_CHECK=REFUSED {e}"))?;
    println!("GEL_MEMORY_CHECK=PASS records={records} samples={samples}");
    Ok(())
}

/// `memory-touch BYTES`: the control of the instrument. Writes every byte of a
/// block, states its size, gives the block back and waits, so that the status
/// file shows a peak of at least BYTES and a last resident set below it.
pub(crate) fn touch(args: &[String]) -> Result<(), String> {
    let [bytes] = args else {
        return Err("usage: memory-touch BYTES".into());
    };
    let bytes = whole_number(bytes).ok_or("usage: memory-touch BYTES")?;
    let block = vec![1u8; bytes as usize];
    println!("SPEED_SETUP example=memory_touch runs=1 touched_bytes={bytes}");
    std::hint::black_box(&block);
    drop(block);
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(op: &str, rep: u64) -> Sample {
        Sample {
            op: op.to_string(),
            rep,
            exit: "0".to_string(),
            values: [
                1_048_576,
                9000 + rep,
                1500,
                2000 + rep,
                300,
                0,
                1,
                40,
                812,
                815,
                90_000,
                90_100,
            ],
        }
    }

    /// The operations of these records: `a` holds its payload, `b` need not.
    const TABLE: [Operation; 2] = [
        Operation {
            name: "a",
            program: Program::Example,
            package: "gel-core",
            target: "speed_crc",
            args: &["1", "WORK_DIR"],
            input: Input::Nothing,
            payload: Payload::Output("record_bytes"),
            resident: true,
            workers_requested: 1,
        },
        Operation {
            name: "b",
            program: Program::Example,
            package: "gel-core",
            target: "speed_crc",
            args: &["1", "WORK_DIR"],
            input: Input::Nothing,
            payload: Payload::Output("record_bytes"),
            resident: false,
            workers_requested: 1,
        },
    ];

    fn check(text: &str) -> Result<(usize, usize), String> {
        check_against(text, &TABLE)
    }

    const FACTS: &str = "GEL_MEMORY_1\ntool=xtask-memory version=0\nrevision=abc working_tree=clean\nos=linux arch=x86_64\nos_version=test\nkernel=7\ncpu_model=test cpu\nlogical_cpus=8 available_parallelism=8\nmemory_total_kib=100\nrustc=rustc 1.85.0\nprofile=release rustflags_set=no\npoll_pause_us=100\nnote=one machine\n";

    /// A whole record of `op` built by the writer, after `change` touched its
    /// samples.
    fn record_of(op: &Operation, change: impl Fn(&mut Vec<Sample>)) -> String {
        let mut samples: Vec<Sample> = (1..=5).map(|rep| sample(op.name, rep)).collect();
        change(&mut samples);
        let total = samples.len();
        let body = body(&[(describe(op), samples)]);
        format!("{FACTS}{body}{}", end_line(1, total, 0))
    }

    fn record_text(change: impl Fn(&mut Vec<Sample>)) -> String {
        record_of(&TABLE[0], change)
    }

    fn refused(text: &str, reason: &str) {
        let error = check(text).expect_err("the record must be refused");
        assert!(error.contains(reason), "{error}");
    }

    #[test]
    fn a_complete_record_passes_and_is_counted() {
        assert_eq!(check(&record_text(|_| {})), Ok((1, 5)));
        // The trailing newline of a file changes nothing.
        let text = record_text(|_| {});
        assert_eq!(check(text.trim_end()), Ok((1, 5)));
    }

    #[test]
    fn a_record_must_describe_its_operation_the_way_this_tool_does() {
        for op in &OPERATIONS {
            let text = record_of(op, |_| {});
            assert_eq!(check_text(&text), Ok((1, 5)), "{}", op.name);
            let d = describe(op);
            let resident = if d.resident { "yes" } else { "no" };
            let workers = d.workers_requested.to_string();
            let fields = [
                ("kind", d.kind.as_str()),
                ("package", d.package.as_str()),
                ("target", d.target.as_str()),
                ("args", d.args.as_str()),
                ("payload_source", d.payload_source.as_str()),
                ("resident", resident),
                ("workers_requested", workers.as_str()),
            ];
            for (key, value) in fields {
                let changed = match key {
                    "resident" if value == "yes" => "no".to_string(),
                    "resident" => "yes".to_string(),
                    "workers_requested" => "977".to_string(),
                    _ => format!("{value}x"),
                };
                let text = text.replacen(
                    &format!(" {key}={value} "),
                    &format!(" {key}={changed} "),
                    1,
                );
                let error = check_text(&text).expect_err(key);
                assert!(
                    error.contains(&format!("field {key} differs from the operation")),
                    "{error}"
                );
                assert!(!error.contains(&changed), "the value is named: {error}");
            }
        }
        // Samples and record agree, and the tool knows no such operation.
        let text = record_text(|_| {});
        let unknown = check_text(&text).unwrap_err();
        assert!(unknown.contains("op is not an operation"), "{unknown}");
    }

    #[test]
    fn min_median_and_max_are_taken_from_the_sorted_values() {
        let samples: Vec<Sample> = [50, 10, 40, 20, 30]
            .iter()
            .enumerate()
            .map(|(i, peak)| {
                let mut s = sample("a", i as u64 + 1);
                s.values[PEAK] = *peak;
                s
            })
            .collect();
        assert_eq!(triple(&samples, PEAK), "10,30,50");
        assert_eq!(triple(&samples[..4], PEAK), "10,20,50");
    }

    #[test]
    fn the_version_line_must_be_first_and_exact() {
        let text = record_text(|_| {});
        refused(
            &text.replacen("GEL_MEMORY_1", "GEL_MEMORY_0", 1),
            "version line",
        );
        refused(&format!("\n{text}"), "version line");
    }

    #[test]
    fn a_missing_field_is_refused_in_every_line_kind() {
        let text = record_text(|_| {});
        for key in MEASURES.iter().chain(&SAMPLE_KEYS) {
            let line = text.lines().find(|l| l.starts_with("sample ")).unwrap();
            let shorter: Vec<&str> = line
                .split(' ')
                .filter(|w| !w.starts_with(&format!("{key}=")))
                .collect();
            refused(
                &text.replacen(line, &shorter.join(" "), 1),
                &format!("field {key} is missing"),
            );
        }
        for key in RECORD_KEYS.iter().chain(&MEASURES) {
            let line = text.lines().find(|l| l.starts_with("record ")).unwrap();
            let shorter: Vec<&str> = line
                .split(' ')
                .filter(|w| !w.starts_with(&format!("{key}=")))
                .collect();
            refused(
                &text.replacen(line, &shorter.join(" "), 1),
                &format!("field {key} is missing"),
            );
        }
        refused(
            &text.replace(" redactions=0", ""),
            "field redactions is missing",
        );
    }

    #[test]
    fn a_missing_or_repeated_header_is_refused() {
        let text = record_text(|_| {});
        for key in HEADER_KEYS {
            let line = text
                .lines()
                .find(|l| l.starts_with(&format!("{key}=")))
                .unwrap();
            refused(
                &text.replacen(&format!("{line}\n"), "", 1),
                &format!("header {key} is missing"),
            );
            refused(
                &text.replacen(&format!("{line}\n"), &format!("{line}\n{line}\n"), 1),
                &format!("header {key} is repeated"),
            );
        }
    }

    #[test]
    fn a_non_zero_exit_is_refused() {
        for exit in ["1", "signal-9", "unknown"] {
            let text = record_text(|s| s[2].exit = exit.to_string());
            refused(&text, "exit is not 0");
        }
    }

    #[test]
    fn a_malformed_number_is_refused() {
        let text = record_text(|_| {});
        for bad in ["-1", "+1", "1.5", "12kB", "0x10", "18446744073709551616"] {
            let line = text.replacen("peak_rss_kib=2001", &format!("peak_rss_kib={bad}"), 1);
            refused(&line, "field peak_rss_kib is not a whole number");
            let summary = text.replacen(
                "peak_rss_kib=2001,2003,2005",
                &format!("peak_rss_kib=2001,{bad},2005"),
                1,
            );
            refused(&summary, "field peak_rss_kib is not three whole numbers");
        }
        refused(
            &text.replacen("peak_rss_kib=2001,2003,2005", "peak_rss_kib=2001,2005", 1),
            "field peak_rss_kib is not three whole numbers",
        );
        refused(&text.replacen("rep=2", "rep=two", 1), "field rep");
        refused(&text.replacen("reps=5", "reps=five", 1), "field reps");
        refused(
            &text.replacen("workers_requested=1", "workers_requested=one", 1),
            "field workers_requested",
        );
        refused(
            &text.replacen("records=1", "records=one", 1),
            "field records",
        );
        refused(
            &text.replacen("samples=5", "samples=five", 1),
            "field samples",
        );
        refused(
            &text.replacen("redactions=0", "redactions=none", 1),
            "field redactions",
        );
    }

    #[test]
    fn a_peak_below_the_payload_of_a_resident_operation_is_refused() {
        // 1 MiB of payload needs 1024 KiB of resident memory; the boundary passes.
        let at = record_text(|s| s[1].values[PEAK] = 1024);
        assert_eq!(check(&at), Ok((1, 5)));
        let below = record_text(|s| s[1].values[PEAK] = 1023);
        refused(&below, "below payload_bytes of a resident operation");
        // An operation that need not hold its payload may stay below it.
        let free = record_of(&TABLE[1], |s| s[1].values[PEAK] = 1023);
        assert_eq!(check(&free), Ok((1, 5)));
        // A record cannot call a resident operation free to get past the check.
        refused(
            &below.replace("resident=yes", "resident=no"),
            "field resident differs from the operation",
        );
        refused(
            &below.replace("resident=yes", "resident=maybe"),
            "resident is neither yes nor no",
        );
        // A number that would overflow when multiplied is still compared.
        let huge = record_text(|s| {
            for sample in s {
                sample.values[PAYLOAD] = u64::MAX;
                sample.values[PEAK] = 1 << 54;
            }
        });
        assert_eq!(check(&huge), Ok((1, 5)));
    }

    #[test]
    fn a_process_that_was_never_read_is_refused() {
        refused(
            &record_text(|s| s[0].values[POLLS] = 0),
            "the process was never read",
        );
        refused(&record_text(|s| s[0].values[PEAK] = 0), "peak_rss_kib is 0");
    }

    #[test]
    fn fewer_than_five_samples_are_refused() {
        for count in [1, 4] {
            let text = record_text(|s| s.truncate(count));
            refused(&text, "at least 5 are required");
        }
    }

    #[test]
    fn repetitions_must_run_in_sequence() {
        refused(&record_text(|s| s[3].rep = 3), "rep is out of sequence");
    }

    #[test]
    fn a_payload_that_changes_between_samples_is_refused() {
        refused(
            &record_text(|s| s[4].values[PAYLOAD] += 1),
            "payload_bytes differs",
        );
    }

    #[test]
    fn a_summary_that_does_not_recompute_is_refused() {
        let text = record_text(|_| {});
        let line = text.lines().find(|l| l.starts_with("record ")).unwrap();
        for key in MEASURES {
            let summary = line
                .split(' ')
                .find_map(|w| w.strip_prefix(&format!("{key}=")))
                .unwrap();
            let parts: Vec<&str> = summary.split(',').collect();
            // The median moves; min and max stay.
            let changed = format!("{},{}9,{}", parts[0], parts[1], parts[2]);
            refused(
                &text.replacen(&format!("{key}={summary}"), &format!("{key}={changed}"), 1),
                &format!("{key} does not recompute"),
            );
        }
        refused(
            &text.replacen("reps=5", "reps=6", 1),
            "reps is not the number of samples",
        );
    }

    #[test]
    fn records_and_samples_must_belong_together() {
        let text = record_text(|_| {});
        refused(
            &text.replace("record op=a", "record op=b"),
            "the operation has no samples",
        );
        let record = text.lines().find(|l| l.starts_with("record ")).unwrap();
        refused(
            &text.replacen(&format!("{record}\n"), "", 1),
            "no record names these samples",
        );
        refused(
            &text.replacen(&format!("{record}\n"), &format!("{record}\n{record}\n"), 1),
            "recorded twice",
        );
    }

    #[test]
    fn a_cut_or_extended_record_is_refused() {
        let text = record_text(|_| {});
        let cut = text.find("end records").unwrap();
        refused(&text[..cut], "the end line is missing");
        refused(&format!("{text}sample op=a\n"), "text after the end line");
        refused(
            &text.replacen("end records=1", "end records=2", 1),
            "does not count the lines",
        );
        refused(
            &text.replacen("samples=5", "samples=4", 1),
            "does not count the lines",
        );
    }

    #[test]
    fn a_line_outside_the_format_is_refused() {
        let text = record_text(|_| {});
        let timing = text.replacen("sample op=a", "elapsed_seconds=3\nsample op=a", 1);
        refused(&timing, "not a line of this format");
        let extra = text.replacen("rep=1", "rep=1 elapsed_ns=3", 1);
        refused(&extra, "unknown field elapsed_ns");
        refused(&text.replacen("rep=1", "rep=1 rep=1", 1), "repeated");
        refused(&text.replacen("rep=1", "rep=", 1), "field rep is empty");
        refused(&text.replacen("rep=1", "rep", 1), "a field has no value");
        let after = text.replacen("record op=a", "kernel=8\nrecord op=a", 1);
        refused(&after, "not a line of this format");
        refused(
            &format!("{FACTS}{}", end_line(0, 0, 0)),
            "the record holds no operation",
        );
    }

    #[test]
    fn status_text_gives_the_peak_the_last_rss_and_the_threads() {
        let first = "Name:\tx\nVmPeak:\t  900 kB\nVmHWM:\t 700 kB\nVmRSS:\t 600 kB\nThreads:\t3\n";
        let later = "VmPeak:\t 1000 kB\nVmHWM:\t 800 kB\nVmRSS:\t 100 kB\nThreads:\t1\n";
        let mut polled = Polled::default();
        polled.absorb(first);
        polled.absorb(later);
        // An exiting process has no memory lines; nothing changes and it is not counted.
        polled.absorb("Name:\tx\nThreads:\t1\n");
        assert_eq!(
            polled,
            Polled {
                mapped_peak_kib: 1000,
                rss_last_kib: 100,
                peak_rss_kib: 800,
                threads_peak: 3,
                polls: 2,
            }
        );
        assert_eq!(
            proc_value("MemAvailable:   123 kB\n", "MemAvailable"),
            Some(123)
        );
        assert_eq!(proc_value("VmHWMx:  1 kB\n", "VmHWM"), None);
    }

    #[test]
    fn the_stat_line_gives_the_faults_of_waited_children() {
        // pid (comm) state ppid pgrp session tty tpgid flags
        // minflt cminflt majflt cmajflt utime
        let stat = "42 (a) b) c) R 1 2 3 4 5 6 700 800 900 1000 11\n";
        assert_eq!(child_faults(stat), Ok((800, 1000)));
        assert!(child_faults("42 (a) R 1 2").is_err());
        assert!(child_faults("no name").is_err());
    }

    #[test]
    fn the_load_is_read_in_hundredths() {
        assert_eq!(load_centi("8.39 4.76 3.12 2/100 7\n"), Some(839));
        assert_eq!(load_centi("0.05 0 0"), Some(5));
        assert_eq!(load_centi("8.3 1 1"), None);
        assert_eq!(load_centi("8 1 1"), None);
        assert_eq!(load_centi(""), None);
    }

    #[test]
    fn the_payload_is_read_from_the_setup_line_only() {
        let out = "note record_bytes=1\nSPEED_SETUP example=e runs=1 bank=2 record_bytes=9437184\n";
        assert_eq!(setup_value(out, "record_bytes"), Some(9_437_184));
        assert_eq!(setup_value(out, "bytes"), None);
        assert_eq!(setup_value("record_bytes=1\n", "record_bytes"), None);
    }

    #[test]
    fn a_single_sample_is_refused_before_anything_runs() {
        let words =
            |text: &str| -> Vec<String> { text.split_whitespace().map(str::to_string).collect() };
        let base = "--out /o --work /w --reps ";
        for bad in ["0", "1", "4", "100"] {
            let error = parse_options(&words(&format!("{base}{bad}")));
            assert!(error.err().unwrap().contains("a single sample"));
        }
        assert_eq!(parse_options(&words("--out /o --work /w")).unwrap().reps, 5);
        assert!(parse_options(&words("--out /o")).is_err());
        assert!(parse_options(&words("--out /o --work /w --reps")).is_err());
        assert!(selected(&["nothing".to_string()]).is_err());
        assert_eq!(selected(&[]).unwrap().len(), OPERATIONS.len());
        let one = selected(&["q8_read_8192".to_string()]).unwrap();
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn other_systems_are_refused_by_name() {
        assert_eq!(linux_only("linux"), Ok(()));
        for os in ["windows", "macos", "freebsd"] {
            let error = linux_only(os).unwrap_err();
            assert!(error.contains("needs Linux"), "{error}");
            assert!(error.contains(os), "{error}");
        }
    }

    /// The child of the next test: it lives until its input ends. Ignored in a
    /// normal run, and it does nothing unless that test sets its variable.
    #[test]
    #[ignore]
    fn memory_waiting_child() {
        if std::env::var_os("GEL_MEMORY_WAITING_CHILD").is_some() {
            let _ = std::io::copy(&mut std::io::stdin(), &mut std::io::sink());
        }
    }

    #[test]
    fn a_child_that_can_no_longer_be_watched_is_ended_and_reaped() {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "memory::tests::memory_waiting_child",
                "--exact",
                "--ignored",
            ])
            .env("GEL_MEMORY_WAITING_CHILD", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let status_file = format!("/proc/{}/status", child.id());
        // An answer passes through and leaves the child alone.
        assert_eq!(watched(&mut child, Ok(None)), Ok(None));
        assert_eq!(child.try_wait().unwrap(), None);

        let failed = Err(std::io::Error::other("no answer"));
        assert_eq!(watched(&mut child, failed), Err("no answer".to_string()));
        // Waiting drops the input as well, and that alone would end it well.
        let status = child.try_wait().unwrap().expect("the child was waited for");
        assert!(!status.success(), "{status}");
        if cfg!(target_os = "linux") {
            assert!(!Path::new(&status_file).exists(), "a zombie stayed");
        }
    }

    #[test]
    fn a_record_refused_by_the_check_is_not_written() {
        let dir = std::env::temp_dir().join(format!("gel-memory-unit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        let file = dir.join("MEMORY.txt");
        let good = record_of(&OPERATIONS[0], |_| {});

        let bad = good.replacen("exit=0", "exit=1", 1);
        let error = write_record(&file, &bad).unwrap_err();
        let unwritten = file.exists();
        assert_eq!(write_record(&file, &good), Ok((1, 5)));
        let written = fs::read_to_string(&file);
        fs::remove_dir_all(&dir).unwrap();

        assert!(error.starts_with("GEL_MEMORY_RUN=REFUSED"), "{error}");
        assert!(!unwritten, "a refused record was written");
        assert_eq!(written.unwrap(), good);
    }

    #[test]
    fn every_operation_describes_itself_in_one_word_per_field() {
        for op in &OPERATIONS {
            let d = describe(op);
            assert!(!d.args.is_empty(), "{}", d.op);
            for value in [&d.op, &d.kind, &d.package, &d.target, &d.args] {
                assert!(!value.chars().any(char::is_whitespace), "{value}");
                assert!(!value.contains('/'), "{value}");
            }
        }
    }

    #[test]
    fn generated_documents_fit_the_collection_and_repeat() {
        assert_eq!(DOCUMENTS * (MAX_TEXT / DOCUMENTS), MAX_TEXT);
        let text = document_text(7, 4096);
        assert_eq!(text.len(), 4096);
        assert_eq!(text, document_text(7, 4096));
        assert_ne!(text, document_text(8, 4096));
    }

    #[test]
    fn the_pin_is_read_from_the_save_record() {
        let line = "{\"status\":\"OK\",\"bundle_sha256\":\"ab12\",\"x\":1}\n";
        assert_eq!(json_text(line, "bundle_sha256"), Some("ab12"));
        assert_eq!(json_text(line, "absent"), None);
    }
}
