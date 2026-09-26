//! Offline multi-document source laboratory. No shell, LLM or background import.
#![forbid(unsafe_code)]
use gel_live_lab::{parse_pin, safe};
use gel_source::{
    collection::{Collection, Hit},
    document, hex, read_regular,
};
use std::{
    io::{self, BufRead, Read, Write},
    path::Path,
    time::Instant,
};

struct Session {
    bank: Collection,
    hits: Vec<Hit>,
}
impl Session {
    fn command(&mut self, input: &str) -> Result<bool, String> {
        let (cmd, arg) = input.trim().split_once(' ').unwrap_or((input.trim(), ""));
        let arg = arg.trim();
        match cmd {
            "exit"|"quit" if arg.is_empty()=>return Ok(false),
            "help" if arg.is_empty()=>println!("add PATH | replace ID PATH | drop ID | list | find PHRASE | proof N | save NEW_PATH | load TRUSTED_SHA256 PATH | exit\nExact source phrases, not semantic AI. All files plaintext. Paths with spaces need no quotes."),
            "add" if !arg.is_empty()=>{
                let p=Path::new(arg);let b=read_regular(p,document::MAX_TEXT).map_err(|e|format!("{e:?}"))?;
                let text=std::str::from_utf8(&b).map_err(|_|"UTF8_REQUIRED")?;
                let title=p.file_name().and_then(|s|s.to_str()).ok_or("UTF8_FILENAME_REQUIRED")?;
                let id=self.bank.add(title,text)?; self.hits.clear();println!("ADDED id={id} bytes={}",b.len());
            },
            "replace"=>{let(id,p)=arg.split_once(' ').ok_or("replace ID PATH")?;let id=id.parse().map_err(|_|"INVALID_ID")?;
                let b=read_regular(Path::new(p.trim()),document::MAX_TEXT).map_err(|e|format!("{e:?}"))?;
                self.bank.replace(id,std::str::from_utf8(&b).map_err(|_|"UTF8_REQUIRED")?)?;self.hits.clear();println!("REPLACED id={id}; previous citations invalidated");},
            "drop"=>{let id=arg.parse().map_err(|_|"INVALID_ID")?;self.bank.remove(id)?;self.hits.clear();println!("REMOVED id={id}; old snapshots remain on disk");},
            "list" if arg.is_empty()=>{for d in self.bank.documents(){println!("id={} title={} bytes={} sha256={}",d.id(),safe(d.title()),d.text().len(),hex(&d.hash()));}},
            "find"=>{let start=Instant::now();let result=self.bank.search(arg)?;let ns=start.elapsed().as_nanos();
                println!("FIND={} documents={} matching_lines={} shown={} skipped_lines={} search_ns={ns}",result.status(),result.documents_examined,result.matching_lines,result.hits.len(),result.skipped_long_lines);
                for (n,h) in result.hits.iter().enumerate(){let d=self.bank.get(h.document_id()).ok_or("MISSING_DOCUMENT")?;
                    println!("RESULT {} doc={} title={} bytes={}..{}\nQUOTE {}",n+1,d.id(),safe(d.title()),h.span().start,h.span().end,safe(h.quote()));}
                self.hits=result.hits;
            },
            "proof"=>{let n=arg.parse::<usize>().map_err(|_|"INVALID_RESULT")?;let h=self.hits.get(n.checked_sub(1).ok_or("RESULTS_START_AT_1")?).ok_or("NO_CURRENT_RESULT")?;
                self.bank.validate(h)?;println!("CITATION=PASS document_sha256={} collection_sha256={}; correspondence, not truth",hex(&h.document_hash()),hex(&h.root()));},
            "save" if !arg.is_empty()=>{let start=Instant::now();let pin=self.bank.save_new(Path::new(arg))?;println!("SAVED revision={} save_ns={}\nBUNDLE_SHA256={}; retain independently",self.bank.revision(),start.elapsed().as_nanos(),hex(&pin));},
            "load"=>{let(pin,p)=arg.split_once(' ').ok_or("load TRUSTED_SHA256 PATH")?;if p.trim().is_empty(){return Err("MISSING_PATH".into());}
                let start=Instant::now();let bank=Collection::load(Path::new(p.trim()),parse_pin(pin)?)?;self.bank=bank;self.hits.clear();println!("REOPEN=PASS revision={} load_ns={}",self.bank.revision(),start.elapsed().as_nanos());},
            _=>return Err("Unknown command; type help".into()),
        }
        Ok(true)
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("gel-evidence [--demo]\nRust/offline multi-document phrase reader. Plaintext, not an encrypted vault.\nUse owner-controlled directories. Type help. Input is never a shell command.");
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
            .take(4097)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if line.len() > 4096 {
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
    if let Err(e) = run() {
        eprintln!("GEL_EVIDENCE=FAIL {}", safe(&e));
        std::process::exit(1);
    }
}
