#![forbid(unsafe_code)]
use gel_live_lab::Lab;
use std::io::{self, BufRead, IsTerminal, Read, Write};
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("gel-live-lab [--plain | --demo]\nOffline terminal laboratory. No network or LLM. open PATH imports UTF-8.\nfind PHRASE searches source lines; match N selects a result.\nsave NEW_PATH writes plaintext. load TRUSTED_SHA256 PATH reopens it.\nText and synthetic Q8 are separate panels, not a semantic encoder.\nUse only directories you control. Never paste commands as document text.");
        return Ok(());
    }
    if !args.is_empty() && args != ["--plain"] && args != ["--demo"] {
        return Err("usage: gel-live-lab --help".into());
    }
    let color = args.is_empty()
        && io::stdout().is_terminal()
        && io::stdin().is_terminal()
        && std::env::var_os("NO_COLOR").is_none();
    let mut lab = Lab::demo()?;
    if args == ["--demo"] {
        for cmd in [
            "show",
            "read 2",
            "find independently trusted",
            "phase 64",
            "view 3",
            "tamper",
        ] {
            lab.command(cmd)?;
            print!("{}", lab.render(false)?);
        }
        println!("GEL_LIVE_LAB_DEMO=PASS");
        return Ok(());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    loop {
        if color {
            print!("\x1b[2J\x1b[H");
        }
        print!("{}\n gel> ", lab.render(color)?);
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut line = String::new();
        let n = input
            .by_ref()
            .take(4097)
            .read_line(&mut line)
            .map_err(|e| format!("Input: {e}"))?;
        if n == 0 {
            break;
        }
        if line.len() > 4096 {
            return Err("Command exceeds 4096-byte limit".into());
        }
        match lab.command(&line) {
            Ok(false) => break,
            Ok(true) => {}
            Err(e) => lab.notice = format!("REFUSED: {e}"),
        }
    }
    println!("GEL Live Lab closed. Only explicitly saved bundles persist.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("GEL_LIVE_LAB=FAIL {}", gel_live_lab::safe(&e));
        std::process::exit(1);
    }
}
