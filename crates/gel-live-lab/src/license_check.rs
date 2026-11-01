// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Checks a distribution tree, and a user's own answers about one use, against
//! GEL RAM Noncommercial Reciprocal License 1.0 (docs/LICENSE-DECISION.md).
//!
//! Every rule of `gel-license` is in this module: the program only reads its
//! arguments and prints what these functions return. The tree check is
//! mechanical, offline and read-only, and reads only the license, notice,
//! third-party notice, header and crate-name items it lists; the use check reads the user's answers
//! and does not interpret facts. Neither is legal advice or a certification:
//! every report carries `LEGAL_CERTIFICATION=NO`, and the LICENSE text
//! controls (LICENSE section 17).
use gel_source::digest;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use gel_core::{source_header_like, LICENSE_ID, REQUIRED_NOTICE, SOURCE_HEADER, UTF8_BOM};

/// The contact of LICENSE section 19.
pub const CONTACT: &str = "gelram.licensing@gmail.com";

/// First line of the source header of a GEL RAM-owned `.rs` file, the one
/// `xtask source-headers` writes (`gel_core::SOURCE_HEADER_LINE_1`).
pub const HEADER_LINE_1: &str = gel_core::SOURCE_HEADER_LINE_1;
/// Second line of the source header: the Required Notice as a `//` comment
/// (`gel_core::SOURCE_HEADER_LINE_2`).
pub const HEADER_LINE_2: &str = gel_core::SOURCE_HEADER_LINE_2;
/// Text that marks a top-level LICENSE or NOTICE file as naming GEL RAM.
const GEL_NAME: &[u8] = b"GEL RAM";
/// The package names of the crates of this workspace. Only these count in
/// `GEL_CRATES`; another crate whose name merely starts with `gel-` does not.
pub const GEL_CRATE_NAMES: [&str; 13] = [
    "gel-bench",
    "gel-cli",
    "gel-core",
    "gel-history",
    "gel-kernel",
    "gel-live-lab",
    "gel-orb",
    "gel-phase-quad",
    "gel-physics",
    "gel-reader",
    "gel-source",
    "gel-store",
    "gel-structural",
];

/// The crate of this workspace that carries third-party material: the tables
/// derived from Unicode data and the MIT-licensed Rust Book fixture.
pub const THIRD_PARTY_CRATE: &str = "gel-source";
/// Name of the file of the tables derived from Unicode data.
pub const UNICODE_TABLES_FILE: &str = "nfc_tables.rs";
/// The top-level file of third-party notices.
pub const THIRD_PARTY_NOTICES_FILE: &str = "THIRD-PARTY-NOTICES.md";
/// The line of [`THIRD_PARTY_NOTICES_FILE`] that names the Unicode License v3.
pub const UNICODE_NOTICE_LINE: &str = "## 2a. Unicode data (Unicode License v3)";
/// Name of the file of the Unicode License v3 text, anywhere in the tree.
pub const UNICODE_LICENSE_FILE: &str = "LICENSE-UNICODE.txt";

/// The root LICENSE of this workspace, pinned into the program at build time.
pub const LICENSE_TEXT: &[u8] = include_bytes!("../../../LICENSE");
/// SHA-256 of [`LICENSE_TEXT`], as listed in docs/LICENSE-PINS.md.
pub const LICENSE_SHA256: &str = "c0b560ffc53ad735c4a6236356ff0cd47d92cf671181e799917975b071e8cd40";

/// Most directory entries (files, directories, links) one check reads; a larger tree is refused.
pub const MAX_ENTRIES: usize = 200_000;
/// Most bytes read from one Cargo.toml, top-level LICENSE, NOTICE or
/// THIRD-PARTY-NOTICES.md; a larger file is skipped and counted.
pub const MAX_READ: u64 = 1 << 20;
/// Bytes read from the start of a `.rs` file: only its first two lines are inspected.
pub const HEADER_READ: u64 = 4096;
/// Largest answers file `use` reads.
pub const MAX_ANSWERS: u64 = 64 << 10;

/// The fixed `NOTE=` line of a `check` report.
pub const CHECK_NOTE: &str = "NOTE=this reads only the items listed above; it does not show that every copyright and third-party notice was kept, nor that any use is licensed";

// ---------------------------------------------------------------------------
// check DIR
// ---------------------------------------------------------------------------

/// `LICENSE_FILE=`: the top-level LICENSE compared with [`LICENSE_TEXT`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LicenseFile {
    /// Byte-equal to the pinned LICENSE.
    Exact,
    /// A regular file named LICENSE that is not byte-equal.
    Changed,
    /// No regular file named LICENSE at the top level.
    #[default]
    Missing,
}

/// `REQUIRED_NOTICE=`: whether a top-level LICENSE or NOTICE carries [`REQUIRED_NOTICE`] as a line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Notice {
    Kept,
    #[default]
    Missing,
}

/// `THIRD_PARTY_NOTICES=`: the notices of this workspace's third-party
/// material, needed when the tree declares [`THIRD_PARTY_CRATE`] in a
/// Cargo.toml or holds a file named [`UNICODE_TABLES_FILE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThirdParty {
    /// Needed, and the top-level THIRD-PARTY-NOTICES.md holds
    /// [`UNICODE_NOTICE_LINE`] and a regular file named LICENSE-UNICODE.txt is
    /// in the tree.
    Kept,
    /// Needed, and one of those was not found.
    Missing,
    /// Neither signal of this workspace's third-party material was found.
    NotNeeded,
}

impl ThirdParty {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kept => "KEPT",
            Self::Missing => "MISSING",
            Self::NotNeeded => "NOT_NEEDED",
        }
    }
}

/// `DISTRIBUTION_NOTICES=`: the overall finding of a tree check, about the
/// listed items only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distribution {
    /// A GEL RAM notice was found, the LICENSE is exact, the Required Notice
    /// is kept, the third-party notices are kept or not needed and no GEL
    /// header is altered.
    ListedItemsPresent,
    /// A GEL RAM notice was found and one of those was not.
    ListedItemsMissing,
    /// None of the listed GEL RAM notices, headers or crate names was found.
    NoGelNoticesFound,
}

impl LicenseFile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Exact => "EXACT",
            Self::Changed => "CHANGED",
            Self::Missing => "MISSING",
        }
    }
}

impl Notice {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kept => "KEPT",
            Self::Missing => "MISSING",
        }
    }
}

impl Distribution {
    pub fn name(self) -> &'static str {
        match self {
            Self::ListedItemsPresent => "LISTED_ITEMS_PRESENT",
            Self::ListedItemsMissing => "LISTED_ITEMS_MISSING",
            Self::NoGelNoticesFound => "NO_GEL_NOTICES_FOUND",
        }
    }
    /// Exit code of `gel-license check`: 0 listed items present, 3 listed
    /// items missing, 4 no GEL RAM notices found.
    pub fn exit_code(self) -> u8 {
        match self {
            Self::ListedItemsPresent => 0,
            Self::ListedItemsMissing => 3,
            Self::NoGelNoticesFound => 4,
        }
    }
}

/// Bounds of one tree check.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_entries: usize,
    pub max_read: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: MAX_ENTRIES,
            max_read: MAX_READ,
        }
    }
}

/// What a tree check saw.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeReport {
    pub license_file: LicenseFile,
    /// The LICENSE is `CHANGED` only because its lines end with CR LF.
    pub license_line_endings_only: bool,
    pub required_notice: Notice,
    /// `.rs` files one of whose first two lines, after an optional byte order
    /// mark, looks like a header line (`gel_core::source_header_like`).
    pub header_files: usize,
    /// Of those, files that start with the exact source header.
    pub header_intact: usize,
    /// Package and dependency names of [`GEL_CRATE_NAMES`] declared in any Cargo.toml.
    pub crates: BTreeSet<String>,
    /// A top-level LICENSE or NOTICE that was read contains the text `GEL RAM`.
    pub names_gel_ram: bool,
    /// A regular file named [`UNICODE_TABLES_FILE`] was seen.
    pub unicode_tables: bool,
    /// The top-level THIRD-PARTY-NOTICES.md, read within the bound, holds
    /// [`UNICODE_NOTICE_LINE`] as a whole line.
    pub unicode_notice: bool,
    /// A regular file named [`UNICODE_LICENSE_FILE`] was seen.
    pub unicode_license: bool,
    /// Regular files seen.
    pub files: usize,
    /// Symbolic links seen and not followed.
    pub symlinks_skipped: usize,
    /// Files that would have been read but exceed [`Limits::max_read`].
    pub oversize_skipped: usize,
}

impl TreeReport {
    pub fn header_altered(&self) -> usize {
        self.header_files - self.header_intact
    }

    /// The rule of `THIRD_PARTY_NOTICES`.
    pub fn third_party_notices(&self) -> ThirdParty {
        if !self.crates.contains(THIRD_PARTY_CRATE) && !self.unicode_tables {
            ThirdParty::NotNeeded
        } else if self.unicode_notice && self.unicode_license {
            ThirdParty::Kept
        } else {
            ThirdParty::Missing
        }
    }

    /// The rule of `DISTRIBUTION_NOTICES`. The third-party items alone are
    /// not a sign of GEL RAM material.
    pub fn distribution(&self) -> Distribution {
        let gel_notice = self.license_file == LicenseFile::Exact
            || self.required_notice == Notice::Kept
            || self.header_files > 0
            || !self.crates.is_empty()
            || self.names_gel_ram;
        if !gel_notice {
            Distribution::NoGelNoticesFound
        } else if self.license_file == LicenseFile::Exact
            && self.required_notice == Notice::Kept
            && self.third_party_notices() != ThirdParty::Missing
            && self.header_altered() == 0
        {
            Distribution::ListedItemsPresent
        } else {
            Distribution::ListedItemsMissing
        }
    }

    /// The report lines of `gel-license check`, each ending in `\n`.
    pub fn lines(&self) -> String {
        let crates = if self.crates.is_empty() {
            "none".to_string()
        } else {
            self.crates.iter().cloned().collect::<Vec<_>>().join(",")
        };
        let detail = if self.license_line_endings_only {
            "LICENSE_DETAIL=line_endings\n"
        } else {
            ""
        };
        format!(
            "LICENSE_FILE={}\n{detail}REQUIRED_NOTICE={}\nTHIRD_PARTY_NOTICES={}\nGEL_HEADER_FILES={} HEADER_INTACT={} HEADER_ALTERED={}\nGEL_CRATES={crates}\nSCAN_FILES={} SKIPPED_SYMLINKS={} SKIPPED_OVERSIZE={}\nDISTRIBUTION_NOTICES={}\n{CHECK_NOTE}\nCONTACT={CONTACT}\nLEGAL_CERTIFICATION=NO\n",
            self.license_file.name(),
            self.required_notice.name(),
            self.third_party_notices().name(),
            self.header_files,
            self.header_intact,
            self.header_altered(),
            self.files,
            self.symlinks_skipped,
            self.oversize_skipped,
            self.distribution().name(),
        )
    }
}

