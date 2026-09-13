# GEL RAM v0.3.0 — public Rust core, NCRL 1.0 and native demo media

Date: 2026-09-13

GEL RAM v0.3.0 marks the current public generation after the controlled publication cutover in PR #16. The version bump is intentionally a minor-version change rather than a patch because the public package now includes materially expanded source-readout functionality, demonstration media, stronger cross-platform verification and a different operative licensing model.

## What defines v0.3.0

1. The public Rust workspace remains source-only and `publish = false` for crates.io.
2. The operative root license is **GEL RAM Noncommercial Reciprocal License 1.0**.
3. Noncommercial use, modification and sharing are permitted under the public license; private noncommercial modifications may remain private without a fixed time limit.
4. Evaluation is available to any individual or organization, including companies, without a fixed evaluation deadline while the use remains non-production and non-monetized.
5. Monetized Use by any individual, freelancer, sole trader, nonprofit, company or other organization requires a separate signed Commercial Agreement before monetization begins.
6. Commercial agreements may use reciprocal terms or separately negotiated closed-modification rights.
7. The public package includes the reviewed Rust source-readout path, integrity/reproduction tooling, two native terminal demonstration films, screenshots and explicit media reuse terms.
8. The complete tracked-source SHA-256 manifest is verified in Linux CI; Linux also runs `xtask verify` and the independent byte/numeric/ranking audit. Windows and macOS execute workspace tests, licensing checks, source demos and the independent audit.

## Evidence boundary

The demonstration films show bounded native Rust source-frame behaviour and measured local timings on the documented setup. They are not token-per-second measurements and do not establish general LLM inference speed, unrestricted language generation, universal semantic accuracy, P2P operation or production readiness.

Historical measurements and files labelled v0.2.x remain historical evidence. A later version number does not retroactively change their measured scope.

## Licensing continuity

The NCRL 1.0 applies to GEL RAM-owned material in this distribution as stated by the root `LICENSE` and `LICENSING.md`. Third-party material remains under its own terms.

Earlier releases and copies already distributed retain the rights validly granted with those releases. v0.3.0 does not revoke or rewrite historical grants.

## Release verification

The publication tree introduced through PR #16 passed pre-merge and post-merge Linux, Windows and macOS gates. The v0.3.0 version-metadata change must pass the same current CI gates before it is accepted into `main`.

`LEGAL_APPROVED=NO` remains an explicit statement that no independent legal-counsel certification is claimed for the custom license.
