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
| gel-resident-addressed-read | timing | MEASURED_LOCAL |
| gel-source-integrity-controls | integrity | MEASURED_LOCAL |
| gel-single-quad-slot-ranking | ranking | MEASURED_LOCAL |
| gel-slot-ranking-update | ranking | MEASURED_LOCAL |
| gel-source-field-dialogue-timing | timing | MEASURED_LOCAL |
| gel-new-article-transfer | retrieval | MEASURED_LOCAL |
| gel-answer-verdict-self-read | retrieval | MEASURED_LOCAL |
| gel-natural-question-answers | retrieval | MEASURED_LOCAL |
| gel-answer-set-v2 | retrieval | MEASURED_LOCAL |
| store-kill-during-learning | persistence | MEASURED_LOCAL |
| gel-beside-bm25-v3 | comparison | MEASURED_LOCAL |
| gel-beside-bm25-v4 | comparison | MEASURED_LOCAL |
| gel-beside-bm25-v5 | comparison | MEASURED_LOCAL |
| gel-beside-bm25-v6 | comparison | MEASURED_LOCAL |
| gel-beside-bm25-v7 | comparison | MEASURED_LOCAL |
| source-roundtrip | bytes | EXECUTABLE_CHECK |
| stale-citation | provenance | EXECUTABLE_CHECK |
| hash-not-structure | structure | EXECUTABLE_CHECK |
| bounded-context | context | EXECUTABLE_CHECK |
| no-cross-document | retrieval | EXECUTABLE_CHECK |
| numeric-loss | numeric | SEPARATE_GATE |
| ranking-oracle | ranking | SEPARATE_GATE |
| q8-one-record-four-views | bytes | SEPARATE_GATE |
| q8-literal-record | bytes | EXECUTABLE_CHECK |
| q8-record-save-stopped | persistence | SEPARATE_GATE |
| q8-score-bits-across-platforms | numeric | SEPARATE_GATE |
| record-history-exact | bytes | EXECUTABLE_CHECK |
| sha256-fips-vectors | bytes | EXECUTABLE_CHECK |
| orb-record-layout | bytes | EXECUTABLE_CHECK |
| structural-exact-xor | structure | EXECUTABLE_CHECK |
| q8-record-size | bytes | EXECUTABLE_CHECK |
| no-third-party-packages | distribution | EXECUTABLE_CHECK |
| no-network-code | environment | EXECUTABLE_CHECK |
| mutation-memory | memory | MEASURED_LOCAL |
| publication-os-faults | persistence | SEPARATE_GATE |
| platform-exclusions | test-scope | SEPARATE_GATE |
| recorder-fail-closed | tooling | SEPARATE_GATE |
| crash-series-no-acknowledged-loss | persistence | SEPARATE_GATE |
| record-history-kill-series | persistence | SEPARATE_GATE |
| batch-contract | interface | SEPARATE_GATE |
| answer-bench-rescoring | scoring | SEPARATE_GATE |
| backup-restore | persistence | SEPARATE_GATE |
| network-isolation | environment | SEPARATE_GATE |
| format-mutations | structure | SEPARATE_GATE |
| history-format-mutations | structure | SEPARATE_GATE |
| binary-packages | distribution | SEPARATE_GATE |
| property-map | test-scope | SEPARATE_GATE |
| tool-version-license | interface | SEPARATE_GATE |
| source-license-headers | provenance | SEPARATE_GATE |
| license-self-check | licensing | SEPARATE_GATE |
| unicode-16-normalization | text | SEPARATE_GATE |
| readme-generated | presentation | SEPARATE_GATE |
| version-line | interface | SEPARATE_GATE |
| goal-statement | presentation | SEPARATE_GATE |
| commitments-fixed-in-time | provenance | SEPARATE_GATE |
| rebuild-identical | distribution | SEPARATE_GATE |
| grep-comparison | comparison | MEASURED_LOCAL |
| mutation-timing | timing | MEASURED_LOCAL |
| gel-sketch-search | timing | MEASURED_LOCAL |
| ocean-full-scan | timing | MEASURED_LOCAL |
| crash-series-recorded | persistence | MEASURED_LOCAL |
| q8-recorded-runs | bytes | MEASURED_LOCAL |
| speed-set-against-v0.6.1 | timing | MEASURED_LOCAL |
| speed-set-quiet-machine | timing | MEASURED_LOCAL |
| speed-sets-rerun | timing | MEASURED_LOCAL |
| media-full-review | presentation | NOT_VERIFIED |
| independent-reproduction | reproduction | NOT_VERIFIED |
| record-history-durability | persistence | NOT_ESTABLISHED |
| hardware-memory-compute | mechanism | NOT_ESTABLISHED |
| commercial-advantage | comparison | NOT_ESTABLISHED |
| text-ai-goal | goal | NOT_ESTABLISHED |
| q8-semantic-accuracy | retrieval | NOT_ESTABLISHED |
| snapshot-power-loss | persistence | NOT_ESTABLISHED |
| extreme-tail-latency | timing | NOT_ESTABLISHED |
<!-- REGISTRY-END -->

Rows named gel-beside-bm25-* are external checks; see [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with).

PASS applies only to the stated finite fixture and its counterexample. It does
not prove arbitrary-language understanding, all possible malformed inputs or
truth of source content. Bytes, numeric loss, ranking and context are separate
dimensions. SEPARATE_GATE is not PASS: run the linked numerical/ranking checks
and retain their results independently. MEASURED_LOCAL means an owner-side
measurement is documented at the linked source; CI does not re-run it.
Deferred entries deliberately stay open.

An open row carries one of four states — SEPARATE_GATE, MEASURED_LOCAL,
NOT_VERIFIED or NOT_ESTABLISHED — and the registry rejects any other word, so a
row cannot declare itself passed. The registry covers the tools added for
v0.5.0, every timing table of the [measured progress](MEASURED-PROGRESS.md)
page, the timing records under [speed results](speed-results/2026-10-20-against-v0.6.1/README.md)
that the README chart is drawn from, the command table and status cells at the
top of the README, the memory-core parts that `xtask` can call directly, and the
limits the README and the release notes state as not established; the GEL
sketch-search and Ocean full-scan rows are author-reported and cannot be re-run
from this checkout.

## Sentences that name a row

Other pages state what a row says: the README, the goal, start-here, the crash
series, the record history, the evidence summary and the boundary sentence of
the external checks. `xtask claims` keeps them tied to the registry in two ways:

- a fixed list in the registry code (`CITED`) holds each such sentence with the
  rows it stands for and their state. The sentence must occur exactly once in
  its file (line breaks count as spaces), every row must exist, and it must
  still carry that state; rewording the sentence or moving the row fails the
  check until both are changed together;
- in every Markdown file, the lines outside a table of a paragraph that links
  this page may name in code only ids of this registry and only its states.

The command prints `CLAIMS_CITATIONS=PASS` with the number of sentences, rows
and paragraphs it checked. The release notes are not in the list: they are
replaced with each version.

This register is not yet an exhaustive map of every historic statement in the
repository. In particular it does not retroactively validate videos or timing
tables, the roadmap rows or the gel-store and gel-reader parts of the README,
which have their own tests but no row here. Preserve the actual tested commit/source manifest and run attempt using
the [CI evidence report](CI-EVIDENCE.md); a SHA is an identity, not authentication.
