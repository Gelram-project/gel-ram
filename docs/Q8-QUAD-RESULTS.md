# Q8 Quad shared readout — historical V1 local measurements (2026-09-08)

One canonical scan preserved the scores of four equivalent transformed views
in every run of this synthetic experiment. This demonstrates removal of
redundant computation, **not better semantic retrieval or four independent
pieces of knowledge**. At measurement time the export was an unreleased
PolyForm Noncommercial 1.0.0 preview; the module has since been released in
v0.3.0, with GEL RAM NCRL 1.0 as the operative license ([licensing](../LICENSING.md)).

## Protocol and hardware

This 48-run performance table is historical: it uses `Q8_QUAD_COMPARE_V1`,
preserved in commit `4e7805990874851907611d1e21f05d2a5c64db35` before the
reference-worker fix. The Q8 library scoring algorithm has not changed.
The later V2 example added fallible reference spawning and fallback counts; the
current example prints Q8_QUAD_COMPARE_V3. The V2
[separate regression validation](Q8-QUAD-VALIDATION.md) does not replace these
timings, and neither V2 nor V3 establishes a new speedup ratio for the changed runner.

Measured 2026-09-08 on AMD Ryzen AI 9 HX 370, 12 cores / 24 logical CPUs,
approximately 93.91 GiB OS-reported RAM, Ubuntu 24.04.4 LTS,
Linux 6.17.0-1030-oem. Rust 1.85.0, release, thin LTO, one codegen unit,
no RUSTFLAGS / target-cpu=native. CPU only; no GPU or LLM involved.
No CPU affinity or exclusive host isolation. Compilation and project tests
finished before measurement; desktop background processes were not stopped.
Host-wide activity per invocation is recorded, not proof of idle hardware.

The matrix was selected before measurements: 512 / 8192 records, dense / sparse
activity, Archive / BodyActivity, 1 / 24 requested workers (effective budgets
also 1 / 24 here). Three repetitions of every combination, nine timed queries
after one warm-up per invocation. Case order changes between repetitions;
the three method orders rotate within every invocation. Same seed and query
schedule across repetitions: repetitions measure timing variation, not new data.

See the [contract and exact reproduction command](Q8-QUAD.md) and
[all raw measurements, hardware and execution order](evidence-q8-quad/README.md).
The raw runner arguments are in RUNS.txt in that directory; no Python dependency
is needed to run the Rust example. The complete matrix contains 48 invocations,
432 timed queries and **8,355,840 exact view-score comparisons**, including
warm-up. These comparisons are correlated and are not a statistical estimate
of knowledge accuracy. All comparisons passed. Mean-of-four scores and
reversal of all stored phase/mask views were also checked.

## All matrix cells

[Color charts of every cell, with interpretation](../README-HISTORY-R2.md#measured-shared-readout-advantage--historical-v1)
and [chart provenance](Q8-VISUALS.md) use this unchanged V1 evidence.

Speed ratio = sum of FourViews times / sum of Shared times for one invocation.
Below are the median and full minimum–maximum of the three repetition ratios.
A value above 1 favors Shared. Latency is the median of the three per-run
median Shared query times, not end-to-end response latency.

| Records | Mask | Policy | Workers | FourViews / Shared median (min–max) | Shared ms/query |
| ---: | --- | --- | ---: | ---: | ---: |
| 512 | dense | Archive | 1 | 3.81 (3.74–3.83) | 0.349 |
| 512 | dense | Archive | 24 | 1.43 (1.26–1.53) | 0.482 |
| 512 | dense | BodyActivity | 1 | 3.57 (3.55–3.68) | 0.385 |
| 512 | dense | BodyActivity | 24 | 1.21 (1.07–1.39) | 0.506 |
| 512 | sparse | Archive | 1 | 3.48 (3.42–3.65) | 0.166 |
| 512 | sparse | Archive | 24 | 1.48 (1.29–1.61) | 0.363 |
| 512 | sparse | BodyActivity | 1 | 3.55 (3.43–3.62) | 0.573 |
| 512 | sparse | BodyActivity | 24 | 1.39 (1.38–1.53) | 0.468 |
| 8192 | dense | Archive | 1 | 4.44 (4.41–4.56) | 4.820 |
| 8192 | dense | Archive | 24 | 3.43 (3.28–3.46) | 1.191 |
| 8192 | dense | BodyActivity | 1 | 3.83 (3.78–4.17) | 5.853 |
| 8192 | dense | BodyActivity | 24 | 2.97 (2.91–2.99) | 1.358 |
| 8192 | sparse | Archive | 1 | 5.31 (5.16–5.61) | 2.511 |
| 8192 | sparse | Archive | 24 | 2.94 (2.78–3.11) | 0.830 |
| 8192 | sparse | BodyActivity | 1 | 4.04 (3.88–4.04) | 5.922 |
| 8192 | sparse | BodyActivity | 24 | 2.79 (2.66–3.07) | 1.117 |

The small-bank parallel cases show why “always use all cores” is not a sound
default. With 512 records, 24 workers were slower than one in three of the
four mask/policy cells. Thread setup is included in these times. No automatic
threshold was tuned on these measurements; callers still choose the budget.

## What the comparison does and does not establish

FourViews does four reference histogram reductions in materialized transformed
data. Shared does one reduction over the canonical record. Timings also differ
in data layout, mask representation, output width and thread-start API; this is
a comparison of complete scan implementations, not an isolated instruction test.
Values above 4 do not imply more than four independent reads were eliminated.
They can include layout/cache effects and timing variation.

ReferenceOne, also retained in every raw log, reads only P0 from the larger
four-view layout. Shared is **not uniformly faster than ReferenceOne**.
That arm is not an optimized canonical single-reader competitor. We make no
claim of superiority to all Q8 readers, vector databases or AI models.

Canonical record material is 1152 bytes versus 4608 for four packed copies
(75% reduction). The measured reference actually uses 8192 bytes for four
Frames with bool masks. Neither figure is peak process RAM or compression of
arbitrary knowledge. The benchmark holds both banks simultaneously for checking.

Only matching transforms of the same contents qualify for shared scoring.
Four genuinely different contents still require their own evaluation.
No new semantic Top1/Top10, full-knowledge gate, response-evidence gate,
physical analog RAM mechanism, macOS/Windows runtime or GPU result is claimed.

## Historical local verification status (051/052)

- Public-base candidate: 103 test executions in debug and 103 in release, all
  passing; includes 19 Q8 library tests and 5 example-harness tests. Four reference
  tests run in both harnesses, so totals are executions, not unique assertions.
- Workspace format check, Clippy with warnings denied, docs and binary selftest
  pass on Rust 1.85.0. Existing data-integrity example reports PASS.
- Source changes originate in the private master, with an explicit export
  mapping; ranking, selection, private banks, evidence/mouth adapters and models
  are not part of this export.
- The original051 snapshot stopped full verify at pending license confirmation.
  The author subsequently confirmed the public PolyForm license. The licensing
  check remains enabled. The subsequent052 full-gate rerun passed on Rust1.85.0
  (`GEL_VERIFY_ALL=PASS`), as did103 release test executions and the independent
  data-integrity example. All11 workspace packages resolve to PolyForm
  Noncommercial1.0.0 in Cargo metadata. These checks are separate from the
  unchanged051 benchmark. A new release has not been issued.
