// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Host facts and the privacy filter of the hardware check
//! (docs/CHECK-ON-YOUR-HARDWARE.md).
//!
//! The facts come from files and the standard library only; nothing is
//! spawned, and a value that cannot be read is `unknown`. A report passes
//! `finish` before it is written or printed: this host's names, its user names
//! and directories, absolute paths, IPv4 and IPv6 addresses, MAC addresses and
//! serial-number values are replaced by `<redacted-…>` labels, and a report in
//! which the same scanner still finds one of them is refused.
use std::{fmt::Write as _, fs, path::Path};

const HOST: &str = "<redacted-host>";
const USER: &str = "<redacted-user>";
const PATH: &str = "<redacted-path>";
const IP: &str = "<redacted-ip>";
const MAC: &str = "<redacted-mac>";
const SERIAL: &str = "<redacted-serial>";

/// What names this host or its user.
pub(crate) struct Private {
    /// Host and user names with their labels, longest first; matched as whole
    /// words, ignoring case.
    words: Vec<(String, &'static str)>,
    /// Directories (home, checkout, output), longest first, matched as written.
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
        words.extend(users.iter().map(|u| (u.trim().to_string(), USER)));
        // A one-letter name would erase ordinary text and identifies nobody.
        words.retain(|(w, _)| w.chars().count() >= 2);
        words.sort_by_key(|(w, _)| (std::cmp::Reverse(w.chars().count()), w.to_lowercase()));
        words.dedup_by(|a, b| a.0.to_lowercase() == b.0.to_lowercase());
        let mut dirs: Vec<String> = dirs
            .iter()
            .map(|d| d.trim_end_matches(['/', '\\']).to_string())
            .filter(|d| d.chars().count() >= 2)
            .collect();
        dirs.sort_by_key(|d| (std::cmp::Reverse(d.chars().count()), d.clone()));
        dirs.dedup();
        Self { words, dirs }
    }

