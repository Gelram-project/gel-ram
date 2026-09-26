# Measurement protocol and comparison rules

A timing number is only comparable with another when both describe the same
operation contract on the same inputs under a recorded setup. This page states
what a timing record must contain, what each current program actually prints,
and which comparisons are not valid. It changes no historical result.

## What a timing record must state

1. **Profile:** debug or release, plus the complete `RUSTFLAGS` (see
   [PERFORMANCE.md](PERFORMANCE.md)); no program records `RUSTFLAGS`, so
   record them in the run notes.
2. **Hardware:** CPU model, OS and architecture.
3. **Threads:** requested and effective workers, and any fallback.
4. **Input:** dataset name, size and hash or generator.
5. **Seed** of every generator used.
6. **Warm-up:** how many rounds ran untimed.
7. **Clock scope:** exactly what lies between start and stop.
8. **Instrumentation:** recorder, profiler or telemetry active while timing.
9. **Operation:** the contract being timed.

Keep every raw observation in execution order, including slow ones. When a
field is not printed by the program, record it in the run notes; a record
missing it is not comparable.

## Percentiles on small samples

Percentiles here are empirical order statistics of one run, never a guarantee
about the tail of a distribution. At least these index rules are in use
(ascending sort, zero-based index):

- `sorted[ceil((n-1)*q)]`: gel-bench and gel-physics. p99 is the maximum
  whenever n < 101, p95 whenever n < 21.
- Nearest rank `sorted[ceil(q*n)-1]`: collection_campaign and
  collection_recheck, document_stats, quantization_matrix (median and p95 of
  21 rounds) and mutation_compare, whose fixed indices 14, 28 and 29 for its
  30 samples are the nearest-rank p50, p95 and p99. p99 is the maximum
  whenever n < 100.
- Middle element `sorted[n/2]` as the median: topk_compare (11 samples per
  mode) and the Q8 run summarizer
  [summarize.rs](evidence-q8-current/summarize.rs). For the median this is the
  same index as the first rule; for an even n it is the upper of the two middle
  values, where nearest rank takes the lower one.

quad_compare and quad_evidence print per-round rows and ratios of round
totals, no percentiles. gel-bench V5 prints `query_p99_is_max=` so the case
is explicit.

## What current programs record

After the 26 September 2026 changes (gel-bench V5, gel-physics sample order,
quad_compare V3 fallback label, quad_evidence warm-up declaration):

| Program | Profile | CPU model | Threads / fallback | Seed | Warm-up | Raw order |
|---|---|---|---|---|---|---|
| gel-bench (V5) | yes | yes (Linux) | yes, incl. fallback counts | yes | yes (1) | yes |
| gel-physics (F0_V3) | yes | no (topology, arch only) | not printed (1) | not printed | none (0) | yes |
| quad_compare (V3) | yes | no (arch only) | yes, incl. shared-scan fallback | yes | yes (1) | per-round rows |
| quad_evidence | no | no | yes (1) | not printed | yes (2) | per-round rows |
| topk_compare | no | no (arch, backend) | not printed (1) | partial | yes (2) | yes |
| collection_build_compare | release enforced, not printed | no | yes (1) | n/a | first sample omitted | yes |
| mutation_compare | yes | no (os, arch) | yes (1) | n/a | yes | yes |
| collection_campaign (historical, pinned) | no | no (os, arch) | yes (1) | n/a | partial | yes |
| quantization_matrix (historical, pinned) | no | no | not printed (1) | n/a | yes (1) | yes |
| gel-evidence, source_find, collection_review, gel-source | no | no | no | n/a | no (cold) | single observations |

Remaining gaps are listed rather than hidden. Historical pinned sources are not
edited; their records keep the fields they had.

Clock scopes that are easy to misread:

- gel-evidence `save_ns` includes serialization, SHA-256, the temporary file,
  file sync, the hard link and the directory sync. `load_ns` includes reading,
  validation, parsing and releasing the previously loaded bank. The films were
  recorded with the scripted recorder running; their times are single
  observations under that load.
- gel-bench round times with THREADS > 1 include creating and joining workers.
- quad_compare with workers > 1 includes thread start in each timed scan.

## Comparisons that are not valid

- Phrase lookup (µs, gel-evidence / source_find / collection search) against
  a numeric full scan (gel-bench, Ocean 1M/10M in ms). Different operation,
  data and size; the ratio is not a speed-up.
- Saving with sync against any in-RAM operation; loading from disk against
  in-RAM deserialization.
- gel-bench (160 MiB bank, 128-byte records, Top-1) against topk_compare
  (16 MiB, Top-K with result allocation) or against the Q8 scans (1152-byte
  records, f64 arithmetic), even per record or per byte.
- quad_compare against quad_evidence: different reference readers, warm-up
  (1 vs 2 rounds) and worker counts.
- A single-observation diagnostic against a campaign percentile.
- A historical campaign against a new run of a program with the same
  operation name but a different implementation (for example the collection
  root computation before and after streaming).
- Mutation costs at different collection sizes as if constant: the root is
  recomputed over the whole collection.
- gel-physics sequential bandwidth against gel-bench throughput unless the
  same `RUSTFLAGS`, tag, core/L3 domain and a working set larger than L3 are
  recorded for both.
- "p99" across programs: at the sample sizes used it is usually the maximum.
- Results from different profiles: CI runs some examples in debug.
