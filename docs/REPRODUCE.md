# Run the public verification suite and record platform exclusions

```text
cargo run --locked --offline -p xtask -- reproduce NEW_DIR_OUTSIDE_CHECKOUT [--require-isolation] [--strict]
```

The command runs, in this order, into one new directory:

| Step | What it runs | Output |
|---|---|---|
| isolation | the [network-isolation check](REPRODUCE-ISOLATED.md) | recorded as VERIFIED or NOT_VERIFIED |
| report | `xtask report`: full `verify` and the runtime campaigns | report/ |
| mutation-matrix | the [format mutation matrix](MUTATION-MATRIX.md) | mutation-matrix.txt |
| bench-compare | the [comparison with grep and sha256sum](BENCHMARK-GREP.md) | bench/ |

It then writes REPRODUCTION.txt: the revision and whether the working tree was
clean, operating system, CPU model, memory, isolation status, host load at the
start and end, the result of every step, the comparison summary and the SHA-256
of every produced file. The last line counts the outcomes, so a skipped step is
visible without opening any log:

```text
REPRODUCTION=PASS pass=3 fail=0 skipped=0 not_run=0 isolation=VERIFIED required_isolation=yes strict=no
```

A failed step stops the run and the later steps are NOT_RUN. A step is SKIPPED
only for a stated reason, for example the comparison without grep or sha256sum.
COMPLETE.txt means a complete report of the declared run, with the same counts;
it does not mean that every possible check passed on every system. Two options
tighten the run:

- `--require-isolation` refuses to start unless network isolation is verified;
- `--strict` turns every SKIPPED step into FAIL. Use it as a release gate, so a
  missing dependency can never pass for a successful check.

## Linux, with proven isolation

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.6.1
rustup toolchain install 1.85.0 --profile minimal
cargo fetch --locked
unshare --user --net -- cargo run --locked --offline -p xtask -- reproduce ../gel-repro --require-isolation --strict
```

## macOS and Windows

Run the same command without `unshare` and without `--require-isolation`; the
file records network_isolation=NOT_VERIFIED. In Windows PowerShell 5.1 type the
commands one per line (`&&` is not available there). The comparison step needs
grep and sha256sum on the PATH (Git Bash provides both); without them it is
SKIPPED with the reason, and the other steps still count.

## A short witness

The full run above takes a while. One shorter command checks the claims of the
first screen on your own machine and prints one block to paste into a report:

```text
cargo run --locked --offline -p xtask -- witness NEW_DIR_OUTSIDE_CHECKOUT [YOUR_FILE]
```

It runs six steps, each as its own process, and reads each step's result from the
line that program prints:

| Step | What it runs | Passing line |
|---|---|---|
| literal | `gel-live-lab --literal YOUR_FILE` (README.md when you name none): your bytes in one record, four views restored, a changed byte refused | `GEL_LIVE_LAB_LITERAL=PASS` |
| quad | `gel-phase-quad --example quad_compare`: every score of the four views equals the reference bit for bit | `Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED` |
| crash-series | `xtask crash-series 20`: 20 kills of a writing process; no acknowledged snapshot lost | `CRASH_SERIES=PASS …` |
| history-crash-series | `xtask history-crash-series 5 20261010 5`: 5 kills of a record-history writer, with 5 in-place control trials | `HISTORY_CRASH_SERIES=PASS …` |
| history-mutation-matrix | `xtask history-mutation-matrix`: 151 changed record-history files, read by the library and a reference reader | `HISTORY_MUTATION_MATRIX=PASS …` |
| same-bits | `gel-live-lab --witness`: your machine's score bits against the author's pins ([same bits on your machine](LIVE-LAB.md#same-bits-on-your-machine)) | `WITNESS=MATCH …` |

A step passes only when its process succeeds and its last line of that kind is the
passing one; a missing line is a failure. Every step runs even when an earlier one
fails. The two kill series need a Unix host and are SKIPPED elsewhere, with the
reason. The block names the commit, the operating system, CPU, memory and rustc,
each step's line and the block's own SHA-256, and ends with
`GEL_WITNESS=PASS|FAIL pass=… fail=… skipped=…`. It contains no path, user name
or host name; the same block is in NEW_DIR/WITNESS.txt, and each step's whole
output is in NEW_DIR/<step>.log. On the author's machine (24 threads) the six
steps take about 70 seconds.

## Report it

Open a [reproduction issue](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml)
and paste REPRODUCTION.txt. After a successful run it contains no user name, host
name or absolute path; an error message of a failed step may include a path, so read
it before posting. Slow and failing runs are as useful as fast ones.
