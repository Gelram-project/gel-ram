# Paired mutation comparison

```sh
cargo run --locked --offline --release -p gel-source --example mutation_compare -- ../new-mutation-comparison
```

Use the pinned Rust 1.85.0 toolchain after fetching dependencies. The output
directory must not exist. The example compiles the unchanged historical
collection and bundle snapshots alongside the current collection. Only add,
replace and remove are timed, not historical persistence code. Historical bundle
tests compiled with this example are repeated reference checks, not new coverage.

The deterministic synthetic corpus has 8, 64 or 256 documents of roughly 16 KiB
each. Each size/operation has one unrecorded warm-up and 30 recorded pairs.
Variant order alternates. State cloning, input construction, result serialization
and equality checks occur outside the timer. Every pair must have identical
serialized result bytes and roots, and the root must equal SHA256(serialization).
Timing includes the entire mutation, including document hashing and allocations
inside it. It does not isolate the root hash kernel.

Generated raw.csv preserves all observations and execution order; summary.csv
uses nearest-rank percentiles. With N=30, p99 is the observed maximum, not a
robust tail guarantee. inputs.txt identifies corpus and implementation hashes.
COMPLETE means the paired correctness checks passed, not a performance win.

This first harness does **not** measure allocation counts, allocator-internal
copies or peak RSS. Both states coexist and the allocator/cache are warm.
The source-level removal of a temporary serialization buffer does not itself
prove a measured RSS reduction. Memory profiling and repeated isolated runs
remain necessary before a broad optimization claim. Do not compare these
mutation timings with Ocean searches, Q8 decoding or private GEL/WAVE timings.
