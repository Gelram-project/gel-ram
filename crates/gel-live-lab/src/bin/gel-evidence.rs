//! Offline multi-document source laboratory. No shell, LLM or background import.
#![forbid(unsafe_code)]
use gel_live_lab::{parse_pin, safe};
use gel_source::{
    collection::{Collection, Hit},
    digest, document, hex, read_regular,
};
use std::{
    io::{self, BufRead, Read, Write},
    path::Path,
    time::Instant,
};

const HELP: &str = "add PATH | replace ID PATH | drop ID | list | find PHRASE | proof N | save NEW_PATH | load TRUSTED_SHA256 PATH | exit\nExact source phrases, not semantic AI. All files plaintext. Paths with spaces need no quotes.";
/// Output schema of `--batch`; see docs/EVIDENCE-BATCH.md.
const SCHEMA: &str = "gel-evidence/1";
const COMMAND_LIMIT: u64 = 4096;

struct Listed {
    id: u64,
    title: String,
    bytes: usize,
    sha256: String,
}

struct Found {
    doc: u64,
    title: String,
    start: usize,
    end: usize,
    quote: String,
    context_start: usize,
    context_end: usize,
    omitted_before: bool,
    omitted_after: bool,
    context: String,
}

/// What one command did, independent of how it is displayed.
enum Outcome {
    Exit,
    Help,
    Added {
        id: u64,
        bytes: usize,
    },
    Replaced {
        id: u64,
    },
    Removed {
        id: u64,
    },
    Listed(Vec<Listed>),
    Searched {
        status: &'static str,
        documents: usize,
        matching_lines: usize,
        skipped_lines: usize,
        search_ns: u128,
        results: Vec<Found>,
    },
    Proved {
        document_sha256: String,
        collection_sha256: String,
    },
    Saved {
        revision: u64,
        save_ns: u128,
        pin: String,
    },
    Reopened {
        revision: u64,
        load_ns: u128,
    },
}

