# Source-readout and archive-report candidate

Unreleased, review only. No public tag, publication approval or license cutover.
This extends the public baseline `82a7e5d6a48109d091113661b69be0a8ef1dccdd`
and includes the staged licensing branch. Exact candidate bytes are identified
by its separately reviewed source-manifest hash, not by an invented release tag.

## Added

- `Corpus::lead_parts`: numbered introduction parts, separate exact quotations,
  per-part source ranges, hashes and generation; explicit ambiguity/gap errors.
- Synthetic PL/EN multipart examples and a small pinned MIT-licensed Rust Book
  excerpt with source provenance. No imported private corpus is needed.
- Rust source-audit/source-bundle commands with bounded inventory checking and
  an independently supplied SHA-256 manifest pin; no publication side effects.
- An [exact external dependency inventory](docs/DEPENDENCY-INVENTORY.md), including
  locked archive checksums and upstream license-file fingerprints.

## Repaired during review

- Report generation from source archives without Git: a supplied reviewed pin
  identifies and checks the source tree before and after the complete campaign.
  Missing Git metadata is explicit; broken Git is not silently ignored.
- Report completion requires successful commands and a matching status line;
  reference fallback on either output stream invalidates the report.
- Root documentation separates GEL-owned licensing from the MIT excerpt.
  The inactive NCRL proposal clarifies grants from authors of modifications,
  third-party boundaries and the need for separate commercial permissions.
- Portable CI records the tested revision/toolchain, checks the active license
  and runs the independent integrity audit in addition to tests and source demos.
  No new Windows/macOS result is implied merely by this configuration change.

## Reproduce

Use the exact reviewed candidate sources, install the pinned toolchain and fetch
locked dependencies once as documented in README, then:

```text
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p gel-source --example source_real
cargo run --locked --offline -p xtask -- report ../gel-report-new REVIEWED_MANIFEST_SHA256
```

Replace the last argument with the reviewed source-manifest hash. The report
directory must not exist. See [validation](docs/SOURCE-CANDIDATE-VALIDATION.md).

## Not included or claimed

No new Q8 scoring formula, semantic accuracy result, compression claim, private
encoder, speaker, multimedia bank, P2P service or physical DRAM synchronization.
Real text readout and numeric Q8 views remain separate demonstrations. Earlier
performance results belong to their recorded revisions and inputs, not a new
speedup of the multipart API. No claim of general security or legal certification.

Root PolyForm remains active for GEL-owned code. The proposed NCRL terms are
not active; legal identity, rights, privacy practices and legal review remain
release prerequisites. Earlier public rights and historical tags are unchanged.
