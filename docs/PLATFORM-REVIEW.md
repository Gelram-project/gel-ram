# Candidate platform verification and handoff

Linux is tested locally and in CI. Native Windows and macOS tests and examples
also passed for the exact revision recorded below. Newer revisions must pass
their own checks; earlier green runs are not proof for changed bytes. No paid
service, larger runner, VM or LFS storage is authorized. Public branch/PR
upload and standard free public-repository CI are now authorized.

## Verified PR #9 head (2026-09-26) — historical for PR #10

Head: `08208c49a12f12f0af0fbaab3e6d78a86947b8b3`, Rust 1.85.0.
[Branch run](https://github.com/Gelram-project/gel-ram/actions/runs/36236486732)
and [PR run](https://github.com/Gelram-project/gel-ram/actions/runs/36236498081)
both passed. The PR merge-test commit `80dbb77c7538ccea714b12667db84f0a7783ad65`
has the same source tree as that head. This was a test merge, not a merge into main.

| Platform | Workspace tests | Additional execution |
|---|---:|---|
| Linux | 305 PASS | Full verify, integrity audit, 111 pinned Ocean archive tests |
| macOS ARM64 | 305 PASS | Source/collection demos, Q reference, integrity audit |
| Windows x64 MSVC | 299 PASS | Source/collection demos, Q reference, integrity audit |

These counts belong to that PR #9 head only. PR #10 changes the test set; read
the Linux, macOS, Windows and core-release counts from the
[CI evidence report](CI-EVIDENCE.md) of the exact run, not from this table.

At this revision six Unix-only tests were absent on Windows: two SIGKILL cases, symlink rejection
and Unix permission checks. They are not counted as Windows passes; later
revisions declare and check each one in the [CI evidence report](CI-EVIDENCE.md). Ocean's
Linux-specific mapping/persistence remains Linux-specific. This table records
the linked revision, not an unqualified guarantee of compatibility.
For later revisions, inspect the checks of the PR that contains them (currently [PR #10](https://github.com/Gelram-project/gel-ram/pull/10/checks)) and its CI evidence report.

## Same source and toolchain on every host

1. Use the exact reviewed ZIP and retain its SHA256 independently. Extract into
   a new directory. Do not mix files from different candidates or build trees.
2. Install Rust **1.85.0**, rustfmt and clippy; populate the locked Cargo cache
   beforehand. Setup may require Internet; runtime does not. Missing tools are
   a failed prerequisite, not a successful test.
3. From the extracted source directory run the commands below. Replace the
   MANIFEST_SHA256 placeholder with the reviewed 64-digit digest from outer
   STATUS.txt, independently retained through a trusted channel. A digest
   provided alongside untrusted files does not authenticate them.

```text
rustc -Vv
cargo run --locked --offline -p xtask -- source-audit MANIFEST_SHA256
cargo run --locked --offline -p xtask -- report ../new-platform-report MANIFEST_SHA256
cargo run --locked --offline --release -p gel-source --example collection_review
```

The report directory must be new and outside the source tree. Keep the whole
report, including failures and slow runs. Check each command's exit status
immediately (PowerShell: `$LASTEXITCODE`; POSIX shell: `$?`); stop on failure.
An archive is identified by its content manifest, not a fabricated Git commit.

Do not equate cross-compilation, Wine, skipped jobs or Linux runs with native
Windows/macOS results. Unix SIGKILL/permission tests are conditional: report the
actual executed count instead of copying Linux's number. Linux-only Ocean
mapping/persistence remains Linux-only. Power-loss durability is unproven on
every platform; non-Unix directory sync is not implemented by this path.

## Approval record

Record manifest, commit if present, rustc host triple, OS, actual test count,
exit status and complete logs per host. Only then mark that host tested.
Publication and main integration after checks are approved; a final release
is not approved. See CANDIDATE-STATUS.md for the current scope.
External independent evaluation of answers is another separate gate.
