# Q8 evidence candidate — numeric sample, checked views, current measurements

Local candidate based on main `339f3649684683da3d8ee54d6fdcc1c624bdfabe`.
The [revision2 audit and repeat measurements](Q8-CANDIDATE-R2-AUDIT.md) add
fixture input hardening. The first campaign below remains a historical record.
Not a new tagged release. PolyForm licensing unchanged. No private application,
encoder, conversation, media bank, donor monolith or model is included.

## Try the sample now

After the existing quick-start dependency fetch:

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_evidence -- --demo
```

Expected: `Q8_EVIDENCE=PASS`. This constructs 32 deterministic numeric records
in memory: zeros, 255, ramps and pseudorandom phase codes, with dense and
alternating activity masks. It is deliberately synthetic, not a knowledge test.
It checks four checked-view inverses and all score comparisons, then reports
every timing round. It does not contact a network or use an LLM.

For a file example on Linux, use a NEW path outside the checkout:

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_evidence -- --generate /tmp/q8-example-new.q8demo 32
cargo run --locked --offline --release -p gel-phase-quad --example quad_evidence -- /tmp/q8-example-new.q8demo 9 active
```

The generated 32-record file is 36,876 bytes. SHA256:
`2997a04ce8a1b9d7b7b8a03e64d718a3353e6963bc6872b27db180a99bf5b055`.
Generation refuses to overwrite. Use `--help` for the interface.
Do not place generated banks inside the source tree's release allow-list.

## Numeric fixture contract

Q8DEMO01 is a testing interchange format, NOT a new persistent GEL store.
It has eight magic bytes, a little-endian u32 record count, then exactly
count records of 1024 phase bytes and 128 activity-mask bytes. Mask bit j
is bit `(j % 8)` of mask byte `(j / 8)`, least-significant bit first.
Count must be 1..8192. No extra bytes, titles, source IDs or executable content.
Maximum file size: 9,437,196 bytes. Input is read with an explicit byte cap;
invalid lengths/counts fail before allocating decoded records. File size is
not a cap on process RSS: canonical and materialized banks coexist for checking.
There is no authentication field; accept external fixtures only as untrusted data.
The CLI requires a regular file in a caller-controlled directory and rejects
symlinks/special files. This is not protection against concurrent path replacement.

## Checked in-memory views

`crates/gel-phase-quad/src/bound_view.rs` adds `Reader::bound_view` and
`Reader::restore_bound_view`, including phase and activity-mask reconstruction.
The descriptor fixes algorithm, dimensions, levels, reader seed and pole.
Unsupported descriptors and another reader's seed are rejected. Existing
low-level APIs remain unchanged; this is not automatic protection of every caller.

This is compatibility checking, **not cryptographic authentication**. A sender
can maliciously relabel a payload. An explicit regression test demonstrates
that limitation. Trusted manifests/signatures are still needed at trust boundaries.
Descriptors are in-memory values; no stable wire format, migration or private
decoder compatibility is introduced. This change does not claim to fix all
private variants merely because their names also contain Quad Grid.

## Current measurement protocol

2026-09-11, AMD Ryzen AI 9 HX 370, 12 cores / 24 logical CPUs, about 93.9 GiB
OS-reported RAM. Linux, Rust 1.85.0 release, thin LTO, one codegen unit,
portable project defaults. CPU only, no affinity or exclusive host isolation.
The owner identifies this host as MINISFORUM AI X1 Pro with 128 GB installed;
[installed capacity and OS-visible memory are distinct](HARDWARE.md).
Compilation and verification finished before the final campaign. Background
desktop processes were not stopped. Hardware/load snapshots accompany the logs.

Two separate campaigns, not interchangeable timing ratios:

1. **Existing Q8 V2:** 48 invocations, 512/8192 records, dense/sparse masks,
   Archive/BodyActivity, 1/24 requested workers, three repetitions, nine timed
   rounds plus one warm-up. Cell and method order rotate. All reported reference
   fallback counts are zero; the shared API has no fallback counter.
   **8,355,840/8,355,840 view-score comparisons passed.**
2. **New canonical baseline:** 24 invocations, 32/512/8192 synthetic records
   and 768 caller-supplied real records, both policies, three repetitions,
   one worker, two warm-ups and nine timed rounds. **2,509,056/2,509,056
   view-score comparisons passed**, including the real and synthetic sets.

The new scalar single-read reference scans the same packed Record layout as
Shared, rather than the old four-Frame allocation. It scans all dimensions
and inspects mask bits; Shared uses precomputed query-support indices. This
is a simple independent baseline, not the fastest possible canonical reader.
FourViews still materializes four Frames outside timing. Query preparation,
fixture loading and correctness checks are outside timing; output allocations
are reused. Memory costs are printed. No end-to-end application latency is implied.

Queries are selected bank records with one phase code perturbed, not held-out
natural questions. Every timed query's result is checked AFTER measurement.
Repeated comparisons share data; their count is not a semantic sample size.

### New canonical-baseline results

Ratios are the median of three run-level ratios, not ratios of median latencies.
Above 1 favors Shared. All rows, including slowdowns, are shown. Full min–max
ranges and all 16 V2 cells are in the [numerical summary](evidence-q8-current/summary.txt).

| Input / policy | FourViews / Shared | CanonicalSingle / Shared | Shared ms/query |
|---|---:|---:|---:|
| Real 768 / BodyActivity | 3.511 | 1.044 | 1.402 |
| Real 768 / Archive | 3.925 | 1.127 | 0.424 |
| Synthetic 32 / BodyActivity | 1.986 | 0.662 | 0.070 |
| Synthetic 32 / Archive | 2.121 | 0.679 | 0.032 |
| Synthetic 512 / BodyActivity | 3.657 | 1.226 | 0.333 |
| Synthetic 512 / Archive | 3.793 | 1.124 | 0.302 |
| Synthetic 8192 / BodyActivity | 4.408 | 1.213 | 5.186 |
| Synthetic 8192 / Archive | 4.731 | 1.256 | 4.446 |

For the real BodyActivity case, CanonicalSingle/Shared ranges **0.997–1.063**:
one repetition slightly favors the single reference. Do not advertise an
established universal single-reader advantage. The 32-record samples favor
the canonical single reader in all repetitions. Ratios above four in other
rows can involve data layout/cache effects, not extra independent information.

### Real input boundary

The external fixture contains 768 existing phase/mask records from a private
256-node encoded sample. Conversion copied all phase and mask bytes unchanged,
without re-quantization. The 884,748-byte test fixture SHA256 is
`23d5f20f4268992f51e6c8d1f838b305bcd96c75563f51ec0734bda2b12a4ce4`.
Each of six invocations passed 3072 view inverses and 33,792 score comparisons:
18,432 inverse checks and 202,752 comparisons across repeated use of this sample.

Only numeric timing/results are included, not the real fixture, original text,
source IDs, private encoder or corpus. These real-input observations cannot
be independently reproduced from this archive alone. The complete synthetic
paths can. They establish numeric readout parity, not verified source facts,
knowledge recall, four independent files, hardware RAM signatures or a PUF.

## Reproduction and remaining limits

See [campaign instructions and raw logs](evidence-q8-current/README.md).
The historical V1 results remain unchanged. New V2 results do not retrospectively
alter them. Full Linux checks and source review are recorded in
[candidate validation](Q8-CANDIDATE-VALIDATION.md). Remote CI has not been run
for this unpublished candidate; no new Windows/macOS runtime result is claimed.
