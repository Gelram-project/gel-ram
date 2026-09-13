# Real-source offline readout

This small reproducible example reads three actual paragraphs about ownership
from *The Rust Programming Language*. It is an excerpt, not a complete chapter.
It uses the public `gel-source` catalog and multipart reader. No private Ocean,
printer, Q8 encoder, model, browser, network call or downloaded executable is used.

After the documented one-time toolchain/dependency setup:

```text
cargo run --locked --offline -p gel-source --example source_real -- --list
cargo run --locked --offline -p gel-source --example source_real -- "Rust ownership"
cargo test --locked --offline -p gel-source --example source_real
```

The default invocation without a title runs the same complete demonstration.
Output includes each part, exact quote, catalog address, excerpt and upstream byte
ranges (zero-based, end exclusive), SHA-256 and catalog generation. The upstream
URL is printed as provenance; the program does not fetch it. Catalog rows are
deliberately shuffled: the reader must return paragraph 1, then 2, then 3.
Title selection is exact after the reader's case/whitespace handling, not
semantic search. An unknown title fails instead of displaying unrelated text.

The demo also changes `Rust` to `GEL!` with unchanged length, changes the catalog,
and requests the wrong expected part count. All must be rejected. The success
marker is `SOURCE_REAL_E2E=PASS`. Tests additionally mutate each individual byte
in turn, truncate the text and inject an invalid UTF-8 byte. These test integrity
against the original approval; they do not prove resistance to every attack.

## Reproducible provenance and rights

- Upstream: [rust-lang/book](https://github.com/rust-lang/book).
- Pinned revision: `1500248d8f230566e4ec9f27fcbb8fe9e2898ab1`.
- Original: [ownership chapter at that revision](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch04-01-what-is-ownership.md).
- Whole upstream chapter SHA-256: `873724c6862ad0cc447becf0e818eb39a324c5d4bfa26ef721286aae1941c0ba`.
- The distributed excerpt is exactly upstream bytes **23..1180**, 1157 bytes.
  Line endings, punctuation and Markdown emphasis are preserved, not translated.
- Excerpt SHA-256: `5284e31747fcb796ef577c64343c36d1627ac44a16b683a4dc4e81fa520dec71`.
- Generated catalog SHA-256: `f5e81e25d4f6f5c4ae1f01c7516ed8313a46c2821bf33731cb540ccd06d5a3a8`.
- Copyright (c) 2010 The Rust Project Developers. The excerpt is distributed
  under the upstream [MIT license](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/LICENSE-MIT),
  reproduced in [the fixture license](../crates/gel-source/fixtures/rust-book/LICENSE.txt).
  Those third-party rights are not replaced by the GEL license or staged terms.

The fixture [text](../crates/gel-source/fixtures/rust-book/ownership.txt) and
[catalog](../crates/gel-source/fixtures/rust-book/catalog.txt) are public demo
data. The catalog title and addresses are GEL demo metadata, not upstream IDs
or physical RAM addresses. Paragraph excerpt ranges are 0..615, 617..909 and
911..1157; adding 23 gives their upstream ranges. Blank separators remain in
the text carrier. The final paragraph includes its terminating newline.

## Trust boundary and limits

Both expected pins and the expected count of three parts are separately fixed
in the reviewed example code. They are not accepted from an imported document.
Changing both the fixture and its compiled approval means changing the trusted
program: hashes cannot protect against an attacker controlling that program.
The source-bundle manifest provides a separate snapshot review boundary.

This demonstrates exact approved-source readout, **not** factual truth checking,
semantic accuracy, autonomous reasoning, compression, quantization quality or
fourfold independent information capacity. The example does not claim to prove
that source bytes were encoded into Q8 ORBs. Numeric Q8 demos remain separate.
There are no speedup or tokens-per-second claims from this tiny fixture.
Use the existing reporting protocol for separately labelled numeric benchmarks.
