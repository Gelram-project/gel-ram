# Reproduce collection and quantization evidence

```sh
cargo run --locked --offline --release -p gel-source --example collection_campaign -- ../new-collection-run
cargo run --locked --offline --release -p gel-source --example collection_recheck -- ../new-collection-run
cargo run --locked --offline --release -p gel-cli --example quantization_matrix
```

The collection campaign generates authored synthetic documents; it does not read
private files or the Internet. Sizes 8/64/256, three passes with reversed middle
order. 50 EXACT/NEAR/MISS/UNICODE samples per size/pass, plus 5 each of serialization,
deserialization, insert, update, delete, save and load: **2115 observations**.
The oracle checks exact generated identifiers and byte-equal reconstruction, not
semantic recall. Four query classes are not independent natural-language datasets.

Raw CSV, summary, corpus pins and environment remain in the new output directory.
The reducer checks the entire configuration/sample matrix and recomputes min,
p50/p95/p99/max and total. Percentiles are nearest-rank; N=5 or 50 within a group
is too small for strong tail claims (at N=50 p99 is the maximum). No p99.9 claim.
Raw times include slower samples. No best-run selection.

Search includes result allocation, not rendering. Update/insert/delete exclude
the preparatory clone but include root hashing/serialization. Save includes sync;
load is a warm-process/page-cache observation, not cold-boot latency. One thread.
No affinity, frequency lock, CPU isolation or other-host reproduction is claimed.
Linux VmRSS/VmHWM samples are process-wide; HWM is cumulative across cases, not
an independently reset bank peak. Input bytes and snapshot bytes exclude allocator,
BTreeMap and query/result overhead. Index overhead and cold start are not measured.

The independent reducer checks arithmetic/configuration, not the honesty of the
host clock or authenticity of a forged matching pair of raw data and summary.
Source pins and original retained logs supply provenance separately.

## Q reference is a different experiment

Q1/Q2/Q4/Q8 affine min/max uses 32 f32 samples per block, packed LSB first,
and two f32 endpoints (8 metadata bytes). Total block sizes 12/16/24/40 bytes,
not just 4/8/16/32 payload bytes. No private encoder or decoder is included.
Six synthetic sample families, 21 timings after warmup, full raw nanoseconds,
RMSE and maximum absolute error. Tiny signals beside outliers can be lost even
in Q8; loss is not necessarily noise. F16 is not implemented by this historical
Q matrix; see the separate data-integrity reference and the public
[precision matrix](PRECISION-MATRIX.md) (F32/F16/Q1–Q16, untimed).

No results here establish media perception, 4x independent capacity, conversation
quality or superiority to commercial AI. Existing Ocean runs are a different
numeric workload and must not be merged into this text-search timing table.

## The comparison protocol

A timing comparison is recorded under the protocol when its record states, in
the record itself and not only in a page about it:

