# Reproduce everything in one command

```text
cargo run --locked --offline -p xtask -- reproduce NEW_DIR_OUTSIDE_CHECKOUT [--require-isolation]
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
start and end, the result of every step, the comparison summary, the SHA-256 of
every produced file and a final REPRODUCTION=PASS or FAIL. A failed step stops
the run and the later steps are marked NOT_RUN. COMPLETE.txt is written only
when every step passed or was skipped for a stated reason. With
`--require-isolation` the command refuses to start unless network isolation is
verified.

## Linux, with proven isolation

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout work/v0.5.0
rustup toolchain install 1.85.0 --profile minimal
cargo fetch --locked
unshare --user --net -- cargo run --locked --offline -p xtask -- reproduce ../gel-repro --require-isolation
```

## macOS and Windows

Run the same command without `unshare` and without `--require-isolation`; the
file records network_isolation=NOT_VERIFIED. In Windows PowerShell 5.1 type the
commands one per line (`&&` is not available there). The comparison step needs
grep and sha256sum on the PATH (Git Bash provides both); without them it is
SKIPPED with the reason, and the other steps still count.

## Report it

Open a [reproduction issue](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml)
and paste REPRODUCTION.txt. After a successful run it contains no user name, host
name or absolute path; an error message of a failed step may include a path, so read
it before posting. Slow and failing runs are as useful as fast ones.
