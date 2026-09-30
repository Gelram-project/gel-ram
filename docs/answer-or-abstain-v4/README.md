# Answer-or-abstain set v4

`answer_or_abstain_v4` · SHA-256 `287eff4ff828d327e10942f07e292546ba31d1601d7ec92f37de928e430e0034`

985 new questions (493 Polish, 492 English), each about a fact stated in one Wikipedia passage of the GEL bank, and the answers of:

- GEL RAM, a development build of 2026-09-30, at two settings;
- the build published in [v3](../answer-or-abstain-v3/README.md), for comparison;
- Tantivy (BM25) and SQLite FTS5 searching the same passages, at two thresholds and always
  answering, plus both engines on the bank as it was in v3.

All answers were recorded on 2026-09-30. Every system answers with one stored passage or with
`UNKNOWN`.

**This set is held out for the build it measures.** The change to GEL and both of its settings
were fixed on v1 + v2 before this set was drawn and written. The set was frozen by SHA-256
before any system ran on it, and each system ran once.

**Status: public diagnostic.** Once published, these questions are no longer held out. The
identity above is the SHA-256 of a list of the hashes of every data file of the set;
`answer-bench check` recomputes it.

## What changed since v3

- **Passages are searched together with their article title and section heading.** In v3,
  7 of GEL RAM's 11 wrong answers came from a correct passage that does not repeat its
  subject's name, which stands only in the article title. Another passage that did contain the
  name won instead. Two more came from records that hold nothing but an article title.
  - The bank now reads each passage with its title and section heading when it is searched.
  - Records that hold only a title are no longer searched.
  - The answer shown is still the stored passage itself, as in v3, so the scorer sees the same
    kind of text.
- **The search engines get the same passages with titles**, so the comparison stays on one
  bank. Their results on the v3 bank are recorded as well, to show what the titles do for
  them.
- **Two GEL settings**, both chosen on v1 + v2 by the rule below: a balanced one (precision at
  least 0.95) and a precise one (precision at least 0.99). The engines' thresholds were chosen
  the same way.

## How the runs were made

One selection rule for every system, applied on v1 + v2 (474 questions) before this set
existed:

> Choose the setting with the most correct answers at a precision of at least p (0.95 for the
> balanced setting and the plain threshold, 0.99 for the precise setting and the strict
> threshold); on a tie, choose the stricter setting.

| Recorded answers | System | Setting |
|---|---|---|
| `gel-ram` | GEL RAM, development build of 2026-09-30 | balanced (p = 0.95) |
| `gel-ram-precise` | the same build and run | precise (p = 0.99) |
| `gel-ram-v3-build` | the build published as `gel-ram` in v3, on the v3 bank | its v3 setting |
| `tantivy-bm25`, `-strict`, `-top1` | Tantivy 0.22.1, default tokenizer, query = question words joined by OR | lead of the best passage over the second, (s1 − s2) / s1, above 0.110 / 0.225 / always answer |
| `sqlite-fts5`, `-strict`, `-top1` | SQLite FTS5 (bm25), same query | 0.100 / 0.225 / always answer |
| `tantivy-bm25-v3-bank`, `sqlite-fts5-v3-bank` | both engines on the v3 bank | their v3 thresholds, 0.230 and 0.235 |

The development build is not part of a release, and its answers cannot be re-run from this
checkout. The engines use default settings without tuning: they are a yardstick, not their best
possible result.

**The set:**
- **Draw:** 1000 passages drawn by a fixed seed from the bank, disjoint from the 1600 passages
  of every earlier set, with the same 150–600 character filter as before.
- **Questions:** written by the project's AI coding assistant in eight separate parts. Each
  writer saw its passages and their article titles, and no system's answers.
- **Passages without an honest question:** 15 were marked unaskable with a reason before any
  run: navigation bars, captions, footnotes, table legends and tables whose cells were run
  together. That leaves 985 questions.
- **Fixes before the freeze:** a checking program flagged two questions. One passage was then
  marked unaskable (a table fused so that the answer is not a separate word), and one accepted
  spelling was added in the form the scorer normalizes to.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| `recorded/with-answer/<system>.txt` | answers recorded on 2026-09-30, one file per system; each file states its system and setting in its first line |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments. The scoring rules are
