//! Finite, explicit mutation matrix for the public file formats GELSRC01,
//! GELSET01 and Q8DEMO01 (docs/MUTATION-MATRIX.md).
//!
//! Every mutant is read twice. With the ORIGINAL pin it must be rejected before
//! any structure is interpreted. With a RECOMPUTED pin (the pin of the mutated
//! bytes) the structural rules decide, and the expected result of each mutant
//! is fixed independently of the library: for GELSET01 and Q8DEMO01 by a small
//! reference reader written from the published format description, for
//! GELSRC01 by an explicit table per region and operator. A valid changed value
//! (another letter, another Q8 phase or mask bit) is an accepted, different
//! file, not an error; accepted content must equal the reference reading.
use gel_phase_quad::{fixture, grid::DIM, Record};
use gel_source::{
    collection::Collection, digest, hex, import_text, load_bundle, write_bundle_new, BundleError,
    Hash,
};
use std::{fmt::Write as _, fs, path::Path};

const REPORT: &str = "docs/evidence-mutation/matrix-r1.txt";

#[derive(Clone, Copy)]
enum Int {
    U32,
    U64,
}

struct Region {
    name: String,
    start: usize,
    end: usize,
    int: Option<Int>,
}

/// Base file, its regions and the mutants derived from it.
type Case = (Vec<u8>, Vec<Region>, Vec<Mutant>);

struct Mutant {
    region: String,
    operator: String,
    bytes: Vec<u8>,
}

fn region(name: &str, start: usize, end: usize, int: Option<Int>) -> Region {
    Region {
        name: name.into(),
        start,
        end,
        int,
    }
}

fn read_int(bytes: &[u8], r: &Region, kind: Int) -> u64 {
    match kind {
        Int::U32 => u64::from(u32::from_le_bytes(
            bytes[r.start..r.end].try_into().unwrap(),
        )),
        Int::U64 => u64::from_le_bytes(bytes[r.start..r.end].try_into().unwrap()),
    }
}

fn write_int(bytes: &mut [u8], r: &Region, kind: Int, value: u64) {
    match kind {
        Int::U32 => bytes[r.start..r.end].copy_from_slice(&(value as u32).to_le_bytes()),
        Int::U64 => bytes[r.start..r.end].copy_from_slice(&value.to_le_bytes()),
    }
}

/// The same finite operator set for every format; identity mutations are skipped.
fn generic(base: &[u8], regions: &[Region]) -> Vec<Mutant> {
    let mut out = Vec::new();
    let mut push = |region: &str, operator: &str, bytes: Vec<u8>| {
        if bytes != base {
            out.push(Mutant {
                region: region.into(),
                operator: operator.into(),
                bytes,
            });
        }
    };
    for r in regions.iter().filter(|r| r.end > r.start) {
        let mut b = base.to_vec();
        b[r.start] ^= 0x01;
        push(&r.name, "flip-first-bit0", b);
        let mut b = base.to_vec();
        b[r.end - 1] ^= 0x80;
        push(&r.name, "flip-last-bit7", b);
        let mut b = base.to_vec();
        b[r.start] = 0x00;
        push(&r.name, "set-first-00", b);
        let mut b = base.to_vec();
        b[r.start] = 0xff;
        push(&r.name, "set-first-ff", b);
        if let Some(kind) = r.int {
            let value = read_int(base, r, kind);
            let max = match kind {
                Int::U32 => u64::from(u32::MAX),
                Int::U64 => u64::MAX,
            };
            for (op, new) in [
                ("int+1", value.checked_add(1).filter(|v| *v <= max)),
                ("int-1", value.checked_sub(1)),
                ("int=0", Some(0)),
                ("int=max", Some(max)),
            ] {
                if let Some(new) = new {
                    let mut b = base.to_vec();
                    write_int(&mut b, r, kind, new);
                    push(&r.name, op, b);
                }
            }
        }
        if r.start > 0 {
            push(&r.name, "truncate-before", base[..r.start].to_vec());
        }
    }
    push(
        "file",
        "truncate-last-byte",
        base[..base.len() - 1].to_vec(),
    );
    let mut b = base.to_vec();
    b.push(0);
    push("file", "append-00", b);
    out
}

