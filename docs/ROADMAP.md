# Development priorities

## Unreleased source-readout and archive-report follow-up

The [current candidate](../RELEASE-NOTES-SOURCE-CANDIDATE.md) adds multipart
source selection, synthetic PL/EN regression fixtures, a pinned real Rust Book
excerpt and reviewed source snapshots. Archive reports use an independently
supplied manifest pin rather than requiring Git metadata. See the
[current validation scope](SOURCE-CANDIDATE-VALIDATION.md).
This is a local candidate, not a published release or a completed legal cutover.
The source examples do not encode text into Q8; a public end-to-end encoder
experiment remains a separately reviewed scope. Historical entries below
describe their original public revisions, not all files now in this candidate.

## PR #10 follow-up: verifiable public demonstrations

[Reporting corrections and demos](PUBLIC-DEMO.md) add raw-time ratio validation,
metadata checks, cross-platform test execution, a Rust numeric playground,
source/tamper E2E and a one-command reproduction report. The public illustrations
show measured R2 results, including slowdowns. Next: independent second-host
reproduction, documented Unicode boundaries and platform-specific regression tests.
The private research system stays outside this export; no new release tag is implied.

The current [illustrated guide](ILLUSTRATED-GUIDE.md) documents public behavior
without exporting the private engine. For the next package, select one bounded
goal: independent reproduction (#11), Unicode cases (#12), or Windows behavior
(#13). Ship a fix with its reproducer and full results; do not expand the release
with private media, speakers or P2P simply to add more features.

## Local candidate: current Q8 evidence and checked views

[The candidate](Q8-EVIDENCE-CANDIDATE.md) repeats V2 timings, adds a packed
canonical-single baseline and a synthetic numeric sample, and reports separate
private-real-input parity checks. It does not publish private banks or establish
semantic accuracy. Checked descriptors address accidental incompatibility, not
authentication. Publication and a new release tag remain separate decisions.
Historical priorities below are retained; this candidate supplies local evidence
for the V2/canonical-baseline tasks but not independent second-host reproduction.

This is a direction, not a delivery-date promise. GEL's intended role is an
AI knowledge bank. The public binary engine is one component, not that entire
system. Private research archives are not a list of shipped features.

## Next: independently reproduce the public core

### Now on main, not in the v0.2.1 release tag: source-bound readout

The [source catalog](SOURCE-READOUT.md) links an approved address to exact text
with independently supplied SHA-256 pins. The candidate contains synthetic
fixtures and parser tests only. It does not include the private speaker,
encoder, media adapter, Ocean or network. It is not part of a tagged release.

### Now on main, not in the v0.2.1 release tag

The experimental [Q8 shared-reader export](Q8-QUAD.md) adds a separate phase
record, four equivalent coordinate readouts and bounded worker handling.
The V2 example also recovers from reference-worker start refusal. See the
[historical performance matrix](Q8-QUAD-RESULTS.md) and the separate
[V2 correctness evidence](Q8-QUAD-VALIDATION.md).

The full V2 timing matrix, fallback counts and canonical single-view baseline
are now available in the PR #10 candidate, including slower cases. Independent
second-host reproduction remains open. Equivalent views are not independent evidence.
No private encoder, bank, inference engine or speaker is scheduled for export
by this roadmap; each future export needs its own bounded review.

### Reproduction priorities

The [binary-core hardening](BINARY-GATE-HARDENING.md) corrects caller/worker
scheduling, raw rotation bounds and sparse validation. The V4 binary benchmark
checks the entire Top-8 and timed Top-1 and reports actual worker fallback.
This is separate from the Q8 V2 benchmark and supplies no new timing campaign
or semantic-accuracy result.

- Collect complete portable and hardware-specific comparison logs on other CPUs.
- Investigate K=1 regressions without weakening exactness or hiding slower cases.
- Retain actual macOS/Windows runtime testing added in PR #10 and extend platform regressions.
- Preserve byte-exact reconstruction, deterministic ranking and bounded resources.

Acceptance: publish the hardware, commands, full result matrix and failure
cases. A speedup on one selected case is not a universal improvement.

## Then: a small end-to-end knowledge-bank evaluation

Define a redistributable corpus, explicit encoder, held-out queries, relevant
source records and fixed K before tuning. Test paraphrases, distractors and
queries whose answer is absent. Prevent exact query text or labels leaking
into the indexed representation. Return evidence references, not unsupported
answers; evaluate an abstention policy separately.

Compare FP16, a specified Q8 format and GEL only with a documented mapping of
the same knowledge and the same queries. Count encoder/context storage and
query costs. Report Recall@K, latency and total memory separately. The target
of 0.99 knowledge retrieval was not achieved in the
[historical v0.2.1 lexical evaluation](VALIDATION-v0.2.1.md)
(Recall@10 0.480 / 0.270). These are not Q8 Quad or private-system results.
Q8 equivalence tests supply no new semantic accuracy measurement.
Byte-exactness and semantic retrieval must remain separate gates.

Publishing a demonstration does not require publishing every private research
module. However, any claim advertised as independently reproducible needs
enough public data, code and instructions to reproduce that particular claim.

## Existing engineering gates retained

The original F0–F4 direction remains in scope; the nearer-term priorities
above do not mark these gates complete or replace them.

Already implemented in v0.2.0: the dependency-free F0 memory-physics harness,
F1 contingency kernel/Reader16/exact ranking, F2 structural XOR codec with
literal fallback and depth at most two, v2 CRC-protected persistence,
exact threaded Top-1 and aligned sequential/random-fetch probes.

- **F0.5 topology closure:** measure 4 KiB versus transparent/explicit huge
  pages, per-domain L3 bandwidth, RAM bandwidth above LLC, batch 8/16/32
  random ORB fetches, pinned per-domain thread scaling and dTLB/LLC counters.
- **F1.5 information separability:** measure the conditional value of each
  Reader16 judgment on a declared dataset. Coordinate isometries do not
  constitute new information.
- **F2.5 capacity:** report exact DER, residual-popcount distributions and
  context-touch cost on real ORBs. No fixed compression-factor claim without
  its denominator and all context costs.
- **F3 locator and sketch:** establish the F0/F2 physical budget before a
  scale-selective path; perfect-hash locators still need membership checks.
- **F4 end-to-end scale:** evaluate ten million ORBs before one billion.
  Latency, capacity and exactness form a combined gate, not independent
  marketing claims.

## Not claimed or scheduled

No arbitrary FP16-to-128-byte lossless compression, replacement for a complete
LLM, zero-hallucination guarantee, production durability certification, or
universal hardware speedup is promised. Proposed work must show measurable
value and retain correctness before it becomes a release headline.
