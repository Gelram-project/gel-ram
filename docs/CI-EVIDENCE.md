# Scoped CI evidence package

After committing the source manifest, with Rust 1.85.0 and dependencies already
available, run:

```sh
cargo run --locked --offline -p xtask -- ci-evidence ../new-ci-evidence
```

The output directory must not exist and must be outside the checkout. A dirty
checkout is rejected. Both before and after execution, the collector validates
the complete source inventory against its manifest and checks the commit.

Files:

- REPORT.txt (generated): commit, source-manifest digest, OS, architecture, toolchain,
  numeric Actions run/attempt (or LOCAL), commands' accepted/exit status and
  count of projected named test executions.
- TESTS.txt (generated): strict ASCII test identifiers and their standard statuses.
- SHA256SUMS.txt (generated): checksums of those two files.
- `COMPLETE`: emitted only after all commands and evidence writes succeed.

The collector actually executes workspace all-target tests, documentation tests
and the saved R1 rechecker. The first two use the test/debug profile; the R1
rechecker uses release. This is not a fresh R1 timing campaign or a report of
every workflow step. It excludes archive Ocean tests, examples outside this
command list, visual review and hardware experiments.

Nested Cargo commands use an ignored, separate target directory so Windows
does not try to replace the running collector executable during test builds.
Failed checks also print the sanitized report; they still never emit COMPLETE.

## Redaction policy and limits

Raw stdout/stderr, environment variables, compiler diagnostics and filesystem
paths are not copied. Test-name projection admits only ASCII letters, digits,
underscore and colon; only `ok`, `FAILED` and `ignored` statuses are accepted.
Tests with other output formats (including path-bearing doctest names) are not
listed; their command exit status still applies. Repeated names remain repeated
executions, not distinct scientific experiments. Compile-time platform
exclusions are not visible as ignored tests and must be documented separately.

No untrusted output is interpreted as a command. A successful process exit is
mandatory; a printed PASS cannot replace it. Checksums provide byte identity,
not author authentication or proof of truth. Failure prevents COMPLETE.

## Native command sequencing

The portable workflow runs its five release demonstrations through the Rust
command below. Each native ExitStatus is checked before starting the next
command; neither PowerShell's final exit status nor a printed PASS substitutes
for that check. The same process runner is used by ordinary xtask commands.

```sh
cargo run --locked --offline -p xtask -- runtime-examples
```

[Process tests](../xtask/tests/process_sequence.rs) compile a tiny native child
which appends an execution trace and deliberately prints PASS even when exiting
with an error. First-failure and middle-failure cases require a stopped trace;
the all-success control requires all three executions. A missing executable is
also rejected. These tests are part of workspace tests on all three platforms.
Their Linux result does not establish Windows execution until that revision's
Windows CI completes. The successful control still runs each real demonstration
in release; this helper does not replace them with fixtures in production CI.

## Zero-cost publication policy

This command never uploads anything. CI prints the sanitized projection between
`CI_EVIDENCE_REPORT_BEGIN` and `CI_EVIDENCE_REPORT_END`; the local package remains
available for a separately reviewed publication. No upload-artifact action,
paid runner, cache or storage purchase is enabled by this change. Therefore a
downloadable Actions artifact is still **NOT_ENABLED**, not falsely marked
complete. Logs have platform retention limits; archive reviewed evidence before
expiry when producing a release.
