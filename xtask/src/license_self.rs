// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! `verify` step: the release `gel-license check` on this workspace itself
//! (docs/LICENSE-DECISION.md). The verdict rules are in
//! crates/gel-live-lab/src/license_check.rs; this step only reads the printed
//! lines, each as a whole line, and requires the workspace's own state.
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// The source headers are in every GEL RAM-owned `.rs` file (the source-header
/// gate of `verify`), so at least one header file is required, every header
/// must be intact, and the check must count as many intact headers as the gate
/// passed files. Set to `false` only for a tree without headers.
const REQUIRE_HEADERS: bool = true;

fn release_binary(root: &Path, name: &str) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("release")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// Runs the release `gel-license check ROOT` (built by the verify release build);
/// `gate_files` is the file count of the passed source-header gate.
pub fn check(root: &Path, gate_files: usize) -> Result<(), String> {
    let bin = release_binary(root, "gel-license");
    let out = Command::new(&bin)
        .arg("check")
        .arg(root)
        .output()
        .map_err(|e| format!("{}: {e}", bin.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    print!("{stdout}");
    let require = REQUIRE_HEADERS.then_some(gate_files);
    println!("{}", judge(&stdout, out.status.code(), require)?);
    Ok(())
}

/// The value of the one line `KEY=value`; a missing or repeated key is an error.
fn value<'a>(stdout: &'a str, key: &str) -> Result<&'a str, String> {
    let found: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
        .collect();
    match found.as_slice() {
        [one] => Ok(one),
        _ => Err(format!(
            "LICENSE_SELF_CHECK=FAIL {key} printed {} times",
            found.len()
        )),
    }
}

/// A number field `KEY=n` of the header line.
fn header_field(stdout: &str, key: &str) -> Result<usize, String> {
    let line = value(stdout, "GEL_HEADER_FILES")?;
    let full = format!("GEL_HEADER_FILES={line}");
    full.split(' ')
        .find_map(|field| field.strip_prefix(key)?.strip_prefix('='))
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| format!("LICENSE_SELF_CHECK=FAIL no {key} on the header line"))
}

