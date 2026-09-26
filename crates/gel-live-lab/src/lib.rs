//! Offline interactive evidence laboratory, not a speaker or semantic encoder.
#![forbid(unsafe_code)]
use gel_phase_quad::{grid::DIM, Policy, Reader, Record};
use gel_source::{
    digest, hex, import_text, load_bundle, read_regular, write_bundle_new, Address, Corpus,
    CorpusBuilder, EncodedCorpus, Error, Hash, MAX_TEXT,
};
use std::{ops::Range, path::Path, time::Instant};

pub fn safe(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}
pub fn parse_pin(raw: &str) -> Result<Hash, String> {
    if raw.len() != 64 || !raw.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Use an independently retained 64-digit SHA256 pin".into());
    }
    let mut out = [0; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&raw[2 * i..2 * i + 2], 16).map_err(|_| "Invalid pin")?;
    }
    Ok(out)
}
#[derive(Clone, Copy)]
pub struct QuadConfig {
    pub phase: u8,
    pub stride: usize,
    pub noise: usize,
    pub pole: u8,
}
impl Default for QuadConfig {
    fn default() -> Self {
        Self {
            phase: 0,
            stride: 1,
            noise: 0,
            pole: 0,
        }
    }
}
pub struct QuadResult {
    pub previews: [String; 4],
    pub score: f64,
    pub inverses: [bool; 4],
    pub elapsed_ns: u128,
    pub record_bytes: usize,
}
pub fn quad(c: QuadConfig) -> Result<QuadResult, String> {
    if c.stride > DIM || c.noise > DIM || c.pole > 3 {
        return Err("Quad parameter out of range".into());
    }
    let reader = Reader::new(510051);
    let original = Record::new(std::array::from_fn(|j| j as u8), &[true; DIM]);
    let body = Record::new(
        std::array::from_fn(|j| {
            (j as u8)
                .wrapping_add(c.phase)
                .wrapping_add(if j < c.noise {
                    (j as u8).wrapping_mul(73).wrapping_add(19)
                } else {
                    0
                })
        }),
        &std::array::from_fn(|j| c.stride != 0 && j % c.stride == 0),
    );
    let query = reader.prepare(original);
    let begin = Instant::now();
    let score = reader
        .read(
            std::hint::black_box(&query),
            std::hint::black_box(&body),
            Policy::BodyActivity,
        )
        .ok_or("empty query")?
        .value();
    let elapsed_ns = begin.elapsed().as_nanos();
    let mut previews = std::array::from_fn(|_| String::new());
    let mut inverses = [false; 4];
    for pole in 0..4 {
        let view = reader.bound_view(&body, pole as u8)?;
        previews[pole] = view.parts().1.phase[..16]
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        let restored = reader.restore_bound_view(&view)?;
        inverses[pole] =
            restored.phase() == body.phase() && restored.active_mask() == body.active_mask();
    }
    if !inverses.iter().all(|v| *v) {
        return Err("view inverse failed".into());
    }
    Ok(QuadResult {
        previews,
        score,
        inverses,
        elapsed_ns,
        record_bytes: std::mem::size_of::<Record>(),
    })
}
pub struct Lab {
    corpus: Corpus,
    encoded: Option<EncodedCorpus>,
    pub origin: String,
    pub quote: String,
    pub quote_label: String,
    pub read_ns: u128,
    pub text_bytes: Option<usize>,
    pub catalog_bytes: Option<usize>,
    pub config: QuadConfig,
    pub notice: String,
    pub page: usize,
    pub last_pin: Option<Hash>,
    matches: Vec<Range<usize>>,
}
impl Lab {
    pub fn demo() -> Result<Self, String> {
        let mut b = CorpusBuilder::new();
        for (i,text) in [
            "RAM holds working data while a program runs. Volatile memory alone does not preserve that data across a power loss. This laboratory can save an exact source catalog and verify it again in a fresh process.",
            "A cryptographic digest detects a change relative to independently trusted bytes. It does not prove that a statement is true. Changing the file and accepting a new digest from its sender would not authenticate the original source.",
            "Four reversible coordinate views describe one synthetic numeric record. The views are not four independent facts or four times the information. In this public implementation, a record includes phase codes and an activity mask.",
            "Unicode remains exact: Zażółć gęślą jaźń. Łódź, café, cafe\u{301}, 🦀. No accent stripping or newline conversion is applied to the source bytes.",
        ].iter().enumerate() {b.push(Address{node:1,role:i as u64+1},1,"GEL Live Lab notes",&format!("Lead ({})",i+1),&format!("{text}\n")).map_err(|e|format!("{e:?}"))?;}
        Self::from_encoded(
            b.finish().map_err(|e| format!("{e:?}"))?,
            "Built-in authored demonstration (4 parts)".into(),
        )
    }
    fn from_encoded(encoded: EncodedCorpus, origin: String) -> Result<Self, String> {
        let corpus = encoded.load().map_err(|e| format!("{e:?}"))?;
        let mut lab = Self {
            text_bytes: Some(encoded.text_bytes().len()),
            catalog_bytes: Some(encoded.catalog_bytes().len()),
            corpus,
            encoded: Some(encoded),
            origin,
            quote: String::new(),
            quote_label: String::new(),
            read_ns: 0,
            config: QuadConfig::default(),
            notice: "Ready. Type help for commands.".into(),
            page: 0,
            last_pin: None,
            matches: Vec::new(),
        };
        lab.select(1)?;
        Ok(lab)
    }
    pub fn count(&self) -> usize {
        self.corpus.records().count()
    }
    /// Search the exact whole source payload, not the byte-bounded part preview.
    /// The public catalog can represent multiple documents: refuse that case
    /// rather than infer continuity between unrelated nodes.
    pub fn find(&mut self, phrase: &str) -> Result<(), String> {
        let node = self
            .corpus
            .records()
            .next()
            .ok_or("Empty catalog")?
            .address
            .node;
        if self.corpus.records().any(|r| r.address.node != node) {
            return Err(
                "find supports a single source node; use read for a multi-node catalog".into(),
            );
        }
        let begin = Instant::now();
        let result = gel_source::document::search(self.corpus.source_text(), phrase)?;
        self.read_ns = begin.elapsed().as_nanos();
        self.matches = result.passages;
        self.page = 0;
        if self.matches.is_empty() {
            self.quote.clear();
            self.quote_label = "No matching quote in examined lines".into();
        } else {
            self.select_match(1)?;
        }
        self.notice = format!(
            "FIND={} matches={} shown={} skipped_long_lines={} | source phrase lookup; not semantic QA. Use match N for another result. Timing: search only; excludes import, load and rendering.",
            if result.skipped_long_lines > 0 { "INCOMPLETE" } else if self.matches.is_empty() { "UNKNOWN" } else { "HIT" },
            result.matching_lines, self.matches.len(), result.skipped_long_lines);
        Ok(())
    }
    pub fn select_match(&mut self, index: usize) -> Result<(), String> {
        let span = self
            .matches
            .get(index.checked_sub(1).ok_or("Matches start at 1")?)
            .ok_or("No such match; run find PHRASE first")?
            .clone();
        let context =
            gel_source::context::surrounding(self.corpus.source_text(), span.clone(), 512)?;
        self.quote = self
            .corpus
            .source_text()
            .get(context.context_span.clone())
            .ok_or("Invalid source span")?
            .to_string();
        self.quote_label = format!(
            "match {index}/{} | MATCH {}..{} CONTEXT {}..{} omitted_before={} omitted_after={} bounded context",
            self.matches.len(),
            span.start,
            span.end,
            context.context_span.start,
            context.context_span.end,
            context.omitted_before,
            context.omitted_after
        );
        self.page = 0;
        Ok(())
    }
    pub fn select(&mut self, index: usize) -> Result<(), String> {
        if index == 0 {
            return Err("Parts are numbered from 1".into());
        }
        let begin = Instant::now();
        let r = self.corpus.records().nth(index - 1).ok_or("No such part")?;
        let p = self.corpus.quote(r.address).map_err(|e| format!("{e:?}"))?;
        self.corpus.validate(&p).map_err(|e| format!("{e:?}"))?;
        self.matches.clear();
        self.read_ns = begin.elapsed().as_nanos();
        self.quote = p.quote().to_string();
        self.quote_label = format!(
            "part {index}/{} | address {}:{} | bytes {}..{}",
            self.count(),
            r.address.node,
            r.address.role,
            r.start,
            r.end
        );
        self.page = 0;
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let encoded = self
            .encoded
            .as_ref()
            .ok_or("Loaded bundle is read-only. Import a document to create another bundle.")?;
        let pin = write_bundle_new(path, encoded).map_err(|e| format!("Save refused: {e:?}"))?;
        self.last_pin = Some(pin);
        self.notice = format!("SAVED. Retain independently: {}", hex(&pin));
        Ok(())
    }
    pub fn load(&mut self, path: &Path, pin: Hash) -> Result<(), String> {
        // Transactional: a failed load must not replace the current session.
        let corpus = load_bundle(path, pin).map_err(|e| format!("Load refused: {e:?}"))?;
        if corpus.records().next().is_none() {
            return Err("Empty catalog".into());
        }
        self.corpus = corpus;
        self.encoded = None;
        self.text_bytes = None;
        self.catalog_bytes = None;
        self.origin = "Reopened bundle; verified against caller-supplied pin".into();
        self.last_pin = Some(pin);
        self.select(1)?;
        self.notice =
            "REOPEN=PASS. Exact bytes verified; statements are not certified true.".into();
        Ok(())
    }
    pub fn tamper(&mut self) -> Result<(), String> {
        if self.quote.is_empty() {
            return Err("Select a nonempty source quote before tamper".into());
        }
        // In-memory copy only: no modification of the document, original bundle or pin.
        let mut b = CorpusBuilder::new();
        b.push(
            Address { node: 1, role: 1 },
            1,
            "Selected quote",
            "Lead (1)",
            &self.quote,
        )
        .map_err(|e| format!("{e:?}"))?;
        let e = b.finish().map_err(|e| format!("{e:?}"))?;
        let mut copy = e.text_bytes().to_vec();
        copy[0] ^= 1;
        if !matches!(
            Corpus::load(e.catalog_bytes(), &copy, e.catalog_pin(), e.text_pin()),
            Err(Error::Integrity)
        ) {
            return Err("Integrity gate unexpectedly accepted changed data".into());
        }
        self.notice = "TAMPER=REJECTED | changed 1 byte in a RAM COPY; original untouched".into();
        Ok(())
    }
    pub fn command(&mut self, line: &str) -> Result<bool, String> {
        let (cmd, arg) = line.trim().split_once(' ').unwrap_or((line.trim(), ""));
        let arg = arg.trim();
        match cmd {
            "exit"|"quit" if arg.is_empty()=>return Ok(false),
            "demo" if arg.is_empty()=>*self=Self::demo()?,
            "open" if !arg.is_empty()=> {let bytes=read_regular(Path::new(arg),MAX_TEXT).map_err(|e|format!("Import refused: {e:?}"))?;let e=import_text(&bytes,"Imported document").map_err(|e|format!("Import refused: {e:?}"))?;*self=Self::from_encoded(e,"Caller-selected UTF-8 document (source path not displayed)".into())?;},
            "read"=>self.select(arg.parse().map_err(|_|"read PART_NUMBER")?)?,
            "find"=>self.find(arg)?,
            "match"=>self.select_match(arg.parse().map_err(|_|"match RESULT_NUMBER")?)?,
            "page"=> {let n=arg.parse::<usize>().map_err(|_|"page NUMBER")?;if n==0 || n>self.pages(){return Err("Page outside current quote".into());}self.page=n-1;},
            "save" if !arg.is_empty()=>self.save(Path::new(arg))?,
            "load"=>{let (pin,path)=arg.split_once(' ').ok_or("load TRUSTED_SHA256 PATH")?; if path.trim().is_empty(){return Err("Missing path".into());}self.load(Path::new(path.trim()),parse_pin(pin)?)?;},
            "tamper" if arg.is_empty()=>self.tamper()?,
            "phase"=>self.config.phase=arg.parse().map_err(|_|"phase 0..255")?,
            "mask"=>{let v=arg.parse::<usize>().map_err(|_|"mask 0..1024 (stride; 0 disables body)")?;if v>DIM{return Err("mask 0..1024".into());}self.config.stride=v;},
            "noise"=>{let v=arg.parse::<usize>().map_err(|_|"noise 0..1024")?;if v>DIM{return Err("noise 0..1024".into());}self.config.noise=v;},
            "view"=>{let v=arg.parse::<u8>().map_err(|_|"view 0..3")?;if v>3{return Err("view 0..3".into());}self.config.pole=v;},
            "help"|"show" if arg.is_empty()=>self.notice="open PATH | find PHRASE | match N | read N | page N | save NEW_PATH | load SHA256 PATH | tamper | phase 0..255 | mask 0..1024 | noise 0..1024 | view 0..3 | demo | exit".into(),
            _=>return Err("Unknown command. Type help. Paths with spaces need no quotes.".into()),
        }
        Ok(true)
    }
    pub fn quote_lines(&self) -> Vec<String> {
        let escaped = safe(&self.quote);
        let chars: Vec<_> = escaped.chars().collect();
        chars.chunks(72).map(|c| c.iter().collect()).collect()
    }
    pub fn pages(&self) -> usize {
        self.quote_lines().len().div_ceil(4).max(1)
    }
    pub fn render(&self, color: bool) -> Result<String, String> {
        let q = quad(self.config)?;
        let mut out = String::new();
        let cyan = if color { "\x1b[1;36m" } else { "" };
        let reset = if color { "\x1b[0m" } else { "" };
        out += &format!("{cyan}  GEL LIVE LAB  /  RUST  /  OFFLINE  /  NO LLM{reset}\n");
        out += "  ========================================================================\n";
        out += &format!("  SOURCE | {}\n", safe(&self.origin));
        out += &format!(
            "  Text: {} B | catalog: {} | parts: {} | plaintext, not a vault\n",
            self.text_bytes
                .map_or("not measured on reopen".into(), |v| v.to_string()),
            self.catalog_bytes
                .map_or("not measured on reopen".into(), |v| format!("{v} B")),
            self.count()
        );
        out += &format!(
            "  {} | {}: {:.3} ms\n",
            self.quote_label,
            if self.matches.is_empty() && !self.quote.is_empty() {
                "quote + validation"
            } else {
                "phrase search"
            },
            self.read_ns as f64 / 1e6
        );
        out += &format!(
            "  Quote page {}/{} (controls escaped; exact bytes remain in source):\n",
            self.page + 1,
            self.pages()
        );
        for line in self.quote_lines().iter().skip(self.page * 4).take(4) {
            out += &format!("  > {line}\n");
        }
        out += &format!(
            "  Quote SHA256:\n  {}\n",
            hex(&digest(self.quote.as_bytes()))
        );
        out += "  ------------------------------------------------------------------------\n";
        out += "  Q QUAD | SEPARATE synthetic numeric record, NOT an encoding of this text\n";
        out += &format!(
            "  Phase={} mask_stride={} noise={} | Record={} B, 256 phase levels\n",
            self.config.phase, self.config.stride, self.config.noise, q.record_bytes
        );
        for i in 0..4 {
            let preview = if color {
                q.previews[i]
                    .split(' ')
                    .map(|s| {
                        let v = u8::from_str_radix(s, 16).expect("internal hex preview");
                        format!(
                            "\x1b[48;2;{};{};{}m\x1b[30m{s}\x1b[0m",
                            90 + v / 2,
                            210 - v / 3,
                            255 - v / 2
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                q.previews[i].clone()
            };
            out += &format!(
                "  {} P{i} {}  inverse={}\n",
                if self.config.pole as usize == i {
                    ">"
                } else {
                    " "
                },
                preview,
                if q.inverses[i] { "PASS" } else { "FAIL" }
            );
        }
        out += &format!(
            "  Shared score: {:+.6} | one read: {} ns (not a benchmark)\n",
            q.score, q.elapsed_ns
        );
        out += "  4 reversible views = 1 evidence item, not 4x independent capacity\n";
        out += "  ------------------------------------------------------------------------\n";
        if let Some(pin) = self.last_pin {
            out += &format!(
                "  Retain bundle pin independently (load PIN PATH):\n  {}\n",
                hex(&pin)
            );
        }
        for line in safe(&self.notice).chars().collect::<Vec<_>>().chunks(74) {
            out += &format!("  {}\n", line.iter().collect::<String>());
        }
        out += "  Commands: help | open PATH | find PHRASE | match N | read N | save PATH | exit\n";
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_inverses_and_phase_response() {
        let a = quad(QuadConfig::default()).unwrap();
        assert_eq!(a.score, 1.);
        assert!(a.inverses.iter().all(|x| *x));
        let b = quad(QuadConfig {
            phase: 128,
            ..QuadConfig::default()
        })
        .unwrap();
        assert_eq!(b.score, -1.);
        assert_ne!(a.previews, b.previews);
    }
    #[test]
    fn no_body_support_means_zero() {
        assert_eq!(
            quad(QuadConfig {
                stride: 0,
                ..QuadConfig::default()
            })
            .unwrap()
            .score,
            0.
        );
    }
    #[test]
    fn views_do_not_mutate_quote() {
        let mut l = Lab::demo().unwrap();
        let before = l.quote.clone();
        for cmd in ["phase 91", "mask 3", "noise 100", "view 2"] {
            l.command(cmd).unwrap();
        }
        assert_eq!(l.quote, before);
        assert!(l.render(false).unwrap().contains("SEPARATE synthetic"));
    }
    #[test]
    fn tamper_never_changes_original() {
        let mut l = Lab::demo().unwrap();
        let before = l.quote.clone();
        l.tamper().unwrap();
        assert!(l.notice.contains("REJECTED"));
        assert_eq!(l.quote, before);
        l.select(1).unwrap();
        assert_eq!(l.quote, before);
    }
    #[test]
    fn bad_commands_leave_controls_unchanged() {
        let mut l = Lab::demo().unwrap();
        for c in [
            "mask 1025",
            "noise 999999",
            "phase 256",
            "view 4",
            "read 0",
            "page 0",
            "read 99",
            "save",
            "load nope file",
        ] {
            assert!(l.command(c).is_err(), "{c}");
        }
        assert_eq!(l.config.stride, 1);
        assert_eq!(l.config.noise, 0);
        assert_eq!(l.config.phase, 0);
    }
    #[test]
    fn terminal_controls_are_escaped() {
        let s = safe("hello\x1b[2J\r\u{202e}bad");
        assert!(!s.contains('\x1b'));
        assert!(!s.contains('\r'));
        assert!(!s.contains('\u{202e}'));
    }
    #[test]
    fn pin_parsing_is_bounded() {
        assert!(parse_pin(&"f".repeat(64)).is_ok());
        for p in ["f".repeat(63), "g".repeat(64), "🦀".repeat(16)] {
            assert!(parse_pin(&p).is_err());
        }
    }
    #[test]
    fn unicode_quote_is_preserved() {
        let mut l = Lab::demo().unwrap();
        l.select(4).unwrap();
        assert!(l.quote.contains("cafe\u{301}"));
        assert!(l.quote.contains("🦀"));
        assert!(!l.render(false).unwrap().contains('\x1b'));
    }
}
