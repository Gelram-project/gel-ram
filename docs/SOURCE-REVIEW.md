# Source-readout pre-publication review (historical, 2026-09-11)

> Historical record from the PolyForm era, before v0.3.0. The module has since
> been released under GEL RAM NCRL 1.0; see [licensing](../LICENSING.md).

2026-09-11. Scope: the proposed gel-source module, its example/tests, locked
dependency additions and associated documentation/CI changes. This is an
engineering review, not a guarantee that all vulnerabilities are absent.

## Checks

- Read the entire new library, example and tests. No networking, filesystem
  access, subprocess execution, dynamic loading or embedded external payload
  is introduced by the module. Source passages remain data.
- The workspace denies unsafe code. Upstream cryptographic/CPU-detection
  dependencies have their own implementations; the prohibition is not transitive.
- Ten registry packages are pinned in Cargo.lock. Their cached archive SHA-256
  values match the lockfile; cached source trees match fresh archive extractions
  apart from Cargo's local .cargo-ok marker. These checks detect mismatches,
  not malicious upstream intent.
- Reviewed the dependency build scripts. generic-array/version_check probes
  the compiler version; libc contains platform/toolchain version probes.
  These are build-time processes, not runtime capabilities of gel-source.
- cargo-audit reported no known vulnerabilities or warnings in the lockfile.
  Advisory database commit: b50980aad8b8f14f77e25a97b32dd94bf008b0af
  (database timestamp 2026-09-09). This covers known advisories only.
- Gitleaks reported no detected secrets in the candidate source tree.
  Pattern-based scanning is not proof that every possible secret is absent.

## Change made during review

Avoided repeated linear membership searches when many distinct nodes share a
title. Each node is now appended once on its first occurrence. Repeated roles
retain source consistency checks, ambiguous titles retain all IDs, and
first-seen order is preserved. No accuracy test or integrity check was weakened.

The module has 12 regression tests, including 20,000 records sharing a title,
the 50,000-record boundary, malformed UTF-8/ranges, mismatched trusted pins,
duplicate addresses, stale source generations, control characters in labels,
and returning source content without interpreting it.

Full local workspace verification passes on Rust 1.85/Linux. The pull request's
CI checks separately report Linux verification and macOS/Windows compilation.
This document does not claim those remote jobs have passed before they run.

## Publication boundary

No private application, media bank, model weights, conversation, TLS keys or
private integration report accompanies this change. The existing workspace
PolyForm Noncommercial license is unchanged. This is not a tagged release.

Consumers still need trusted catalog/text pins and safe rendering of passages.
Hash verification establishes correspondence to approved bytes, not truth,
semantic relevance or authenticity of a catalog's claims about ORB addresses.
