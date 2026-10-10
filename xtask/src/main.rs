#![forbid(unsafe_code)]
mod answer_bench;
mod audit_io;
mod bench_compare;
mod ci_evidence;
mod claims;
mod crash_series;
mod disclosure;
mod isolation;
mod license_metadata;
mod measured_sources;
mod mutation_campaign;
mod mutation_matrix;
mod package;
mod process_sequence;
mod property_map;
#[cfg(test)]
mod publication_status_tests;
mod recorder_lint;
mod reproduce;
mod reproduction;
mod roadmap;
mod source_bundle;
mod workflow_policy;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const ALLOWED_EXTENSIONS: &[&str] = &["rs", "md", "toml", "yml", "txt", "gel", "cff"];
// Exact reviewed media and source archive only; no general binary exception.
// Pins detect changed bytes; they do not prove decoding safety or semantic truth.
const REVIEWED_ASSETS: &[(&str, &str)] = &[
    (
        "media/gifs/01-evidence-light.gif",
        "53636b0287f01a2651191d954d974e384c91c686b5e3d4bb55bbe3102da18f57",
    ),
    (
        "media/gifs/01-evidence-light.png",
        "4a6e692d1e8a0b86788d2a0e3ee9657e6e6440224f64e3dfbed09c22c105cfe3",
    ),
    (
        "media/gifs/01-evidence-dark.gif",
        "0bb3403ce698f2e5ce6fcc70c04c25a276265e6781b56627b6ca33bb8a8e9d1d",
    ),
    (
        "media/gifs/01-evidence-dark.png",
        "dba278f6e2f0a81d4704e7bf652e22df349a9f5e88798db8d886afbe0ded2883",
    ),
    (
        "media/gifs/02-stale-light.gif",
        "ca2606ca4e62cc298841f07bc0edcb1c11177ec4ed819a5bc5284fe5533a634c",
    ),
    (
        "media/gifs/02-stale-light.png",
        "ea2a12aedc96dd0abd88b5d73817c98bcfbabfa68dcedd6132bb71d9b571a18d",
    ),
    (
        "media/gifs/02-stale-dark.gif",
        "389e6f5f1b9f320a4d85a96122344e77fbcace48fdc0794273738d779d08f7b2",
    ),
    (
        "media/gifs/02-stale-dark.png",
        "a117038d615597d4cba2b54fadbb6dad3ef44abe01df515e59df2a89b93f1d0a",
    ),
    (
        "media/gifs/03-backup-light.gif",
        "82f46988e1b2863180768f11e77fcaf911cb89eeebd63e1e84e1d792932e27fa",
    ),
    (
        "media/gifs/03-backup-light.png",
        "46a309b109085bb02487924bfcffde13df933f619dab1a0e5fdcd0144bbe1e1f",
    ),
    (
        "media/gifs/03-backup-dark.gif",
        "2184ebaaf47fdf5cb086dc2267d6d91c78a7d30ae35c6322f73ad73f2776b610",
    ),
    (
        "media/gifs/03-backup-dark.png",
        "d3e9f5ca90cc52fc0575bdab8d380f6c77597f1d39907db2c6694fd9242286bf",
    ),
    (
        "media/gifs/04-reproduce-light.gif",
        "1adb7e15937b10372e0c19160972cf29581ac17f819d442609457b48d49ba77f",
    ),
    (
        "media/gifs/04-reproduce-light.png",
        "c2bff0a465523f468914f0a04d4f1291e0070d20da7919787e273561a9e4fddc",
    ),
    (
        "media/gifs/04-reproduce-dark.gif",
        "dcaa8afae19d72af698270f4649fd98dd1eb1a3a3bb9271d7ab8c9fb07ac7e2e",
    ),
    (
        "media/gifs/04-reproduce-dark.png",
        "cea27d1bcb3757430b7685bec6611b923247b63ecfa77ba1c75b9d8e4dad7f33",
    ),
    (
        "media/gifs/05-integrity-light.gif",
        "ba6e1e8449c45bffc80923e200fdc502b36616d6c9a8b126be2b79008e7df496",
    ),
    (
        "media/gifs/05-integrity-light.png",
        "c49159370d28a1b5b7f1d6b61b369bc53814de733641f40f83a9671c6533300e",
    ),
    (
        "media/gifs/05-integrity-dark.gif",
        "da4b7699891e9ad334ed7f28155415d305dd08a56620f3b2506dc9187db86719",
    ),
    (
        "media/gifs/05-integrity-dark.png",
        "8ded780ada24b9beb953f6579f69754c92afa4a0e2dc4153edb5b4152621e277",
    ),
    (
        "media/gifs/07-literal-light.gif",
        "603423555fe9ab91d721bc0667c07af7830ff7fc5fbf7f8a7ebea372fc418c33",
    ),
    (
        "media/gifs/07-literal-light.png",
        "4d494435ff510eb122448d5f0c8541fa97ba9b5c12f608e6b04e8e198cd9562f",
    ),
    (
        "media/gifs/07-literal-dark.gif",
        "2659273a529620a822a47a17cbeb1d73bed30920c81dc9eb893e7891b8eb45fd",
    ),
    (
        "media/gifs/07-literal-dark.png",
        "5829514c64f0f50367a8f1dab0ee06d0e76e369fe916789171f88806e2ca4537",
    ),
    // Exact script-free documentation presentation assets; no HTML/SVG wildcard.
    (
        "README-MULTIMEDIA.html",
        "d68cc675473c8822b435965ddd44faee93432f8cdd55fb8d0b8ba72e49c9af90",
    ),
    (
        "media/presentation/header-light.svg",
        "b54fcff8c5fb76b474b5ac0808fcff85a1db02a83ef5cc7788bf3b4bd9cec5c8",
    ),
    (
        "media/presentation/header-dark.svg",
        "c0755f16a769d392955fdc6b10fafaee9839579a283ea6614c3f559d3197327a",
    ),
    (
        "media/presentation/header-still-light.svg",
        "b2baa837b0108bf86ca62be91b1f7dbcabe6521e1d159983964adcb52c254557",
    ),
    (
        "media/presentation/header-still-dark.svg",
        "933da2774f1fa1bebf1fef64d7662461d9c1388890d165d67151b59b3717de07",
    ),
    (
        "media/presentation/record-light.svg",
        "e58907c27302a6808e3277ae099b12c353fb85b2586374684920b33c71704ea0",
    ),
    (
        "media/presentation/record-dark.svg",
        "45cefe87675ed8537af6c8a71d1729efdcca76ccb60e46818df19741172e9837",
    ),
    (
        "media/presentation/flow-light.svg",
        "f69dccd7b004f5b95d36fbc4773e8c962b2c39be1a5c23186b152991d1274f5a",
    ),
    (
        "media/presentation/facts-light.svg",
        "93624f881d896900e63d1694c1cc4219150d48b5ef95461ccedcb353ba89a44e",
    ),
    (
        "media/presentation/wall-light.svg",
        "1587f805345b09c171b74af481190f74ac49a78c54f373ccda7142cd0f273d83",
    ),
    (
        "media/presentation/bars-light.svg",
        "d83fb2f2e9abaa13e7371fd9d6df1a2f7bc50bc92f6bda61a43404b6121e05e0",
    ),
    (
        "media/presentation/logo-light.svg",
        "0bfa881db2ea8d988c385664eb840f8faef1c5e931e37a7e8d6d3f101b67b0b5",
    ),
    (
        "media/presentation/logo-still-light.svg",
        "2cb1b673a1c1791b5cdd30b7194f6b2bf4f3f9c7d909c939470e2de7c79f26e4",
    ),
    (
        "media/presentation/flow-dark.svg",
        "4248e73198f4274634098a70f0b6739f981da45f040f20a8444e8051dc005032",
    ),
    (
        "media/presentation/facts-dark.svg",
        "9f12978745d6dc6cb1298fd5bd1b85f032e670b0401b5a9f8ab5326a6675a2a5",
    ),
    (
        "media/presentation/wall-dark.svg",
        "db451d62af58b85ac21d9f9f68822b709e85c48a9a34a4eff1109a3dddebb3ff",
    ),
    (
        "media/presentation/bars-dark.svg",
        "ff0e0e44a62c8d17f7ba8017fafcca308b9510738085194d11166039fa36e5f4",
    ),
    (
        "media/presentation/logo-dark.svg",
        "1bb4a8be9ff7c951a62e1a18e1ee7ba6d9861213df7cd7ca4e1f64413286be6b",
    ),
    (
        "media/presentation/logo-still-dark.svg",
        "0863d48ebe5fbb4199e9947f29518f6f3224250142cb4d4eb4c3c9f68926a132",
    ),
    (
        "media/presentation/chips/01-light.svg",
        "d4207f2334a255e66fc3a92cfeba06ff3ee66dd50b045496c41005c4694f3fc2",
    ),
    (
        "media/presentation/chips/01-dark.svg",
        "68bbe4c1bed153874c101d9e5d751637f2bcbf4068c8778dfbd2e041ed303b1f",
    ),
    (
        "media/presentation/chips/02-light.svg",
        "4e399c79f4e0bfa56bdf4a86b551191a5534fd499d51de414e5b0d31cd63b3aa",
    ),
    (
        "media/presentation/chips/02-dark.svg",
        "e03c7ac8e5a4a2794180cd22913477ed7e54e5d32dde3066f0089c1cf5465ce3",
    ),
    (
        "media/presentation/chips/03-light.svg",
        "865090b3f865e2682231124d870338099a17ec37e9407e5be3973c8571f079d9",
    ),
    (
        "media/presentation/chips/03-dark.svg",
        "88c6121b08bc8a95f8666d9a02af346ffc0776e5ece865a605e323ef11599c86",
    ),
    (
        "media/presentation/chips/04-light.svg",
        "058fafbbe1bc80d77cd2b24f506674b3e8940b70e40ef2ce7f8500f0506f770d",
    ),
    (
        "media/presentation/chips/04-dark.svg",
        "458ebc39c0b25944f5fe249fa709f7fb011bb4359afc6721c2289d9aa1b91177",
    ),
    (
        "media/presentation/chips/05-light.svg",
        "f6696cfb93f899c380b24988cd3473fd8987f2c71b156752355407e9aae76a70",
    ),
    (
        "media/presentation/chips/05-dark.svg",
        "256f86e09deaee7505000945f5c67eec176fcdfbeebe17666701bbf6d4e9d804",
    ),
    (
        "media/presentation/chips/06-light.svg",
        "2204aa54dee2fe1b9b31d8341ae6caee88532f10616a1fb36bc14ae350d5afb4",
    ),
    (
        "media/presentation/chips/06-dark.svg",
        "6be92d4d5fd3f3474b4e7237ad097989083bbe31c4862c9791b490d3c87e98fd",
    ),
    (
        "media/evidence-lab/03-update-restart-102s.png",
        "d3de2029ce993457d411649bcac5a41f27f1ec1ed9586cb2f9b4dcbbc8675d14",
    ),
    (
        "media/GEL-EVIDENCE-LAB-EN.mp4",
        "a7b4e85d5db1274c61a49d8814908321130dad6c3eec9324de0500071d9c9835",
    ),
    (
        "media/evidence-lab/01-source-17s.png",
        "9c73954fe6b732f56140d7d541e2ba4e04768ced5f3f4ea90036de93b2e240c0",
    ),
    (
        "media/evidence-lab/02-reopened-56s.png",
        "3d385ca2c4862ac873c089be1502770550ea810d5af51d16a15656335e08a82a",
    ),
    (
        "docs/images/q8-four-views-en.png",
        "061a5e99f1fa5ba8380a5b8ed6c60d672904b2b8069382971a60c0192a95c104",
    ),
    (
        "docs/images/evidence-limits-en.png",
        "6a6b9af2ea626591863d56b6d082613794d52ae1b8f825244ba0b62fecd8b9fd",
    ),
];
const ALLOWED_EXTENSIONLESS: &[&str] = &[
    "Cargo.lock",
    "LICENSE",
    "NOTICE",
    ".gitignore",
    ".gitattributes",
];

