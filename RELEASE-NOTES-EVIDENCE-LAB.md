# Evidence Lab — unpublished local candidate

Base: `8b92deb912a159a78ae1e8a5f7e7234b717cbd6a` (published Live Lab / PR #5).
No permission to publish this candidate. No version/tag change. Root licensing
and external dependency versions are unchanged. No private implementation imported.

Local integration review also includes public documentation updates through
`44b6d7af4d1ffc5715585d2d5c5db42ee401e4f8` (PRs #6 and #7).
Those updates are already public, not new Evidence Lab functionality. The
unpublished additions below remain relative to the original implementation base.
Start with [candidate quickcheck](docs/CANDIDATE-QUICKCHECK.md).

## New, not recycled from the base

- Multi-document collection: independent IDs/titles/source hashes, bounded UTF-8.
- Exact original citations, revision/content-bound validation and invalidation on mutation.
- Add/replace/delete, plaintext no-replace snapshots and fresh-process reopen.
- `gel-evidence` Rust terminal; explicit UNKNOWN/INCOMPLETE, source IDs, proof command.
- A separate Q1/Q2/Q4/Q8 affine reference test with packed sizes and loss measurements.
- Reproducible 8/64/256-document workload: raw samples and independently recomputed summaries.
- Fault and subprocess tests, including Unix SIGKILL at observable publication and after save ACK.

The already published Ocean 1M/10M and single-document Live Lab are retained as
the base, not presented as new measurements. The two existing films do not show
this new application. A [separate new terminal recording](media/EVIDENCE-LAB-GUIDE.md)
now shows the candidate with a real process restart. `gel-evidence --demo` remains
a reproducible quick demonstration.

## Gates

Run `cargo run --locked --offline -p xtask -- verify`, then
`cargo run --locked --offline -p xtask -- report ../new-report REVIEWED_MANIFEST_SHA256`.
The latter directory must be new and outside the source tree. Includes corpus
snapshots, so review contents before sharing reports from your own data.
Cached dependencies and Rust 1.85.0 are prerequisites for offline compilation.

Candidate-specific Windows/macOS execution remains untested. Prepared workflows
do not count as executed CI. No external network CI is triggered by preparation.
Technical PASS does not change PUBLICATION_APPROVED=NO.

## Limits

Not semantic QA, a speaker, multimedia understanding, encrypted personal memory,
an anti-rollback protocol or hardware DRAM synchronization. Q reference and source
lookup are separate; they do not prove fourfold independent storage. The collection
snapshot includes full text and metadata, not compressed ORBs. Mutation rehashes
the entire snapshot; it is not an incremental index or constant-time update.
OS crash tests do not establish physical power-failure durability.

[Contract](docs/EVIDENCE-LAB.md) · [benchmark method](docs/EVIDENCE-CAMPAIGN.md).

Measured local results and raw evidence: [Linux review](docs/evidence-collection/README.md).

Follow-up preparation adds [24 post-freeze contract cases](docs/ASSESSMENT-REVIEW.md)
without runtime tuning and [native platform handoff](docs/PLATFORM-REVIEW.md).
This is developer-authored assessment, not independent blind validation.
