# Measured progress — author-run diagnostics

> **Private measurement — not runnable from this repository.** Every result on
> this page was produced by the author on the separate private implementation.

These measurements concern a separate private implementation, not this public
checkout. Private source, banks and credentials are not included. This is an
aggregate disclosure, not a publicly reproducible benchmark or an LLM leaderboard.

## Ranking update

Same 400 source-text probes, 500k PL + 500k EN bank split into four 250k slots.
Each query knows its slot. Single top-1/top-10: 368/393 → 379/397 out of 400.
Quad: 310/361 → 371/396. Quad recovered 63 top-1 matches and lost two.
Two changes to the private pipeline were made between the runs.
Do not attribute the entire improvement to either alone. No per-answer
latency series for the new profile is reported here.

## Source-field dialogue: first timing run

Five real articles, 82 fields, resident bank. Release build,
Ryzen AI9 HX370. Nearest-rank, N=1000 warm repetitions per row, 20 warmups.
Timer covers the dialogue response call through completed reply construction,
excluding loading, display, report writes and subsequent Q8 fallback.
The first run is shown, not a selected best run. All times are **µs**.

| ID / subject | Result | p1 | p10 | p50 | p95 | p99 | max |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 Austria capital | source field | 1.793 | 1.894 | 2.074 | 3.957 | 5.511 | 13.385 |
| 1 Austria borders | UNHANDLED | 4.408 | 4.518 | 4.789 | 8.987 | 12.624 | 16.952 |
| 2 Belgium spoken languages | UNHANDLED | 4.718 | 4.829 | 5.099 | 9.368 | 12.994 | 20.879 |
| 3 Belgium capital | source field | 1.824 | 1.933 | 2.124 | 4.228 | 7.394 | 18.385 |
| 4 Argentina continent | UNHANDLED | 4.198 | 4.308 | 4.579 | 8.496 | 12.093 | 15.910 |
| 5 Argentina name origin | UNHANDLED | 3.577 | 3.657 | 3.847 | 7.094 | 7.564 | 14.437 |
| 6 Brazil capital | source field | 1.924 | 2.024 | 2.234 | 4.509 | 7.324 | 29.966 |
| 7 Brazil official language | source field | 3.647 | 3.907 | 4.368 | 9.588 | 14.557 | 40.125 |
| 8 Bulgaria capital | source field | 1.924 | 2.024 | 2.224 | 4.329 | 6.943 | 12.373 |
| 9 Bulgaria borders | UNHANDLED | 4.609 | 4.749 | 5.040 | 9.147 | 12.463 | 18.315 |
| 10 Austria border negation | UNHANDLED | 4.629 | 4.729 | 5.019 | 10.109 | 12.934 | 17.623 |
| 11 Belgium and Austria | whole-field comparison | 18.224 | 18.615 | 19.967 | 39.404 | 50.926 | 68.138 |
| 12 unknown subject | UNKNOWN | 0.952 | 0.972 | 1.032 | 2.415 | 4.749 | 24.496 |

Across all three runs, field-answer medians span 2.054–4.819 µs.
Whole-field comparison medians span 19.417–21.220 µs.
The slowest warm sample, 711.017 µs, occurred in run B, question 8.
No affinity/governor pinning or host isolation measurement; intermediate CSV
writes may influence caches/scheduling. No latency subtraction.
39,780 reply identity checks are repetitions of 13 questions, not that many
independent quality cases. UNHANDLED is not a successful semantic decision.
Newer quotation/list support was not timed in this campaign.

## New-article transfer

Four new PL articles, 69 fields, 13 questions: six source-field answers,
two quotations, one limited whole-field comparison, three UNHANDLED and
one UNKNOWN. Six field outputs were checked against HTML cells.
Japan's language answer retained both “de facto” and “de iure” qualifiers.
Two offline runs had identical output. No timing measurement; not 9/13
semantic accuracy, not a million-record deployment. Known templates and no
independent adjudicator limit generalization.

## Evidence identities

Retained privately; SHA-256 pins identify artifacts but do not make absent
data independently reproducible or establish source truth.

| Artifact | SHA-256 |
|---|---|
| Timing A | 650f48bbbfa366be64038a70476202251b77f21b9067930ed809d766c8253368 |
| Timing B | 61bb6d1c68382b0746c6cd4bdf6c15152fe75d09c103363a9abe887ba09c4b07 |
| Timing C | 9729a7231caef9f6ceca8d62503032f0a8f4e91015b3eb2b31ea4f4e23311885 |
| Transfer questions | 4645a55e858e3fb5c9ddbbf08a9e43e303301b2d0ffa6a89d1ebfad26934df3a |
| Transfer output | e477db6d267567f9356a0e389c9c4692634d0f227dc354863f3f1777c75eb26b |

## Relation to Groq and animation

[Existing Groq diagnostic](GEL-GROQ-DIAGNOSTIC.md) reports the separate
supplied-source decision task, including incomplete attempts and protocol
errors. The graphic's 537.134–1219.564 ms range covers six completed HTTP
batches, not percentiles and not six flawless responses.
GEL's bounded adapter returned only UNKNOWN on that shared task.
No cross-task speedup multiplier is justified.

The SVG contains static figures and CSS row highlights, no scripts, external
resources, tracking or private code. The full table stays visible when
animation is unsupported or reduced motion is requested. Animation pacing
does not represent measured execution. Existing repository licensing applies.

## Answer verdict — measured after the v0.5.0 release

Same private implementation and the same 1M PL/EN bank (four 250k slots) as the
ranking section above. A verdict answers only when the best passage leads the
runner-up by more than a threshold that was fixed in advance
from a different corpus and never tuned on these probes. Correct means the
answer comes from the same article as the probe passage. Probes are stored
passages read back and ranked within their own slot, sampled at a fixed step.

