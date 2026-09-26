# Recorder safety

The scripted recorder is not a benchmark and does not supply substitute
application output. Existing videos remain historical recordings.

Current recorder safeguards:

- Markers are searched in raw bytes, including when a pipe splits a UTF-8 character.
- Each stdout/stderr capture is limited to 8 MiB. Exceeding the limit fails the run.
- Raw stdout, stderr and process status are recorded in separate files.
- Existing output paths, including symlinks, are refused before recording.
  Individual logs use create_new, never truncate an existing file.
- EOF before a required marker is an error, even if the child exited with status 0.
- COMPLETE is created only after both processes have finished successfully.
  A failed COMPLETE write removes the partial marker.
- Display, log, lock, reader-thread start, emitted-pin and snapshot failures
  end in `RECORDING_FAILED` with exit 1; a running child is killed first and its
  status file records `FAILED`.
- Every refusal before any output is written ends in `RECORDING_REFUSED` with
  exit 2: a missing or relative binary path, an unknown flag (including a
  non-UTF-8 flag), `--update` without a binary, or an existing output path. The
  binary may be any absolute OS path, including a non-UTF-8 one.
- `cargo xtask recorder-lint` (part of `verify`) runs clippy-driver on the
  standalone tool with 14 lints forbidden from the command line, so an
  `#[allow]` inside the file cannot relax them: `unwrap_used`, `expect_used`,
  `panic`, `unreachable`, `todo`, `unimplemented`, `indexing_slicing`,
  `string_slice`, `arithmetic_side_effects`, `print_stdout`, `print_stderr`,
  `dbg_macro`,
  and `disallowed_macros` / `disallowed_methods` configured to reject the
  assert macros, `std::thread::spawn`, `str::split_at`, `Vec::remove` and
  `std::env::args`
  ([tools/recorder-lint/clippy.toml](../tools/recorder-lint/clippy.toml)).
  The gate then lints ten copies, each with one injected construct (unwrap,
  an `#[allow]` bypass, assert, indexing, string slicing, arithmetic, thread
  spawn, print, dbg, env::args) and fails unless every copy is rejected by the
  expected lint. This is a denylist of named constructs: code hidden inside a
  local `macro_rules!` is not covered, and the tool defines no such macro.

Use a new directory controlled by the operator. This is not a security boundary
against another process replacing paths concurrently in that directory.
The display and the diagnostic pin parser may use replacement characters for
malformed UTF-8, but the captured stream bytes are not rewritten.

Eight safety tests execute through xtask's platform test targets: every split
position of a Polish/emoji string, existing-output preservation, capture error
propagation, the buffer limit, interrupted-read retry, failure after a prompt,
display write/flush failure and unwritable-log refusal. A local Linux subprocess test with a child
that exits before the prompt returned a controlled failure and no COMPLETE;
a second attempt refused existing logs.

Three process tests compile the real recorder and small substitute children:
existing log, missing executable, EOF before the prompt, closed child stdin and
an unwritable command log; wrong command lines and a display closed while the
child is alive (plus `/dev/full` on Linux); an invalid emitted snapshot pin.
Each must fail without a panic and without COMPLETE. Substitute children test
the operator's failure handling only; they are not application evidence.

A local re-run of both live walkthroughs with the hardened operator produced
byte-identical snapshots and command logs to the previous runs; the displayed
text differed only in measured timing digits. Since then the default mode's
two closing lines were reworded on purpose ("citation hash correspondence
checked", "Nothing was published"); the `--update` display is unchanged.

This does not certify old films or replace full movie decoding and visual review.
