# Bounded document readout

The public API `gel_source::document::search(text, phrase)` returns ranges in
the exact input text. It searches a sequence of alphanumeric tokens within
one line; punctuation/whitespace delimit tokens. NFC and lowercase matching
do not rewrite source bytes. Final/non-final Greek sigma are normalized alike;
this is not full Unicode case folding (for example, no `ß` → `ss` expansion).
It is neither byte-substring matching nor semantic
question answering. Negation and role order remain in the returned quote;
conflicting source lines are not resolved into a single true statement.

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

## Unicode normalization

NFC is implemented in gel-source itself (`crates/gel-source/src/nfc.rs`,
UAX #15: canonical decomposition, canonical ordering, canonical composition),
with tables generated from the Unicode Character Database 15.1.0 by
`cargo run -p xtask -- ucd-tables`. The two UCD files it reads are kept
unmodified in `crates/gel-source/tests/data/ucd-15.1.0`, and a test regenerates
the tables from them and requires the same bytes.

**Versions.** Normalization follows Unicode 15.1.0; lowercasing is the Rust
standard library's `to_lowercase`, which follows Unicode 16.0.0 on the Rust
1.85 toolchain the releases are built with. Search normalizes the query and
each line when it runs and stores nothing normalized, so a later table version
changes only which lines match, never stored data.

**Checks.**

- All 19,074 data lines of NormalizationTest 15.1.0 pass, and every scalar
  value it does not list is unchanged.
- NFC of every scalar value, and of 100,000 seeded strings built from the
  characters that take part in normalization, is byte-identical to the output
  of unicode-normalization 0.1.25 (the crate used up to v0.6.1, recorded before
  it left the tree) and of ICU4X icu_normalizer 2.3.0.
- NFC of 10,005 adversarial strings (long runs of combining marks, Hangul, and
  strings two thirds of whose characters are combining marks) is byte-identical
  to ICU4X icu_normalizer 2.3.0. This is the only check that catches an
  unstable sort of combining marks.
- NFC is idempotent on every string above, and a line break splits it:
  NFC(a + "\n" + b) = NFC(a) + "\n" + NFC(b), so normalizing line by line is
  the same as normalizing the text.

**Difference from v0.6.1.** unicode-normalization 0.1.25 followed Unicode 17.0.
On NormalizationTest 17.0.0 this implementation differs on 162 of 20,034 lines,
and each of them contains a character assigned after Unicode 15.1; it agrees on
every line made only of characters assigned in 15.1, as the Unicode stability
policy requires. Matching therefore differs from v0.6.1 only on text containing
characters assigned in Unicode 16.0 or 17.0.

**Cost.** Text that passes the NFC_Quick_Check rule (most text, all Polish text
among it) is returned without normalization. Otherwise each character's
combining class is looked up once and each run of combining marks is sorted
with a stable n log n sort; a test counts the lookups on the worst 4095-byte
line. Latency on the author's machine:

```sh
cargo run --release --locked --offline -p gel-source --example nfc_bench -- 200
```

Against v0.6.1, five rounds alternating with it in one session while other work
loaded the machine, p50 per line divided by v0.6.1's p50 in the same round
(lowest–highest of the five rounds; below 1 is faster):

| Line | This change / v0.6.1 |
|---|---|
| Polish text | 0.36–0.88 |
| Hangul syllables | 0.87–1.82 |
| 2047 combining marks in reverse class order | 0.93–2.14 |
| 2047 combining marks in canonical order | 1.40–1.86 |
| Decomposed Hangul jamo | 1.41–1.95 |

Only Polish text is faster in every round, and only canonically ordered marks and
decomposed jamo are slower in every round; the other two lines are within the
noise of this machine. A document of 4096 lines of reversed marks took
0.54–0.63 s (v0.6.1: 0.39–0.57 s); an earlier draft of this change, which sorted
marks by insertion, took 53 s, and the lookup-count test now rejects it.
