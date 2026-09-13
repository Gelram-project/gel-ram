# Measurement host: installed capacity versus OS-visible memory

Hardware clarification, 2026-09-11. This adds provenance to the existing
measurements; it does not change their raw logs, timings or correctness results.

| Field | Value | Evidence |
| --- | --- | --- |
| Computer | MINISFORUM AI X1 Pro | Owner-reported model; local DMI identifies only a generic AI Series |
| Installed memory | 128 GB | Owner-reported configuration; not independently verified from DIMM inventory |
| Processor | AMD Ryzen AI 9 HX 370 with Radeon 890M | Local CPU identification |
| CPU topology | 12 cores, 24 logical CPUs | Local CPU topology |
| OS-visible total memory | 98,474,972 kB, approximately 93.91 GiB | Linux `/proc/meminfo` `MemTotal`, checked 2026-09-11; consistent with the historical measurements |

`MemTotal` is the total usable memory reported by Linux, not the installed
module capacity, current free memory, application RSS or memory used by GEL.
Installed capacity and OS-visible total must not be substituted for each other.
The cause of the difference has not been established here; no particular GPU
or firmware reservation is asserted. GB and GiB are also different units.

The R2 readout campaign ran on the CPU, with no GPU or LLM acceleration,
using Rust 1.85.0 on Linux. The host was shared, without CPU affinity or
exclusive isolation. Worker counts are reported separately for each experiment:
24 logical CPUs available does not mean every operation used 24 workers or
that all installed RAM was exercised. The canonical-baseline campaign used
one worker; the V2 matrix tested one and 24 requested workers.

See the [measurement protocol](Q8-EVIDENCE-CANDIDATE.md#current-measurement-protocol)
and [R2 results](Q8-CANDIDATE-R2-AUDIT.md). Historical hardware snapshots retain
their original OS measurements. The reproduction command records the machine
on which it actually runs; it must not hardcode this owner's configuration.