/// The lines of `text` split at LF; a CR before the LF is not part of the line.
fn text_lines(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    text.split(|b| *b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
}

/// Whether `text` holds `line` as one whole line.
pub fn has_line(text: &[u8], line: &str) -> bool {
    text_lines(text).any(|l| l == line.as_bytes())
}

/// `text` with every CR LF turned into LF.
fn crlf_to_lf(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for (i, b) in text.iter().enumerate() {
        if *b != b'\r' || text.get(i + 1) != Some(&b'\n') {
            out.push(*b);
        }
    }
    out
}

/// `None` when neither of the first two lines of `start`, after an optional
/// UTF-8 byte order mark, looks like a source header line (the rule of the
/// source-header gate, `gel_core::source_header_like`); otherwise whether the
/// file starts with the exact source header, as the gate requires. A byte
/// order mark or CR LF line endings make a header altered.
pub fn header_state(start: &[u8]) -> Option<bool> {
    let body = start.strip_prefix(UTF8_BOM).unwrap_or(start);
    let mut lines = body.split(|b| *b == b'\n');
    let first = lines.next().unwrap_or_default();
    let second = lines.next().unwrap_or_default();
    if !source_header_like(first) && !source_header_like(second) {
        return None;
    }
    Some(start.starts_with(SOURCE_HEADER.as_bytes()))
}

fn is_gel_name(name: &str) -> bool {
    GEL_CRATE_NAMES.contains(&name)
}

fn is_dependency_table(table: &str) -> bool {
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .any(|kind| table == *kind || table.ends_with(&format!(".{kind}")))
}

/// The line up to a `#` that is outside a basic or literal string.
fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match (quote, c) {
            (Some('"'), '\\') if !escaped => {
                escaped = true;
                continue;
            }
            (Some(q), _) if c == q && !escaped => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, '#') => return &line[..i],
            _ => {}
        }
        escaped = false;
    }
    line
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    for q in ['"', '\''] {
        if let Some(inner) = s.strip_prefix(q).and_then(|t| t.strip_suffix(q)) {
            return inner;
        }
    }
    s
}

/// The first part of a dotted key, unquoted: `gel-core.workspace` and
/// `"gel-core".path` both give `gel-core`.
fn first_key(key: &str) -> &str {
    let key = key.trim();
    for q in ['"', '\''] {
        if let Some(rest) = key.strip_prefix(q) {
            return rest.split(q).next().unwrap_or_default();
        }
    }
    key.split('.').next().unwrap_or_default().trim()
}

/// The quoted string that follows `package =` in `value`, if any.
fn renamed_package(value: &str) -> Option<&str> {
    let mut rest = value;
    while let Some(at) = rest.find("package") {
        let before = rest[..at].chars().next_back();
        let after = rest[at + "package".len()..].trim_start();
        rest = &rest[at + "package".len()..];
        if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            continue;
        }
        let Some(after) = after.strip_prefix('=') else {
            continue;
        };
        let after = after.trim_start();
        for q in ['"', '\''] {
            if let Some(body) = after.strip_prefix(q) {
                return body.split(q).next();
            }
        }
    }
    None
}

/// Package and dependency names of [`GEL_CRATE_NAMES`] that a Cargo.toml
/// declares, read as text: `name` in `[package]`, keys of dependency tables
/// (including `target.*` and `workspace` ones, and dotted keys such as
/// `gel-core.workspace = true`), `[dependencies.NAME]` tables and
/// `package = "..."` renames.
pub fn cargo_gel_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut table = String::new();
    let add = |name: &str, names: &mut BTreeSet<String>| {
        if is_gel_name(name) {
            names.insert(name.to_string());
        }
    };
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let inner = line.trim_start_matches('[').trim_end_matches(']');
            table = inner.split('.').map(unquote).collect::<Vec<_>>().join(".");
            if let Some((parent, name)) = table.rsplit_once('.') {
                if is_dependency_table(parent) {
                    add(name, &mut names);
                }
            }
            continue;
        }
        let Some((raw_key, value)) = line.split_once('=') else {
            continue;
        };
        let key = unquote(raw_key);
        let entry_table = table
            .rsplit_once('.')
            .is_some_and(|(parent, _)| is_dependency_table(parent));
        if table == "package" && key == "name" {
            add(unquote(value), &mut names);
        }
        if is_dependency_table(&table) {
            add(first_key(raw_key), &mut names);
            if let Some(renamed) = renamed_package(value) {
                add(renamed, &mut names);
            }
        }
        if entry_table && key == "package" {
            add(unquote(value), &mut names);
        }
    }
    names
}

/// Up to `limit` bytes of a regular file and whether it holds more. The type
/// is checked before the file is opened, so a FIFO or device is never opened,
/// and again after.
fn read_bounded(path: &Path, limit: u64) -> std::io::Result<(Vec<u8>, bool)> {
    let not_regular = || std::io::Error::other("not a regular file");
    if !fs::metadata(path)?.is_file() {
        return Err(not_regular());
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(not_regular());
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    let over = bytes.len() as u64 > limit;
    bytes.truncate(limit as usize);
    Ok((bytes, over))
}

/// `gel-license check DIR` with the documented bounds.
pub fn check_dir(dir: &Path) -> Result<TreeReport, String> {
    check_dir_with(dir, Limits::default())
}

/// `gel-license check DIR` with explicit bounds. Reads only; follows no
/// symbolic link below `dir` and skips `.git` and `target` directories.
pub fn check_dir_with(dir: &Path, limits: Limits) -> Result<TreeReport, String> {
    let meta = fs::metadata(dir).map_err(|e| format!("cannot read the directory: {e}"))?;
    if !meta.is_dir() {
        return Err("not a directory".into());
    }
    let mut report = TreeReport::default();
    let mut entries = 0usize;
    let mut pending: Vec<(PathBuf, bool)> = vec![(dir.to_path_buf(), true)];
    // A path below `dir`, or `.` for `dir` itself: a refusal never names an empty path.
    let relative = |path: &Path| {
        let rel = path.strip_prefix(dir).unwrap_or(path).display().to_string();
        if rel.is_empty() {
            ".".to_string()
        } else {
            rel
        }
    };
    while let Some((current, top)) = pending.pop() {
        let listing = fs::read_dir(&current).map_err(|e| format!("{}: {e}", relative(&current)))?;
        for entry in listing {
            let entry = entry.map_err(|e| format!("{}: {e}", relative(&current)))?;
            entries += 1;
            if entries > limits.max_entries {
                return Err(format!(
                    "more than {} directory entries; check a smaller directory",
                    limits.max_entries
                ));
            }
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", relative(&path)))?;
            let name = entry.file_name();
            if kind.is_symlink() {
                report.symlinks_skipped += 1;
            } else if kind.is_dir() {
                if name != ".git" && name != "target" {
                    pending.push((path, false));
                }
            } else if kind.is_file() {
                report.files += 1;
                inspect_file(&path, &name.to_string_lossy(), top, limits, &mut report)
                    .map_err(|e| format!("{}: {e}", relative(&path)))?;
            }
        }
    }
    Ok(report)
}

fn inspect_file(
    path: &Path,
    name: &str,
    top: bool,
    limits: Limits,
    report: &mut TreeReport,
) -> std::io::Result<()> {
    if top && (name == "LICENSE" || name == "NOTICE") {
        // Room for a copy of the LICENSE with CR LF line endings.
        let limit = if name == "LICENSE" {
            limits.max_read.max(2 * LICENSE_TEXT.len() as u64)
        } else {
            limits.max_read
        };
        let (bytes, over) = read_bounded(path, limit)?;
        if name == "LICENSE" {
            if !over && digest(&bytes) == digest(LICENSE_TEXT) {
                report.license_file = LicenseFile::Exact;
            } else {
                report.license_file = LicenseFile::Changed;
                report.license_line_endings_only =
                    !over && crlf_to_lf(&bytes).as_slice() == LICENSE_TEXT;
            }
        }
        if over {
            report.oversize_skipped += 1;
            return Ok(());
        }
        if has_line(&bytes, REQUIRED_NOTICE) {
            report.required_notice = Notice::Kept;
        }
        if bytes.windows(GEL_NAME.len()).any(|w| w == GEL_NAME) {
            report.names_gel_ram = true;
        }
    } else if top && name == THIRD_PARTY_NOTICES_FILE {
        let (bytes, over) = read_bounded(path, limits.max_read)?;
        if over {
            report.oversize_skipped += 1;
        } else if has_line(&bytes, UNICODE_NOTICE_LINE) {
            report.unicode_notice = true;
        }
    } else if name == UNICODE_LICENSE_FILE {
        // Counted by name and type only; the file is not opened.
        report.unicode_license = true;
    } else if name.ends_with(".rs") {
        if name == UNICODE_TABLES_FILE {
            report.unicode_tables = true;
        }
        let (start, _) = read_bounded(path, HEADER_READ)?;
        if let Some(intact) = header_state(&start) {
            report.header_files += 1;
            report.header_intact += usize::from(intact);
        }
    } else if name == "Cargo.toml" {
        let (bytes, over) = read_bounded(path, limits.max_read)?;
        if over {
            report.oversize_skipped += 1;
        } else {
            report
                .crates
                .extend(cargo_gel_names(&String::from_utf8_lossy(&bytes)));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// use ANSWERS_FILE
// ---------------------------------------------------------------------------

/// The answer keys, in template order. The first [`MONETIZED`] are the items
/// of Monetized Use.
pub const KEYS: [&str; 7] = [
    "money",
    "customer_production",
    "hosted",
    "paid_work",
    "cost_reduction",
    "evaluation_only",
    "noncommercial_purpose",
];
/// How many of [`KEYS`], from the first, are items of Monetized Use.
pub const MONETIZED: usize = 5;

/// The question of each key in [`KEYS`]. Text in double quotes is quoted from
/// the LICENSE word for word.
pub const QUESTIONS: [&str; 7] = [
    "Is this \"use of the Software or a Modification, other than permitted Evaluation, that directly or indirectly receives, earns, enables, supports, protects, or is intended to produce money or other material financial benefit for You or another person\"? (section 2, \"Monetized Use\"). Its examples include \"sale or licensing of a product; paid subscriptions;\" and \"sponsorship tied to access or operation\", and section 2 adds: \"Donations, tips, crowdfunding, sponsorships, or similar payments received specifically because of a GEL RAM-based activity are Monetized Use when they fund, reward, or support that activity. An unrelated personal gift that is not connected to GEL RAM use is outside this definition.\"",
    "Does the use include \"customer-facing production systems\" or \"deployment in revenue-generating infrastructure\"? (examples of Monetized Use, section 2)",
    "Does the use include \"SaaS or hosted services\" or \"advertising-supported services\"? (examples of Monetized Use, section 2)",
    "Does the use include \"paid consulting or contract work\" (an example of Monetized Use, section 2), or \"billable work\" or \"generation of deliverables for a paying customer\", which section 2 excludes from Evaluation?",
    "Does the use include \"production use that materially reduces the operating cost of a commercial activity\"? (an example of Monetized Use, section 2)",
    "Is the use only \"testing, benchmarking, studying, prototyping, modifying, or integrating the Software or a Modification in a non-production environment solely to assess, learn about, validate, or decide whether to obtain commercial rights\"? (section 2, \"Evaluation\"). Section 2 adds: \"Evaluation does not include customer-facing deployment, production operation, paid service delivery, sale, subscription use, advertising-supported operation, billable work, use of GEL RAM to train or tune a production system, generation of deliverables for a paying customer, or use of the Software or a Modification to operate or materially improve a revenue-generating service or product.\"",
    "Is the use a \"use that does not generate, enable, support, or form part of a Monetized Use\"? (section 2, \"Noncommercial Purpose\"). Section 2 defines Monetized Use as use \"other than permitted Evaluation\".",
];

/// One answer: `yes`, `no` or `unsure`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Answer {
    Yes,
    #[default]
    No,
    Unsure,
}

impl Answer {
    /// The answer a value of an answers file names, if any.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "yes" => Some(Self::Yes),
            "no" => Some(Self::No),
            "unsure" => Some(Self::Unsure),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unsure => "unsure",
        }
    }
}

/// One complete set of answers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Answers {
    pub money: Answer,
    pub customer_production: Answer,
    pub hosted: Answer,
    pub paid_work: Answer,
    pub cost_reduction: Answer,
    pub evaluation_only: Answer,
    pub noncommercial_purpose: Answer,
}

