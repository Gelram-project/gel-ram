# Release gates

A release candidate is green only when all applicable gates pass on the same source tree.

1. Rust-only source/tooling gate.
2. Licensing-mode gate: the only accepted LICENSE-MODE.txt value is GEL RAM
   NCRL 1.0 + Commercial + CLA 2.0, the root LICENSE must match the pinned NCRL
   bytes, and the publication status must keep LEGAL_APPROVED=NO. This checks
   consistency of the active license files, not legal validity.
3. CI policy gate: pinned checkout SHA, read-only checkout credentials and a
   metadata-only CLA workflow that cannot execute pull-request code.
4. `cargo fmt --check`.
5. `cargo clippy -- -D warnings`.
6. all workspace tests.
7. release build of the complete workspace.
8. rustdoc with warnings denied.
9. integrated `gel-cli selftest`.
10. v2 header mutation: 512/512 single-bit and 130,816/130,816 two-bit flips rejected; v1 single-bit sweep frozen.
11. full Top-K == progressive Top-K.
12. structural original == decode bit-for-bit.
13. rollback generation <= current rejected.
14. pull request CLA acknowledgement (`xtask cla-ack` locally; trusted-base CI
    on `pull_request_target`, without checking out or executing pull-request
    code, including description-edit events).
15. multi-thread Top-1 == single-thread Top-1: gel-reader equality tests plus the gel-bench smoke run in verify (`thread_scan_exact=PASS`).
16. docs-refs gate: every backtick-quoted repository path in .md files exists.
17. recorder-lint: clippy restriction lints on the standalone film recorder,
    with injected panicking constructs that must be rejected (negative controls).
18. historical measured-source pins (`MEASURED_SOURCES_R1=PASS`).
19. claim registry (`xtask claims`): documented table equals the Rust registry
    and executable claims pass their positive and negative cases.
20. workspace documentation tests (`cargo test --doc`).
21. saved R1 collection recheck (collection_recheck over the published r1 evidence).
22. runtime demonstrations: `verify` runs the source, collection, Q8 and Live Lab
    examples, including gel-evidence --demo; Windows/macOS CI runs the fail-fast
    `xtask runtime-examples` sequence.
23. CI only: a physically full 1 MiB tmpfs publication check and the pinned
    Ocean archive tests (Linux), and the per-platform CI evidence report
    (`xtask ci-evidence`) on Linux, Windows and macOS.

Performance results are evidence, not correctness substitutes.

## Additional Q8 checks on main

- Shared scores agree with equivalent materialized views, including mask
  semantics and reversible transforms; these are not semantic-retrieval tests.
- Workspace tests exercise bounded budgets, serial paths, full and partial
  worker-start refusal, joined workers and complete reference-buffer recovery.
- `verify` includes a 32-record Q8 correctness smoke run with two requested
  workers, limited by available parallelism. This is not the 1/24-worker
  hardware performance matrix.
- [Separate local V2 validation](Q8-QUAD-VALIDATION.md) records an actual Linux
  OS-refusal probe and a 16-cell post-fix correctness matrix. That probe is
  not an automatic cross-platform CI gate or general OOM/panic certification.

Historical V1 and V2 timing results remain frozen under their own headers.
Any new performance claim for the current V3 comparison
requires new timing evidence with effective budgets and fallback counts.
macOS/Windows CI executes workspace tests, documentation tests, the fail-fast
runtime demonstrations and the independent integrity audit; the full `verify`
gate, pinned Ocean archive tests and the full-disk tmpfs check run on Linux only.

## Binary-core hardening checks

The [binary-core hardening](BINARY-GATE-HARDENING.md) adds:

- raw Rotate roundtrip coverage for every u16 field value;
- canonical sparse validation shared by counting and decoding;
- caller/child overlap before join, first/partial start refusal, complete
  retry after joining children and accurate per-call execution reports;
- deliberate incorrect timed-hit and non-first Top-8 result tests that must
  reject the result, not print a successful exactness marker;
- a documented distinction between unwind tests and panic=abort releases.

These gates do not prove knowledge accuracy, all possible allocation recovery
or platform-wide runtime coverage. CLA acknowledgement still checks a PR
declaration, not a private registry of signed agreements. Full source-manifest
verification remains a separate packaging check, not part of xtask verify.
