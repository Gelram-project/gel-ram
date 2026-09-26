# R1 evidence: executable negative cases

The integration test in
[collection_recheck_cli.rs](../xtask/tests/collection_recheck_cli.rs) compiles
the unchanged measured checker and executes it as a separate process against
temporary copies of the actual published R1 CSV files. Historical evidence and
its checker source are not rewritten.

```sh
cargo test --locked --offline -p xtask --test collection_recheck_cli
```

There are 26 subprocess cases inside one Rust integration test:

- Two accepted inputs: the original matrix and a consistent reverse ordering of
  both files. Identities, not positions, determine membership.
- Nine raw-field refusals: false/invalid correctness flags, repetition outside
  the campaign, wrong bank size, unknown operation, sample outside its range,
  negative/overflowing duration and a changed duration inconsistent with summary.
- Seven summary-field refusals: count, minimum, p50, p95, p99, maximum and total.
- Eight further refusals: duplicate/missing raw row, duplicate/missing summary
  row, unknown summary operation, invalid summary number and two changed headers.

Every negative case requires a nonzero process exit, the expected error class
and no PASS on stdout. The setup checks that each field mutation really changes
the input. This exercises actual process behavior, not merely a reducer function.
It runs with the normal workspace all-targets test target; platform success must
still be checked at the exact CI revision.

The statistical checker detects internal inconsistencies. It does not authenticate
an author, reconstruct historical timing, or detect a coordinated replacement of
raw values and matching summaries by itself. The separately pinned source and
evidence manifests cover byte identity; changing those pins is a distinct review
decision. These finite cases are not exhaustive fuzzing or semantic validation.
