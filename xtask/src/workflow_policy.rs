//! Fail-closed policy for the repository's deliberately restricted YAML profile.
//! This is not a general YAML parser: aliases, tags, quoted keys, flow-style
//! jobs/permissions, multiline keys and duplicate mappings require review.
//! Literal/folded script bodies are data, not permission or action declarations.
use std::collections::BTreeSet;

#[derive(Debug)]
struct Row {
    path: Vec<String>,
    value: String,
    line: usize,
}

fn uncomment(text: &str) -> Result<String, String> {
    let (mut single, mut double, mut escaped) = (false, false, false);
    let mut out = String::new();
    let mut previous_space = true;
    for c in text.chars() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        if double && c == '\\' {
            escaped = true;
        } else if c == '\'' && !double {
            single = !single;
        } else if c == '"' && !single {
            double = !double;
        } else if c == '#' && !single && !double && previous_space {
            break;
        }
        out.push(c);
        previous_space = c.is_whitespace();
    }
    if single || double || escaped {
        return Err("multiline or unbalanced quoted YAML is outside the policy profile".into());
    }
    Ok(out.trim_end().to_string())
}

fn rows(text: &str) -> Result<Vec<Row>, String> {
    if text.len() > 128 * 1024 || text.contains('\0') || text.lines().count() > 4096 {
        return Err("workflow size or encoding limit".into());
    }
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut block = None;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let indent = raw.bytes().take_while(|b| *b == b' ').count();
        if block.is_some_and(|at| raw.trim().is_empty() || indent > at) {
            continue;
        }
        block = None;
        let line = uncomment(raw)?;
        let body = line.trim();
        if body.is_empty() {
            continue;
        }
        if raw[..raw.len() - raw.trim_start().len()].contains('\t') || indent % 2 != 0 {
            return Err(format!(
                "line {}: use spaces and even indentation",
                index + 1
            ));
        }
        while stack.last().is_some_and(|(at, _)| *at >= indent) {
            stack.pop();
        }
        let (body, key_indent) = if let Some(body) = body.strip_prefix("- ") {
            stack.push((indent, format!("[{}]", index + 1)));
            if !body.contains(':') {
                // A sequence scalar cannot introduce a mapping. Anchored or
                // aliased sequence nodes are intentionally unsupported.
                if body.starts_with(['&', '*', '!', '{']) {
                    return Err("noncanonical sequence node".into());
                }
                continue;
            }
            (body, indent + 2)
        } else {
            (body, indent)
        };
        let (key, value) = body
            .split_once(':')
            .ok_or_else(|| format!("line {}: expected canonical YAML mapping", index + 1))?;
        let key = key.trim();
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        {
            return Err(format!("line {}: noncanonical YAML key", index + 1));
        }
        let value = value.trim();
        if value.starts_with(['&', '*', '!']) {
            return Err(format!(
                "line {}: YAML anchors, aliases and tags are not supported",
                index + 1
            ));
        }
        let mut path: Vec<String> = stack.iter().map(|(_, k)| k.clone()).collect();
        path.push(key.to_string());
        if path.len() > 32 || !seen.insert(path.clone()) {
            return Err(format!(
                "line {}: duplicate mapping or excessive nesting",
                index + 1
            ));
        }
        if (path == ["jobs"] || path.len() == 2 && path[0] == "jobs" || key == "steps")
            && !value.is_empty()
        {
            return Err(format!(
                "line {}: jobs and steps require block structure",
                index + 1
            ));
        }
        if key == "permissions" && !matches!(value, "" | "{}") {
            return Err(format!(
                "line {}: permissions require a block map or empty map",
                index + 1
            ));
        }
        out.push(Row {
            path,
            value: value.to_string(),
            line: index + 1,
        });
        if matches!(value, "|" | "|-" | "|+" | ">" | ">-" | ">+") {
            block = Some(key_indent);
        } else if value.starts_with(['|', '>']) {
            return Err(format!("line {}: unsupported scalar indicator", index + 1));
        }
        stack.push((key_indent, key.to_string()));
    }
    Ok(out)
}

fn scalar(value: &str) -> &str {
    for quote in ['\'', '"'] {
        if let Some(v) = value
            .strip_prefix(quote)
            .and_then(|v| v.strip_suffix(quote))
        {
            return v;
        }
    }
    value
}

