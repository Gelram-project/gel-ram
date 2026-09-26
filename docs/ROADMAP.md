# GEL RAM public roadmap

This roadmap describes only work that is safe to discuss in the public repository.
The complete private research direction, unpublished execution mechanisms and
private performance work are intentionally outside this document.

## Version boundary
- Tagged `v0.3.0` remains an immutable earlier release.
- Ocean Scale R3 is on `main`: numerical 1M/10M evidence, bounded offline
  verification and Linux-only research persistence/mapping. See [scope](OCEAN-SCALE.md).
- Document readout is on `main` through [PR #4](https://github.com/Gelram-project/gel-ram/pull/4):
  exact source quotations, Unicode/CR regressions and runtime CI on three OSes.
- [Live Lab](LIVE-LAB.md) and [source building](SOURCE-BUILDER.md) were published
  in PR #5. [Evidence Lab](EVIDENCE-LAB.md) was merged into main through
  PR #9 (71142a2, workspace 0.4.0-rc.1). The PR #10 review repairs follow it;
  no v0.4.0 tag or release exists ([status](../CANDIDATE-STATUS.md)).
## Public main — already delivered

The following capabilities are present on public `main`:

- Rust-only public workspace with locked toolchain/dependency verification.
- Exact source-bound quotations with SHA-256 provenance and stale/corrupt input rejection.
- Multipart source readout with explicit gaps, ambiguity and UNKNOWN handling.
- GELSRC01 plaintext source bundles with no-replace publication, caller-retained
  full-file SHA-256 pins and fresh-process reopen.
- Offline GEL Live Lab for caller-selected UTF-8 documents.
- Synthetic Q8 records with four reversible coordinate views and exact inverse checks.
- Independent byte/numeric/ranking audit.
- Ocean Scale R3 with reproducible 1M/10M synthetic numerical evidence,
  persistence/replay research and raw measurements.
- Linux verification plus Windows/macOS workspace/runtime checks.
- Evidence Lab (0.4.0-rc.1, PR #9): multi-document collection, exact citations,
  stale-result rejection, GELSET01 snapshots and fresh-process reopen; native
  Linux/Windows/macOS CI at the PR #9 head.
- Active branch rules requiring pull requests, current required checks and squash merges.

## Open review items
1. Independent, non-author review of the multi-document Evidence Lab: source
   boundaries, stale citation invalidation, snapshots and independently
   recomputed raw measurements. The developer-authored assessment is not blind
   validation. Single-document import/read/save/reopen belongs to the earlier published base.
2. Keep per-platform CI evidence for every revision. Unix permission and
   directory-sync checks remain platform-specific, not portable guarantees.
3. Provide one-command local reporting with raw output, resource/timing scope and
   failures retained. Never infer robust performance from a single UI timing.
4. Reproduce numeric Q8/Ocean evidence on an independent second host, including
   regressions and complete memory costs. Keep addressed readout and full scan
   distinct; coordinate views are not independent information.
5. Evaluate knowledge retrieval only after defining a redistributable corpus,
   an explicit public encoder, held-out questions and abstention policy. Phrase
   matching, source integrity and synthetic numerical ranking are not semantic QA.
## Published baseline and longer-term direction
The tagged `v0.3.0` release remains an earlier immutable release point. Public
`main` contains additional reviewed work that has not yet been cut as a new tag.

## Public performance boundary

The published 1M/10M Ocean measurements are deliberately retained as a
**CPU/RAM full-scan baseline**: CPU workers scan, score, rank and select records
resident in memory. They are valuable comparison evidence, but they are not
presented as the final GEL RAM execution model.

The private project contains newer experimental work. Author-reported component
measurements are published separately in
[GEL-EXPERIMENTAL-MEASUREMENTS.md](GEL-EXPERIMENTAL-MEASUREMENTS.md); they
measure different tasks and establish no end-to-end advantage or multiplier over
this baseline. Any such claim requires an exact public reproduction package.

## Next public acceptance gates

1. **Independent reproduction on a second host.** Re-run the public Ocean/Q8 and
   source-integrity evidence on different hardware. Publish slow cases and
   failures, not only best runs.
2. **Larger statistical campaigns.** Where performance claims need tail latency,
   retain raw samples and use a sufficient N before reporting p99.9 or deeper tails.
3. **Memory-accounting evidence.** Report payload, mapped bytes, RSS/peak RSS,
   page faults, worker count and host conditions alongside latency.
4. **Exactness before speed.** Any faster public path must be differential-tested
   against the existing exact reference and must not weaken integrity,
   abstention, ordering or restart/replay checks.
5. **Fault and concurrency coverage.** Extend corruption, truncation, competing
   writer, restart, timing and observer-effect tests.
6. **Public retrieval evaluation only with a suitable corpus.** Semantic or
   knowledge-quality claims require a redistributable corpus, a documented
   public encoder, held-out evaluation and an explicit abstention policy.
7. **Next release package.** Cut a new tagged release only after its exact source
   tree, release assets, checksums, documentation and required CI are all green.

## Reproduction priorities for outside researchers

The most useful independent work is currently:

- Ocean Scale R3 on a second Linux host;
- public workspace verification on additional CPU families;
- corruption/restart/concurrency edge cases;
- Unicode/source-boundary cases for the document reader;
- complete memory-cost measurements;
- slower or negative performance results that help identify the real limits.

Start with [TRY-IT](TRY-IT.md), [Ocean Scale](OCEAN-SCALE.md) and
[VERIFIED RESULTS](VERIFIED-RESULTS.md). A benchmark report does not require a
code contribution or a CLA; code intended for merge follows [CONTRIBUTING](../CONTRIBUTING.md).

## Research boundaries retained

The public project does not currently establish:

- unrestricted semantic question answering;
- universal speedup on all hardware or workloads;
- stable p99.9–p99.999 from campaigns with too few observations;
- physical memory-side compute;
- physical DRAM-refresh coupling;
- production power-loss durability;
- fourfold independent information capacity from four coordinate views;
- private-system performance or architecture.

These boundaries are deliberate. Public evidence should remain reproducible
without exposing unpublished private mechanisms.

## Publication policy

Prefer a bounded, independently runnable capability over a larger unverifiable
claim. Keep source inventory, exact revision, hardware context, raw evidence,
licensing and known limitations together. A green technical gate proves only
the scope it actually tests.
