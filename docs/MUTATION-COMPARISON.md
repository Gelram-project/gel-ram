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

## Separate Linux process-memory experiment

```sh
cargo build --locked --offline --release -p gel-source --example mutation_compare
target/release/examples/mutation_compare --memory stream 256 replace
target/release/examples/mutation_compare --memory historical_vec 256 replace
```

Run each command in a fresh process. Supported sizes are 8, 64, 256 and operations
are add, replace, remove. This mode builds only the selected bank, samples Linux
VmRSS and VmHWM before and after one mutation, then serializes and checks its
result with the current decoder and SHA256 oracle. Compare result roots and
lengths across variants before comparing memory. Preserve every output and
repeat with alternating execution order for a performance campaign.

The reported bytes convert Linux kB by 1024. VmHWM is a lifetime high-water mark:
it includes bank construction and is not reset at the operation boundary.
RSS includes process/runtime state and allocator-retained pages. Sampling and
reading proc also have overhead. These are not allocation counts, copied-byte
counts, or a precise isolated mutation peak. A zero HWM increase does not mean
zero allocation. The single operation latency is diagnostic, not a percentile.
No unsafe allocator hook is added; workspace safety policy is unchanged.

## Optional external allocation profiling

Build a separate release binary with debug symbols and without symbol stripping.
The memory mode contains named, non-inlined profile_current_change and
profile_historical_change boundaries around the mutation only. External stack
profiling can therefore exclude bank construction and the later oracle without
linking a profiler into GEL or changing the workspace's unsafe-code policy.

```sh
CARGO_TARGET_DIR=../profile-target CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=none cargo build --locked --offline --release -p gel-source --example mutation_compare
heaptrack --record-only -o ../new-stream-profile ../profile-target/release/examples/mutation_compare --memory stream 256 replace
heaptrack_print ../new-stream-profile.zst --filter-bt-function profile_current_change -F ../new-stream-stacks.txt
```

Use fresh output names. Repeat for historical_vec with the historical boundary;
preserve the trace, executable/source hashes, tool version and result roots.
Compression suffix depends on the installed profiler. This is an optional Linux
diagnostic, not a new build dependency or a required service.

For heaptrack 1.5, scope counts to the exported filtered allocation stacks.
Its final report totals and allocation histogram can still describe the whole
process despite the backtrace filter; do not label them mutation-only numbers.
Check a known allocating control before interpreting an empty filtered stack.
Allocation calls, requested bytes, memory-copy volume, heap peak and RSS are
different metrics. This procedure does not measure memory-copy volume.

Profiling changes execution time and process memory. Never mix its latency or
RSS readings with uninstrumented benchmark results. Raw symbolized profiles can
contain local paths: review/redact before any publication. No trace or profiler
binary is added to this repository by these instructions.
