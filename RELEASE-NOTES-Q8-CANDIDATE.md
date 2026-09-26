# Q8 evidence and checked-view candidate

> Historical pre-v0.3.0 notes; status and license statements describe that time.
> Current terms: GEL RAM NCRL 1.0 ([LICENSING.md](LICENSING.md)).

Revision2: reject stable special-file/symlink inputs before open, strengthen
campaign validation and repeat all measurements. See the
[R2 audit](docs/Q8-CANDIDATE-R2-AUDIT.md); no scoring formula changes.

PR #10 candidate of the earlier pre-v0.3.0 repository, based on its main 339f364
(not PR #10 of Gelram-project/gel-ram); no new release tag or version
number assigned. Existing v0.2.1 historical evidence remains unchanged.

Follow-up: validate log configuration and derive ratios from raw durations,
execute tests on all CI platforms, consolidate README, add a Rust playground,
source/tamper demo and one-command report. Two reviewed static SVGs present
the public structure and all R2 canonical-baseline cells. See
[demonstrations and limits](docs/PUBLIC-DEMO.md).

Visual follow-up: two reviewed educational illustrations in both English and
Polish, accessible English explanations, explicit measurement limits and exact image fingerprints
in the Rust release gate. Correct the host description to owner-reported
MINISFORUM AI X1 Pro / 128 GB installed, separate from measured 93.91 GiB
OS-visible memory. No historical numeric results or private application code change.

- Add explicit in-memory descriptors and checked restoration for public Q8 views.
- Reject unsupported profiles/poles and mismatched reader seeds; retain low-level APIs.
- Demonstrate the boundary: compatibility metadata is not authentication.
- Add a one-command Rust numeric sample and bounded Q8DEMO01 fixture handling.
- Add a canonical single-read baseline using the same packed record layout.
- Repeat the full Q8 V2 timing campaign and report a separate private-real-input
  numerical check, including cases where a canonical single reader is faster.
- Retain exact score checks, masks, weights, existing file formats and PolyForm license.

No private application, encoder, bank, model or conversations are shipped.
No claim of semantic99%, fourfold independent capacity or physical RAM fingerprint.
See [results and limitations](docs/Q8-EVIDENCE-CANDIDATE.md) and
[validation](docs/Q8-CANDIDATE-VALIDATION.md).
