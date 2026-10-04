# Multipart source readout

Released in v0.3.0. This adds `Corpus::lead_parts(title, expected_parts)`
to the public source catalog. No binary ORB format, Q8 algorithm, dependency
or license is changed. The implementation uses only the public catalog API;
it needs no private application, corpus, importer, model or sidecar.

## English

The exact-title lookup must resolve to one node. Its introduction may be
one `Lead` or `WSTĘP` section, or one family numbered `Lead (1)`, `Lead (2)`…
or `WSTĘP (1)`, `WSTĘP (2)`… . Numbers are positive canonical ASCII decimal
integers, at most50,000. Ordering follows these numbers, not catalog order,
role IDs or text offsets. The API returns separate `Passage` values and never
joins them into a fabricated source quote.

Missing/ambiguous titles, missing introductions, duplicates, numbering gaps,
mixed families and mixed single/numbered layouts return explicit errors.
Parenthesis-shaped labels in these families must use the exact spelling and
spacing above. Other section labels are ignored. Title matching follows the
existing trim/lowercase rule: no Unicode normalization, accent stripping,
full case folding or semantic inference. Returned bytes remain unchanged.

`Some(n)` requires exactly n parts, based on an independently approved
expectation. `None` validates only the available sequence starting at1:
it cannot detect an omitted last part. Computing n from the received catalog
does not establish completeness. Neither mode proves that an upstream importer
retained the complete original article or that the source is true.

Each passage retains its own address, entry, byte range, SHA-256 and catalog
generation; `Corpus::validate` rejects passages from another generation.
Approved catalog/text pins remain the caller's responsibility. Total returned
quote bytes are capped at64MiB before cloning, even with overlapping source
ranges. Index nodes, record metadata, strings and allocator costs are extra;
this is not a process RSS limit. No speedup or semantic accuracy is claimed.

## Reproduce

Fetch locked dependencies once if needed, then use offline commands:

```text
cargo test --locked --offline -p gel-source
cargo run --locked --offline -p gel-source --example source_parts
cargo run --locked --offline -p xtask -- verify
```

Expected example marker: `SOURCE_PARTS_E2E=PASS`. The example also demonstrates
rejection of modified text with original pins and a wrong expected count.
Linux verification runs this example; the CI matrix also runs it on Windows
and macOS. Configuration is not evidence those remote jobs already passed.
