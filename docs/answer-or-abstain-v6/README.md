# Answer-or-abstain set v6

`answer_or_abstain_v6` · SHA-256 `1efb3453d12ed525bc4323deec7eb49e1b73eea9b768f26d3a08bcb057819d89`

990 new questions (495 Polish, 495 English), each about a fact stated in one Wikipedia passage of
the GEL bank, and the answers of:

- GEL RAM, the development build measured in [v4](../answer-or-abstain-v4/README.md) and
  [v5](../answer-or-abstain-v5/README.md), at both of its settings, unchanged;
- Tantivy (BM25) and SQLite FTS5 searching the same passages, at two thresholds and always
  answering.

All answers were recorded on 2026-10-04. Every system answers with one stored passage or with
`UNKNOWN`.

**This set is held out for every system it measures.** The build and every setting were fixed on
v1 + v2 before this set was drawn and written. The passages were drawn and frozen privately
before v5 was published, the questions were frozen by SHA-256 before any system ran on them, and
each system ran once.

**One earlier internal use, stated for completeness.** After the freeze and before the runs
recorded here, the set was used once for an internal experiment on the private implementation.
That experiment changed neither the build nor any setting reported here. The answers of GEL RAM
recorded here are the same passages as in that run: 465 of 465 at the balanced setting.

**Status: public diagnostic.** Once published, these questions are no longer held out. Later
checks use passages that were drawn and frozen privately before this set was published. The
identity above is the SHA-256 of a list of the hashes of every data file of the set;
`answer-bench check` recomputes it.

## The question fixed before the run

**Does the result of v4 and v5 hold on a third new set?** The same build at the same settings.

## How the runs were made

One selection rule for every system, applied on v1 + v2 (474 questions) before this set
existed:

> Choose the setting with the most correct answers at a precision of at least p (0.95 for the
> balanced setting and the plain threshold, 0.99 for the precise setting and the strict
> threshold); on a tie, choose the stricter setting.

| Recorded answers | System | Setting |
|---|---|---|
| `gel-ram` | GEL RAM, development build of 2026-09-30, as in v4 and v5 | balanced (p = 0.95), as in v4 and v5 |
| `gel-ram-precise` | the same build | precise (p = 0.99), as in v4 and v5 |
| `tantivy-bm25`, `-strict`, `-top1` | Tantivy 0.22.1, default tokenizer, query = question words joined by OR | lead of the best passage over the second, (s1 − s2) / s1, above 0.110 / 0.225 / always answer |
| `sqlite-fts5`, `-strict`, `-top1` | SQLite FTS5 (bm25), same query | 0.100 / 0.225 / always answer |

The engines search the passages together with their article title and section heading, as in
v4 and v5. The development build is not part of a release, and its answers cannot be re-run from
this checkout. The engines use default settings without tuning: they are a yardstick, not their
best possible result.

**The set:**
- **Draw:** 1000 passages drawn by a fixed seed from the bank, with the same 150–600 character
  filter as before. They are disjoint from the 3,600 passages of every earlier set and from the
  2,001 passages used in development checks.
- **Questions:** written by the project's AI coding assistant in eight separate parts. Each
  writer saw its passages and their article titles, and no system's answers.
- **Passages without an honest question:** 10 were marked unaskable with a reason before any
  run: navigation bars or image captions without a fact, a repeated article title, a list of
  references, a pair of coordinates, and tables run together without separators. That leaves
  990 questions.
- **Fixes before the freeze:** one. A writer proposed marking a table run together without
  separators as unaskable instead of the substitute question it had written; this was accepted.
  While the set was assembled, the text of two questions was shown on a check screen; no rule or
  setting was changed afterwards.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| `recorded/with-answer/<system>.txt` | answers recorded on 2026-10-04, one file per system; each file states its system and setting in its first line |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments. The scoring rules are
the same as for [v1](../answer-or-abstain/README.md), [v2](../answer-or-abstain-v2/README.md),
[v3](../answer-or-abstain-v3/README.md), [v4](../answer-or-abstain-v4/README.md) and
[v5](../answer-or-abstain-v5/README.md).

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`. Write one tab-separated line per
question: `nr`, a tab, then the answer, for numbers 1 to 990. Then run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v6:with-answer my-answers.txt
```

## Recorded results

The tables show results after the manual review; automatic scores are in brackets where they
differ. The block is generated from the recorded answers (`answer-bench tables v6`).
`answer-bench check`, which `xtask verify` runs, fails if the block shows other numbers.

