# Development priorities

## Public status and current work

- Tagged `v0.3.0` remains an immutable earlier release.
- Ocean Scale R3 is on `main`: numerical 1M/10M evidence, bounded offline
  verification and Linux-only research persistence/mapping. See [scope](OCEAN-SCALE.md).
- Document readout is on `main` through [PR #4](https://github.com/Gelram-project/gel-ram/pull/4):
  exact source quotations, Unicode/CR regressions and runtime CI on three OSes.
- [Live Lab](LIVE-LAB.md) and [source building](SOURCE-BUILDER.md) were published
  in PR #5. [Evidence Lab](EVIDENCE-LAB.md) is the new local-only candidate;
  [status](../CANDIDATE-STATUS.md) explicitly withholds publication permission.

These public components do not include the private speaker, encoder, knowledge
banks, encrypted user profiles or private P2P application. Existing videos are
previews of that separate application, not demonstrations of every public API.

## Next acceptance gates

1. Review the multi-document candidate: independent source boundaries, stale
   citation invalidation, snapshots and independently recomputed raw measurements.
   Single-document import/read/save/reopen already belongs to the published base.
2. After separate publication permission, run the same reviewed revision on Linux, Windows and macOS. Unix permission
   and directory-sync checks remain platform-specific, not portable guarantees.
3. Provide one-command local reporting with raw output, resource/timing scope and
   failures retained. Never infer robust performance from a single UI timing.
4. Reproduce numeric Q8/Ocean evidence on an independent second host, including
   regressions and complete memory costs. Keep addressed readout and full scan
   distinct; coordinate views are not independent information.
5. Evaluate knowledge retrieval only after defining a redistributable corpus,
   an explicit public encoder, held-out questions and abstention policy. Phrase
   matching, source integrity and synthetic numerical ranking are not semantic QA.

## Research boundaries retained

- F0.5: cache/NUMA topology, pages, bandwidth beyond LLC and worker contention.
- F1.5: measure conditional information in Reader16 judgments; isometries alone
  cannot establish new independent evidence.
- F2.5: account for predictor pools, residuals, metadata and context-touch cost.
  The structural codec's in-memory exactness is not an on-disk format guarantee.
- F3: selective locator/sketch experiments need membership and recall checks.
- F4: retain 1M/10M latency, memory, exactness and failure-behaviour evidence
  together; synthetic scale results do not establish general AI accuracy.

The historical lexical [v0.2.1 evaluation](VALIDATION-v0.2.1.md) reported
Recall@10 0.480 / 0.270. It is neither a current Q8 result nor private-system
accuracy. Historical R2 reports and former PR numbers describe their original
review context; they are not outstanding tasks or PRs in this repository.

## Publication policy

Prefer a bounded, independently runnable capability over a larger archive.
No private module is automatically approved because a public test passed.
Keep licensing, source inventories, exact-revision CI and evidence provenance
explicit. No arbitrary lossless compression, general conversation, production
power-loss durability, physical DRAM-refresh coupling or universal speedup is
promised. A future version tag requires its own final release decision.
