# Document-readout local validation evidence

2026-09-19. Base public commit:
`d0c9b67110e43bb9e72c463a55f1c084175bc3df`.
This table records the local candidate checks preceding publication. The owner
subsequently authorized the reviewed public diff; the evidence below does not
itself grant that permission. No new version or tag is introduced.

| Check | Result and scope |
|---|---|
| Root workspace, Rust 1.85.0 / Linux | 225 tests passed, 0 failed, including three review regressions |
| xtask verify | PASS: scope/license/CI/docs gates, fmt, Clippy with warnings denied, build, docs, tests and runtime examples |
| Independent byte/numeric/ranking audit | GEL_DATA_INTEGRITY_ALL=PASS; 256/256 deterministic ranking queries, not semantic accuracy |
| Unchanged Ocean R3 archive | 111 tests passed; pinned offline verifier and saved-evidence recomputation passed |
| New real-document demo | Source pins, exact original UTF-8 range, MISS and corruption rejection checked |
| New synthetic probe | 1000 raw observations, independently checked and recomputed, no extreme-tail claim |
| Dependencies | 12 locked external packages; cached archive SHA256 and recorded license-file bytes verified |
| Root LICENSE and two films | Unchanged SHA256; both films decoded completely on CPU without reported decoder errors |
| Windows/macOS during this local pre-publication check | Not run locally; see the subsequent exact-revision CI below |

Subsequent [CI for published commit c0058a1](https://github.com/Gelram-project/gel-ram/actions/runs/35467294385)
executed 225 workspace tests on Linux/macOS and 222 on Windows, all passing,
plus 111 Ocean tests on Linux. Three existing Unix-only tests explain the
Windows count difference. This remains historical document-update evidence,
not the test count of future additions such as Live Lab.

The 225 workspace tests and 111 separate Ocean tests are different suites.
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

PR review reproduced three failing regressions for standalone CR boundaries,
mixed CR/CRLF/LF byte offsets, and Greek sigma variants. The corrected reader
passes all three. The earlier 222-test snapshot and its diagnostic timing series
are retained, not overwritten. That older timing series is not a measurement of
the corrected revision. The current manifest identifies the current source.
