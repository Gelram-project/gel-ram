# GEL RAM public roadmap

## Snapshot and publication boundary

Status checked on 2026-09-26. This is a public execution plan, not the private
research roadmap and not a release approval.

| Scope | Recorded state | Source |
|---|---|---|
| Historical release | v0.3.0 remains the earlier release point | [Release](https://github.com/Gelram-project/gel-ram/releases/tag/v0.3.0) |
| Public main baseline | Evidence Lab 0.4.0-rc.1 merged through PR #9 at `71142a25e7ad75e4d75acf4e244d8e4a996c4a92` | [PR #9](https://github.com/Gelram-project/gel-ram/pull/9) |
| Follow-up implementation | PR #10 is open at the time of this snapshot; code reviewed for this inventory: `6f8d3c1e27707e854217a7eb2e448ee66cd2b2f0` | [PR #10](https://github.com/Gelram-project/gel-ram/pull/10) |
| Final v0.4.0 release | Not authorized by the recorded integration approval | [Publication status](../CANDIDATE-STATUS.md) |

A PR test merge is not a merge into main. Later commits need their own checks.
Check the linked PR and Actions before relying on this dated inventory. Moving
an item into this document cannot change its implementation or approval state.

## What is already on main

The PR #9 baseline includes the Rust-only workspace, locked toolchain and
dependencies, exact source-bound quotations, multipart readout, Unicode and
source-boundary checks, explicit UNKNOWN/INCOMPLETE states, stale-result
rejection, plaintext no-replace snapshots and fresh-process reopen. It includes
both single-document Live Lab and multi-document Evidence Lab.

The public numeric scope includes Q8 reversible coordinate views, independent
byte/numeric/ranking checks and the separate Ocean Scale R3 source/evidence
archive. The [Ocean guide](OCEAN-SCALE.md) separates Linux-only research
persistence/mapping from portable workspace checks. [Platform records](PLATFORM-REVIEW.md)
identify the tested revisions; platform exclusions are not passes.

## How to read the work list

Implementation state and acceptance state are separate. `IN_REVIEW` means a
change exists in PR #10, not that it is merged. `PARTIAL` means a stated part is
implemented or documented while an acceptance condition remains open. `OPEN`
means this public inventory does not establish the implementation or evidence.
`CURRENT` applies only to repository settings already visible independently of
the branch. No completion percentage is calculated from unequal tasks.

The identifiers A01 to A24 retain the audit work-item order. Evidence links are
contracts and entry points, not blanket PASS labels. For each accepted item,
retain the implementation commit, fixture or corpus hash, exact command,
expected result and counterexample, actual result, platform/profile, run attempt,
reviewer and remaining exclusions. Responsibility belongs to the PR implementer;
final scope and acceptance decisions belong to the maintainer. A second-host
reproduction must identify its independent operator rather than invent one.

## Audit work items and next acceptance

| ID | State at reviewed code snapshot | Work and next acceptance condition | Evidence / entry point |
|---|---|---|---|
| A01 | IN_REVIEW | Historical source/lockfile mapping exists. Keep old hashes and pass both positive and changed-source checks. | [Measured-source guide](evidence-collection/README.md) |
| A02 | IN_REVIEW | CI rechecks the retained R1 dataset. Require rejection of missing, duplicated or inconsistent evidence. This is not a new timing campaign. | [R1 negative checks](R1-NEGATIVE-CHECKS.md) |
| A03 | IN_REVIEW | Bounded quote context and omission flags exist. Retain negation, condition, unit and line-ending regressions; do not claim unrestricted context understanding. | [Quote context](QUOTE-CONTEXT.md) |
| A04 | IN_REVIEW | Rust runtime-example sequencing checks each process. Verify first/middle failures on native CI, including misleading PASS output. | [Process tests](../xtask/tests/process_sequence.rs) |
| A05 | PARTIAL | Decode report exists; complete visual/privacy review, small-screen readability and timeline-to-log reconciliation remain open for all three films. | [Media review limits](MEDIA-DECODE-REVIEW.md) |
| A06 | IN_REVIEW | Bounded recorder streams and no-replace logs exist. Retain split-UTF8, EOF, failed-write, missing-marker and child-exit checks. | [Recorder safety](RECORDER-SAFETY.md) |
| A07 | PARTIAL | Reports separate named test executions, profiles and exclusions. Complete property-to-test mapping and declared release-profile coverage; zero doctests is not added coverage. | [CI evidence](CI-EVIDENCE.md) |
| A08 | IN_REVIEW | Structural/admission and publication-fault tests exist. Preserve active state on rejection and document possible publication before a directory-sync error. | [Publication fault tests](PUBLICATION-FAULT-TESTS.md) |
| A09 | PARTIAL | Streaming roots and new-collection builder exist. Retain byte/pin compatibility and publish fresh time/allocation/peak-memory comparisons before claiming a measured gain. | [Mutation comparison](MUTATION-COMPARISON.md), [builder](COLLECTION-BUILDER.md) |
| A10 | PARTIAL | Comparison protocols exist. Apply explicit input, profile, clock boundary, warmup, order and host metadata across campaigns. | [Campaign protocol](EVIDENCE-CAMPAIGN.md) |
| A11 | PARTIAL | A public F32/F16/affine Q1 to Q16 reference and versioned container now exist. Review its exact-revision tests and retained output; this does not establish integration with every GEL reader/writer or private Q2.5. | [Precision matrix](PRECISION-MATRIX.md), [codec scope](CODEC-SCOPE.md) |
| A12 | IN_REVIEW | Bytes, numeric error, ranking and context are separated. Keep distinct acceptance results rather than deriving all four from transport PASS. | [Claims](CLAIMS.md), [codec scope](CODEC-SCOPE.md) |
| A13 | PARTIAL | Revision-bound CI report generator exists. Retain a reviewed, retrievable evidence package; a generated temporary report is not permanent publication. | [CI evidence](CI-EVIDENCE.md) |
| A14 | PARTIAL | README entry point is improved. Check fresh-checkout instructions and first-screen rendering; keep main, review branch and historical tag distinct. | [README](../README.md) |
| A15 | CURRENT / PARTIAL | Description and topics are repository settings, not code. Recheck their accuracy; social-preview artwork and any extra landing page require separate visual acceptance. | [Repository](https://github.com/Gelram-project/gel-ram) |
| A16 | IN_REVIEW | One film index separates the public walkthrough from private previews. Verify destinations and scope labels. | [Film index](../media/INDEX.md) |
| A17 | PARTIAL | A two-process update/restart scenario and tests exist. A new reviewed recording of that scenario is not yet established. | [Update/restart scenario](UPDATE-RESTART-SCENARIO.md) |
| A18 | PARTIAL | PL/EN descriptive transcripts exist. Full timeline accuracy and accessible small-screen presentation remain open; translations are not new engine outputs. | [Transcripts](../media/TRANSCRIPTS-PL-EN.md) |
| A19 | PARTIAL | An executable claim registry exists. Expand coverage and align current summaries without rewriting historical evidence; deferred rows must stay deferred. | [Claims](CLAIMS.md) |
| A20 | OPEN | Reproduction form exists. Obtain and retain an independent second-host campaign and first-use feedback. Hosted CI alone does not close this item. | [Reproduction form](../.github/ISSUE_TEMPLATE/reproduction.yml) |
| A21 | PARTIAL | Scoped author-reported component measurements are available in the review branch. An identical-task, end-to-end comparison with the exact baseline remains open. | [Component scope](GEL-EXPERIMENTAL-MEASUREMENTS.md) |
| A22 | OPEN | Implement a public execution-identity gate only within approved disclosure scope. Substituting the full-scan baseline must fail that gate even when results match. | [Baseline boundaries](OCEAN-SCALE.md), [claims](CLAIMS.md) |
| A23 | OPEN | Adaptive precision needs a declared error budget, independent oracle, full metadata costs and stable task decisions, including weak-signal failures. | [Codec scope](CODEC-SCOPE.md) |
| A24 | PARTIAL | Preserve small, reviewable commits on the existing review branch. Keep PR metadata, manifest and exact-revision checks aligned; no implicit merge, release or private disclosure. | [PR #10](https://github.com/Gelram-project/gel-ram/pull/10) |

## Execution order and dependencies

### M0. Coherent information and review scope

Update this roadmap, README, publication-status scope and PR description together
with the source manifest. Dependencies: the checked baseline and PR inventory.
Acceptance: readers can distinguish merged features, branch-only changes,
historical evidence and open gates; PR metadata matches the actual diff. No
runtime, license, dependency, film or historical measurement changes belong to
this documentation patch. This is a maintained snapshot, not an implemented
automatic status synchronizer.

### M1. Integrity and failure-path acceptance

After M0, review A01-A04, A06-A08, A12-A13 and A19 at one exact revision. Run
configured native checks and retain both success and expected-rejection evidence.
Separate finite fixture coverage, ignored/platform-excluded tests, repeated test
executions, doctests and debug/release profiles. Scope any unresolved limitation
explicitly before main integration. Do not mark a gate PASS from prose alone.

### M2. Reproducible user experience

After the relevant M1 checks, close the visual parts of A05 and A14-A18. Review
all three films from start to finish, check original bytes and identifiers,
record reviewed time ranges and reconcile captions with actual logs. Keep
historical films unchanged; publish a new recording only after reviewing the
new A17 scenario. A decode-only report or sampled screenshots cannot close M2.

### M3. Measured scaling

After correctness acceptance, address A09-A10 and A20. Run paired baseline/new
measurements on the same task and inputs, then on independent hardware. Retain
all raw samples, slower runs and failures. Report payload, metadata, mapped
bytes, RSS/peak RSS, faults, allocations/copies where measured, worker count,
profile, host load and timing boundaries. Choose sample size and uncertainty
reporting before a tail-latency claim; do not infer deep tails from a small N.

### M4. Distinctive memory research

A21-A23 depend on an exact task contract, the M1 failure gates and approved
public disclosure. Keep the existing full scan as the comparison oracle, never
as evidence that a different execution path was used. Require differential and
execution-identity checks together. Use the public [precision reference](PRECISION-MATRIX.md) as a scoped
starting point, not as proof of private codec or application integration.
Evaluate adaptive selection against the declared task error budget.
A public retrieval-quality claim additionally needs a redistributable corpus,
a documented public encoder, held-out questions and an abstention policy.

Author-reported [component measurements](GEL-EXPERIMENTAL-MEASUREMENTS.md) have
different inputs and timing boundaries from [Ocean full scans](OCEAN-SCALE.md).
They do not establish same-task end-to-end superiority. These public gates do
not publish private engines, banks, speaker or private Ocean/P2P internals.

### M5. Separately approved release

A bounded release does not require pretending that all longer-term research is
complete. Freeze its declared scope, carry open items and exclusions into release
notes, verify the exact tree/assets/manifest and configured CI, and obtain separate
maintainer release approval. M0 and the M1 checks for that scope are required;
any included media must have its stated M2 review. M3/M4 are prerequisites only
for performance or capability claims included in the release. Do not move the
historical v0.3.0 tag or infer approval from a green job.

## Evidence and privacy policy

Preserve historical source pins, measurements, films, slow cases and negative
results. A new source tree is not a new measurement. Hashes identify bytes;
they do not authenticate an untrusted publisher or establish factual truth.
Process-kill and injected I/O tests do not certify physical power-loss survival.
Four coordinate views do not establish fourfold independent information capacity.
No unrestricted semantic QA, universal speedup, physical memory-side compute or
DRAM-refresh coupling is established by this work list.

For independent work, start with [TRY-IT](TRY-IT.md), [Ocean Scale](OCEAN-SCALE.md)
and [verified-results scope](VERIFIED-RESULTS.md). Retain revision, hardware,
commands, complete results and exclusions. A benchmark report does not require
a code contribution; intended code contributions follow [CONTRIBUTING](../CONTRIBUTING.md).
Private mechanisms and personal data stay outside this public plan. Security
findings follow [SECURITY](../SECURITY.md), not a public disclosure by default.
