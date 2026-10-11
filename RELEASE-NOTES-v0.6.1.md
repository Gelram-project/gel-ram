# GEL RAM v0.6.1 — the same score bits on every platform, and what a stopped save leaves

Date: 2026-10-11

v0.6.1 is an evidence release. The public Q8 reader now takes its 256 cosine
values from one frozen table, so a Q8 score is the same bits on Linux, macOS and
Windows; a new command, `gel-live-lab --witness`, checks that on your machine.
Three checks show what a stopped save leaves and how the record-history format
refuses changed bytes. One public trait and one function are added to
gel-source. The file formats, the Rust pin and third-party dependencies are
unchanged from v0.6.0; the workspace version moves to 0.6.1. Every change since
v0.6.0 is listed with its pull request in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/ROADMAP.md).

## What the project is

GEL RAM prints knowledge into memory as fixed Q8 records, instead of training it
into model weights, and reads the records back exactly, working toward
hardware-level memory computation. The goal is a text AI that answers in Polish
and English from what it holds, or says it does not know. **The text AI is a
goal, not a result of this repository**, and hardware-level memory computation
is not established: the claim registry lists it as `NOT_ESTABLISHED`. The step
that prints text into records is private, so the Q8 records in this checkout
hold synthetic values or literal bytes.

## The same score bits on every platform

A public Q8 score is a sum of 256 cosine values weighted by a histogram. Until
this release the reader took those cosines from the platform's maths library.
The new witness read all 256 of them through the public `Reader` on the three CI
platforms. Linux matched the author's machine. macOS on arm64 gave a different
last bit at 13 of the 256 levels and Windows on x86_64 at 7, each by one unit in
the last place, so the same Q8 score was not the same bits everywhere.

The reader, the reference reader and the Q8 examples now use one frozen table,
`grid::COS256_BITS`: the values glibc 2.39 gives on the author's Linux host.
Every Linux result is unchanged bit for bit. On all three CI platforms the
witness now matches both the table and 1,048,576 scores of a synthetic bank, and
it still shows, for information only, where the platform's own cosine differs.

```text
cargo run --locked --offline --release -p gel-live-lab -- --witness
```

A `MISMATCH` names every differing level and exits 1. Please report the four
lines with the commit, the operating system and the CPU: a run on a machine
outside the project is the open acceptance. See
[same bits on your machine](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/LIVE-LAB.md#same-bits-on-your-machine).

## What a stopped save leaves

- **A Q8 record stopped at known points.** A child process saves the
  1,164-byte record `gel-live-lab --literal FILE --save NEW_PATH` writes and
  ends at each of 23 known points: after every 64-byte piece of the temporary
  file, after its flush, after the link and after the directory flush. At every
  point the new path holds nothing or the whole record under its pin, never part
  of it. A naive writer stopped at the same points leaves part of a record at 19
  of 21, which the test requires. The test runs on all three CI platforms
  ([stopped at known points](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/PUBLICATION-FAULT-TESTS.md#a-q8-record-save-stopped-at-known-points)).
- **A record history killed while it grows.** `xtask history-crash-series`
  kills a writer that saves a growing 240-state history with `write_atomic`.
  In the recorded run of 200 trials on Linux, 0 of 23,379 acknowledged states
  were lost, no file was partial and all 200 trials resumed to the bytes of a
  run without a kill; a writer that rewrites the file in place left a partial
  file in 14 of 20 control trials
  ([kill series](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/CRASH-SERIES.md#the-record-history-one-file-replaced-while-it-grows)).
- `quad_evidence --generate` now writes a flushed temporary file and links it
  into place, instead of writing the new file directly.

A process kill or stop is not a power cut; power-loss durability stays
`NOT_ESTABLISHED`.

## The record-history format in the mutation matrix

`xtask history-mutation-matrix` changes one 338-byte GELHIS01 file of four
states in 151 ways, with its CRC64 fields recomputed as anyone who rewrites the
file can do, and reads every mutant with the library and with a reference reader
written from the format description. Every row agrees; 14 valid changes are
accepted with the same rebuilt states, and a reader without the canonical-form
rule is caught. It has its own report, so the three-format matrix is unchanged
([format mutation matrix](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/MUTATION-MATRIX.md#gelhis01-record-history-files)).

## README replays

The sixth replay now records `gel-live-lab --literal` saving a record and
`--reopen` checking it under the pin, instead of the comparison with grep. All
six replays were recorded again on the author's Linux host.

## Library and tool changes

- gel-phase-quad: `grid::COS256_BITS` and `grid::cos256()`; the reader and
  `Grid::similarity` for 256 levels use them. Z5 keeps the platform cosine.
- gel-source: `PublicationIo` and `write_file_new_with` are public, so a test can
  fail or stop the no-replace publisher at a known step; `write_file_new`
  behaves as before.
- gel-live-lab: `--witness` and its `witness` module.
- xtask: `history-crash-series`, `history-mutation-matrix` and the witness in
  `runtime-examples`.
- The claim registry gains four rows: `record-history-kill-series`,
  `history-format-mutations`, `q8-score-bits-across-platforms` and
  `q8-record-save-stopped`, all `SEPARATE_GATE`; `record-history-durability` now
  covers a power cut only and stays `NOT_ESTABLISHED`.

## Earlier versions

The notes of v0.6.0 leave the tree, as earlier notes did.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.6.1
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline --release -p gel-live-lab -- --witness
```

Expected final markers: `GEL_VERIFY_ALL=PASS` and `WITNESS=MATCH`.
`SOURCE-SHA256SUMS.txt` lists every other tracked file of the tagged tree. The
release asset GEL-RAM-v0.6.1-SOURCE.zip is `git archive` of the tagged commit;
its SHA-256 is published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in
[binaries](https://github.com/Gelram-project/gel-ram/blob/v0.6.1/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, measured
conversation, private engine or bank, hardware-level memory computation, speed
advantage over grep or any language model, or commercial superiority is claimed.
Four views of one record are not four independent memories, four votes or a 4×
speedup. The frozen cosine table makes the public scores reproducible; it is not
a claim about the accuracy of any maths library. A process kill or stop is not a
power cut; power-loss safety is not established. Answering natural questions
remains open: GEL answers a minority of them, and a 99% precision is not
claimed.
