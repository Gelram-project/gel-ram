# Ocean Scale: source, raw evidence and independent checks

This owner-authorized numerical research bundle extends the public v0.3.0 line
without replacing its stable reader or changing its release tag. It is an
experimental Linux research snapshot, not a full AI or P2P application.

## What is included

- O8/T9: adaptive numerical Q8 reader, full ranked scans and evidence checks.
- P2: incremental journal, immutable snapshots, restart/replay and tombstones.
- M1: verified read-only mapping of a kernel-sealed copy, not zero-copy disk I/O.
- C1/R10: 1M/10M numerical campaigns, worker-count comparison and longer repeats.
- Rust 1.85.0 sources, exact vendored dependencies, complete license notices,
  raw CSV, manifest, generated chart and English results/limitations.

No private chat engine, private encoder, private knowledge bank, user data,
keys or private network implementation is included. Large synthetic banks are
generated locally rather than stored in the repository.

## Download and verify on Linux

Archive: [ocean-scale-r2.tar.gz](../research/ocean-scale-r2.tar.gz), 3,006,950 bytes.
SHA256: `f3f5bdcc7a9177b76f14d0a0acc90521bd6cba9893587807a1bbd2d8352b8fde`.

From the repository root, with Rust 1.85.0 already installed:

```sh
printf '%s\n' 'f3f5bdcc7a9177b76f14d0a0acc90521bd6cba9893587807a1bbd2d8352b8fde  research/ocean-scale-r2.tar.gz' | sha256sum --check -
ocean_run="$(mktemp -d)"
tar -xzf research/ocean-scale-r2.tar.gz -C "$ocean_run"
package="$ocean_run/p2-m1-c1-review-r2"
rustc +1.85.0 --edition=2021 "$package/tools/verify_review.rs" -o "$ocean_run/verify-ocean"
"$ocean_run/verify-ocean" "$package" "$ocean_run/evidence"
```

The verifier uses a fresh Cargo home and target, locked offline dependencies,
and keeps build outputs outside the immutable bundle. It runs 100 project
tests, 3 manifest tests and 5 saved-storage-log tests; independently recalculates
C1/R10 statistics and their cross-campaign links; checks P2/M1 log consistency;
then verifies the file inventory and hashes again. It does **not** allocate or
recreate the full 10M bank by default. Local clean runs also used an isolated
network namespace. CI runs locked offline Cargo; it does not claim network
namespace isolation of the whole GitHub-hosted runner.

Inside the extracted bundle, start with README.md, RESULTS.md and CHANGES-R2.md.
The research sources are independent workspaces. Unpacking preserves their
recorded manifest; they are not added as members of the stable root workspace.
Follow the opt-in large-campaign instructions only with sufficient RAM/disk.
They require Linux resource monitoring, including the tested AMD sensor.

## Measured scope and limits

EXACT full scan → top10 → decision, milliseconds, 24 workers:

| ORB count | Seed | N | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|
| 1M | 41119 | 100 | 72.427 | 81.018 | 82.859 | 90.949 |
| 1M | 61141 | 100 | 71.962 | 79.860 | 80.576 | 88.227 |
| 10M | 41119 | 100 | 615.428 | 684.888 | 710.313 | 734.085 |
| 10M | 61141 | 100 | 704.995 | 757.900 | 783.038 | 810.236 |

The shorter C1 campaign had a faster 10M/24 median of 498.265 ms. It is not
substituted for the longer runs. C1 compares 1/12/24 workers with the same
queries; its median paired 1→24 speedup is 9.425× at 10M, not 24×.
Do not interpret these full scans as addressed-read latency or general AI recall.
100 observations do not establish stable p99.9–p99.999 tails.

R10 includes 2,800 operations, 660M bitwise score comparisons and 177,600 scalar
oracle checks, all matching expectations. Synthetic EXACT/NEAR/PARTIAL/MISS/
WEAK/TIE/EMPTY are not natural-language evaluation classes. TIE repeats one
distinct query; EMPTY does not scan the full bank.

Host: AMD Ryzen AI 9 HX 370, 24 logical CPUs, Linux, Rust 1.85.0. The owner reports
MINISFORUM AI X1 Pro with 128 GB installed; the OS exposed about 93.9 GiB RAM.
A live desktop could run in the background. GPU inference, cold-cache behaviour,
semantic truth, physical DRAM-refresh synchronization and PUF are not established.
SIGKILL tests do not prove survival of physical power loss. Hashes are not
signatures; storage requires a separately trusted owner-controlled anchor.

## Provenance, license and publication state

The source archive is the exact previously reviewed R2 snapshot. Its internal
status files preserve the **historical pre-publication state**; they are not
silently rewritten. The current owner-authorized distribution status is in
[CANDIDATE-STATUS.md](../CANDIDATE-STATUS.md). No new legal certification is claimed.
The existing root GEL RAM NCRL 1.0 remains operative for GEL-owned material;
all included upstream dependencies retain their original license terms.

Linux CI verifies the exact archive hash **before extraction**, runs its small
offline verification and prints the result. Existing Windows/macOS jobs cover
the stable workspace, not these Linux-only persistence/mapping modules.
Only existing standard public-repository runners are used; no paid larger
runner, model API, LFS storage, artifact upload or cloud benchmark is added.
See [GitHub's standard-runner policy](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
