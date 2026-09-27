# Measured progress — author-run diagnostics

These measurements concern a separate private implementation, not this public
checkout. Private source, banks and credentials are not included. This is an
aggregate disclosure, not a publicly reproducible benchmark or an LLM leaderboard.

## Ranking update

Same 400 source-text probes, 500k PL + 500k EN bank split into four 250k slots.
Each query knows its slot. Single top-1/top-10: 368/393 → 379/397 out of 400.
Quad: 310/361 → 371/396. Quad recovered 63 top-1 matches and lost two.
Sequence encoding and removal of a known export footer both changed.
Do not attribute the entire improvement to either alone. No per-answer
latency series for the new profile is reported here.

## Source-field dialogue: first timing run

Five real articles, 82 fields, resident bank. Release, Rust 1.97.0,
Ryzen AI9 HX370. Nearest-rank, N=1000 warm repetitions per row, 20 warmups.
Timer covers Dialogue::respond through completed reply construction,
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
