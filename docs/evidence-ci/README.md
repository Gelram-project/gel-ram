# Recorded CI evidence

GitHub Actions artifacts and, after the repository's retention period, the
workflow runs and logs themselves expire, and artifacts can be downloaded only
by signed-in users. The sanitized per-platform reports produced by
`xtask ci-evidence` are therefore also kept here, for selected commits.

Each directory is named after the tested commit and holds, per platform, the
REPORT and TESTS files printed by the collector in that job's log and the
collector's own checksum list. The files were rebuilt from the log block
between `CI_EVIDENCE_REPORT_BEGIN` and `CI_EVIDENCE_REPORT_END`; the checksums
printed in CI match the rebuilt files. Check them with:

```sh
cd docs/evidence-ci/2b29236 && sha256sum -c linux-SHA256SUMS.txt windows-SHA256SUMS.txt macos-SHA256SUMS.txt
cargo run --locked --offline -p xtask -- platform-diff docs/evidence-ci/2b29236/linux-TESTS.txt docs/evidence-ci/2b29236/windows-TESTS.txt docs/evidence-ci/2b29236/linux-REPORT.txt docs/evidence-ci/2b29236/windows-REPORT.txt
```

| Commit | Push run | Linux / macOS workspace | Windows workspace | Core release (Linux/macOS, Windows) | Doctests | platform-diff |
|---|---|---:|---:|---|---:|---|
| [2b29236](2b29236/) | 36251557201 | 365 / 365 | 358 (7 declared Unix-only) | 163, 158 | 2 | PASS |

A checksum identifies bytes; it does not authenticate the CI system or prove
that the tests cover every property. The report scope is stated in
[CI-EVIDENCE.md](../CI-EVIDENCE.md).
