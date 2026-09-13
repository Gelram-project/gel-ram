# Release gates

A release candidate is green only when all applicable gates pass on the same source tree.

1. Rust-only source/tooling gate.
2. Licensing-mode gate (PolyForm Noncommercial 1.0.0 only).
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

Performance results are evidence, not correctness substitutes.

## Additional Q8 preview checks on main

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

Historical V1 timing results remain frozen. Any new V2 performance claim
requires new timing evidence with effective budgets and fallback counts.
macOS/Windows CI currently checks compilation, not runtime behavior.

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
