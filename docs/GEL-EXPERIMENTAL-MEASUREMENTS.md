# GEL experimental measurements — author-reported, CPU/RAM

These additional measurements describe components of a private GEL prototype.
They are **not** a replacement for the reproducible [Ocean Scale](OCEAN-SCALE.md)
benchmark, a commercial-product comparison, or a claim of full AI performance.
No private engine, source bank, encoder or speaker is included.

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

## Phase evolution kernel — historical measurements, 19 September 2026

| Active graph / steps | Run | N per implementation | Reference p50 µs | GEL candidate p50 µs | Ratio |
|---|---|---:|---:|---:|---:|
| 2 nodes / 20 | R3 | 24 | 11.5420 | 6.8375 | 1.688× |
| 2 nodes / 20 | R4 | 24 | 8.2350 | 4.7790 | 1.723× |
| 2 nodes / 512 | R3 | 24 | 289.0590 | 166.5350 | 1.736× |
| 2 nodes / 512 | R4 | 24 | 206.5500 | 119.4475 | 1.729× |
| 256 nodes, 8192 connections / 20 | R3 | 24 | 28081.9400 | 26222.0540 | 1.071× |
| 256 nodes, 8192 connections / 20 | R4 | 24 | 28023.7575 | 26175.9185 | 1.071× |

Same CPU family, Linux, Rust 1.97.0, release, one thread pinned to CPU8.
ABBA/BAAB order; p50 averages the two central observations.
The timer includes evolution and its allocations, but excludes graph creation,
ORB retrieval, answer construction and durable storage. These are active
graphs; large shortcuts for inactive graphs are not presented as active gains.
Recorded results matched the reference bitwise on tested inputs.
On 26 September, the saved CSV reduction was repeated and matched the saved
summary byte-for-byte. This was not a new execution of the phase campaign.
Its underlying implementation and raw campaign remain private: this table
is explicitly **author-reported**, not independently reproducible evidence.

## What these numbers do not establish

- Sketch search, phase evolution and Ocean full Q8 scan are different tasks.
  Do not divide their times to claim an end-to-end speedup.
- No 1M/10M sketch replay or equivalence to the Q8 top-10 ranking is established here.
- Phase graph nodes are not automatically Ocean ORBs.
- The computations execute on CPU with data in memory. Physical DRAM refresh
  computing, operation without CPU and permanent CPU-cache residency are not proven.
- There is no measured superiority over a named commercial system.

A valid next comparison needs the same corpus, queries, accuracy target,
top-k and HIT/UNKNOWN decision, with preparation, updates and memory counted.
Only non-sensitive numerical observations and scope descriptions are released.
