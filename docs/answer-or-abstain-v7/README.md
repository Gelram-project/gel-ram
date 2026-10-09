# Answer-or-abstain set v7

`answer_or_abstain_v7` · SHA-256 `f4ac0d328093455d0cd723f32f0dae7f328ec18df4ec934917e8f6a2d81d2f41`

1,581 new questions in two parts:

- **982 with an answer** (487 Polish, 495 English), each about a fact stated in one Wikipedia
  passage of the GEL bank;
- **599 without an answer in the bank** (299 Polish, 300 English):
  - 399 ask about real Wikipedia topics whose articles the bank does not hold;
  - 200 ask about subjects made up for this set.

The right reply to a question without an answer is `UNKNOWN`. This is the first set since
[v1](../answer-or-abstain/README.md) with such a part, and the first that asks about real topics
outside the bank.

The recorded answers are those of:

- GEL RAM, the development build measured in [v4](../answer-or-abstain-v4/README.md) to
  [v6](../answer-or-abstain-v6/README.md), at both of its settings, unchanged;
- the same build with one candidate change, at two settings; the change was rolled back after
  this run;
- Tantivy (BM25) and SQLite FTS5 searching the same passages, at two thresholds and always
  answering.

All answers were recorded on 2026-10-08. Every system answers with one stored passage or with
`UNKNOWN`.

**No system and no setting was chosen or tuned on this set.** The build and its two settings were
fixed on v1 + v2. The candidate change and its two settings were fixed the day before this set
was drawn. A prediction of its result was frozen after the draw, before the questions were frozen
and before any system ran. The questions were frozen by SHA-256 before any system ran on them,
and the answers of each system were recorded in one run. The set had not been used before those
runs.

**Status: public diagnostic.** Once published, these questions are no longer held out. The
identity above is the SHA-256 of a list of the hashes of every data file of the set;
`answer-bench check` recomputes it.

## The questions this set asks

1. **Does the result of v4 to v6 hold on a fourth new set?** The same build at the same settings.
2. **What does each system do when the answer is not in the bank?** A wrong answer to a question
   with an answer and any answer to a question without one are both counted as wrong.
3. **Is the candidate change an improvement?** Its prediction, frozen before the run: at its
   answering setting, more correct answers than GEL RAM at the balanced setting (two-sided exact
   sign test, p < 0.05), with no more wrong answers; at its precise setting, no more wrong answers
   than GEL RAM at the precise setting.

## How the runs were made

The build and the engines use one selection rule, applied on v1 + v2 (474 questions) before this
set existed:

> Choose the setting with the most correct answers at a precision of at least p (0.95 for the
> balanced setting and the plain threshold, 0.99 for the precise setting and the strict
> threshold); on a tie, choose the stricter setting.

The candidate change has two settings, an answering one and a precise one, both fixed before
this set was drawn. The change and how its settings were chosen remain private.

| Recorded answers | System | Setting |
|---|---|---|
| `gel-ram` | GEL RAM, development build of 2026-09-30, as in v4 to v6 | balanced (p = 0.95), as in v4 to v6 |
| `gel-ram-precise` | the same build | precise (p = 0.99), as in v4 to v6 |
| `gel-ram-candidate` | the same build with one candidate change (its details remain private) | answering, fixed before the draw |
| `gel-ram-candidate-precise` | the same, same run | precise, fixed before the draw |
| `tantivy-bm25`, `-strict`, `-top1` | Tantivy 0.22.1, default tokenizer, query = question words joined by OR | lead of the best passage over the second, (s1 − s2) / s1, above 0.110 / 0.225 / always answer |
| `sqlite-fts5`, `-strict`, `-top1` | SQLite FTS5 (bm25), same query | 0.100 / 0.225 / always answer |

The engines search the passages together with their article title and section heading, as in
v4 to v6. The development build is not part of a release, and its answers cannot be re-run from
this checkout. The engines use default settings without tuning: they are a yardstick, not their
best possible result.

## The set

