//! Public disclosure gate (`xtask disclosure`, part of `xtask verify`).
//!
//! The public tree must not carry user directory paths, network addresses,
//! credentials, or e-mail addresses other than the published contacts. The gate
//! holds no list of words: a list kept in a public file, even as hashes, can be
//! recovered by guessing. Wording is reviewed before publication, outside this
//! repository; see docs/PUBLIC-DISCLOSURE-GATE.md for what this gate does not cover.
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Published contact addresses; any other e-mail address is a finding.
const CONTACTS: &[&str] = &["gelram.licensing@gmail.com", "noreply@anthropic.com"];
/// Domains that never identify a person: GitHub no-reply and documentation examples.
const CONTACT_DOMAINS: &[&str] = &[
    "users.noreply.github.com",
    "example.com",
    "example.org",
    "example.net",
];
/// Absolute directory prefixes followed by a user name.
const USER_DIRS: &[&str] = &["/home/", "/media/", "/Users/", "\\Users\\", "/run/media/"];
/// Names after a user directory that identify no person: CI runners and placeholders.
const NEUTRAL_USERS: &[&str] = &[
    "runner",
    "runneradmin",
    "private",
    "user",
    "username",
    "example",
];
/// Credential prefixes and the least number of key characters that follow them.
const TOKEN_PREFIXES: &[(&str, usize)] = &[
    ("ghp_", 30),
    ("gho_", 30),
    ("ghu_", 30),
    ("ghs_", 30),
    ("ghr_", 30),
    ("github_pat_", 20),
    ("gsk_", 20),
    ("sk-", 20),
    ("xoxb-", 10),
    ("xoxp-", 10),
    ("AKIA", 16),
];

/// Findings in one text.
fn scan(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    // A user name after an absolute user directory; relative paths such as `../media/x`
    // or `main/media/x` are not absolute and are not read.
    for dir in USER_DIRS {
        for (at, _) in text.match_indices(dir) {
            let absolute = text[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || "._-/".contains(c)));
            if !absolute {
                continue;
            }
            let name: String = text[at + dir.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "._-".contains(*c))
                .collect();
            if !name.is_empty() && !NEUTRAL_USERS.contains(&name.to_lowercase().as_str()) {
                found.push(format!("user directory path after {dir}"));
            }
        }
    }
    // IPv4 addresses outside loopback and the documentation ranges (RFC 5737).
    for part in text.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
        let octets: Vec<&str> = part.split('.').collect();
        let ip = octets.len() == 4
            && octets
                .iter()
                .all(|o| (1..=3).contains(&o.len()) && o.parse::<u16>().is_ok_and(|v| v <= 255));
        let allowed = part.starts_with("127.")
            || part == "0.0.0.0"
            || part == "255.255.255.255"
            || ["192.0.2.", "198.51.100.", "203.0.113."]
                .iter()
                .any(|p| part.starts_with(p));
        if ip && !allowed {
            found.push(format!("network address {part}"));
        }
    }
    // Credentials.
    for (prefix, least) in TOKEN_PREFIXES {
        for (at, _) in text.match_indices(prefix) {
            let boundary = text[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_ascii_alphanumeric());
            let key = text[at + prefix.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .count();
            if boundary && key >= *least {
                found.push(format!("credential-like string with prefix {prefix}"));
            }
        }
    }
    // Markers assembled at run time, so this file is not a finding itself.
    let (begin, end) = (
        ["-----", "BEGIN "].concat(),
        [" PRIVATE", " KEY-----"].concat(),
    );
    if text.contains(&begin) && text.contains(&end) {
        found.push("private key block".into());
    }
    // E-mail addresses other than the published contacts.
    for (at, _) in text.match_indices('@') {
        let local: String = text[..at]
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || "._%+-".contains(*c))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let domain: String = text[at + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || ".-".contains(*c))
            .collect();
        let domain = domain.trim_end_matches('.');
        let tld = domain.rsplit('.').next().unwrap_or("");
        if local.is_empty()
            || !domain.contains('.')
            || tld.len() < 2
            || !tld.chars().all(|c| c.is_ascii_alphabetic())
        {
            continue;
        }
        let address = format!("{local}@{domain}").to_lowercase();
        if !CONTACTS.contains(&address.as_str())
            && !CONTACT_DOMAINS.contains(&domain.to_lowercase().as_str())
        {
            found.push(format!("e-mail address at {domain}"));
        }
    }
    found
}

