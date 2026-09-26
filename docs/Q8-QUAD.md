# Experimental Q8 Quad readout (public since v0.3.0) — not a knowledge encoder

This additive module does not change the ORB128 format, binary Store, existing
Reader16, weights, Top-K ordering or current public release version.
It uses only Rust std. There are no private datasets, models or runtime paths.
GEL RAM-owned material is licensed under GEL RAM NCRL 1.0 ([licensing](../LICENSING.md)).
This export was first distributed under PolyForm Noncommercial 1.0.0; historical
releases retain the license under which they were originally distributed.
The complete local matrix is in [measured results](Q8-QUAD-RESULTS.md).

## Exact contract

A Record holds1024 phase indices in Z256 and1024 explicit activity bits:
1024 +128 =1152 bytes on the tested ABI. This is an in-memory representation,
not a new persistence/serialization format. It is not an exact conversion of
arbitrary floating-point data into128 bytes. Quantization loses phase precision;
the readout preserves the already encoded Q8 values.

With active query indices Q, the histogram counts `(q[j] - b[j]) mod256`.
The result is `sum_d histogram[d] * cos(2*pi*d/256) / len(Q)`.
Histogram reduction uses the fixed order0..255. The same-pole transformed
reference uses that same reduction order, allowing a bitwise comparison on
the same build/platform; libm/cos bits across all platforms are not promised.

- `Policy::Archive` deliberately ignores body activity for historical comparison.
  Inactive body phases can contribute. Do not interpret it as a missing-data gate.
- `Policy::BodyActivity` contributes zero where the body is inactive, but
  keeps the **query-support denominator**, not the intersection size.
- Phase0 may be active. It is not the silence marker.
- An empty query mask gives None/false and clears previous output.
  An inactive body with an active query gives score0 in BodyActivity mode,
  not a semantic UNKNOWN decision. No answer-abstention calibration is included.

The four poles combine coordinate reversal and seeded additive offsets modulo256.
Both sides must use the same seed, pole and transformed masks. Their difference
histograms are then equal. `SharedScore::at_pole` exposes the same scalar for
each valid pole; it does not calculate four independent confidence votes.
Cross-pole comparisons, different transforms and four different contents are
outside this shortcut. Grid also retains a small Z5 geometry helper for
compatibility tests; no Q2.5 codec, residual encoder or text features are exported.

## Resources and errors

Requested workers are capped by OS-reported parallelism,64 and record count.
The budget includes the calling thread. With one worker no OS thread is created;
empty banks/queries do not launch workers. Failure to start a worker causes a
complete serial recomputation after started threads finish. The budget is
per-call, not a process-wide scheduler; simultaneous callers can still oversubscribe.
The current comparison example (header Q8_QUAD_COMPARE_V3) applies this recovery
to both reference paths as well. It prints reference and shared-scan fallback
counts (including warm-up); any such fallback marks the timing comparison as
degraded, not a normal parallel run. V1/V2 logs keep their own headers.

Zero requested workers returns an error and clears output. Score-buffer
reservation is fallible. Unit tests cover usize::MAX requests, partial chunks,
empty banks/masks, active zero, invalid poles, buffer reuse and both policies.
Six additional example tests cover first/partial worker-start refusal, completion
of started workers, unchanged scores for both reference widths and policies,
output reuse, and no spawning for serial/empty inputs. A separate unprivileged
Linux process-limit test exercises a real OS refusal. This does not certify
all OOM behavior or recovery from panics inside a worker; see [validation](Q8-QUAD-VALIDATION.md).

## Reproduce

```text
cargo test --locked --offline -p gel-phase-quad --all-targets
cargo test --locked --offline --release -p gel-phase-quad --all-targets
cargo run --locked --offline --release -p gel-phase-quad --example quad_compare -- --orbs 8192 --rounds 9 --workers 24 --sparse 1 --policy active
```

The example caps records at32768 and rounds at99 before allocation. Defaults:
512 records,9 timed rounds,1 worker, dense masks, BodyActivity. Seed510051,
varying self-derived queries with one phase perturbation, one warm-up round.
This is intentionally a kernel test, not a semantic held-out dataset.

Three methods rotate order per round:

1. ReferenceOne: one reference score from P0 in the materialized four-view layout.
2. FourViews: independent reference scores over all four materialized views.
3. Shared: one canonical scan, exposing the result for all four poles.

Views and active-index lists are prepared outside the timer. Every method
reuses its output buffer. Scan timing includes per-call thread handling and
output writes, excludes view construction, query preparation, ranking and
correctness checking. ReferenceOne is not an optimal canonical single
reader: **Shared already is that single read**. ReferenceOne uses the old
larger layout; its comparison mixes implementation/layout effects.

For every query, each record and each reference view must match Shared
bit-for-bit; the mean of four reference values is checked as well. All bank
views and masks are reversed and compared before timings. Final marker:
`Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED`.

Four packed canonical records would occupy4608 bytes versus1152:75% less
record material, not75% less process RAM. The actual reference Frames use
bool masks and occupy8192 bytes per four-view group. Grid tables, query
indices, outputs and benchmark-side duplicate banks also consume memory.
The example prints both denominators, not just the more impressive number.

No physical analog RAM resonance, autonomous inference,99% knowledge
retrieval or universal hardware speedup is established by this experiment.
