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
3. Save the original snapshot, replace the document, and require rejection of
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