The last table compares pairs of recorded runs question by question. It counts the questions
that only one of the two answered correctly, and those that only one answered wrongly. p is the
two-sided exact sign test (McNemar's exact test) on those questions.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 465 | 447 (448) | 18 (17) | 525 | 0 |
| GEL RAM | strict | 465 | 447 (448) | 18 (17) | 525 | 0 |
| GEL RAM, precise setting | published | 315 | 313 | 2 | 675 | 0 |
| GEL RAM, precise setting | strict | 315 | 313 | 2 | 675 | 0 |
| Tantivy BM25, threshold | published | 621 | 588 (592) | 33 (29) | 369 | 0 |
| Tantivy BM25, threshold | strict | 621 | 587 (591) | 34 (30) | 369 | 0 |
| Tantivy BM25, strict threshold | published | 418 | 414 | 4 | 572 | 0 |
| Tantivy BM25, strict threshold | strict | 418 | 413 | 5 | 572 | 0 |
| Tantivy BM25, always top 1 | published | 990 | 799 (818) | 191 (172) | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 990 | 798 (817) | 192 (173) | 0 | 0 |
| SQLite FTS5, threshold | published | 652 | 616 (620) | 36 (32) | 338 | 0 |
| SQLite FTS5, threshold | strict | 652 | 615 (619) | 37 (33) | 338 | 0 |
| SQLite FTS5, strict threshold | published | 422 | 417 (418) | 5 (4) | 568 | 0 |
| SQLite FTS5, strict threshold | strict | 422 | 416 (417) | 6 (5) | 568 | 0 |
| SQLite FTS5, always top 1 | published | 990 | 806 (826) | 184 (164) | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 990 | 805 (825) | 185 (165) | 0 | 0 |

| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |
|---|---|---|---:|---:|
| GEL RAM | 96.1% (94.0–97.5%) | 1.8% (1.2–2.9%) | 263 / 274 | 184 / 191 |
| GEL RAM, precise setting | 99.4% (97.7–99.8%) | 0.2% (0.1–0.7%) | 188 / 190 | 125 / 125 |
| Tantivy BM25, threshold | 94.7% (92.6–96.2%) | 3.3% (2.4–4.6%) | 328 / 351 | 260 / 270 |
| Tantivy BM25, strict threshold | 99.0% (97.6–99.6%) | 0.4% (0.2–1.0%) | 260 / 264 | 154 / 154 |
| Tantivy BM25, always top 1 | 80.7% (78.1–83.0%) | 19.3% (17.0–21.9%) | 383 / 495 | 416 / 495 |
| SQLite FTS5, threshold | 94.5% (92.5–96.0%) | 3.6% (2.6–5.0%) | 337 / 363 | 279 / 289 |
| SQLite FTS5, strict threshold | 98.8% (97.3–99.5%) | 0.5% (0.2–1.2%) | 261 / 266 | 156 / 156 |
| SQLite FTS5, always top 1 | 81.4% (78.9–83.7%) | 18.6% (16.3–21.1%) | 388 / 495 | 418 / 495 |

| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |
|---|---:|---:|
| GEL RAM beside Tantivy BM25, threshold | 49 / 190 (p < 0.001) | 11 / 26 (p = 0.020) |
| GEL RAM beside SQLite FTS5, threshold | 42 / 211 (p < 0.001) | 10 / 28 (p = 0.005) |
| GEL RAM, precise setting beside Tantivy BM25, strict threshold | 45 / 146 (p < 0.001) | 1 / 3 (p = 0.625) |
| GEL RAM, precise setting beside SQLite FTS5, strict threshold | 42 / 146 (p < 0.001) | 1 / 4 (p = 0.375) |
<!-- ANSWER-BENCH-RESULTS-END -->

After the review, the verdicts of GEL RAM are the same under the strict rules as under the
published ones; the strict rules count one more answer as wrong for each engine run.

- **The result held a third time.** GEL RAM answered 465 of the 990 questions: 447 correct and
  18 wrong. Its precision was 96.1% (95% Wilson interval 94.0–97.5%), after 95.6% on v1 + v2,
  where the setting was chosen, 95.7% on v4 and 96.3% on v5. Wrong answers were 1.8% of all
  questions.
- **Beside the search engines at the same rule (precision at least 0.95 on v1 + v2)**, the
  picture of v4 and v5 repeats:
  - the engines answered 621 and 652 questions: 588 and 616 correct, 33 and 36 wrong;
  - question by question, the engines answered correctly 190 and 211 questions that GEL RAM did
    not, and GEL RAM 49 and 42 that they did not (p < 0.001): **the engines find more answers**;
  - the engines answered wrongly 26 and 28 questions where GEL RAM did not, and GEL RAM 11 and
    10 where they did not (p = 0.020 and p = 0.005): **GEL RAM gives fewer than half as many
    wrong answers**.
- **At the precise setting GEL RAM gave 313 correct and 2 wrong answers**, a precision of 99.4%
  (97.7–99.8%). On v4 the same setting gave 97.2% and on v5 99.7%, so the three sets together do
  not establish a precision of 0.99.
  - The engines at their strict thresholds gave 414 and 417 correct, with 4 and 5 wrong.
  - They found more answers (p < 0.001); the wrong answers, 1 against 3 and 1 against 4, do not
    differ beyond chance (p = 0.625 and p = 0.375).
- **The review covered every answered pair of the recorded runs, 1,053 distinct pairs.**
  - 700 pairs had already been reviewed blind, by the same rules, in the internal experiment;
    their verdicts were kept, which changed 4 of them.
  - The other 353 pairs were reviewed blind in five separate parts. 18 changes were proposed and
    each was checked against its row; all 18 were accepted (17 to WRONG, 1 to CORRECT).
  - The 53 lines of [review.txt](recorded/with-answer/review.txt) apply these 22 changes to the
    published runs.
  - At the balanced setting 1 of GEL RAM's answers was changed to WRONG; none at the precise
    setting.

Times were not measured on this set.

## Limits

- **Who wrote the set and did the review:** the project's AI coding assistant wrote the
  questions and accepted spellings, and did the review. The review was blind to the systems, in
  separate parts. This is not an independent benchmark.
- **What can be re-run:** GEL's answers come from a development build of the separate private
  implementation, which is not part of a release. Nothing here can be re-run from this checkout,
  because the bank of 671,416 passages is not published.
- **The earlier internal use:** described above; it changed no build or setting reported here.
- **The questions** ask about randomly drawn passages, so many concern narrow facts. They test
  answering from the bank, not general knowledge or conversation.
- **The engines:** they ran with default settings, and a tuned setup may do better.
- **One run:** each system ran once on this set.
- **Text, not truth:** the scorer checks text. A returned passage is long and may contain an
  accepted spelling by chance; the review found such pairs and may have missed others.

## Attribution

Source passages and expected answers quote Wikipedia, available under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions paraphrase it.
The answers are passages of the same bank, reproduced as returned. Third-party material keeps
its own terms, as described in [LICENSING](../../LICENSING.md).
