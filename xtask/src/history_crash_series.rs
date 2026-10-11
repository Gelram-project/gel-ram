//! Kill series for the public record history. A writer process appends 240
//! states to one record and saves the whole history after each with
//! `RecordHistory::write_atomic`, so one file is replaced while it grows; the
//! process is killed (SIGKILL on Unix) at a random moment. Afterwards the file
//! must open with `open_verified` and hold exactly the last acknowledged history
//! or the one after it, never a partial file, and a fresh process must continue
//! from it to the same bytes as a run without a kill. A control writer that
//! rewrites the file in place, in four flushed pieces, must leave partial files
//! under the same kind of kill, so the check is shown to see them. Killing a
//! process is not a power cut: disk caches are not tested.
use crate::crash_series::Rng;
use gel_core::{splitmix64, ORB_BITS, ORB_WORDS};
use gel_history::RecordHistory;
use gel_orb::Orb1024;
use std::{
    fmt::Write as _,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process::{Child, Command, Stdio},
    thread::JoinHandle,
    time::{Duration, Instant},
};

const STATES: usize = 240;
const STATE_SEED: u64 = 20_261_010;
/// Bits changed per step, in turn: residuals, the 92/93-bit boundary and literals.
const CHANGED_BITS: [usize; 8] = [1, 4, 16, 40, 92, 93, 7, 300];
const FILE: &str = "record.gelhis";
const MAX_PAYLOAD: u64 = 1 << 20;

/// The deterministic states 1..=STATES; each changes `CHANGED_BITS` bits of the one before.
fn states() -> Vec<Orb1024> {
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(STATE_SEED.wrapping_add(i as u64));
    }
    let mut state = Orb1024::from_words(words);
    (1..=STATES)
        .map(|n| {
            // 37 is coprime with 1024, so the k positions are distinct.
            for j in 0..CHANGED_BITS[n % CHANGED_BITS.len()] {
                let bit = (n * 101 + 37 * j) % ORB_BITS;
                state.words_mut()[bit >> 6] ^= 1u64 << (bit & 63);
            }
            state
        })
        .collect()
}

