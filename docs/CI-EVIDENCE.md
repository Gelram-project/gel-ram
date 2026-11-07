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
  is compiled only on Unix or only on Linux and one `PARTIAL_PLATFORM_BRANCH` line per test with
  an extra platform-only assertion block; its branch field says whether that
  block was active on the recording platform (active or inactive). Then one
  `PROPERTY` line per row of the [property-to-test map](PROPERTY-TESTS.md) and a
  `PROPERTY_MAP=CHECKED` total; a mapped test that did not run and pass exactly
  once, other than a declared platform-only test elsewhere, withholds COMPLETE.
- TESTS.txt (generated): strict ASCII test identifiers and their standard statuses.
- SHA256SUMS.txt (generated): checksums of those two files.
- `COMPLETE`: emitted only after all commands and evidence writes succeed.

The collector actually executes workspace all-target tests (test/debug profile),
the core-path tests of gel-source, gel-store and gel-live-lab again in the
release profile, documentation tests and the saved R1 rechecker (release). The
release pass repeats tests already counted in the debug pass; its executions are
a separate profile check, not additional coverage. This is not a fresh R1 timing
campaign or a report of every workflow step. It excludes
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
Tests with other output formats (including path-bearing doctest names and an
ignored test that prints its reason) are not listed but are counted in
`unlisted_executions`; their command exit status still applies. Repeated names remain repeated
executions, not distinct scientific experiments.

No untrusted output is interpreted as a command. A successful process exit is
mandatory; a printed PASS cannot replace it. Checksums provide byte identity,
not author authentication or proof of truth. Failure prevents COMPLETE.

## Platform exclusions

Twenty-three whole tests are platform-only. Nineteen are compiled only on Unix:
three SIGKILL process tests, eight symlink tests, one FIFO test, four
permission-bit tests and three that race processes on a file replaced by rename.
Four are compiled only on Linux: three read a procfs file that reports length zero,
and one runs `xtask memory-run`, which reads `/proc`.
They are declared in the collector with their condition. Where the condition
holds each must appear exactly once in the debug workspace pass as `ok`
(`here=RAN_PASSED`); elsewhere each must be absent
(`here=EXCLUDED_NOT_COUNTED`). Any other combination — missing, repeated,
ignored, or present where excluded — fails the report, so a stale declaration
cannot pass and an exclusion is never counted as a success elsewhere. Three named
tests that run everywhere but contain an extra Unix- or Linux-only assertion block
must run and pass once on every platform (`PARTIAL_PLATFORM_BRANCH … here=RAN_PASSED`);
this list is not complete, and other tests hold such a block without being listed.
Platform reports are separate; do not add them into one number of unique tests.

Reports for selected commits are kept in [evidence-ci](evidence-ci/README.md),
because Actions runs, logs and artifacts expire. `xtask ci-evidence-import`
rebuilds them from a job log and `xtask ci-evidence-recorded`, part of `verify`,
checks them; a report altered, a platform file missing or an unlisted file fails
it. They are sanitized reports of hosted CI runs of one revision each; hosted CI
is not an independent second host.

A single job cannot see a test that another platform never compiled, so the
per-job check alone cannot find an undeclared platform-only test, and no CI job
compares two platforms. A unit test of xtask therefore reads the workspace's Rust
sources on every platform: a test under a platform `cfg`, on the test, on an
enclosing module or at the head of its file, must be declared with that
condition, and a declared test must stand under it. So a platform-only test is
found when it is added. It reads source text only: it does not see a test made by
a macro, a platform-only block inside a test that runs everywhere, or a check of
the platform at run time, and it says nothing about what ran.

The comparison of two platforms runs when a revision is recorded. Compare two
reports' test lists (for example the TESTS.txt of a Linux and a Windows run
of the same commit, or the same lines printed in their CI logs):

```sh
cargo run --locked --offline -p xtask -- platform-diff UNIX_TESTS.txt WINDOWS_TESTS.txt UNIX_REPORT.txt WINDOWS_REPORT.txt
```

It fails unless the tests passing on Unix but not on Windows are exactly the
declared exclusions (each run exactly once on Unix) and nothing passes on
Windows only. A Linux-only test may not appear in a macOS list. Every line must
be a projected test line of a known case; an ignored line is accepted only for
the five declared child-process helpers, and each must be ignored equally on both sides.
With the two REPORT files it also requires one commit and a Unix and a Windows
host, and takes the first report's OS; a test declared after that revision, so
not named in that report, may be missing from an older list but still may not
run on Windows. Without them the first list is taken as Linux. Test names are not
qualified by crate; each declared name is currently defined once.

## Native command sequencing

The portable workflow runs its eight release demonstrations through the Rust
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

## Additional audit regression

The report-inventory regression runs on every platform. Its symlink rejection
assertion is Unix-only and is declared separately in the partial-platform map;
it is never reported as a Windows symlink test. Workflow policy and roadmap
consistency regressions run as ordinary workspace tests on all platforms.
