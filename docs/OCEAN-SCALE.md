# Ocean Scale: recorded 1M/10M full-scan measurements

The Ocean Scale source/evidence archive is not part of the public tree from the
0.4.0 version line onward. The numbers below remain as **historical,
author-reported measurements** of a conventional CPU/RAM full scan. They cannot
be re-run from the current public tree, and no independent reproduction is
claimed. The stable public workspace never depended on them.

## Measured scope and limits

Separate [GEL component measurements](GEL-EXPERIMENTAL-MEASUREMENTS.md)
cover private-prototype search measurements. They measure
different tasks and are not speedup comparisons against this full scan.

EXACT full scan → top10 → decision, milliseconds, 24 workers:

| ORB count | Seed | N | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|
| 1M | 41119 | 100 | 72.427 | 81.018 | 82.859 | 90.949 |
| 1M | 61141 | 100 | 71.962 | 79.860 | 80.576 | 88.227 |
| 10M | 41119 | 100 | 615.428 | 684.888 | 710.313 | 734.085 |
| 10M | 61141 | 100 | 704.995 | 757.900 | 783.038 | 810.236 |

A shorter campaign had a faster 10M/24 median of 498.265 ms. It is not
substituted for the longer runs. Comparing 1/12/24 workers with the same
queries gave a median paired 1→24 speedup of 9.425× at 10M, not 24×.
Do not interpret these full scans as addressed-read latency or general AI recall.
100 observations do not establish stable p99.9–p99.999 tails.

The recorded campaign included 2,800 operations, 660M bitwise score comparisons
and 177,600 scalar oracle checks, all matching expectations. Synthetic
EXACT/NEAR/PARTIAL/MISS/WEAK/TIE/EMPTY are not natural-language evaluation
classes. TIE repeats one distinct query; EMPTY does not scan the full bank.

Host: AMD Ryzen AI 9 HX 370, 24 logical CPUs, Linux, Rust 1.85.0. The owner reports
MINISFORUM AI X1 Pro with 128 GB installed; the OS exposed about 93.9 GiB RAM.
A live desktop could run in the background. GPU inference, cold-cache behaviour,
semantic truth and hardware-level memory effects are not established. Hashes are
not signatures.

## What this is not

It is not a full AI or networked application, not the private execution path and not
a release artifact. The root GEL RAM NCRL 1.0 license remains operative for
GEL-owned material.
