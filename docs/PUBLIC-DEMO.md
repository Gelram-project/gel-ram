# Public demonstrations and reporting correction

2026-09-11, follow-up to R2. No scoring-kernel or license change; the private research system
remains outside this export. Current README replaces the accumulated openings;
the previous version is preserved verbatim in [README history](../README-HISTORY-R2.md).

## Reporting fixes

The summary reader previously trusted filenames for cell assignment and copied
reported ratios. It now compares count, policy, sparsity and requested workers
with metadata where those fields exist, checks the V2 seed, rejects duplicate
log contents, sums the nine raw timed rounds and derives each speed ratio.
It checks reported ratios at their six-decimal serialization precision and uses
unrounded derived values for aggregation. Both complete R1/R2 summary tables
recompute unchanged. No original measurement log has been modified.

Existing logs do not contain a cryptographic dataset identity or repetition ID.
Checking names/metadata does not authenticate provenance, distinguish two different
same-sized external banks, or detect swapping two repetitions with otherwise valid
contents. Deliberately forged logs remain outside this consistency checker.

Regression tests are included in the gel-phase-quad Cargo test harness, including
swapped counts/policies/sparsity/workers, false ratios and missing rounds.
Windows/macOS CI now executes workspace tests, not --no-run compilation alone.

Local verification passed 160 test executions in both debug and release
(including repeated reference harnesses). The report command completed full
verify and36 scan invocations on the development PC. An interactive sequence
phase128 → mask3 → noise100 → show → quit also passed reference/inverse checks.
These are functional checks, not semantic accuracy. Final remote CI is tracked in PR #10.

## Interactive Q8

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_playground -- --interactive
```

Commands: phase0..255 shifts body phase; mask0..1024 is the active-dimension
stride (0 disables all body dimensions); noise0..1024 perturbs that many leading
dimensions deterministically. Use spaces, for example `noise 100`.
The query stays fully active; empty body scores zero, not UNKNOWN. No active
query would be UNKNOWN in the library. Noise is a repeatable perturbation, not
a measured physical noise process. Input lines are capped at1024 bytes.

Each view is materialized and scored independently for comparison with Shared.
Inverse phase and mask equality are checked. Timing is one shared read,
excluding query preparation and verification; it is NOT a stable benchmark.
Payload sizes and Linux process RSS are labelled separately. Other platforms
report RSS as unmeasured; no invented memory or timing values.

## Source E2E

```text
cargo run --locked --offline --release -p gel-source --example source_readout -- "Demo vessel"
```

Built-in synthetic text includes Unicode. The example prints title, quotation,
entry, section, byte range and pins; a2→9 modification is rejected with the
original pins. Unknown titles return an explicit error. This does not prove
the synthetic pressure is a fact, nor link the text to a privately encoded Q8 bank.

## One-command report

```text
cargo run --locked --offline -p xtask -- report ../gel-report-new
```

The new output directory must be outside the checkout; an existing directory
is never reused. Logs capture the revision, dirty-tree status, toolchain,
available threads, verification and scan results. Review local paths before
sharing. No data is uploaded. Linux CPU/RAM readings are snapshots, not sustained
utilization; other OSs need manually supplied model/system RAM information.

The bounded synthetic scan campaign uses32/512/8192 records, both policies,
1 and min(available threads,24) workers (deduplicated), three repetitions,
nine timed rounds and rotating per-method order. All logs are retained,
including slower outcomes. This is a new portable reproduction profile, not
the historical sparse/dense campaign or the private real-input experiment.
Failure or reference fallback stops the runner; COMPLETE is written only after
all configured commands pass. Partial files are retained for diagnosis.

## Reports from source archives

Archive follow-up: when running the report from source without `.git`, supply
`REVIEWED_MANIFEST_SHA256` after the destination argument. It must be the exact
SHA-256 of the reviewed source manifest obtained from an independent trusted
review. The complete inventory is checked before and after the campaign.
Git revision/status are explicitly unavailable, not fabricated. Invalid Git
metadata remains an error; it is not silently treated as archive mode.
An archive with missing, changed or extra source files is rejected.
All commands must exit successfully and emit their expected status line;
fallback notices on either stdout or stderr invalidate the run.

## Current images

The two SVGs under docs/images are static, code-native documentation assets,
not AI-generated measurements or screenshots. The architecture image depicts
two separate public demos. The result chart contains all eight canonical R2
cells from [the raw-derived summary](evidence-q8-r2/summary.txt), including
slowdowns. Bars start at zero; panel scales differ and are labelled. Every
range remains in the text table; the real active single-reader range also
appears on the image. These observations are not confidence intervals.

These two exact SVG paths are allowed by the release gate; no broad
binary/image extension exception was added. No scripts, external resources,
private screenshots, credentials or original bank data are embedded in those SVGs.
Historical visuals remain in the preserved documentation.

Two additional [educational illustrations, each in English and Polish](ILLUSTRATED-GUIDE.md),
stored as four PNG files,
are permitted only at exact paths and with pinned SHA-256 bytes. They are
AI-assisted concept diagrams, not measurements. A Rust regression rejects
changed image bytes and an otherwise identical image at an unapproved path
for every language variant.
The gate reuses the existing workspace gel-source digest; no new third-party
dependency or image decoder is added. PNG provenance metadata is retained.
The image pins are a release-content check, not a claim of decoder security.
