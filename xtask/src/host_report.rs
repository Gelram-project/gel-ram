// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Host facts and the privacy filter of the hardware check
//! (docs/CHECK-ON-YOUR-HARDWARE.md).
//!
//! A report has two parts. The fact lines (tool, revision, system, CPU, cores,
//! memory, rustc) are made by this program from system files and the standard
//! library; to fill them it runs only `git` and `rustc -V`, and `hostname` to
//! learn the name it must hide. The other lines carry what the steps printed
//! and pass `sanitise`: this host's names, the user's names and directories,
//! absolute paths, IPv4, IPv6 and MAC addresses and serial-number values that
//! the scanner recognises become `<redacted-…>` labels.
//!
//! `finish` then checks the whole report by a second, separate method: it
//! searches the text for this host's names, the user's names (also each part of
//! a name with spaces) and the directories, in any letter case and with either
//! path separator, and refuses the report if one is still there. Paths and
//! addresses that do not belong to this host or its user are only as well
//! hidden as the scanner recognises them.
use std::{fmt::Write as _, fs, path::Path, process::Command};

const HOST: &str = "<redacted-host>";
const USER: &str = "<redacted-user>";
const PATH: &str = "<redacted-path>";
const IP: &str = "<redacted-ip>";
const MAC: &str = "<redacted-mac>";
const SERIAL: &str = "<redacted-serial>";
/// The labels this filter writes; nothing else is copied unread.
const LABELS: [&str; 6] = [HOST, USER, PATH, IP, MAC, SERIAL];

/// Keys that name a serial number, longest first.
const SERIAL_KEYS: &[&str] = &[
    "serial number",
    "serial_number",
    "serial-number",
    "serialnumber",
    "serial no",
    "serial_no",
    "serial",
    "s/n",
    "sn",
];

