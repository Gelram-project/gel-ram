# Exact external dependency inventory

Review snapshot, 2026-09-19, updated 2026-10-17 when SHA-256 moved into
gel-source and ten packages left. These are the 2 external packages in Cargo.lock,
including transitive/target-specific/build dependencies, not just the host subset.
Names, versions and declared license expressions come from locked offline Cargo
metadata. Each cached .crate SHA-256 was compared to Cargo.lock. The license-file
hashes below were also checked against the files extracted from those archives.
No dependency source or binary is vendored by this source-only candidate.

This records provenance, not a vulnerability scan or legal approval. The full
upstream license text governs; a binary/vendored release needs its own notices
review. The historical metadata spelling `MIT/Apache-2.0` is preserved verbatim.
Toolchain, OS libraries and CI actions are not Cargo dependencies and are not
included in these two rows. CI action revisions are pinned in the workflow.
The Ocean research archives that were on main before 0.4.0 carried their own vendored
dependencies and inventories; these rows describe the root workspace only.

| Package | Version | Declared license | Upstream |
|---|---|---|---|
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | [repository](https://github.com/Lokathor/tinyvec) |
| unicode-normalization | 0.1.25 | MIT OR Apache-2.0 | [repository](https://github.com/unicode-rs/unicode-normalization) |

## Archive and license-file SHA-256

Names in the blocks are paths within the named upstream archive, not GEL files.

### tinyvec 1.13.3

```text
fd3ca314f692efd6c868f8408f53fe444634a845f96c028b97d35f6a1f79f0ee  tinyvec-1.13.3.crate
cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30  tinyvec-1.13.3/LICENSE-APACHE.md
fd80a26fbb3f644af1fa994134446702932968519797227e07a1368dea80f0bc  tinyvec-1.13.3/LICENSE-MIT.md
84b34dd7608f7fb9b17bd588a6bf392bf7de504e2716f024a77d89f1b145a151  tinyvec-1.13.3/LICENSE-ZLIB.md
```

### unicode-normalization 0.1.25

```text
5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8  unicode-normalization-0.1.25.crate
a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2  unicode-normalization-0.1.25/LICENSE-APACHE
7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0  unicode-normalization-0.1.25/LICENSE-MIT
```

## Other redistributed third-party material

The MIT Rust Book excerpt is identified separately in [real-source provenance](REAL-SOURCE-DEMO.md), with its full license beside the fixture. It is not a Cargo dependency. GEL-owned material remains under the root license; this inventory neither changes that license nor licenses third-party material on GEL terms. Recheck the inventory whenever Cargo.lock or distributed fixtures change.

The collection-review fixture reuses that exact same excerpt with its MIT notice
copied beside it; [provenance](../crates/gel-source/fixtures/collection-review/PROVENANCE.md).
No new upstream source or Cargo dependency is introduced by this copy. Capture
tools (FFmpeg, Xvfb, xterm) are not shipped in the source package.