/// A backtick-quoted token with one of these extensions is a repository path citation.
const REFERENCE_EXTENSIONS: &[&str] = &["md", "toml", "txt", "yml", "rs", "lock", "cff"];
/// A backtick-quoted token starting with one of these prefixes is a repository path citation.
const REFERENCE_PREFIXES: &[&str] = &["docs/", ".github/", "crates/", "xtask/"];

/// The ticked CLA acknowledgement line from `.github/PULL_REQUEST_TEMPLATE.md`.
const CLA_ACK_TICKED: &[&str] = &[
    "[x] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
    "[X] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
];

const USAGE: &str =
    "verify|report|reproduce|isolation-check|mutation-matrix|mutation-campaign|bench-compare|package-binaries|ci-evidence|claims|roadmap|crash-series|runtime-examples|source-audit|source-bundle|rust-only|licensing|ci-policy|docs-refs|cla-ack|fmt|clippy|recorder-lint|platform-diff|test|bench|physics";
const CHECKOUT_SHA: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const PROJECT_EMAIL: &str = "gelram.licensing@gmail.com";

// Exact active-license text pin; not evidence of legal approval or authorship.
fn canonical_license(bytes: &[u8]) -> bool {
    gel_source::hex(&gel_source::digest(bytes))
        == "c0b560ffc53ad735c4a6236356ff0cd47d92cf671181e799917975b071e8cd40"
}

