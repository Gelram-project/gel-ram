# Reviewed source snapshots

The Rust-only source snapshot commands do not publish, activate a license,
sign a release, or approve ownership. They require a SHA-256 pin of the exact
reviewed source manifest, supplied independently by the reviewer.

```text
cargo run --locked --offline -p xtask -- source-audit REVIEWED_MANIFEST_SHA256
cargo run --locked --offline -p xtask -- source-bundle REVIEWED_MANIFEST_SHA256 ../gel-review-new
```

Replace REVIEWED_MANIFEST_SHA256 with the 64 lowercase hexadecimal characters
recorded during review. The destination must not exist, its parent must exist,
and it must be outside this checkout. Nothing is uploaded.

The bundle contains a source subdirectory and an outer status file. The latter
always says PUBLICATION_APPROVED=NO. Copying an inactive licensing proposal does
not make it the operative license. Review the root license, metadata, rights,
privacy process and release tests before any publication decision.

## Checks

- Manifest uses canonical LF records, lowercase SHA-256 and portable relative
  paths; traversal, absolute paths, duplicate paths and case aliases are rejected.
- Every source file except the manifest itself must be listed exactly once;
  missing or extra files fail even if their extension looks like source.
- Root Git metadata and build output are explicitly excluded and never copied.
  Nested metadata/build paths, symlinks and special files are rejected.
- On Unix, executable permissions on source files are rejected.
- Manifest size is at most 1 MiB; each other file at most 8 MiB; total source
  payload at most 64 MiB; at most 4096 files including the manifest; directory
  nesting at most 16. These are input limits, not total process RSS guarantees.
- Bytes are read with limits, hashed and retained in memory before copying.
  The source files are not reread for the copy. The written copy is verified
  against the same manifest pin before its status file is written.
- An existing destination is never overwritten. Failed output is retained
  for inspection without a successful completion status. This is not a durable
  transactional backup protocol and does not promise survival of power loss.

## Trust limits

The pin must come from a trusted review, not from the same untrusted download
as the manifest. Computing a new pin after arbitrary edits proves no approval.
An attacker who can replace both the manifest and the trusted pin can replace
the code too. These checks neither establish code authorship nor detect all
secrets or malicious implementations; independent code and secret review remain
necessary. The tools assume a trusted local filesystem without a concurrent
hostile process replacing directories during the operation. Do not run them
with elevated privileges or treat them as a sandbox for hostile filesystem races.

The bundled sources must be built and tested separately. Historic timing logs
retain their original scope and are not new performance results of a snapshot.
The commands use the checkout embedded when xtask is compiled, not an arbitrary
source directory chosen from the caller's current working directory.
