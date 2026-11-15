# Additive Q8 / QuadGrid evaluation plan

Status: **PLAN ONLY**. This document is not evidence that a new codec works and does not alter the existing public Q8 implementation.

Branch: `fix/q8-256-qg4-contract`. The target is research and compatibility validation, **not** an immediate change of the public default.

Status update (2026-11-14): a private prototype and sealed private test fixtures exist outside this repository. Nothing from them (code, data, method or results) is published in this branch. The 256 B candidates in section 2 remain unimplemented. Any public step still needs the gates in sections 4 and 5 and the owner's approval.

## 1. Frozen public baseline

- Keep `gel-phase-quad::Record` unchanged: `DIM = 1024`, `Carrier = [u8; 1024]`, and `mask = [u64; 16]`, giving **1152 bytes** on the tested ABI.
- Keep the published four-view readout semantics, frozen cosine table, policies, existing tests, public API, ORB128 binary format, and existing saved formats unchanged.
- The four current coordinate views are equivalent readouts. **They are not evidence of four independent payloads or fourfold information density.**
- The public affine Q8 reference and phase-Q8 record are different representations; do not combine their byte counts.
- No private GEL source, encoding algorithm, private data, keys, or unpublished benchmark fixtures enter this branch or its CI logs.

## 2. Separate candidates, not a replacement

| Candidate | Proposed physical budget | Meaning | Status |
| --- | ---: | --- | --- |
| Existing public phase-Q8 | 1152 B in memory | 1024 phase bytes + 128 B active mask | Keep as-is |
| Experimental Q8-256 | 256 B | Fixed-size candidate; payload, framing, and any external metadata must be declared and measured | Not implemented |
| Experimental Q8-QG4-256 | 256 B | Research target: up to 4 x 256 B logical Q8 symbols under explicitly stated data restrictions / accuracy threshold | Not implemented |

For a packed array of **256 independent Q8 symbols**, the raw payload alone is 256 B. A self-describing record, mask, scale, index, dictionary, or sidecar may consume additional space; it must never be omitted from an end-to-end storage claim.

**Information bound:** a 256 B physical record has 2048 bits. It cannot losslessly and uniquely reconstruct arbitrary 1024 B (8192 bits). Any 4x claim must define its dataset, dependencies, fidelity, metadata, and what is counted as distinct recovered information. Four reversible views of the same content do not qualify.

## 3. Proposed integration boundary

1. Leave existing public code paths and binary contracts intact.
2. Prototype experimental encoding and decoding in a **private** workspace or repository first. Do not implement the unpublished method here.
3. Any later public integration requires an **opt-in, versioned interface** with explicit format IDs and safe legacy fallback. No implicit reinterpretation of existing records.
4. Keep ORB128 separate. No changes to GEL Ocean or external runtimes as part of this branch.
5. Promote only explicitly reviewed, non-sensitive interfaces and reproducible, publishable evidence; do not assume private files can be reconstructed from public commitments.

## 4. Validation gates

### G0: Baseline and reproducibility
- Record clean branch/head, compiler and host details. Run the existing unit and integration tests with `--locked --offline` when the vendored/toolchain environment permits.
- Record existing `Record` size assertion at **1152 B** and the public quad exactness tests. Do not rewrite their expected values.

### G1: Byte accounting and versioning
- Specify whether **256 B** denotes payload, in-memory record, wire record, or persisted record; report each separately.
- Measure complete encoded bytes including masks, headers, scales, index tables, dictionaries, sidecars, alignment and checkpoint overhead; attribute shared overhead per record at stated dataset size.
- Reject malformed lengths, unknown version identifiers, out-of-bounds access, truncations, and corrupt records.

### G2: Fidelity and independence
- Compare a raw Q8-256 baseline against the QG4 candidate on the **same** held-out source data.
- Include random, independent high-entropy inputs as a negative control: 4x lossless recovery is not possible in general.
- Measure per-lane reconstruction errors, worst cases, aggregate errors, exact stored-code roundtrips and application-level retrieval/decision correctness. Distinguish quantitative error from semantic accuracy.
- Verify that claimed additional logical data are **independently useful** and not repeated/transformed views of identical information.

### G3: Runtime and resource costs
- Measure encode/decode latency (p50/p95/p99), throughput, peak RSS, allocations, CPU load, and cold/warm behavior with matched workloads.
- Include preprocessing, lookup, side data and all four readouts in end-to-end numbers; do not time only a convenient kernel.
- Test deterministic restart, migration/refusal, and untrusted input handling without touching legacy production data.

### G4: Compatibility and publication
- Existing GEL RAM APIs, saved files, checksums, reference vectors and public tests remain green.
- The new candidate must be strictly additive and explicitly opt-in; failure returns an error/fallback without silent data reinterpretation.
- Publish results only with reproducible evidence and clear limitations. Do not publish private implementation, datasets or secrets.
- **NO-GO** for default adoption until all gates pass and the owner approves disclosure.

## 5. Licensing, ownership and publication integration

The separate [QuadGrid licensing and joint-release plan](QUADGRID-LICENSING-INTEGRATION.md) is a **required** part of this candidate's acceptance. It defines legal checkpoints L0–L4 alongside technical G0–G4. The existing [NCRL 1.0](../LICENSE) remains unchanged; private/proprietary code is not added to this public branch and no new commercial grant is implied. The intended release decision must examine rights provenance, independently developed versus GEL-derived modules, contributor permissions, third-party material, confidentiality, exact versioned public surface and owner sign-off **together** with the technical evidence.

## 6. Handoff checklist

- [x] Preserve public baseline; no modifications to existing Q8 code.
- [x] Document proposed 256 B experiments and the information-theoretic boundary.
- [x] Define gates for measurement, compatibility and disclosure.
- [x] Private prototype with sealed test fixtures, outside this repository (2026-11-14; not published).
- [ ] Build the 256 B candidates from section 2 and pin their test fixtures.
- [ ] Execute fresh compiler/tests/benchmarks and attach reproducible results that are approved for publication.
- [ ] Decide whether any public opt-in API is useful and safe to reveal.
- [ ] Pass legal gates L0–L4 together with technical G0–G4; complete rights inventory and disclosure review.
- [ ] Review the patch and decide on a PR; do not merge automatically.

Related public references: [Q8 contract](Q8-QUAD.md), [precision reference](PRECISION-MATRIX.md), [format guide](FORMAT.md), [public quad validation](Q8-QUAD-VALIDATION.md).