/// `ACCEPT` with the read content (compared, not shown) or `REJECT:CLASS`.
#[derive(PartialEq, Eq)]
enum Reading {
    Accept(String),
    Reject(String),
}
impl Reading {
    fn label(&self) -> String {
        match self {
            Self::Accept(_) => "ACCEPT".into(),
            Self::Reject(class) => format!("REJECT:{class}"),
        }
    }
}

// ---------- GELSET01: reference reader from docs/EVIDENCE-LAB.md ----------

fn gelset_summary(revision: u64, docs: &[(u64, String, String)]) -> String {
    let mut s = format!("rev={revision}");
    for (id, title, text) in docs {
        let _ = write!(
            s,
            ";{id}:{}:{}",
            hex(&digest(title.as_bytes())),
            hex(&digest(text.as_bytes()))
        );
    }
    s
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.bytes.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(s)
    }
}

/// Sequential reading in file order: header, then each entry, then no trailing bytes.
fn gelset_reference(b: &[u8]) -> Reading {
    gelset_read(b, true)
}

/// `strict_end: false` is the deliberately lenient reader of the negative control.
fn gelset_read(b: &[u8], strict_end: bool) -> Reading {
    let reject = |c: &str| Reading::Reject(format!("COLLECTION_{c}"));
    let mut cur = Cursor { bytes: b, at: 0 };
    let u64_at = |s: &[u8]| u64::from_le_bytes(s.try_into().unwrap());
    let Some(magic) = cur.take(8) else {
        return reject("TRUNCATED");
    };
    if magic != b"GELSET01" {
        return reject("MAGIC");
    }
    let (Some(rev), Some(next), Some(count)) = (cur.take(8), cur.take(8), cur.take(8)) else {
        return reject("TRUNCATED");
    };
    let (revision, next_id, count) = (u64_at(rev), u64_at(next), u64_at(count));
    if next_id == 0 || next_id - 1 > revision || count > 1024 {
        return reject("HEADER");
    }
    let (mut previous, mut total, mut docs) = (0u64, 0usize, Vec::new());
    for _ in 0..count {
        let (Some(id), Some(tl), Some(xl)) = (cur.take(8), cur.take(4), cur.take(8)) else {
            return reject("TRUNCATED");
        };
        let id = u64_at(id);
        let title_len = u32::from_le_bytes(tl.try_into().unwrap()) as usize;
        let text_len = u64_at(xl) as usize;
        if id <= previous
            || id >= next_id
            || title_len > 512
            || text_len > 16 << 20
            || text_len > (64 << 20) - total
        {
            return reject("RECORD");
        }
        let Some(title) = cur.take(title_len) else {
            return reject("TRUNCATED");
        };
        let Ok(title) = std::str::from_utf8(title) else {
            return reject("UTF8");
        };
        let Some(text) = cur.take(text_len) else {
            return reject("TRUNCATED");
        };
        let Ok(text) = std::str::from_utf8(text) else {
            return reject("UTF8");
        };
        if title.trim().is_empty() || title.chars().any(char::is_control) {
            return reject("TITLE");
        }
        if text.is_empty() {
            return reject("DOCUMENT_LIMIT");
        }
        docs.push((id, title.to_string(), text.to_string()));
        total += text_len;
        previous = id;
    }
    if strict_end && cur.at != b.len() {
        return reject("TRAILING_BYTES");
    }
    Reading::Accept(gelset_summary(revision, &docs))
}

fn gelset_library(b: &[u8], pin: Hash) -> Reading {
    match Collection::from_bytes(b, pin) {
        Ok(c) => {
            let docs: Vec<_> = c
                .documents()
                .map(|d| (d.id(), d.title().to_string(), d.text().to_string()))
                .collect();
            Reading::Accept(gelset_summary(c.revision(), &docs))
        }
        Err(class) => Reading::Reject(class),
    }
}

