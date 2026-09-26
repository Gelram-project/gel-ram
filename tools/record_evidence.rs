//! Scripted typing, unmodified live subprocess output; not a benchmark harness.
//! Run in a NEW directory containing memory.txt and unicode.txt.
mod record_support;
use record_support::{new_file, pump, Capture};
use std::{
    fs,
    io::Write,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
fn type_text(s: &str) {
    for c in s.chars() {
        print!("{c}");
        std::io::stdout().flush().unwrap();
        pause(28);
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
impl App {
    fn launch(binary: &str, number: usize) -> Self {
        let stdout_log = new_file(std::path::Path::new(&format!("process-{number}.txt")))
            .expect("new stdout log");
        let stderr_log = new_file(std::path::Path::new(&format!(
            "process-{number}-stderr.txt"
        )))
        .expect("new stderr log");
        let status_log = new_file(std::path::Path::new(&format!(
            "process-{number}-status.txt"
        )))
        .expect("new status log");
        print!("$ ");
        type_text("gel-evidence");
        pause(1500);
        println!();
        let mut child = match Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                let mut status_log = status_log;
                let _ = writeln!(status_log, "FAILED spawn: {error}");
                let _ = status_log.sync_all();
                eprintln!("RECORDING_FAILED: spawn: {error}");
                std::process::exit(1);
            }
        };
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let input = child.stdin.take();
        let output = Arc::new(Mutex::new(Capture::default()));
        let shared = output.clone();
        let reader = thread::spawn(move || pump(stdout, stdout_log, std::io::stdout(), shared));
        let errors = Arc::new(Mutex::new(Capture::default()));
        let shared_errors = errors.clone();
        let error_reader =
            thread::spawn(move || pump(stderr, stderr_log, std::io::stderr(), shared_errors));
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
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap().bytes).into_owned()
    }
    fn wait_for(&mut self, from: usize, needle: &str) {
        let start = Instant::now();
        loop {
            let state = self.output.lock().unwrap();
            match state.marker(from, needle.as_bytes()) {
                Ok(true) => return,
                Err(e) => {
                    drop(state);
                    self.fail(&e.to_string());
                }
                Ok(false) => {}
            }
            let done = state.done;
            drop(state);
            let error = self.errors.lock().unwrap().error.clone();
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
        eprintln!("RECORDING_FAILED: {message}");
        std::process::exit(1);
    }
    fn command(&mut self, cmd: &str, marker: &str, hold: u64) {
        let from = self.output.lock().unwrap().bytes.len();
        type_text(cmd);
        if cmd == "exit" {
            pause(1800);
        } else {
            pause(300);
        }
        println!();
        std::io::stdout().flush().unwrap();
        let stdin = self.input.as_mut().unwrap();
        writeln!(stdin, "{cmd}").unwrap();
        stdin.flush().unwrap();
        self.wait_for(from, marker);
        pause(hold);
        let mut events = fs::OpenOptions::new()
            .append(true)
            .open("commands.txt")
            .unwrap();
        writeln!(events, "{cmd}").unwrap();
    }
    fn finish(mut self) {
        self.command("exit", "Closed.", 1000);
        self.input.take();
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                writeln!(self.status_log, "{status}").unwrap();
                self.status_log.sync_all().unwrap();
                if !status.success() {
                    self.fail("nonzero subprocess exit");
                }
                break;
            }
            if start.elapsed() >= Duration::from_secs(5) {
                self.fail("subprocess exit timeout");
            }
            pause(10);
        }
        self.reader.take().unwrap().join().unwrap();
        self.error_reader.take().unwrap().join().unwrap();
        let failed = self.output.lock().unwrap().error.is_some()
            || self.errors.lock().unwrap().error.is_some();
        if failed {
            self.fail("stream capture failed");
        }
    }
}
fn latest_pin(app: &App) -> String {
    let output = app.text();
    let pin = output
        .rsplit("BUNDLE_SHA256=")
        .next()
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .to_string();
    assert!(
        pin.len() == 64 && pin.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid emitted pin"
    );
    pin
}

