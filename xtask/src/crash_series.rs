//! Crash series for the public collection tool. `gel-evidence` adds documents
//! one at a time and saves a new snapshot after each, so the collection grows
//! while it is written; the process is killed (SIGKILL on Unix) at a random
//! moment. Afterwards every acknowledged snapshot must reload with its pin and
//! hold exactly the expected documents, an unacknowledged snapshot may be absent
//! or complete but never partial, and a fresh process must continue from the
//! last acknowledged snapshot to the same final collection as a run without a
//! kill. Killing a process is not a power cut: disk caches are not tested.
use gel_source::{collection::Collection, digest, hex, Hash};
use std::{
    fmt::Write as _,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

const DOCUMENTS: usize = 24;
const DOCUMENT_LINES: usize = 1_200;

/// Deterministic document `i` (1-based): distinct lines, about 64 KiB.
fn document(i: usize) -> (String, String) {
    let mut text = String::new();
    for line in 0..DOCUMENT_LINES {
        let _ = writeln!(
            text,
            "crash series document {i} line {line} carries source text {}",
            i * 7919 + line
        );
    }
    (format!("doc-{i:03}.txt"), text)
}

/// splitmix64, so a seed always gives the same kill schedule.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// Uniform in 1..=max.
    fn upto(&mut self, max: u64) -> u64 {
        1 + self.next() % max.max(1)
    }
}

fn parse_pin(raw: &str) -> Result<Hash, String> {
    if raw.len() != 64 || !raw.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid pin {raw}"));
    }
    let mut out = [0; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&raw[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn snapshot(dir: &Path, k: usize) -> PathBuf {
    dir.join(format!("snap-{k:03}"))
}

/// A running `gel-evidence` fed a whole script; stdout lines are collected.
struct Session {
    child: Child,
    lines: mpsc::Receiver<String>,
    reader: JoinHandle<()>,
}
impl Session {
    fn start(binary: &Path, script: &str) -> Result<Self, String> {
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("{}: {e}", binary.display()))?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        child
            .stdin
            .take()
            .ok_or("no stdin")?
            .write_all(script.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(Session {
            child,
            lines,
            reader,
        })
    }
    /// Kill after `after` (or wait for exit when `None`); returns all stdout lines.
    fn finish(mut self, after: Option<Duration>) -> Result<Vec<String>, String> {
        if let Some(after) = after {
            std::thread::sleep(after);
            let _ = self.child.kill();
        }
        self.child.wait().map_err(|e| e.to_string())?;
        self.reader.join().map_err(|_| "reader panicked")?;
        Ok(self.lines.try_iter().collect())
    }
}

fn acknowledged(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|l| l.strip_prefix("BUNDLE_SHA256="))
        .map(|rest| rest.split(';').next().unwrap_or("").to_owned())
        .collect()
}

fn write_script(docs: &Path, dir: &Path, from: usize) -> String {
    let mut s = String::new();
    for k in from..=DOCUMENTS {
        let _ = writeln!(s, "add {}", docs.join(document(k).0).display());
        let _ = writeln!(s, "save {}", snapshot(dir, k).display());
    }
    s.push_str("exit\n");
    s
}

/// Outcome of one killed run: counts and the resume check.
#[derive(Default)]
struct Trial {
    kill_ms: u64,
    acknowledged: usize,
    acknowledged_lost: usize,
    unacknowledged_complete: usize,
    partial: usize,
    temporaries: usize,
    resumed: bool,
    reopen_us: Option<u128>,
}

fn trial(
    binary: &Path,
    docs: &Path,
    dir: &Path,
    expected: &[Vec<u8>],
    kill_ms: u64,
) -> Result<Trial, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let lines = Session::start(binary, &write_script(docs, dir, 1))?
        .finish(Some(Duration::from_millis(kill_ms)))?;
    let acks = acknowledged(&lines);
    let mut t = Trial {
        kill_ms,
        acknowledged: acks.len(),
        ..Trial::default()
    };
    for (i, pin) in acks.iter().enumerate() {
        let path = snapshot(dir, i + 1);
        let ok = fs::read(&path).is_ok_and(|b| hex(&digest(&b)) == *pin && b == expected[i + 1])
            && parse_pin(pin)
                .and_then(|p| Collection::load(&path, p))
                .is_ok();
        if !ok {
            t.acknowledged_lost += 1;
        }
    }
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    names.sort();
    for name in &names {
        match name
            .strip_prefix("snap-")
            .and_then(|n| n.parse::<usize>().ok())
        {
            Some(k) if (1..=DOCUMENTS).contains(&k) && k > acks.len() => {
                let path = snapshot(dir, k);
                let whole = fs::read(&path).is_ok_and(|b| b == expected[k])
                    && Collection::load(&path, digest(&expected[k])).is_ok();
                if whole {
                    t.unacknowledged_complete += 1
                } else {
                    t.partial += 1
                }
            }
            Some(_) => {}
            None => t.temporaries += 1,
        }
    }
    // Resume in a fresh process from the last acknowledged snapshot.
    let last = acks.len();
    let mut script = String::new();
    if last > 0 {
        let _ = writeln!(
            script,
            "load {} {}",
            acks[last - 1],
            snapshot(dir, last).display()
        );
    }
    let resume = dir.join("resume");
    fs::create_dir_all(&resume).map_err(|e| e.to_string())?;
    script.push_str(&write_script(docs, &resume, last + 1));
    let out = Session::start(binary, &script)?.finish(None)?;
    let reopened = last == 0 || out.iter().any(|l| l.contains("REOPEN=PASS"));
    let reached = last == DOCUMENTS
        || acknowledged(&out)
            .last()
            .is_some_and(|p| *p == hex(&digest(&expected[DOCUMENTS])));
    t.resumed = reopened && reached;
    t.reopen_us = out
        .iter()
        .find_map(|l| {
            l.split_once("REOPEN=PASS")
                .map(|(_, r)| r)
                .and_then(|r| r.split("load_ns=").nth(1))
        })
        .and_then(|v| v.trim().parse::<u128>().ok())
        .map(|ns| ns / 1_000);
    Ok(t)
}

