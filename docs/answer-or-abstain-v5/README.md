# Answer-or-abstain set v5

`answer_or_abstain_v5` · SHA-256 `342c2558a1a17db06b2853949b1207e33c22d23c614c0826466ca06dc9da30fc`

987 new questions (491 Polish, 496 English), each about a fact stated in one Wikipedia passage of
the GEL bank, and the answers of:

- GEL RAM, the development build measured in [v4](../answer-or-abstain-v4/README.md), at both of
  its settings, unchanged;
- the same build with one candidate change, at two settings chosen the same way;
- Tantivy (BM25) and SQLite FTS5 searching the same passages, at two thresholds and always
  answering.

All answers were recorded on 2026-10-03. Every system answers with one stored passage or with
`UNKNOWN`.

**This set is held out for every system it measures.** The build, the candidate change and every
setting were fixed on v1 + v2 before this set was drawn and written. The set was frozen by
SHA-256 before any system ran on it, and each system ran once.

**Status: public diagnostic.** Once published, these questions are no longer held out; the
passages for the next clean check were drawn and frozen privately before this set was
published, and their questions will be frozen before any run. The identity
above is the SHA-256 of a list of the hashes of every data file of the set; `answer-bench check`
recomputes it.

## Two questions fixed before the run

1. **Does the v4 result hold on new questions?** The same build at the same settings.
2. **Does the candidate change keep its precision?** On v1 + v2 it gave more correct answers than
   the build without it (256 against 196) with 13 wrong against 9. The rule fixed in advance:
   keep the change only if its precision at the balanced setting stays at least 0.95 on this
   set.

## How the runs were made

One selection rule for every system, applied on v1 + v2 (474 questions) before this set
existed:

> Choose the setting with the most correct answers at a precision of at least p (0.95 for the
> balanced setting and the plain threshold, 0.99 for the precise setting and the strict
> threshold); on a tie, choose the stricter setting.

| Recorded answers | System | Setting |
|---|---|---|
| `gel-ram` | GEL RAM, development build of 2026-09-30, as in v4 | balanced (p = 0.95), as in v4 |
| `gel-ram-precise` | the same build and run | precise (p = 0.99), as in v4 |
| `gel-ram-candidate` | the same build with one candidate change; its details remain private | balanced (p = 0.95) |
| `gel-ram-candidate-precise` | the same candidate and run | precise (p = 0.99) |
| `tantivy-bm25`, `-strict`, `-top1` | Tantivy 0.22.1, default tokenizer, query = question words joined by OR | lead of the best passage over the second, (s1 − s2) / s1, above 0.110 / 0.225 / always answer |
| `sqlite-fts5`, `-strict`, `-top1` | SQLite FTS5 (bm25), same query | 0.100 / 0.225 / always answer |

The engines search the passages together with their article title and section heading, as in
v4. The development build is not part of a release, and its answers cannot be re-run from this
checkout. The engines use default settings without tuning: they are a yardstick, not their best
possible result.

**The set:**
- **Draw:** 1000 passages drawn by a fixed seed from the bank, with the same 150–600 character
  filter as before. They are disjoint from the 2,600 passages of every earlier set and from the
  2,001 passages used in development checks.
- **Questions:** written by the project's AI coding assistant in eight separate parts. Each
  writer saw its passages and their article titles, and no system's answers.
- **Passages without an honest question:** 13 were marked unaskable with a reason before any
  run: navigation bars, a repeated article title, general sentences without a fact, and tables
  cut off without their headers or values. That leaves 987 questions.
- **Fixes before the freeze:** a checking program and the writers flagged four questions. Two
  were rewritten to ask about a fact the passage states, one accepted spelling was added for
  words run together in the passage, and one question was kept as written; its risk of a chance
  match was left to the manual review.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| `recorded/with-answer/<system>.txt` | answers recorded on 2026-10-03, one file per system; each file states its system and setting in its first line |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments. The scoring rules are