// Narrow accidental-contact check, NOT a general secret or email scanner.
// A bare domain suffix in source code is not an email address.
fn has_nonproject_gmail(text: &str) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || ".+-_@".contains(c)))
        .any(|token| {
            let token = token.trim_matches('.');
            let Some((local, domain)) = token.rsplit_once('@') else {
                return false;
            };
            !local.is_empty()
                && domain.eq_ignore_ascii_case("gmail.com")
                && !token.eq_ignore_ascii_case(PROJECT_EMAIL)
        })
}

fn project_contact_privacy(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|e| e.to_string())?;
    for path in files {
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Ok(text) = std::str::from_utf8(&bytes) {
            if has_nonproject_gmail(text) {
                // Do not echo a potentially private address into public CI logs.
                return Err(format!("non-project Gmail address in {}", path.display()));
            }
        }
    }
    println!("PROJECT_CONTACT_GATE=PASS");
    Ok(())
}
const FMT_CHECK_ARGS: &[&str] = &["fmt", "--all", "--", "--check"];
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
];
const TEST_ARGS: &[&str] = &[
    "test",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
];

fn walk(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x == "target" || x == ".git")
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            out.push(path);
        } else if metadata.is_dir() {
            walk(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn workspace_root() -> Result<&'static Path, String> {
    // Resolve the source checkout embedded by this build, not the caller's cwd.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "missing workspace root".into())
}

fn rust_only_at(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|e| e.to_string())?;
    let mut bad = Vec::new();
    for path in files {
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            bad.push(format!("symlink is not allowed: {}", path.display()));
            continue;
        }
        #[cfg(unix)]
        if metadata.permissions().mode() & 0o111 != 0 {
            bad.push(format!(
                "executable file is not allowed in the release tree: {}",
                path.display()
            ));
            continue;
        }
        // Exactly two reviewed, static vector illustrations; not a general
        // SVG or executable-asset exception. Implementation remains Rust-only.
        let documentation_image = [
            "docs/images/public-readout.svg",
            "docs/images/q8-r2-results.svg",
        ]
        .iter()
        .any(|p| path == root.join(p));
        let approved_png = REVIEWED_ASSETS
            .iter()
            .find(|(name, _)| path == root.join(name));
        if let Some((_, expected)) = approved_png {
            if !metadata.is_file() || metadata.len() > 4 * 1024 * 1024 {
                bad.push(format!(
                    "invalid reviewed asset size/type: {}",
                    path.display()
                ));
                continue;
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if gel_source::hex(&gel_source::digest(&bytes)) != *expected {
                bad.push(format!(
                    "reviewed asset fingerprint mismatch: {}",
                    path.display()
                ));
                continue;
            }
        }
        // Only these reviewed text evidence paths, not a general CSV/sha256 exception.
        // Their exact bytes are pinned by the enclosing source manifest.
        let collection_evidence = [
            "docs/evidence-collection/MEASURED-SOURCES.sha256",
            "docs/evidence-collection/r1/raw.csv",
            "docs/evidence-collection/r1/summary.csv",
            "docs/evidence-gel-components/run-1.csv",
            "docs/evidence-gel-components/run-2.csv",
            "docs/evidence-gel-components/run-3.csv",
        ]
        .iter()
        .any(|p| path == root.join(p));
        if collection_evidence {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.len() > 1024 * 1024
                || std::str::from_utf8(&bytes).is_err()
                || bytes.iter().any(|b| *b == 0 || *b == 27)
            {
                bad.push(format!(
                    "invalid collection text evidence: {}",
                    path.display()
                ));
                continue;
            }
        }
        let allowed = collection_evidence
            || documentation_image
            || approved_png.is_some()
            || path
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|ext| ALLOWED_EXTENSIONS.contains(&ext))
            || path
                .file_name()
                .and_then(|x| x.to_str())
                .is_some_and(|name| ALLOWED_EXTENSIONLESS.contains(&name));
        if !allowed {
            bad.push(format!(
                "file type is outside the release allow-list: {}",
                path.display()
            ));
        }
    }
    bad.sort();
    bad.dedup();
    if bad.is_empty() {
        println!("RUST_ONLY_GATE=PASS");
        Ok(())
    } else {
        for message in bad {
            eprintln!("{message}");
        }
        Err("RUST_ONLY_GATE=FAIL".into())
    }
}