| Field | What it states |
|---|---|
| input | the program's arguments and the SHA-256 of the program source that generates its inputs from fixed seeds |
| profile | release, the cargo command, the value of `RUSTFLAGS` and the name and value of every cargo variable that changes code generation (`CARGO_PROFILE_*`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_BUILD_TARGET`, `CARGO_ENCODED_RUSTFLAGS` split at its separator, `CARGO_TARGET_*_RUSTFLAGS` and `_LINKER`, `CARGO_INCREMENTAL`), with paths written as labels; locations such as `CARGO_HOME` are not recorded, and a name without its value is refused |
| clock boundary | what lies inside the timed interval and what lies outside it |
| warm-up | how many untimed rounds ran first; their outputs are kept and excluded from the medians |
| order | the run order of every round; the three trees rotate, so a multiple of three rounds puts each tree in each position equally often |
| A/A control | a second tree of the baseline source, built and run like the first, and whether the two builds are the same binary |
| host metadata | operating system, architecture, logical CPUs, CPU model, memory, kernel, load average at the start and the end |

Host metadata never contains the host name, the user name, serial numbers or
paths; tree roots and the waiting hook are written as labels, and so is
every argument with a piece (split at `=` and `,`) that is absolute, starts
with `~`, has a `home` or `Users` component or climbs with `..`. Every raw
run output is kept beside the manifest and the summary.

`xtask speed-ab` writes such a record (`GEL_SPEED_AB_2`, [A/B tool](SPEED-AB.md)),
and `xtask speed-ab-check DIR` accepts it only when every field is present,
the record has an A/A control, the rotation is complete, no raw run file is
missing or extra and the summary recomputes byte for byte from the raw runs.
A and A2 must name the same full commit and every tree a clean working tree.
A candidate-only run has no A/A control and is refused. A refusal names the
manifest line and key, never the refused value. Records written before
this protocol (`GEL_SPEED_AB_1`) are historical and are refused, not
re-labelled. The check concerns the record; it cannot show that a host clock
was honest or that a machine was quiet.

## Historical timing campaigns and the protocol

| Campaign | Task and inputs | Inputs in the tree | Re-run under the protocol |
|---|---|---|---|
| [Speed set 2026-10-20 against v0.6.1](speed-results/2026-10-20-against-v0.6.1/README.md), the README speed card | 24 timing examples; A and A2 at v0.6.1, B with the library code of 876247f; inputs generated from fixed seeds | yes; both revisions are public | yes, [plan](speed-campaign/rerun-2026-10-20-against-v0.6.1.txt); B becomes a checkout of 876247f instead of the recorded local tree |
| [Quiet machine 2026-10-25, package 2](speed-results/2026-10-25-quiet-machine/README.md) | 6 timing examples; bb3da1b against 90bd091 | yes | yes, [plan](speed-campaign/rerun-2026-10-25-package-2.txt) |
| [Quiet machine 2026-10-25, package 3](speed-results/2026-10-25-quiet-machine/README.md) | 4 timing examples against local trees of PR #89; the baseline of the collection load, 37d509d, is not on `main` | the examples yes, the recorded trees no | only with the nearest public revisions, [store](speed-campaign/rerun-2026-10-25-package-3-store.txt) and [loads](speed-campaign/rerun-2026-10-25-package-3-loads.txt); the loads then compare the whole of PR #89; the peak-memory tables were read with an external timer and stay outside |
| [Grep comparison r1](BENCHMARK-GREP.md) | GEL batch search against grep and sha256sum on the public corpus | the corpus yes; revision dc38282 is not in the public history | `bench-compare` records input, clock boundary, warm-up, order and host load, but compares two different tools: it has no A/A control |
| [Collection campaign r1](evidence-collection/README.md) and the quantization matrix (top of this page) | synthetic documents and sample families | yes, through the pinned measured sources | a single-tree program: no A/A control, no rotation of builds |
| [Mutation campaign r1](MUTATION-COMPARISON.md) | streamed against historical mutation inside one binary; synthetic corpus | the corpus yes; the measured revision is not in the public history | `mutation-campaign` alternates the two variants after a warm-up; no A/A control of a variant against itself |
| [Q8 V1, 48 runs](Q8-QUAD-RESULTS.md), [Q8 current](evidence-q8-current/README.md) and r2 | Q8 view comparisons on synthetic records, and rows from an external input | synthetic generators yes; the V1 runner revision and the external input no | synthetic rows through their own runner, one tree, no A/A control |
| [Document probe](evidence-document/README.md), [Top-K v0.2.1](evidence-v0.2.1/final-topk-v3.txt), [silicon 2026-09-03](SILICON-2026-09-03.md) and [2026-09-04](SILICON-2026-09-04.md) | generated text, synthetic records, memory probes | generators yes; the measured revisions or probe versions are no longer current | no as recorded; the current gel-bench and gel-physics runs are inside the 2026-10-20 plan through `speed_bench_physics` |
| [Measured progress](MEASURED-PROGRESS.md) and [component measurements](GEL-EXPERIMENTAL-MEASUREMENTS.md) | a separate private implementation | no | no |

The withdrawn Ocean Scale 1M/10M full scans ([historical numbers](OCEAN-SCALE.md))
are excluded: their source and evidence archive is a research archive outside
the tree, so they cannot be re-run here. Single observations, such as the film
timings, are not campaigns.

## Re-running the recorded speed sets

```text
cargo run --locked --offline -p xtask -- speed-campaign \
  --plan docs/speed-campaign/PLAN.txt \
  --a BASELINE --a2 BASELINE_COPY --b CANDIDATE \
  --target-root BUILD_DIR --out NEW_DIR [--before-run PROGRAM [--before-run-arg ARG]...]
```

A, A2 and B must be clean checkouts of the plan's `baseline` and `candidate`
commits; any other revision or a changed tree is refused before anything is
built. A plan fixes the examples, their arguments, the rounds (a multiple of
three) and one or more warm-up rounds; each example runs through `speed-ab`
into `NEW_DIR/EXAMPLE/` and is then checked with `speed-ab-check`. The file CAMPAIGN.txt
lists each example with its expected and actual verdict and ends with
`SPEED_CAMPAIGN=COMPLETE`, or `INCOMPLETE` with a non-zero exit when an example
did not build, failed or printed an unexpected verdict. `--runs N` and `--only
EXAMPLE` shrink a campaign for a trial run; they require `--invalid-for-timing`,
which every record of that run carries, and the last line is then `PARTIAL`.

All four plans were run once on 2026-11-04, on the author's machine; the
records are in [2026-11-04-reruns](speed-results/2026-11-04-reruns/README.md).
The recorded sets of 2026-10-20 and 2026-10-25 stay as they were measured and
are not re-labelled; the re-run stands beside them.

## Memory accounting

`xtask memory-run` records payload, mapped and resident peaks, page faults,
threads and host conditions of public operations, at least five processes each,
and `xtask memory-check` refuses an incomplete record
([instrument, format and limits](MEMORY-ACCOUNTING.md)). It records no time, so
its records are not timing campaigns and do not follow the comparison protocol.
One run at revision 390136f, nine processes of each of the eight operations on
one machine, is retained in
[memory-results/2026-11-08](memory-results/2026-11-08/README.md); it passes
`memory-check`.