fn gelset_case() -> Result<Case, String> {
    let mut c = Collection::new();
    c.add("a.txt", "Alpha one.\n")?;
    c.add("b.txt", "Beta two.\n")?;
    let base = c.to_bytes();
    let mut regions = vec![
        region("magic", 0, 8, None),
        region("revision", 8, 16, Some(Int::U64)),
        region("next_id", 16, 24, Some(Int::U64)),
        region("count", 24, 32, Some(Int::U64)),
    ];
    let mut at = 32;
    let mut entries = Vec::new();
    for n in 1..=2 {
        let title_len = u32::from_le_bytes(base[at + 8..at + 12].try_into().unwrap()) as usize;
        let text_len = u64::from_le_bytes(base[at + 12..at + 20].try_into().unwrap()) as usize;
        let start = at;
        regions.push(region(&format!("entry{n}.id"), at, at + 8, Some(Int::U64)));
        regions.push(region(
            &format!("entry{n}.title_len"),
            at + 8,
            at + 12,
            Some(Int::U32),
        ));
        regions.push(region(
            &format!("entry{n}.text_len"),
            at + 12,
            at + 20,
            Some(Int::U64),
        ));
        at += 20;
        regions.push(region(&format!("entry{n}.title"), at, at + title_len, None));
        at += title_len;
        regions.push(region(&format!("entry{n}.text"), at, at + text_len, None));
        at += text_len;
        entries.push(start..at);
    }
    let mut mutants = generic(&base, &regions);
    let (e1, e2) = (entries[0].clone(), entries[1].clone());
    let mut swapped = base[..32].to_vec();
    swapped.extend_from_slice(&base[e2.clone()]);
    swapped.extend_from_slice(&base[e1.clone()]);
    mutants.push(Mutant {
        region: "entries".into(),
        operator: "swap-order".into(),
        bytes: swapped,
    });
    let mut duplicated = base.clone();
    write_int(&mut duplicated, &regions[3], Int::U64, 3);
    duplicated.extend_from_slice(&base[e1.clone()]);
    mutants.push(Mutant {
        region: "entries".into(),
        operator: "duplicate-first".into(),
        bytes: duplicated,
    });
    let text = regions
        .iter()
        .find(|r| r.name == "entry1.text")
        .unwrap()
        .start;
    let mut letter = base.clone();
    letter[text] = b'Q';
    mutants.push(Mutant {
        region: "entry1.text".into(),
        operator: "valid-letter-change".into(),
        bytes: letter,
    });
    let title = regions
        .iter()
        .find(|r| r.name == "entry1.title")
        .unwrap()
        .start;
    let mut control = base.clone();
    control[title] = 0x07;
    mutants.push(Mutant {
        region: "entry1.title".into(),
        operator: "control-character".into(),
        bytes: control,
    });
    Ok((base, regions, mutants))
}

// ---------- Q8DEMO01: reference reader from the fixture description ----------

fn q8_summary(records: &[(&[u8], &[u8])]) -> String {
    records
        .iter()
        .map(|(phase, mask)| format!("{}:{}", hex(&digest(phase)), hex(&digest(mask))))
        .collect::<Vec<_>>()
        .join(";")
}

fn q8_reference(b: &[u8]) -> Reading {
    if b.len() < 12 || &b[..8] != b"Q8DEMO01" {
        return Reading::Reject("Q8_HEADER".into());
    }
    let count = u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize;
    if !(1..=8192).contains(&count) || b.len() != 12 + count * 1152 {
        return Reading::Reject("Q8_LENGTH".into());
    }
    let records: Vec<_> = b[12..]
        .chunks_exact(1152)
        .map(|r| (&r[..1024], &r[1024..]))
        .collect();
    Reading::Accept(q8_summary(&records))
}

fn q8_library(b: &[u8], pin: Hash) -> Reading {
    // Q8DEMO01 has no built-in checksum: the caller applies its retained pin first.
    if digest(b) != pin {
        return Reading::Reject("PIN(caller)".into());
    }
    match fixture::decode(b) {
        Ok(records) => {
            let masks: Vec<Vec<u8>> = records
                .iter()
                .map(|r| {
                    let m = r.active_mask();
                    (0..DIM / 8)
                        .map(|i| (0..8).fold(0u8, |a, bit| a | (u8::from(m[i * 8 + bit]) << bit)))
                        .collect()
                })
                .collect();
            let pairs: Vec<_> = records
                .iter()
                .zip(&masks)
                .map(|(r, m)| (&r.phase()[..], m.as_slice()))
                .collect();
            Reading::Accept(q8_summary(&pairs))
        }
        Err(e) if e.contains("header") => Reading::Reject("Q8_HEADER".into()),
        Err(_) => Reading::Reject("Q8_LENGTH".into()),
    }
}

