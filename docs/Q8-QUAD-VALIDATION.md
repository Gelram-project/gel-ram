# Q8 benchmark V2 — worker-refusal regression validation

2026-09-08. This fixes the reference paths of the comparison example, not the
Q8 scoring formula, stored data, weights or semantic accuracy. The public
module remains an experimental PolyForm Noncommercial preview; the existing
v0.2.1 release tag is unchanged.

## What changed

ReferenceOne and FourViews now use fallible scoped thread creation. If a worker
cannot start, the example waits for started workers and recomputes the complete
reference output serially. No partially filled score buffer is accepted.
The shared library reader already had this recovery path and is unchanged.

The header is now `Q8_QUAD_COMPARE_V2`. It prints `reference_one_fallback_scans`
and `four_views_fallback_scans`, including warm-up. If either is nonzero it also
prints `TIMING_COMPARISON=DEGRADED_REFERENCE_SERIAL_FALLBACK`. Do not interpret
such timings as a successful full-worker performance run. These counters
describe the two reference arms, not a measurement of shared-reader scheduling
or instantaneous CPU utilization.

Six new regression tests exercise refusal before any child starts, after one
or two real children start, and normal parallel completion. Both one-view and
four-view outputs are checked bit-for-bit against serial reference scores,
with dense/sparse masks and Archive/BodyActivity policies, uneven chunks and
buffer reuse. Serial and empty-bank paths must not attempt a spawn. Failure
injection is local to a test-only spawner instance, not a production switch.

## Actual operating-system refusal

On the local Ryzen AI 9 HX 370 Linux host (12 cores / 24 logical CPUs), the old
and fixed Rust 1.85.0 release binaries were run as the same unprivileged user
with identical synthetic arguments. Only each child process had RLIMIT_NPROC=1
and RLIMIT_CORE=0; no other application's limits were changed.

| Same 32-record, 24-requested-worker probe | Result |
| --- | --- |
| V1 reference spawn | OS error 11, SIGABRT (shell status 134); no successful completion |
| V2 reference spawn | Exit 0; exact comparisons PASS; two fallback scans per reference arm, including warm-up |

[Before: raw output](evidence-q8-fallback/before-os-refusal.txt) ·
[After: raw output](evidence-q8-fallback/after-os-refusal.txt).
Source SHA-256 values and build metadata are included in both logs.

For an optional Linux reproduction of the **fixed** binary:

```text
cargo build --locked --offline --release -p gel-phase-quad --example quad_compare
prlimit --nproc=1:1 --core=0:0 -- target/release/examples/quad_compare --orbs 32 --rounds 1 --workers 24 --sparse 1 --policy active
```

Use an unprivileged account. A one-worker machine or a privileged process that
bypasses this limit may not exercise refusal; inspect the fallback counts.
Unit tests also exercise partial launch, which this all-refused OS probe does not.

## Post-fix matrix

All 16 synthetic correctness runs passed: 512/8192 records × dense/sparse ×
Archive/BodyActivity × 1/24 effective workers. Each used three timed rounds
and one warm-up, seed 510051. Neither reference arm fell back in these runs.
[Complete raw matrix](evidence-q8-fallback/smoke-matrix.txt).

These are regression checks on a shared, non-isolated host; other validation
work could be running. Their timings are diagnostic, not new performance claims.
The separate [48-run V1 performance campaign](Q8-QUAD-RESULTS.md) is preserved.
Neither campaign establishes end-to-end knowledge accuracy or whole-system
validation. This fix does not provide general OOM or worker-panic recovery.
