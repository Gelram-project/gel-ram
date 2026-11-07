# Recorded CI evidence

GitHub Actions artifacts and, after the repository's retention period, the
workflow runs and logs themselves expire, and artifacts can be downloaded only
by signed-in users. The sanitized per-platform reports produced by
`xtask ci-evidence` are therefore also kept here, for selected commits.

These are sanitized reports of hosted CI runs of one revision each, on the
three platforms of one push run. They are not a report for every revision, and
hosted CI is not an independent second host.

Each directory is named after the tested commit and holds, per platform, the
REPORT and TESTS files printed by the collector in that job's log and the
collector's own checksum list. The files are rebuilt from the log block
between `CI_EVIDENCE_REPORT_BEGIN` and `CI_EVIDENCE_REPORT_END`, and the rebuild
is refused unless the checksums printed in CI match the rebuilt files:

```sh
gh api repos/Gelram-project/gel-ram/actions/jobs/JOB_ID/logs > JOB.log
cargo run --locked --offline -p xtask -- ci-evidence-import JOB.log linux docs/evidence-ci/REVISION
```

Check the bytes and the strict comparison of one revision with:

```sh
cd docs/evidence-ci/79f6ac5 && sha256sum -c linux-SHA256SUMS.txt windows-SHA256SUMS.txt macos-SHA256SUMS.txt
cargo run --locked --offline -p xtask -- platform-diff docs/evidence-ci/79f6ac5/linux-TESTS.txt docs/evidence-ci/79f6ac5/windows-TESTS.txt docs/evidence-ci/79f6ac5/linux-REPORT.txt docs/evidence-ci/79f6ac5/windows-REPORT.txt
```

`xtask ci-evidence-recorded`, part of `verify`, checks every directory below
against the list of recorded revisions: all nine files and no others, the
checksums, the report format, the run and commit of each report, and the strict
comparison of Linux and of macOS with Windows, and it recomputes the table.

| Commit | Push run | Linux / macOS workspace | Windows workspace | Core release (Linux / macOS, Windows) | Doctests | platform-diff |
|---|---|---:|---:|---|---:|---|
| [2b29236](2b29236/) | 36251557201 | 365 / 365 | 358 (7 declared platform-only) | 163 / 163, 158 | 2 | PASS (strict: commit and hosts checked) |
| [fba6c43](fba6c43/) | 36257696669 | 366 / 366 | 359 (7 declared platform-only) | 163 / 163, 158 | 2 | PASS (strict: commit and hosts checked) |
| [79f6ac5](79f6ac5/) | 42077523488 | 812 / 809 | 794 (18 declared platform-only) | 381 / 378, 369 | 2 | PASS (strict: commit and hosts checked) |

Reading the 79f6ac5 row: the 18 in it is what the reports give, Windows passing
18 fewer than Linux. A report states the declarations of its own revision, and
that revision's reports declare 7. Its lists hold 11 more platform-only tests, 8
Unix-only and 3 Linux-only, that were declared only after it was recorded; the
PASS is the comparison under the current declarations. Under the 7 it declares
itself, the strict comparison fails. Each of the three platforms lists one
ignored child-process helper in two cases, on all three alike.

A checksum identifies bytes; it does not authenticate the CI system or prove
that the tests cover every property. The report scope is stated in
[CI-EVIDENCE.md](../CI-EVIDENCE.md).
