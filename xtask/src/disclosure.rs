//! Public disclosure gate (`xtask disclosure`, part of `xtask verify`).
//!
//! The public tree must not carry private file paths or user names, internal
//! names of the separate private project, network addresses, credentials, or
//! e-mail addresses other than the published contacts. Internal names are
//! listed only as SHA-256 of their normalized form (lowercase words; a phrase
//! is its two words joined by one space), so this file discloses none of them.
//! The owner keeps the plain list outside the repository.
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

/// SHA-256 of internal words and two-word phrases that must not appear publicly.
const FORBIDDEN: &[&str] = &[
    "458d334febf1a8b2c09e4cb739a62aa8af94a80b0c89c76ad6bf1f13f1bc8382",
    "0b371d37ed17420fd32c87f9457f629a0cb7ee9bac54964f879203f60165c609",
    "7294f58aabdac49d0f575d5609d9662f57f6d1e90cbfdd34a43c3e768d3a11bd",
    "2cb9d2eea4e5bdd9a9eaef9d641baa733012a4e39f76577852c9650c0432a697",
    "6d50af042f44621b82cecafd34e83c2faa0cd77297de972898be37c4d33a9e2a",
    "221a8b06e12d2215ff280a50115a10d51625abb251e07d462178696abfefc2fa",
    "1b580361fc03dc714c5f761e9814b90ed5f131fa63d21cfc108994b0bc51f5fd",
    "79bdffcb89e2674c8b4ec7c39900e2895a49b22ec5de6de3c019b1e20cd6dc56",
    "b0088d5f7e869cdc3e567706297fdc27674a500c2470991c585053a4e9a411d6",
    "4e0e74892d693a124d0ded230ea0ae5751d2e7a74052db4f3f76891d59428894",
    "275bab244d6b7be342da7e1a55baa376e23ef7a80c37145134c3c912e17b34d0",
    "5f06e112c6882c36ca4e3a25d0d18714b16a2b07b64778bd85fc0af6175ee641",
    "0ca25dd05fddc63803568cb54d561562123b7e3f7cae3e7f1518ad71063e9436",
    "069961936c6bbceee1498003640b53870b285d702928654212d8b9b08e5ff281",
    "5b1a82f9343927fe713dbf99d77aa00f704fbedca22ea9a5d2218f41ca826cd4",
    "8811665faca629fd005521886f8b8340311490f98812eed4d9ef8302a1478a79",
];
/// SHA-256 of private user names that must not follow a home or media directory.
const PRIVATE_USERS: &[&str] =
    &["597c28c381ef1feee61f3e9677a628b4cbd41cfb2539c8938062e1df2a882d39"];
/// Published contact addresses; any other e-mail address is a finding.
const CONTACTS: &[&str] = &["gelram.licensing@gmail.com", "noreply@anthropic.com"];
/// Domains that never identify a person: GitHub no-reply and documentation examples.
const CONTACT_DOMAINS: &[&str] = &[
    "users.noreply.github.com",
    "example.com",
    "example.org",
    "example.net",
];
/// Directory prefixes followed by a user name.
const USER_DIRS: &[&str] = &["/home/", "/media/", "/Users/", "\\Users\\", "/run/media/"];
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

fn sha(text: &str) -> String {
    gel_source::hex(&gel_source::digest(text.as_bytes()))
}

/// Findings in one text. Internal names are reported by an 8-character hash prefix only.
fn scan(text: &str, forbidden: &HashSet<&str>, users: &HashSet<&str>) -> Vec<String> {
    let mut found = Vec::new();
    // Internal words and two-word phrases, case-insensitive, across any punctuation.
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let mut seen = HashSet::new();
    for (i, word) in words.iter().enumerate() {
        let mut candidates = vec![word.clone()];
        if let Some(next) = words.get(i + 1) {
            candidates.push(format!("{word} {next}"));
        }
        for candidate in candidates {
            if !seen.insert(candidate.clone()) {
                continue;
            }
            let hash = sha(&candidate);
            if forbidden.contains(hash.as_str()) {
                found.push(format!("internal name (sha256 {})", &hash[..8]));
            }
        }
    }
    // Private user names after a home or media directory.
    for dir in USER_DIRS {
        for (at, _) in text.match_indices(dir) {
            let name: String = text[at + dir.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "._-".contains(*c))
                .collect();
            if !name.is_empty() && users.contains(sha(&name.to_lowercase()).as_str()) {
                found.push(format!("private user directory after {dir}"));
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

/// Scans every file of the tree (not `target` or `.git`); binary files are skipped.
pub fn check(root: &Path, files: &[PathBuf]) -> Result<(), String> {
    let forbidden: HashSet<&str> = FORBIDDEN.iter().copied().collect();
    let users: HashSet<&str> = PRIVATE_USERS.iter().copied().collect();
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
        for finding in scan(&text, &forbidden, &users) {
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
    println!(
        "PUBLIC_DISCLOSURE_GATE=PASS text_files={scanned} internal_names={} private_users={}",
        FORBIDDEN.len(),
        PRIVATE_USERS.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every sample is assembled at run time, so the gate never flags this file.
    fn sets() -> (Vec<String>, Vec<String>) {
        (
            vec![sha("example secret"), sha(&["exam", "pleword"].concat())],
            vec![sha(&["some", "one"].concat())],
        )
    }

    fn run(text: &str) -> Vec<String> {
        let (f, u) = sets();
        let forbidden: HashSet<&str> = f.iter().map(String::as_str).collect();
        let users: HashSet<&str> = u.iter().map(String::as_str).collect();
        scan(text, &forbidden, &users)
    }

    #[test]
    fn finds_internal_names_across_case_and_punctuation() {
        let word = ["Exam", "pleWord"].concat();
        assert_eq!(run(&format!("a {word}, b")).len(), 1);
        assert_eq!(run("the Example-Secret here").len(), 1);
        assert!(run("an example of a secret").is_empty());
    }

    #[test]
    fn finds_private_user_directories_but_not_relative_paths() {
        let home = format!("/{}/{}{}/work", "home", "some", "one");
        assert_eq!(run(&home).len(), 1);
        assert!(run("see ../media/gifs/a.gif and /home/runner/x").is_empty());
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
    fn allows_only_published_contacts() {
        assert!(run(&format!("{}@{}", "gelram.licensing", "gmail.com")).is_empty());
        assert!(run(&format!("{}@{}", "1+x", "users.noreply.github.com")).is_empty());
        assert_eq!(run(&format!("{}@{}", "someone", "mail.test")).len(), 1);
        assert!(run("#[derive] @owner mention").is_empty());
    }
}
