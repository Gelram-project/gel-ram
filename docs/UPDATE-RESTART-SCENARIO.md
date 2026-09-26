# Executable update, restart and corruption scenario

Run the public CLI integration test with the pinned toolchain and previously
fetched dependencies:

```sh
cargo test --locked --offline -p gel-live-lab --test evidence complete_update_restart_and_corruption_scenario
```

The test creates synthetic English safety instructions, not personal data. It
executes two actual `gel-evidence` processes through their command interface:

1. Add a document and find a phrase whose preceding line contains `Do not`.
   Check that the displayed bounded context preserves this negation.
2. Verify the citation and obtain UNKNOWN for an absent phrase.
3. Save the original snapshot, select and verify the original citation again
   after UNKNOWN cleared the earlier selection, replace the document, and require rejection of
   the now-invalid selected result before running another search.
4. Require the old phrase to be absent and verify the revised phrase. Save a
   second snapshot with a different content pin; exit the process.
5. In a new process, load and verify the revised snapshot, then load and verify
   the original historical snapshot using their separately pinned hashes.
6. Attempt to load a single-byte-corrupted copy with the original pin. Require
   rejection and verify that the currently loaded original remains usable.
7. Check that neither historical snapshot was modified.

This exercises phrase readout, source correspondence and persisted state, not
natural-language understanding, hardware refresh computation or physical power
loss. The test is portable; separate Unix-only tests exercise SIGKILL. It does
not replace a new filmed demonstration or full visual review of old films.

During test development the first assertion incorrectly expected no REFUSED
output even though stale-proof refusal is required. The assertion was corrected
to require exactly that refusal, plus NO_CURRENT_RESULT. This was a test
expectation error, not a change that bypassed production rejection.

A later scenario review found a second test weakness: UNKNOWN had cleared the
selection before replace, so the later NO_CURRENT_RESULT did not by itself
demonstrate invalidation by replace. The scenario now reselects and verifies a
real hit immediately before replacement and checks that ordering. Historical
test results are not reinterpreted as having covered this stronger precondition.

## Live typing driver for a separate, longer recording

The [Rust driver](../tools/record_evidence.rs) also accepts --update after the
absolute application binary path. Compile it with Rust 1.85.0, then run it from
a fresh owner-controlled directory containing the authored
[original](../crates/gel-source/fixtures/evidence-lab/original.txt) and
[revised](../crates/gel-source/fixtures/evidence-lab/revised.txt) text fixtures.
The synthetic instructions are test data, not operational safety advice.

This mode executes the scenario above with scripted typing and genuine process
output. It retains two snapshots and explicitly announces making a one-byte
corrupted copy between processes. The original snapshots are compared byte for
byte at the end. It creates separate stdout, stderr and status logs for each
process, plus a command log and completion marker only after success.

It does not record pixels itself. Screen capture and a visual review remain
separate work; the existence of the driver or its successful terminal log is
not evidence that a fourth MP4 has been filmed or approved. The original
two-document mode and three historical videos remain available and unchanged.
No answers are substituted, and pauses are outside application timers.

## Recorder failure regression

`cargo test --locked --offline -p xtask --test recorder_process` compiles and
runs the actual Rust recorder. It checks refusal of an existing command log,
a missing application executable, an application exiting before its first
prompt, one exiting just after that prompt (closed stdin), and a command-log
path replaced with a directory in the disposable test environment.
All must fail without a completion marker or a panic; the existing log
must remain byte-identical. Spawn failure and early EOF retain a failed process
status log. Command transmission and command-log write/sync failures terminate
and reap the subprocess through the same failure path. Command-log sync is
outside application operation timers; it is not benchmark work.
This complements stream-fragment and I/O injection tests; it does
not establish coverage of every recorder error path.