fn q8_case() -> Result<Case, String> {
    let phase: [u8; DIM] = std::array::from_fn(|j| j as u8);
    let active: [bool; DIM] = std::array::from_fn(|j| j % 3 != 0);
    let base = fixture::encode(&[
        Record::new(phase, &active),
        Record::new([200; DIM], &[true; DIM]),
    ])?;
    let regions = vec![
        region("magic", 0, 8, None),
        region("count", 8, 12, Some(Int::U32)),
        region("record0.phase", 12, 12 + 1024, None),
        region("record0.mask", 12 + 1024, 12 + 1152, None),
        region("record1.phase", 12 + 1152, 12 + 1152 + 1024, None),
        region("record1.mask", 12 + 1152 + 1024, 12 + 2 * 1152, None),
    ];
    let mut mutants = generic(&base, &regions);
    let mut value = base.clone();
    value[12 + 5] = value[12 + 5].wrapping_add(1);
    mutants.push(Mutant {
        region: "record0.phase".into(),
        operator: "valid-phase-change".into(),
        bytes: value,
    });
    Ok((base, regions, mutants))
}

// ---------- GELSRC01: explicit expectations per region and operator ----------

/// Declared by hand from docs/SOURCE-BUILDER.md: a 24-byte header (magic and two
/// lengths) must add up to the file size; the catalog starts with its format
/// line and every passage carries the SHA-256 of its quoted text bytes.
fn gelsrc_expected(region: &str, operator: &str) -> &'static str {
    match (region, operator) {
        ("catalog_len" | "text_len", "int=max" | "flip-last-bit7") => "REJECT:LIMIT",
        ("text", "flip-first-bit0" | "set-first-00") => "REJECT:INTEGRITY",
        _ => "REJECT:FORMAT",
    }
}

fn gelsrc_library(dir: &Path, n: usize, b: &[u8], pin: Hash) -> Result<Reading, String> {
    let path = dir.join(format!("case-{n}.gelsrc"));
    fs::write(&path, b).map_err(|e| e.to_string())?;
    let result = load_bundle(&path, pin);
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(match result {
        Ok(_) => Reading::Accept(String::new()),
        Err(BundleError::Data(e)) => Reading::Reject(format!("{e:?}").to_uppercase()),
        Err(e) => return Err(format!("case {n}: {e:?}")),
    })
}

fn gelsrc_case(dir: &Path) -> Result<Case, String> {
    let corpus =
        import_text(b"Alpha line one.\nBeta line two.\n", "notes").map_err(|e| format!("{e:?}"))?;
    let path = dir.join("base.gelsrc");
    write_bundle_new(&path, &corpus).map_err(|e| format!("{e:?}"))?;
    let base = fs::read(&path).map_err(|e| e.to_string())?;
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    let catalog = u64::from_le_bytes(base[8..16].try_into().unwrap()) as usize;
    let regions = vec![
        region("magic", 0, 8, None),
        region("catalog_len", 8, 16, Some(Int::U64)),
        region("text_len", 16, 24, Some(Int::U64)),
        region("catalog", 24, 24 + catalog, None),
        region("text", 24 + catalog, base.len(), None),
    ];
    let mutants = generic(&base, &regions);
    Ok((base, regions, mutants))
}

// ---------- matrix ----------

fn row(out: &mut String, cells: [&str; 8]) {
    out.push_str(&cells.join("\t"));
    out.push('\n');
}

/// Builds the whole report; returns it with the number of failed rows.
fn build() -> Result<(String, usize, usize), String> {
    let dir = std::env::temp_dir().join(format!("gel-mutation-matrix-{}", std::process::id()));
    fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let result = build_in(&dir);
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    result
}