/// `expected[n]` is the GELHIS01 file of the first `n` states (`expected[0]` is never written).
fn expected() -> Result<Vec<Vec<u8>>, String> {
    let mut history = RecordHistory::new();
    let mut out = vec![Vec::new()];
    for state in states() {
        history.append(state).map_err(|e| e.to_string())?;
        out.push(history.to_bytes().map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// The control: the file truncated and rewritten in place, in four flushed pieces.
fn write_in_place(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    for piece in bytes.chunks(bytes.len().div_ceil(4)) {
        file.write_all(piece)
            .and_then(|()| file.sync_data())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// `history-writer PATH atomic|in-place`: the process the series kills. It
/// continues the history saved at PATH, or starts one, and prints
/// `HISTORY_ACK n=N` after each save returns.
pub fn writer(args: &[String]) -> Result<(), String> {
    let [path, mode] = args else {
        return Err("usage: history-writer PATH atomic|in-place".into());
    };
    let in_place = match mode.as_str() {
        "atomic" => false,
        "in-place" => true,
        _ => return Err(format!("unknown mode {mode}")),
    };
    let path = Path::new(path);
    let mut history = match fs::metadata(path) {
        Ok(_) => {
            let start = Instant::now();
            let history = RecordHistory::open_verified(path, STATES as u64, MAX_PAYLOAD)
                .map_err(|e| format!("HISTORY_OPEN=FAIL {e}"))?;
            println!(
                "HISTORY_OPEN states={} open_us={}",
                history.len(),
                start.elapsed().as_micros()
            );
            history
        }
        Err(e) if e.kind() == ErrorKind::NotFound => RecordHistory::new(),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let states = states();
    for n in history.len() + 1..=STATES {
        history.append(states[n - 1]).map_err(|e| e.to_string())?;
        if in_place {
            write_in_place(path, &history.to_bytes().map_err(|e| e.to_string())?)?;
        } else {
            history.write_atomic(path).map_err(|e| e.to_string())?;
        }
        println!("HISTORY_ACK n={n}");
    }
    println!("HISTORY_WRITER=DONE n={STATES}");
    Ok(())
}

/// A running writer; its stdout lines are collected until it exits.
struct Run {
    child: Child,
    reader: JoinHandle<Vec<String>>,
}
impl Run {
    fn start(exe: &Path, path: &Path, mode: &str) -> Result<Self, String> {
        let mut child = Command::new(exe)
            .arg("history-writer")
            .arg(path)
            .arg(mode)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("{}: {e}", exe.display()))?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        let reader = std::thread::spawn(move || {
            BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
                .collect()
        });
        Ok(Run { child, reader })
    }
    /// Kill after `after` (or wait for exit when `None`); returns the stdout
    /// lines and whether the writer exited successfully.
    fn finish(mut self, after: Option<Duration>) -> Result<(Vec<String>, bool), String> {
        if let Some(after) = after {
            std::thread::sleep(after);
            let _ = self.child.kill();
        }
        let status = self.child.wait().map_err(|e| e.to_string())?;
        let lines = self.reader.join().map_err(|_| "reader panicked")?;
        Ok((lines, status.success()))
    }
}

/// The acknowledged counts in order; they must run from `from` + 1 one by one.
fn acknowledged(lines: &[String], from: usize) -> Result<usize, String> {
    let mut last = from;
    for n in lines
        .iter()
        .filter_map(|l| l.strip_prefix("HISTORY_ACK n="))
    {
        if n.parse::<usize>().ok() != Some(last + 1) {
            return Err(format!("acknowledgement n={n} after n={last}"));
        }
        last += 1;
    }
    Ok(last)
}

/// What the history file holds after a run.
#[derive(Debug, PartialEq)]
enum Held {
    Absent,
    /// Opens with `open_verified` and equals `expected[n]` byte for byte.
    History(usize),
    /// Anything else.
    Partial,
}

fn held(path: &Path, expected: &[Vec<u8>]) -> Result<Held, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Held::Absent),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let opens = RecordHistory::open_verified(path, STATES as u64, MAX_PAYLOAD).is_ok();
    Ok(match (1..expected.len()).find(|&n| expected[n] == bytes) {
        Some(n) if opens => Held::History(n),
        _ => Held::Partial,
    })
}

#[derive(Debug, Default, PartialEq)]
struct Judged {
    acknowledged_lost: bool,
    unacknowledged_complete: bool,
    partial: bool,
}

/// The file may hold the last acknowledged history or the next one; nothing else.
fn judge(held: &Held, acked: usize) -> Judged {
    match *held {
        Held::Absent => Judged {
            acknowledged_lost: acked > 0,
            ..Judged::default()
        },
        Held::History(n) if n == acked => Judged::default(),
        Held::History(n) if n == acked + 1 => Judged {
            unacknowledged_complete: true,
            ..Judged::default()
        },
        Held::History(n) if n < acked => Judged {
            acknowledged_lost: true,
            ..Judged::default()
        },
        Held::History(_) | Held::Partial => Judged {
            acknowledged_lost: acked > 0,
            partial: true,
            ..Judged::default()
        },
    }
}

/// Outcome of one killed run.
struct Trial {
    kill_ms: u64,
    acknowledged: usize,
    held: Held,
    judged: Judged,
    temporaries: usize,
    resumed: Option<bool>,
    open_us: Option<u128>,
}

fn temporaries(dir: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let name = entry.map_err(|e| e.to_string())?.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&format!("{FILE}.tmp-")) {
            count += 1;
        } else if name != FILE {
            return Err(format!("unexpected file {name} in {}", dir.display()));
        }
    }
    Ok(count)
}

fn trial(
    exe: &Path,
    dir: &Path,
    mode: &str,
    expected: &[Vec<u8>],
    kill_ms: u64,
) -> Result<Trial, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(FILE);
    let (lines, _) = Run::start(exe, &path, mode)?.finish(Some(Duration::from_millis(kill_ms)))?;
    let acked = acknowledged(&lines, 0)?;
    let held = held(&path, expected)?;
    let judged = judge(&held, acked);
    let temporaries = temporaries(dir)?;
    let (mut resumed, mut open_us) = (None, None);
    if mode == "atomic" {
        // A fresh process continues from the file as it is, temporaries left in place.
        let (out, ok) = Run::start(exe, &path, "atomic")?.finish(None)?;
        let from = match held {
            Held::History(n) => n,
            _ => 0,
        };
        let done = out.last() == Some(&format!("HISTORY_WRITER=DONE n={STATES}"));
        resumed = Some(
            ok && done
                && acknowledged(&out, from) == Ok(STATES)
                && self::held(&path, expected)? == Held::History(STATES),
        );
        open_us = out
            .iter()
            .find_map(|l| l.strip_prefix("HISTORY_OPEN "))
            .and_then(|r| r.split("open_us=").nth(1))
            .and_then(|v| v.trim().parse().ok());
    }
    Ok(Trial {
        kill_ms,
        acknowledged: acked,
        held,
        judged,
        temporaries,
        resumed,
        open_us,
    })
}