- **With an answer:**
  - **Draw:** 1000 passages drawn by a fixed seed from the bank, 500 Polish and 500 English, with
    the same 150–600 character filter as before. The draw program and its seed were frozen before
    the draw, on 2026-10-08. That was after set v6 was published, unlike v6, whose passages were
    frozen before v5 was published.
  - **Disjoint:** the draw shares no passage with any earlier set or with any passage used in
    development checks; all of them were excluded before the draw.
  - **Unaskable passages:** 18 were marked unaskable with a reason before any run. They were
    navigation bars, repeated titles, references or link titles, a heading without content,
    bare lists that do not say what they list, and tables or tournament brackets run together
    without readable results. That leaves 982 questions.
- **Topics not in the bank:**
  - **Draw:** 400 passages drawn by a fixed seed from the same Wikipedia dumps as the bank, one
    per article, 200 Polish and 200 English, with the same length filter.
  - **The article is outside the bank:** its title is not the title of any bank article, and the
    title without a bracketed note does not occur as whole words in any bank passage.
  - **No reuse:** articles used in earlier internal checks were excluded.
  - **Questions:** each asks a fact its passage states, so the question is answerable from
    Wikipedia, but not from the bank. One passage was marked unaskable, which leaves 399.
- **Invented subjects:** 200 questions (100 Polish, 100 English) about made-up people, places,
  works and things, in natural wording. The names that make each subject made up occur as whole
  words in no passage of the bank.
- **Writers:** the project's AI coding assistant wrote the questions in 16 separate parts. Each
  writer saw only its passages and their article titles, never any system's answers. Each part
  was checked by the same program as before (format, accepted spellings present in the passage,
  checked names absent from the bank).

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| [no-answer-questions.txt](no-answer-questions.txt) | question, kind (`absent` or `invented`), why it has no answer, the names checked against the bank, and for a topic outside the bank the source fields of the passage the question was written from |
| `recorded/with-answer/<system>.txt`, `recorded/no-answer/<system>.txt` | answers recorded on 2026-10-08, one file per system and part; each file states its system and setting in its first line |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of the answers to questions with an answer, with reasons |
| [recorded/no-answer/review.txt](recorded/no-answer/review.txt) | empty: every answer to a question without an answer is a stored passage, counted as answered |

All files are tab-separated text; lines starting with `#` are comments. The scoring rules are
the same as for [v1](../answer-or-abstain/README.md) to [v6](../answer-or-abstain-v6/README.md).

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`. Write one tab-separated line per
question: `nr`, a tab, then the answer, for numbers 1 to 982 of the first part and 1 to 599 of
the second. Then run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v7:with-answer my-answers.txt
cargo run --locked --offline -p xtask -- answer-bench score v7:no-answer my-no-answers.txt
```

## Recorded results

The tables show results after the manual review; automatic scores are in brackets where they
differ. The block is generated from the recorded answers (`answer-bench tables v7`).
`answer-bench check`, which `xtask verify` runs, fails if the block shows other numbers.