fn disclosure() -> Result<(), String> {
    let root = workspace_root()?;
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|e| e.to_string())?;
    disclosure::check(root, &files)
}

fn rust_only() -> Result<(), String> {
    rust_only_at(workspace_root()?)
}

fn require(text: &str, needle: &str, file: &str) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(format!("{file} is missing required text: {needle}"))
    }
}

// Technical tests must run before publication approval; PASS is not permission.
fn publication_status(text: &str) -> Result<bool, String> {
    let flag = |key: &str| -> Result<bool, String> {
        let values: Vec<_> = text
            .lines()
            .filter_map(|line| {
                let (name, value) = line.trim().split_once('=')?;
                (name == key).then_some(value)
            })
            .collect();
        match values.as_slice() {
            ["YES"] => Ok(true),
            ["NO"] => Ok(false),
            _ => Err(format!("missing, duplicated or invalid {key}")),
        }
    };
    let review = flag("REVIEW_PUBLICATION_APPROVED")?;
    let publication = flag("PUBLICATION_APPROVED")?;
    if publication && !review {
        return Err("publication cannot precede scope review".into());
    }
    Ok(publication)
}

fn licensing() -> Result<(), String> {
    let root = workspace_root()?;
    let read = |name: &str| fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"));
    let mode = read("docs/LICENSE-MODE.txt")?;
    let license = read("LICENSE")?;
    let notice = read("NOTICE")?;
    let commercial = read("COMMERCIAL-LICENSE.md")?;
    let cla = read("CLA.md")?;
    licensing_scope(&read("LICENSING.md")?, &notice)?;
    let fixture_license = fs::read(root.join("crates/gel-source/fixtures/rust-book/LICENSE.txt"))
        .map_err(|e| e.to_string())?;
    if gel_source::hex(&gel_source::digest(&fixture_license))
        != "0621878e61f0d0fda054bcbe02df75192c28bde1ecc8289cbd86aeba2dd72720"
    {
        return Err("reviewed third-party fixture license is missing or changed".into());
    }
    license_metadata::check(root)?;
    if !canonical_license(license.as_bytes()) {
        return Err("LICENSE differs from the pinned active NCRL bytes".into());
    }
    match mode.trim() {
        "GEL-RAM-NCRL-1.0 + Commercial + CLA-2.0" => {
            require(
                &license,
                "GEL RAM Noncommercial Reciprocal License 1.0",
                "LICENSE",
            )?;
            require(
                &notice,
                "Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)",
                "NOTICE",
            )?;
        }
        other => return Err(format!("unsupported docs/LICENSE-MODE.txt value: {other}")),
    }
    require(
        &commercial,
        "not itself a commercial license grant",
        "COMMERCIAL-LICENSE.md",
    )?;
    for text in [
        "Contributor License Agreement 2.0",
        "right to license or relicense",
        "Patent license",
        "legal authority",
        "third-party material",
        "AI system",
        "completed Agreement is on file",
        "Pin the exact accepted agreement text",
        "public pseudonym",
        "CLA-PRIVACY.md",
    ] {
        require(&cla, text, "CLA.md")?;
    }
    require(
        &read("CLA-PRIVACY.md")?,
        "RR is a public pseudonym",
        "CLA-PRIVACY.md",
    )?;
    let publication = publication_status(&read("CANDIDATE-STATUS.md")?)?;
    println!(
        "PUBLICATION_APPROVED={}",
        if publication { "YES" } else { "NO" }
    );
    require(
        &read("CANDIDATE-STATUS.md")?,
        "LEGAL_APPROVED=NO",
        "CANDIDATE-STATUS.md",
    )?;
    project_contact_privacy(root)?;
    println!("LICENSING_GATE=PASS");
    println!("LICENSE_MODE={}", mode.trim());
    println!("LICENSE_CHECK_SCOPE=ACTIVE_LICENSE_CONSISTENCY_NOT_LEGAL_CERTIFICATION");
    Ok(())
}