/// An uninterrupted run must reach the last expected file; returns the kill window in ms.
fn reference(exe: &Path, dir: &Path, mode: &str, expected: &[Vec<u8>]) -> Result<u64, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(FILE);
    let start = Instant::now();
    let (lines, ok) = Run::start(exe, &path, mode)?.finish(None)?;
    let window_ms = (start.elapsed().as_millis() as u64 * 6 / 5).max(10);
    if !ok
        || acknowledged(&lines, 0) != Ok(STATES)
        || held(&path, expected)? != Held::History(STATES)
    {
        return Err(format!(
            "uninterrupted {mode} run did not reach the expected history"
        ));
    }
    Ok(window_ms)
}

fn percentile(sorted: &[u128], p: usize) -> String {
    if sorted.is_empty() {
        return "-".into();
    }
    let i = (sorted.len() * p).div_ceil(100).saturating_sub(1);
    format!("{:.1}", sorted[i] as f64 / 1000.0)
}

/// Runs `trials` killed runs of the atomic writer and `control_trials` of the
/// in-place control, with kill times drawn from `seed`; returns the report text.
pub fn run(trials: usize, seed: u64, control_trials: usize) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let scratch =
        std::env::temp_dir().join(format!("gel-history-kill-{}-{nonce}", std::process::id()));
    let expected = expected()?;
    let window_ms = reference(&exe, &scratch.join("reference"), "atomic", &expected)?;
    let control_window_ms = if control_trials > 0 {
        reference(
            &exe,
            &scratch.join("control-reference"),
            "in-place",
            &expected,
        )?
    } else {
        0
    };
    let mut report = format!(
        "# history kill series: {STATES} states of one record, the whole GELHIS01 file saved with \
         write_atomic after each ({} bytes at the end); kill after 1..={window_ms} ms (uninterrupted \
         run + 20%), seed {seed}; control: in-place rewrite in four flushed pieces, kill after \
         1..={control_window_ms} ms\n\
         # trial\tmode\tkill_ms\tacknowledged\theld\tacknowledged_lost\tunacknowledged_complete\tpartial\ttemporaries\tresumed\topen_us\n",
        expected[STATES].len()
    );
    let mut rng = Rng(seed);
    let (mut acked, mut lost, mut partial, mut resumed, mut complete, mut temps, mut after_last) =
        (0, 0, 0, 0, 0, 0, 0);
    let mut control_partial = 0;
    let mut open = Vec::new();
    for i in 1..=trials + control_trials {
        let (mode, window) = if i <= trials {
            ("atomic", window_ms)
        } else {
            ("in-place", control_window_ms)
        };
        let dir = scratch.join(format!("trial-{i:04}"));
        let t = trial(&exe, &dir, mode, &expected, rng.upto(window))?;
        let _ = fs::remove_dir_all(&dir);
        let _ = writeln!(
            report,
            "{i}\t{mode}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            t.kill_ms,
            t.acknowledged,
            match t.held {
                Held::Absent => "absent".to_owned(),
                Held::History(n) => n.to_string(),
                Held::Partial => "partial".to_owned(),
            },
            t.judged.acknowledged_lost,
            t.judged.unacknowledged_complete,
            t.judged.partial,
            t.temporaries,
            t.resumed.map_or("-".to_owned(), |r| r.to_string()),
            t.open_us.map_or("-".to_owned(), |u| u.to_string())
        );
        if mode == "in-place" {
            control_partial += usize::from(t.judged.partial);
            continue;
        }
        acked += t.acknowledged;
        lost += usize::from(t.judged.acknowledged_lost);
        partial += usize::from(t.judged.partial);
        resumed += usize::from(t.resumed == Some(true));
        complete += usize::from(t.judged.unacknowledged_complete);
        temps += usize::from(t.temporaries > 0);
        after_last += usize::from(t.acknowledged == STATES);
        open.extend(t.open_us);
    }
    let _ = fs::remove_dir_all(&scratch);
    open.sort_unstable();
    let pass = lost == 0
        && partial == 0
        && resumed == trials
        && (control_trials == 0 || control_partial > 0);
    let _ = writeln!(
        report,
        "# reopen before resuming, ms: median {} p95 {} max {}",
        percentile(&open, 50),
        percentile(&open, 95),
        percentile(&open, 100)
    );
    let _ = writeln!(
        report,
        "HISTORY_CRASH_SERIES={} trials={trials} acknowledged={acked} acknowledged_lost={lost} partial={partial} resumed={resumed} unacknowledged_complete={complete} temporaries_left={temps} after_last_ack={after_last} control_trials={control_trials} control_partial={control_partial}",
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

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gel-history-kill-test-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_walk_mixes_residuals_and_literals_and_grows_every_step() {
        let expected = expected().unwrap();
        assert_eq!(expected.len(), STATES + 1);
        assert_eq!(states(), states());
        for n in 1..STATES {
            assert!(expected[n + 1].len() > expected[n].len());
        }
        let mut history = RecordHistory::new();
        for state in states() {
            history.append(state).unwrap();
        }
        let literals = history
            .entries()
            .iter()
            .filter(|e| matches!(e, gel_history::HistoryEntry::Literal(_)))
            .count();
        assert!(literals > STATES / 4 && literals < STATES);
    }

    #[test]
    fn a_cut_or_changed_file_is_partial_and_an_older_one_is_lost() {
        let expected = expected().unwrap();
        let dir = scratch("judge");
        let path = dir.join(FILE);
        assert_eq!(held(&path, &expected).unwrap(), Held::Absent);
        assert_eq!(judge(&Held::Absent, 0), Judged::default());
        assert!(judge(&Held::Absent, 3).acknowledged_lost);

        fs::write(&path, &expected[5]).unwrap();
        assert_eq!(held(&path, &expected).unwrap(), Held::History(5));
        assert_eq!(judge(&Held::History(5), 5), Judged::default());
        assert!(judge(&Held::History(5), 4).unacknowledged_complete);
        assert!(judge(&Held::History(5), 6).acknowledged_lost);
        assert!(!judge(&Held::History(5), 6).partial);
        assert!(judge(&Held::History(7), 5).partial);

        fs::write(&path, &expected[5][..expected[5].len() - 1]).unwrap();
        assert_eq!(held(&path, &expected).unwrap(), Held::Partial);
        let mut changed = expected[5].clone();
        changed[100] ^= 1;
        fs::write(&path, &changed).unwrap();
        assert_eq!(held(&path, &expected).unwrap(), Held::Partial);
        let judged = judge(&Held::Partial, 5);
        assert!(judged.partial && judged.acknowledged_lost);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn acknowledgements_must_follow_one_by_one() {
        let lines: Vec<String> = [
            "HISTORY_OPEN states=3 open_us=9",
            "HISTORY_ACK n=4",
            "noise",
            "HISTORY_ACK n=5",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(acknowledged(&lines, 3), Ok(5));
        assert!(acknowledged(&lines, 0).is_err());
        assert_eq!(acknowledged(&[], 7), Ok(7));
    }

    #[test]
    fn the_in_place_control_writes_the_same_bytes_and_temporaries_are_counted() {
        let expected = expected().unwrap();
        let dir = scratch("control");
        let path = dir.join(FILE);
        write_in_place(&path, &expected[9]).unwrap();
        write_in_place(&path, &expected[3]).unwrap();
        assert_eq!(fs::read(&path).unwrap(), expected[3]);
        fs::write(dir.join(format!("{FILE}.tmp-1-0")), b"x").unwrap();
        assert_eq!(temporaries(&dir), Ok(1));
        fs::write(dir.join("other"), b"x").unwrap();
        assert!(temporaries(&dir).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
