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
| GELSRC01 persistence | No-replace publication, full-file pin, fresh-process reopen, truncation/corruption rejection and competing-writer tests | Plaintext source bundle, not encrypted storage |
| GEL Live Lab | Offline Rust terminal integration for import, phrase lookup, readout, save/reopen and a separate synthetic Q8 panel | Not the private application and not a semantic chatbot |
| Q8 coordinate views | Four reversible views of one 1152-byte public Q8 record with inverse/exactness checks | Four views are not four independent facts or 4× storage capacity |
| Independent integrity audit | Byte/numeric/ranking audit reports exact recovery for the encoded representations it tests | Conversion loss and semantic quality are separate questions |
| Ocean Scale R3 (historical) | Recorded 1M/10M synthetic full-scan measurements; the research archive was withdrawn from the tree in 0.4.0, so they cannot be re-run from it | CPU/RAM full-scan baseline, not the private execution path |
| Cross-platform public core | Linux, macOS and Windows required checks execute workspace/runtime verification | Linux-only persistence tests are declared per platform |
| Evidence Lab collections (main since PR #9) | Multi-document add/replace/drop, exact citations, stale-result rejection, GELSET01 no-replace snapshots and fresh-process reopen ([guide](EVIDENCE-LAB.md)) | Phrase retrieval over plaintext sources, not a semantic chatbot or encrypted storage |
| Precision matrix (PR #10) | F32/F16/affine Q1–Q16 reference precisions in a GPMX v1 example container, with independent checks ([matrix](PRECISION-MATRIX.md)) | Not the private codec; timing not measured |
| Publication faults (PR #10) | Injected I/O failures, kernel permission denial (Unix test) and a physically full 1 MiB tmpfs in Linux CI ([fault matrix](PUBLICATION-FAULT-TESTS.md)) | Not a power-cut test |
| Workflow and benchmark hardening (PR #19) | Fail-closed workflow permission and action-pin checks, full benchmark answer inventories and 40,000 bounded format mutants ([hardening audit](AUDIT-HARDENING.md)) | Not independent second-host acceptance |
| Answer-or-abstain re-scoring (PR #31) | `xtask answer-bench` re-scores the recorded answers of GEL and three language models on the 160 questions of v1 and the 394 of v2, and of GEL and two BM25 search engines on the 979 of v3, the 985 of v4, the 987 of v5 and the 990 of v6, and the 982 with an answer and 599 without of v7, under strict and published rules; it checks the published tables, the v3 to v7 precision intervals and paired tests, and the passage hashes in every verify run ([v1](answer-or-abstain/README.md), [v2](answer-or-abstain-v2/README.md), [v3](answer-or-abstain-v3/README.md), [v4](answer-or-abstain-v4/README.md), [v5](answer-or-abstain-v5/README.md), [v6](answer-or-abstain-v6/README.md), [v7](answer-or-abstain-v7/README.md)) | Re-scores recorded answers; it does not re-run GEL, the models or the engines |
| Crash series (PR #36) | `xtask crash-series` kills the public `gel-evidence` at random moments while a collection grows; acknowledged snapshots, partial snapshots and resume are checked ([crash series](CRASH-SERIES.md)) | A process kill, not a power cut |
| History kill series (PR #67) | `xtask history-crash-series` kills a writer that saves one growing record history with `write_atomic`; acknowledged states, partial files and resume are checked, beside an in-place control writer that must be caught leaving a partial file ([crash series](CRASH-SERIES.md#the-record-history-one-file-replaced-while-it-grows)) | A process kill, not a power cut |

## Author-run measurements of the private implementation

These results come from the separate private implementation and its bank. They
cannot be re-run from this checkout; the [claim registry](CLAIMS.md) lists them
as `MEASURED_LOCAL`, and the linked pages give the protocol and evidence
identities.

| Result | Recorded numbers | Boundary | Details |
|---|---|---|---|
| Answer verdict on stored passages | 50,000 probes: 46,376 answered (92.8%), 46,353 correct (99.95% of answers), 23 wrong (0.046% of probes) | Stored passages, not questions; the answer bank shrank to 671,416 passages | [Measured progress](MEASURED-PROGRESS.md) |
| Natural questions (question set v1) | 80 frozen questions: right article first for 40; 11 answers, all correct; 69 UNKNOWN | Too small a set for a precision rate | [Measured progress](MEASURED-PROGRESS.md) |
| Larger frozen question set v2 | 394 questions: GEL 63 answers, 59 correct and 4 wrong; 331 UNKNOWN | One run; questions written by the project's AI coding assistant | [Set v2](answer-or-abstain-v2/README.md) |
| Question set v3 beside BM25 search engines | 979 questions, one bank for all: GEL 191 answers, 180 correct and 11 wrong; Tantivy and SQLite FTS5 with thresholds 238 and 241 correct, 28 wrong each. Question by question the engines find more answers (p < 0.001) and GEL gives fewer wrong ones (p = 0.006) | For the final GEL build a re-test after a fix, not a held-out set; engines in default settings; questions written by the project's AI coding assistant | [Set v3](answer-or-abstain-v3/README.md) |
| Question set v4, held out for the tested build | 985 new questions, frozen before any run: GEL 423 answers, 405 correct and 18 wrong (95.7%); the v3 build 172 correct, 17 wrong; Tantivy and SQLite FTS5 at the same rule 531 and 558 correct, 40 and 41 wrong. At precision >= 0.99 the engines are ahead | A development build, not a release; engines in default settings; questions written by the project's AI coding assistant | [Set v4](answer-or-abstain-v4/README.md) |
| Question set v5, held out for every system it measures | 987 new questions, frozen before any run: the v4 build 432 answers, 416 correct and 16 wrong (96.3%); at its precise setting 297 correct and 1 wrong; Tantivy and SQLite FTS5 at the same rule 567 and 597 correct, 43 and 47 wrong. A candidate change gave 92.3%, below the 0.95 fixed in advance, and was rolled back | A development build, not a release; 99% precision is not established across v4 and v5; engines in default settings; questions written by the project's AI coding assistant | [Set v5](answer-or-abstain-v5/README.md) |
| Question set v6, no setting chosen on it | 990 new questions, frozen before any run: the v4 build 465 answers, 447 correct and 18 wrong (96.1%); at its precise setting 313 correct and 2 wrong; Tantivy and SQLite FTS5 at the same rule 588 and 616 correct, 33 and 36 wrong | A development build, not a release; the set was used internally before (an experiment and an analysis of its errors) without changing any build or setting, so GEL's results on it were known before the recorded runs; 99% precision is not established across v4 to v6; engines in default settings; questions written by the project's AI coding assistant | [Set v6](answer-or-abstain-v6/README.md) |
| Question set v7, with questions whose answer is not in the bank | 982 new questions with an answer and 599 without one (399 about real topics outside the bank, 200 about invented subjects), frozen before any run and not used before: the v4 build 425 answers, 413 correct and 12 wrong (97.2%), and 26 answers to the 599 questions without one; at its precise setting 291 correct, 3 wrong and 3 answers without one; Tantivy and SQLite FTS5 at their strict threshold 380 and 384 correct with 34 and 38 wrong in all; a candidate change 245 correct and 11 wrong in all, rolled back | A development build, not a release; topics outside the bank are checked by their titles, and one of them is in the bank under another word form; 99% precision is not established across v4 to v7; engines in default settings; questions written by the project's AI coding assistant | [Set v7](answer-or-abstain-v7/README.md) |
| Kill during learning, private store | 207 trials: 0 of 235,712 confirmed records lost; every reopen check passed; every resume equal to a run without a kill | A process kill, not a power cut; a small store | [Measured progress](MEASURED-PROGRESS.md#kill-during-learning--the-private-knowledge-store) |

What has nothing to compare with: [external checks](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with).

## Recorded Ocean full-scan baseline (historical)

EXACT full scan → top10 → decision, 24 CPU workers:

| ORB count | seed | N | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|
| 1M | 41119 | 100 | 72.427 ms | 81.018 ms | 82.859 ms | 90.949 ms |
| 1M | 61141 | 100 | 71.962 ms | 79.860 ms | 80.576 ms | 88.227 ms |
| 10M | 41119 | 100 | 615.428 ms | 684.888 ms | 710.313 ms | 734.085 ms |
| 10M | 61141 | 100 | 704.995 ms | 757.900 ms | 783.038 ms | 810.236 ms |

These are intentionally published as a **conventional CPU/RAM baseline**.
They do not represent the intended final GEL RAM execution model. The public
Ocean record also lists a shorter 10M/24-worker campaign with p50 498.265 ms,
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
- physical memory-side compute or other hardware-level memory computation;
- production power-loss durability;
- fourfold independent capacity from four reversible views.

Separate [author-reported component measurements](GEL-EXPERIMENTAL-MEASUREMENTS.md)
cover different tasks (compact-sketch search, a second private component); they do not
establish an end-to-end advantage over this baseline. No private-path speedup
is claimed; the private mechanism remains unpublished until a reproducible
public evidence package exists.

## Reproduce instead of trusting the summary

Start with [TRY-IT](TRY-IT.md). The recorded Ocean results are in
[Ocean Scale](OCEAN-SCALE.md). When reporting results, include the exact commit,
commands, hardware, Rust version, complete output and failures.

Open an independent reproduction report:
https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml

A slower result is useful evidence too.
