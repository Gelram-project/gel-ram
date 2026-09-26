# Local Linux evidence — Evidence Lab

2026-09-19 measurement record. At that time this was a candidate without
publication approval; publication and native platform CI followed (see the
[publication status](../../CANDIDATE-STATUS.md)). These are real program
outputs, not screenshots with invented timings. The existing two films concern
the previously published application, not this new multi-document terminal.

## Checked implementation

- Rust 1.85.0, Linux x86_64, AMD Ryzen AI 9 HX 370, 24 logical CPUs available.
- The collection experiment uses **one worker**, CPU only. The OS reports about
  93.9 GiB total usable RAM; this is not a measurement of installed DIMM capacity.
- Execution with a separate network namespace (`bwrap --unshare-net`), read-only
  source and writable build/test directories. No LLM or remote service.
- Full workspace verification: **304 passed, zero failed**. Includes 14 collection
  tests, 7 terminal/process tests, 3 independent reducer tests and 7 Q reference
  tests and the scoped evidence allow-list regression. The retained log replaces
  the checkout path with `<CHECKOUT>` only.
- Windows/macOS: **not executed for this 2026-09-19 measurement**; later native
  CI is recorded in [PLATFORM-REVIEW.md](../PLATFORM-REVIEW.md).
- Process termination tests check acknowledged reopen and old-or-complete-new
  publication. They do not simulate controller failure or sudden power loss.

[Verification log](verify-linux.txt) · [Rust version](rustc.txt) ·
[Actual terminal output](demo.txt) · [Q matrix output](quantization-matrix.txt).

## All three collection passes retained

R1 comprises **2115 observations**: 8/64/256 documents, three passes (middle
pass in reverse size order), four query classes and seven lifecycle operations.
All synthetic oracle checks passed. This is exact phrase/source behavior, not
semantic intelligence or a 1M/10M Ocean result.

| Documents | Text bytes | EXACT p50 across three passes (ms) | Worst EXACT sample (ms) |
|---|---:|---:|---:|
| 8 | 57,912 | 0.091272–0.200446 | 0.238237 |
| 64 | 463,458 | 0.724892–1.057979 | 1.262614 |
| 256 | 1,854,390 | 2.924498–2.953141 | 7.041377 |

The first pass is visibly slower at smaller sizes. All observations remain in
the CSV; no explanation such as cache or scheduling is asserted without a trace.
This linear scan scales with text volume. No speedup against another product is
claimed. Per-group N is 50 for queries and 5 for lifecycle operations: p99 is
the maximum, not a statistically established extreme-tail guarantee.

The maximum recorded process VmHWM in R1 is 12,432 kB, cumulative across cases;
it is not bytes per ORB or an isolated per-bank peak. Save includes filesystem
sync. Search includes allocation of the returned quotes, not terminal rendering.

[Raw observations](r1/raw.csv) · [Recomputed table](r1/summary.csv) ·
[Corpus roots and construction times](r1/corpora.txt) · [Environment/RSS](r1/environment.txt).

```sh
cargo run --locked --offline --release -p gel-source --example collection_recheck -- docs/evidence-collection/r1
cargo run --locked --offline -p xtask -- verify
```

The historical Cargo.lock is preserved as [Cargo.lock.measured.txt](Cargo.lock.measured.txt),
recovered byte-for-byte from local integration revision
`5d01398dab8c8e4354e46c3549a9212499a89378` (before the version bump).
That integration revision is provenance metadata, not a promise that the commit
exists in public Git history. The preserved file is available in this checkout.
Its SHA256 is `fdbd6a9fd24483aa0b95012aff6e9edabf7b3c7da4fce43019383c474afab7bf`.
The measurement verifier maps the historical manifest's Cargo.lock entry
to this file, and its collection.rs entry to the unchanged
[collection.measured.rs.txt](collection.measured.rs.txt) snapshot. The current
publisher's measured source is likewise preserved as
[bundle.measured.rs.txt](bundle.measured.rs.txt); its manifest entry is checked
against that snapshot after adding the testable I/O boundary.
The current
collection implementation uses streaming root hashing rather than allocating
its complete serialization for each mutation. Historical measurements still
describe the old implementation, not timings of the new one.
The current checkout lockfile is separately pinned by the root
manifest. Running the old sha256sum command against the current root lockfile
is not the historical verification procedure.
The historical source pins remain unchanged; future implementation changes
require preserving their measured source snapshots, not rewriting old hashes.

The source hashes identify the measured implementation, independent of later
documentation changes. They are not signatures or protection against an attacker
who can replace the entire package. The enclosing SOURCE-SHA256SUMS.txt pins the
whole source snapshot; retain its digest separately.

[Measurement protocol and limitations](../EVIDENCE-CAMPAIGN.md).

## Rebuild the measured configuration from public history

The measured code is public: commit `71142a25e7ad75e4d75acf4e244d8e4a996c4a92`
differs from the measured tree only in the workspace version string. Restoring
that string reproduces every pinned measured source byte for byte:

```sh
git checkout 71142a25e7ad75e4d75acf4e244d8e4a996c4a92
sed -i 's/^version = "0.4.0-rc.1"$/version = "0.3.0"/' Cargo.toml Cargo.lock
sha256sum -c docs/evidence-collection/MEASURED-SOURCES.sha256   # 8 of 8 OK, including Cargo.lock
cargo +1.85.0 run --locked --offline --release -p gel-source --example collection_campaign -- ../new-collection-run
cargo +1.85.0 run --locked --offline --release -p gel-source --example collection_recheck -- ../new-collection-run
```

What must match, and what cannot:

- **Deterministic, must match [r1](r1/):** every `sha256`, `text_bytes` and
  `snapshot_bytes` row of [corpora.txt](r1/corpora.txt); 2115 observations with the same
  `(rep, documents, operation, sample)` keys in the same order; `correct=1` on
  every row; the same 99 summary groups; recheck PASS.
- **Not reproducible by rebuilding:** every time (`build_ns`, percentiles) and
  RSS. They depend on the host state, which the original run did not record.

Owner-side rerun on 26 September 2026 (Rust 1.85.0, the same machine, network
namespace, CPU governor `powersave`, other desktop processes running): all
deterministic items above matched r1, recheck PASS, and a copy with one altered
observation failed recheck with `SUMMARY_MISMATCH`. Binaries built from the
public variant were byte-identical to binaries built from the local measured
revision. Per-group p50 ratios new/r1 ranged 0.77–3.38 (median 2.33); this is a
record of a different host state, not evidence of a slowdown or speed-up.