| Stage (each measured separately) | Passages per slot | Probes | Answered | Correct answers | Wrong among all probes | UNKNOWN |
|---|---:|---:|---:|---:|---:|---:|
| Original bank | 250,000 | 10,000 | 77.0% | 97.7% | 1.78% | 23.1% |
| Identical passages (also ones the encoder cannot tell apart) stored once, all sources and variants kept | 231,033 | 10,000 | 83.7% | 98.6% | 1.17% | 16.3% |
| Reference, link and bibliography sections left out | 167,854 | 10,000 | 88.5% | 98.8% | 1.04% | 11.5% |
| Encoder keeps numbers and short words (selection sample) | 167,854 | 10,000 | 92.8% | 99.94% | 0.06% | 7.2% |
| Same, validation sample | 167,854 | 9,600 | 93.1% | 99.96% | 0.04% | 6.9% |
| Same, large sample | 167,854 | 50,000 | 92.8% | 99.95% | 0.046% | 7.2% |

The encoder setting was chosen among three variants named before the run, on
the selection sample; the validation and large samples use different probe
steps. Large sample per slot: correct answers 0.9997, 0.9991, 0.9998, 0.9993;
95% Wilson lower bounds 0.9993, 0.9984, 0.9994, 0.9986. At equal bank size, a
control with the reference sections kept reached 83.8% answered and 1.26% wrong,
so the gain from that stage is not an effect of the smaller bank. Leaving those
sections out changes the scope of the answer bank (Polish slots −27%, English
−3–5%); they remain available as sources.

## Natural questions on the same bank

80 questions (40 PL, 40 EN), written by the project's AI coding assistant as
paraphrases for passages sampled with a fixed seed, frozen by SHA-256 before
any run. The
question is not told which slot to search. A separate set of 80 calibration
questions, sampled from other passages, fixed the verification settings (how
many candidates are checked and the lead threshold: the largest lead of a wrong winner in
calibration); the test set was then run once.

| Metric | Without verification | With source verification |
|---|---:|---:|
| Top-1 from the right article | 27/80 | 40/80 |
| Right passage among the verified candidates | — | 61/80 |
| Answers given | 9 (8 correct) | 11 (11 correct) |
| UNKNOWN | 71 | 69 |

Calibration at the chosen threshold: 14 answers, 14 correct. 11/11 on the test
set has a 95% Wilson lower bound of about 0.74; this small assistant-written set
is not an independent benchmark and supports no precision rate. The open
problem is coverage: 14% of the questions are answered.

The same 80 questions were later put, closed book, to three language models on
the Groq API in one recorded run with one scoring rule:
[GEL beside three language models](GEL-BESIDE-GROQ.md).

## Evidence identities — answer verdict

Retained privately; SHA-256 pins identify artifacts, they do not make absent
data reproducible.

| Artifact | SHA-256 |
|---|---|
| Original bank, 10,000 probes | fda0ee049c1c9007e6494dcf9ade020553e98f4af8a39cd42963a0a98fdfcf8f |
| Final bank, validation sample | 892613f7c0447834f110e6a394c2fa4fceca6a29b4c340b0031027e22157839e |
| Final bank, 50,000 probes | 7292c0360fc4d3c1786c7301a898ed777d126c00fae0d2f3510afcea25b60eb2 |
| Test questions with reference passages | 9a1b13450bb4767bfdb43071d00ed0506e0a7ca718fb780b41c5ee35bb103e4a |
| Calibration questions with reference passages | 9dd133b7eacb8e52ade3f36ac85205e551da36527c2ad41b06b27d6a8fe1d3c6 |
| Calibration answers | 119fd25af3e4d6cd34ee920e4c6d6b9c72afd7783a9f60c5c7699c027c05c1fa |
| Test answers with verification | 50714466b576a525a4627b24165740bbc1734d5bff9ab1710e5fd6fc6469797e |

## Kill during learning — the private knowledge store

> **Private measurement.** Author-run on 2026-09-29 on the separate private
> implementation; it cannot be re-run from this checkout. The
> [crash series](CRASH-SERIES.md) runs the same kind of test on the public tool,
> and anyone can run it. The claim registry lists this result as `MEASURED_LOCAL`.

The private store grows while it learns and writes what it has confirmed to
disk. The learning process was killed with SIGKILL at a seeded random moment
(series A and B, which differ in how often the store writes to disk) or stopped
at seven fixed points inside writing (series C). The store was then reopened
from disk and checked, and the same learning was resumed to the end and
compared with a run without a kill. The plan and the script were fixed before
the first trial.

| Series | Trials | Killed while learning | Confirmed records lost | Reopen check failed | Result after resume ≠ run without a kill | Killed inside an atomic write |
|:---|---:|---:|---:|---:|---:|---:|
| A | 100 | 84 | **0** | 0 | 0 | 12 |
| B | 100 | 98 | **0** | 0 | 0 | 56 |
| C — 7 fixed points | 7 | 5 (2 more while writing) | **0** | 0 | 0 | 1 |

In total 235,712 confirmed records were checked after the kills; none was lost.

A process kill is not a power cut: caches of the operating system and the disk
are not tested. The store is small, on one machine and one file system; the
random kills fell within the first seconds of each run, so later writing points are
covered only by series C.

Evidence identity: `fc66b40594cbdc18476620f5c88891785c8dfda31fe12de8a6baa730ae86aa08`,
the SHA-256 of the six SHA-256 hashes of the private artifacts (plan, series
script, instrument, input checksums, per-trial results, summary), one lowercase
hash per line in that order. It shows later tampering; it does not make the
private data reproducible.