fn build_in(dir: &Path) -> Result<(String, usize, usize), String> {
    let mut out =
        String::from("format\tcase\tregion\toperator\tpin\texpected\tobserved\tverdict\n");
    let (mut cases, mut failed) = (0, 0);
    let mut record = |out: &mut String,
                      format: &str,
                      n: usize,
                      m: &Mutant,
                      pin: &str,
                      expected: &Reading,
                      observed: &Reading| {
        let ok = expected == observed;
        failed += usize::from(!ok);
        row(
            out,
            [
                format,
                &n.to_string(),
                &m.region,
                &m.operator,
                pin,
                &expected.label(),
                &observed.label(),
                if ok { "PASS" } else { "FAIL" },
            ],
        );
    };

    let (base, _, mutants) = gelsrc_case(dir)?;
    let base_pin = digest(&base);
    for (n, m) in mutants.iter().enumerate() {
        cases += 1;
        let old = gelsrc_library(dir, n, &m.bytes, base_pin)?;
        record(
            &mut out,
            "GELSRC01",
            n,
            m,
            "original",
            &Reading::Reject("INTEGRITY".into()),
            &old,
        );
        let declared = gelsrc_expected(&m.region, &m.operator);
        let expected = Reading::Reject(declared.trim_start_matches("REJECT:").into());
        let new = gelsrc_library(dir, n, &m.bytes, digest(&m.bytes))?;
        record(&mut out, "GELSRC01", n, m, "recomputed", &expected, &new);
    }

    let (base, _, mutants) = gelset_case()?;
    let base_pin = digest(&base);
    for (n, m) in mutants.iter().enumerate() {
        cases += 1;
        let old = gelset_library(&m.bytes, base_pin);
        record(
            &mut out,
            "GELSET01",
            n,
            m,
            "original",
            &Reading::Reject("COLLECTION_INTEGRITY".into()),
            &old,
        );
        let new = gelset_library(&m.bytes, digest(&m.bytes));
        record(
            &mut out,
            "GELSET01",
            n,
            m,
            "recomputed",
            &gelset_reference(&m.bytes),
            &new,
        );
    }

    let (base, _, mutants) = q8_case()?;
    let base_pin = digest(&base);
    for (n, m) in mutants.iter().enumerate() {
        cases += 1;
        let old = q8_library(&m.bytes, base_pin);
        record(
            &mut out,
            "Q8DEMO01",
            n,
            m,
            "original",
            &Reading::Reject("PIN(caller)".into()),
            &old,
        );
        let new = q8_library(&m.bytes, digest(&m.bytes));
        record(
            &mut out,
            "Q8DEMO01",
            n,
            m,
            "recomputed",
            &q8_reference(&m.bytes),
            &new,
        );
    }
    Ok((out, cases, failed))
}

/// Negative control: how many GELSET01 mutants tell the strict reference apart
/// from a reader that ignores trailing bytes. Zero would mean the matrix cannot
/// catch such a reader, so the gate fails.
fn negative_control() -> Result<usize, String> {
    let (_, _, mutants) = gelset_case()?;
    Ok(mutants
        .iter()
        .filter(|m| gelset_read(&m.bytes, true) != gelset_read(&m.bytes, false))
        .count())
}

fn summary(report: &str, cases: usize, failed: usize, control: usize) -> String {
    let count = |needle: &str| report.lines().filter(|l| l.contains(needle)).count();
    format!(
        "MUTATION_MATRIX={} mutants={cases} rows={} accepted_valid_changes={} failed={failed} negative_control_distinguishing_mutants={control}",
        if failed == 0 && control > 0 { "PASS" } else { "FAIL" },
        2 * cases,
        count("\trecomputed\tACCEPT\tACCEPT\t"),
    )
}

/// Builds the report and the summary line; Err carries the summary on failure.
fn evaluate() -> Result<(String, String), String> {
    let (report, cases, failed) = build()?;
    let control = negative_control()?;
    let line = summary(&report, cases, failed, control);
    if failed > 0 || control == 0 {
        for fail in report.lines().filter(|l| l.ends_with("\tFAIL")) {
            eprintln!("{fail}");
        }
        return Err(line);
    }
    Ok((report, line))
}

/// `mutation-matrix`: prints the summary; `--write NEW_FILE` also saves the report.
pub fn run(args: &[String]) -> Result<(), String> {
    let path = match args {
        [] => None,
        [flag, path] if flag == "--write" => Some(path),
        _ => return Err("usage: cargo run -p xtask -- mutation-matrix [--write NEW_FILE]".into()),
    };
    let (report, line) = evaluate()?;
    if let Some(path) = path {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, report.as_bytes()))
            .map_err(|e| format!("{path}: {e}"))?;
    }
    println!("{line}");
    Ok(())
}

/// verify: the matrix must pass and equal the committed report byte for byte.
pub fn check(root: impl AsRef<Path>) -> Result<(), String> {
    let (report, line) = evaluate()?;
    let committed =
        fs::read_to_string(root.as_ref().join(REPORT)).map_err(|e| format!("{REPORT}: {e}"))?;
    if committed != report {
        return Err(format!("{REPORT} differs from the regenerated matrix"));
    }
    println!("{line} report={REPORT}");
    Ok(())
}