/// Frozen answer-or-abstain question files quote Wikipedia passages byte for byte in their last
/// column, and `answer-bench check` verifies each passage against its SHA-256. A race time
/// written as four dot-separated numbers in a quoted results table has the shape of an IPv4 address. Only in those files,
/// and only for that check, an address found solely inside the quoted passages is not a finding;
/// every other column and every other check is read as usual.
fn quoted_only_address(name: &str, text: &str, finding: &str) -> bool {
    let Some(addr) = finding.strip_prefix("network address ") else {
        return false;
    };
    if !(name.starts_with("docs/answer-or-abstain") && name.ends_with("-questions.txt")) {
        return false;
    }
    let outside = text
        .lines()
        .map(|l| {
            if l.starts_with('#') {
                l
            } else {
                l.rsplit_once('\t').map_or(l, |(head, _)| head)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    !outside
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .any(|part| part == addr)
}

/// Paths whose Polish text is quoted data (questions, recorded answers, frozen records),
/// not documentation prose.
const POLISH_DATA_PATHS: &[&str] = &[
    "docs/answer-or-abstain",
    "docs/evidence-",
    "docs/GEL-BESIDE-GROQ.md",
    "docs/GEL-BESIDE-GROQ-NO-ANSWER.md",
];

/// Documentation prose is English only. In Markdown, SVG and HTML files outside the data
/// paths, a line with a letter used only in Polish is a finding; Markdown code blocks and
/// inline code (identifiers, search examples) are not prose and are skipped.
fn polish_prose(name: &str, text: &str) -> Vec<String> {
    let prose = [".md", ".svg", ".html"].iter().any(|e| name.ends_with(e));
    let crate_fixture = name.starts_with("crates/") && name.contains("/fixtures/");
    if !prose || crate_fixture || POLISH_DATA_PATHS.iter().any(|p| name.starts_with(p)) {
        return Vec::new();
    }
    let markdown = name.ends_with(".md");
    let mut fence = false;
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if markdown && line.trim_start().starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        let outside_code: String = if markdown {
            line.split('`').step_by(2).collect()
        } else {
            line.to_string()
        };
        if outside_code.chars().any(|c| "ąćęłńśźżĄĆĘŁŃŚŹŻ".contains(c)) {
            found.push(format!(
                "Polish text in documentation prose at line {}",
                index + 1
            ));
        }
    }
    found
}

/// Scans every file of the tree (not `target` or `.git`); binary files are skipped.
pub fn check(root: &Path, files: &[PathBuf]) -> Result<(), String> {
    let (mut scanned, mut bad) = (0usize, Vec::new());
    for path in files {
        let Ok(bytes) = fs::read(path) else { continue };
        if bytes.contains(&0) {
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        scanned += 1;
        let name = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        for finding in polish_prose(&name, &text) {
            bad.push(format!("{name}: {finding}"));
        }
        for finding in scan(&text) {
            if quoted_only_address(&name, &text, &finding) {
                continue;
            }
            bad.push(format!("{name}: {finding}"));
        }
    }
    if !bad.is_empty() {
        return Err(format!(
            "PUBLIC_DISCLOSURE_GATE=FAIL findings={}\n{}",
            bad.len(),
            bad.join("\n")
        ));
    }
    println!("PUBLIC_DISCLOSURE_GATE=PASS text_files={scanned}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every sample is assembled at run time, so the gate never flags this file.
    fn run(text: &str) -> Vec<String> {
        scan(text)
    }

    #[test]
    fn holds_no_word_list() {
        let source = include_str!("disclosure.rs");
        let hex_constant = source
            .split(|c: char| !c.is_ascii_hexdigit())
            .any(|t| t.len() >= 32);
        assert!(!hex_constant, "the gate must not carry hashes of words");
    }

    #[test]
    fn finds_user_directories_but_not_relative_or_neutral_paths() {
        let home = format!("/{}/{}{}/work", "home", "some", "one");
        assert_eq!(run(&home).len(), 1);
        let mounted = format!("/{}/{}/x", "media", "disk");
        assert_eq!(run(&mounted).len(), 1);
        assert!(run("see ../media/gifs/a.gif and /home/runner/x").is_empty());
        assert!(run("https://example.com/blob/main/media/gifs/a.gif").is_empty());
        assert!(run("test /home/private ... ok").is_empty());
    }

    #[test]
    fn finds_addresses_outside_documentation_ranges() {
        let ip = ["10", "1", "2", "3"].join(".");
        assert_eq!(run(&format!("host {ip} up")).len(), 1);
        for fine in [
            "203.0.113.1",
            "127.0.0.1",
            "1.2.3.4.5",
            "3.184029",
            "v0.5.2",
        ] {
            assert!(run(fine).is_empty(), "{fine}");
        }
    }

    #[test]
    fn finds_credentials_and_key_blocks() {
        let token = format!("gh{}_{}", "p", "a".repeat(36));
        assert_eq!(run(&format!("key {token}")).len(), 1);
        assert!(run(&format!("task-{}", "b".repeat(30))).is_empty());
        let block = format!("-----BEGIN {} KEY-----", "PRIVATE");
        assert_eq!(run(&block).len(), 1);
    }

    #[test]
    fn quoted_passages_are_not_read_as_addresses() {
        let time = ["1", "15", "23", "6"].join(".");
        let finding = format!("network address {time}");
        let file = "docs/answer-or-abstain-v4/with-answer-questions.txt";
        let quoted = format!("# header\n1\ten\tWho won?\tX\tx\tresults {time} X\n");
        assert!(quoted_only_address(file, &quoted, &finding));
        let asked = format!("1\ten\tWhat is {time}?\tX\tx\tresults {time} X\n");
        assert!(!quoted_only_address(file, &asked, &finding));
        assert!(!quoted_only_address("docs/NOTES.md", &quoted, &finding));
        assert!(!quoted_only_address(
            file,
            &quoted,
            "e-mail address at mail.test"
        ));
    }

    #[test]
    fn polish_prose_only_outside_code_and_data() {
        let word = ["zaż", "ółć"].concat();
        assert_eq!(
            polish_prose("docs/NOTES.md", &format!("a {word} b")).len(),
            1
        );
        assert!(polish_prose("docs/NOTES.md", &format!("run `find {word}`")).is_empty());
        let fenced = format!("```\nfind {word}\n```\nplain");
        assert!(polish_prose("docs/NOTES.md", &fenced).is_empty());
        let data = "docs/answer-or-abstain-v4/README.md";
        assert!(polish_prose(data, &word).is_empty());
        assert!(polish_prose("crates/x/fixtures/a.md", &word).is_empty());
        assert!(polish_prose("crates/x/src/lib.rs", &word).is_empty());
        assert_eq!(polish_prose("media/a.svg", &word).len(), 1);
    }

    #[test]
    fn allows_only_published_contacts() {
        assert!(run(&format!("{}@{}", "gelram.licensing", "gmail.com")).is_empty());
        assert!(run(&format!("{}@{}", "1+x", "users.noreply.github.com")).is_empty());
        assert_eq!(run(&format!("{}@{}", "someone", "mail.test")).len(), 1);
        assert!(run("#[derive] @owner mention").is_empty());
    }
}
