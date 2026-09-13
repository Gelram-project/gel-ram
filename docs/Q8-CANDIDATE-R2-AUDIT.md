# Q8 candidate revision 2 — adversarial input and repeatability audit

2026-09-11. Local-only follow-up to the first evidence candidate. No public
push, PR, release, private engine export or dependency change.

## Confirmed defect and correction

The fixture reader checked regular-file status only AFTER opening its input.
A named pipe with no writer blocked during open; a bounded Linux probe timed
out (exit124). The reader now rejects stable special files and symbolic links
before opening, and also retains the handle metadata check and byte limit.
The same pipe, a symlink and a directory now return exit2 promptly.

This is NOT a race-proof filesystem sandbox: a hostile process able to replace
the pathname between checks may still interfere. Use a caller-controlled input
directory. Network filesystem stalls and allocator exhaustion are not ruled out.
The fixture is numeric data with no embedded authentication; valid payload
changes are not corruption-detectable without an external trusted digest.

## Expanded verification

- 145 test executions in debug and145 in release, all successful. These include
  repeated reference harnesses, not145 separate features.
- Additional parser tests cover every truncation of a one-record fixture,
  all96 header-bit flips, maximum8192 records, all1024 activity-bit positions,
  exact phase preservation and invalid command arguments.
- Four standalone Rust summary tests cover invalid/duplicate/non-finite metrics,
  failed statuses, worker fallback and expected campaign shape.
- The summary tool now requires all expected cells, three distinct repetitions,
  nine ordered rounds, finite positive timings/ratios, correct comparison counts
  and a successful status. An empty campaign is rejected before printing a table.
- The stricter summary reproduces the first campaign table byte-for-byte.
- Full `xtask verify` and the independent data-integrity audit passed again.
  The latter checks262400 structural cases and256 ranking queries.

The standalone helper tests can be run from the repository root:

```text
rustc --edition=2021 --test docs/evidence-q8-current/summarize.rs -o /tmp/gel-summary-tests-r2
/tmp/gel-summary-tests-r2
```

These validations detect accidental incomplete results, not deliberately forged
logs or sources. Checked view descriptors still do not authenticate the sender.
No new claim is made about the private donor variants or general semantic quality.

## Complete rerun after the input fix

All72 invocations were repeated:48 V2 runs and24 canonical-baseline runs.
All8,355,840 V2 and2,509,056 canonical view comparisons passed again. No reference
worker fallback was reported. The scoring kernels were unchanged.

| Real768 records | FourViews / Shared median (range) | Single / Shared median (range) | Shared median ms/query |
|---|---:|---:|---:|
| BodyActivity |3.627 (3.338–4.478)|1.145 (0.993–1.420)|1.385|
| Archive |3.953 (3.838–4.178)|1.174 (1.130–1.630)|0.413|

These are medians of three run-level ratios. The small synthetic32 bank still
favors the single reader. The real BodyActivity range still includes a slower
Shared run. The full [R2 summary](evidence-q8-r2/summary.txt) includes every cell,
including unfavorable results; the first campaign remains intact, not replaced
by the fastest repeat. This supports numeric parity and a measured advantage
over four materialized views, not a universal single-reader speedup.

Same Linux PC and portable Rust1.85 settings as the first campaign, no affinity
or exclusive host reservation. A release-test process and a short integrity
check overlapped parts of this repeat; host load was not isolated. Do not use
small timing differences between revisions as evidence of an optimization.
The [hardware snapshot](evidence-q8-r2/hardware-after.txt) was taken AFTER the run;
it is not a pre-run snapshot or proof of constant load during measurement.

All raw numerical logs and final [source/binary fingerprints](evidence-q8-r2/MEASURED-SOURCE-SHA256SUMS.txt)
are included. Binaries and the private768-record input are not included. The
old fingerprint file belongs to the old campaign, before the R2 input fix.
Neither campaign measures four independent memories, media understanding,
physical DRAM signatures, native reasoning or semantic99% accuracy.

## Publication gate

Ready for local package review once the delivery manifest and clean-unpack
verification pass. New remote CI, maintainer review and publication approval
remain separate gates. Linux tests do not establish Windows/macOS runtime.
Secret and dependency scanner limits remain as documented in
[candidate validation](Q8-CANDIDATE-VALIDATION.md).