/// The verdict of the step on the printed report and exit code.
/// `require_headers` is `Some(n)` when headers are required: then at least one
/// header file, and exactly `n` intact headers (the source-header gate's count).
fn judge(
    stdout: &str,
    code: Option<i32>,
    require_headers: Option<usize>,
) -> Result<String, String> {
    let need = |key: &str, want: &str| -> Result<(), String> {
        let got = value(stdout, key)?;
        if got == want {
            Ok(())
        } else {
            Err(format!(
                "LICENSE_SELF_CHECK=FAIL {key}={got}, required {want}"
            ))
        }
    };
    need("LICENSE_FILE", "EXACT")?;
    need("REQUIRED_NOTICE", "KEPT")?;
    // This workspace holds gel-source and its Unicode tables, so the
    // third-party notices are needed here and must be kept, not NOT_NEEDED.
    need("THIRD_PARTY_NOTICES", "KEPT")?;
    need("DISTRIBUTION_NOTICES", "LISTED_ITEMS_PRESENT")?;
    need("CONTACT", "gelram.licensing@gmail.com")?;
    need("LEGAL_CERTIFICATION", "NO")?;
    if value(stdout, "GEL_CRATES")? == "none" {
        return Err("LICENSE_SELF_CHECK=FAIL no GEL crate found in the workspace".into());
    }
    let files = header_field(stdout, "GEL_HEADER_FILES")?;
    let intact = header_field(stdout, "HEADER_INTACT")?;
    let altered = header_field(stdout, "HEADER_ALTERED")?;
    if altered != 0 || intact + altered != files {
        return Err(format!(
            "LICENSE_SELF_CHECK=FAIL headers files={files} intact={intact} altered={altered}"
        ));
    }
    if code != Some(0) {
        return Err(format!("LICENSE_SELF_CHECK=FAIL exit {code:?}"));
    }
    if let Some(gate_files) = require_headers {
        if files == 0 {
            return Err("LICENSE_SELF_CHECK=FAIL no source file carries the header".into());
        }
        if intact != gate_files {
            return Err(format!(
                "LICENSE_SELF_CHECK=FAIL intact={intact}, the source-header gate passed files={gate_files}"
            ));
        }
        Ok(format!(
            "LICENSE_SELF_CHECK=PASS distribution=LISTED_ITEMS_PRESENT headers=required intact={intact} gate_files={gate_files}"
        ))
    } else {
        Ok(format!(
            "LICENSE_SELF_CHECK=PASS distribution=LISTED_ITEMS_PRESENT headers=not-required-yet header_files={files}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::judge;

    fn report(license: &str, headers: &str, crates: &str, distribution: &str) -> String {
        format!(
            "LICENSE_FILE={license}\nREQUIRED_NOTICE=KEPT\nTHIRD_PARTY_NOTICES=KEPT\nGEL_HEADER_FILES={headers}\nGEL_CRATES={crates}\nSCAN_FILES=9 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0\nDISTRIBUTION_NOTICES={distribution}\nNOTE=this reads only the items listed above\nCONTACT=gelram.licensing@gmail.com\nLEGAL_CERTIFICATION=NO\n"
        )
    }
    const NO_HEADERS: &str = "0 HEADER_INTACT=0 HEADER_ALTERED=0";
    const ALL_INTACT: &str = "5 HEADER_INTACT=5 HEADER_ALTERED=0";
    const PRESENT: &str = "LISTED_ITEMS_PRESENT";
    const MISSING: &str = "LISTED_ITEMS_MISSING";

    #[test]
    fn the_workspace_without_headers_passes_until_headers_are_required() {
        let r = report("EXACT", NO_HEADERS, "gel-core", PRESENT);
        assert!(judge(&r, Some(0), None)
            .unwrap()
            .contains("not-required-yet"));
        assert!(judge(&r, Some(0), Some(0)).is_err());
    }

    #[test]
    fn intact_headers_pass_with_headers_required() {
        let r = report("EXACT", ALL_INTACT, "gel-core", PRESENT);
        assert_eq!(
            judge(&r, Some(0), Some(5)).unwrap(),
            "LICENSE_SELF_CHECK=PASS distribution=LISTED_ITEMS_PRESENT headers=required intact=5 gate_files=5"
        );
    }

    #[test]
    fn intact_headers_must_be_as_many_as_the_gate_passed() {
        let r = report("EXACT", ALL_INTACT, "gel-core", PRESENT);
        for gate_files in [0, 4, 6] {
            let error = judge(&r, Some(0), Some(gate_files)).unwrap_err();
            assert!(error.contains("intact=5"), "{error}");
        }
    }

    #[test]
    fn every_other_state_fails() {
        let good = report("EXACT", NO_HEADERS, "gel-core", PRESENT);
        let bad = [
            report("CHANGED", NO_HEADERS, "gel-core", MISSING),
            report("CHANGED", NO_HEADERS, "gel-core", PRESENT),
            report(
                "EXACT",
                "5 HEADER_INTACT=4 HEADER_ALTERED=1",
                "gel-core",
                PRESENT,
            ),
            report("EXACT", NO_HEADERS, "none", PRESENT),
            report("EXACT", NO_HEADERS, "gel-core", MISSING),
            // The value names of before this release are not accepted.
            report("EXACT", NO_HEADERS, "gel-core", "COMPLETE"),
            good.replace("REQUIRED_NOTICE=KEPT", "REQUIRED_NOTICE=MISSING"),
            // The workspace needs its third-party notices: neither missing
            // nor "not needed" passes, and the line must be printed.
            good.replace("THIRD_PARTY_NOTICES=KEPT", "THIRD_PARTY_NOTICES=MISSING"),
            good.replace("THIRD_PARTY_NOTICES=KEPT", "THIRD_PARTY_NOTICES=NOT_NEEDED"),
            good.replace("THIRD_PARTY_NOTICES=KEPT\n", ""),
            good.replace("LEGAL_CERTIFICATION=NO\n", ""),
            good.replace("CONTACT=gelram.licensing@gmail.com\n", ""),
            good.clone() + "LICENSE_FILE=EXACT\n",
            // A value is read from its own line, not from a longer one.
            good.replace("LICENSE_FILE=EXACT", "LICENSE_FILE=EXACTLY"),
        ];
        for (i, r) in bad.iter().enumerate() {
            for require in [None, Some(0), Some(5)] {
                assert!(judge(r, Some(0), require).is_err(), "case {i}");
            }
        }
        assert!(judge(&good, Some(3), None).is_err());
        assert!(judge(&good, None, None).is_err());
    }
}
