# Local Linux evidence — Evidence Lab

2026-09-19. Candidate only; no publication approval. These are real program
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
- Windows/macOS: **not executed for this candidate**. Prepared CI is not evidence.
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
