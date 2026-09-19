# Document readout — public core update

Base: public commit `d0c9b67110e43bb9e72c463a55f1c084175bc3df`.
The owner authorized this reviewed public-core addition on 2026-09-19.
No version bump or new release tag: the immutable v0.3.0 release is unchanged.
Platform checks must pass for the exact PR revision before merging.

## New since that base

- Public `gel_source::document::search`: bounded token-phrase search returning
  original UTF-8 line ranges; NFC/case normalization preserves quoted bytes.
- Explicit total matches, four returned passages and skipped-overlong-line count.
- Pinned real-source CLI example: source gate → phrase → quote → tamper rejection.
- Unit/differential tests, exact-byte/terminal-control regression tests and
  inclusion of the example in the local verifier and portable CI configuration.
- Locked Unicode dependencies, updated dependency inventory and package manifest.
- Reproducible text probe, raw 1000-operation CSV and independent Rust statistics
  checker; [validation and limits](docs/PACKAGE-VALIDATION.md).
- Technical verification can run on an unapproved candidate. Approval flags are
  parsed independently; successful tests never approve publication by themselves.

## Unchanged

Numeric Q8 representation, Ocean research archives, published videos, root
license, CLA and commercial terms. Previous benchmark figures remain evidence
for their previous snapshots, not measurements of this new document API.

## Not included or claimed

No private speaker, application, vault, encoder, P2P internals, conversation or
knowledge bank. No LLM, paid service, unrestricted dialogue or semantic recall
claim. Exact-revision Windows/macOS results are recorded by
[CI](https://github.com/Gelram-project/gel-ram/actions), not inferred from older
releases. The two films still preview the separate private app.