    /// This host: its name, the user's names and home directory from the
    /// environment and the host-name files, and the directories of the run.
    pub(crate) fn of_this_host(run_dirs: &[&Path]) -> Self {
        let env = |k: &str| std::env::var(k).ok();
        let file = |p: &str| fs::read_to_string(p).ok();
        let hosts: Vec<String> = [
            env("HOSTNAME"),
            env("COMPUTERNAME"),
            file("/proc/sys/kernel/hostname"),
            file("/etc/hostname"),
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
        for home in dirs.clone() {
            if let Some(name) = Path::new(&home).file_name() {
                users.push(name.to_string_lossy().into_owned());
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

/// A hyphen ends a word, so a name is also found inside `name-laptop`.
fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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

/// An absolute path: `/x`, `file:///x`, `~/x`, `C:\x` or `\\server\x`.
fn path(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    if !before.is_none_or(|b| b.is_whitespace() || "=([{\"'`,;:<>|".contains(b)) {
        return None;
    }
    let at = |k: usize| c.get(i + k).copied().unwrap_or(' ');
    let start = match (at(0), at(1), at(2)) {
        ('/', '/', '/') => true,
        ('/', n, _) => n.is_alphanumeric() || "._~-".contains(n),
        ('~', '/' | '\\', _) => true,
        (d, ':', '\\' | '/') => d.is_ascii_alphabetic(),
        ('\\', '\\', n) => n.is_alphanumeric() || "?.".contains(n),
        _ => false,
    };
    start.then(|| path_end(c, i) - i)
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

/// Four decimal numbers of at most 255 joined by dots.
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
    // Not followed by another digit or by a fifth number.
    let longer = digit_at(j) || (c.get(j) == Some(&'.') && digit_at(j + 1));
    (!longer).then_some(j - i)
}

/// Groups of at most four hexadecimal digits joined by `:`, with `::` or at
/// least three groups and a digit, an optional IPv4 tail and zone.
fn ipv6(c: &[char], i: usize, before: Option<char>) -> Option<usize> {
    if before.is_some_and(|b| b.is_alphanumeric() || b == ':' || b == '_' || b == '.') {
        return None;
    }
    let run = c[i..]
        .iter()
        .take_while(|x| x.is_ascii_hexdigit() || **x == ':')
        .count();
    let text: String = c[i..i + run].iter().collect();
    let groups: Vec<&str> = text.split(':').collect();
    let compressed = text.contains("::");
    let shaped = text.matches(':').count() >= 2
        && groups.iter().all(|g| g.len() <= 4)
        && groups.iter().any(|g| !g.is_empty())
        && (compressed
            || (groups.len() >= 3
                && groups.iter().all(|g| !g.is_empty())
                && text.chars().any(|x| x.is_ascii_digit())));
    if !shaped {
        return None;
    }
    let mut end = i + run;
    if c.get(end) == Some(&'.') {
        let last = i + text.rfind(':')? + 1;
        end = last + ipv4(c, last, Some(':'))?;
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
/// `serial=X`); the key is kept, the value is replaced.
fn serial(c: &[char], i: usize, before: Option<char>) -> Option<(usize, String)> {
    if before.is_some_and(|b| b.is_alphanumeric() || b == '_') || !c[i].is_alphanumeric() {
        return None;
    }
    let key = c[i..]
        .iter()
        .take(40)
        .take_while(|x| x.is_alphanumeric() || " _.-".contains(**x))
        .count();
    let name: String = c[i..i + key].iter().collect();
    let sep = i + key;
    if !name.to_lowercase().contains("serial") || !matches!(c.get(sep), Some('=' | ':')) {
        return None;
    }
    let mut value = sep + 1;
    while c.get(value).is_some_and(|&x| x == ' ' || x == '\t') {
        value += 1;
    }
    let end = value + c[value..].iter().take_while(|x| !x.is_whitespace()).count();
    let rest: String = c[value..end].iter().collect();
    if rest.is_empty() || rest.starts_with("<redacted-") {
        return None;
    }
    let kept: String = c[i..value].iter().collect();
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
        if starts_with(c, i, dir, false) {
            let end = path_end(c, i + dir.chars().count());
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

/// The length of a `<redacted-…>` label that starts at `i`.
fn label_len(c: &[char], i: usize) -> Option<usize> {
    if !starts_with(c, i, "<redacted-", false) {
        return None;
    }
    Some(c[i..].iter().take(24).position(|&x| x == '>')? + 1)
}

/// The text with every finding replaced, and the kinds found in order.
///
/// A label is copied as it is, and the character after a label or a
/// replacement is read as if it followed a letter. So the scan reads its own
/// output as it read the input, and a refusal points at a finding that the
/// replacement did not cover.
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

/// The text with this host's names, paths and addresses replaced, and the
/// number of replacements.
pub(crate) fn sanitise(text: &str, p: &Private) -> (String, usize) {
    let (out, kinds) = scan(text, p);
    (out, kinds.len())
}

/// The kinds of private detail the scanner finds in `text`, each once.
pub(crate) fn leaks(text: &str, p: &Private) -> Vec<&'static str> {
    let mut kinds = scan(text, p).1;
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

/// The last gate before a report is written: a text that still names a private
/// detail is refused.
fn refuse_leaks(text: &str, p: &Private) -> Result<(), String> {
    let left = leaks(text, p);
    if left.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "REFUSED the report still holds {} after filtering; nothing was written",
            left.join(",")
        ))
    }
}

/// The filtered report and its number of replacements, or a refusal.
pub(crate) fn finish(text: &str, p: &Private) -> Result<(String, usize), String> {
    let (clean, n) = sanitise(text, p);
    refuse_leaks(&clean, p)?;
    Ok((clean, n))
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

    /// A made-up host. The paths are assembled at run time, so the public tree
    /// itself holds no user directory path.
    fn fake() -> (Private, String) {
        let home = ["/", "home", "/", "jdoe"].concat();
        let windows = ["C:", "\\Use", "rs\\", "jdoe", "\\gel-ram"].concat();
        let p = Private::new(
            &["lab-box-7.example.net".into()],
            &["jdoe".into()],
            std::slice::from_ref(&home),
        );
        let text = format!(
            "host=lab-box-7 full=LAB-BOX-7.example.net by jdoe on jdoe-laptop\n\
             step=literal result=FAIL marker=GEL_LIVE_LAB_LITERAL=FAIL error={home}/gel-ram/README.md: denied\n\
             cwd={windows}\n\
             addr=203.0.113.7:8080 v6=2001:db8::7:1 mapped=::ffff:198.51.100.9\n\
             mac=02:00:5e:10:00:01 mac2=02-00-5E-10-00-02\n\
             Serial Number: SN-48151623\n\
             tilde=~/notes.txt url=file:///srv/data\n"
        );
        (p, text)
    }

    #[test]
    fn names_paths_and_addresses_of_the_host_are_removed() {
        let (p, raw) = fake();
        let found = leaks(&raw, &p);
        for kind in [HOST, USER, PATH, IP, MAC, SERIAL] {
            assert!(found.contains(&kind), "{kind} not seen: {found:?}");
        }
        let (clean, n) = finish(&raw, &p).expect("the filtered report is accepted");
        let lower = clean.to_lowercase();
        for secret in [
            "lab-box-7",
            "jdoe",
            "home",
            "users",
            "203.0.113.7",
            "2001:db8",
            "198.51.100.9",
            "02:00:5e",
            "02-00-5e",
            "48151623",
            "notes.txt",
            "srv",
        ] {
            assert!(!lower.contains(secret), "{secret} is left in:\n{clean}");
        }
        assert_eq!(n, 14, "{clean}");
        // What the reader needs survives: keys, markers and the port.
        for kept in [
            "step=literal result=FAIL marker=GEL_LIVE_LAB_LITERAL=FAIL error=",
            ":8080 v6=",
            "Serial Number: <redacted-serial>",
            "url=file:<redacted-path>",
        ] {
            assert!(clean.contains(kept), "{kept} is missing in:\n{clean}");
        }
    }

    #[test]
    fn a_report_the_filter_did_not_clean_is_refused() {
        let (p, raw) = fake();
        assert!(refuse_leaks(&raw, &p).is_err());
        let (clean, _) = sanitise(&raw, &p);
        assert_eq!(refuse_leaks(&clean, &p), Ok(()));
        // The labels themselves are never read as a finding, nor what follows them.
        assert_eq!(sanitise(&clean, &p), (clean.clone(), 0));
        let next = "by jdoe/notes and jdoe::1 at lab-box-7:/srv\n";
        let (once, n) = finish(next, &p).expect("a filtered text is accepted");
        assert_eq!(n, 4, "{once}");
        assert!(!once.contains("srv"), "{once}");
        assert_eq!(sanitise(&once, &p), (once.clone(), 0));
    }

    #[test]
    fn a_clean_report_passes_unchanged() {
        let (p, _) = fake();
        let clean = "GEL_WITNESS_2\n\
             tool=xtask-witness version=0.8.0\n\
             revision=319b4e8d0c1f2a3b4c5d6e7f8091a2b3c4d5e6f7 working_tree=clean\n\
             os=linux arch=x86_64\n\
             os_version=Debian GNU/Linux 12 (bookworm)\n\
             kernel=6.17.0-1033-oem\n\
             cpu_model=Intel(R) Core(TM) i7-8565U CPU @ 1.80GHz w/ Radeon 890M\n\
             logical_cpus=8 available_parallelism=8\n\
             memory_total_kib=16310532\n\
             rustc=rustc 1.85.0 (4d91de4e4 2025-02-17)\n\
             step=literal result=PASS exit=0 seconds=12.34 marker=GEL_LIVE_LAB_LITERAL=PASS\n\
             step=quad result=PASS exit=0 seconds=1.05 marker=Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED\n\
             step=crash-series result=SKIPPED reason=the kill series needs a Unix host\n\
             step=history-crash-series result=PASS exit=0 seconds=9.80 marker=HISTORY_CRASH_SERIES=PASS kills=5/5 date=2026-11-03\n\
             step=same-bits result=FAIL exit=signal-9 seconds=0.50 marker=WITNESS=MISMATCH platform=windows-x86_64 | bits pinned on the author's Linux host before any other run\n\
             note=DEGRADED_REFERENCE_SERIAL_FALLBACK absent; std::fs ok; see https://github.com/Gelram-project/gel-ram/blob/main/docs/REPRODUCE.md\n\
             GEL_WITNESS=FAIL pass=3 fail=1 skipped=1\n";
        assert_eq!(sanitise(clean, &p), (clean.to_string(), 0));
        assert_eq!(finish(clean, &p), Ok((clean.to_string(), 0)));
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
