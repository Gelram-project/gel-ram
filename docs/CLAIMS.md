# Claim and counterexample registry

```sh
cargo run --locked --offline -p xtask -- claims
```

The canonical registry is Rust code in [claims.rs](../xtask/src/claims.rs).
Each row defines its scope, input fixture, expected behavior, counterexample,
source and evidence mode. The command runs the executable rows against the
current library and emits a tab-separated report. A failed check exits nonzero.
`xtask verify` runs this gate. The table below is checked byte-for-byte against
the registry; editing a prose status cannot manufacture a runtime PASS.

<!-- REGISTRY-BEGIN -->
| ID | Dimension | Evidence mode |
|---|---|---|
| source-roundtrip | bytes | EXECUTABLE_CHECK |
| stale-citation | provenance | EXECUTABLE_CHECK |
| hash-not-structure | structure | EXECUTABLE_CHECK |
| bounded-context | context | EXECUTABLE_CHECK |
| no-cross-document | retrieval | EXECUTABLE_CHECK |
| numeric-loss | numeric | SEPARATE_GATE |
| ranking-oracle | ranking | SEPARATE_GATE |
| mutation-memory | memory | MEASURED_LOCAL |
| publication-os-faults | persistence | SEPARATE_GATE |
| platform-exclusions | test-scope | SEPARATE_GATE |
| recorder-fail-closed | tooling | SEPARATE_GATE |
| media-full-review | presentation | NOT_VERIFIED |
| physical-refresh-compute | mechanism | NOT_ESTABLISHED |
| commercial-advantage | comparison | NOT_ESTABLISHED |
<!-- REGISTRY-END -->

PASS applies only to the stated finite fixture and its counterexample. It does
not prove arbitrary-language understanding, all possible malformed inputs or
truth of source content. Bytes, numeric loss, ranking and context are separate
dimensions. SEPARATE_GATE is not PASS: run the linked numerical/ranking checks
and retain their results independently. MEASURED_LOCAL means an owner-side
measurement is documented at the linked source; CI does not re-run it.
Deferred entries deliberately stay open.

This register is not yet an exhaustive map of every historic statement in the
repository. In particular it does not retroactively validate videos or timing
tables. Preserve the actual tested commit/source manifest and run attempt using
the [CI evidence report](CI-EVIDENCE.md); a SHA is an identity, not authentication.
