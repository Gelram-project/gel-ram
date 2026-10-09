#![forbid(unsafe_code)]
use gel_live_lab::{literal, parse_pin, Lab};
use gel_source::digest;
use std::ffi::OsString;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::Path;
const USAGE: &str = "usage: gel-live-lab --help";
/// `--literal FILE [--save NEW_PATH]` or `--reopen TRUSTED_SHA256 PATH`.
/// Paths need not be UTF-8; a path starting with `--` is taken as a misplaced flag.
fn literal_run(args: &[OsString]) -> Result<(), String> {
    let word = |i: usize| args.get(i).and_then(|a| a.to_str());
    let path = |i: usize| match args.get(i) {
        Some(a) if !a.to_string_lossy().starts_with("--") => Ok(Path::new(a)),
        _ => Err(USAGE.to_string()),
    };
    let (lit, save) = match (word(0), args.len()) {
        (Some("--literal"), 2) => (literal::read(path(1)?)?, None),
        (Some("--literal"), 4) if word(2) == Some("--save") => {
            (literal::read(path(1)?)?, Some(path(3)?))
        }
        (Some("--reopen"), 3) => {
            let pin = parse_pin(word(1).ok_or(USAGE)?)?;
            (literal::reopen(path(2)?, &pin)?, None)
        }
        _ => return Err(USAGE.into()),
    };
    let views = literal::views(lit.record())?;
    let raw = literal::encode(lit.record())?;
    let pin = digest(&raw);
    // Every 97th byte, plus the first and last placed value and the last byte.
    let positions: Vec<usize> = (0..raw.len())
        .step_by(97)
        .chain([12, 12 + lit.placed() - 1, raw.len() - 1])
        .collect();
    let rejected = literal::tamper_rejected(&raw, &pin, &positions);
    if !literal::passed(&views, rejected) {
        print!(
            "{}",
            literal::report(
                &lit,
                &views,
                &pin,
                rejected,
                "Not written: the check failed."
            )
        );
        return Err("literal record check failed".into());
    }
    // Written only after every check passed.
    let line = match save {
        Some(new_path) => {
            if literal::save(new_path, lit.record())? != pin {
                return Err("saved bytes differ from the checked record".into());
            }
            "SAVED without replacing any file. Retain the pin independently."
        }
        None if lit.total().is_none() => {
            "REOPEN=PASS | the file matches the retained pin; record restored."
        }
        None => "Not written. Add --save NEW_PATH to keep it, then --reopen PIN PATH.",
    };
    print!("{}", literal::report(&lit, &views, &pin, rejected, line));
    println!("GEL_LIVE_LAB_LITERAL=PASS");
    Ok(())
}
fn run() -> Result<(), String> {
    let raw: Vec<OsString> = std::env::args_os().skip(1).collect();
    if matches!(
        raw.first().and_then(|a| a.to_str()),
        Some("--literal" | "--reopen")
    ) {
        return literal_run(&raw);
    }
    let args: Vec<&str> = raw.iter().map(|a| a.to_str().unwrap_or("\u{0}")).collect();
    if args == ["--help"] {
        println!("gel-live-lab [--plain | --demo | --literal FILE [--save NEW_PATH] | --reopen SHA256 PATH]\nOffline terminal laboratory. No network or LLM. open PATH imports UTF-8.\nfind PHRASE searches source lines; match N selects a result.\nsave NEW_PATH writes plaintext. load TRUSTED_SHA256 PATH reopens it.\nText and synthetic Q8 are separate panels, not a semantic encoder.\n--literal FILE places the first 1,024 bytes of one regular file into one Q8\nrecord and shows its four views; --save keeps it without replacing a file and\n--reopen checks it under the retained pin.\nLiteral bytes, not GEL knowledge printing.\nUse only directories you control. Never paste commands as document text.");
        return Ok(());
    }
    if !args.is_empty() && args != ["--plain"] && args != ["--demo"] {
        return Err(USAGE.into());
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