impl Answers {
    /// Answers from values in [`KEYS`] order.
    pub fn from_values(v: [Answer; 7]) -> Self {
        Self {
            money: v[0],
            customer_production: v[1],
            hosted: v[2],
            paid_work: v[3],
            cost_reduction: v[4],
            evaluation_only: v[5],
            noncommercial_purpose: v[6],
        }
    }
    /// The values in [`KEYS`] order.
    pub fn values(&self) -> [Answer; 7] {
        [
            self.money,
            self.customer_production,
            self.hosted,
            self.paid_work,
            self.cost_reduction,
            self.evaluation_only,
            self.noncommercial_purpose,
        ]
    }
    /// Whether an item of Monetized Use is answered `yes`.
    pub fn monetized(&self) -> bool {
        self.values()[..MONETIZED].contains(&Answer::Yes)
    }
    /// Whether an item of Monetized Use and `evaluation_only` are both `yes`.
    pub fn conflicting(&self) -> bool {
        self.monetized() && self.evaluation_only == Answer::Yes
    }
    /// Whether `evaluation_only=yes` and `noncommercial_purpose=no`: permitted
    /// Evaluation is excluded from Monetized Use, so these answers conflict.
    pub fn evaluation_not_noncommercial(&self) -> bool {
        self.evaluation_only == Answer::Yes && self.noncommercial_purpose == Answer::No
    }
}

/// `USE_VERDICT=`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseVerdict {
    CommercialAgreementRequired,
    AskTheLicensor,
    Evaluation,
    Noncommercial,
}

/// The rule of `USE_VERDICT`, in this order:
/// 1. an item of Monetized Use answered `yes` needs a Commercial Agreement;
/// 2. otherwise any `unsure` answer: ask the Licensor;
/// 3. otherwise `evaluation_only=yes` with `noncommercial_purpose=no`: ask the
///    Licensor, because section 2 excludes permitted Evaluation from Monetized
///    Use and the two answers conflict;
/// 4. otherwise `noncommercial_purpose=no` needs a Commercial Agreement: by the
///    LICENSE definition, a use that is not a Noncommercial Purpose forms part
///    of a Monetized Use;
/// 5. otherwise `evaluation_only=yes` is Evaluation;
/// 6. otherwise (`noncommercial_purpose=yes`) the use is noncommercial.
pub fn use_verdict(a: &Answers) -> UseVerdict {
    if a.monetized() {
        UseVerdict::CommercialAgreementRequired
    } else if a.values().contains(&Answer::Unsure) || a.evaluation_not_noncommercial() {
        UseVerdict::AskTheLicensor
    } else if a.noncommercial_purpose == Answer::No {
        UseVerdict::CommercialAgreementRequired
    } else if a.evaluation_only == Answer::Yes {
        UseVerdict::Evaluation
    } else {
        UseVerdict::Noncommercial
    }
}

/// The `NOTE=` line printed when an item of Monetized Use and
/// `evaluation_only` are both answered `yes`.
pub const CONFLICT_NOTE: &str = "NOTE=evaluation_only=yes and an item of Monetized Use answered yes: a Monetized Use is not Evaluation; section 2 defines Monetized Use as use \"other than permitted Evaluation\"";
/// The `NOTE=` line printed when `evaluation_only=yes` and
/// `noncommercial_purpose=no` are answered together.
pub const EVALUATION_NOTE: &str = "NOTE=evaluation_only=yes and noncommercial_purpose=no: section 2 defines Monetized Use as use \"other than permitted Evaluation\", so these answers conflict";
/// The last `NOTE=` line of every `use` report.
pub const USE_NOTE: &str =
    "NOTE=answers are your own; this program reads the license, it does not interpret your facts";
/// Joins sentences of one section that are not next to each other in a quote.
pub const QUOTE_GAP: &str = " … ";

impl UseVerdict {
    pub fn name(self) -> &'static str {
        match self {
            Self::CommercialAgreementRequired => "COMMERCIAL_AGREEMENT_REQUIRED",
            Self::AskTheLicensor => "ASK_THE_LICENSOR",
            Self::Evaluation => "EVALUATION",
            Self::Noncommercial => "NONCOMMERCIAL",
        }
    }
    /// The LICENSE sections the verdict rests on: the definitions and the
    /// grant, rule or contact.
    pub fn sections(self) -> &'static str {
        match self {
            Self::CommercialAgreementRequired => "2,5",
            Self::AskTheLicensor => "2,19",
            Self::Evaluation => "2,4",
            Self::Noncommercial => "2,3",
        }
    }
    /// Sentences quoted word for word from the last section the verdict
    /// cites; [`QUOTE_GAP`] stands between sentences that are not adjacent.
    pub fn quote(self) -> &'static str {
        match self {
            Self::CommercialAgreementRequired => {
                "No Monetized Use is licensed by these public terms. Before beginning a Monetized Use, You must obtain a Commercial Agreement from the Licensor."
            }
            Self::AskTheLicensor => {
                "Commercial licensing and licensing questions: gelram.licensing@gmail.com"
            }
            Self::Evaluation => {
                "Evaluation may use ordinary rented or owned computing infrastructure, but the Software or a Modification may not be deployed as part of a paid plan, customer-facing production environment, revenue-generating service, or other Monetized Use under this section. Evaluation does not grant commercial rights. If Evaluation becomes a Monetized Use, commercial rights are required before that Monetized Use begins."
            }
            Self::Noncommercial => {
                "Subject to this license, the Licensor grants You a worldwide, non-exclusive, non-transferable, non-sublicensable, royalty-free copyright license to reproduce, run, study, modify, and share the Software for Noncommercial Purposes. … You may share the Software or a Modification with other people for Noncommercial Purposes, including private sharing, research collaboration, teaching, hobby work, community experimentation, and noncommercial redistribution of demonstration media, provided You preserve this license and the notices required by section 8."
            }
        }
    }
}

/// The report lines of `gel-license use` for `a`, each ending in `\n`.
pub fn use_report(a: &Answers) -> String {
    let verdict = use_verdict(a);
    let mut conflict = String::new();
    if a.conflicting() {
        conflict.push_str(CONFLICT_NOTE);
        conflict.push('\n');
    }
    // With a Monetized item answered yes, CONFLICT_NOTE already says why.
    if a.evaluation_not_noncommercial() && !a.monetized() {
        conflict.push_str(EVALUATION_NOTE);
        conflict.push('\n');
    }
    format!(
        "USE_VERDICT={}\nLICENSE_SECTIONS={}\nLICENSE_QUOTE={}\n{conflict}LEGAL_CERTIFICATION=NO\nCONTACT={CONTACT}\n{USE_NOTE}\n",
        verdict.name(),
        verdict.sections(),
        verdict.quote()
    )
}

/// Parses `key=yes|no|unsure` lines. A leading byte order mark, blank lines
/// and lines starting with `#` are ignored; spaces around keys and values and
/// a CR before LF are allowed. Every key of [`KEYS`] must appear exactly once;
/// any other line is refused. Error texts are not escaped: the program
/// escapes what it prints, once.
pub fn parse_answers(text: &str) -> Result<Answers, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut values: [Option<Answer>; 7] = [None; 7];
    for (index, raw) in text.split('\n').enumerate() {
        let number = index + 1;
        let line = raw.strip_suffix('\r').unwrap_or(raw).trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "line {number}: expected key=yes, key=no or key=unsure"
            ));
        };
        let key = key.trim();
        let Some(slot) = KEYS.iter().position(|k| *k == key) else {
            return Err(format!("line {number}: unknown key {key}"));
        };
        if values[slot].is_some() {
            return Err(format!("line {number}: duplicate key {key}"));
        }
        let Some(answer) = Answer::parse(value.trim()) else {
            return Err(format!("line {number}: {key} must be yes, no or unsure"));
        };
        values[slot] = Some(answer);
    }
    let missing: Vec<&str> = KEYS
        .iter()
        .zip(values)
        .filter_map(|(key, value)| value.is_none().then_some(*key))
        .collect();
    if !missing.is_empty() {
        return Err(format!("missing keys: {}", missing.join(", ")));
    }
    Ok(Answers::from_values(values.map(Option::unwrap_or_default)))
}

