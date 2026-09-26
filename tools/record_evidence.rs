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
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
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
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 2, "record_evidence ABSOLUTE_APP_BINARY");
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
