//! Scripted typing, unmodified live subprocess output; not a benchmark harness.
//! Run in a NEW directory containing memory.txt and unicode.txt.
//! Display, log, lock, thread, parse and snapshot failures end in
//! RECORDING_FAILED (exit 1); every refusal before any output is written (wrong
//! command line, relative binary, existing output) ends in RECORDING_REFUSED
//! (exit 2). `xtask recorder-lint` enforces the lints below with clippy.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::print_stderr
)]
mod record_support;
use record_support::{new_file, pump, Capture};
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{self, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
/// A broken stderr cannot turn a reported failure into a panic.
fn report(prefix: &str, message: &str) {
    let _ = writeln!(io::stderr(), "{prefix}: {message}");
}
/// Failure while no application subprocess is running.
fn abort(message: &str) -> ! {
    report("RECORDING_FAILED", message);
    std::process::exit(1);
}
fn show(text: &str) -> io::Result<()> {
    let mut out = io::stdout();
    out.write_all(text.as_bytes())?;
    out.flush()
}
fn show_or_abort(text: &str) {
    if let Err(e) = show(text) {
        abort(&format!("display I/O: {e}"));
    }
}
fn type_text(s: &str) -> io::Result<()> {
    let mut out = io::stdout();
    for c in s.chars() {
        write!(out, "{c}")?;
        out.flush()?;
        pause(28);
    }
    Ok(())
}
fn new_output(path: &str) -> fs::File {
    new_file(Path::new(path)).unwrap_or_else(|e| abort(&format!("new output {path}: {e}")))
}
fn emitted_pin(segment: Option<&str>) -> Result<String, String> {
    let pin = segment.unwrap_or("").split(';').next().unwrap_or("");
    if pin.len() == 64 && pin.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(pin.to_string())
    } else {
        Err("invalid emitted pin".into())
    }
}
struct App {
    child: Child,
    input: Option<ChildStdin>,
    output: Arc<Mutex<Capture>>,
    errors: Arc<Mutex<Capture>>,
    error_reader: Option<thread::JoinHandle<()>>,
    status_log: fs::File,
    reader: Option<thread::JoinHandle<()>>,
}
impl Drop for App {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
/// Failure after the status log exists but before an App owns a subprocess.
fn fail_unlaunched(status_log: &mut fs::File, message: &str) -> ! {
    let _ = writeln!(status_log, "FAILED {message}");
    let _ = status_log.sync_all();
    abort(message);
}
impl App {
    fn launch(binary: &OsStr, number: usize) -> Self {
        let stdout_log = new_output(&format!("process-{number}.txt"));
        let stderr_log = new_output(&format!("process-{number}-stderr.txt"));
        let mut status_log = new_output(&format!("process-{number}-status.txt"));
        let typed = write!(io::stdout(), "$ ").and_then(|()| type_text("gel-evidence"));
        if let Err(e) = typed {
            fail_unlaunched(&mut status_log, &format!("display I/O: {e}"));
        }
        pause(1500);
        if let Err(e) = show("\n") {
            fail_unlaunched(&mut status_log, &format!("display I/O: {e}"));
        }
        let mut child = match Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => fail_unlaunched(&mut status_log, &format!("spawn: {error}")),
        };
        let (stdout, stderr) = match (child.stdout.take(), child.stderr.take()) {
            (Some(stdout), Some(stderr)) => (stdout, stderr),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                fail_unlaunched(&mut status_log, "subprocess pipes unavailable");
            }
        };
        let input = child.stdin.take();
        let output = Arc::new(Mutex::new(Capture::default()));
        let shared = output.clone();
        // A reader thread that cannot start is a controlled failure, not a panic.
        let reader = match thread::Builder::new()
            .spawn(move || pump(stdout, stdout_log, io::stdout(), shared))
        {
            Ok(reader) => reader,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                fail_unlaunched(&mut status_log, &format!("reader thread: {e}"));
            }
        };
        let errors = Arc::new(Mutex::new(Capture::default()));
        let shared_errors = errors.clone();
        let error_reader = match thread::Builder::new()
            .spawn(move || pump(stderr, stderr_log, io::stderr(), shared_errors))
        {
            Ok(reader) => reader,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                fail_unlaunched(&mut status_log, &format!("reader thread: {e}"));
            }
        };
        let mut app = Self {
            child,
            input,
            output,
            reader: Some(reader),
            errors,
            error_reader: Some(error_reader),
            status_log,
        };
        app.wait_for(0, "gel> ");
        pause(1800);
        app
    }
    fn text(&mut self) -> String {
        let shared = self.output.clone();
        let text = shared
            .lock()
            .map(|state| String::from_utf8_lossy(&state.bytes).into_owned())
            .map_err(|_| ());
        match text {
            Ok(text) => text,
            Err(()) => self.fail("capture poisoned"),
        }
    }
    fn captured_len(&mut self) -> usize {
        let shared = self.output.clone();
        let len = shared.lock().map(|state| state.bytes.len()).map_err(|_| ());
        match len {
            Ok(len) => len,
            Err(()) => self.fail("capture poisoned"),
        }
    }
    fn wait_for(&mut self, from: usize, needle: &str) {
        let start = Instant::now();
        let output = self.output.clone();
        let errors = self.errors.clone();
        loop {
            let seen = output
                .lock()
                .map(|state| {
                    (
                        state
                            .marker(from, needle.as_bytes())
                            .map_err(|e| e.to_string()),
                        state.done,
                    )
                })
                .map_err(|_| ());
            let (found, done) = match seen {
                Ok(seen) => seen,
                Err(()) => self.fail("capture poisoned"),
            };
            match found {
                Ok(true) => return,
                Err(e) => self.fail(&e),
                Ok(false) => {}
            }
            let error = match errors.lock() {
                Ok(state) => state.error.clone(),
                Err(_) => Some("capture poisoned".into()),
            };
            if let Some(e) = error {
                self.fail(&e);
            }
            if done || start.elapsed() >= Duration::from_secs(10) {
                self.fail("EOF or timeout before marker");
            }
            pause(10);
        }
    }
    fn fail(&mut self, message: &str) -> ! {
        let _ = self.child.kill();
        let status = self.child.wait();
        let _ = writeln!(self.status_log, "FAILED {message}; status={status:?}");
        let _ = self.status_log.sync_all();
        abort(message);
    }
    fn command(&mut self, cmd: &str, marker: &str, hold: u64) {
        let from = self.captured_len();
        if let Err(e) = type_text(cmd) {
            self.fail(&format!("display I/O: {e}"));
        }
        if cmd == "exit" {
            pause(1800);
        } else {
            pause(300);
        }
        if let Err(e) = show("\n") {
            self.fail(&format!("display I/O: {e}"));
        }
        let sent = (|| -> io::Result<()> {
            let stdin = self
                .input
                .as_mut()
                .ok_or_else(|| io::Error::other("subprocess stdin unavailable"))?;
            writeln!(stdin, "{cmd}")?;
            stdin.flush()
        })();
        if let Err(error) = sent {
            self.fail(&format!("command I/O: {error}"));
        }
        self.wait_for(from, marker);
        pause(hold);
        let logged = (|| -> io::Result<()> {
            let mut events = fs::OpenOptions::new().append(true).open("commands.txt")?;
            writeln!(events, "{cmd}")?;
            events.sync_all()
        })();
        if let Err(error) = logged {
            self.fail(&format!("command log I/O: {error}"));
        }
    }
    fn finish(mut self) {
        self.command("exit", "Closed.", 1000);
        self.input.take();
        let start = Instant::now();
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    let logged = writeln!(self.status_log, "{status}")
                        .and_then(|()| self.status_log.sync_all());
                    if let Err(e) = logged {
                        self.fail(&format!("status log I/O: {e}"));
                    }
                    if !status.success() {
                        self.fail("nonzero subprocess exit");
                    }
                    break;
                }
                Ok(None) => {}
                Err(e) => self.fail(&format!("subprocess wait: {e}")),
            }
            if start.elapsed() >= Duration::from_secs(5) {
                self.fail("subprocess exit timeout");
            }
            pause(10);
        }
        for reader in [self.reader.take(), self.error_reader.take()] {
            match reader.map(thread::JoinHandle::join) {
                Some(Ok(())) => {}
                _ => self.fail("stream reader did not finish cleanly"),
            }
        }
        for shared in [self.output.clone(), self.errors.clone()] {
            let clean = match shared.lock() {
                Ok(state) => state.error.is_none(),
                Err(_) => false,
            };
            if !clean {
                self.fail("stream capture failed");
            }
        }
    }
}
fn latest_pin(app: &mut App) -> String {
    let output = app.text();
    match emitted_pin(output.rsplit("BUNDLE_SHA256=").next()) {
        Ok(pin) => pin,
        Err(e) => app.fail(&e),
    }
}
fn read_snapshot(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| abort(&format!("read {path}: {e}")))
}
fn write_complete(marker: &[u8]) {
    let mut complete = new_output("COMPLETE.txt");
    if let Err(e) = complete
        .write_all(marker)
        .and_then(|()| complete.sync_all())
    {
        // Never leave a partial completion marker behind.
        let _ = fs::remove_file("COMPLETE.txt");
        abort(&format!("completion I/O: {e}"));
    }
}

