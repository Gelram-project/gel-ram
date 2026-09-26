# GEL RAM v0.4.0 — Evidence Lab and review repairs

Date: 2026-09-26

v0.4.0 is the first tagged release after v0.3.0. It collects everything merged
into public `main` since then: document readout, Live Lab, the Evidence Lab
(first merged as 0.4.0-rc.1) and the repairs made after an external 24-point
review of that candidate. The version line changes from 0.4.0-rc.1 to 0.4.0;
this change alone alters no source-bundle, collection or Q8 wire format.

## Load, cite, reject, reopen

- **Evidence Lab** ([guide](docs/EVIDENCE-LAB.md), PR #9): a multi-document
  collection with independent IDs, exact source-bound quotations, add, replace
  and delete, rejection of stale citations, plaintext GELSET01 snapshots saved
  without replacing existing files, and reopening in a fresh process. The
  offline `gel-evidence` terminal reports UNKNOWN and INCOMPLETE explicitly.
- **Live Lab** ([guide](docs/LIVE-LAB.md), PR #5): import your own UTF-8 text,
  search it, save a pinned GELSRC01 bundle and reopen it.
- **Document readout** ([guide](docs/DOCUMENT-READOUT.md), PR #4): exact
  quotations with byte ranges, Unicode and CR regressions, and modification
  rejection.

## Review repairs (PR #10)

- Source-bound quote context with explicit omission flags.
- Streaming collection-root hashing and staged collection construction.
- Publication tested against structural corruption, admission failures,
  kernel permission denial, SIGKILL and a physically full bounded tmpfs
  ([fault matrix](docs/PUBLICATION-FAULT-TESTS.md)).
- A fail-closed film recorder with a lint gate that proves it can fail
  ([recorder safety](docs/RECORDER-SAFETY.md)).
- Native exit-status checks for each runtime example, including misleading
  PASS output.
- Historical measurement sources preserved and R1 recomputed from them
  ([measured sources](docs/evidence-collection/README.md)).
- Per-platform CI evidence with declared Unix-only tests, a strict
  cross-platform diff and reports kept in the repository
  ([CI evidence](docs/CI-EVIDENCE.md), [recorded reports](docs/evidence-ci/README.md)).
- A public F32/F16/affine Q1–Q16 [precision reference](docs/PRECISION-MATRIX.md)
  with a versioned example container and independent oracles.
- An executable [claim registry](docs/CLAIMS.md) that keeps checked, separately
  gated, locally measured and unverified claims apart.
- Allocations, user-space copies and peak additional heap per collection
  mutation, measured locally ([mutation comparison](docs/MUTATION-COMPARISON.md)).
- A [measurement protocol](docs/MEASUREMENT-PROTOCOL.md), PL/EN film
  [transcripts](media/TRANSCRIPTS-PL-EN.md) and the A01–A24 review work list in
  the [roadmap](docs/ROADMAP.md).

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.4.0
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. `SOURCE-SHA256SUMS.txt` lists every
tracked file of the tagged tree. The release asset
GEL-RAM-v0.4.0-SOURCE.zip is `git archive` of the tagged commit; its SHA-256 is
published next to it. Then follow [TRY-IT](docs/TRY-IT.md).

## Still open

A human start-to-finish review of the films, an independent second-host
reproduction, a paired end-to-end timing comparison, isolated RSS measurements
and public execution-identity evidence remain open. See the
[roadmap work list](docs/ROADMAP.md) and the [claim registry](docs/CLAIMS.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, private
engine or bank, encrypted private vault, physical DRAM-refresh computation,
fourfold independent capacity from coordinate views or commercial superiority
is claimed. Earlier films and measurements keep their original revision labels.

## Licensing and compatibility

Licensing is unchanged: GEL RAM Noncommercial Reciprocal License 1.0 for GEL
RAM-owned material, the commercial path and CLA 2.0 ([LICENSING](LICENSING.md)).
Rust remains pinned to 1.85.0; third-party dependency versions are unchanged.
The v0.3.0 tag is not moved.
