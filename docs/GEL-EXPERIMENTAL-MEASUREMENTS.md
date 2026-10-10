# GEL experimental measurements — author-reported, CPU/RAM

These additional measurements describe components of a private GEL prototype.
They are **not** a replacement for the reproducible [Ocean Scale](OCEAN-SCALE.md)
benchmark, a commercial-product comparison, or a claim of full AI performance.
No private engine, source bank or other private component is included.

## Replayed compact-sketch search — 26 September 2026

A query searches for a minimum-distance sketch without being given its address.
Each sketch occupies 128 bytes. This is not the 1152-byte public Q8 record.

| Sketch count | Run 1 p50 µs | Run 2 p50 µs | Run 3 p50 µs |
|---:|---:|---:|---:|
| 16,384 | 21.310 | 15.521 | 17.918 |
| 131,072 | 337.910 | 279.205 | 269.472 |
| 294,378 | 885.044 | 750.340 | 728.198 |
| 500,000 | 1440.327 | 1326.322 | 1264.046 |

Each run contains 201 measured batches per size after three warm-up rounds.
Data and the query are synthetic and reused. The reported time is wall-clock
batch duration divided by scans per batch, not cold single-request latency.
Three workers search disjoint partitions; their minima are reduced globally.
Thread creation/join and result checks are included. Data generation and an
independent bit-by-bit oracle are outside the timer. All three processes
completed without a failed oracle assertion. This is numerical correctness,
not semantic recall or multimedia understanding.

Hardware: AMD Ryzen AI 9 HX 370, Linux, three workers restricted to logical
CPUs 0–2; Rust 1.97.0, optimized native-CPU build. Frequency, temperature and
other desktop activity were not controlled. No GPU inference or LLM is used.
The 500,000-sketch payload is 64,000,000 bytes, not total process RSS.

The historical combined median for 500,000 sketches was 949.195 µs.
The new runs did **not** reproduce that speed. The historical toolchain was
reported as Rust 1.98.0, and host conditions were not held identical.
We do not attribute the difference to a proven code regression.

Raw numeric observations: [run 1](evidence-gel-components/run-1.csv),
[run 2](evidence-gel-components/run-2.csv), [run 3](evidence-gel-components/run-3.csv).
For each row, amortized microseconds = wall_ns / batch / 1000.
Sort the 201 values per size; zero-based element 100 is p50 and 190 is p95.
All measured rows, including slow ones, are retained. Recomputing statistics
from these files is possible; reproducing the private engine is not.
The CSV's checks field counts worker scans, not independent semantic queries.

## A second private component — historical timing (19 September 2026)

Author-reported timings of a second private component were compared with a
reference implementation on a small and a larger input (ratios 1.07–1.74×). The
implementation and raw data remain private; these figures measure a different
task and establish no end-to-end advantage.

## What these numbers do not establish

- Sketch search, the second private component and the full Q8 scan are different
  tasks. Do not divide their times to claim an end-to-end speedup.
- No 1M/10M sketch replay or equivalence to the Q8 top-10 ranking is established here.
- The computations execute on CPU with data in memory. Hardware-level memory
  computation and permanent CPU-cache residency are not proven.
- There is no measured superiority over a named commercial system.

A valid next comparison needs the same corpus, queries, accuracy target,
top-k and HIT/UNKNOWN decision, with preparation, updates and memory counted.
Only non-sensitive numerical observations and scope descriptions are released.