struct Session {
    bank: Collection,
    hits: Vec<Hit>,
}
impl Session {
    fn execute(&mut self, input: &str) -> Result<Outcome, String> {
        let (cmd, arg) = input.trim().split_once(' ').unwrap_or((input.trim(), ""));
        let arg = arg.trim();
        Ok(match cmd {
            "exit" | "quit" if arg.is_empty() => Outcome::Exit,
            "help" if arg.is_empty() => Outcome::Help,
            "add" if !arg.is_empty() => {
                let p = Path::new(arg);
                let b = read_regular(p, document::MAX_TEXT).map_err(|e| format!("{e:?}"))?;
                let text = std::str::from_utf8(&b).map_err(|_| "UTF8_REQUIRED")?;
                let title = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("UTF8_FILENAME_REQUIRED")?;
                let id = self.bank.add(title, text)?;
                self.hits.clear();
                Outcome::Added { id, bytes: b.len() }
            }
            "replace" => {
                let (id, p) = arg.split_once(' ').ok_or("replace ID PATH")?;
                let id = id.parse().map_err(|_| "INVALID_ID")?;
                let b = read_regular(Path::new(p.trim()), document::MAX_TEXT)
                    .map_err(|e| format!("{e:?}"))?;
                self.bank
                    .replace(id, std::str::from_utf8(&b).map_err(|_| "UTF8_REQUIRED")?)?;
                self.hits.clear();
                Outcome::Replaced { id }
            }
            "drop" => {
                let id = arg.parse().map_err(|_| "INVALID_ID")?;
                self.bank.remove(id)?;
                self.hits.clear();
                Outcome::Removed { id }
            }
            "list" if arg.is_empty() => Outcome::Listed(
                self.bank
                    .documents()
                    .map(|d| Listed {
                        id: d.id(),
                        title: d.title().to_string(),
                        bytes: d.text().len(),
                        sha256: hex(&d.hash()),
                    })
                    .collect(),
            ),
            "find" => {
                let start = Instant::now();
                let result = self.bank.search(arg)?;
                let search_ns = start.elapsed().as_nanos();
                let mut results = Vec::with_capacity(result.hits.len());
                for h in &result.hits {
                    let d = self.bank.get(h.document_id()).ok_or("MISSING_DOCUMENT")?;
                    self.bank.validate(h)?;
                    let c = gel_source::context::surrounding(d.text(), h.span(), 512)?;
                    results.push(Found {
                        doc: d.id(),
                        title: d.title().to_string(),
                        start: h.span().start,
                        end: h.span().end,
                        quote: h.quote().to_string(),
                        context_start: c.context_span.start,
                        context_end: c.context_span.end,
                        omitted_before: c.omitted_before,
                        omitted_after: c.omitted_after,
                        context: d.text()[c.context_span.clone()].to_string(),
                    });
                }
                let outcome = Outcome::Searched {
                    status: result.status(),
                    documents: result.documents_examined,
                    matching_lines: result.matching_lines,
                    skipped_lines: result.skipped_long_lines,
                    search_ns,
                    results,
                };
                self.hits = result.hits;
                outcome
            }
            "proof" => {
                let n = arg.parse::<usize>().map_err(|_| "INVALID_RESULT")?;
                let h = self
                    .hits
                    .get(n.checked_sub(1).ok_or("RESULTS_START_AT_1")?)
                    .ok_or("NO_CURRENT_RESULT")?;
                self.bank.validate(h)?;
                Outcome::Proved {
                    document_sha256: hex(&h.document_hash()),
                    collection_sha256: hex(&h.root()),
                }
            }
            "save" if !arg.is_empty() => {
                let start = Instant::now();
                let pin = self.bank.save_new(Path::new(arg))?;
                Outcome::Saved {
                    revision: self.bank.revision(),
                    save_ns: start.elapsed().as_nanos(),
                    pin: hex(&pin),
                }
            }
            "load" => {
                let (pin, p) = arg.split_once(' ').ok_or("load TRUSTED_SHA256 PATH")?;
                if p.trim().is_empty() {
                    return Err("MISSING_PATH".into());
                }
                let start = Instant::now();
                self.bank = Collection::load(Path::new(p.trim()), parse_pin(pin)?)?;
                self.hits.clear();
                Outcome::Reopened {
                    revision: self.bank.revision(),
                    load_ns: start.elapsed().as_nanos(),
                }
            }
            _ => return Err("Unknown command; type help".into()),
        })
    }
    /// Interactive and demo rendering; the text is unchanged from earlier releases.
    fn command(&mut self, input: &str) -> Result<bool, String> {
        let outcome = self.execute(input)?;
        human(&outcome);
        Ok(!matches!(outcome, Outcome::Exit))
    }
}

fn human(outcome: &Outcome) {
    match outcome {
        Outcome::Exit => {}
        Outcome::Help => println!("{HELP}"),
        Outcome::Added { id, bytes } => println!("ADDED id={id} bytes={bytes}"),
        Outcome::Replaced { id } => println!("REPLACED id={id}; previous citations invalidated"),
        Outcome::Removed { id } => println!("REMOVED id={id}; old snapshots remain on disk"),
        Outcome::Listed(rows) => {
            for r in rows {
                println!(
                    "id={} title={} bytes={} sha256={}",
                    r.id,
                    safe(&r.title),
                    r.bytes,
                    r.sha256
                );
            }
        }
        Outcome::Searched {
            status,
            documents,
            matching_lines,
            skipped_lines,
            search_ns,
            results,
        } => {
            println!("FIND={status} documents={documents} matching_lines={matching_lines} shown={} skipped_lines={skipped_lines} search_ns={search_ns}", results.len());
            for (n, f) in results.iter().enumerate() {
                println!(
                    "RESULT {} doc={} title={} bytes={}..{}\nQUOTE {}",
                    n + 1,
                    f.doc,
                    safe(&f.title),
                    f.start,
                    f.end,
                    safe(&f.quote)
                );
                println!("CONTEXT bytes={}..{} omitted_before={} omitted_after={} bounded_not_complete_sentence=true\n{}", f.context_start, f.context_end, f.omitted_before, f.omitted_after, safe(&f.context));
            }
        }
        Outcome::Proved {
            document_sha256,
            collection_sha256,
        } => println!("CITATION=PASS document_sha256={document_sha256} collection_sha256={collection_sha256}; correspondence, not truth"),
        Outcome::Saved {
            revision,
            save_ns,
            pin,
        } => println!("SAVED revision={revision} save_ns={save_ns}\nBUNDLE_SHA256={pin}; retain independently"),
        Outcome::Reopened { revision, load_ns } => {
            println!("REOPEN=PASS revision={revision} load_ns={load_ns}")
        }
    }
}

