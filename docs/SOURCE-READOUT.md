# Source-bound readout (released in v0.3.0)

The gel-source crate reads an immutable UTF-8 catalog and returns an exact
source passage for a node/role address. It checks independent SHA-256 pins,
fragment hashes, byte ranges, UTF-8 boundaries, duplicate addresses and source
generation. Exact title lookup returns all matching node IDs: callers must
not silently pick one when a title is ambiguous.

This is NOT a language model, a semantic search engine, a truth verifier, or
proof that an ORB encodes the claimed text. Pins must come from a trusted
manifest, not from the same untrusted response as the data. Hashes alone are
not signatures. No network, private conversation, media or speaker is included.

Catalog wire version GELCORPUS082 is preserved for compatibility:
header followed by eight tab-separated fields per row: node, role, entry,
byte start, byte length, SHA-256, hex UTF-8 title, hex UTF-8 section.
The text is stored separately. Each file ends as specified by the parser;
catalog rows are newline-terminated. Labels cannot contain control characters.

Limits: 64 MiB text, 16 MiB catalog, 50,000 records, 32,768 bytes per passage.
These are input limits, not a promise about total process RSS.
Title normalization trims surrounding whitespace and lowercases Unicode;
it does not perform full Unicode case folding, accent removal or semantic matching.

## Reproduce

Since PR #10 of the earlier pre-v0.3.0 repository (not PR #10 of
Gelram-project/gel-ram), the source_readout example prints title, quote,
source entry/section/byte range and
pins, then demonstrates rejection of modified synthetic text with unchanged
pins. An optional exact title argument selects the fixture; unknown titles
return UNKNOWN. See [public demos](PUBLIC-DEMO.md). This is not semantic search.

Run cargo fetch --locked once on a clean machine, then:

    cargo test --locked --offline -p gel-source
    cargo run --locked --offline -p gel-source --example source_readout
    cargo run --locked --offline -p xtask -- verify

The SHA-256 implementation is the pinned RustCrypto sha2 dependency, not a
new home-made hash. Its transitive dependencies are recorded in Cargo.lock.
CI fetches those locked dependencies before the existing offline checks.

## Provenance and release boundary

This is a narrow extraction of the project owner's private source-catalog
module and its regression tests. The adapter replaces the private hash-provider
dependency with sha2 and bounds TSV splitting to nine fields. The catalog and
passage contracts are otherwise preserved. GEL RAM-owned material is under
GEL RAM NCRL 1.0 ([licensing](../LICENSING.md)); the extraction was first
prepared under the then-active PolyForm licensing, and no private master
licensing files are changed.

Only synthetic test text accompanies this readout. No Wikipedia corpus,
machine-specific paths, models, TLS credentials, private reports, donor monolith
or the private application is included. Released in v0.3.0; platform results
belong to the CI of each exact revision.

## Security boundaries

See the [pre-publication review](SOURCE-REVIEW.md) for the scoped checks and
their limitations.

Passages can contain arbitrary UTF-8, including control characters, HTML or
instructions. They are returned as data, never evaluated. An application must
escape them for its terminal/browser and must not grant authority to their text.
The example computes hashes only for its own trusted synthetic fixture: real
applications need independently approved pins.

Many nodes may share a title. The loader appends each node to its title index
only once, without repeatedly scanning the growing list of matching IDs.
This preserves ambiguity and first-seen order without quadratic deduplication.
Catalog limits bound inputs, not total memory or every caller's processing cost.
The workspace's unsafe-code prohibition does not extend into dependencies.
