# GEL RAM — verified public evidence

This page is the compact evidence map for the current public repository.
It separates **measured public results**, **public engineering checks** and
**private/unpublished work**.

GEL RAM-owned material is distributed under the
[GEL RAM Noncommercial Reciprocal License 1.0](../LICENSE). Third-party material
retains its own license terms.

## Current public capabilities

| Capability | Public evidence | Boundary |
|---|---|---|
| Source-bound exact readout | Exact UTF-8 quotations, byte ranges, source/catalog SHA-256 pins, corruption and stale-generation rejection | Integrity of approved bytes is not proof that the source statement is true |
| GELSRC01 persistence | No-replace publication, full-file pin, fresh-process reopen, truncation/corruption rejection and competing-writer tests | Plaintext source bundle, not an encrypted vault |
| GEL Live Lab | Offline Rust terminal integration for import, phrase lookup, readout, save/reopen and a separate synthetic Q8 panel | Not the private application and not a semantic chatbot |
| Q8 coordinate views | Four reversible views of one 1152-byte public Q8 record with inverse/exactness checks | Four views are not four independent facts or 4× storage capacity |
| Independent integrity audit | Byte/numeric/ranking audit reports exact recovery for the encoded representations it tests | Conversion loss and semantic quality are separate questions |
| Ocean Scale R3 | Public 1M/10M synthetic numerical source/evidence archive with raw measurements and independent verifier | CPU/RAM full-scan baseline, not the private execution path |
| Cross-platform public core | Linux, macOS and Windows required checks execute workspace/runtime verification | Linux-only Ocean persistence/mapping research is not claimed portable |

## Published Ocean full-scan baseline

EXACT full scan → top10 → decision, 24 CPU workers:

| ORB count | seed | N | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|
| 1M | 41119 | 100 | 72.427 ms | 81.018 ms | 82.859 ms | 90.949 ms |
| 1M | 61141 | 100 | 71.962 ms | 79.860 ms | 80.576 ms | 88.227 ms |
| 10M | 41119 | 100 | 615.428 ms | 684.888 ms | 710.313 ms | 734.085 ms |
| 10M | 61141 | 100 | 704.995 ms | 757.900 ms | 783.038 ms | 810.236 ms |

These are intentionally published as a **conventional CPU/RAM baseline**.
They do not represent the intended final GEL RAM execution model. The public
Ocean guide also records a shorter 10M/24-worker campaign with p50 498.265 ms,
and explicitly does not substitute that faster shorter run for the longer runs.

With only 100 observations per long run, the project does **not** claim stable
p99.9, p99.99 or p99.999 tails from this dataset.

Full provenance and limitations:
[Ocean Scale](OCEAN-SCALE.md).

## Q8 evidence

The public Q8 record contains 1024 phase bytes plus a 128-byte activity mask:
**1152 bytes per record**, excluding runtime tables and buffers.

Historical R2 evidence recorded:

- 8,355,840 V2 view-score comparisons passing;
- 2,509,056 canonical-baseline comparisons passing;
- complete raw timing/evidence retained in the repository;
- no claim that the coordinate transforms create independent information.

See [Q8 R2 audit](Q8-CANDIDATE-R2-AUDIT.md) and
[Q8 contract](Q8-QUAD.md).

## Source and Live Lab evidence

The current public integration includes regression coverage for:

- exact source pins and byte ranges;
- composed/decomposed Unicode and Greek sigma handling;
- CR, LF and CRLF boundaries;
- overlong lines reported as incomplete rather than false absence;
- corruption and truncation;
- no-replace persistence;
- competing writers;
- fresh-process reopen;
- terminal-control escaping.

See [Document Readout](DOCUMENT-READOUT.md),
[Source Builder](SOURCE-BUILDER.md) and [Live Lab](LIVE-LAB.md).

## What is intentionally not a public claim

The public repository does not claim that it proves:

- private-system architecture or performance;
- unrestricted semantic AI accuracy;
- 30M queries/s or any conversion from internal update-rate measurements to queries/s;
- stable extreme-tail latency without sufficient observations;
- physical memory-side compute or physical DRAM-refresh synchronization;
- production power-loss durability;
- fourfold independent capacity from four reversible views.

The project owner reports materially better private experimental measurements
than the public full-scan baseline. The mechanism and exact performance claims
remain intentionally unpublished until the owner chooses to provide a
reproducible public evidence package.

## Reproduce instead of trusting the summary

Start with [TRY-IT](TRY-IT.md). For the larger research snapshot, use
[Ocean Scale](OCEAN-SCALE.md). When reporting results, include the exact commit,
commands, hardware, Rust version, complete output and failures.

Open an independent reproduction report:
https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml

A slower result is useful evidence too.
