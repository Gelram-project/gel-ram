# Multipart readout — candidate validation

Historical scope of the initial multipart addition. The later real-source
example and archive-report repairs are tracked in the
[later candidate validation (also historical)](SOURCE-CANDIDATE-VALIDATION.md); the counts below
are not the current total number of workspace or example tests.

2026-09-12. Local Linux, Rust1.85.0. Unreleased candidate based on public
commit `82a7e5d6a48109d091113661b69be0a8ef1dccdd`.

-23 source tests pass: 11 new multipart tests and 12 existing catalog tests.
-Full workspace `xtask verify` passes, including formatting, Clippy, tests
  and runtime demos. Multipart demo ends with `SOURCE_PARTS_E2E=PASS`.
-Tests include PL/EN Unicode, reordered roles/offsets, duplicates, mixed
  layouts, missing/ambiguous titles, malformed/overflow numbers, missing
  initial/internal/final parts, changed pins and cross-generation passages.
-Overlapping catalog ranges cannot expand the returned quote bytes beyond
  the 64 MiB output budget. This is not a total process memory bound.
-No dependencies added; Cargo.lock unchanged. Known-advisory audit reports
  no vulnerabilities or warnings against database commit
  `b50980aad8b8f14f77e25a97b32dd94bf008b0af` (2026-09-09).
  This is not proof that dependencies contain no defects or malicious code.

All examples are author-created synthetic text. No private corpus, speaker,
media, personal data or algorithm is needed. This implementation adds only
selection and validation over the existing public catalog, not an importer.

Windows/macOS CI is configured to execute the tests and the new demo, but
remote execution of this candidate is pending publication approval. Local
Linux success must not be presented as three-platform certification.

No new speedup, compression ratio, independent storage capacity, semantic
accuracy or physical RAM synchronization result is claimed. Existing timing
tables describe their original revisions, not a new measurement of this API.