fn update_walkthrough(binary: &OsStr) {
    show_or_abort("UPDATE / STALE CITATION / RESTART / CORRUPTED COPY\nSynthetic document; phrase lookup, not semantic reasoning.\n\n");
    let mut first = App::launch(binary, 1);
    first.command("add original.txt", "ADDED id=1", 1500);
    first.command("find open the valve", "FIND=HIT", 4000);
    first.command("proof 1", "CITATION=PASS", 2000);
    first.command("find invented instruction", "FIND=UNKNOWN", 2000);
    first.command("save original.gelset", "BUNDLE_SHA256=", 2000);
    let original_pin = latest_pin(&mut first);
    first.command("find open the valve", "FIND=HIT", 2000);
    first.command("proof 1", "CITATION=PASS", 1500);
    first.command("replace 1 revised.txt", "REPLACED id=1", 2000);
    first.command("proof 1", "REFUSED NO_CURRENT_RESULT", 3000);
    first.command("find open the valve", "FIND=UNKNOWN", 2000);
    first.command("find keep the valve closed", "FIND=HIT", 4000);
    first.command("proof 1", "CITATION=PASS", 2000);
    first.command("save revised.gelset", "BUNDLE_SHA256=", 2000);
    let revised_pin = latest_pin(&mut first);
    if original_pin == revised_pin {
        first.fail("revised snapshot pin equals original pin");
    }
    first.finish();
    let original = read_snapshot("original.gelset");
    let revised = read_snapshot("revised.gelset");
    let mut changed = original.clone();
    let Some(last) = changed.last_mut() else {
        abort("original.gelset is empty");
    };
    *last ^= 1;
    let mut bad = new_output("corrupt.gelset");
    if let Err(e) = bad.write_all(&changed).and_then(|()| bad.sync_all()) {
        abort(&format!("corrupt test copy I/O: {e}"));
    }
    show_or_abort("\n--- First process ended. TEST FIXTURE: flipped one byte in a COPY. ---\nOriginal and revised snapshots retained unchanged. Starting new process.\n\n");
    pause(2500);
    let mut second = App::launch(binary, 2);
    second.command(
        &format!("load {revised_pin} revised.gelset"),
        "REOPEN=PASS",
        2000,
    );
    second.command("find keep the valve closed", "FIND=HIT", 4000);
    second.command("proof 1", "CITATION=PASS", 2000);
    second.command(
        &format!("load {original_pin} original.gelset"),
        "REOPEN=PASS",
        2000,
    );
    second.command("find open the valve", "FIND=HIT", 4000);
    second.command("proof 1", "CITATION=PASS", 2000);
    second.command(
        &format!("load {original_pin} corrupt.gelset"),
        "REFUSED",
        3000,
    );
    second.command("find open the valve", "FIND=HIT", 4000);
    second.command("proof 1", "CITATION=PASS", 2000);
    second.finish();
    for (path, expected) in [("original.gelset", &original), ("revised.gelset", &revised)] {
        if read_snapshot(path) != *expected {
            abort(&format!("{path} changed during walkthrough"));
        }
    }
    show_or_abort("\nWalkthrough complete: old/revised snapshots reopened; corrupt copy refused.\nTimes are individual operation measurements, not performance percentiles.\n\n");
}

