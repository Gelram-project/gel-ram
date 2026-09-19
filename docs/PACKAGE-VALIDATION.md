# Document-readout local validation evidence

2026-09-19. Base public commit:
`d0c9b67110e43bb9e72c463a55f1c084175bc3df`.
This table records the local candidate checks preceding publication. The owner
subsequently authorized the reviewed public diff; the evidence below does not
itself grant that permission. No new version or tag is introduced.

| Check | Result and scope |
|---|---|
| Root workspace, Rust 1.85.0 / Linux | 222 tests passed, 0 failed |
| xtask verify | PASS: scope/license/CI/docs gates, fmt, Clippy with warnings denied, build, docs, tests and runtime examples |
| Independent byte/numeric/ranking audit | GEL_DATA_INTEGRITY_ALL=PASS; 256/256 deterministic ranking queries, not semantic accuracy |
| Unchanged Ocean R3 archive | 111 tests passed; pinned offline verifier and saved-evidence recomputation passed |
| New real-document demo | Source pins, exact original UTF-8 range, MISS and corruption rejection checked |
| New synthetic probe | 1000 raw observations, independently checked and recomputed, no extreme-tail claim |
| Dependencies | 12 locked external packages; cached archive SHA256 and recorded license-file bytes verified |
| Root LICENSE and two films | Unchanged SHA256; both films decoded completely on CPU without reported decoder errors |
| New Windows/macOS execution | NOT TESTED; CI configuration includes new tests/demo but execution is not claimed |

The 222 workspace tests and 111 separate Ocean tests are different suites.
Neither is the private application's regression count or a semantic recall
percentage. Large 1M/10M timing campaigns were not rerun for this document
addition; their old raw evidence remains associated with its original snapshot.

The first integration verification caught the initially stale dependency
inventory after adding Unicode crates. The inventory was updated from exact
locked archive/license bytes; subsequent complete verification passed. Initial
failure logs remain in the operator's local audit record. Gates were not disabled.

Testing uses offline Cargo and a network-isolated Linux namespace. The root
source package references dependencies; a new machine needs the toolchain and
cached/fetched dependencies first. The embedded Ocean archives retain their
own vendored dependencies. Runtime demo needs no LLM, service or internet.

Publication flags are set only from the owner's authorization, never from
technical PASS. [Exact-revision CI](https://github.com/Gelram-project/gel-ram/actions)
records subsequent Linux/Windows/macOS runs separately. The active license
is unchanged and no new legal certification is asserted. This is not a guarantee
of freedom from every security issue, nor a substitute for review of the actual
publication diff and exact new CI revision.