/// Reads and parses an answers file of at most [`MAX_ANSWERS`] bytes.
pub fn read_answers(path: &Path) -> Result<Answers, String> {
    let (bytes, over) = read_bounded(path, MAX_ANSWERS)
        .map_err(|e| format!("cannot read the answers file: {e}"))?;
    if over {
        return Err(format!("answers file larger than {MAX_ANSWERS} bytes"));
    }
    let text = String::from_utf8(bytes).map_err(|_| "answers file is not UTF-8".to_string())?;
    parse_answers(&text)
}

/// `text` as `#` comment lines of at most `width` characters after the prefix.
fn wrap(text: &str, prefix: &str, width: usize) -> String {
    let mut out = String::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            out.push_str(prefix);
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push_str(prefix);
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// The answers template of `gel-license use --template`: each key with its
/// question as comments and an empty value, which `use` refuses until it is
/// answered.
pub fn template() -> String {
    let mut out = format!(
        "# Your own answers about one use of GEL RAM ({LICENSE_ID}).\n# Answer every key with yes, no or unsure, then run: gel-license use THIS_FILE\n# Any unsure answer leads to ASK_THE_LICENSOR unless an item of Monetized\n# Use is yes. The questions quote section 2 of the LICENSE; the LICENSE\n# text controls. Your answers are your own; this program does not interpret\n# your facts and is not legal advice.\n# Licensing questions: {CONTACT}\n"
    );
    for (key, question) in KEYS.iter().zip(QUESTIONS) {
        out.push('\n');
        out.push_str(&wrap(&format!("{key}: {question}"), "# ", 76));
        out.push_str(key);
        out.push_str("=\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Answer::{No, Unsure, Yes};
    use super::*;
    use gel_core::{tool_identity, tool_license, LICENSE_NAME};
    use std::sync::atomic::{AtomicU64, Ordering};

    const LICENSE_STR: &str = include_str!("../../../LICENSE");
    const NOTICE_TEXT: &[u8] = include_bytes!("../../../NOTICE");
    const THIRD_PARTY_TEXT: &[u8] = include_bytes!("../../../THIRD-PARTY-NOTICES.md");
    const PINS: &str = include_str!("../../../docs/LICENSE-PINS.md");
    const WORKSPACE_TOML: &str = include_str!("../../../Cargo.toml");

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "gel-license-check-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn put(&self, rel: &str, bytes: &[u8]) {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        fn check(&self) -> TreeReport {
            check_dir(&self.0).unwrap()
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn header(body: &str) -> String {
        format!("{HEADER_LINE_1}\n{HEADER_LINE_2}\n{body}")
    }

    fn report(license: LicenseFile, notice: Notice, files: usize, intact: usize) -> TreeReport {
        TreeReport {
            license_file: license,
            required_notice: notice,
            header_files: files,
            header_intact: intact,
            ..TreeReport::default()
        }
    }

    /// The fixed tail of every `check` report.
    fn tail() -> String {
        format!("{CHECK_NOTE}\nCONTACT={CONTACT}\nLEGAL_CERTIFICATION=NO\n")
    }

    // --- constants tied to the files ---------------------------------------

    #[test]
    fn constants_match_the_license_and_notice_files() {
        assert_eq!(LICENSE_STR.lines().next(), Some(LICENSE_NAME));
        assert!(has_line(LICENSE_TEXT, REQUIRED_NOTICE));
        assert!(has_line(NOTICE_TEXT, REQUIRED_NOTICE));
        assert_eq!(
            HEADER_LINE_1,
            format!("// GEL RAM — {LICENSE_NAME}; see LICENSE and NOTICE.")
        );
        assert_eq!(HEADER_LINE_2, format!("// {REQUIRED_NOTICE}"));
        assert_eq!(SOURCE_HEADER, format!("{HEADER_LINE_1}\n{HEADER_LINE_2}\n"));
        assert!(LICENSE_STR.contains(&format!(
            "Contact\n\nCommercial licensing and licensing questions: {CONTACT}"
        )));
    }

    #[test]
    fn the_gel_crate_names_are_the_workspace_members() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let members = WORKSPACE_TOML
            .split("members = [")
            .nth(1)
            .and_then(|rest| rest.split(']').next())
            .unwrap();
        let mut names: Vec<String> = members
            .split(',')
            .map(|m| m.trim().trim_matches('"'))
            .filter(|m| !m.is_empty() && *m != "xtask")
            .map(|m| {
                let toml = fs::read_to_string(root.join(m).join("Cargo.toml")).unwrap();
                let line = toml
                    .lines()
                    .find(|l| l.starts_with("name = "))
                    .unwrap_or_else(|| panic!("{m}"));
                unquote(&line["name = ".len()..]).to_string()
            })
            .collect();
        names.sort();
        assert_eq!(names, GEL_CRATE_NAMES);
    }

    #[test]
    fn the_third_party_items_are_those_of_this_workspace() {
        assert!(has_line(THIRD_PARTY_TEXT, UNICODE_NOTICE_LINE));
        assert!(UNICODE_NOTICE_LINE.contains("Unicode License v3"));
        assert!(GEL_CRATE_NAMES.contains(&THIRD_PARTY_CRATE));
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../gel-source");
        let toml = fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(cargo_gel_names(&toml).contains(THIRD_PARTY_CRATE));
        assert!(crate_dir.join(UNICODE_LICENSE_FILE).is_file());
        assert!(crate_dir.join("src").join(UNICODE_TABLES_FILE).is_file());
        let text = String::from_utf8_lossy(THIRD_PARTY_TEXT);
        assert!(text.contains(&format!("crates/gel-source/{UNICODE_LICENSE_FILE}")));
        assert!(text.contains(&format!("crates/gel-source/src/{UNICODE_TABLES_FILE}")));
        // The guide names the same line.
        let guide = include_str!("../../../docs/LICENSE-DECISION.md");
        assert!(guide.contains(&format!("`{UNICODE_NOTICE_LINE}`")));
    }

    #[test]
    fn the_pinned_license_hash_is_the_listed_one() {
        assert_eq!(gel_source::hex(&digest(LICENSE_TEXT)), LICENSE_SHA256);
        let row = PINS
            .lines()
            .find(|l| l.contains("[LICENSE](../LICENSE)"))
            .unwrap();
        assert!(row.contains(LICENSE_SHA256), "{row}");
    }

    #[test]
    fn identity_and_license_texts_are_exact() {
        assert_eq!(
            tool_identity("gel-license", "0.7.0"),
            "GEL_TOOL=gel-license VERSION=0.7.0 LICENSE=GEL-RAM-NCRL-1.0"
        );
        assert_eq!(
            tool_license("gel-license", "0.7.0"),
            "GEL_TOOL=gel-license VERSION=0.7.0 LICENSE=GEL-RAM-NCRL-1.0\n\
             GEL RAM Noncommercial Reciprocal License 1.0\n\
             Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)\n\
             Noncommercial use, modification, sharing and Evaluation are permitted on the terms of the LICENSE file. Monetized Use requires a separate written Commercial Agreement: gelram.licensing@gmail.com\n\
             Third-party material in this program, including the Unicode data compiled into it, remains under its own license: see THIRD-PARTY-NOTICES.md.\n"
        );
    }

    // --- DISTRIBUTION_NOTICES rule table -----------------------------------

    #[test]
    fn row_no_gel_notice_is_not_found() {
        assert_eq!(
            TreeReport::default().distribution(),
            Distribution::NoGelNoticesFound
        );
        // A LICENSE of another project is CHANGED but is not a sign by itself.
        let other = report(LicenseFile::Changed, Notice::Missing, 0, 0);
        assert_eq!(other.distribution(), Distribution::NoGelNoticesFound);
    }

    #[test]
    fn row_exact_license_kept_notice_no_altered_header_is_present() {
        for (files, intact) in [(0, 0), (3, 3)] {
            let r = report(LicenseFile::Exact, Notice::Kept, files, intact);
            assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        }
    }

    #[test]
    fn row_changed_or_missing_license_is_missing() {
        for license in [LicenseFile::Changed, LicenseFile::Missing] {
            let r = report(license, Notice::Kept, 0, 0);
            assert_eq!(
                r.distribution(),
                Distribution::ListedItemsMissing,
                "{license:?}"
            );
        }
    }

    #[test]
    fn row_missing_required_notice_is_missing() {
        let mut r = report(LicenseFile::Changed, Notice::Missing, 0, 0);
        r.names_gel_ram = true;
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        // Not reachable from a real tree (the exact LICENSE holds the line), but the rule says so.
        let r = report(LicenseFile::Exact, Notice::Missing, 0, 0);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
    }

    #[test]
    fn row_any_altered_header_is_missing() {
        let r = report(LicenseFile::Exact, Notice::Kept, 3, 2);
        assert_eq!(r.header_altered(), 1);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
    }

    #[test]
    fn row_each_sign_alone_marks_a_gel_notice() {
        // The third-party items are not signs (row_the_unicode_tables_alone_are_not_a_gel_notice).
        let signs: [fn(&mut TreeReport); 5] = [
            |r| r.license_file = LicenseFile::Exact,
            |r| r.required_notice = Notice::Kept,
            |r| {
                r.header_files = 1;
                r.header_intact = 1
            },
            |r| {
                r.crates.insert("gel-core".into());
            },
            |r| r.names_gel_ram = true,
        ];
        for (i, sign) in signs.iter().enumerate() {
            let mut r = TreeReport::default();
            sign(&mut r);
            assert_ne!(
                r.distribution(),
                Distribution::NoGelNoticesFound,
                "sign {i}"
            );
        }
    }

    /// A report with an exact LICENSE and a kept Required Notice.
    fn exact() -> TreeReport {
        report(LicenseFile::Exact, Notice::Kept, 0, 0)
    }

    #[test]
    fn row_third_party_notices_are_not_needed_without_either_signal() {
        let mut r = exact();
        r.crates.insert("gel-core".into());
        r.unicode_notice = true;
        r.unicode_license = true;
        assert_eq!(r.third_party_notices(), ThirdParty::NotNeeded);
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        r.unicode_notice = false;
        r.unicode_license = false;
        assert_eq!(r.third_party_notices(), ThirdParty::NotNeeded);
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
    }

    #[test]
    fn row_third_party_notices_need_both_items_for_either_signal() {
        let signals: [fn(&mut TreeReport); 2] = [
            |r| {
                r.crates.insert(THIRD_PARTY_CRATE.into());
            },
            |r| r.unicode_tables = true,
        ];
        for (i, signal) in signals.iter().enumerate() {
            for (notice, license, want) in [
                (true, true, ThirdParty::Kept),
                (true, false, ThirdParty::Missing),
                (false, true, ThirdParty::Missing),
                (false, false, ThirdParty::Missing),
            ] {
                let mut r = exact();
                signal(&mut r);
                r.unicode_notice = notice;
                r.unicode_license = license;
                let case = format!("signal {i} notice {notice} license {license}");
                assert_eq!(r.third_party_notices(), want, "{case}");
                let distribution = if want == ThirdParty::Kept {
                    Distribution::ListedItemsPresent
                } else {
                    Distribution::ListedItemsMissing
                };
                assert_eq!(r.distribution(), distribution, "{case}");
            }
        }
    }

    #[test]
    fn row_the_unicode_tables_alone_are_not_a_gel_notice() {
        let mut r = TreeReport {
            unicode_tables: true,
            ..TreeReport::default()
        };
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
        r.unicode_notice = true;
        r.unicode_license = true;
        assert_eq!(r.third_party_notices(), ThirdParty::Kept);
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    #[test]
    fn exit_codes_follow_the_verdict() {
        assert_eq!(Distribution::ListedItemsPresent.exit_code(), 0);
        assert_eq!(Distribution::ListedItemsMissing.exit_code(), 3);
        assert_eq!(Distribution::NoGelNoticesFound.exit_code(), 4);
    }

    // --- line and header readers -------------------------------------------

    #[test]
    fn a_notice_line_must_be_whole() {
        let n = REQUIRED_NOTICE;
        assert!(has_line(format!("a\n{n}\nb").as_bytes(), n));
        assert!(has_line(format!("{n}\r\n").as_bytes(), n));
        assert!(has_line(n.as_bytes(), n));
        assert!(!has_line(format!("x{n}").as_bytes(), n));
        assert!(!has_line(format!("{n} extra").as_bytes(), n));
        assert!(!has_line(n.replace("2026", "2025").as_bytes(), n));
    }

    #[test]
    fn header_states() {
        assert_eq!(
            header_state(header("fn main() {}\n").as_bytes()),
            Some(true)
        );
        assert_eq!(header_state(b"fn main() {}\n"), None);
        assert_eq!(header_state(b""), None);
        assert_eq!(header_state(b"//! GEL RAM crate docs.\n"), None);
        assert_eq!(
            header_state(format!("{HEADER_LINE_1}\n").as_bytes()),
            Some(false)
        );
        // Both lines without the closing line feed: altered, as in the gate.
        assert_eq!(
            header_state(format!("{HEADER_LINE_1}\n{HEADER_LINE_2}").as_bytes()),
            Some(false)
        );
        assert_eq!(header_state(HEADER_LINE_1.as_bytes()), Some(false));
        let altered_first = header("").replacen("see LICENSE", "see LICENCE", 1);
        assert_eq!(header_state(altered_first.as_bytes()), Some(false));
        let altered_second = header("").replacen("2026", "2027", 1);
        assert_eq!(header_state(altered_second.as_bytes()), Some(false));
        // The Required Notice line moved below the first line is altered too.
        let moved = format!("{HEADER_LINE_1}\n\n{HEADER_LINE_2}\n");
        assert_eq!(header_state(moved.as_bytes()), Some(false));
    }

    #[test]
    fn a_header_is_found_on_either_line_with_the_gate_rule() {
        // Line 1 no longer starts with "// GEL RAM —", line 2 is exact.
        for first in ["// GEL  RAM — tweaked", "//GEL RAM", "// Some other line"] {
            let text = format!("{first}\n{HEADER_LINE_2}\nfn f() {{}}\n");
            assert_eq!(header_state(text.as_bytes()), Some(false), "{first}");
        }
        // Only line 2 looks like a header line.
        assert_eq!(
            header_state(b"fn f() {}\n// Required Notice: someone else\n"),
            Some(false)
        );
    }

    #[test]
    fn a_byte_order_mark_or_crlf_makes_a_header_altered() {
        let bom = [UTF8_BOM, header("fn f() {}\n").as_bytes()].concat();
        assert_eq!(header_state(&bom), Some(false));
        let bom_changed = [UTF8_BOM, header("").replacen("2026", "2027", 1).as_bytes()].concat();
        assert_eq!(header_state(&bom_changed), Some(false));
        // A byte order mark in front of an ordinary file is not a header file.
        assert_eq!(header_state(&[UTF8_BOM, b"fn f() {}\n"].concat()), None);
        let crlf = header("fn f() {}\n").replace('\n', "\r\n");
        assert_eq!(header_state(crlf.as_bytes()), Some(false));
    }

    #[test]
    fn cargo_names_are_read_from_every_declaration_form() {
        let toml = r#"
[package]
name = "gel-store" # a comment
version = "1.0.0"

[dependencies]
gel-core = { path = "../gel-core" }
"gel-orb" = "1"
renamed = { package = "gel-structural", path = "../s" }
serde = "1"
# gel-cli = "1"
note = { version = "1", features = ["x#gel-bench"] }
gel-phase-quad.workspace = true
"gel-physics".path = "../p"
gel-bench . version = "1"

[dev-dependencies.gel-reader]
path = "../r"

[dependencies.alias]
package = "gel-history"

[target.'cfg(unix)'.dependencies]
gel-kernel = "1"

[workspace.dependencies]
gel-source = { path = "crates/gel-source" }
gel-live-lab.path = "crates/gel-live-lab"

[features]
gel-cli = []

[workspace]
members = [
  "crates/gel-member",
]
"#;
        let names: Vec<String> = cargo_gel_names(toml).into_iter().collect();
        assert_eq!(
            names,
            [
                "gel-bench",
                "gel-core",
                "gel-history",
                "gel-kernel",
                "gel-live-lab",
                "gel-orb",
                "gel-phase-quad",
                "gel-physics",
                "gel-reader",
                "gel-source",
                "gel-store",
                "gel-structural",
            ]
        );
        assert!(cargo_gel_names("[package]\nname = \"gel\"\n").is_empty());
        assert!(cargo_gel_names("[package]\nname = \"angel-x\"\n").is_empty());
    }

    #[test]
    fn other_crates_named_gel_something_are_not_gel_crates() {
        let toml = "[package]\nname = \"gel-thing\"\n\n[dependencies]\ngel-tokio = \"0.10\"\ngel-protocol.workspace = true\ngel-core-extra = \"1\"\n";
        assert!(cargo_gel_names(toml).is_empty());
        let t = Scratch::new();
        t.put("LICENSE", b"MIT License\n");
        t.put("Cargo.toml", toml.as_bytes());
        let r = t.check();
        assert!(r.crates.is_empty());
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    // --- directory scans ---------------------------------------------------

    #[test]
    fn null_world_an_unrelated_project_has_no_gel_notices() {
        let t = Scratch::new();
        t.put(
            "LICENSE",
            b"MIT License\n\nCopyright (c) 2026 Example Authors\n",
        );
        t.put("README.md", b"# hello\n");
        t.put(
            "Cargo.toml",
            b"[package]\nname = \"hello\"\n\n[dependencies]\nserde = \"1\"\n",
        );
        t.put("src/main.rs", b"// Example Authors\nfn main() {}\n");
        let r = t.check();
        assert_eq!(r.license_file, LicenseFile::Changed);
        assert!(!r.license_line_endings_only);
        assert_eq!(r.required_notice, Notice::Missing);
        assert_eq!((r.header_files, r.header_altered()), (0, 0));
        assert!(r.crates.is_empty());
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
        assert_eq!(
            r.lines(),
            format!("LICENSE_FILE=CHANGED\nREQUIRED_NOTICE=MISSING\nTHIRD_PARTY_NOTICES=NOT_NEEDED\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=none\nSCAN_FILES=4 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=NO_GEL_NOTICES_FOUND\n{}", tail())
        );
        let empty = Scratch::new();
        assert_eq!(empty.check().license_file, LicenseFile::Missing);
        assert_eq!(
            empty.check().distribution(),
            Distribution::NoGelNoticesFound
        );
    }

    #[test]
    fn an_exact_license_copy_is_exact_and_present() {
        let t = Scratch::new();
        t.put("LICENSE", LICENSE_TEXT);
        t.put("NOTICE", NOTICE_TEXT);
        t.put("crates/a/Cargo.toml", b"[package]\nname = \"gel-core\"\n");
        t.put("crates/a/src/lib.rs", header("pub fn f() {}\n").as_bytes());
        let r = t.check();
        assert_eq!(r.license_file, LicenseFile::Exact);
        assert_eq!(r.required_notice, Notice::Kept);
        assert_eq!((r.header_files, r.header_intact), (1, 1));
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        assert!(r.lines().contains("\nGEL_CRATES=gel-core\n"));
        assert!(r.lines().ends_with(&format!(
            "\nDISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT\n{}",
            tail()
        )));
        assert!(!r.lines().contains("LICENSE_DETAIL"));
    }

    #[test]
    fn a_license_with_one_changed_byte_is_changed() {
        for at in [0, LICENSE_TEXT.len() / 2, LICENSE_TEXT.len() - 1] {
            let t = Scratch::new();
            let mut bytes = LICENSE_TEXT.to_vec();
            bytes[at] ^= 0x01;
            t.put("LICENSE", &bytes);
            t.put("NOTICE", NOTICE_TEXT);
            let r = t.check();
            assert_eq!(r.license_file, LicenseFile::Changed, "byte {at}");
            assert!(!r.license_line_endings_only, "byte {at}");
            assert_eq!(r.required_notice, Notice::Kept);
            assert_eq!(
                r.distribution(),
                Distribution::ListedItemsMissing,
                "byte {at}"
            );
        }
        // One byte more or one less is changed too.
        for bytes in [
            [LICENSE_TEXT, b"\n"].concat(),
            LICENSE_TEXT[..LICENSE_TEXT.len() - 1].to_vec(),
        ] {
            let t = Scratch::new();
            t.put("LICENSE", &bytes);
            assert_eq!(t.check().license_file, LicenseFile::Changed);
            assert_eq!(t.check().distribution(), Distribution::ListedItemsMissing);
        }
    }

    #[test]
    fn a_license_with_crlf_line_endings_is_changed_with_a_detail_line() {
        let crlf = crlf_from(LICENSE_TEXT);
        assert_ne!(crlf.as_slice(), LICENSE_TEXT);
        let t = Scratch::new();
        t.put("LICENSE", &crlf);
        let r = t.check();
        assert_eq!(r.license_file, LicenseFile::Changed);
        assert!(r.license_line_endings_only);
        assert_eq!(r.required_notice, Notice::Kept);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        assert!(r.lines().starts_with(
            "LICENSE_FILE=CHANGED\nLICENSE_DETAIL=line_endings\nREQUIRED_NOTICE=KEPT\n"
        ));
        // CR LF and one more changed byte: no detail line.
        let mut other = crlf.clone();
        other[10] ^= 0x01;
        t.put("LICENSE", &other);
        assert!(!t.check().license_line_endings_only);
    }

    fn crlf_from(text: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for b in text {
            if *b == b'\n' {
                out.push(b'\r');
            }
            out.push(*b);
        }
        out
    }

    #[test]
    fn a_notice_without_the_required_notice_line_is_missing() {
        let without: Vec<u8> = text_lines(NOTICE_TEXT)
            .filter(|l| *l != REQUIRED_NOTICE.as_bytes())
            .collect::<Vec<_>>()
            .join(&b'\n');
        assert_ne!(without, NOTICE_TEXT);
        let t = Scratch::new();
        t.put("NOTICE", &without);
        let r = t.check();
        assert_eq!(r.license_file, LicenseFile::Missing);
        assert_eq!(r.required_notice, Notice::Missing);
        assert!(r.names_gel_ram);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        // The exact LICENSE carries the same line, so the line is still received.
        t.put("LICENSE", LICENSE_TEXT);
        let r = t.check();
        assert_eq!(r.required_notice, Notice::Kept);
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        // A notice line below the top level does not count.
        let deep = Scratch::new();
        deep.put("sub/NOTICE", NOTICE_TEXT);
        deep.put("sub/LICENSE", LICENSE_TEXT);
        let r = deep.check();
        assert_eq!(
            (r.license_file, r.required_notice),
            (LicenseFile::Missing, Notice::Missing)
        );
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    #[test]
    fn intact_and_altered_headers_are_counted() {
        let t = Scratch::new();
        t.put("LICENSE", LICENSE_TEXT);
        t.put("src/intact.rs", header("fn a() {}\n").as_bytes());
        t.put("src/plain.rs", b"fn b() {}\n");
        t.put("src/deep/also_intact.rs", header("").as_bytes());
        let r = t.check();
        assert_eq!(
            (r.header_files, r.header_intact, r.header_altered()),
            (2, 2, 0)
        );
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        for altered in [
            header("fn c() {}\n").replacen("License 1.0", "License 2.0", 1),
            header("fn c() {}\n").replacen("(C) 2026 RR", "(C) 2026 XX", 1),
            format!("{HEADER_LINE_1}\nfn c() {{}}\n"),
            format!("// GEL RAM — something else\n{HEADER_LINE_2}\n"),
            format!("// GEL  RAM — tweaked\n{HEADER_LINE_2}\n"),
            format!("\u{feff}{}", header("fn c() {}\n")),
            header("fn c() {}\n").replace('\n', "\r\n"),
        ] {
            t.put("src/altered.rs", altered.as_bytes());
            let r = t.check();
            assert_eq!(
                (r.header_files, r.header_intact, r.header_altered()),
                (3, 2, 1),
                "{altered:?}"
            );
            assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        }
    }

    #[test]
    fn git_and_target_directories_are_skipped() {
        let t = Scratch::new();
        t.put(".git/x.rs", header("").as_bytes());
        t.put("target/debug/y.rs", header("").as_bytes());
        t.put("crates/target/z.rs", header("").as_bytes());
        t.put("target/Cargo.toml", b"[package]\nname = \"gel-core\"\n");
        let r = t.check();
        assert_eq!(r.header_files, 0);
        assert!(r.crates.is_empty());
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_skipped_not_followed() {
        use std::os::unix::fs::symlink;
        let outside = Scratch::new();
        outside.put("LICENSE", LICENSE_TEXT);
        outside.put("NOTICE", NOTICE_TEXT);
        outside.put("h.rs", header("").as_bytes());
        outside.put("d/Cargo.toml", b"[package]\nname = \"gel-core\"\n");
        let t = Scratch::new();
        symlink(outside.0.join("LICENSE"), t.0.join("LICENSE")).unwrap();
        symlink(outside.0.join("NOTICE"), t.0.join("NOTICE")).unwrap();
        symlink(outside.0.join("h.rs"), t.0.join("h.rs")).unwrap();
        symlink(outside.0.join("d"), t.0.join("d")).unwrap();
        let r = t.check();
        assert_eq!(r.symlinks_skipped, 4);
        assert_eq!(r.files, 0);
        assert_eq!(r.license_file, LicenseFile::Missing);
        assert_eq!(r.required_notice, Notice::Missing);
        assert_eq!(r.header_files, 0);
        assert!(r.crates.is_empty());
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    // --- third-party notices ----------------------------------------------

    const TABLES: &[u8] =
        b"// Generated from the Unicode Character Database 16.0.0.\npub static T: [u8; 0] = [];\n";
    const SOURCE_TOML: &[u8] = b"[package]\nname = \"gel-source\"\n";

    /// The reviewer's tree t2: LICENSE, NOTICE and the gel-source crate with
    /// its Unicode tables, without THIRD-PARTY-NOTICES.md or LICENSE-UNICODE.txt.
    fn tree_t2() -> Scratch {
        let t = Scratch::new();
        t.put("LICENSE", LICENSE_TEXT);
        t.put("NOTICE", NOTICE_TEXT);
        t.put("crates/gel-source/Cargo.toml", SOURCE_TOML);
        t.put("crates/gel-source/src/nfc_tables.rs", TABLES);
        t
    }

    #[test]
    fn unicode_tables_without_third_party_notices_miss_listed_items() {
        let t = tree_t2();
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        assert_eq!(
            r.lines(),
            format!("LICENSE_FILE=EXACT\nREQUIRED_NOTICE=KEPT\nTHIRD_PARTY_NOTICES=MISSING\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=gel-source\nSCAN_FILES=4 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=LISTED_ITEMS_MISSING\n{}", tail())
        );
        // Each signal alone makes the notices needed.
        for (rel, bytes) in [
            ("crates/gel-source/src/nfc_tables.rs", TABLES),
            (
                "crates/x/Cargo.toml",
                b"[dependencies]\ngel-source.workspace = true\n",
            ),
        ] {
            let one = Scratch::new();
            one.put("LICENSE", LICENSE_TEXT);
            one.put(rel, bytes);
            let r = one.check();
            assert_eq!(r.third_party_notices(), ThirdParty::Missing, "{rel}");
            assert_eq!(r.distribution(), Distribution::ListedItemsMissing, "{rel}");
        }
    }

    #[test]
    fn third_party_notices_and_the_unicode_license_are_kept() {
        let t = tree_t2();
        t.put("THIRD-PARTY-NOTICES.md", THIRD_PARTY_TEXT);
        t.put(
            "crates/gel-source/LICENSE-UNICODE.txt",
            b"UNICODE LICENSE V3\n",
        );
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Kept);
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        assert_eq!(
            r.lines(),
            format!("LICENSE_FILE=EXACT\nREQUIRED_NOTICE=KEPT\nTHIRD_PARTY_NOTICES=KEPT\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=gel-source\nSCAN_FILES=6 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT\n{}", tail())
        );
        // Without LICENSE-UNICODE.txt the notices file alone is not enough.
        fs::remove_file(t.0.join("crates/gel-source/LICENSE-UNICODE.txt")).unwrap();
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        // Nor the license text alone.
        t.put("LICENSE-UNICODE.txt", b"UNICODE LICENSE V3\n");
        fs::remove_file(t.0.join("THIRD-PARTY-NOTICES.md")).unwrap();
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
    }

    #[test]
    fn third_party_notices_without_the_unicode_line_are_missing() {
        let without: Vec<u8> = text_lines(THIRD_PARTY_TEXT)
            .filter(|l| *l != UNICODE_NOTICE_LINE.as_bytes())
            .collect::<Vec<_>>()
            .join(&b'\n');
        assert_ne!(without, THIRD_PARTY_TEXT);
        let t = tree_t2();
        t.put("LICENSE-UNICODE.txt", b"UNICODE LICENSE V3\n");
        t.put("THIRD-PARTY-NOTICES.md", &without);
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        // The line must be whole; a CR before the LF is allowed.
        let longer = format!("# Notices\n{UNICODE_NOTICE_LINE} and more\n");
        t.put("THIRD-PARTY-NOTICES.md", longer.as_bytes());
        assert_eq!(t.check().third_party_notices(), ThirdParty::Missing);
        let crlf = format!("# Notices\r\n{UNICODE_NOTICE_LINE}\r\n");
        t.put("THIRD-PARTY-NOTICES.md", crlf.as_bytes());
        assert_eq!(t.check().third_party_notices(), ThirdParty::Kept);
        // Below the top level the notices file does not count.
        fs::remove_file(t.0.join("THIRD-PARTY-NOTICES.md")).unwrap();
        t.put("docs/THIRD-PARTY-NOTICES.md", THIRD_PARTY_TEXT);
        assert_eq!(t.check().third_party_notices(), ThirdParty::Missing);
        // A notices file over the read cap is skipped, counted and not kept.
        t.put("THIRD-PARTY-NOTICES.md", THIRD_PARTY_TEXT);
        let small = Limits {
            max_entries: MAX_ENTRIES,
            max_read: 100,
        };
        let r = check_dir_with(&t.0, small).unwrap();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        // NOTICE and the top-level THIRD-PARTY-NOTICES.md; the copy under docs is not read.
        assert_eq!(r.oversize_skipped, 2);
    }

    #[cfg(unix)]
    #[test]
    fn a_unicode_license_link_or_directory_is_not_counted() {
        use std::os::unix::fs::symlink;
        let outside = Scratch::new();
        outside.put("LICENSE-UNICODE.txt", b"UNICODE LICENSE V3\n");
        outside.put("THIRD-PARTY-NOTICES.md", THIRD_PARTY_TEXT);
        let t = tree_t2();
        t.put("THIRD-PARTY-NOTICES.md", THIRD_PARTY_TEXT);
        symlink(
            outside.0.join("LICENSE-UNICODE.txt"),
            t.0.join("crates/gel-source/LICENSE-UNICODE.txt"),
        )
        .unwrap();
        fs::create_dir(t.0.join("LICENSE-UNICODE.txt")).unwrap();
        let r = t.check();
        assert_eq!(r.symlinks_skipped, 1);
        assert!(!r.unicode_license);
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::ListedItemsMissing);
        // A linked THIRD-PARTY-NOTICES.md or nfc_tables.rs is not read either.
        let linked = Scratch::new();
        linked.put("LICENSE", LICENSE_TEXT);
        linked.put("LICENSE-UNICODE.txt", b"UNICODE LICENSE V3\n");
        symlink(
            outside.0.join("THIRD-PARTY-NOTICES.md"),
            linked.0.join("THIRD-PARTY-NOTICES.md"),
        )
        .unwrap();
        symlink(
            t.0.join("crates/gel-source/src/nfc_tables.rs"),
            linked.0.join("nfc_tables.rs"),
        )
        .unwrap();
        let r = linked.check();
        assert!(!r.unicode_notice && !r.unicode_tables);
        assert_eq!(r.third_party_notices(), ThirdParty::NotNeeded);
    }

    #[test]
    fn an_unrelated_tree_does_not_need_the_third_party_notices() {
        let t = Scratch::new();
        t.put("LICENSE", b"MIT License\n");
        t.put("THIRD-PARTY-NOTICES.md", b"# Notices\n");
        t.put("LICENSE-UNICODE.txt", b"UNICODE LICENSE V3\n");
        t.put("Cargo.toml", b"[package]\nname = \"gel-sourcery\"\n\n[dependencies]\nunicode-normalization = \"0.1\"\n");
        t.put("src/tables.rs", TABLES);
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::NotNeeded);
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
        assert_eq!(
            r.lines(),
            format!("LICENSE_FILE=CHANGED\nREQUIRED_NOTICE=MISSING\nTHIRD_PARTY_NOTICES=NOT_NEEDED\nGEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0\nGEL_CRATES=none\nSCAN_FILES=5 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES=NO_GEL_NOTICES_FOUND\n{}", tail())
        );
        // A file named nfc_tables.rs alone makes the notices needed, but is
        // not a GEL RAM notice: the verdict stays NO_GEL_NOTICES_FOUND.
        t.put("src/nfc_tables.rs", TABLES);
        let r = t.check();
        assert_eq!(r.third_party_notices(), ThirdParty::Missing);
        assert_eq!(r.distribution(), Distribution::NoGelNoticesFound);
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_root_is_named_dot_not_an_empty_path() {
        use std::os::unix::fs::PermissionsExt;
        let t = Scratch::new();
        fs::set_permissions(&t.0, fs::Permissions::from_mode(0o000)).unwrap();
        let result = check_dir(&t.0);
        fs::set_permissions(&t.0, fs::Permissions::from_mode(0o755)).unwrap();
        // A user who may read any directory cannot run this case.
        if let Err(error) = result {
            assert!(error.starts_with(".: "), "{error}");
        }
    }

    #[test]
    fn the_entry_cap_refuses_a_larger_tree() {
        let t = Scratch::new();
        for i in 0..5 {
            t.put(&format!("f{i}.txt"), b"x");
        }
        let limits = |max_entries| Limits {
            max_entries,
            max_read: MAX_READ,
        };
        assert!(check_dir_with(&t.0, limits(5)).is_ok());
        let error = check_dir_with(&t.0, limits(4)).unwrap_err();
        assert!(error.contains("more than 4 directory entries"), "{error}");
    }

    #[test]
    fn the_read_cap_skips_and_counts_oversize_files() {
        let t = Scratch::new();
        let mut toml = b"[package]\nname = \"gel-core\"\n".to_vec();
        toml.resize(200, b'\n');
        t.put("Cargo.toml", &toml);
        t.put("NOTICE", NOTICE_TEXT);
        t.put("LICENSE", LICENSE_TEXT);
        let small = Limits {
            max_entries: MAX_ENTRIES,
            max_read: 100,
        };
        let r = check_dir_with(&t.0, small).unwrap();
        // NOTICE and Cargo.toml exceed 100 bytes; the LICENSE is read up to
        // twice its pinned length.
        assert_eq!(r.oversize_skipped, 2);
        assert!(r.crates.is_empty());
        assert_eq!(r.license_file, LicenseFile::Exact);
        assert_eq!(r.distribution(), Distribution::ListedItemsPresent);
        let r = t.check();
        assert_eq!(r.oversize_skipped, 0);
        assert!(r.crates.contains("gel-core"));
        // A LICENSE larger than the cap is CHANGED, skipped and counted.
        let big = [LICENSE_TEXT, LICENSE_TEXT, b"x"].concat();
        t.put("LICENSE", &big);
        let r = check_dir_with(&t.0, small).unwrap();
        assert_eq!(r.license_file, LicenseFile::Changed);
        assert!(!r.license_line_endings_only);
        assert_eq!(r.oversize_skipped, 3);
    }

    #[test]
    fn only_the_first_bytes_of_a_source_file_are_read() {
        let t = Scratch::new();
        let mut big = header("").into_bytes();
        big.resize(4 * HEADER_READ as usize, b'/');
        t.put("big.rs", &big);
        let r = t.check();
        assert_eq!(
            (r.header_files, r.header_intact, r.oversize_skipped),
            (1, 1, 0)
        );
    }

    #[test]
    fn a_file_is_not_a_directory() {
        let t = Scratch::new();
        t.put("LICENSE", LICENSE_TEXT);
        assert_eq!(
            check_dir(&t.0.join("LICENSE")).unwrap_err(),
            "not a directory"
        );
        assert!(check_dir(&t.0.join("absent")).is_err());
    }

    // --- USE_VERDICT rule table --------------------------------------------

    /// All answers `no` except `key`, which is `value`.
    fn one(key: &str, value: Answer) -> Answers {
        Answers::from_values(KEYS.map(|k| if k == key { value } else { No }))
    }

    /// All answers `no` except `noncommercial_purpose=yes`, then `key` is `value`.
    fn one_nc(key: &str, value: Answer) -> Answers {
        Answers::from_values(KEYS.map(|k| match k {
            _ if k == key => value,
            "noncommercial_purpose" => Yes,
            _ => No,
        }))
    }

    #[test]
    fn row_1_each_monetized_item_requires_a_commercial_agreement() {
        for key in &KEYS[..MONETIZED] {
            for other in [No, Yes, Unsure] {
                // Whatever the other answers are, even unsure or a noncommercial purpose.
                let mut a = Answers::from_values([other; 7]);
                let mut values = a.values();
                values[KEYS.iter().position(|k| k == key).unwrap()] = Yes;
                a = Answers::from_values(values);
                assert_eq!(
                    use_verdict(&a),
                    UseVerdict::CommercialAgreementRequired,
                    "{key} with {other:?}"
                );
            }
        }
    }

    #[test]
    fn row_2_any_unsure_asks_the_licensor() {
        for key in KEYS {
            assert_eq!(
                use_verdict(&one_nc(key, Unsure)),
                UseVerdict::AskTheLicensor,
                "{key}"
            );
        }
        // Unsure wins over noncommercial_purpose=no and over evaluation_only=yes.
        let mut a = one("money", Unsure);
        a.evaluation_only = Yes;
        assert_eq!(use_verdict(&a), UseVerdict::AskTheLicensor);
    }

    #[test]
    fn row_3_evaluation_that_is_not_a_noncommercial_purpose_asks_the_licensor() {
        let a = one("evaluation_only", Yes);
        assert_eq!(use_verdict(&a), UseVerdict::AskTheLicensor);
        assert!(use_report(&a).contains(EVALUATION_NOTE));
        assert!(!use_report(&one_nc("evaluation_only", Yes)).contains(EVALUATION_NOTE));
    }

    #[test]
    fn row_4_not_a_noncommercial_purpose_requires_a_commercial_agreement() {
        assert_eq!(
            use_verdict(&Answers::default()),
            UseVerdict::CommercialAgreementRequired
        );
    }

    #[test]
    fn row_5_evaluation_only_is_evaluation() {
        assert_eq!(
            use_verdict(&one_nc("evaluation_only", Yes)),
            UseVerdict::Evaluation
        );
    }

    #[test]
    fn row_6_a_noncommercial_purpose_is_noncommercial() {
        assert_eq!(
            use_verdict(&one("noncommercial_purpose", Yes)),
            UseVerdict::Noncommercial
        );
    }

    /// An independent restatement of the rule, as a table over three counts:
    /// Monetized items answered yes, unsure answers, and the last two answers.
    fn restated(values: [Answer; 7]) -> (&'static str, &'static str) {
        let monetized_yes = values[..5].iter().filter(|v| **v == Yes).count();
        let unsure = values.iter().filter(|v| **v == Unsure).count();
        match (monetized_yes, unsure, values[5], values[6]) {
            (1.., _, _, _) => ("COMMERCIAL_AGREEMENT_REQUIRED", "2,5"),
            (0, 1.., _, _) => ("ASK_THE_LICENSOR", "2,19"),
            (0, 0, Yes, No) => ("ASK_THE_LICENSOR", "2,19"),
            (0, 0, No, No) => ("COMMERCIAL_AGREEMENT_REQUIRED", "2,5"),
            (0, 0, Yes, Yes) => ("EVALUATION", "2,4"),
            (0, 0, No, Yes) => ("NONCOMMERCIAL", "2,3"),
            other => unreachable!("{other:?}"),
        }
    }

    #[test]
    fn every_answer_combination_follows_the_restated_rule() {
        let all = [Yes, No, Unsure];
        let mut seen = std::collections::BTreeMap::new();
        for code in 0..3usize.pow(7) {
            let values: [Answer; 7] = std::array::from_fn(|i| all[code / 3usize.pow(i as u32) % 3]);
            let a = Answers::from_values(values);
            let v = use_verdict(&a);
            assert_eq!((v.name(), v.sections()), restated(values), "{values:?}");
            *seen.entry(v.name()).or_insert(0usize) += 1;
            // The file form of the same answers parses to the same answers.
            let mut text = String::new();
            for (key, value) in KEYS.iter().zip(values) {
                text.push_str(key);
                text.push('=');
                text.push_str(value.name());
                text.push('\n');
            }
            assert_eq!(parse_answers(&text), Ok(a), "{values:?}");
            // The conflict note appears exactly when a Monetized item and
            // evaluation_only are both yes.
            let conflict = values[..5].contains(&Yes) && values[5] == Yes;
            assert_eq!(
                use_report(&a).contains(CONFLICT_NOTE),
                conflict,
                "{values:?}"
            );
            let evaluation_conflict = values[5] == Yes && values[6] == No && !conflict;
            assert_eq!(
                use_report(&a).contains(EVALUATION_NOTE),
                evaluation_conflict,
                "{values:?}"
            );
        }
        // Every verdict is reached.
        assert_eq!(seen.len(), 4, "{seen:?}");
        assert_eq!(seen.values().sum::<usize>(), 2187);
    }

    #[test]
    fn use_reports_are_exact() {
        assert_eq!(
            use_report(&one("money", Yes)),
            format!("USE_VERDICT=COMMERCIAL_AGREEMENT_REQUIRED\nLICENSE_SECTIONS=2,5\nLICENSE_QUOTE=No Monetized Use is licensed by these public terms. Before beginning a Monetized Use, You must obtain a Commercial Agreement from the Licensor.\nLEGAL_CERTIFICATION=NO\nCONTACT=gelram.licensing@gmail.com\n{USE_NOTE}\n")
        );
        assert_eq!(
            USE_NOTE,
            "NOTE=answers are your own; this program reads the license, it does not interpret your facts"
        );
        let mut conflict = one("customer_production", Yes);
        conflict.evaluation_only = Yes;
        assert_eq!(
            use_report(&conflict),
            format!("USE_VERDICT=COMMERCIAL_AGREEMENT_REQUIRED\nLICENSE_SECTIONS=2,5\nLICENSE_QUOTE={}\n{CONFLICT_NOTE}\nLEGAL_CERTIFICATION=NO\nCONTACT=gelram.licensing@gmail.com\n{USE_NOTE}\n", UseVerdict::CommercialAgreementRequired.quote())
        );
        assert!(use_report(&one_nc("hosted", Unsure)).starts_with(
            "USE_VERDICT=ASK_THE_LICENSOR\nLICENSE_SECTIONS=2,19\nLICENSE_QUOTE=Commercial licensing and licensing questions: gelram.licensing@gmail.com\nLEGAL_CERTIFICATION=NO\n"
        ));
    }

    /// The whitespace-normalized text of LICENSE section `n`, up to section
    /// `next` or the end.
    fn section(n: &str, next: Option<&str>) -> String {
        let start = LICENSE_STR.find(&format!("\n{n}. ")).unwrap();
        let end = next.map_or(LICENSE_STR.len(), |next| {
            LICENSE_STR.find(&format!("\n{next}. ")).unwrap()
        });
        LICENSE_STR[start..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn verdict_quotes_are_word_for_word_from_the_cited_section() {
        for (verdict, n, next) in [
            (UseVerdict::CommercialAgreementRequired, "5", Some("6")),
            (UseVerdict::Evaluation, "4", Some("5")),
            (UseVerdict::Noncommercial, "3", Some("4")),
            (UseVerdict::AskTheLicensor, "19", None),
        ] {
            let text = section(n, next);
            for part in verdict.quote().split(QUOTE_GAP) {
                assert!(text.contains(part), "{verdict:?}: {part}");
            }
            assert!(verdict.sections().ends_with(n), "{verdict:?}");
        }
        // The conditions the grants depend on are quoted.
        assert!(UseVerdict::Noncommercial.quote().ends_with(
            "provided You preserve this license and the notices required by section 8."
        ));
        assert!(UseVerdict::Evaluation
            .quote()
            .contains("may not be deployed as part of a paid plan"));
        let definitions = section("2", Some("3"));
        assert!(CONFLICT_NOTE.contains("\"other than permitted Evaluation\""));
        assert!(definitions.contains("other than permitted Evaluation"));
    }

    #[test]
    fn question_quotes_are_word_for_word_from_the_license() {
        let definitions = {
            let start = LICENSE_STR.find("\n2. Definitions").unwrap();
            let end = LICENSE_STR.find("\n3. Noncommercial License").unwrap();
            &LICENSE_STR[start..end]
        };
        for (key, question) in KEYS.iter().zip(QUESTIONS) {
            let quotes: Vec<&str> = question.split('"').skip(1).step_by(2).collect();
            assert!(!quotes.is_empty(), "{key}");
            for quote in quotes {
                assert!(definitions.contains(quote), "{key}: {quote}");
            }
        }
        // The noncommercial_purpose question quotes the whole definition.
        let definition = definitions
            .lines()
            .find(|l| l.starts_with("\"Noncommercial Purpose\" means "))
            .unwrap();
        let quoted = definition
            .strip_prefix("\"Noncommercial Purpose\" means ")
            .unwrap()
            .strip_suffix('.')
            .unwrap();
        assert!(QUESTIONS[6].contains(&format!("\"{quoted}\"")), "{quoted}");
    }

    // --- answers parser refusals -------------------------------------------

    fn all(value: &str) -> String {
        KEYS.iter()
            .map(|k| format!("{k}={value}\n"))
            .collect::<Vec<_>>()
            .concat()
    }

    #[test]
    fn answers_allow_comments_blank_lines_spaces_crlf_and_a_byte_order_mark() {
        let text = format!(
            "\u{feff}# mine\n\n  money = no \r\n{}",
            &all("no")[KEYS[0].len() + 4..]
        )
        .replace("evaluation_only=no", "evaluation_only=yes")
        .replace("noncommercial_purpose=no", "noncommercial_purpose = yes");
        assert_eq!(
            parse_answers(&text).map(|a| use_verdict(&a)),
            Ok(UseVerdict::Evaluation)
        );
        assert_eq!(
            parse_answers(&format!("\u{feff}{}", all("unsure"))).map(|a| use_verdict(&a)),
            Ok(UseVerdict::AskTheLicensor)
        );
    }

    #[test]
    fn a_missing_key_is_refused() {
        let text = all("no").replace("hosted=no\n", "");
        assert_eq!(parse_answers(&text), Err("missing keys: hosted".into()));
        let text = all("no").replace("noncommercial_purpose=no\n", "");
        assert_eq!(
            parse_answers(&text),
            Err("missing keys: noncommercial_purpose".into())
        );
        assert_eq!(
            parse_answers(""),
            Err(format!("missing keys: {}", KEYS.join(", ")))
        );
    }

    #[test]
    fn an_unknown_key_is_refused_and_not_escaped_here() {
        let text = format!("{}nonprofit=yes\n", all("no"));
        assert_eq!(
            parse_answers(&text),
            Err("line 8: unknown key nonprofit".into())
        );
        let text = format!("Money=no\n{}", all("no"));
        assert_eq!(
            parse_answers(&text),
            Err("line 1: unknown key Money".into())
        );
        // Only one leading byte order mark is set aside; a second is part of
        // the key, which the program escapes once when it prints the reason.
        let text = format!("\u{feff}\u{feff}money=no\n{}", all("no"));
        assert_eq!(
            parse_answers(&text),
            Err("line 1: unknown key \u{feff}money".into())
        );
    }

    #[test]
    fn a_duplicate_key_is_refused() {
        let text = format!("{}money=no\n", all("no"));
        assert_eq!(
            parse_answers(&text),
            Err("line 8: duplicate key money".into())
        );
        let text = format!("{}cost_reduction=unsure\n", all("no"));
        assert_eq!(
            parse_answers(&text),
            Err("line 8: duplicate key cost_reduction".into())
        );
    }

    #[test]
    fn a_value_other_than_yes_no_or_unsure_is_refused() {
        for bad in [
            "", "Yes", "NO", "Unsure", "y", "1", "true", "yes no", "maybe", "not sure",
        ] {
            let text = all("no").replacen("money=no", &format!("money={bad}"), 1);
            assert_eq!(
                parse_answers(&text),
                Err("line 1: money must be yes, no or unsure".into()),
                "{bad:?}"
            );
        }
        assert_eq!(
            parse_answers("money\n"),
            Err("line 1: expected key=yes, key=no or key=unsure".into())
        );
    }

    #[test]
    fn the_template_lists_every_key_once_and_is_refused_until_answered() {
        let t = template();
        for key in KEYS {
            assert_eq!(
                t.lines().filter(|l| *l == format!("{key}=")).count(),
                1,
                "{key}"
            );
        }
        assert!(t
            .lines()
            .all(|l| l.starts_with('#') || l.is_empty() || l.ends_with('=')));
        assert!(t.lines().all(|l| l.chars().count() <= 78), "{t}");
        let money_line = t.lines().position(|l| l == "money=").unwrap() + 1;
        assert_eq!(
            parse_answers(&t),
            Err(format!(
                "line {money_line}: money must be yes, no or unsure"
            ))
        );
        let mut answered = String::new();
        for line in t.lines() {
            answered.push_str(line);
            if line.ends_with('=') {
                answered.push_str("no");
            }
            answered.push('\n');
        }
        assert_eq!(parse_answers(&answered), Ok(Answers::default()));
    }

    #[test]
    fn the_answers_file_is_bounded() {
        let t = Scratch::new();
        t.put("answers", all("no").as_bytes());
        assert_eq!(read_answers(&t.0.join("answers")), Ok(Answers::default()));
        t.put("big", &vec![b'#'; MAX_ANSWERS as usize + 1]);
        assert!(read_answers(&t.0.join("big"))
            .unwrap_err()
            .contains("larger than"));
        t.put("bin", b"money=\xff\n");
        assert_eq!(
            read_answers(&t.0.join("bin")),
            Err("answers file is not UTF-8".into())
        );
        assert!(read_answers(&t.0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_fifo_is_refused_without_being_opened() {
        let t = Scratch::new();
        let fifo = t.0.join("fifo");
        let made = std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .is_ok_and(|s| s.success());
        if !made {
            return;
        }
        let (send, receive) = std::sync::mpsc::channel();
        let path = fifo.clone();
        std::thread::spawn(move || {
            let _ = send.send(read_answers(&path));
        });
        let result = receive
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("reading a FIFO blocked: it was opened before its type was checked");
        assert_eq!(
            result,
            Err("cannot read the answers file: not a regular file".into())
        );
    }
}