fn licensing_scope(guide: &str, notice: &str) -> Result<(), String> {
    require(
        guide,
        "Identified third-party material remains under its own license terms",
        "LICENSING.md",
    )?;
    require(guide, "remains MIT-licensed", "LICENSING.md")?;
    require(notice, "The Rust Book excerpt", "NOTICE")?;
    require(notice, "distributed under MIT", "NOTICE")?;
    let normalized = guide
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    for obsolete in [
        "everything in this distribution is",
        "no other public license is granted for this tree",
        "this tree is offered under polyform noncommercial 1.0.0 only",
    ] {
        if normalized.contains(obsolete) {
            return Err(
                "overbroad whole-tree licensing claim conflicts with third-party material".into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod licensing_scope_tests {
    const GUIDE: &str = include_str!("../../LICENSING.md");
    const NOTICE: &str = include_str!("../../NOTICE");
    #[test]
    fn root_third_party_scope_and_notices() {
        super::licensing_scope(GUIDE, NOTICE).unwrap();
        assert!(
            super::licensing_scope(&GUIDE.replace("remains MIT-licensed", "omitted"), NOTICE)
                .is_err()
        );
        assert!(
            super::licensing_scope(GUIDE, &NOTICE.replace("distributed under MIT", "omitted"))
                .is_err()
        );
    }
    #[test]
    fn appended_overbroad_claim_is_rejected() {
        for claim in [
            "Everything in this distribution is ours.",
            "No other public license is granted for this tree.",
            "This tree is offered under PolyForm Noncommercial 1.0.0 only.",
        ] {
            assert!(super::licensing_scope(&format!("{GUIDE}\n{claim}"), NOTICE).is_err());
        }
    }
}

fn ci_policy() -> Result<(), String> {
    let root = workspace_root()?;
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .map_err(|e| format!(".github/workflows/ci.yml: {e}"))?;
    let cla = fs::read_to_string(root.join(".github/workflows/cla.yml"))
        .map_err(|e| format!(".github/workflows/cla.yml: {e}"))?;
    require(&ci, CHECKOUT_SHA, ".github/workflows/ci.yml")?;
    require(
        &ci,
        "persist-credentials: false",
        ".github/workflows/ci.yml",
    )?;
    require(&ci, "contents: read", ".github/workflows/ci.yml")?;
    require(&cla, "pull_request_target:", ".github/workflows/cla.yml")?;
    require(&cla, "permissions: {}", ".github/workflows/cla.yml")?;
    require(
        &cla,
        "github.event.pull_request.author_association",
        ".github/workflows/cla.yml",
    )?;
    require(
        &cla,
        "repository owner is the Project Licensor",
        ".github/workflows/cla.yml",
    )?;
    require(&cla, "grep -Fqx", ".github/workflows/cla.yml")?;
    if cla.contains("actions/checkout") || cla.contains("cargo run") {
        return Err(
            ".github/workflows/cla.yml must not check out or execute pull-request code".into(),
        );
    }
    let binaries_path = ".github/workflows/binaries.yml";
    let binaries = fs::read_to_string(root.join(binaries_path))
        .map_err(|e| format!("{binaries_path}: {e}"))?;
    require(&binaries, CHECKOUT_SHA, binaries_path)?;
    require(&binaries, "persist-credentials: false", binaries_path)?;
    require(&binaries, "contents: read", binaries_path)?;
    // Only the hand-started release job may request a signing identity.
    if binaries.matches("id-token: write").count() != 1
        || !binaries.contains("if: github.event_name == 'workflow_dispatch'\n")
    {
        return Err(format!(
            "{binaries_path}: id-token must be granted once, to the workflow_dispatch job"
        ));
    }
    // Every workflow pins its actions to commits. Only the hand-started release
    // job of the binaries workflow may hold a write permission; no workflow
    // writes to the repository itself.
    let dir = root.join(".github/workflows");
    let workflows = audit_io::paths(&dir)?;
    for path in &workflows {
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file()
            || !matches!(
                path.extension().and_then(OsStr::to_str),
                Some("yml" | "yaml")
            )
        {
            return Err("workflow inventory contains a non-regular or unsupported entry".into());
        }
        let file = format!(
            ".github/workflows/{}",
            path.file_name()
                .and_then(OsStr::to_str)
                .ok_or("non-UTF8 workflow name")?
        );
        let text = fs::read_to_string(path).map_err(|e| format!("{file}: {e}"))?;
        workflow_policy::check(&text, &file)?;
    }
    println!("CI_POLICY_GATE=PASS workflows={}", workflows.len());
    Ok(())
}

/// Width of a code-fence line (three or more leading backticks), if `line` is one.
fn fence_width(line: &str) -> Option<usize> {
    let width = line.trim_start().bytes().take_while(|b| *b == b'`').count();
    (width >= 3).then_some(width)
}

/// Contents of the inline code spans in one Markdown line.
///
/// A span opens with a run of backticks and closes with the next run of the
/// same width; a run without a closer is literal text and scanning resumes
/// after it. Backticks are ASCII, so every slice boundary is a char boundary.
fn code_spans(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let open = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let width = i - open;
        let start = i;
        let mut close = None;
        while i < bytes.len() {
            if bytes[i] != b'`' {
                i += 1;
                continue;
            }
            let run = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            if i - run == width {
                close = Some(run);
                break;
            }
        }
        match close {
            Some(end) => spans.push(&line[start..end]),
            None => i = start,
        }
    }
    spans
}

/// One space of padding on both sides of a span is not part of its content.
fn strip_span_padding(span: &str) -> &str {
    if span.len() >= 2 && span.starts_with(' ') && span.ends_with(' ') && !span.trim().is_empty() {
        &span[1..span.len() - 1]
    } else {
        span
    }
}

/// Whether a code-span token is shaped like a repository path that must exist.
fn is_reference(token: &str) -> bool {
    if token.is_empty()
        || token
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '*' | '{' | '$'))
    {
        return false;
    }
    if ALLOWED_EXTENSIONLESS.contains(&token) {
        return true;
    }
    if REFERENCE_PREFIXES
        .iter()
        .any(|prefix| token.starts_with(prefix))
    {
        return true;
    }
    if !token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
    {
        return false;
    }
    token
        .rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && REFERENCE_EXTENSIONS.contains(&ext))
}