The table of all 1,581 questions counts as wrong both a wrong answer to a question with an
answer and any answer to a question without one. The last table compares pairs of recorded runs
question by question on the 982 questions with an answer. It counts the questions that only one
of the two answered correctly, and those that only one answered wrongly. p is the two-sided
exact sign test (McNemar's exact test) on those questions.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 425 | 413 (415) | 12 (10) | 557 | 0 |
| GEL RAM | strict | 425 | 413 (414) | 12 (11) | 557 | 0 |
| GEL RAM, precise setting | published | 294 | 291 | 3 | 688 | 0 |
| GEL RAM, precise setting | strict | 294 | 291 | 3 | 688 | 0 |
| GEL RAM, candidate change | published | 248 | 245 | 3 | 734 | 0 |
| GEL RAM, candidate change | strict | 248 | 245 | 3 | 734 | 0 |
| GEL RAM, candidate change, precise setting | published | 177 | 176 | 1 | 805 | 0 |
| GEL RAM, candidate change, precise setting | strict | 177 | 176 | 1 | 805 | 0 |
| Tantivy BM25, threshold | published | 593 | 550 (551) | 43 (42) | 389 | 0 |
| Tantivy BM25, threshold | strict | 593 | 550 (551) | 43 (42) | 389 | 0 |
| Tantivy BM25, strict threshold | published | 389 | 380 | 9 | 593 | 0 |
| Tantivy BM25, strict threshold | strict | 389 | 380 | 9 | 593 | 0 |
| Tantivy BM25, always top 1 | published | 982 | 750 (764) | 232 (218) | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 982 | 750 (763) | 232 (219) | 0 | 0 |
| SQLite FTS5, threshold | published | 624 | 581 (583) | 43 (41) | 358 | 0 |
| SQLite FTS5, threshold | strict | 624 | 581 (583) | 43 (41) | 358 | 0 |
| SQLite FTS5, strict threshold | published | 393 | 384 | 9 | 589 | 0 |
| SQLite FTS5, strict threshold | strict | 393 | 384 | 9 | 589 | 0 |
| SQLite FTS5, always top 1 | published | 982 | 762 (775) | 220 (207) | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 982 | 762 (775) | 220 (207) | 0 | 0 |

| Without an answer | Rules | Answered anyway: topic not in the bank / invented | UNKNOWN | Rejected the premise | Needs review | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 18 / 8 | 573 | 0 | 0 | 0 |
| GEL RAM | strict | 18 / 8 | 573 | 0 | 0 | 0 |
| GEL RAM, precise setting | published | 2 / 1 | 596 | 0 | 0 | 0 |
| GEL RAM, precise setting | strict | 2 / 1 | 596 | 0 | 0 | 0 |
| GEL RAM, candidate change | published | 5 / 3 | 591 | 0 | 0 | 0 |
| GEL RAM, candidate change | strict | 5 / 3 | 591 | 0 | 0 | 0 |
| GEL RAM, candidate change, precise setting | published | 0 / 0 | 599 | 0 | 0 | 0 |
| GEL RAM, candidate change, precise setting | strict | 0 / 0 | 599 | 0 | 0 | 0 |
| Tantivy BM25, threshold | published | 87 / 33 | 479 | 0 | 0 | 0 |
| Tantivy BM25, threshold | strict | 87 / 33 | 479 | 0 | 0 | 0 |
| Tantivy BM25, strict threshold | published | 18 / 7 | 574 | 0 | 0 | 0 |
| Tantivy BM25, strict threshold | strict | 18 / 7 | 574 | 0 | 0 | 0 |
| Tantivy BM25, always top 1 | published | 399 / 200 | 0 | 0 | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 399 / 200 | 0 | 0 | 0 | 0 |
| SQLite FTS5, threshold | published | 105 / 39 | 455 | 0 | 0 | 0 |
| SQLite FTS5, threshold | strict | 105 / 39 | 455 | 0 | 0 | 0 |
| SQLite FTS5, strict threshold | published | 20 / 9 | 570 | 0 | 0 | 0 |
| SQLite FTS5, strict threshold | strict | 20 / 9 | 570 | 0 | 0 | 0 |
| SQLite FTS5, always top 1 | published | 399 / 200 | 0 | 0 | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 399 / 200 | 0 | 0 | 0 | 0 |

| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |
|---|---|---|---:|---:|
| GEL RAM | 97.2% (95.1–98.4%) | 1.2% (0.7–2.1%) | 238 / 247 | 175 / 178 |
| GEL RAM, precise setting | 99.0% (97.0–99.7%) | 0.3% (0.1–0.9%) | 170 / 173 | 121 / 121 |
| GEL RAM, candidate change | 98.8% (96.5–99.6%) | 0.3% (0.1–0.9%) | 152 / 154 | 93 / 94 |
| GEL RAM, candidate change, precise setting | 99.4% (96.9–99.9%) | 0.1% (0.0–0.6%) | 105 / 106 | 71 / 71 |
| Tantivy BM25, threshold | 92.7% (90.4–94.6%) | 4.4% (3.3–5.8%) | 312 / 343 | 238 / 250 |
| Tantivy BM25, strict threshold | 97.7% (95.7–98.8%) | 0.9% (0.5–1.7%) | 242 / 250 | 138 / 139 |
| Tantivy BM25, always top 1 | 76.4% (73.6–78.9%) | 23.6% (21.1–26.4%) | 370 / 487 | 380 / 495 |
| SQLite FTS5, threshold | 93.1% (90.8–94.8%) | 4.4% (3.3–5.8%) | 325 / 354 | 256 / 270 |
| SQLite FTS5, strict threshold | 97.7% (95.7–98.8%) | 0.9% (0.5–1.7%) | 247 / 255 | 137 / 138 |
| SQLite FTS5, always top 1 | 77.6% (74.9–80.1%) | 22.4% (19.9–25.1%) | 380 / 487 | 382 / 495 |

| All 1581 questions, published rules, after the review | Correct | Wrong: wrong answer + answered without an answer | UNKNOWN | Polish: correct / wrong | English: correct / wrong |
|---|---:|---:|---:|---:|---:|
| GEL RAM | 413 | 12 + 26 = 38 | 1130 | 238 / 26 | 175 / 12 |
| GEL RAM, precise setting | 291 | 3 + 3 = 6 | 1284 | 170 / 5 | 121 / 1 |
| GEL RAM, candidate change | 245 | 3 + 8 = 11 | 1325 | 152 / 6 | 93 / 5 |
| GEL RAM, candidate change, precise setting | 176 | 1 + 0 = 1 | 1404 | 105 / 1 | 71 / 0 |
| Tantivy BM25, threshold | 550 | 43 + 120 = 163 | 868 | 312 / 97 | 238 / 66 |
| Tantivy BM25, strict threshold | 380 | 9 + 25 = 34 | 1167 | 242 / 24 | 138 / 10 |
| Tantivy BM25, always top 1 | 750 | 232 + 599 = 831 | 0 | 370 / 416 | 380 / 415 |
| SQLite FTS5, threshold | 581 | 43 + 144 = 187 | 813 | 325 / 108 | 256 / 79 |
| SQLite FTS5, strict threshold | 384 | 9 + 29 = 38 | 1159 | 247 / 25 | 137 / 13 |
| SQLite FTS5, always top 1 | 762 | 220 + 599 = 819 | 0 | 380 / 406 | 382 / 413 |

| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |
|---|---:|---:|
| GEL RAM, candidate change beside GEL RAM | 7 / 175 (p < 0.001) | 3 / 12 (p = 0.035) |
| GEL RAM, candidate change, precise setting beside GEL RAM, precise setting | 10 / 125 (p < 0.001) | 1 / 3 (p = 0.625) |
| GEL RAM beside Tantivy BM25, strict threshold | 119 / 86 (p = 0.025) | 11 / 8 (p = 0.648) |
| GEL RAM beside SQLite FTS5, strict threshold | 117 / 88 (p = 0.050) | 11 / 8 (p = 0.648) |
| GEL RAM, precise setting beside Tantivy BM25, strict threshold | 51 / 140 (p < 0.001) | 3 / 9 (p = 0.146) |
| GEL RAM, precise setting beside SQLite FTS5, strict threshold | 51 / 144 (p < 0.001) | 3 / 9 (p = 0.146) |
<!-- ANSWER-BENCH-RESULTS-END -->

After the review, the verdicts of every GEL RAM run are the same under the strict rules as under
the published ones.

- **The result held a fourth time.** GEL RAM answered 425 of the 982 questions with an answer:
  413 correct and 12 wrong. Its precision was 97.2% (95% Wilson interval 95.1–98.4%). On the
  earlier sets it was 95.6% on v1 + v2, where the setting was chosen, 95.7% on v4, 96.3% on v5
  and 96.1% on v6.
- **Questions without an answer in the bank:** GEL RAM answered 18 of the 399 about topics
  outside the bank and 8 of the 200 about invented subjects, and said `UNKNOWN` to the other 573.
  - **All 1,581 questions:** 413 correct and 38 wrong (2.4%).
- **Beside the search engines at their strict thresholds**, which give about as many wrong answers
  in all (34 and 38):
  - the engines answered 380 and 384 questions correctly; GEL RAM answered 413;
  - question by question, GEL RAM answered correctly 119 and 117 questions that the engines did
    not, and the engines 86 and 88 that GEL RAM did not (p = 0.025 and p = 0.050);
  - in English GEL RAM gave 175 correct answers, the engines 138 and 137; in Polish 238, the
    engines 242 and 247.
  - At their plain thresholds the engines gave 550 and 581 correct answers, with 163 and 187
    wrong in all; they answered 87 and 105 of the questions about topics outside the bank.
- **At the precise setting GEL RAM gave 291 correct and 3 wrong answers**, a precision of 99.0%
  (97.0–99.7%). It answered 3 of the 599 questions without an answer: 6 wrong in all, 0.4% of all
  questions. On v4 to v6 the same setting gave 97.2%, 99.7% and 99.4%, so four sets together do
  not establish a precision of 0.99.
- **The candidate change was not an improvement, and it was rolled back.**
  - At its answering setting it gave 245 correct answers against 413 for GEL RAM at the balanced
    setting: 7 questions correct only for the candidate, 175 only for GEL RAM (p < 0.001). The
    prediction fails.
  - It did give fewer wrong answers: 11 in all against 38. At its precise setting it gave 176
    correct and 1 wrong answer, against 291 and 6 for GEL RAM at the precise setting.
  - On the questions without an answer it answered 8 of the 599 (1.3%) at its answering setting
    and none at its precise setting.
- **The review covered every answered pair of the recorded runs, 1,062 distinct pairs.**
  - Reviewers saw no system names. The pairs were reviewed blind in six parts.
  - The reviewers proposed 18 changes. Each was checked against its row and all were accepted:
    17 to WRONG, 1 to CORRECT.
  - The 34 lines of [review.txt](recorded/with-answer/review.txt) apply them to the recorded runs.
  - Two of GEL RAM's answers at the balanced setting were changed to WRONG. No answer of GEL RAM
    at the precise setting and none of the candidate's runs was changed.

Times were not measured on this set.

## What has nothing to compare with

The search engines on these pages are a reference for one narrow task, scored
the same way for every system: return one stored passage for a question, or
UNKNOWN. The question sets measure the answers of private builds; they do not
measure the record, the reader or the memory beneath them, and the parts below
have no counterpart in that task, so no number here compares them:

- **One record, four exact views.** One 1,152-byte Q8 record is read through
  four equivalent views without four copies; each view's score equals a
  separate reference read bit for bit, and each of the 48 recorded comparison
  runs behind the README record card ends with `Q8_QUAD_EXACT=PASS`
  ([contract](../Q8-QUAD.md), [the 48 runs](../evidence-q8-current/README.md)).
- **Reader16.** One fused comparison of two records returns 16 judgments; they
  are not 16 independent measurements ([Reader16](../READER16.md)).
- **F0 memory physics.** The cost of the memory the records live in is measured
  in nanoseconds and GiB/s instead of assumed ([method](../PERFORMANCE.md)).

Hardware-level memory computation, the direction of the project, is not
established by any run on these pages or in this repository
([claim registry](../CLAIMS.md)). UNKNOWN is not unique to GEL RAM: at their
thresholds the engines also return UNKNOWN, and at the strict threshold they
abstain on 574 and 570 of the 599 questions without an answer.

## Limits

- **Who wrote the set and did the review:** the project's AI coding assistant wrote the
  questions and accepted spellings, and did the review. The review was blind to the systems, in
  separate parts. This is not an independent benchmark.
- **What can be re-run:** GEL's answers come from a development build of the separate private
  implementation, which is not part of a release. Nothing here can be re-run from this checkout,
  because the bank of 671,416 passages is not published.
- **Topics outside the bank are checked by their titles.** A topic can still be in the bank under
  another word form or inside another article.
  - **Question 96 of the second part is such a case.** It asks which title Yad Vashem gave Natalia
    Likos in 1993. A bank passage states it: "Jad Waszem uhonorował rodzinę Likosów (Piotra,
    Apolonię i Natalię) tytułem Sprawiedliwy wśród Narodów Świata".
  - The candidate change and both engine runs at their strict thresholds returned that passage.
    By the rules it counts as an answer to a question without one, and it is left so.
- **Invented subjects are checked against the bank only.** The checked names occur in no bank
  passage, which does not prove that such a subject exists nowhere.
- **The questions** ask about randomly drawn passages, so many concern narrow facts. They test
  answering from the bank, not general knowledge or conversation.
- **The engines:** they ran with default settings, and a tuned setup may do better.
- **One recorded run:** the answers of each system were recorded in one run on this set. After
  the runs, the set was used privately to find out why the candidate change lost. That changed
  nothing recorded here.
- **Text, not truth:** the scorer checks text. A returned passage is long and may contain an
  accepted spelling by chance; the review found such pairs and may have missed others.

## Attribution

Source passages, the passages of topics outside the bank and expected answers quote Wikipedia,
available under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions
paraphrase it. The answers are passages of the same bank, reproduced as returned. Third-party
material keeps its own terms, as described in [LICENSING](../../LICENSING.md).
