//! Scripted typing, unmodified live subprocess output; not a benchmark harness.
//! Run in a NEW directory containing memory.txt and unicode.txt.
use std::{
    fs,
    io::{Read, Write},
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
    output: Arc<Mutex<Vec<u8>>>,
    reader: Option<thread::JoinHandle<()>>,
}
impl Drop for App {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl App {
    fn launch(binary: &str) -> Self {
        print!("$ ");
        type_text("gel-evidence");
        pause(1500);
        println!();
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let input = child.stdin.take();
        let output = Arc::new(Mutex::new(Vec::new()));
        let shared = output.clone();
        let reader = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                let n = stdout.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                shared.lock().unwrap().extend_from_slice(&buf[..n]);
                std::io::stdout().write_all(&buf[..n]).unwrap();
                std::io::stdout().flush().unwrap();
            }
        });
        let app = Self {
            child,
            input,
            output,
            reader: Some(reader),
        };
        app.wait_for(0, "gel> ");
        pause(1800);
        app
    }
    fn text(&self) -> String {
        String::from_utf8(self.output.lock().unwrap().clone()).unwrap()
    }
    fn wait_for(&self, from: usize, needle: &str) {
        let start = Instant::now();
        while !self.text()[from..].contains(needle) {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "application response timeout"
            );
            pause(10);
        }
    }
    fn command(&mut self, cmd: &str, marker: &str, hold: u64) {
        let from = self.output.lock().unwrap().len();
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
            .create(true)
            .open("commands.txt")
            .unwrap();
        writeln!(events, "{cmd}").unwrap();
    }
    fn finish(mut self, name: &str) {
        self.command("exit", "Closed.", 1000);
        self.input.take();
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(start.elapsed() < Duration::from_secs(5));
            pause(10);
        }
        self.reader.take().unwrap().join().unwrap();
        fs::write(name, self.text()).unwrap();
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 2, "record_evidence ABSOLUTE_APP_BINARY");
    assert!(!std::path::Path::new("checkpoint.gelset").exists());
    assert!(!std::path::Path::new("COMPLETE.txt").exists());
    println!("GEL EVIDENCE LAB  |  Linux terminal  |  OFFLINE / CPU\nScripted typing; real application output. No LLM.\nPhrase retrieval, NOT semantic conversation.\n");
    pause(2000);
    let mut first = App::launch(&args[1]);
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
    first.finish("process-1.txt");
    println!("\n--- Process ended. Starting a new process; loading the saved snapshot. ---\n");
    pause(2000);
    let mut second = App::launch(&args[1]);
    second.command(
        &format!("load {pin} checkpoint.gelset"),
        "REOPEN=PASS",
        4500,
    );
    second.command("find ram is volatile", "FIND=HIT", 8000);
    second.command("proof 1", "CITATION=PASS", 4500);
    second.finish("process-2.txt");
    println!("\n$  Walkthrough complete. Snapshot reopened; citation verified.\nNo publication. Times above are application measurements, not typing delays.");
    pause(3000);
    fs::write("COMPLETE.txt", "LIVE_SUBPROCESS_WALKTHROUGH=PASS\n").unwrap();
}