/// Strict JSON string: every Unicode scalar is kept; only the characters JSON
/// requires are escaped, so the quoted bytes survive exactly.
fn json_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// One JSON object built field by field, in a fixed order.
struct Object(String);
impl Object {
    fn bare() -> Self {
        Self(String::from("{"))
    }
    fn record(kind: &str) -> Self {
        let mut o = Self::bare();
        o.text("schema", SCHEMA).text("record", kind);
        o
    }
    fn key(&mut self, key: &str) -> &mut Self {
        if self.0.len() > 1 {
            self.0.push(',');
        }
        json_string(&mut self.0, key);
        self.0.push(':');
        self
    }
    fn text(&mut self, key: &str, value: &str) -> &mut Self {
        self.key(key);
        json_string(&mut self.0, value);
        self
    }
    /// Numbers and booleans, whose Display form is valid JSON.
    fn value(&mut self, key: &str, value: impl std::fmt::Display) -> &mut Self {
        self.key(key).0.push_str(&value.to_string());
        self
    }
    fn raw(&mut self, key: &str, json: &str) -> &mut Self {
        self.key(key).0.push_str(json);
        self
    }
    fn finish(&mut self) -> String {
        self.0.push('}');
        std::mem::take(&mut self.0)
    }
}

fn array(items: impl Iterator<Item = String>) -> String {
    format!("[{}]", items.collect::<Vec<_>>().join(","))
}

/// The batch record of one executed command, and its status.
fn record(seq: u64, command: &str, outcome: &Outcome) -> (&'static str, String) {
    let mut o = Object::record("result");
    o.value("seq", seq).text("command", command);
    let status = match outcome {
        Outcome::Searched { status, .. } => *status,
        _ => "OK",
    };
    o.text("status", status);
    match outcome {
        Outcome::Exit => {}
        Outcome::Help => {
            o.text("help", HELP);
        }
        Outcome::Added { id, bytes } => {
            o.value("id", id).value("bytes", bytes);
        }
        Outcome::Replaced { id } => {
            o.value("id", id)
                .value("previous_citations_invalidated", true);
        }
        Outcome::Removed { id } => {
            o.value("id", id)
                .value("old_snapshots_remain_on_disk", true);
        }
        Outcome::Listed(rows) => {
            o.raw(
                "documents",
                &array(rows.iter().map(|r| {
                    Object::bare()
                        .value("id", r.id)
                        .text("title", &r.title)
                        .value("bytes", r.bytes)
                        .text("sha256", &r.sha256)
                        .finish()
                })),
            );
        }
        Outcome::Searched {
            documents,
            matching_lines,
            skipped_lines,
            search_ns,
            results,
            ..
        } => {
            o.value("documents_examined", documents)
                .value("matching_lines", matching_lines)
                .value("shown", results.len())
                .value("skipped_long_lines", skipped_lines)
                .value("search_ns", search_ns);
            o.raw(
                "results",
                &array(results.iter().enumerate().map(|(n, f)| {
                    let context = Object::bare()
                        .value("start", f.context_start)
                        .value("end", f.context_end)
                        .value("omitted_before", f.omitted_before)
                        .value("omitted_after", f.omitted_after)
                        .text("text", &f.context)
                        .finish();
                    Object::bare()
                        .value("n", n + 1)
                        .value("doc", f.doc)
                        .text("title", &f.title)
                        .value("start", f.start)
                        .value("end", f.end)
                        .text("quote", &f.quote)
                        .text("quote_sha256", &hex(&digest(f.quote.as_bytes())))
                        .raw("context", &context)
                        .finish()
                })),
            );
        }
        Outcome::Proved {
            document_sha256,
            collection_sha256,
        } => {
            o.text("document_sha256", document_sha256)
                .text("collection_sha256", collection_sha256)
                .value("correspondence_not_truth", true);
        }
        Outcome::Saved {
            revision,
            save_ns,
            pin,
        } => {
            o.value("revision", revision)
                .value("save_ns", save_ns)
                .text("bundle_sha256", pin);
        }
        Outcome::Reopened { revision, load_ns } => {
            o.value("revision", revision).value("load_ns", load_ns);
        }
    }
    (status, o.finish())
}

