# Source candidate: validation index and release gates

Local candidate, not a published version. This document separates implemented
checks from approvals that software tests cannot supply.

## Reproduction and scope

Recorded Linux verification for the preceding repair snapshot: **194 tests passed**, including six
new report/licensing-scope regressions compared with the earlier 188-test run.
The [recorded output](evidence-source-candidate/verify-local.txt) preserves all
test results and runtime demo output. Only the local checkout path was replaced
by a SOURCE_ROOT placeholder. The log predates adding this evidence/documentation;
it is not a claim of remote CI or a final archive hash. Test timings are incidental,
not a new performance campaign or semantic accuracy result.

The subsequent inactive-license/privacy follow-up adds one regression. The
historical log above does not certify that changed snapshot. Re-run verification
and pin the resulting source tree before review or release. Portable CI now also
records revision/toolchain and runs operative licensing and the independent
integrity audit; configuring those steps is not evidence that they have run.

```text
cargo test --locked --offline --workspace --all-targets
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p gel-source --example source_real
```

- Multipart: 11 tests plus 12 existing source-catalog tests; explicit ordering,
  missing/ambiguous titles, incomplete numbering, Unicode and bounded output.
- Synthetic source example: three tests. Real-source example: five tests,
  including 1543 single-byte mutation attempts with the original approval pins.
  Those mutations measure integrity, not 1543 semantic questions.
- Archive report: independent pin required, modified/extra files rejected,
  broken Git not accepted as archive, command failure/false status/fallback rejected.
- Licensing guards: selected root scope and attribution checks, preserved
  upstream MIT license hash and regression checks on the inactive proposal.
  These are not a legal parser, proof of ownership or legal approval.

The earlier [multipart validation](SOURCE-PARTS-VALIDATION.md) describes that
addition alone. [Real-source provenance](REAL-SOURCE-DEMO.md) identifies the
independent upstream excerpt. Historical Q8 timing evidence is unchanged.

## Archive report

```text
cargo run --locked --offline -p xtask -- report ../gel-report-new REVIEWED_MANIFEST_SHA256
```

The pin must be obtained independently from the approved review, not accepted
from an arbitrary download together with its own hash. Archive identity records
Git revision as UNAVAILABLE and validates every source file before and after
the campaign. The caller uses the source tree embedded when xtask was compiled.
Git mode without a pin records revision/status, not a complete content snapshot.
Transient changes between checks and hostile filesystem races are outside this
trusted-local-filesystem protocol. Never use the report runner as a sandbox for
untrusted code. Use a separate sandbox when executing candidates.

Full reporting runs verification then the declared synthetic scan matrix:
32/512/8192 records, both policies, 1 and at most 24 available workers,
three repetitions, nine rounds. Full-worker reference fallback invalidates the
campaign; slow ratios remain in its logs. This is not the full historical R2
campaign or a new measurement of semantic retrieval.

## Still required before release

- A final reviewed source snapshot, corresponding public-safe evidence and
  a final archive hash. Recheck after any source or documentation change.
- Actual remote platform CI for that exact change; configured jobs are not
  evidence that Windows/macOS have run locally or remotely for this candidate.
- Licensor/chain-of-rights/CLA privacy review and review by appropriate counsel,
  followed by an atomic licensing cutover if approved. Root PolyForm remains.
- Explicit approval of the public file list. Do not include private audit logs,
  original private corpora, model weights, account records or the private engine.

No source-integrity check, test count or secret scan closes those release gates.