/// Repository path citations in a Markdown document as `(line, token)`, lines 1-based.
/// Fenced code blocks are not scanned.
fn reference_tokens(text: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut fence = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(open) = fence {
            let closes = fence_width(line).is_some_and(|width| width >= open)
                && line.trim().trim_start_matches('`').trim().is_empty();
            if closes {
                fence = None;
            }
            continue;
        }
        if let Some(width) = fence_width(line) {
            fence = Some(width);
            continue;
        }
        for span in code_spans(line) {
            let token = strip_span_padding(span);
            if is_reference(token) {
                found.push((index + 1, token));
            }
        }
    }
    found
}

fn docs_refs() -> Result<(), String> {
    let root = workspace_root()?;
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|e| e.to_string())?;
    files.sort();
    let mut checked = 0usize;
    let mut missing = Vec::new();
    for path in files
        .iter()
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("md"))
    {
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let citing = path.strip_prefix(root).unwrap_or(path.as_path());
        for (line, token) in reference_tokens(&text) {
            checked += 1;
            if !root.join(token).exists() {
                missing.push(format!(
                    "{}:{line}: missing repository reference: {token}",
                    citing.display()
                ));
            }
        }
    }
    if checked == 0 {
        return Err("DOCS_REFS_GATE=FAIL: no repository references found in any .md file".into());
    }
    if missing.is_empty() {
        println!("DOCS_REFS_GATE=PASS");
        println!("DOCS_REFS_CHECKED={checked}");
        Ok(())
    } else {
        for line in &missing {
            eprintln!("{line}");
        }
        Err("DOCS_REFS_GATE=FAIL".into())
    }
}

fn cla_acknowledged(body: &str) -> bool {
    // Match the workflow's grep -Fqx contract: one complete checkbox line.
    body.split('\n').any(|line| {
        CLA_ACK_TICKED
            .iter()
            .any(|ack| line.strip_prefix("- ") == Some(*ack))
    })
}

fn cla_ack() -> Result<(), String> {
    let body = std::env::var_os("PR_BODY")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    if cla_acknowledged(&body) {
        println!("CLA_ACK_GATE=PASS");
        Ok(())
    } else {
        eprintln!(
            "PR_BODY is unset or does not contain the ticked CLA acknowledgement line from .github/PULL_REQUEST_TEMPLATE.md."
        );
        eprintln!(
            "A completed CLA must be on file with the project before a pull request is opened; see CLA.md and CONTRIBUTING.md."
        );
        Err("CLA_ACK_GATE=FAIL".into())
    }
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(workspace_root()?);
    process_sequence::sequence(&mut [command])
}

fn runtime_examples() -> Result<(), String> {
    let root = workspace_root()?;
    let specs: &[&[&str]] = &[
        &["-p", "gel-source", "--example", "source_build"],
        &["-p", "gel-live-lab", "--", "--demo"],
        &["-p", "gel-live-lab", "--", "--literal", "README.md"],
        &[
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
        &["-p", "gel-cli", "--example", "quantization_matrix"],
        &["-p", "gel-cli", "--example", "precision_matrix"],
        &["-p", "gel-source", "--example", "collection_review"],
        &["-p", "gel-history", "--example", "record_history"],
    ];
    let mut commands = Vec::new();
    for spec in specs {
        let mut command = Command::new("cargo");
        command
            .args(["run", "--locked", "--offline", "--release"])
            .args(*spec)
            .current_dir(root);
        commands.push(command);
    }
    process_sequence::sequence(&mut commands)?;
    println!("RUNTIME_EXAMPLES=PASS");
    Ok(())
}

fn run_docs() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["doc", "--locked", "--offline", "--workspace", "--no-deps"])
        .env("RUSTDOCFLAGS", "-D warnings")
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo doc failed: {status}"))
    }
}

/// `cargo run --release -p <package> -- <args>` inside the workspace.
fn run_release_binary<S: AsRef<OsStr>>(package: &str, args: &[S]) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args([
        "run",
        "--locked",
        "--offline",
        "--release",
        "-p",
        package,
        "--",
    ]);
    command.args(args);
    let status = command
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{package} failed: {status}"))
    }
}