fn main() {
    // args_os: the binary may be any absolute OS path, including non-UTF-8;
    // a relative binary, a missing binary or an unknown flag is refused.
    let args: Vec<OsString> = std::env::args_os().collect();
    let (binary, update) = match (args.get(1), args.get(2), args.len()) {
        (Some(binary), None, 2) if Path::new(binary).is_absolute() => (binary.clone(), false),
        (Some(binary), Some(flag), 3) if flag == "--update" && Path::new(binary).is_absolute() => {
            (binary.clone(), true)
        }
        _ => {
            report(
                "RECORDING_REFUSED",
                "usage: record_evidence ABSOLUTE_APP_BINARY [--update]",
            );
            std::process::exit(2);
        }
    };
    for path in [
        "checkpoint.gelset",
        "COMPLETE.txt",
        "commands.txt",
        "process-1.txt",
        "process-2.txt",
        "process-1-stderr.txt",
        "process-2-stderr.txt",
        "process-1-status.txt",
        "process-2-status.txt",
        "original.gelset",
        "revised.gelset",
        "corrupt.gelset",
    ] {
        match fs::symlink_metadata(path) {
            Ok(_) => {
                report("RECORDING_REFUSED", &format!("existing output {path}"));
                std::process::exit(2);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                report("RECORDING_REFUSED", &e.to_string());
                std::process::exit(2);
            }
        }
    }
    let _commands = new_output("commands.txt");
    show_or_abort("GEL EVIDENCE LAB  |  Linux terminal  |  OFFLINE / CPU\nScripted typing; real application output. No LLM.\nPhrase retrieval, NOT semantic conversation.\n\n");
    pause(2000);
    if update {
        update_walkthrough(&binary);
        pause(2000);
        write_complete(b"LIVE_UPDATE_WALKTHROUGH=PASS\n");
        return;
    }
    let mut first = App::launch(&binary, 1);
    first.command("add memory.txt", "ADDED id=1", 1200);
    first.command("add unicode.txt", "ADDED id=2", 1200);
    first.command("find ram is volatile", "FIND=HIT", 8000);
    first.command("proof 1", "CITATION=PASS", 3500);
    first.command("find imaginary evidence", "FIND=UNKNOWN", 5000);
    first.command("save checkpoint.gelset", "BUNDLE_SHA256=", 3500);
    let output = first.text();
    let pin = match emitted_pin(output.split("BUNDLE_SHA256=").nth(1)) {
        Ok(pin) => pin,
        Err(e) => first.fail(&e),
    };
    first.finish();
    show_or_abort(
        "\n--- Process ended. Starting a new process; loading the saved snapshot. ---\n\n",
    );
    pause(2000);
    let mut second = App::launch(&binary, 2);
    second.command(
        &format!("load {pin} checkpoint.gelset"),
        "REOPEN=PASS",
        4500,
    );
    second.command("find ram is volatile", "FIND=HIT", 8000);
    second.command("proof 1", "CITATION=PASS", 4500);
    second.finish();
    show_or_abort("\n$  Walkthrough complete. Snapshot reopened; citation hash correspondence checked.\nNothing was published. Times above are application measurements, not typing delays.\n");
    pause(3000);
    write_complete(b"LIVE_SUBPROCESS_WALKTHROUGH=PASS\n");
}
