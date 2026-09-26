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

Use a new directory controlled by the operator. This is not a security boundary
against another process replacing paths concurrently in that directory.
The display and the diagnostic pin parser may use replacement characters for
malformed UTF-8, but the captured stream bytes are not rewritten.

Four safety tests execute through xtask's platform test targets: every split
position of a Polish/emoji string, existing-output preservation, capture error
propagation and the buffer limit. A local Linux subprocess test with a child
that exits before the prompt returned a controlled failure and no COMPLETE;
a second attempt refused existing logs.

This does not certify old films or replace full movie decoding and visual review.
