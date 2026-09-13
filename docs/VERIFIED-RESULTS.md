# GEL RAM — start with the evidence

**Four equivalent Q8 views, one stored record. Exactness checks beside the timings.**

GEL's public Rust core explores memory for an AI knowledge bank. These are
bounded engineering results, not proof of a complete AI system or semantic
understanding. No private encoder, dataset or model is needed for the public
synthetic demonstrations below. The public license is
[PolyForm Noncommercial 1.0.0](../LICENSE), not an unrestricted commercial license.

## What has actually been checked?

| Capability | Recorded result | Evidence and boundary |
| :--- | :--- | :--- |
| Shared Q8 Quad readout | 8,355,840 / 8,355,840 view-score comparisons passed in 48 V1 invocations | [Full matrix](Q8-QUAD-RESULTS.md), [raw logs](evidence-q8-quad/README.md). Correlated synthetic comparisons, not independent knowledge questions. |
| Q8 record material | 1152 bytes for one canonical record, versus 4608 bytes for four packed copies | [Contract](Q8-QUAD.md). Includes activity mask; excludes runtime tables and buffers. Four views of the same content, not four distinct memories. |
| Historical Q8 scan timing | Configuration medians span 1.21–5.31× FourViews/Shared | [All 16 configurations, ranges and timings](Q8-QUAD-RESULTS.md). Complete implementation comparison, not universal speedup; Shared is not uniformly faster than ReferenceOne. |
| Exact binary Top-K | Published comparison checks indices, scores and tie ordering | [Protocol and reproduction](TOPK-BENCHMARK.md), [recorded results](VALIDATION-v0.2.1.md). K=1 includes regressions; exact ranking is not semantic accuracy. |
| Binary reconstruction | Tests compare restored ORB bytes to their input | [Integrity audit](DATA-INTEGRITY.md). This does not establish lossless conversion of arbitrary source knowledge into a 128-byte ORB. |
| Q8 V2 worker refusal | A real Linux worker-start refusal completed with exact comparisons passing after the fix | [Before/after evidence and 16-run regression matrix](Q8-QUAD-VALIDATION.md). Not a replacement timing campaign or a guarantee of all failure recovery. |

The Q8 timing campaign ran on **AMD Ryzen AI 9 HX 370, 12 cores / 24 logical
CPUs, about 93.91 GiB RAM, Rust 1.85.0 release**, on 2026-09-08. It used 1 and
24 workers on a shared host without exclusive isolation. These were CPU-only
scan timings with prepared data, not end-to-end question answering. The
historical binary Top-K campaign has a different protocol, documented separately.

The public Q8 preview is on main, not in the existing v0.2.1 release tag.
The current Q8 V2 runner differs from the preserved V1 timing runner.

## Try it on your machine

In a fresh checkout, install the pinned Rust toolchain and record the commit:

```text
git clone https://github.com/gelramlicensing-wq/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
git rev-parse HEAD
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Cloning, toolchain installation and fetching any locked dependencies need
network access. After setup, the commands below run locally without a model
or external data API. Run separately, not concurrently when comparing timings:

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_compare -- --orbs 8192 --rounds 9 --workers 1 --sparse 1 --policy active
cargo run --locked --offline --release -p gel-reader --example topk_compare -- --orbs 8192
```

Expected checks: `GEL_VERIFY_ALL=PASS`, `Q8_QUAD_EXACT=PASS` and
`TOPK_COMPARE_EXACT=PASS`. The Q8 demo also reports
`SEMANTIC_ACCURACY=NOT_MEASURED`. Investigate failed checks before interpreting
timings. One invocation is a reproduction probe, not a full performance campaign.

## Help test the claim, not the headline

We welcome reproductions on different CPUs, including slower cases and failures.
[Open a reproduction report](https://github.com/gelramlicensing-wq/gel-ram/issues/new?template=reproduction.yml)
with the commit, full command and output, CPU/OS, requested and effective worker
counts, and other host load. Remove personal paths, usernames and secrets before
uploading logs; no private knowledge bank or personal conversations are needed.

The next valuable evidence is a complete V2 timing matrix and a fair canonical
single-view baseline, followed by independent hardware reproductions. See the
[roadmap](ROADMAP.md). Neither has been marked complete by this presentation.

## Also on main: source-bound readout

[PR #8](https://github.com/gelramlicensing-wq/gel-ram/pull/8) added a bounded
source reader with 12 regression tests covering corrupted or inconsistent input,
stale source generations and the 50,000-record boundary. It was merged on
2026-09-11 and is **an unreleased preview on main**, not part of the v0.2.1 tag.
Its [scoped review](https://github.com/gelramlicensing-wq/gel-ram/blob/384679bc383793e228cbf02045ed656b5b796fed/docs/SOURCE-REVIEW.md)
documents the audit and its limits. Consult the PR for current status.

CI reports Linux verification and macOS/Windows compilation separately.
Compilation is not runtime testing on those operating systems. Integrity
checks establish correspondence to trusted source bytes, not their truth.

Private multimedia, native inference and conversational research are not
included here. This page adds no implementation details beyond existing public
documentation and makes no new performance or semantic-accuracy claim.