fn verify() -> Result<(), String> {
    rust_only()?;
    licensing()?;
    ci_policy()?;
    docs_refs()?;
    disclosure()?;
    run("cargo", FMT_CHECK_ARGS)?;
    run("cargo", CLIPPY_ARGS)?;
    recorder_lint::check(workspace_root()?)?;
    run(
        "cargo",
        &["build", "--locked", "--offline", "--release", "--workspace"],
    )?;
    run_docs()?;
    measured_sources::verify(workspace_root()?)?;
    claims::check(workspace_root()?)?;
    answer_bench::check(workspace_root()?)?;
    roadmap::check(workspace_root()?)?;
    if cfg!(unix) {
        let report = crash_series::run(&gel_evidence_binary(workspace_root()?), 5, 20_260_929)?;
        println!("{}", report.lines().last().unwrap_or(""));
    } else {
        println!("CRASH_SERIES=SKIPPED not a Unix host");
    }
    mutation_matrix::check(workspace_root()?)?;
    println!(
        "PROPERTY_MAP_FORMAT=PASS rows={}",
        property_map::read(workspace_root()?)?.len()
    );
    run(
        "cargo",
        &["test", "--locked", "--offline", "--workspace", "--doc"],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-source",
            "--example",
            "collection_recheck",
            "--",
            "docs/evidence-collection/r1",
        ],
    )?;
    run("cargo", TEST_ARGS)?;
    run_release_binary("gel-cli", &["selftest"])?;
    run_release_binary("gel-bench", &["8192", "3", "2"])?;
    run_release_binary("gel-bench", &["8192", "3", "1"])?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-phase-quad",
            "--example",
            "quad_compare",
            "--",
            "--orbs",
            "32",
            "--rounds",
            "3",
            "--workers",
            "2",
            "--sparse",
            "1",
        ],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-phase-quad",
            "--example",
            "quad_evidence",
            "--",
            "--demo",
        ],
    )?;
    for (package, example, args) in [
        ("gel-phase-quad", "quad_playground", vec!["--demo"]),
        ("gel-source", "source_readout", vec![]),
        ("gel-source", "source_parts", vec![]),
        ("gel-source", "source_real", vec![]),
        ("gel-source", "source_find", vec![]),
        ("gel-source", "source_build", vec![]),
        ("gel-source", "collection_review", vec![]),
        ("gel-cli", "quantization_matrix", vec![]),
        ("gel-history", "record_history", vec![]),
    ] {
        let mut command = vec![
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            package,
            "--example",
            example,
            "--",
        ];
        command.extend(args);
        run("cargo", &command)?;
    }
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--",
            "--demo",
        ],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
    )?;
    println!("GEL_VERIFY_ALL=PASS");
    Ok(())
}

fn collect_args() -> Result<Vec<String>, String> {
    std::env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|bad| format!("argument is not valid UTF-8: {}", bad.to_string_lossy()))
        })
        .collect()
}

fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None | Some("verify") => verify(),
        Some("report") => reproduce::report(&args[1..]),
        Some("reproduce") => reproduction::run(&args[1..]),
        Some("isolation-check") => isolation::check(&args[1..]),
        Some("mutation-matrix") => mutation_matrix::run(&args[1..]),
        Some("mutation-campaign") => mutation_campaign::run(&args[1..]),
        Some("answer-bench") => answer_bench::run(&args[1..]),
        Some("bench-compare") => bench_compare::run(&args[1..]),
        Some("package-binaries") => package::run(&args[1..]),
        Some("ci-evidence") => ci_evidence::report(&args[1..]),
        Some("claims") => claims::check(workspace_root()?),
        Some("roadmap") => roadmap::check(workspace_root()?),
        Some("crash-series") => crash_series_cmd(&args[1..]),
        Some("runtime-examples") => runtime_examples(),
        Some("source-audit") => source_bundle::audit(&args[1..]),
        Some("source-bundle") => source_bundle::bundle(&args[1..]),
        Some("rust-only") => rust_only(),
        Some("licensing") => licensing(),
        Some("ci-policy") => ci_policy(),
        Some("docs-refs") => docs_refs(),
        Some("disclosure") => disclosure(),
        Some("cla-ack") => cla_ack(),
        Some("fmt") => run("cargo", FMT_CHECK_ARGS),
        Some("clippy") => run("cargo", CLIPPY_ARGS),
        Some("recorder-lint") => recorder_lint::check(workspace_root()?),
        Some("platform-diff") => ci_evidence::platform_diff(&args[1..]),
        Some("test") => run("cargo", TEST_ARGS),
        Some("bench") => run_release_binary("gel-bench", &args[1..]),
        Some("physics") => run_release_binary("gel-physics", &args[1..]),
        Some(x) => Err(format!("unknown xtask: {x}; use {USAGE}")),
    }
}

