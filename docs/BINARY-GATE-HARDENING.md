# Binary-core hardening — not part of the v0.2.1 release tag

This changes the binary reader/codec/benchmark, not phase-Q8 scoring or
semantic ranking. Historical logs and the v0.2.1 tag are unchanged. The Q8
V1 charts must not be used as performance measurements of this change.

| Audit item | Change | Verification |
| --- | --- | --- |
| K01 | Caller scans before joining child workers | Two-way rendezvous; answer equality and deterministic ties retained |
| K02 | Normalize raw Rotate shift before subtraction | All 65536 u16 values; checked-overflow release probe |
| K03 | Sparse popcount uses the same canonical decoder validation as apply | Duplicates, ordering, padding, length/count errors; valid counts 0–1024 |
| K04 | Compare every Top-8 entry against independent per-bit scoring + full sort | Corrupt every index/score, tail order and list length |
| K05 | Reject incomplete timed Top-1 exactness with an error | One corrupted timed hit rejected; warm-up left correct |
| K06 | Explicit unwind versus abort contract | Worker panic caught only in unwind; release profile terminates |
| K07 | Report actual execution separately from requested workers | Empty/serial/guarded paths, partial starts and actual Linux start refusal |

The public ViewSpec struct is retained for compatibility; no mandatory new
opaque configuration type is imposed. Sparse counting now validates by
decoding the bounded position list; no performance improvement is claimed
for that extra validation. Worker budgets are per call, not process-wide.

## Reproduce the automated checks

```text
cargo run --locked --offline -p xtask -- verify
cargo test --locked --offline --release --workspace --all-targets
cargo test --locked --offline -p gel-reader caller_and_child_scans_overlap_before_join
cargo test --locked --offline -p gel-bench
```

The overlap test has a bounded wait, not a timing speedup threshold. Resource
exhaustion during that test is a failure to establish overlap, not a PASS.
The worker-panic unit test runs only where unwinding is enabled; Cargo test
behavior must not be mistaken for a release executable's abort behavior.

For a Linux-only, unprivileged child-process refusal probe:

```text
cargo build --locked --offline --release -p gel-bench
prlimit --nproc=1:1 --core=0:0 -- target/release/gel-bench 128 3 2
```

When the OS denies every worker start, expect correct answers and exit zero,
but `effective_workers_min=1`, `effective_workers_max=1`,
`spawned_workers_total=0`, a nonzero thread-start fallback count and
`TIMING_COMPARISON=DEGRADED_SERIAL_FALLBACK`. No other process's resource
limits are changed. Privileged accounts may bypass the limit; inspect the
actual report. See [gel-bench telemetry (V4 fields; current header GEL_BENCH_V5)](PERFORMANCE.md) for scope and denominators.

Private validation additionally runs deliberately corrupted benchmark copies
and overflow-checked release binaries. Fault injection is not a runtime switch
in the public benchmark. No new throughput or semantic-accuracy claim is made.

Local validation on 2026-09-08: Linux x86_64, Ryzen AI 9 HX 370 (12 cores,
24 logical CPUs), Rust 1.85.0 for the public workspace. Full verify and
release tests each passed 118 test executions. The independent integrity
audit passed 262400/262400 structural cases and 256/256 ranking queries.
Sixteen additional process checks included correct scans on 64/8192 records
with 1/2/24 workers, rejected invalid budgets, OS start refusal and deliberate
corruption/panic controls. Expected nonzero exits and SIGABRT were successful
negative controls, not accepted incorrect answers. The host was not isolated;
these runs establish correctness for these cases, not a new speedup.

## Outside this patch

No new store format, buffering/CRC optimization, global worker pool, P2P,
LLM, license change, CLA registry, full secret scanner or GPU implementation
is introduced. Those audit suggestions require their own scope and evidence.
