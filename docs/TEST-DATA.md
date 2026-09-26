# Test-data policy

The release contains no model weights, personal data or large external dataset.
Besides fixtures generated deterministically in Rust tests, it ships small
pinned text fixtures: an MIT-licensed Rust Book excerpt
([provenance](REAL-SOURCE-DEMO.md)), authored Evidence Lab texts and frozen
project-document copies for the post-freeze assessment.
Generated correctness fixtures include
a legacy v1 store that is verified and migrated to v2, CRC64-ECMA vectors,
header mutations, exact structural residuals and full/progressive Top-K pairs.

Large synthetic banks are generated at runtime by `gel-bench`; they are not
stored in the repository. A future semantic dataset must be distributed
separately with provenance, its own license and cryptographic hashes.
