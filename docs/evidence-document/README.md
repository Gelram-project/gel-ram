# Document probe evidence

2026-09-19, Linux 6.17.0-1030-oem, x86_64, Ryzen AI 9 HX 370,
24 logical CPUs, MemTotal 98474964 KiB visible to Linux.
Rust 1.85.0 / LLVM 19.1.7, workspace release profile and portable target.
One worker, network-isolated local execution; background desktop processes
were active. No affinity/CPU-frequency isolation or cold-cache claim.

[Raw CSV](latency.txt), [generator metadata](probe.txt),
[independently recomputed statistics](summary.txt), [real-document demo](demo.txt).
The CSV has a .txt suffix for the repository's bounded source allow-list.

The probe generated 16097 bytes of fictional text, SHA256
`b621ad1c734b4642d8c3cd4eef4db76f5c46db814349b469562e2401d3ef5c2c`.
It executed 1000 searches: 250 each HIT, MISS, CONFLICTING_QUOTES, UNICODE.
Known phrases repeat; this is not 1000 independent knowledge questions.
Class p50: 23.755–24.506 microseconds. The full maxima and slower observations
remain in the CSV. No speedup claim is made from this single public run.

Only search is timed; source generation, hashing, printing, disk/ORB access,
encryption and startup are outside that timer. The separate demo uses the
real MIT Rust Book fixture and reports its own validation/search timing.
These metrics are not interchangeable with encrypted readout or Ocean scan.

Historical probe source SHA256 (before the CR and sigma review fixes):

```text
4b4c5fd794796a6d65edc464b798e9e2524e06db82cd07b8b32c90441012966b  crates/gel-source/src/document.rs
0246c4df1508c98b5de2b09df86f3983520fd6ba78339d3198ad2399f97ccac2  crates/gel-source/examples/document_bench.rs
```

These source bytes are retained in Git commit
`1d9dc4880dde2c648969d49ffe36b9fa92ba436b`. The raw measurements have not been
rerun or relabelled as performance of the corrected reader. Use the current
generator to measure the current implementation on your own machine.

Recompute with `cargo run --locked --offline -p gel-source --example document_stats -- docs/evidence-document/latency.txt`.
Nearest-rank percentiles use actual per-class N. At N=250 p99 describes only
a few tail observations; no stable p99.9/p99.999 claim is supported.
The checker rejects missing/duplicated IDs, class/count mismatches and invalid
durations, but does not authenticate the origin of a CSV supplied by a stranger.