the same as for [v1](../answer-or-abstain/README.md), [v2](../answer-or-abstain-v2/README.md),
[v3](../answer-or-abstain-v3/README.md) and [v4](../answer-or-abstain-v4/README.md).

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`. Write one tab-separated line per
question: `nr`, a tab, then the answer, for numbers 1 to 987. Then run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v5:with-answer my-answers.txt
```

## Recorded results

The tables show results after the manual review; automatic scores are in brackets where they
differ. The block is generated from the recorded answers (`answer-bench tables v5`).
`answer-bench check`, which `xtask verify` runs, fails if the block shows other numbers.

The last table compares pairs of recorded runs question by question. It counts the questions
that only one of the two answered correctly, and those that only one answered wrongly. p is the
two-sided exact sign test (McNemar's exact test) on those questions.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 432 | 416 (420) | 16 (12) | 555 | 0 |
| GEL RAM | strict | 432 | 416 (420) | 16 (12) | 555 | 0 |
| GEL RAM, precise setting | published | 298 | 297 | 1 | 689 | 0 |
| GEL RAM, precise setting | strict | 298 | 297 | 1 | 689 | 0 |
| GEL RAM, candidate change | published | 572 | 528 (533) | 44 (39) | 415 | 0 |
| GEL RAM, candidate change | strict | 572 | 528 (533) | 44 (39) | 415 | 0 |
| GEL RAM, candidate change, precise setting | published | 275 | 273 | 2 | 712 | 0 |
| GEL RAM, candidate change, precise setting | strict | 275 | 273 | 2 | 712 | 0 |
| Tantivy BM25, threshold | published | 610 | 567 (571) | 43 (39) | 377 | 0 |
| Tantivy BM25, threshold | strict | 610 | 567 (571) | 43 (39) | 377 | 0 |
| Tantivy BM25, strict threshold | published | 381 | 378 | 3 | 606 | 0 |
| Tantivy BM25, strict threshold | strict | 381 | 378 | 3 | 606 | 0 |
| Tantivy BM25, always top 1 | published | 987 | 776 (796) | 211 (191) | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 987 | 776 (796) | 211 (191) | 0 | 0 |
| SQLite FTS5, threshold | published | 644 | 597 (603) | 47 (41) | 343 | 0 |
| SQLite FTS5, threshold | strict | 644 | 597 (603) | 47 (41) | 343 | 0 |
| SQLite FTS5, strict threshold | published | 382 | 379 | 3 | 605 | 0 |
| SQLite FTS5, strict threshold | strict | 382 | 379 | 3 | 605 | 0 |
| SQLite FTS5, always top 1 | published | 987 | 786 (807) | 201 (180) | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 987 | 786 (807) | 201 (180) | 0 | 0 |

| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |
|---|---|---|---:|---:|
| GEL RAM | 96.3% (94.1–97.7%) | 1.6% (1.0–2.6%) | 255 / 262 | 161 / 170 |
| GEL RAM, precise setting | 99.7% (98.1–99.9%) | 0.1% (0.0–0.6%) | 187 / 188 | 110 / 110 |
| GEL RAM, candidate change | 92.3% (89.8–94.2%) | 4.5% (3.3–5.9%) | 275 / 298 | 253 / 274 |
| GEL RAM, candidate change, precise setting | 99.3% (97.4–99.8%) | 0.2% (0.1–0.7%) | 163 / 165 | 110 / 110 |
| Tantivy BM25, threshold | 93.0% (90.6–94.7%) | 4.4% (3.3–5.8%) | 328 / 364 | 239 / 246 |
| Tantivy BM25, strict threshold | 99.2% (97.7–99.7%) | 0.3% (0.1–0.9%) | 246 / 249 | 132 / 132 |
| Tantivy BM25, always top 1 | 78.6% (76.0–81.1%) | 21.4% (18.9–24.0%) | 386 / 491 | 390 / 496 |
| SQLite FTS5, threshold | 92.7% (90.4–94.5%) | 4.8% (3.6–6.3%) | 338 / 374 | 259 / 270 |
| SQLite FTS5, strict threshold | 99.2% (97.7–99.7%) | 0.3% (0.1–0.9%) | 244 / 247 | 135 / 135 |
| SQLite FTS5, always top 1 | 79.6% (77.0–82.0%) | 20.4% (18.0–23.0%) | 392 / 491 | 394 / 496 |

| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |
|---|---:|---:|
| GEL RAM, candidate change beside GEL RAM | 144 / 32 (p < 0.001) | 39 / 11 (p < 0.001) |
| GEL RAM, candidate change, precise setting beside GEL RAM, precise setting | 39 / 63 (p = 0.022) | 2 / 1 (p = 1.000) |
| GEL RAM beside Tantivy BM25, threshold | 47 / 198 (p < 0.001) | 15 / 42 (p < 0.001) |
| GEL RAM beside SQLite FTS5, threshold | 39 / 220 (p < 0.001) | 14 / 45 (p < 0.001) |
| GEL RAM, precise setting beside Tantivy BM25, strict threshold | 67 / 148 (p < 0.001) | 1 / 3 (p = 0.625) |
| GEL RAM, precise setting beside SQLite FTS5, strict threshold | 67 / 149 (p < 0.001) | 1 / 3 (p = 0.625) |
<!-- ANSWER-BENCH-RESULTS-END -->

After the review, every verdict is the same under the strict rules as under the published ones.

- **The v4 result held.** GEL RAM answered 432 of the 987 questions: 416 correct and 16 wrong.
  Its precision was 96.3% (95% Wilson interval 94.1–97.7%), after 95.6% on v1 + v2, where the
  setting was chosen, and 95.7% on v4. Wrong answers were 1.6% of all questions.
- **Beside the search engines at the same rule (precision at least 0.95 on v1 + v2)**, the
  picture of v4 repeats:
  - the engines answered 610 and 644 questions: 567 and 597 correct, 43 and 47 wrong;
  - question by question, the engines answered correctly 198 and 220 questions that GEL RAM did
    not, and GEL RAM 47 and 39 that they did not (p < 0.001): **the engines find more answers**;
  - the engines answered wrongly 42 and 45 questions where GEL RAM did not, and GEL RAM 15 and
    14 where they did not (p < 0.001): **GEL RAM gives about a third as many wrong answers**.
- **At the precise setting GEL RAM gave 297 correct and 1 wrong answer**, a precision of 99.7%
  (98.1–99.9%). On v4 the same setting gave 97.2%, so the two sets together do not establish a
  precision of 0.99.
  - The engines at their strict thresholds gave 378 and 379 correct, with 3 wrong each.
  - They found more answers (p < 0.001); the wrong answers, 1 against 3, do not differ beyond
    chance (p = 0.625).
- **The candidate change was rolled back.**
  - It gave more correct answers than the build without it: 144 questions only it answered
    correctly, 32 only the build without it (p < 0.001).
  - It also gave more wrong ones: 39 against 11 (p < 0.001).
  - Its precision fell from 95.2% on v1 + v2 to 92.3% (89.8–94.2%), below the 0.95 fixed in
    advance, and its wrong answers, 44, are as many as the engines' at the same rule.
  - At the precise setting it gave fewer correct answers than the build without it: 39 against
    63 (p = 0.022).
- **The review covered every answered pair of the recorded runs, 1094 distinct pairs.**
  - 26 changes were proposed and each was checked against its row; all 26 were accepted (25 to
    WRONG, 1 to CORRECT).
  - The 64 lines of [review.txt](recorded/with-answer/review.txt) apply them to the published
    runs.
  - At the balanced setting 4 of GEL RAM's answers and 5 of the candidate's were changed to
    WRONG; none at either precise setting.

Times were not measured on this set.

## Limits

- **Who wrote the set and did the review:** the project's AI coding assistant wrote the
  questions and accepted spellings, and did the review. The review was blind to the systems, in
  ten separate parts. This is not an independent benchmark.
- **What can be re-run:** GEL's answers come from a development build of the separate private
  implementation, which is not part of a release. Nothing here can be re-run from this checkout,
  because the bank of 671,416 passages is not published.
- **The candidate change** is described only by its result; its details remain private.
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
