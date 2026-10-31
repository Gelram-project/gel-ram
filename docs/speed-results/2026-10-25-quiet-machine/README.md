# Timing on a quiet machine, 2026-10-25

The records behind the table "Measured on a quiet machine" in
[ROADMAP](../../ROADMAP.md). The files in each directory are the same kind as in
[2026-10-20-against-v0.6.1](../2026-10-20-against-v0.6.1/README.md):

- manifest.txt: the `GEL_SPEED_AB_1` manifest written by `xtask speed-ab`, with
  the tree roots written as `A`, `A2` and `B`, the scratch directory of an
  example as `WORK_DIR`, and the waiting hook described instead of named.
- summary.txt: one `SPEED_AB` line per case (medians in nanoseconds, `aa` =
  A2/A, `gain` = A/B), the `SPEED_CHECK` digest of each tree and the verdict.
- a-rN.txt, a2-rN.txt, b-rN.txt: the full output of each run, round N.

## Setup

| Item | Value |
|---|---|
| package-2 | A and A2: two clean checkouts of `bb3da1b` (before PR #87); B: a clean checkout of `90bd091` (`main` after PR #87) |
| package-3, `speed_store_overlap` | A and A2: two clean checkouts of `bb3da1b`; B: a local tree equal to this change in `Cargo.toml`, `Cargo.lock`, `tools/` and `crates/`, except the later collection change in `crates/gel-source/src/collection.rs` and a later doc comment in `crates/gel-store/src/lib.rs` |
| package-3, `speed_bundle_memory` | A and A2: two clean checkouts of `3cf1be5` (`main` before this change); B: a local tree equal to this change in `crates/gel-source/src` except the later collection change in `crates/gel-source/src/collection.rs`, in the example, `tools/` and `Cargo.lock` |
| package-3, `speed_collection_memory` and `speed_source_load` | A and A2: two clean checkouts of `37d509d` (this change before the collection change); B: a local tree equal to this change in `Cargo.toml`, `Cargo.lock`, `tools/` and `crates/` |
| Tool | `xtask speed-ab`, 3 rounds, order rotated each round |
| Runs per round | 9 (`speed_bundle_memory` and `speed_collection_memory`: 3) |
| Toolchain | `rustc 1.85.0 (4d91de4e4 2025-02-17)`, release profile of each tree |
| Machine | AMD Ryzen AI 9 HX 370, 24 hardware threads (8 at up to 5.16 GHz, 16 at up to 3.29 GHz), 94 GiB RAM, Linux 6.17 |
| Load | load average 0.8 to 4.0 during package-2 and the bundle load, 3.3 to 3.5 during the package-3 store runs, 2.6 to 2.8 at the start of the collection runs, with no other measurement running. Two earlier collection runs at load averages of 3 to 11, while other work was running on the machine, gave `collection_load_64mib` gains of 0.84 (A/A 1.35) and 0.81 (A/A 0.97); they are not published here |
| Pinning | none: runs were not pinned to cores |

Every example printed the same output digest in A, A2 and B
(`SPEED_AB_VERDICT=SAME_OUTPUT`).

## Peak memory of a bundle load

`speed_bundle_memory 1` built in release from each tree and run three times under
`/usr/bin/time -v`; "Maximum resident set size":

| Tree | Run 1 | Run 2 | Run 3 | Output digest |
|---|---|---|---|---|
| `3cf1be5` | 134 168 KiB | 133 980 KiB | 134 072 KiB | `38222401…c2c0` in every run |
| this change | 68 676 KiB | 68 244 KiB | 68 612 KiB | `38222401…c2c0` in every run |

## Peak memory of a collection load

`speed_collection_memory 1` built in release from each tree and run three times
under `/usr/bin/time -v`, alternating the trees, on 2026-10-31 (a first series on
2026-10-25, before a comment and test change, gave 133 248 to 133 280 KiB and
68 608 to 68 956 KiB):

| Tree | Run 1 | Run 2 | Run 3 | Output digest |
|---|---|---|---|---|
| `37d509d` | 133 080 KiB | 133 160 KiB | 133 176 KiB | `6777ef0b…7ac6` in every run |
| this change | 68 700 KiB | 68 364 KiB | 68 640 KiB | `6777ef0b…7ac6` in every run |

## Repeating it

```sh
cargo run --locked --offline -p xtask -- speed-ab \
  --a A --a2 A2 --b B --package PKG --example NAME \
  --target-root BUILD_DIR --out NEW_DIR --rounds 3
```

with `--with-bins gel-bench` for `speed_bench_checks` (docs/SPEED-AB.md).
Results on other machines will differ; the output digests should not.
