# v0.4.0 candidate — Evidence Lab

Local workspace version: **0.4.0-rc.1**. Target: v0.4.0.
Not published, not tagged, not approved for release.

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
Older Linux/Windows/macOS results cannot certify these new bytes. Local Linux
results, pending platform checks and publication gates are recorded separately.
No semantic chatbot, encrypted private vault, private speaker, physical DRAM
synchronization or fourfold independent information capacity is claimed.

PUBLICATION_APPROVED=NO
TAG_CREATED=NO
RELEASE_CREATED=NO