/// What names this host or its user.
pub(crate) struct Private {
    /// Host and user names with their labels, longest first; matched as whole
    /// words, ignoring case.
    words: Vec<(String, &'static str)>,
    /// Directories (home, checkout, output), longest first; matched ignoring
    /// case, with `/`, `\` or a doubled `\` for each separator.
    dirs: Vec<String>,
}

impl Private {
    pub(crate) fn new(hosts: &[String], users: &[String], dirs: &[String]) -> Self {
        let mut words: Vec<(String, &'static str)> = Vec::new();
        for host in hosts.iter().map(|h| h.trim()) {
            words.push((host.to_string(), HOST));
            // The first label of a full name is the name on its own.
            if let Some((first, _)) = host.split_once('.') {
                words.push((first.to_string(), HOST));
            }
        }
        for user in users.iter().map(|u| u.trim()) {
            words.push((user.to_string(), USER));
            // "John Doe" is also found as "John" and as "Doe".
            words.extend(user.split_whitespace().map(|part| (part.to_string(), USER)));
        }
        // A one-letter name would erase ordinary text and identifies nobody.
        words.retain(|(w, _)| w.chars().count() >= 2);
        words.sort_by_key(|(w, _)| (std::cmp::Reverse(w.chars().count()), w.to_lowercase()));
        words.dedup_by(|a, b| a.0.to_lowercase() == b.0.to_lowercase());
        let mut all: Vec<String> = Vec::new();
        for dir in dirs {
            let dir = dir.trim().trim_end_matches(['/', '\\']);
            all.push(dir.to_string());
            // A canonical Windows path starts with \\?\, which text rarely shows.
            if let Some(plain) = dir.strip_prefix(r"\\?\") {
                all.push(plain.to_string());
            }
        }
        all.retain(|d| d.chars().count() >= 2);
        all.sort_by_key(|d| (std::cmp::Reverse(d.chars().count()), d.to_lowercase()));
        all.dedup_by(|a, b| a.to_lowercase() == b.to_lowercase());
        Self { words, dirs: all }
    }

    /// This host: its name from the environment, the host-name files and the
    /// `hostname` command; the user's names and home directory from the
    /// environment; and the directories of the run.
    pub(crate) fn of_this_host(run_dirs: &[&Path]) -> Self {
        let env = |k: &str| std::env::var(k).ok();
        let file = |p: &str| fs::read_to_string(p).ok();
        // macOS exports no host name and has no host-name file.
        let command = Command::new("hostname")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
        let hosts: Vec<String> = [
            env("HOSTNAME"),
            env("COMPUTERNAME"),
            file("/proc/sys/kernel/hostname"),
            file("/etc/hostname"),
            command,
        ]
        .into_iter()
        .flatten()
        .collect();
        let mut users: Vec<String> = [env("USER"), env("USERNAME"), env("LOGNAME")]
            .into_iter()
            .flatten()
            .collect();
        let mut dirs: Vec<String> = [env("HOME"), env("USERPROFILE")]
            .into_iter()
            .flatten()
            .collect();
        // The last part of a home directory is usually the user name.
        for home in &dirs {
            if let Some(name) = home.rsplit(['/', '\\']).find(|s| !s.is_empty()) {
                users.push(name.to_string());
            }
        }
        dirs.extend(run_dirs.iter().map(|d| d.to_string_lossy().into_owned()));
        Self::new(&hosts, &users, &dirs)
    }
}

fn same(a: char, b: char, fold: bool) -> bool {
    a == b || (fold && a.to_lowercase().eq(b.to_lowercase()))
}

fn starts_with(c: &[char], i: usize, s: &str, fold: bool) -> bool {
    s.chars()
        .enumerate()
        .all(|(k, ch)| c.get(i + k).is_some_and(|&x| same(x, ch, fold)))
}

/// Anything but a letter or digit ends a word, so a name is also found inside
/// `name-laptop` or `name_x`.
fn word_char(c: char) -> bool {
    c.is_alphanumeric()
}

fn separator(c: char) -> bool {
    c == '/' || c == '\\'
}

/// The length of `dir` written at `i`, in any letter case, with `/`, `\` or a
/// doubled `\` (as in escaped text) for each separator.
fn dir_at(c: &[char], i: usize, dir: &str) -> Option<usize> {
    let mut j = i;
    for d in dir.chars() {
        let x = *c.get(j)?;
        if separator(d) {
            if !separator(x) {
                return None;
            }
            j += 1;
            if x == '\\' && c.get(j) == Some(&'\\') {
                j += 1;
            }
        } else if same(x, d, true) {
            j += 1;
        } else {
            return None;
        }
    }
    Some(j - i)
}

/// Where a path that starts at `i` ends: at white space or a closing mark.
fn path_end(c: &[char], mut i: usize) -> usize {
    while c
        .get(i)
        .is_some_and(|&x| !x.is_whitespace() && !"\"'`,;)]}<>|".contains(x))
    {
        i += 1;
    }
    i
}

/// A path may hold spaces (`John Doe\gel-ram`): it goes on across one space
/// when the next piece holds a path separator.
fn path_extent(c: &[char], i: usize) -> usize {
    let mut end = path_end(c, i);
    while c.get(end) == Some(&' ') {
        let next = path_end(c, end + 1);
        let piece = &c[end + 1..next];
        // `Doe\gel-ram` goes on; `key=/x` or a new path does not.
        let more = piece.first().is_some_and(|x| x.is_alphanumeric())
            && piece.iter().any(|&x| separator(x))
            && !piece.iter().any(|&x| x == '=' || x == ':');
        if !more {
            break;
        }
        end = next;
    }
    end
}

/// An absolute path: `/x`, `file:///x`, `~/x`, `C:\x` or `\\server\x`, after
/// any character that cannot be part of a path.
fn path(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    if before.is_some_and(|b| b.is_alphanumeric() || "_.-/\\~".contains(b)) {
        return None;
    }
    let at = |k: usize| c.get(i + k).copied().unwrap_or(' ');
    let start = match (at(0), at(1), at(2)) {
        ('/', '/', '/') => true,
        ('/', n, _) => n.is_alphanumeric() || "._~".contains(n),
        ('~', '/' | '\\', _) => true,
        (d, ':', '\\' | '/') => d.is_ascii_alphabetic(),
        ('\\', '\\', n) => n.is_alphanumeric() || "?.".contains(n),
        _ => false,
    };
    start.then(|| path_extent(c, i) - i)
}

/// Six pairs of hexadecimal digits joined by `:` or `-`.
fn mac(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    let sep = *c.get(i + 2)?;
    if !matches!(sep, ':' | '-') || before.is_some_and(|b| b.is_alphanumeric() || b == sep) {
        return None;
    }
    for g in 0..6 {
        let hex = |k: usize| c.get(i + 3 * g + k).is_some_and(|x| x.is_ascii_hexdigit());
        if !(hex(0) && hex(1)) || (g < 5 && c.get(i + 3 * g + 2) != Some(&sep)) {
            return None;
        }
    }
    let after = c.get(i + 17);
    after
        .is_none_or(|&a| !(a.is_alphanumeric() || a == sep || a == '_'))
        .then_some(17)
}

/// Four decimal numbers of at most 255 joined by dots, not part of a longer
/// four-part version with a suffix, as some kernels have.
fn ipv4(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    if before.is_some_and(|b| b.is_alphanumeric() || b == '.') {
        return None;
    }
    let mut j = i;
    for part in 0..4 {
        if part > 0 {
            if c.get(j) != Some(&'.') {
                return None;
            }
            j += 1;
        }
        let digits = c[j.min(c.len())..]
            .iter()
            .take_while(|x| x.is_ascii_digit())
            .count();
        if !(1..=3).contains(&digits) {
            return None;
        }
        let value: u32 = c[j..j + digits].iter().collect::<String>().parse().ok()?;
        if value > 255 {
            return None;
        }
        j += digits;
    }
    let digit_at = |k: usize| c.get(k).is_some_and(|x| x.is_ascii_digit());
    // Not followed by another digit, a fifth number or a version suffix.
    let longer = digit_at(j)
        || (c.get(j) == Some(&'.') && digit_at(j + 1))
        || (c.get(j) == Some(&'-') && c.get(j + 1).is_some_and(|x| x.is_alphanumeric()));
    (!longer).then_some(j - i)
}

/// An IPv6 address: eight groups of at most four hexadecimal digits, or fewer
/// with exactly one `::`; the last two groups may be an IPv4 address; an
/// optional zone.
fn ipv6(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    if before.is_some_and(|b| b.is_alphanumeric() || b == ':' || b == '_' || b == '.') {
        return None;
    }
    let run = c[i..]
        .iter()
        .take_while(|x| x.is_ascii_hexdigit() || **x == ':')
        .count();
    let text: String = c[i..i + run].iter().collect();
    let compressed = text.matches("::").count();
    if text.matches(':').count() < 2 || compressed > 1 || text.contains(":::") {
        return None;
    }
    let mut groups: Vec<&str> = text.split(':').collect();
    let mut end = i + run;
    let tail = c.get(end) == Some(&'.');
    if tail {
        let last = i + text.rfind(':')? + 1;
        end = last + ipv4(c, last, Some(':'))?;
        groups.pop();
    }
    if groups.iter().any(|g| g.len() > 4) {
        return None;
    }
    let filled = groups.iter().filter(|g| !g.is_empty()).count();
    let room = if tail { 6 } else { 8 };
    let shaped = if compressed == 1 {
        (1..room).contains(&filled)
    } else {
        filled == room && groups.len() == room
    };
    if !shaped {
        return None;
    }
    if c.get(end) == Some(&'%') {
        end += 1 + c[end + 1..]
            .iter()
            .take_while(|x| x.is_ascii_alphanumeric())
            .count();
    }
    c.get(end)
        .is_none_or(|&a| !(a.is_alphanumeric() || a == '_'))
        .then_some(end - i)
}

/// The value after a key that names a serial number (`Serial Number: X`,
/// `serial\t: X`, `"serial": "X"`, `S/N=X`, or `serial X` when X has a digit);
/// the key is kept, the value is replaced.
fn serial(c: &[char], i: usize, before: Option<char>) -> Option<(usize, String)> {
    if before.is_some_and(|b| b.is_alphanumeric() || b == '_') {
        return None;
    }
    let key = SERIAL_KEYS.iter().find(|k| {
        starts_with(c, i, k, true)
            && c.get(i + k.len())
                .is_none_or(|x| !(x.is_alphanumeric() || *x == '_'))
    })?;
    let blank = |x: &char| *x == ' ' || *x == '\t';
    let quote = |x: Option<&char>| matches!(x, Some('"' | '\''));
    let mut j = i + key.len();
    if quote(c.get(j)) {
        j += 1;
    }
    let gap = j;
    while c.get(j).is_some_and(blank) {
        j += 1;
    }
    let separated = matches!(c.get(j), Some(':' | '='));
    if separated {
        j += 1;
        while c.get(j).is_some_and(blank) {
            j += 1;
        }
    } else if j == gap || *key == "sn" {
        return None;
    }
    if quote(c.get(j)) {
        j += 1;
    }
    let end = j + c[j..]
        .iter()
        .take_while(|x| !x.is_whitespace() && !"\"'`,;)]}".contains(**x))
        .count();
    let value: String = c[j..end].iter().collect();
    let digit = value.chars().any(|x| x.is_ascii_digit());
    if value.is_empty() || value.starts_with("<redacted-") || (!separated && !digit) {
        return None;
    }
    let kept: String = c[i..j].iter().collect();
    Some((end - i, format!("{kept}{SERIAL}")))
}

/// The redaction that starts at `i` after the character `before`, if any:
/// characters taken, replacement and kind.
fn redaction_at(
    c: &[char],
    i: usize,
    before: Option<char>,
    p: &Private,
) -> Option<(usize, String, &'static str)> {
    for dir in &p.dirs {
        if let Some(n) = dir_at(c, i, dir) {
            let end = path_extent(c, i + n);
            return Some((end - i, PATH.into(), PATH));
        }
    }
    if before.is_none_or(|b| !word_char(b)) {
        for (word, label) in &p.words {
            let n = word.chars().count();
            if starts_with(c, i, word, true) && c.get(i + n).is_none_or(|&a| !word_char(a)) {
                return Some((n, (*label).into(), label));
            }
        }
    }
    if let Some(n) = mac(c, i, before) {
        return Some((n, MAC.into(), MAC));
    }
    if let Some(n) = ipv4(c, i, before).or_else(|| ipv6(c, i, before)) {
        return Some((n, IP.into(), IP));
    }
    if let Some((n, text)) = serial(c, i, before) {
        return Some((n, text, SERIAL));
    }
    path(c, i, before).map(|n| (n, PATH.into(), PATH))
}

/// The length of one of this filter's own labels at `i`.
fn label_len(c: &[char], i: usize) -> Option<usize> {
    LABELS
        .iter()
        .find(|l| starts_with(c, i, l, false))
        .map(|l| l.len())
}

/// The text with every finding replaced, and the kinds found in order. One of
/// this filter's own labels is copied as it is, and the character after a
/// label or a replacement is read as if it followed a letter.
fn scan(text: &str, p: &Private) -> (String, Vec<&'static str>) {
    let c: Vec<char> = text.chars().collect();
    let (mut out, mut kinds) = (String::with_capacity(text.len()), Vec::new());
    let (mut i, mut before) = (0, None);
    while i < c.len() {
        if let Some(n) = label_len(&c, i) {
            out.extend(&c[i..i + n]);
            (i, before) = (i + n, Some('a'));
            continue;
        }
        match redaction_at(&c, i, before, p) {
            Some((n, replacement, kind)) if n > 0 => {
                out.push_str(&replacement);
                kinds.push(kind);
                (i, before) = (i + n, Some('a'));
            }
            _ => {
                out.push(c[i]);
                (i, before) = (i + 1, Some(c[i]));
            }
        }
    }
    (out, kinds)
}

/// The text with what the scanner recognises replaced, and the number of
/// replacements.
pub(crate) fn sanitise(text: &str, p: &Private) -> (String, usize) {
    let (out, kinds) = scan(text, p);
    (out, kinds.len())
}

/// Lower case, with every separator (also a doubled `\`) written as `/`.
fn flat(text: &str) -> String {
    text.to_lowercase().replace("\\\\", "/").replace('\\', "/")
}

/// The words of the system's own name in the fact lines (`os=`,
/// `os_version=`). A host or user named like the system (`ubuntu`, `kali`) has
/// its default name, which identifies nobody; in the fact lines it is allowed.
fn system_words(facts: &str) -> Vec<String> {
    facts
        .lines()
        .filter_map(|l| {
            l.strip_prefix("os_version=")
                .or_else(|| l.strip_prefix("os=").and_then(|v| v.split(' ').next()))
        })
        .flat_map(|v| {
            v.split(|x: char| !x.is_alphanumeric())
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
        })
        .filter(|w| w.chars().count() >= 2)
        .collect()
}

/// The second check, made without the scanner: which of this host's names,
/// user names and directories `text` still holds as plain text, in any letter
/// case and with either separator. Names in `allowed` do not count.
fn still_named(text: &str, p: &Private, allowed: &[String]) -> Vec<&'static str> {
    let text = LABELS
        .iter()
        .fold(text.to_string(), |t, l| t.replace(l, " "));
    let lower = text.to_lowercase();
    let mut found = Vec::new();
    for (word, kind) in &p.words {
        let w = word.to_lowercase();
        if allowed.contains(&w) {
            continue;
        }
        let whole = lower.match_indices(&w).any(|(at, _)| {
            let before = lower[..at].chars().next_back();
            let after = lower[at + w.len()..].chars().next();
            before.is_none_or(|b| !b.is_alphanumeric())
                && after.is_none_or(|a| !a.is_alphanumeric())
        });
        if whole {
            found.push(if *kind == HOST {
                "host name"
            } else {
                "user name"
            });
        }
    }
    let text = flat(&text);
    if p.dirs.iter().any(|d| text.contains(&flat(d))) {
        found.push("directory");
    }
    found
}

/// The report: the fact lines as they are, the other lines filtered, and the
/// number of replacements; or a refusal when the second check still finds this
/// host's name, the user's name or a directory anywhere in it.
pub(crate) fn finish(facts: &str, body: &str, p: &Private) -> Result<(String, usize), String> {
    finish_with(facts, body, p, sanitise)
}

/// The first pass over the step lines: text and number of replacements.
type Filter = fn(&str, &Private) -> (String, usize);

fn finish_with(
    facts: &str,
    body: &str,
    p: &Private,
    filter: Filter,
) -> Result<(String, usize), String> {
    let (clean, n) = filter(body, p);
    let mut left = still_named(facts, p, &system_words(facts));
    left.extend(still_named(&clean, p, &[]));
    left.sort_unstable();
    left.dedup();
    if left.is_empty() {
        Ok((format!("{facts}{clean}"), n))
    } else {
        Err(format!(
            "REFUSED the report still holds a {} after filtering; nothing was written",
            left.join(", a ")
        ))
    }
}

/// One line of a value, with no control characters.
fn one_line(value: &str) -> String {
    let v: String = value
        .chars()
        .map(|x| if x.is_control() { ' ' } else { x })
        .collect();
    let v = v.trim();
    if v.is_empty() {
        "unknown".into()
    } else {
        v.to_string()
    }
}

/// The value of `key` in an os-release file, without quotes.
fn os_release(text: &str, key: &str) -> Option<String> {
    let line = text
        .lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))?;
    let value = line.trim().trim_matches(['"', '\'']).to_string();
    (!value.is_empty()).then_some(value)
}

