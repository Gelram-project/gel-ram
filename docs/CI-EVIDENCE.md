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

- REPORT.txt (generated, `FORMAT=GEL_CI_EVIDENCE_2`): commit, source-manifest
  digest, OS, architecture, toolchain, numeric Actions run/attempt (or LOCAL).
  Per command: accepted/exit status, projected named test executions, and the
  sums of cargo's `test result:` lines — passed, failed, ignored, filtered out —
  plus `unlisted_executions` (executions whose names are not projected, such as
  path-bearing doctests). Then one `PLATFORM_EXCLUSION` line per whole test that
  is compiled only on Unix and one `PARTIAL_PLATFORM_BRANCH` line per test with
  an extra platform-only assertion block.
- TESTS.txt (generated): strict ASCII test identifiers and their standard statuses.
- SHA256SUMS.txt (generated): checksums of those two files.
- `COMPLETE`: emitted only after all commands and evidence writes succeed.

The collector actually executes workspace all-target tests (test/debug profile),
the core-path tests of gel-source, gel-store and gel-live-lab again in the
release profile, documentation tests and the saved R1 rechecker (release). The
release pass repeats tests already counted in the debug pass; its executions are
a separate profile check, not additional coverage. This is not a fresh R1 timing
campaign or a report of every workflow step. It excludes archive Ocean tests,
examples outside this command list, visual review and hardware experiments.
Each test command must produce at least one well-formed summary line with zero
failures; a malformed summary fails the report. The documentation tests run the
Rust examples of [SOURCE-BUILDER.md](SOURCE-BUILDER.md) and
[COLLECTION-BUILDER.md](COLLECTION-BUILDER.md); other guides contain only shell
commands, which are not executed as doctests.

Nested Cargo commands use an ignored, separate target directory so Windows
does not try to replace the running collector executable during test builds.
Failed checks also print the sanitized report; they still never emit COMPLETE.

## Redaction policy and limits

Raw stdout/stderr, environment variables, compiler diagnostics and filesystem
paths are not copied. Test-name projection admits only ASCII letters, digits,
underscore and colon; only `ok`, `FAILED` and `ignored` statuses are accepted.
Tests with other output formats (including path-bearing doctest names) are not
listed; their command exit status still applies. Repeated names remain repeated
executions, not distinct scientific experiments.

No untrusted output is interpreted as a command. A successful process exit is
mandatory; a printed PASS cannot replace it. Checksums provide byte identity,
not author authentication or proof of truth. Failure prevents COMPLETE.

## Platform exclusions

Seven whole tests are compiled only on Unix: two SIGKILL process tests, three
symlink tests and two permission-bit tests. They are declared in the collector.
On Linux/macOS each must appear exactly once in the debug workspace pass as
`ok` (`here=RAN_PASSED`); on Windows each must be absent
(`here=EXCLUDED_NOT_COUNTED`). Any other combination — missing, repeated,
ignored, or present where excluded — fails the report, so a stale declaration
cannot pass and an exclusion is never counted as a Windows success. Two tests
that run everywhere but contain an extra Unix- or Linux-only assertion block
must run and pass once on every platform (`PARTIAL_PLATFORM_BRANCH … here=RAN_PASSED`).
Platform reports are separate; do not add them into one number of unique tests.

A single job cannot see a test that another platform never compiled, so the
per-job check alone cannot find an undeclared Unix-only test. Compare two
reports' test lists (for example the TESTS.txt of a Linux and a Windows run of
the same commit, or the same lines printed in their CI logs):

```sh
cargo run --locked --offline -p xtask -- platform-diff UNIX_TESTS.txt WINDOWS_TESTS.txt
```

It fails unless the tests passing on Unix but not on Windows are exactly the
declared exclusions and nothing passes on Windows only. Test names are not
qualified by crate; each declared name is currently defined once.

## Native command sequencing

The portable workflow runs its six release demonstrations through the Rust
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