fn main() -> ExitCode {
    match collect_args().and_then(|args| dispatch(&args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-component-gate-{}-{nonce}", std::process::id()));
        let dir = root.join("docs/evidence-gel-components");
        fs::create_dir_all(&dir).unwrap();
        for i in 1..=3 {
            let path = dir.join(format!("run-{i}.csv"));
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
            assert!(rust_only_at(&root).is_ok());
            for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
                fs::write(&path, bad).unwrap();
                assert!(rust_only_at(&root).is_err());
            }
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
        }
        fs::write(dir.join("run-4.csv"), b"unreviewed\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn collection_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-evidence-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/evidence-collection/r1")).unwrap();
        let raw = root.join("docs/evidence-collection/r1/raw.csv");
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        assert!(rust_only_at(&root).is_ok());
        for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
            fs::write(&raw, bad).unwrap();
            assert!(rust_only_at(&root).is_err());
        }
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        fs::write(root.join("unreviewed.csv"), b"text\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn documentation_png_gate_requires_exact_reviewed_bytes_and_path() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-image-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/images")).unwrap();
        for (name, _) in REVIEWED_ASSETS {
            let bytes = fs::read(workspace_root().unwrap().join(name)).unwrap();
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(rust_only_at(&root).is_ok());
            let mut changed = bytes.clone();
            changed[100] ^= 1;
            fs::write(&path, &changed).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::write(&path, &bytes).unwrap();
            let unexpected = root.join("docs/images/unreviewed.png");
            fs::write(&unexpected, &bytes).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(unexpected).unwrap();
            fs::write(&path, b"not a PNG").unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_license_rejects_changed_or_appended_bytes() {
        let original = include_bytes!("../../LICENSE");
        assert!(canonical_license(original));
        let mut changed = original.to_vec();
        changed[0] ^= 1;
        assert!(!canonical_license(&changed));
        changed = original.to_vec();
        changed.push(b'\n');
        assert!(!canonical_license(&changed));
    }

    #[test]
    fn contact_gate_distinguishes_source_suffix_from_an_address() {
        assert!(!has_nonproject_gmail("token.ends_with(\"@gmail.com\")"));
        assert!(!has_nonproject_gmail(PROJECT_EMAIL));
        assert!(!has_nonproject_gmail(&format!("<mailto:{PROJECT_EMAIL}>.")));
        assert!(!has_nonproject_gmail(&PROJECT_EMAIL.to_uppercase()));
        for local in ["fixture", "test.user", "test+tag", "test_user"] {
            assert!(has_nonproject_gmail(&format!("`{local}@{}.`", "gmail.com")));
        }
    }

    #[cfg(unix)]
    #[test]
    fn rust_only_gate_rejects_symlinks_and_executable_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-rust-only-gate-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("good.rs");
        fs::write(&source, "fn main() {}\n").unwrap();
        fs::write(
            root.join("CITATION.cff"),
            "cff-version: 1.2.0\ntitle: \"fixture\"\n",
        )
        .unwrap();
        assert!(rust_only_at(&root).is_ok());

        let link = root.join("link.rs");
        symlink("good.rs", &link).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_file(&link).unwrap();

        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn code_spans_follow_backtick_run_width() {
        assert_eq!(
            code_spans("see `a.md` and ``b `x` c`` here"),
            vec!["a.md", "b `x` c"]
        );
        assert_eq!(code_spans("`` unmatched ` one.md `"), vec![" one.md "]);
        assert_eq!(code_spans("no spans ` here"), Vec::<&str>::new());
        assert_eq!(strip_span_padding(" one.md "), "one.md");
        assert_eq!(strip_span_padding("  "), "  ");
    }

    #[test]
    fn cla_ack_requires_the_exact_workflow_line() {
        for ack in CLA_ACK_TICKED {
            let line = format!("- {ack}");
            assert!(cla_acknowledged(&format!("Context\n{line}\n")));
            for altered in [
                format!("quote {line}"),
                format!("{line} not true"),
                format!("{line}\r"),
                ack.to_string(),
            ] {
                assert!(!cla_acknowledged(&altered));
            }
        }
        assert!(!cla_acknowledged(""));
    }

    #[test]
    fn fenced_blocks_are_not_scanned() {
        let text = "```text\n`docs/inside.md`\n```\n`docs/outside.md`\n````\n```\n`docs/still-inside.md`\n````\n`CLA.md`\n";
        assert_eq!(
            reference_tokens(text),
            vec![(4, "docs/outside.md"), (9, "CLA.md")]
        );
    }

    #[test]
    fn reference_shape_matches_repository_paths_only() {
        for yes in [
            "CLA.md",
            "docs/",
            ".github/workflows/ci.yml",
            "crates/gel-core/src/lib.rs",
            "xtask/Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "LICENSE-MODE.txt",
            "LICENSE",
            "NOTICE",
            ".gitignore",
        ] {
            assert!(is_reference(yes), "{yes}");
        }
        for no in [
            "",
            ".gel",
            "xtask",
            "gel-bench",
            "u64::count_ones()",
            "gelram.licensing@gmail.com",
            "<ORB_COUNT> <ROUNDS> [THREADS]",
            "docs/<name>.md",
            "record_count * 128",
            "GEL_BENCH_V3",
            "unsafe_code = \"forbid\"",
            "$HOME/x.md",
            "{root}/x.md",
            "x86_64-unknown-linux-gnu",
        ] {
            assert!(!is_reference(no), "{no}");
        }
    }

    #[test]
    fn cla_ack_needles_match_the_template_line() {
        for needle in CLA_ACK_TICKED {
            assert!(needle.starts_with("[x] ") || needle.starts_with("[X] "));
            assert!(needle.ends_with(" before opening this pull request."));
        }
        assert_eq!(CLA_ACK_TICKED[0][4..], CLA_ACK_TICKED[1][4..]);
    }
}

fn gel_evidence_binary(root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("release")
        .join(format!("gel-evidence{}", std::env::consts::EXE_SUFFIX))
}

/// `crash-series [TRIALS] [SEED]`: builds the release `gel-evidence` and prints the full report.
fn crash_series_cmd(args: &[String]) -> Result<(), String> {
    let number = |i: usize, default: u64| -> Result<u64, String> {
        args.get(i)
            .map_or(Ok(default), |s| s.parse().map_err(|e| format!("{s}: {e}")))
    };
    let (trials, seed) = (number(0, 200)? as usize, number(1, 20_260_929)?);
    run(
        "cargo",
        &[
            "build",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
        ],
    )?;
    print!(
        "{}",
        crash_series::run(&gel_evidence_binary(workspace_root()?), trials, seed)?
    );
    Ok(())
}