/// The system name and version of a Linux os-release file.
fn linux_version(text: &str) -> Option<String> {
    os_release(text, "PRETTY_NAME").or_else(|| {
        let name = os_release(text, "NAME")?;
        Some(match os_release(text, "VERSION_ID") {
            Some(v) => format!("{name} {v}"),
            None => name,
        })
    })
}

/// The product version in macOS's SystemVersion.plist.
fn macos_version(plist: &str) -> Option<String> {
    let after = plist.split("<key>ProductVersion</key>").nth(1)?;
    let value = after.split("<string>").nth(1)?.split("</string>").next()?;
    Some(format!("macOS {}", value.trim()))
}

/// The value of the first `key : value` line of /proc/cpuinfo.
fn cpuinfo_value(cpuinfo: &str, key: &str) -> Option<String> {
    cpuinfo.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == key && !v.trim().is_empty()).then(|| v.trim().to_string())
    })
}

/// The CPU name in /proc/cpuinfo; some ARM systems give only a board model.
fn cpu_model(cpuinfo: &str) -> Option<String> {
    ["model name", "Model", "Hardware", "cpu model", "cpu"]
        .iter()
        .find_map(|k| cpuinfo_value(cpuinfo, k))
}

/// The number of logical CPUs listed in /proc/cpuinfo.
fn logical_cpus(cpuinfo: &str) -> Option<usize> {
    let n = cpuinfo
        .lines()
        .filter(|l| {
            l.split_once(':')
                .is_some_and(|(k, _)| k.trim() == "processor")
        })
        .count();
    (n > 0).then_some(n)
}

