# Comparing the run time of one example between two builds

`xtask speed-ab` times one timing example in a baseline tree and in a
candidate tree, with a second copy of the baseline as a control. It reports
medians and ratios, and whether every tree printed the same output check. It
is an instrument: it never says that one build is faster.

```text
cargo run --locked --offline -p xtask -- speed-ab \
  --a BASELINE --a2 BASELINE_COPY --b CANDIDATE \
  --package PKG --example NAME \
  --target-root BUILD_DIR_OUTSIDE_TREES --out NEW_DIR_OUTSIDE_TREES \
  [--rounds R] [--with-bins PKG]... [--replace-helpers] \
  [--before-run PROGRAM [--before-run-arg ARG]...] [-- EXAMPLE_ARG...]

cargo run --locked --offline -p xtask -- speed-ab --candidate-only \
  --b CANDIDATE --package PKG --example NAME \
  --target-root BUILD_DIR_OUTSIDE_TREES --out NEW_DIR_OUTSIDE_TREES \
  [--rounds R] [--with-bins PKG]... \
  [--before-run PROGRAM [--before-run-arg ARG]...] [-- EXAMPLE_ARG...]
```

Everything after `--` is passed to the example. R is 1 to 999 (default 3).

## What a timing example prints

The tool reads two kinds of lines from the example's standard output and
ignores all others:

```text
SPEED name=<case> run=<i> ns=<n>
SPEED_CHECK <value>
```

`SPEED_CHECK` is a fingerprint of everything the example computed, for
example `sha256=<64 hex digits>`. A malformed `SPEED` or `SPEED_CHECK` line is
an error, not a skipped line.

## The three trees and the A/A control

- A is the baseline, A2 a second, independent copy of the same baseline, B the
  candidate.
- The example file is taken from B (the file of that name in the examples
  directory of PKG) together with its support directory when the example uses
  one, and with every file it includes through `#[path = "..."]`. An include is
  resolved relative to the directory of the file that contains it and is
  followed through included files, so a shared helper such as
  `tools/speed_support.rs` and anything it includes are carried as well.
  Attributes inside comments and string literals are not includes; an include
  that leaves the tree, is not a plain string literal, is conditional
  (`cfg_attr`) or is missing in B is an error.
- These files are placed into A and A2 for the run and removed afterwards, so
  all three trees build exactly the same example source. A destination that
  already holds the same bytes is left untouched. One that holds different
  bytes is refused before anything is written to either tree: the trees would
  otherwise build different code. The only exception is an included file under
  `tools/` when `--replace-helpers` is given: its bytes in A and A2 are kept in
  the output directory, the file is replaced for the run, and the original
  bytes are written back afterwards. Files and directories that the tool
  created are removed again, and replaced files get their original bytes back,
  also when a build or a run fails.
- Each tree is built in release with its own build directory under
  `--target-root`:
  `cargo build --locked --offline --release -p PKG --example NAME`
  (with `-p P ... --bins` added for each `--with-bins P`).
- Round r runs the trees in a rotated order: B, A, A2 when r mod 3 = 1;
  A2, B, A when r mod 3 = 2; A, A2, B when r mod 3 = 0. Three consecutive
  rounds put each tree in each position once.
- Before each build and each run, the optional `--before-run` program is
  executed with its `--before-run-arg` arguments, in order, and must exit 0
  (for example a command that waits for an idle host). Its output goes to
  standard error.

A and A2 are the same source built twice. Their ratio `aa` shows how much two
identical builds differ on this host at this time; a difference between A and
B that is not clearly larger than the difference between A and A2 is not a
measured difference.

## What it prints

For every case, the medians (the lower middle value for an even count) and two
ratios, each with exactly three decimals, computed from integers:

```text
SPEED_AB case=<case> a_ns=<n> a2_ns=<n> b_ns=<n> aa=<A2/A> gain=<A/B> runs=<n>
SPEED_AB_CHECK tree=a <value>
SPEED_AB_CHECK tree=a2 <value>
SPEED_AB_CHECK tree=b <value>
SPEED_AB_VERDICT=SAME_OUTPUT
```

`gain` above 1 means a lower median in B. `runs` is the sample count per tree;
when the trees differ it is printed as `a/a2/b`. A missing value is `NA`.

The last line is exactly one of:

| Line | Meaning | Exit |
|---|---|---|
| `SPEED_AB_VERDICT=SAME_OUTPUT` | every tree printed at least one `SPEED` line and the same non-empty set of `SPEED_CHECK` values | 0 |
| `SPEED_AB_VERDICT=DIFFERENT_OUTPUT` | the sets of `SPEED_CHECK` values differ | non-zero |
| `SPEED_AB_VERDICT=NO_DATA` | a tree printed no `SPEED` line or no `SPEED_CHECK` value | non-zero |

`--candidate-only` runs only B, R times, for an example that exists only in the
candidate. It prints `SPEED_AB case=<case> b_ns=<n> runs=<n>`, the check values
of B and `SPEED_AB_VERDICT=CANDIDATE_ONLY`, or `NO_DATA` with a non-zero exit.

## What the verdict says and does not say

`SAME_OUTPUT` says that the three builds printed the same check values for the
inputs the example uses. It does not prove equal results for other inputs, and
it says nothing about speed. The medians and ratios are numbers from one host
at one time; the tool does not decide whether a ratio is a gain, and it applies
no threshold.

## Files in the output directory

```text
a-r1.txt a2-r1.txt b-r1.txt ...  standard output of each run, by tree and round
manifest.txt                     arguments, SHA-256 of every carried source file,
                                 files placed into and replaced in A and A2,
                                 revision and compiler of each tree, run order of each round
summary.txt                      the same lines as the standard output
replaced-a/... replaced-a2/...   with --replace-helpers only: the original bytes of every
                                 replaced helper, at its path in the tree
```

The output directory must be new, and it and the build directory must be
outside all three trees. The tool is implemented in `xtask/src/speed_ab.rs`.
