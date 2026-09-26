# Candidate platform handoff — no remote run performed

Linux is tested locally. Windows and macOS execution for this new candidate is
**pending**, not covered by old green CI. The prepared workflow executes tests
and demos; preparation neither uploads code nor triggers a runner. No paid
service, larger runner, VM or LFS storage is authorized. Public branch/PR
upload and standard free public-repository CI are now authorized.

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
Publication permission is limited to public branch/PR review; merging and
release are not approved. See CANDIDATE-STATUS.md for the current scope.
External independent evaluation of answers is another separate gate.