/// Reads one input line of at most COMMAND_LIMIT bytes plus its newline.
/// Returns None at end of input.
fn next_line(input: &mut impl BufRead) -> Result<Option<Vec<u8>>, String> {
    let mut line = Vec::new();
    let n = input
        .by_ref()
        .take(COMMAND_LIMIT + 1)
        .read_until(b'\n', &mut line)
        .map_err(|e| format!("stdin: {e}"))?;
    Ok((n > 0).then_some(line))
}

/// Skips the rest of an over-long line, so it is not counted as another command.
fn discard_line(input: &mut impl BufRead) -> Result<(), String> {
    while let Some(chunk) = next_line(input)? {
        if chunk.last() == Some(&b'\n') {
            break;
        }
    }
    Ok(())
}

/// Counts the remaining non-empty input lines without executing them.
fn count_remaining(input: &mut impl BufRead) -> Result<u64, String> {
    let mut lines = 0;
    let mut open = false;
    while let Some(chunk) = next_line(input)? {
        let ends = chunk.last() == Some(&b'\n');
        let blank = chunk.iter().all(u8::is_ascii_whitespace);
        if !blank {
            open = true;
        }
        if ends || chunk.len() <= COMMAND_LIMIT as usize {
            if open {
                lines += 1;
            }
            open = false;
        }
    }
    Ok(lines + u64::from(open))
}

const STATUSES: [&str; 5] = ["OK", "HIT", "UNKNOWN", "INCOMPLETE", "ERROR"];

