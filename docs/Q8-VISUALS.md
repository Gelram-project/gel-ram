# Q8 visuals: measured values, bounded claims

The [historical README charts](../README-HISTORY-R2.md#measured-shared-readout-advantage--historical-v1)
visualize the complete historical 48-run V1 campaign. This is a documentation
change, not a new benchmark or a change to the scoring algorithm.

## Source and calculation

- Source: [machine-readable summary](evidence-q8-quad/SUMMARY.txt), all
  [48 raw logs and hardware metadata](evidence-q8-quad/README.md).
- For each invocation, sum the nine timed FourViews samples and divide by
  the sum of the nine timed Shared samples; exclude warm-up timing.
- For each record-count/mask/policy/worker cell, take the median of its three
  invocation ratios and round to two decimals for the bars.
- Keep the complete minimum–maximum range of those same three ratios in the
  [numerical results table](Q8-QUAD-RESULTS.md#all-matrix-cells). A range of
  three observations is not a confidence interval.
- Both plots start at zero and end at six. The 1× line denotes equal time.
  Color is redundant with the worker count in each title; values and ranges
  remain available as a text table if diagrams cannot render.

All 16 cells are included. Repetitions share the seed and query schedule;
they quantify timing variation, not accuracy on independent new queries.
The 8,355,840 exact comparisons include warm-up, unlike the timing sums.

## Interpretation

The comparison is FourViews versus Shared, **not Shared versus the fastest
existing Q8 reader**. The ReferenceOne arm is retained in raw logs and Shared
does not beat it uniformly. Ratios above four are whole-implementation results
that may include layout/cache effects, not evidence of extra information.

The two worker plots must not be read as a parallel scaling curve: their
denominators differ. Consult absolute Shared latency in the results table.
In three of four 512-record cells, 24 workers were slower than one.
24 workers is a budget used in these runs, not sustained 100% CPU utilization.

Hardware: Ryzen AI 9 HX 370, 12 cores / 24 logical CPUs, Rust 1.85.0 release,
2026-09-08. Shared host; no exclusive isolation or CPU affinity.
The [V2 worker-refusal fix](Q8-QUAD-VALIDATION.md) has separate correctness
evidence and no replacement performance campaign. Do not relabel these V1
bars as measured V2 performance.

The colored flow diagram shows one canonical record and equivalent views.
It is not a physical-wave circuit, four independent facts, a new encoder or
an end-to-end knowledge-answering system. Packed record sizes exclude bank
allocation, indices and other process overhead.

## Rendering

These are text-native Mermaid diagrams, not AI-generated benchmark images.
They use [GitHub's Markdown diagram support](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams)
and [Mermaid XY charts](https://mermaid.js.org/syntax/xyChart.html).
No runtime dependency or release file-type exception is added to GEL.
