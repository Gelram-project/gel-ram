# Re-runs of the recorded speed sets, 2026-11-04

The four plans in [speed-campaign](../../speed-campaign/) run once with
`xtask speed-campaign`, as described in
[the campaign protocol](../../EVIDENCE-CAMPAIGN.md#re-running-the-recorded-speed-sets).
One directory per plan:

- CAMPAIGN.txt: the plan and its SHA-256, the three trees with their full
  revision and whether each was clean, each example with its expected and actual
  verdict, and the last line `SPEED_CAMPAIGN=COMPLETE`.
- one directory per example, written by `xtask speed-ab`: manifest.txt (the
  `GEL_SPEED_AB_2` manifest with the comparison protocol, host metadata, the
  binary SHA-256 of each tree and the order of each round), summary.txt (one
  `SPEED_AB` line per case: medians of A, A2 and B in nanoseconds, `aa` = A2/A,
  `gain` = A/B; the `SPEED_CHECK` digest of each tree; the verdict),
  a-rN.txt, a2-rN.txt, b-rN.txt (each measured round) and w-a-r1.txt,
  w-a2-r1.txt, w-b-r1.txt (the warm-up round, kept and not counted).

Every record passes `speed-ab-check` with `timing_validity=AS_MEASURED`:

```sh
cargo run --locked --offline -p xtask -- speed-ab-check docs/speed-results/2026-11-04-reruns/PLAN/EXAMPLE
```

## Setup

| Item | Value |
|---|---|
| Trees | A and A2: two clean checkouts of the plan's `baseline`; B: a clean checkout of its `candidate` (all public revisions) |
| Tool | `xtask speed-campaign` from `main` at `4ffab16`, 3 measured rounds and 1 warm-up round, order rotated so that each tree takes each position once |
| Runs per round | 9 (`speed_bench_physics`, `speed_bundle_memory` and `speed_collection_memory`: 3) |
| Toolchain | `rustc 1.85.0 (4d91de4e4 2025-02-17)`, release profile of each tree, no `RUSTFLAGS` and no cargo variable that changes code generation |
| Machine | the author's machine: AMD Ryzen AI 9 HX 370, 24 hardware threads, 94 GiB RAM, Linux 6.17 |
| Load | no other measurement or build running; load average 1.4 to 4.7 at the start of the examples (each manifest gives its own) |
| Pinning | none: runs were not pinned to cores |
| Duration | 1 h 40 min for all four plans, builds included |

## What it shows

| Plan | Programs | Cases | Outcome |
|---|---|---|---|
| [against v0.6.1](rerun-2026-10-20-against-v0.6.1/CAMPAIGN.txt) | 23 (`speed_top1_many` skipped: its API does not exist in v0.6.1) | 133 | 105 faster beyond their A/A band, 24 slower beyond it, 4 inside it |
| [package 2](rerun-2026-10-25-package-2/CAMPAIGN.txt) | 6 | 41 | ratios in each summary |
| [package 3, store](rerun-2026-10-25-package-3-store/CAMPAIGN.txt) | 1 | 12 | ratios in the summary |
| [package 3, loads](rerun-2026-10-25-package-3-loads/CAMPAIGN.txt) | 3 | 9 | ratios in each summary |

A case is beyond its A/A band when `gain` lies outside the interval between
`aa` and 1/`aa`; this rule gives 103, 18 and 12 for the
[record of 2026-10-20](../2026-10-20-against-v0.6.1/README.md). Of the 103
cases faster there, 102 have a ratio above 1 again, 100 of them beyond their band. The cases slower beyond the band are
those of loading a corpus, a bundle or a collection, the SHA-256 runs, the
backup inspection and restore and a few within 4% of 1; on this quieter
machine the A/A band is narrower (median |aa − 1| 0.004 here, 0.04 on
2026-10-20), so some loads that sat inside the band before now sit below it.

Every example prints the same output digest in A, A2 and B, except
`speed_tree_gates`, which counts the files of the tree it runs in. The digests
equal those recorded on 2026-10-20 and 2026-10-25 for every example and tree
but one: B of `speed_tree_gates`, because the B of 2026-10-20 was a local tree,
not the public commit.

The plan for the package-3 loads uses public revisions (`3cf1be5` against
`331f494`), while the record of 2026-10-25 compared `speed_source_load` and
`speed_collection_memory` from `37d509d` against a local tree. The gains of
1.07 to 1.18 recorded there for `speed_source_load` are not reproduced here
(0.99 to 1.07): the two are not the same comparison.

These are timings of one machine. A re-run on other hardware is the open part of
[A20](../../ROADMAP.md#review-work-list-and-next-acceptance).