fn update_walkthrough(binary: &str) {
    println!("UPDATE / STALE CITATION / RESTART / CORRUPTED COPY\nSynthetic document; phrase lookup, not semantic reasoning.\n");
    let mut first = App::launch(binary, 1);
    first.command("add original.txt", "ADDED id=1", 1500);
    first.command("find open the valve", "FIND=HIT", 4000);
    first.command("proof 1", "CITATION=PASS", 2000);
    first.command("find invented instruction", "FIND=UNKNOWN", 2000);
    first.command("save original.gelset", "BUNDLE_SHA256=", 2000);
    let original_pin = latest_pin(&first);
    first.command("find open the valve", "FIND=HIT", 2000);
    first.command("proof 1", "CITATION=PASS", 1500);
    first.command("replace 1 revised.txt", "REPLACED id=1", 2000);
    first.command("proof 1", "REFUSED NO_CURRENT_RESULT", 3000);
    first.command("find open the valve", "FIND=UNKNOWN", 2000);
    first.command("find keep the valve closed", "FIND=HIT", 4000);
    first.command("proof 1", "CITATION=PASS", 2000);
    first.command("save revised.gelset", "BUNDLE_SHA256=", 2000);
    let revised_pin = latest_pin(&first);
    assert_ne!(original_pin, revised_pin);
    first.finish();
    let original = fs::read("original.gelset").expect("saved original snapshot");
    let revised = fs::read("revised.gelset").expect("saved revised snapshot");
    let mut changed = original.clone();
    *changed.last_mut().expect("nonempty snapshot") ^= 1;
    let mut bad = new_file(std::path::Path::new("corrupt.gelset")).expect("new corrupt test copy");
    bad.write_all(&changed).unwrap();
    bad.sync_all().unwrap();
    println!("\n--- First process ended. TEST FIXTURE: flipped one byte in a COPY. ---\nOriginal and revised snapshots retained unchanged. Starting new process.\n");
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
    assert_eq!(fs::read("original.gelset").unwrap(), original);
    assert_eq!(fs::read("revised.gelset").unwrap(), revised);
    println!("\nWalkthrough complete: old/revised snapshots reopened; corrupt copy refused.\nTimes are individual operation measurements, not performance percentiles.\n");
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 2 || (args.len() == 3 && args[2] == "--update"),
        "record_evidence ABSOLUTE_APP_BINARY [--update]"
    );
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
                eprintln!("RECORDING_REFUSED: existing output {path}");
                std::process::exit(1);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                eprintln!("RECORDING_REFUSED: {e}");
                std::process::exit(1);
            }
        }
    }
    assert!(!std::path::Path::new("checkpoint.gelset").exists());
    assert!(!std::path::Path::new("COMPLETE.txt").exists());
    let _commands = new_file(std::path::Path::new("commands.txt")).expect("new commands log");
    println!("GEL EVIDENCE LAB  |  Linux terminal  |  OFFLINE / CPU\nScripted typing; real application output. No LLM.\nPhrase retrieval, NOT semantic conversation.\n");
    pause(2000);
    if args.len() == 3 {
        update_walkthrough(&args[1]);
        pause(2000);
        let mut complete = new_file(std::path::Path::new("COMPLETE.txt")).expect("new completion");
        complete
            .write_all(b"LIVE_UPDATE_WALKTHROUGH=PASS\n")
            .unwrap();
        complete.sync_all().unwrap();
        return;
    }
    let mut first = App::launch(&args[1], 1);
    first.command("add memory.txt", "ADDED id=1", 1200);
    first.command("add unicode.txt", "ADDED id=2", 1200);
    first.command("find ram is volatile", "FIND=HIT", 8000);
    first.command("proof 1", "CITATION=PASS", 3500);
    first.command("find imaginary evidence", "FIND=UNKNOWN", 5000);
    first.command("save checkpoint.gelset", "BUNDLE_SHA256=", 3500);
    let output = first.text();
    let pin = output
        .split("BUNDLE_SHA256=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    assert_eq!(pin.len(), 64);
    assert!(pin.bytes().all(|b| b.is_ascii_hexdigit()));
    first.finish();
    println!("\n--- Process ended. Starting a new process; loading the saved snapshot. ---\n");
    pause(2000);
    let mut second = App::launch(&args[1], 2);
    second.command(
        &format!("load {pin} checkpoint.gelset"),
        "REOPEN=PASS",
        4500,
    );
    second.command("find ram is volatile", "FIND=HIT", 8000);
    second.command("proof 1", "CITATION=PASS", 4500);
    second.finish();
    println!("\n$  Walkthrough complete. Snapshot reopened; citation verified.\nNo publication. Times above are application measurements, not typing delays.");
    pause(3000);
    let mut complete = new_file(std::path::Path::new("COMPLETE.txt")).expect("new completion");
    complete
        .write_all(b"LIVE_SUBPROCESS_WALKTHROUGH=PASS\n")
        .unwrap();
    complete.sync_all().unwrap();
}