/// Runs `trials` killed runs with kill times drawn from `seed`; returns the report text.
pub fn run(binary: &Path, trials: usize, seed: u64) -> Result<String, String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let scratch =
        std::env::temp_dir().join(format!("gel-crash-series-{}-{nonce}", std::process::id()));
    let docs = scratch.join("docs");
    fs::create_dir_all(&docs).map_err(|e| e.to_string())?;
    let mut expected = vec![Vec::new()];
    let mut c = Collection::new();
    for k in 1..=DOCUMENTS {
        let (title, text) = document(k);
        fs::write(docs.join(&title), &text).map_err(|e| e.to_string())?;
        c.add(&title, &text)?;
        expected.push(c.to_bytes());
    }
    // An uninterrupted run sets the kill window and must reach the same final pin.
    let reference = scratch.join("reference");
    fs::create_dir_all(&reference).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let lines = Session::start(binary, &write_script(&docs, &reference, 1))?.finish(None)?;
    let window_ms = (start.elapsed().as_millis() as u64 * 6 / 5).max(10);
    let acks = acknowledged(&lines);
    if acks.len() != DOCUMENTS || acks[DOCUMENTS - 1] != hex(&digest(&expected[DOCUMENTS])) {
        return Err("uninterrupted reference run did not reach the expected collection".into());
    }
    let mut report = format!(
        "# gel-evidence crash series: {DOCUMENTS} documents of about 64 KiB, a new snapshot after each; \
         kill after 1..={window_ms} ms (uninterrupted run + 20%), seed {seed}\n\
         # trial\tkill_ms\tacknowledged\tacknowledged_lost\tunacknowledged_complete\tpartial\ttemporaries\tresumed\treopen_us\n"
    );
    let mut rng = Rng(seed);
    let (mut acked, mut lost, mut partial, mut resumed, mut mid) = (0, 0, 0, 0, 0);
    for i in 1..=trials {
        let t = trial(
            binary,
            &docs,
            &scratch.join(format!("trial-{i:04}")),
            &expected,
            rng.upto(window_ms),
        )?;
        let _ = writeln!(
            report,
            "{i}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            t.kill_ms,
            t.acknowledged,
            t.acknowledged_lost,
            t.unacknowledged_complete,
            t.partial,
            t.temporaries,
            t.resumed,
            t.reopen_us.map_or("-".to_owned(), |u| u.to_string())
        );
        acked += t.acknowledged;
        lost += t.acknowledged_lost;
        partial += t.partial;
        resumed += usize::from(t.resumed);
        mid += usize::from(t.temporaries > 0 || t.unacknowledged_complete > 0);
        let _ = fs::remove_dir_all(scratch.join(format!("trial-{i:04}")));
    }
    let _ = fs::remove_dir_all(&scratch);
    let pass = lost == 0 && partial == 0 && resumed == trials;
    let _ = writeln!(
        report,
        "CRASH_SERIES={} trials={trials} acknowledged={acked} acknowledged_lost={lost} partial={partial} resumed={resumed} killed_mid_publication={mid}",
        if pass { "PASS" } else { "FAIL" }
    );
    if pass {
        Ok(report)
    } else {
        Err(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_and_schedule_are_deterministic() {
        assert_eq!(document(3), document(3));
        assert_ne!(document(3).1, document(4).1);
        assert!(document(1).1.len() > 60_000);
        let (mut a, mut b) = (Rng(7), Rng(7));
        for _ in 0..100 {
            let x = a.upto(50);
            assert_eq!(x, b.upto(50));
            assert!((1..=50).contains(&x));
        }
    }

    #[test]
    fn acknowledgements_are_read_in_order() {
        let lines = vec![
            "SAVED revision=1 save_ns=5".to_owned(),
            format!("BUNDLE_SHA256={}; retain independently", "a".repeat(64)),
            "noise".to_owned(),
            format!("BUNDLE_SHA256={}; retain independently", "b".repeat(64)),
        ];
        assert_eq!(acknowledged(&lines), vec!["a".repeat(64), "b".repeat(64)]);
        assert!(parse_pin(&"a".repeat(63)).is_err());
    }
}
