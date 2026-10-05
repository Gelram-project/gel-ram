# GEL RAM public roadmap

This roadmap describes only work that is safe to discuss in the public repository.
The complete private research direction, unpublished execution mechanisms and
private performance work are intentionally outside this document.

## Version boundary
- The 0.5.3 version line contains everything below; tagged releases are listed on
  the Releases page. Tagged `v0.5.3` is the current release; the tags `v0.5.2`, `v0.5.1`,
  `v0.5.0`, `v0.4.0` and `v0.3.0` remain without a release page.
- Ocean Scale R3 1M/10M full-scan results remain as historical measurements;
  its research archive is not part of the tree from 0.4.0 onward. See [results](OCEAN-SCALE.md).
- Document readout is on `main` through [PR #4](https://github.com/Gelram-project/gel-ram/pull/4):
  exact source quotations, Unicode/CR regressions and runtime CI on three OSes.
- [Live Lab](LIVE-LAB.md) and [source building](SOURCE-BUILDER.md) were published
  in PR #5. [Evidence Lab](EVIDENCE-LAB.md) was merged into main through
  PR #9 (71142a2, workspace 0.4.0-rc.1); the PR #10 review repairs followed as
  0c17f6e and became version 0.4.0 ([status](../CANDIDATE-STATUS.md)).
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
- Recorded Ocean Scale 1M/10M synthetic full-scan measurements (historical;
  the research archive was withdrawn in 0.4.0).
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
4. Reproduce numeric Q8 evidence on an independent second host, including
   regressions and complete memory costs. Keep addressed readout and full scan
   distinct; coordinate views are not independent information.
5. Evaluate knowledge retrieval only after defining a redistributable corpus,
   an explicit public encoder, held-out questions and abstention policy. Phrase
   matching, source integrity and synthetic numerical ranking are not semantic QA.

## After v0.5.3 (on `main`)

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Answer-or-abstain set v6: 990 new questions frozen before any run, used internally before (an experiment and an analysis of its errors) without changing any build or setting; the v4 build unchanged beside two BM25 engines, the third held-out set with the same result | PR #62 | `xtask answer-bench check` | [set v6](answer-or-abstain-v6/README.md) | questions from an independent writer; a held-out check of the next change |
| GEL beside two search engines and three language models on 400 questions of set v6 drawn by a fixed seed, with an 11-minute film of 150 of them: the models, closed book, 97–230 wrong answers each, GEL 10, the engines 13 and 14 | PR #63 | `answer-bench score v6:with-answer` | [side by side](GEL-BESIDE-ENGINES-AND-LLMS-V6.md) | models given the bank; an independent question writer |

## v0.5.3

A documentation and evidence release; public file formats and third-party dependencies
are unchanged from v0.5.2. `answer-bench` gains question set v5; the disclosure gate no
longer carries a word list; documentation is English only, with a guard.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| The project goal restated: a text AI that answers in Polish or English from its sources, or says it does not know, and holds a free conversation in both languages; still a goal, not a result | ba1884b (PR #45) | `tools/readme_presentation.rs`, workflow `readme-presentation` | [README](../README.md), [start here](START-HERE.md) | a public measurement of conversation in both languages |
| One license set with pinned file hashes, Wikipedia text attributed under CC BY-SA 4.0, documentation in English only with a guard, private results reported by their outcome only | aec70d5 to cfec9fb (PR #46 to PR #53) | `xtask verify` (licensing, disclosure) | [licensing](../LICENSING.md), [license pins](LICENSE-PINS.md), [third-party notices](../THIRD-PARTY-NOTICES.md) | an independent legal review of the license |
| README numbers made consistent: each result card names its question set and build, exact counts for the answer verdict, earlier versions kept out of the tree | c49d2c3, 804db2f, dfb2857 (PR #54 to PR #56) | `tools/readme_presentation.rs`, `xtask claims` | [README](../README.md), [measured progress](MEASURED-PROGRESS.md) | — |
| Answer-or-abstain set v5: 987 new questions frozen before any run; the v4 build unchanged and one candidate change beside two BM25 engines; the candidate rolled back under the rule fixed in advance | b3970bb (PR #57) | `xtask answer-bench check` | [set v5](answer-or-abstain-v5/README.md) | questions from an independent writer; a further held-out set for the next change |
| Fixes from three reviews of the v0.5.3 draft: the cut of the answer-bank slots to 167,854 passages stated, a timing ratio rounded correctly, one frozen review fixture sentence reworded with its old SHA-256 recorded, the README example card credited under CC BY-SA 4.0, stale sentences and one broken link | f64596a (PR #59) | `xtask verify`, `cargo run -p gel-source --example collection_review` | [measured progress](MEASURED-PROGRESS.md), [fixture provenance](../crates/gel-source/fixtures/collection-review/PROVENANCE.md) | — |
| Fixes from a second review of the v0.5.3 draft: every slot, not only the English ones, was cut at its end to the size of the smallest, with the number of passages each cut removed; Polish scope change −26–27% | PR #60 | `xtask verify` | [measured progress](MEASURED-PROGRESS.md) | — |

## v0.5.2

A documentation and evidence release; public file formats and third-party dependencies
are unchanged from v0.5.1. `xtask` gains `answer-bench` (re-scoring the recorded answers
of the v1 to v4 sets), `crash-series` and the public disclosure gate `disclosure`.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Public evidence and CI hardening: fail-closed workflow permission and action-pin checks, full benchmark answer inventories, 40,000 bounded format mutants | 7a4d237 (PR #19) | `xtask verify` | [hardening audit](AUDIT-HARDENING.md) | independent second-host acceptance; human visual review; power-loss guarantees |
| Answer-or-abstain set: 160 frozen questions with the recorded answers of GEL and three models, re-scored under strict and published rules, registered as a `SEPARATE_GATE` claim | 5c6342d (PR #31) | `xtask answer-bench` | [set and method](answer-or-abstain/README.md) | the answers are recorded, not re-run; a larger, independently written question set |
| No-answer control: 80 questions with no correct answer, GEL beside three language models, one run, registered as a `MEASURED_LOCAL` claim | 57acddf (PR #30) | `xtask claims` | [no-answer control](GEL-BESIDE-GROQ-NO-ANSWER.md) | GEL answered 6 of 40 false premises; independent reproduction |
| README "Evidence you can inspect": figures drawn from the recorded public data, goals labelled as not achieved | fd7a469 (PR #32) | `tools/readme_presentation.rs`, workflow `readme-presentation` | [README](../README.md) | a human review of the rendered page |
| The project goal stated as a goal, not a result | 0dc4c64 (PR #34) | `tools/readme_presentation.rs` | [README](../README.md) | — |
| What is different here and what can be checked without the private code; pages brought up to date after an external review | 7367313 (PR #35) | `xtask docs-refs`, `xtask answer-bench` | [README](../README.md), [verified results](VERIFIED-RESULTS.md) | an independent reading by a second reviewer |
| Crash series: a growing collection in the public tool killed at random moments; acknowledged snapshots, partial snapshots and resume checked, registered as a `SEPARATE_GATE` claim; the author-run series on the private store registered as `MEASURED_LOCAL` | 1075225 (PR #36) | `xtask crash-series` (5 trials in `xtask verify` on Unix) | [crash series](CRASH-SERIES.md) | a power-loss test; runs by independent operators |
| Answer-or-abstain set v2: 394 new frozen questions, GEL beside three language models in one run, re-scored by `answer-bench`; registered as a `MEASURED_LOCAL` claim | 1f27a89 (PR #37) | `xtask answer-bench` | [set v2](answer-or-abstain-v2/README.md) | a question set written independently of the project; an independent reproduction |
| Answer-or-abstain set v3: 979 new frozen questions; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank, all four GEL runs on the set published; `answer-bench` also checks precision with Wilson intervals, the split by language and paired exact tests; registered as a `MEASURED_LOCAL` claim | 4690933 (PR #42) | `xtask answer-bench` | [set v3](answer-or-abstain-v3/README.md) | a clean held-out set for the final GEL build; a tuned BM25 baseline; an independent reproduction |
| Public/private boundary: author-run measurements of the private implementation labelled as such on every results page, engine telemetry removed from public pages | 0a0c089 (PR #38) | `xtask docs-refs` | [measured progress](MEASURED-PROGRESS.md) | an independent reading of the boundary |
| Public disclosure gate in `xtask verify`: private paths, internal names (compared by hash), network addresses, credentials and e-mail addresses other than the published contacts | 2f7876a (PR #39) | `xtask disclosure` | [gate](PUBLIC-DISCLOSURE-GATE.md) | a review of meaning; the gate finds only what it is told to look for |
| Paths for people who do not write code: break it, evidence integrity, corpus and review, with issue forms and labels | 579032d (PR #40) | — | [contributing](../CONTRIBUTING.md) | first outside reports |
| Answer-or-abstain set v4: 985 new questions written after the tested change was fixed, frozen before any run; passages searched with their article title and section heading; GEL at two settings beside Tantivy BM25 and SQLite FTS5 on the same bank and on the v3 bank; registered as a `MEASURED_LOCAL` claim | 0190687 (PR #43) | `xtask answer-bench` | [set v4](answer-or-abstain-v4/README.md) | a released build; a tuned BM25 baseline; an independent reproduction |

## v0.5.1

A documentation and version release; public file formats and dependencies are
unchanged from v0.5.0. `xtask verify` gains one documentation check (the summary
of the review work list below).

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Answer-verdict and natural-question results of the private implementation, registered as `MEASURED_LOCAL` claims | 739c452 | `xtask claims` | [measured progress](MEASURED-PROGRESS.md) | independent reproduction; a larger, independently written question set |
| GEL beside three language models (closed book) on the same 80 frozen questions, one run, registered as a `MEASURED_LOCAL` claim | b85ce20 (PR #29) | `xtask claims` | [side by side](GEL-BESIDE-GROQ.md) | independent reproduction; a larger question set |
| Review work list summary recounted from its table on every verify run | PR #33 | `xtask roadmap` | [review work list](#review-work-list-and-next-acceptance) | none beyond the check itself |

## v0.5.0

Finishing a tool, measuring performance and independent acceptance are three
different events. This table records the first two with their commits; the last
column says what acceptance still needs. The A01–A24 list below keeps its own
states.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Limits stated without naming private components | 6b3b6b9 | — | current documents | owner-approved content allowlist and manual review of meaning; history, forks and copies keep earlier text |
| Network-isolation evidence | 8693840, 9937e1a, 9555b6e, 133d225 | `xtask isolation-check` | CI fails it on the networked runner and passes it in a namespace | Linux only; macOS and Windows report NOT_VERIFIED |
| Batch mode, schema gel-evidence/1 | 40c107a; peak_rss_kb in dc38282 | `gel-evidence --batch` | [schema](EVIDENCE-BATCH.md), batch tests | review by an outside consumer of the schema |
| Backup, restore, withdraw, delete, clear | fc710d4 | `gel-backup` | [backup](BACKUP.md), unit and CLI tests, full-disk CI | Windows directory durability not confirmed; independent review |
| Format mutation matrix | 855af81 | `xtask mutation-matrix` | [matrix r1](evidence-mutation/matrix-r1.txt), checked by verify | finite matrix on small fixtures, not exhaustive |
| Comparison with grep and sha256sum | dc38282, 940f5c0 | `xtask bench-compare` | [run r1](evidence-bench-r1/summary.txt) | one host; an independent second host; does not close A09 (single-mutation costs) |
| GEL search times beside the baseline | 1738220 | — | README table, author-reported | a same-task comparison (A21) before any speed-up factor |
| Public verification suite runner | 80f91d7 to 133d225; counts and `--strict` in 4cbfff5 | `xtask reproduce` | REPRODUCTION.txt of an owner run at 133d225: 3 of 3 steps passed, isolation verified | an independent operator on a second host (A20) |
| Documentation review fixes: work branch, historical baseline, security surface | 7bcca67 | — | README, TRY-IT, REPRODUCE, SECURITY | — |
| Still of the update, restart and corrupted-copy scenario | 3c088cc | — | [screenshot](../media/evidence-lab/03-update-restart-102s.png), final screen of an AI-reviewed recording; the recording is not published | — |
| Binary packages for Linux, macOS and Windows, built only in CI | 3c088cc | `xtask package-binaries`, workflow `binaries` | [binaries](BINARIES.md): license files checked against the inventory, build-path scan, per-platform smoke run, checksums; provenance attestation on a hand-started run | first owner-started run; packages are unsigned and checked only on their build runners |
| Fresh paired timing and fresh-process RSS of one mutation (A09) | 306c7f1 | `xtask mutation-campaign` | [campaign r1](evidence-mutation-campaign-r1/CAMPAIGN.txt), [results and limits](MUTATION-COMPARISON.md) | one host, 10 processes per combination; a second host |
| Private previews described only by scope and limits; removed from the tree on 2026-10-04 | 1f7693c, ffa85b6 | — | the public Evidence Lab film keeps its full [transcript](../media/TRANSCRIPTS-EN.md) | — |
| Property-to-test map (A07) | fc5cf90 | `xtask ci-evidence` | [map](PROPERTY-TESTS.md): 35 properties, 71 tests, one `PROPERTY` line per row in each platform report | properties without a row are not claimed untested; a row shows the tests ran, not full coverage |
| Claim registry for the v0.5.0 tools and every README timing table (A19) | 118e86d | `xtask claims` | [claims](CLAIMS.md) | not an exhaustive map of historical statements |
| README visual entry: six edited log replays of recorded public workflows | 034cd33 (PR #14) | `tools/readme_media.rs` | [gallery](../media/gifs/README.md), [source and hashes](../media/gifs/MANIFEST.txt) | edited replays, not wall time; the one-off publishing workflow is not part of the release |
| Multimedia README, animated citation diagram, checked facts and workflow badges; script-free full-page edition | 9e1b63d (PR #16), b8ed840 (PR #17) and the animated-graphics pull request | `tools/readme_presentation.rs`, workflow `readme-presentation` (read-only check) | [presentation guide](README-PRESENTATION.md), [full-page edition](../README-MULTIMEDIA.html) | built locally and rechecked in CI; a human review of the rendered page in browsers |

## How to read the review work list

Implementation state and acceptance state are separate. `DONE_SCOPED` means the
change and its checks are in the repository at this revision, within the stated
scope; it is not independent acceptance. `PARTIAL` means a stated part is
implemented or documented while an acceptance condition remains open. `OPEN`
means this public inventory does not establish the implementation or evidence.
`OWNER` means the remaining step is a repository setting or owner decision, not
code. No completion percentage is calculated from unequal tasks.

The identifiers A01 to A24 retain the order of the 24-point external review.
Evidence links are contracts and entry points, not blanket PASS labels. For each
accepted item, retain the implementation commit, fixture or corpus hash, exact
command, expected result and counterexample, actual result, platform/profile,
run attempt, reviewer and remaining exclusions. A second-host reproduction must
identify its independent operator rather than invent one.

## Review work list and next acceptance

Summary: 14 DONE_SCOPED · 6 PARTIAL · 3 OPEN · 1 OWNER.

| ID | State | Work and next acceptance condition | Evidence / entry point |
|---|---|---|---|
| A01 | DONE_SCOPED | Historical source/lockfile mapping and a public R1 reproduction procedure exist; old hashes are kept and both positive and changed-source checks pass. | [Measured-source guide](evidence-collection/README.md) |
| A02 | DONE_SCOPED | CI rechecks the retained R1 dataset and rejects missing, duplicated or inconsistent evidence. This is not a new timing campaign. | [R1 negative checks](R1-NEGATIVE-CHECKS.md) |
| A03 | DONE_SCOPED | Bounded quote context and omission flags with negation, condition, unit and line-ending regressions; no claim of unrestricted context understanding. | [Quote context](QUOTE-CONTEXT.md) |
| A04 | DONE_SCOPED | Runtime-example sequencing checks each native exit status, including first/middle failures and misleading PASS output. | [Process tests](../xtask/tests/process_sequence.rs) |
| A05 | PARTIAL | Decode report and an AI review of every visually distinct frame of all three films exist. A human start-to-finish review remains open. | [Media review](MEDIA-DECODE-REVIEW.md) |
| A06 | DONE_SCOPED | The recorder fails closed on split UTF-8, EOF, failed writes, missing markers and child exit; a lint gate forbids 14 lints and rejects 10 probes. | [Recorder safety](RECORDER-SAFETY.md) |
| A07 | DONE_SCOPED | Reports sum cargo's test results per profile, list 7 declared Unix-only tests and 2 partial branches, and a strict cross-platform diff checks them. A property-to-test map of 35 documented properties and 71 uniquely named tests is checked on every CI platform; it is not an exhaustive map of every statement. | [CI evidence](CI-EVIDENCE.md), [property map](PROPERTY-TESTS.md) |
| A08 | DONE_SCOPED | Structural, admission and OS-level publication faults (permission denial, SIGKILL, full disk on a bounded tmpfs) keep the previous snapshot; possible publication before a directory-sync error is documented. | [Publication fault tests](PUBLICATION-FAULT-TESTS.md) |
| A09 | DONE_SCOPED | Allocations, user-space copies and peak additional heap per mutation are measured locally (DHAT, heaptrack). A fresh paired timing comparison and the resident-set peak of one mutation in fresh processes, with the peak reset before the mutation, are recorded on one host; a second host remains open. | [Mutation comparison](MUTATION-COMPARISON.md), [builder](COLLECTION-BUILDER.md) |
| A10 | PARTIAL | A comparison protocol (input, profile, clock boundary, warmup, order, host metadata) exists and governs the [grep comparison](BENCHMARK-GREP.md); historical campaigns are not re-run under it. | [Campaign protocol](EVIDENCE-CAMPAIGN.md) |
| A11 | DONE_SCOPED | A public F32/F16/affine Q1–Q16 reference with a versioned container, independent oracles and retained output exists. It does not establish integration with every GEL reader/writer or with any private format. | [Precision matrix](PRECISION-MATRIX.md), [codec scope](CODEC-SCOPE.md) |
| A12 | DONE_SCOPED | Bytes, numeric error, ranking and context are separate acceptance results, not derived from transport PASS. | [Claims](CLAIMS.md), [codec scope](CODEC-SCOPE.md) |
| A13 | DONE_SCOPED | Sanitized per-platform CI reports of selected revisions are kept in the repository, because Actions artifacts and logs expire. | [CI evidence](CI-EVIDENCE.md), [recorded reports](evidence-ci/README.md) |
| A14 | DONE_SCOPED | README entry point, fresh-checkout instructions and a first-screen link to the public Evidence Lab walkthrough; main, review branches and the historical tag are distinct. | [README](../README.md) |
| A15 | OWNER | Description and topics are repository settings. A social-preview image is prepared; uploading it is a manual settings step. | [Repository](https://github.com/Gelram-project/gel-ram) |
| A16 | DONE_SCOPED | One film index lists the public walkthrough with its destination and scope label; the private previews were removed from the tree on 2026-10-04. | [Film index](../media/INDEX.md) |
| A17 | PARTIAL | A two-process update/restart scenario and tests exist. One still of the final screen of a recording of it is published; the recording itself is not. | [Update/restart scenario](UPDATE-RESTART-SCENARIO.md) |
| A18 | PARTIAL | An English descriptive transcript of the public Evidence Lab film exists; the private previews were removed from the tree on 2026-10-04. Human confirmation of the timeline remains open; translations are not new engine outputs. | [Transcript](../media/TRANSCRIPTS-EN.md) |
| A19 | PARTIAL | An executable claim registry separates executable checks, separate gates, locally measured rows and deferred rows, and rejects any other state. It covers the v0.5.0 tools and every README timing table; it is not yet an exhaustive map of historical statements. | [Claims](CLAIMS.md) |
| A20 | OPEN | Reproduction form and a [verification suite runner](REPRODUCE.md) exist. Obtain and retain an independent second-host campaign and first-use feedback. Hosted CI alone does not close this item. | [Reproduction form](../.github/ISSUE_TEMPLATE/reproduction.yml) |
| A21 | PARTIAL | Scoped author-reported component measurements are published. An identical-task, end-to-end comparison with the exact baseline remains open. | [Component scope](GEL-EXPERIMENTAL-MEASUREMENTS.md) |
| A22 | OPEN | Implement a public execution-identity gate only within approved disclosure scope. Substituting the full-scan baseline must fail that gate even when results match. | [Baseline boundaries](OCEAN-SCALE.md), [claims](CLAIMS.md) |
| A23 | OPEN | Adaptive precision needs a declared error budget, independent oracle, full metadata costs and stable task decisions, including weak-signal failures. | [Codec scope](CODEC-SCOPE.md) |
| A24 | DONE_SCOPED | PR #10 was built in small commits, each with the source manifest, full verify and a leak scan; later PRs follow the same practice. A green job alone does not authorize a merge, release or disclosure. | [PR #10](https://github.com/Gelram-project/gel-ram/pull/10) |

## Execution order and dependencies

### M0. Coherent information and review scope

Update this roadmap, README, publication-status scope and PR description together
with the source manifest. Acceptance: readers can distinguish merged features,
branch-only changes, historical evidence and open gates; PR metadata matches the
actual diff. This is a maintained snapshot, not an automatic status synchronizer.

### M1. Integrity and failure-path acceptance

Review A01-A04, A06-A08, A12-A13 and A19 at one exact revision. Run configured
native checks and retain both success and expected-rejection evidence. Separate
finite fixture coverage, ignored/platform-excluded tests, repeated test
executions, doctests and debug/release profiles. Do not mark a gate PASS from
prose alone.

### M2. Reproducible user experience

After the relevant M1 checks, close the visual parts of A05 and A14-A18. Review
all three films from start to finish, check original bytes and identifiers,
record reviewed time ranges and reconcile captions with actual logs. Keep
historical films unchanged; publish a new recording only after reviewing the
A17 scenario. A decode-only report or sampled screenshots cannot close M2.

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
execution-identity checks together. Use the public [precision reference](PRECISION-MATRIX.md)
as a scoped starting point, not as proof of private codec or application
integration. A public retrieval-quality claim additionally needs a
redistributable corpus, a documented public encoder, held-out questions and an
abstention policy. These public gates do not publish private engines, banks or
other private internals.

### M5. Separately approved release

A bounded release does not require pretending that all longer-term research is
complete. Freeze its declared scope, carry open items and exclusions into release
notes, verify the exact tree/assets/manifest and configured CI, and obtain separate
maintainer release approval. Any included media must have its stated M2 review.
M3/M4 are prerequisites only for performance or capability claims included in
the release. Do not move the historical v0.3.0 tag or infer approval from a
green job.

## Published baseline and longer-term direction
The tagged `v0.3.0` release remains an earlier immutable release point. Public
`main` has advanced since; later tagged releases are listed on the Releases page.

## Public performance boundary

The historical, author-reported 1M/10M Ocean measurements are retained as a
**CPU/RAM full-scan baseline**: CPU workers scan, score, rank and select records
resident in memory. They are valuable comparison evidence, but they are not
presented as the final GEL RAM execution model.

The private project contains newer experimental work. Author-reported component
measurements are published separately in
[GEL-EXPERIMENTAL-MEASUREMENTS.md](GEL-EXPERIMENTAL-MEASUREMENTS.md); they
measure different tasks and establish no end-to-end advantage or multiplier over
this baseline. Any such claim requires an exact public reproduction package.

## Next public acceptance gates

1. **Independent reproduction on a second host.** Re-run the public Q8 and
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

- public workspace verification on additional CPU families;
- corruption/restart/concurrency edge cases;
- Unicode/source-boundary cases for the document reader;
- complete memory-cost measurements;
- slower or negative performance results that help identify the real limits.

Start with [TRY-IT](TRY-IT.md) and
[VERIFIED RESULTS](VERIFIED-RESULTS.md). A benchmark report does not require a
code contribution or a CLA; code intended for merge follows [CONTRIBUTING](../CONTRIBUTING.md).

## Research boundaries retained

The public project does not currently establish:

- unrestricted semantic question answering;
- universal speedup on all hardware or workloads;
- stable p99.9–p99.999 from campaigns with too few observations;
- physical memory-side compute;
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
