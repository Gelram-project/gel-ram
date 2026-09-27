# GEL RAM v0.5.0 — scripted evidence, backups and checked packages

Date: 2026-09-27

Run the Evidence Lab from a script, back up and restore its snapshots, check
the public file formats against a finite mutation matrix, compare it with grep
on the same corpus, and reproduce the whole public suite in one command with
the network cut off — offline, in Rust, without an LLM. After the setup below,
try:

```text
printf 'help\nexit\n' | cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --batch
```

The first line is a `"record":"session"` JSON object with schema
`gel-evidence/1`; the last is the summary with `"exit_code":0`.

v0.5.0 is the first release after v0.4.0. It adds tools, evidence and the backup
directory layout (a copied snapshot plus MANIFEST); it does
not change the GELSRC01, GELSET01 or Q8DEMO01 formats; the Q8DEMO01 reader only
moved from an example into the gel-phase-quad crate.

## Script, back up, restore

- **Batch mode** ([batch](docs/EVIDENCE-BATCH.md)): `gel-evidence --batch` reads
  the interactive commands from standard input and writes JSON Lines with
  schema `gel-evidence/1` to standard output and diagnostics to standard error.
  UNKNOWN, INCOMPLETE and ERROR stay distinct, with exit codes 0, 3 and 2; the
  first ERROR stops the batch and later commands are counted as not run. A new
  `clear` command forgets the collection in memory only.
- **Backup** ([backup](docs/BACKUP.md)): `gel-backup` creates, inspects,
  restores, withdraws and deletes backups of pinned snapshots. A backup without
  a committed manifest is INCOMPLETE and cannot be restored; restore writes only
  to a path that does not exist, and a failed directory sync after publication
  is reported as PUBLISHED_UNCONFIRMED, not as nothing written.

## Check and reproduce

- **Format mutation matrix** ([matrix](docs/MUTATION-MATRIX.md)): 179 mutants
  of the three public formats, each rejected or its acceptance explained, with
  a lenient-reader control; `xtask verify` compares the result with the recorded
  report.
- **Network isolation evidence** ([isolated reproduction](docs/REPRODUCE-ISOLATED.md)):
  `xtask isolation-check` reads the process's own network view and probes
  documentation addresses. CI requires it to fail on the networked runner and to
  pass inside a network namespace.
- **One-command reproduction** ([reproduce](docs/REPRODUCE.md)):
  `xtask reproduce` runs verification, the mutation matrix and the grep
  comparison into one new directory and records revision, host, load and
  hashes; `--require-isolation` and `--strict` make it a release gate.
- **Property-to-test map** ([map](docs/PROPERTY-TESTS.md)): 35 documented
  properties mapped to 71 tests; on every CI platform each mapped test must run
  and pass exactly once, or be a declared Unix-only test elsewhere.
- **Claim registry** ([claims](docs/CLAIMS.md)): the new tools and every README
  timing table are registered; an open row can carry only SEPARATE_GATE,
  MEASURED_LOCAL, NOT_VERIFIED or NOT_ESTABLISHED.

## Measure without overstating

- **Comparison with grep and sha256sum** ([comparison](docs/BENCHMARK-GREP.md)):
  answers are compared before any timing; 20 of 36 public queries give the same
  answer. On those, one host, GEL is slower: p50 72.7 ms against 62.8 ms for the
  whole process workload and 3.13 ms against 1.95 ms for the integrity check.
- **GEL's own search path** ([README](README.md)): three replays of the private
  sketch-search engine are shown next to the historical full-scan baseline as
  author-reported numbers that cannot be re-run from this checkout. No speed-up
  factor is claimed.
- **Cost of one collection change** ([mutation comparison](docs/MUTATION-COMPARISON.md)):
  paired timing and the resident-set peak of one change in fresh processes,
  with the peak reset before the change (`xtask mutation-campaign`).

## Binary packages

`gel-evidence`, `gel-backup` and `gel-live-lab` for 64-bit x86 Linux (glibc),
Apple silicon macOS and 64-bit x86 Windows, built only in CI
([binaries](docs/BINARIES.md)). A package is refused unless every third-party
license file matches the reviewed inventory, no build-machine path is in a
binary and the packaged programs pass a smoke run on their platform. Each
archive has a build-provenance attestation:
`gh attestation verify ARCHIVE --repo Gelram-project/gel-ram`. The programs are
not code-signed or notarized and are checked only on their build runners.

## Documentation

- The README opens with a light and dark header, an animated diagram of the
  citation check (drawn, not recorded; it stops under reduced motion), a strip
  of checked facts whose numbers the builder reads from the property map and
  the mutation matrix, and six short replays of recorded public workflows with
  colour-coded badges ([gallery](media/gifs/README.md)); each replay is labelled
  as an edited log replay, not wall time. A script-free full-page edition,
  [README-MULTIMEDIA.html](README-MULTIMEDIA.html), opens from a checkout without
  network access ([presentation guide](docs/README-PRESENTATION.md)).
- Public limits are stated without naming private components.
- The update, restart and corrupted-copy scenario is shown as one still of its
  final screen.
- The two private application previews are described by scope and limits
  only; the public Evidence Lab film keeps its full PL/EN transcript.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.5.0
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. On Linux,
`unshare --user --net -- cargo run --locked --offline -p xtask -- reproduce ../gel-repro --require-isolation --strict`
reruns the public suite without network access ([reproduce](docs/REPRODUCE.md)).
`SOURCE-SHA256SUMS.txt` lists every other tracked
file of the tagged tree. The release asset GEL-RAM-v0.5.0-SOURCE.zip is
`git archive` of the tagged commit; its SHA-256 is published next to it.

## Still open

A human start-to-finish review of the films and transcripts, an independent
second-host reproduction and mutation campaign, a same-task end-to-end
comparison, public execution-identity evidence and an adaptive-precision error
budget remain open. See the [roadmap](docs/ROADMAP.md) and the
[claim registry](docs/CLAIMS.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, private
engine or bank, encrypted storage, hardware-level memory computation, fourfold
independent capacity from coordinate views, speed advantage over grep or any
language model, or commercial superiority is claimed.

## Licensing and compatibility

Licensing is unchanged: GEL RAM Noncommercial Reciprocal License 1.0 for GEL
RAM-owned material, the commercial path and CLA 2.0 ([LICENSING](LICENSING.md)).
Rust remains pinned to 1.85.0; third-party dependency versions are unchanged.
The v0.3.0 and v0.4.0 tags are not moved.
