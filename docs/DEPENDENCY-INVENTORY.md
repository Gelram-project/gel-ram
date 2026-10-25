# Exact external dependency inventory

Review snapshot, 2026-10-17. Cargo.lock lists **no external packages**: every
crate in it is a member of this workspace. SHA-256 (FIPS 180-4) and Unicode
Normalization Form C are implemented in gel-source itself
([source readout](SOURCE-READOUT.md), [document readout](DOCUMENT-READOUT.md)).
Earlier releases used twelve registry packages; the last two,
unicode-normalization and tinyvec, left on 2026-10-17.

This records provenance, not a vulnerability scan or legal approval. Toolchain,
OS libraries and CI actions are not Cargo dependencies and are not listed here.
CI action revisions are pinned in the workflow. The Ocean research archives that
were on main before 0.4.0 carried their own vendored dependencies and
inventories; this file describes the root workspace only.

| Package | Version | Declared license | Upstream |
|---|---|---|---|

## Archive and license-file SHA-256

None: there is no external package archive to check.

## Other redistributed third-party material

The MIT Rust Book excerpt is identified separately in [real-source provenance](REAL-SOURCE-DEMO.md), with its full license beside the fixture. It is not a Cargo dependency. GEL-owned material remains under the root license; this inventory neither changes that license nor licenses third-party material on GEL terms. Recheck the inventory whenever Cargo.lock or distributed fixtures change.

The collection-review fixture reuses that exact same excerpt with its MIT notice
copied beside it; [provenance](../crates/gel-source/fixtures/collection-review/PROVENANCE.md).
No new upstream source or Cargo dependency is introduced by this copy. Capture
tools (FFmpeg, Xvfb, xterm) are not shipped in the source package.

The normalization tables in `crates/gel-source/src/nfc_tables.rs` are derived
from the Unicode Character Database 16.0.0, and
`crates/gel-source/tests/data/NormalizationTest-16.0.0.txt`, the two files in
`crates/gel-source/tests/data/ucd-16.0.0`, and, of the version the tables
followed before (kept for tests),
`crates/gel-source/tests/data/NormalizationTest-15.1.0.txt` and the two files in
`crates/gel-source/tests/data/ucd-15.1.0` are unmodified Unicode data files, all
under the Unicode License v3 (`crates/gel-source/LICENSE-UNICODE.txt`);
see [third-party notices](../THIRD-PARTY-NOTICES.md).
