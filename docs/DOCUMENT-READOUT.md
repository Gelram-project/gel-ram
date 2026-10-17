# Bounded document readout

The public API `gel_source::document::search(text, phrase)` returns ranges in
the exact input text. It searches a sequence of alphanumeric tokens within
one line; punctuation/whitespace delimit tokens. NFC and lowercase matching
do not rewrite source bytes. Final/non-final Greek sigma are normalized alike;
this is not full Unicode case folding (for example, no `ß` → `ss` expansion).
It is neither byte-substring matching nor semantic
question answering. Negation and role order remain in the returned quote;
conflicting source lines are not resolved into a single true statement.

NFC is implemented in gel-source itself (`crates/gel-source/src/nfc.rs`,
UAX #15: canonical decomposition, canonical ordering, canonical composition),
with tables generated from the Unicode Character Database 15.1.0 by
`cargo run -p xtask -- ucd-tables`. Its tests run all 19,074 data lines of the
Unicode NormalizationTest 15.1.0 and check that every other scalar value is
unchanged, and two pins recorded before the earlier crate
(unicode-normalization 0.1.25) left the tree: NFC of every scalar value, and
of 100,000 seeded strings built from the characters that take part in
normalization, are byte-identical to that crate's output. That crate followed
a later Unicode version, so matching can differ from v0.6.1 only on text that
contains characters assigned in Unicode 16.0 or later. Lowercasing is the
Rust standard library's `to_lowercase` and is unchanged.

Limits: 16 MiB text, 512 query bytes, 1–32 query words, 4096 bytes per line,
four returned lines in source order. All matching eligible lines are counted.
Overlong lines are counted as skipped. No match with skipped lines is an
incomplete search, not an exhaustive absence claim. Empty/punctuation-only or
control-containing queries are rejected. Phrases do not cross LF, CRLF or
standalone CR line breaks. Original quotation byte offsets are preserved.

## Reproduce

Requires the pinned Rust 1.85.0 toolchain and dependencies already available
(initial `cargo fetch --locked` needs network on a clean installation).

```text
cargo test --locked --offline -p gel-source --all-targets
cargo run --locked --offline -p gel-source --example source_find -- "garbage collection"
cargo run --locked --offline -p gel-source --example source_find -- "ownership"
cargo run --locked --offline -p gel-source --example source_find -- "a nonexistent phrase"
cargo run --locked --offline -p xtask -- verify
```

To collect an optional synthetic text-kernel probe (not end-to-end storage),
write the output outside the snapshot:

```text
cargo run --release --locked --offline -p gel-source --example document_bench -- 1000 > ../document-latency.csv
cargo run --locked --offline -p gel-source --example document_stats -- ../document-latency.csv
```

The deterministic 16097-byte fixture and its SHA256 are reported on stderr.
Every timed search is written to CSV; four known query classes repeat. Search
excludes hashing, printing, generation, source verification and encryption.
No claim about semantic recall, high percentiles or general dialogue follows.
The statistics reader validates IDs, class order, expected counts and positive
durations without invoking the search function. It recomputes per-class N,
nearest-rank p50/p95/p99 and mean. It cannot prove a supplied CSV is authentic.

The demo uses the existing MIT Rust Book excerpt and independent compiled
catalog/text pins. [Provenance and license](REAL-SOURCE-DEMO.md).
It verifies the source catalog before searching, then rejects a modified copy.
Its final `SOURCE_FIND_E2E=PASS` means the demo and rejection check completed,
not that every query was found. UNKNOWN is a valid result.

`search_ns` times only search. `verified_search_ns` includes fixture integrity
and catalog validation plus search. Neither includes process startup, terminal
printing or the subsequent corruption test. A single run is not a performance
distribution. Do not compare these numbers with LLM tokens/s or Ocean ORB/s.

## Integration and safety

The raw search API does not authenticate text. Applications must bind the
returned ranges to the same immutable source generation, validate their trusted
pins and escape display controls. A hash delivered by the same untrusted party
as the bytes is not an independent authenticity proof. The CLI renderer escapes
terminal controls and directional formatting; rendering is distinct from the
original source byte ranges. No quoted command is executed.

Tests cover Unicode composition/Polish accents, matches beyond previews,
negation/order, whole tokens, result/line/query limits, conflicting quotes,
differential comparison, actual pinned-source spans and mutation rejection.
These are implementation regressions, not independent semantic accuracy labels.
The caller still owns authorization, persistence and secret management.
