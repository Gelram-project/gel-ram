# Try GEL RAM: exact readout on your hardware

This demo uses the public core and deterministic synthetic data only. It does
not encode text, answer questions, or require private models. Start with the
[README setup](../README.md#quick-start), then run from the repository root.

## 1. Check the implementation

```text
git rev-parse HEAD
rustc --version
cargo run --locked --offline -p xtask -- verify
```

Keep the commit ID with your results. The final marker must be
`GEL_VERIFY_ALL=PASS`; any failure needs investigation before performance claims.

## 2. Compare old and new exact Top-K

```text
cargo run --locked --offline --release -p gel-reader --example topk_compare -- --orbs 8192
```

This small demo uses 1 MiB banks and a frozen v0.2.0 reference in the same
binary as the new reader. It runs uniform, clustered and all-tied banks;
K=1/8/32/256; and full/progressive search. It checks equal indices, scores,
tie ordering and progressive stage counts, and prints every timing sample.

Expected final marker: `TOPK_COMPARE_EXACT=PASS`. Each RESULT row reports
52/52 exact method outputs (including warmups) and stage agreement. Times
are nanoseconds. The old/new median latency ratio is greater than 1 for a
speedup and less than 1 for a slowdown. Preserve every row, not just the best.
There is no expected speedup on every CPU or workload.

For the documented 16 MiB synthetic matrix, omit `--orbs 8192`:

```text
cargo run --locked --offline --release -p gel-reader --example topk_compare
```

This portable command does not reproduce x86-64-v3 code generation. On a CPU
that supports x86-64-v3, a Linux example is:

```bash
RUSTFLAGS="-C target-cpu=x86-64-v3" CARGO_TARGET_DIR=target/v3 taskset -c 2 cargo run --locked --offline --release -p gel-reader --example topk_compare
```

Choose a CPU in your allowed affinity set; CPU 2 is an example, not a universal
setting. Do not run the v3 build on unsupported hardware. Record CPU model,
RAM, OS, compiler flags, affinity, load and power settings. Shared-host timings
can vary substantially; eleven measured queries do not establish reliable p99.
See [the full protocol](TOPK-BENCHMARK.md) and [all recorded results](VALIDATION-v0.2.1.md).

## 3. Separate GEL byte integrity from FP16/Q8 conversion loss

```text
cargo run --locked --offline --release -p gel-cli --example data_integrity -- target/integrity-demo-01
```

The output directory must not exist. On a second run choose a different name,
such as target/integrity-demo-02. Fixtures remain on disk for inspection;
nothing is overwritten. Expected final marker: `GEL_DATA_INTEGRITY_ALL=PASS`.

GEL must recover every encoded byte exactly for all representations. FP16
and reference Q8 can still differ numerically from the original FP32 values:
that is conversion loss, not corrupted readout. The audit prints sizes and
numeric errors separately. It includes no model inference or GPU benchmark.
The [integrity protocol](DATA-INTEGRITY.md) specifies the Q8 format and tests.

## 4. Compare four Q8 views with one shared scan (unreleased, on main)

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_compare -- --orbs 8192 --rounds 9 --workers 1 --sparse 1 --policy active
```

Expect `Q8_QUAD_COMPARE_V2` and `Q8_QUAD_EXACT=PASS`.
Repeat with `--workers 24` only as a requested budget; record the effective
budget reported on your host. Keep all samples and both reference fallback
counts. A `TIMING_COMPARISON=DEGRADED_REFERENCE_SERIAL_FALLBACK` marker means
the reference did not complete with its intended parallel budget.

This phase-Q8 record uses 1024 phase bytes plus a 128-byte activity mask.
It is not the numerical reference-Q8 conversion in step 3, nor the 128-byte
binary ORB. Four matching coordinate transforms preserve the same score;
they do not add four independent facts. Semantic accuracy is not measured.

See the [contract and full matrix commands](Q8-QUAD.md),
[historical V1 timings](Q8-QUAD-RESULTS.md), and [V2 regression tests](Q8-QUAD-VALIDATION.md).
The existing v0.2.1 tag does not include this example.

## Share a useful result

Open a [reproduction report](https://github.com/gelramlicensing-wq/gel-ram/issues/new?template=reproduction.yml)
with your commit, exact command, hardware, flags, final PASS/failure markers
and all timing trials. Include regressions. Redact usernames, private paths
and any confidential content from logs. Never upload tokens or private banks.

An independently reproduced result or a small failing input is more useful
than a star alone. No code contribution or signed agreement is needed merely
to report a test result; code contributions follow [CONTRIBUTING](../CONTRIBUTING.md).
