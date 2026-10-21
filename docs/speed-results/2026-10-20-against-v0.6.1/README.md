# Timing against v0.6.1, 2026-10-20

The records behind the table "Measured against v0.6.1" in
[ROADMAP](../../ROADMAP.md). One directory per timing example:

- manifest.txt: the `GEL_SPEED_AB_1` manifest written by `xtask speed-ab`.
  It lists the package and example, the SHA-256 of every carried source file
  (the example and `tools/speed_support.rs`), the three trees with their
  revision and `rustc`, the files placed into A and A2, the order of each round
  and whether the trees were restored. Tree roots are written as `A`, `A2` and
  `B`, and the waiting hook is described instead of named.
- summary.txt: one `SPEED_AB` line per case (medians of A, A2 and B in
  nanoseconds, `aa` = A2/A, `gain` = A/B), the `SPEED_CHECK` digest of each
  tree and the verdict.
- a-rN.txt, a2-rN.txt, b-rN.txt: the full output of each run, round N.

## Setup

| Item | Value |
|---|---|
| A and A2 | two clean checkouts of the v0.6.1 tag, `db9cee36bca3c589fe4f6e0395aaaf89ca8b2b1d` |
| B | a local integration tree equal to `main` at `876247f` in `tools/`, `Cargo.toml`, `Cargo.lock` and every file under `crates/` except `crates/gel-source/examples/identity_tree_gates.rs`, an identity program that times nothing |
| Tool | `xtask speed-ab` built from B, 3 rounds, order rotated each round |
| Runs per round | 9 (`speed_bench_physics`: 3) |
| Toolchain | `rustc 1.85.0 (4d91de4e4 2025-02-17)`, release profile of each tree |
| Machine | AMD Ryzen AI 9 HX 370, 24 hardware threads, 94 GiB RAM, Linux 6.17 |
| Load | load average 2.7 at the start, 12.6 to 19.4 at the end, from other work on the machine |
| Pinning | none: runs were not pinned to cores or to one cache domain |

Because the runs were not pinned and the machine was loaded, the A/A column in
each summary.txt is the measure of noise to read every ratio against.
`speed_top1_many` has only a manifest: its API does not exist in v0.6.1, so the
example does not build there. `speed_tree_gates` prints `DIFFERENT_OUTPUT` because
it counts the files of the tree it runs in, and the two trees differ.

## Repeating it

```sh
cargo run --locked --offline -p xtask -- speed-ab \
  --a A --a2 A2 --b B --package PKG --example NAME \
  --target-root BUILD_DIR --out NEW_DIR --rounds 3
```

with A and A2 two checkouts of v0.6.1 and B a checkout of `main` (docs/SPEED-AB.md).
Results on other machines will differ; the output digests should not.