/// Non-interactive mode: commands on stdin, JSON Lines on stdout, diagnostics
/// on stderr. Stops at the first ERROR. Returns the process exit code.
fn batch(s: &mut Session, input: &mut impl BufRead, out: &mut impl Write) -> Result<i32, String> {
    let mut emit = |line: String| writeln!(out, "{line}").map_err(|e| format!("stdout: {e}"));
    emit(
        Object::record("session")
            .text("tool", "gel-evidence")
            .text("version", env!("CARGO_PKG_VERSION"))
            .finish(),
    )?;
    let mut counts = [0u64; 5];
    let mut seq = 0;
    let mut not_run = 0;
    while let Some(bytes) = next_line(input)? {
        if bytes.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        seq += 1;
        let line = String::from_utf8_lossy(&bytes);
        let command = line.split_whitespace().next().unwrap_or("").to_string();
        // Same limit as the interactive prompt: the line including its newline.
        let result = if bytes.len() as u64 > COMMAND_LIMIT {
            if bytes.last() != Some(&b'\n') {
                discard_line(input)?;
            }
            Err("COMMAND_LIMIT".to_string())
        } else if std::str::from_utf8(&bytes).is_err() {
            Err("UTF8_REQUIRED".to_string())
        } else {
            s.execute(&line)
        };
        match result {
            Ok(outcome) => {
                let (status, json) = record(seq, &command, &outcome);
                counts[STATUSES.iter().position(|s| *s == status).unwrap_or(0)] += 1;
                emit(json)?;
                if matches!(outcome, Outcome::Exit) {
                    not_run = count_remaining(input)?;
                    break;
                }
            }
            Err(error) => {
                counts[4] += 1;
                emit(
                    Object::record("result")
                        .value("seq", seq)
                        .text("command", &command)
                        .text("status", "ERROR")
                        .text("error", &error)
                        .finish(),
                )?;
                eprintln!(
                    "ERROR seq={seq} command={} {}",
                    safe(&command),
                    safe(&error)
                );
                not_run = count_remaining(input)?;
                break;
            }
        }
    }
    let exit = if counts[4] > 0 {
        2
    } else if counts[3] > 0 {
        3
    } else {
        0
    };
    let mut statuses = Object::bare();
    for (name, count) in STATUSES.iter().zip(counts) {
        statuses.value(name, count);
    }
    emit(
        Object::record("summary")
            .value("executed", seq)
            .value("not_run", not_run)
            .raw("statuses", &statuses.finish())
            .value("exit_code", exit)
            .finish(),
    )?;
    Ok(exit)
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("gel-evidence [--demo | --batch]\nRust/offline multi-document phrase reader. Plaintext, not an encrypted vault.\nUse owner-controlled directories. Type help. Input is never a shell command.");
        return Ok(());
    }
    if !args.is_empty() && args != ["--demo"] {
        return Err("gel-evidence --help".into());
    }
    let mut s = Session {
        bank: Collection::new(),
        hits: Vec::new(),
    };
    println!("GEL EVIDENCE LAB | RUST | OFFLINE | NO LLM\nSource lookup, not conversation. Timers exclude terminal rendering. No automatic saving.");
    if args == ["--demo"] {
        s.bank.add(
            "Memory notes",
            "RAM is volatile.\nA saved source snapshot can be reopened.",
        )?;
        s.bank
            .add("Polish notes", "Zażółć gęślą jaźń.\nŁódź is a city name.")?;
        for cmd in [
            "list",
            "find ram is volatile",
            "proof 1",
            "find zażółć gęślą",
            "proof 1",
            "find invented answer",
            "drop 1",
            "find ram is volatile",
        ] {
            println!("gel> {cmd}");
            s.command(cmd)?;
        }
        println!("GEL_EVIDENCE_DEMO=PASS");
        return Ok(());
    }
    let mut input = io::stdin().lock();
    loop {
        print!("gel> ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut line = String::new();
        let n = input
            .by_ref()
            .take(COMMAND_LIMIT + 1)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if line.len() > COMMAND_LIMIT as usize {
            return Err("COMMAND_LIMIT".into());
        }
        match s.command(&line) {
            Ok(false) => break,
            Ok(true) => {}
            Err(e) => println!("REFUSED {}", safe(&e)),
        }
    }
    println!("Closed. Only explicitly saved snapshots persist.");
    Ok(())
}
fn main() {
    if std::env::args().skip(1).eq(["--batch"]) {
        let mut s = Session {
            bank: Collection::new(),
            hits: Vec::new(),
        };
        let code =
            batch(&mut s, &mut io::stdin().lock(), &mut io::stdout().lock()).unwrap_or_else(|e| {
                eprintln!("GEL_EVIDENCE=FAIL {}", safe(&e));
                2
            });
        std::process::exit(code);
    }
    if let Err(e) = run() {
        eprintln!("GEL_EVIDENCE=FAIL {}", safe(&e));
        std::process::exit(1);
    }
}