/// MemTotal of /proc/meminfo in KiB.
fn memory_total_kib(meminfo: &str) -> Option<u64> {
    let line = meminfo.lines().find_map(|l| l.strip_prefix("MemTotal:"))?;
    line.split_whitespace().next()?.parse().ok()
}

/// The host lines of a report: system, version, CPU, cores and memory.
pub(crate) fn facts() -> String {
    let read = |p: &str| fs::read_to_string(p).ok();
    let cpuinfo = read("/proc/cpuinfo").unwrap_or_default();
    let env = |k: &str| {
        if cfg!(windows) {
            std::env::var(k).ok()
        } else {
            None
        }
    };
    let version = read("/etc/os-release")
        .or_else(|| read("/usr/lib/os-release"))
        .and_then(|t| linux_version(&t))
        .or_else(|| {
            read("/System/Library/CoreServices/SystemVersion.plist").and_then(|t| macos_version(&t))
        });
    let unknown = || "unknown".to_string();
    let mut s = String::new();
    let _ = writeln!(
        s,
        "os={} arch={}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let _ = writeln!(
        s,
        "os_version={}",
        one_line(&version.unwrap_or_else(unknown))
    );
    let kernel = read("/proc/sys/kernel/osrelease").unwrap_or_else(unknown);
    let _ = writeln!(s, "kernel={}", one_line(&kernel));
    let model = cpu_model(&cpuinfo)
        .or_else(|| env("PROCESSOR_IDENTIFIER"))
        .unwrap_or_else(unknown);
    let _ = writeln!(s, "cpu_model={}", one_line(&model));
    let logical = logical_cpus(&cpuinfo)
        .or_else(|| env("NUMBER_OF_PROCESSORS").and_then(|n| n.trim().parse().ok()))
        .map_or_else(unknown, |n| n.to_string());
    let available =
        std::thread::available_parallelism().map_or_else(|_| unknown(), |n| n.to_string());
    let _ = writeln!(
        s,
        "logical_cpus={logical} available_parallelism={available}"
    );
    let memory = read("/proc/meminfo")
        .and_then(|m| memory_total_kib(&m))
        .map_or_else(unknown, |k| k.to_string());
    let _ = writeln!(s, "memory_total_kib={memory}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up host: its two home directories and the hostile text. The
    /// paths are assembled at run time, so the public tree itself holds no
    /// user directory path.
    fn fake() -> (Private, String, String, String) {
        let home = ["/", "home", "/", "jdoe"].concat();
        let windows = ["C:", "\\Use", "rs\\", "John Doe"].concat();
        let p = Private::new(
            &["lab-box-7.example.net".into()],
            &["jdoe".into(), "John Doe".into()],
            &[home.clone(), windows.clone()],
        );
        let lower = windows.to_lowercase();
        let escaped = windows.replace('\\', "\\\\");
        let text = format!(
            "host=lab-box-7 full=LAB-BOX-7.example.net by jdoe on jdoe-laptop\n\
             step=literal result=FAIL marker=GEL_LIVE_LAB_LITERAL=FAIL error={home}/gel-ram/README.md: denied\n\
             cwd={lower}\\gel-ram\\x.rs done\n\
             debug=\"{escaped}\\\\gel\" end\n\
             other=D:\\Data\\Jane Roe\\notes\\x.txt end\n\
             addr=203.0.113.7:8080 v6=2001:db8::7:1 mapped=::ffff:198.51.100.9\n\
             mac=02:00:5e:10:00:01 mac2=02-00-5E-10-00-02\n\
             Serial Number: SN-48151623\n\
             Serial\t\t: 100000001\n\
             {{\"serial\": \"ABC123\"}}\n\
             serial XYZ987\n\
             S/N: Q7W8E9\n\
             tilde=~/notes.txt url=file:///srv/data at=x@/srv/secret/x star=a*/srv/s2 plus=b+/srv/s3\n\
             label=<redacted-{home}>\n"
        );
        (p, text, home, windows)
    }

    #[test]
    fn names_paths_and_addresses_of_the_host_are_removed() {
        let (p, raw, _, _) = fake();
        let kinds = scan(&raw, &p).1;
        for kind in LABELS {
            assert!(kinds.contains(&kind), "{kind} not seen: {kinds:?}");
        }
        let (clean, n) =
            finish("os=linux arch=x86_64\n", &raw, &p).expect("the filtered report is accepted");
        let lower = clean.to_lowercase();
        for secret in [
            "lab-box-7",
            "jdoe",
            "john",
            "doe",
            "home",
            "users",
            "jane",
            "roe",
            "data",
            "notes",
            "srv",
            "secret",
            "203.0.113.7",
            "2001:db8",
            "198.51.100.9",
            "02:00:5e",
            "02-00-5e",
            "48151623",
            "100000001",
            "abc123",
            "xyz987",
            "q7w8e9",
        ] {
            assert!(!lower.contains(secret), "{secret} is left in:\n{clean}");
        }
        assert_eq!(n, 24, "{clean}");
        // What the reader needs survives: keys, markers and the port.
        for kept in [
            "step=literal result=FAIL marker=GEL_LIVE_LAB_LITERAL=FAIL error=<redacted-path> denied",
            "cwd=<redacted-path> done",
            "debug=\"<redacted-path>\" end",
            ":8080 v6=",
            "Serial Number: <redacted-serial>",
            "Serial\t\t: <redacted-serial>",
            "{\"serial\": \"<redacted-serial>\"}",
            "serial <redacted-serial>",
            "S/N: <redacted-serial>",
            "url=file:<redacted-path>",
            "at=x@<redacted-path>",
        ] {
            assert!(clean.contains(kept), "{kept} is missing in:\n{clean}");
        }
    }

    /// No filter at all, and a weak one that removes only IPv4 addresses.
    fn no_filter(text: &str, _: &Private) -> (String, usize) {
        (text.to_string(), 0)
    }

    fn ipv4_only(text: &str, _: &Private) -> (String, usize) {
        let c: Vec<char> = text.chars().collect();
        let (mut out, mut n, mut i) = (String::new(), 0, 0);
        while i < c.len() {
            match ipv4(&c, i, i.checked_sub(1).map(|j| c[j])) {
                Some(k) => {
                    out.push_str(IP);
                    (n, i) = (n + 1, i + k);
                }
                None => {
                    out.push(c[i]);
                    i += 1;
                }
            }
        }
        (out, n)
    }

    #[test]
    fn the_second_check_refuses_what_a_weak_filter_left() {
        let (p, raw, home, windows) = fake();
        let facts = "os=linux arch=x86_64\n";
        let weak: [Filter; 2] = [no_filter, ipv4_only];
        for filter in weak {
            let refused = finish_with(facts, &raw, &p, filter).unwrap_err();
            for kind in ["host name", "user name", "directory"] {
                assert!(refused.contains(kind), "{kind}: {refused}");
            }
        }
        // Each one alone, in any letter case and with either separator.
        let escaped = windows.replace('\\', "\\\\");
        for (text, kind) in [
            ("built by JDOE\n".to_string(), "user name"),
            ("for Doe only\n".to_string(), "user name"),
            ("on Lab-Box-7\n".to_string(), "host name"),
            (format!("in {}\n", home.to_uppercase()), "directory"),
            (format!("in {}\n", home.replace('/', "\\")), "directory"),
            (format!("in \"{escaped}\\\\x\"\n"), "directory"),
        ] {
            let refused = finish_with(facts, &text, &p, no_filter).unwrap_err();
            assert!(refused.contains(kind), "{text}: {refused}");
        }
        // A name in the fact lines counts too.
        let named = "os=linux arch=x86_64\ncpu_model=lab-box-7 CPU\n";
        assert!(finish(named, "", &p).is_err());
        // The filter's own labels are not names, whatever the names are.
        let odd = Private::new(&["host".into()], &["user".into()], &[]);
        let labelled = format!("{HOST} {USER} {PATH}\n");
        assert_eq!(
            finish(facts, &labelled, &odd),
            Ok((format!("{facts}{labelled}"), 0))
        );
    }

    #[test]
    fn system_names_and_look_alikes_are_left_alone() {
        let p = Private::new(&["ubuntu".into()], &["kali".into()], &[]);
        // A four-part kernel version, assembled so that the source holds no
        // text in the shape of an address.
        let wsl = ["6.6.87", ".2-microsoft-standard-WSL2"].concat();
        let facts = format!(
            "os=linux arch=x86_64\n\
             os_version=Ubuntu 24.04.1 LTS\n\
             kernel={wsl}\n\
             cpu_model=Example CPU 8C @ 3.10GHz\n\
             rustc=rustc 1.85.0 (4d91de4e4 2025-02-17)\n"
        );
        let body = format!(
            "step=x result=FAIL exit=1 seconds=0.10 marker=X=FAIL at 12:34:56 in a::b::c, \
             deserialize: bad, serial port ok, build {wsl}\n"
        );
        assert_eq!(finish(&facts, &body, &p), Ok((format!("{facts}{body}"), 0)));
        let facts = facts.as_str();
        let kali = "os=linux arch=x86_64\nos_version=Kali GNU/Linux Rolling\n";
        assert_eq!(finish(kali, "", &p), Ok((kali.to_string(), 0)));
        // In the step lines the same name is still a name.
        let (clean, n) = finish(facts, "seen on ubuntu\n", &p).unwrap();
        assert_eq!((n, clean.ends_with("seen on <redacted-host>\n")), (1, true));
    }

    #[test]
    fn a_clean_report_passes_unchanged() {
        let (p, _, _, _) = fake();
        let facts = "GEL_WITNESS_2\n\
             tool=xtask-witness version=0.8.0\n\
             revision=0123456789abcdef0123456789abcdef01234567 working_tree=clean\n\
             os=linux arch=x86_64\n\
             os_version=Example GNU/Linux 12 (codename)\n\
             kernel=5.10.0-27-example\n\
             cpu_model=ExampleCorp(R) Model(TM) X7-1234U CPU @ 1.80GHz w/ Example Graphics 600\n\
             logical_cpus=8 available_parallelism=8\n\
             memory_total_kib=16310532\n\
             rustc=rustc 1.85.0 (4d91de4e4 2025-02-17)\n";
        let body = "step=literal result=PASS exit=0 seconds=12.34 marker=GEL_LIVE_LAB_LITERAL=PASS\n\
             step=quad result=PASS exit=0 seconds=1.05 marker=Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED\n\
             step=crash-series result=SKIPPED reason=the kill series needs a Unix host\n\
             step=history-crash-series result=PASS exit=0 seconds=9.80 marker=HISTORY_CRASH_SERIES=PASS kills=5/5 date=2026-11-03\n\
             step=same-bits result=FAIL exit=signal-9 seconds=0.50 marker=WITNESS=MISMATCH platform=windows-x86_64 | bits pinned on the author's Linux host before any other run\n\
             note=DEGRADED_REFERENCE_SERIAL_FALLBACK absent; std::fs ok; see https://github.com/Gelram-project/gel-ram/blob/main/docs/REPRODUCE.md\n";
        assert_eq!(sanitise(body, &p), (body.to_string(), 0));
        assert_eq!(finish(facts, body, &p), Ok((format!("{facts}{body}"), 0)));
    }

    #[test]
    fn host_facts_are_read_from_the_system_files() {
        let cpuinfo = "processor\t: 0\nmodel name\t: Example CPU 9\n\nprocessor\t: 1\nmodel name\t: Example CPU 9\n";
        assert_eq!(cpu_model(cpuinfo).as_deref(), Some("Example CPU 9"));
        assert_eq!(logical_cpus(cpuinfo), Some(2));
        let board = "processor\t: 0\nSerial\t\t: 100000001\nModel\t\t: Example Board 4\n";
        assert_eq!(cpu_model(board).as_deref(), Some("Example Board 4"));
        assert_eq!(cpu_model(""), None);
        assert_eq!(
            memory_total_kib("MemTotal:       16310532 kB\nMemFree: 1 kB\n"),
            Some(16310532)
        );
        let release = "NAME=\"Example OS\"\nVERSION_ID=\"7.1\"\n";
        assert_eq!(linux_version(release).as_deref(), Some("Example OS 7.1"));
        let pretty = format!("PRETTY_NAME=\"Example OS 7.1 LTS\"\n{release}");
        assert_eq!(
            linux_version(&pretty).as_deref(),
            Some("Example OS 7.1 LTS")
        );
        let plist = "<dict><key>ProductName</key><string>macOS</string>\
                     <key>ProductVersion</key>\n\t<string>15.1</string></dict>";
        assert_eq!(macos_version(plist).as_deref(), Some("macOS 15.1"));
        let lines = facts();
        for key in [
            "os=",
            "os_version=",
            "kernel=",
            "cpu_model=",
            "logical_cpus=",
            "memory_total_kib=",
        ] {
            assert!(lines.lines().any(|l| l.starts_with(key)), "{key}: {lines}");
        }
    }
}
