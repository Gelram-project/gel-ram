# v0.4.0 candidate — Evidence Lab

Workspace version: **0.4.0-rc.1** (public main since PR #9). Target: v0.4.0.
Approved for public main integration after checks; not approved for a final release.

## New relative to reviewed public main

Public baseline: `44b6d7af4d1ffc5715585d2d5c5db42ee401e4f8`.

- Multi-document collection with independent IDs and exact source quotations.
- Add, replace, delete, source-bound citation checks and stale-result rejection.
- Plaintext snapshots saved without replacing existing files and fresh-process reopen.
- Offline Rust `gel-evidence` terminal, with explicit UNKNOWN/INCOMPLETE states.
- Collection workloads, raw timing evidence, independent summary recalculation
  and bounded Q1/Q2/Q4/Q8 reference checks.
- A new 70-second English terminal recording and a one-command review guide.

[Full scope](RELEASE-NOTES-EVIDENCE-LAB.md) · [Quickcheck](docs/CANDIDATE-QUICKCHECK.md).
These rc.1 additions were merged into public main through PR #9 (71142a2).

## Review repairs in PR #10

These review repairs come from PR #10. They are part of main only after that PR
is merged (see its GitHub page); they are not part of the rc.1 content of PR #9:

- Public F32/F16/affine Q1–Q16 [precision matrix](docs/PRECISION-MATRIX.md)
  (GPMX v1 example container); the private Q2.5 codec is not implemented.
- Publication tested against kernel permission denial (Unix test) and, in Linux
  CI, a physically full 1 MiB tmpfs ([fault matrix](docs/PUBLICATION-FAULT-TESTS.md)).
- Per-platform [CI evidence report](docs/CI-EVIDENCE.md) with declared platform
  exclusions, and workspace documentation tests.
- Fail-closed film recorder with a lint gate ([recorder safety](docs/RECORDER-SAFETY.md));
  gel-bench header GEL_BENCH_V5 and quad_compare header Q8_QUAD_COMPARE_V3
  ([measurement protocol](docs/MEASUREMENT-PROTOCOL.md)).
- A [claim registry](docs/CLAIMS.md) separating executable checks from
  explicitly unverified claims.

## Already published, not claimed as new

The stable v0.3.0 tag and subsequent public-main source-readout, Live Lab,
Ocean Scale and documentation updates are retained. Ocean measurements,
older films and earlier test logs keep their original revision/version labels.
This candidate does not rerun the 10M campaign merely by changing its version.

## Compatibility and verification

Rust remains pinned to 1.85.0. Version changes apply to the 13 workspace members;
third-party dependency versions and operative licensing are not intentionally changed.
This version bump itself introduces no source-bundle or Q8 wire-format change.
The collection format is a separate feature, not an implicit conversion of old banks.

Re-run `cargo run --locked --offline -p xtask -- verify` on the exact candidate.
Older Linux/Windows/macOS results cannot certify these new bytes. Native
Linux/Windows/macOS results for the PR #9 head are pinned in
[the platform record](docs/PLATFORM-REVIEW.md); every later revision needs its
own per-platform [CI evidence](docs/CI-EVIDENCE.md).
No semantic chatbot, encrypted private vault, private speaker, physical DRAM
synchronization or fourfold independent information capacity is claimed.

PUBLICATION_APPROVED=YES (public main integration)
TAG_CREATED=NO
RELEASE_CREATED=NO