pub fn check(text: &str, file: &str) -> Result<(), String> {
    let rows = rows(text).map_err(|e| format!("{file}: {e}"))?;
    if !rows.iter().any(|r| r.path == ["permissions"]) || !rows.iter().any(|r| r.path == ["jobs"]) {
        return Err(format!(
            "{file}: explicit root permissions and jobs are required"
        ));
    }
    for row in &rows {
        let key = row.path.last().map(String::as_str).unwrap_or("");
        if key == "permissions" {
            let root = row.path.len() == 1;
            let job = row.path.len() == 3 && row.path[0] == "jobs";
            if !root && !job {
                return Err(format!("{file}:{} misplaced permissions", row.line));
            }
            let children: Vec<_> = rows
                .iter()
                .filter(|r| r.path.len() == row.path.len() + 1 && r.path.starts_with(&row.path))
                .collect();
            if row.value.is_empty() && children.is_empty() {
                return Err(format!("{file}:{} empty implicit permissions", row.line));
            }
            if row.value == "{}" && !children.is_empty() {
                return Err(format!(
                    "{file}:{} children of an inline empty map",
                    row.line
                ));
            }
            let release = file == ".github/workflows/binaries.yml"
                && row.path == ["jobs", "release", "permissions"]
                && rows.iter().any(|r| {
                    r.path == ["jobs", "release", "if"]
                        && scalar(&r.value) == "github.event_name == 'workflow_dispatch'"
                });
            for child in children {
                let permission = child.path.last().unwrap().as_str();
                if ![
                    "actions",
                    "attestations",
                    "checks",
                    "contents",
                    "deployments",
                    "discussions",
                    "id-token",
                    "issues",
                    "models",
                    "packages",
                    "pages",
                    "pull-requests",
                    "repository-projects",
                    "security-events",
                    "statuses",
                ]
                .contains(&permission)
                {
                    return Err(format!("{file}:{} unknown permission key", child.line));
                }
                match scalar(&child.value) {
                    "read" | "none" => {}
                    "write" if release && matches!(permission, "id-token" | "attestations") => {}
                    _ => return Err(format!("{file}:{} unapproved permission grant", child.line)),
                }
            }
        }
        if key == "uses" {
            let action = scalar(&row.value);
            let pinned = action.rsplit_once('@').is_some_and(|(name, sha)| {
                !name.is_empty()
                    && sha.len() == 40
                    && sha
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            });
            if !pinned {
                return Err(format!(
                    "{file}:{} action must use a full lowercase commit SHA",
                    row.line
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SIMPLE: &str = "name: check\non:\n  push:\npermissions:\n  contents: read\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n";
    const BINARY: &str = "name: binaries\non:\n  workflow_dispatch:\npermissions:\n  contents: read\njobs:\n  release:\n    if: github.event_name == 'workflow_dispatch'\n    permissions:\n      contents: read\n      id-token: write\n      attestations: write\n    steps:\n      - run: echo ok\n";
    const FILE: &str = ".github/workflows/ci.yml";
    #[test]
    fn all_current_workflows_meet_policy() {
        for (name, text) in [
            (FILE, include_str!("../../.github/workflows/ci.yml")),
            (
                ".github/workflows/cla.yml",
                include_str!("../../.github/workflows/cla.yml"),
            ),
            (
                ".github/workflows/binaries.yml",
                include_str!("../../.github/workflows/binaries.yml"),
            ),
            (
                ".github/workflows/readme-presentation.yml",
                include_str!("../../.github/workflows/readme-presentation.yml"),
            ),
        ] {
            check(text, name).unwrap();
        }
    }
    #[test]
    fn quoted_spaced_escaped_and_multiline_writes_are_rejected() {
        for grant in [
            "write",
            "'write'",
            "\"write\"",
            "   write",
            "\"\\x77rite\"",
            ">-\n    write",
            "|\n    write",
            "&rw write",
            "*rw",
            "!!str write",
        ] {
            assert!(
                check(
                    &SIMPLE.replace("contents: read", &format!("contents: {grant}")),
                    FILE
                )
                .is_err(),
                "{grant}"
            );
        }
    }
    #[test]
    fn shorthand_flow_quoted_keys_and_duplicate_maps_fail_closed() {
        for map in [
            "permissions: write-all",
            "permissions: 'write-all'",
            "permissions: \"write-all\"",
            "permissions: {contents: write}",
            "'permissions':\n  contents: read",
            "permissions: {}\npermissions:\n  contents: read",
            "permissions:\n  contents: read\n  contents: read",
            "permissions: &rw\n  contents: read",
        ] {
            assert!(
                check(&SIMPLE.replace("permissions:\n  contents: read", map), FILE).is_err(),
                "{map}"
            );
        }
        assert!(check(
            &SIMPLE.replace(
                "jobs:\n  check:",
                "jobs: {check: {permissions: {contents: write}}}\n  check:"
            ),
            FILE
        )
        .is_err());
    }
    #[test]
    fn release_permission_is_bound_to_actual_job_and_dispatch_guard() {
        let file = ".github/workflows/binaries.yml";
        check(BINARY, file).unwrap();
        assert!(check(BINARY, FILE).is_err());
        assert!(check(&BINARY.replace("  release:", "  check:"), file).is_err());
        assert!(check(&BINARY.replace("    if:", "    # if:"), file).is_err());
        assert!(check(
            &BINARY.replace("github.event_name == 'workflow_dispatch'", "always()"),
            file
        )
        .is_err());
        assert!(check(
            &BINARY.replace(
                "permissions:\n  contents: read",
                "permissions:\n  contents: read\n  id-token: write"
            ),
            file
        )
        .is_err());
        assert!(check(
            &BINARY.replace("attestations: write", "contents: write"),
            file
        )
        .is_err());
    }
    #[test]
    fn scripts_and_comments_cannot_spoof_or_invalidate_policy() {
        let text = SIMPLE.replace("run: echo ok", "run: |\n          echo 'contents: write'\n          echo 'uses: something@main'\n          echo \"permissions: write-all\"");
        check(&text, FILE).unwrap();
        assert!(check(
            &SIMPLE.replace("permissions:\n  contents: read\n", "# permissions: {}\n"),
            FILE
        )
        .is_err());
    }
    #[test]
    fn action_versions_are_parsed_as_declarations_not_comments() {
        let at = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
        check(
            &SIMPLE.replace("run: echo ok", &format!("uses: '{at}' # pinned")),
            FILE,
        )
        .unwrap();
        assert!(check(
            &SIMPLE.replace("run: echo ok", "uses: actions/checkout@main"),
            FILE
        )
        .is_err());
        assert!(check(
            &SIMPLE.replace("run: echo ok", "'uses': actions/checkout@main"),
            FILE
        )
        .is_err());
    }
}