the same as for [v1](../answer-or-abstain/README.md), [v2](../answer-or-abstain-v2/README.md)
and [v3](../answer-or-abstain-v3/README.md).

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`. Write one tab-separated line per
question: `nr`, a tab, then the answer, for numbers 1 to 985. Then run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v4:with-answer my-answers.txt
```

## Recorded results

The tables show results after the manual review; automatic scores are in brackets where they
differ. The block is generated from the recorded answers (`answer-bench tables v4`).
`answer-bench check`, which `xtask verify` runs, fails if the block shows other numbers.

The last table compares pairs of recorded runs question by question. It counts the questions
that only one of the two answered correctly, and those that only one answered wrongly. p is the
two-sided exact sign test (McNemar's exact test) on those questions.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 423 | 405 | 18 | 562 | 0 |
| GEL RAM | strict | 423 | 405 | 18 | 562 | 0 |
| GEL RAM, precise setting | published | 289 | 281 | 8 | 696 | 0 |
| GEL RAM, precise setting | strict | 289 | 281 | 8 | 696 | 0 |
| GEL RAM, v3 build | published | 189 | 172 (173) | 17 (16) | 796 | 0 |
| GEL RAM, v3 build | strict | 189 | 172 (173) | 17 (16) | 796 | 0 |
| Tantivy BM25, threshold | published | 571 | 531 (534) | 40 (37) | 414 | 0 |
| Tantivy BM25, threshold | strict | 571 | 531 (534) | 40 (37) | 414 | 0 |
| Tantivy BM25, strict threshold | published | 366 | 361 | 5 | 619 | 0 |
| Tantivy BM25, strict threshold | strict | 366 | 361 | 5 | 619 | 0 |
| Tantivy BM25, always top 1 | published | 985 | 747 (760) | 238 (225) | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 985 | 746 (759) | 239 (226) | 0 | 0 |
| SQLite FTS5, threshold | published | 599 | 558 (561) | 41 (38) | 386 | 0 |
| SQLite FTS5, threshold | strict | 599 | 558 (561) | 41 (38) | 386 | 0 |
| SQLite FTS5, strict threshold | published | 371 | 363 | 8 | 614 | 0 |
| SQLite FTS5, strict threshold | strict | 371 | 363 | 8 | 614 | 0 |
| SQLite FTS5, always top 1 | published | 985 | 750 (763) | 235 (222) | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 985 | 749 (762) | 236 (223) | 0 | 0 |
| Tantivy BM25, v3 bank | published | 248 | 220 (223) | 28 (25) | 737 | 0 |
| Tantivy BM25, v3 bank | strict | 248 | 220 (223) | 28 (25) | 737 | 0 |
| SQLite FTS5, v3 bank | published | 243 | 215 (218) | 28 (25) | 742 | 0 |
| SQLite FTS5, v3 bank | strict | 243 | 215 (218) | 28 (25) | 742 | 0 |

| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |
|---|---|---|---:|---:|
| GEL RAM | 95.7% (93.4–97.3%) | 1.8% (1.2–2.9%) | 232 / 244 | 173 / 179 |
| GEL RAM, precise setting | 97.2% (94.6–98.6%) | 0.8% (0.4–1.6%) | 165 / 171 | 116 / 118 |
| GEL RAM, v3 build | 91.0% (86.1–94.3%) | 1.7% (1.1–2.7%) | 94 / 106 | 78 / 83 |
| Tantivy BM25, threshold | 93.0% (90.6–94.8%) | 4.1% (3.0–5.5%) | 295 / 323 | 236 / 248 |
| Tantivy BM25, strict threshold | 98.6% (96.8–99.4%) | 0.5% (0.2–1.2%) | 222 / 227 | 139 / 139 |
| Tantivy BM25, always top 1 | 75.8% (73.1–78.4%) | 24.2% (21.6–26.9%) | 368 / 493 | 379 / 492 |
| SQLite FTS5, threshold | 93.2% (90.8–94.9%) | 4.2% (3.1–5.6%) | 308 / 336 | 250 / 263 |
| SQLite FTS5, strict threshold | 97.8% (95.8–98.9%) | 0.8% (0.4–1.6%) | 223 / 231 | 140 / 140 |
| SQLite FTS5, always top 1 | 76.1% (73.4–78.7%) | 23.9% (21.3–26.6%) | 371 / 493 | 379 / 492 |
| Tantivy BM25, v3 bank | 88.7% (84.2–92.1%) | 2.8% (2.0–4.1%) | 140 / 163 | 80 / 85 |
| SQLite FTS5, v3 bank | 88.5% (83.9–91.9%) | 2.8% (2.0–4.1%) | 134 / 156 | 81 / 87 |

| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |
|---|---:|---:|
| GEL RAM beside GEL RAM, v3 build | 240 / 7 (p < 0.001) | 13 / 12 (p = 1.000) |
| GEL RAM beside Tantivy BM25, threshold | 66 / 192 (p < 0.001) | 15 / 37 (p = 0.003) |
| GEL RAM beside SQLite FTS5, threshold | 57 / 210 (p < 0.001) | 15 / 38 (p = 0.002) |
| GEL RAM, precise setting beside Tantivy BM25, strict threshold | 58 / 138 (p < 0.001) | 8 / 5 (p = 0.581) |
| GEL RAM, precise setting beside SQLite FTS5, strict threshold | 59 / 141 (p < 0.001) | 8 / 8 (p = 1.000) |
<!-- ANSWER-BENCH-RESULTS-END -->

After the review, every verdict is the same under the strict rules as under the published ones,
except in the always-top-1 runs of both engines, where one more answer counts as wrong.

- **GEL RAM answered 423 of the 985 questions: 405 correct and 18 wrong.** Its precision was
  95.7% (95% Wilson interval 93.4–97.3%), close to the 95.6% it had on v1 + v2, where the
  setting was chosen.
  Wrong answers were 1.8% of all questions.
- **The change raised GEL RAM from 172 to 405 correct answers, with 17 and 18 wrong.**
  - Question by question, the new build answered correctly 240 questions the v3 build did not,
    and the v3 build 7 that the new build did not (p < 0.001).
  - Each build answered wrongly questions the other did not: 13 and 12 (p = 1.0).
- **Beside the search engines at the same rule (precision at least 0.95 on v1 + v2)**, neither
  side is better on both counts:
  - the engines answered 571 and 599 questions: 531 and 558 correct, 40 and 41 wrong;
  - question by question, the engines answered correctly 192 and 210 questions that GEL RAM did
    not, and GEL RAM 66 and 57 that they did not (p < 0.001): **the engines find more answers**;
  - the engines answered wrongly 37 and 38 questions where GEL RAM did not, and GEL RAM 15 where
    they did not (p = 0.003 and 0.002): **GEL RAM gives about half as many wrong answers**.
- **At the precise setting GEL RAM did not keep the 0.99 it had on v1 + v2.**
  - It gave 281 correct and 8 wrong answers, a precision of 97.2% (94.6–98.6%).
  - The engines at their strict thresholds gave 361 and 363 correct, with 5 and 8 wrong.
  - They found more answers (p < 0.001), and the numbers of wrong answers do not differ beyond
    chance.
  - At this setting the engines are ahead.
- **The titles help the engines as much as GEL RAM.** On the v3 bank, without titles, the same
  engines gave 220 and 215 correct and 28 wrong answers each.
- **The review covered every answered pair of the recorded runs, 1592 distinct pairs.** These
  include two runs not published here: both engines always answering on the v3 bank.
  - 48 changes were proposed and each was checked against its row; 47 were accepted (46 to
    WRONG, 1 to CORRECT), and 1 was rejected as doubtful.
  - The 43 lines of [review.txt](recorded/with-answer/review.txt) apply the accepted changes to
    the published runs.
  - None of GEL RAM's answers at either setting was changed.

Times were not measured on this set; see [v3](../answer-or-abstain-v3/README.md#time) for the
previous build.

## Limits

- **Who wrote the set and did the review:** the project's AI coding assistant wrote the
  questions and accepted spellings, and did the review. The review was blind to the systems, in
  ten separate parts. This is not an independent benchmark.
- **What can be re-run:** GEL's answers come from a development build of the separate private
  implementation, which is not part of a release. Nothing here can be re-run from this checkout,
  because the bank of 671,416 passages is not published.
- **The engines:** they ran with default settings, and a tuned setup may do better.
- **One run:** each system ran once on this set.
- **Text, not truth:** the scorer checks text. A returned passage is long and may contain an
  accepted spelling by chance; the review found such pairs and may have missed others.

## Attribution

Source passages and expected answers quote Wikipedia, available under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions paraphrase it.
The answers are passages of the same bank, reproduced as returned. Third-party material keeps
its own terms, as described in [LICENSING](../../LICENSING.md).
