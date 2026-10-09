# Answer-or-abstain set v3

`answer_or_abstain_v3` · SHA-256 `7b4ef03c8a8832e56c142facea73e222ac6f5b334820169c54704b238c68e3a8`

979 questions (488 Polish, 491 English), each about a fact stated in one
Wikipedia passage of the GEL bank, and the answers that GEL RAM and two
standard full-text search engines, Tantivy (BM25) and SQLite FTS5, gave from
that same bank of 671,416 passages, recorded on 2026-09-30. Every system
answers with one stored passage or with `UNKNOWN`. The set therefore measures
two things: whether a system finds the passage that states the fact, and
whether it abstains when it has not found it.

**Status: public diagnostic.** Once published, these questions are no longer
held out. For the final GEL build this set was already a re-test after a fix;
see [How the runs were made](#how-the-runs-were-made). The identity above is
the SHA-256 of a list of the hashes of every data file of the set;
`answer-bench check` recomputes it.

## What changed from v2

- **Two and a half times larger**: 1000 passages drawn by a fixed seed from
  the bank, disjoint from the 600 passages of every earlier set. The same filter
  as in v1 and v2 keeps passages of 150–600 characters (57.5% of the Polish and
  61.6% of the English passages).
- **Passages without an honest question are counted**: 21 of the 1000 (12
  Polish, 9 English) were marked unaskable, each with a reason, before any
  system ran. These were navigation bars, bare formulas, captions and footnotes,
  and tables whose cells were run together in extraction so that no answer
  could be matched as a whole word. That leaves 979 questions.
- **Search engines instead of language models**: every system reads the same
  bank. The comparison is between retrieval systems under one rule, not between
  a bank and a model's memory.
- **More of the analysis is computed by the checker**: precision and wrong
  answers with 95% Wilson intervals, the split by language, and a
  question-by-question comparison with an exact test are part of the generated
  block below.

## How the runs were made

The questions, the accepted spellings, the measurement plan and one selection
rule for every system were fixed by SHA-256 on 2026-09-29, before any system
ran on this set. The rule:

> On the development questions (v1 and v2, 474 questions), choose the setting
> with the most correct answers at a precision of at least 0.95; on a tie,
> choose the stricter setting.

- **GEL RAM** chose its settings by this rule. Each choice was written down
  before that build's run on this set.
- **Tantivy 0.22.1 and SQLite FTS5** index the same 671,416 passages with their
  default tokenizers. The query is the words of the question joined by OR.
  Each engine answers with its best passage when the lead of that passage over
  the second, (s1 − s2) / s1, exceeds a threshold. The threshold is the smallest
  one with a precision of at least 0.95 on v1 + v2: 0.230 for Tantivy and 0.235
  for FTS5. The rows marked `always top 1` show the same engines answering every
  question with their best passage. The engines use default settings, without
  tuning: they are a yardstick, not their best possible result.

Four GEL runs were made on this set, and all four are published:

| Recorded answers | Build | Settings fixed |
|---|---|---|
| `gel-ram-first-run` | the build planned for this measurement | on v1 + v2, before its run |
| `gel-ram-v2-build` | the build that produced the published v1 and v2 results; the plan named it as the reference point | as for v1 and v2 |
| `gel-ram-prototype` | a prototype of a change to how GEL finds the passage | on v1 + v2, before its run |
| `gel-ram` | the final build of that change, 2026-09-30 | on v1 + v2, before its run |

**The change was made because the first run on this set was weak**: it
answered 62 questions. The change and all of its settings were developed and
chosen on v1 + v2 only, and nothing was fitted to the answers recorded on this
set. The decision to change GEL, however, and part of the diagnosis behind it,
came from the first run on this set. For `gel-ram-prototype` and `gel-ram`
this set is therefore a re-test after a fix, not an untouched held-out set.
Only a new frozen set can give a clean held-out check of the final build.
[v4](../answer-or-abstain-v4/README.md) records this build again on new questions (`gel-ram-v3-build`).

For diagnosis only, the rankings of the first run and of the prototype were
also read at other thresholds, including none. No setting was chosen from
them.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| `recorded/with-answer/<system>.txt` | answers recorded on 2026-09-30: `gel-ram`, `gel-ram-first-run`, `gel-ram-v2-build`, `gel-ram-prototype`, `tantivy-bm25`, `tantivy-bm25-top1`, `sqlite-fts5`, `sqlite-fts5-top1`; each file states its system and settings in its first line |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments. The
scoring rules are the same as for [v1](../answer-or-abstain/README.md) and
[v2](../answer-or-abstain-v2/README.md).

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`. Write one
tab-separated line per question: `nr`, a tab, then the answer, for numbers 1
to 979. Then run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v3:with-answer my-answers.txt
```

## Recorded results

The tables show results after the manual review; automatic scores are in
brackets where they differ. The block is generated from the recorded answers
(`answer-bench tables v3`). `answer-bench check`, which `xtask verify` runs,
fails if the block shows other numbers.

The last table compares the recorded answers question by question. It counts
the questions that only one of two systems answered correctly, and the
questions that only one of them answered wrongly. p is the two-sided exact
sign test (McNemar's exact test) on those questions.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 191 | 180 (181) | 11 (10) | 788 | 0 |
| GEL RAM | strict | 191 | 180 (181) | 11 (10) | 788 | 0 |
| GEL RAM, first run | published | 62 | 58 (59) | 4 (3) | 917 | 0 |
| GEL RAM, first run | strict | 62 | 58 (59) | 4 (3) | 917 | 0 |
| GEL RAM, v1/v2 build | published | 163 | 154 | 9 | 816 | 0 |
| GEL RAM, v1/v2 build | strict | 163 | 154 | 9 | 816 | 0 |
| GEL RAM, prototype | published | 187 | 176 (177) | 11 (10) | 792 | 0 |
| GEL RAM, prototype | strict | 187 | 176 (177) | 11 (10) | 792 | 0 |
| Tantivy BM25, threshold | published | 266 | 238 (241) | 28 (25) | 713 | 0 |
| Tantivy BM25, threshold | strict | 266 | 238 (241) | 28 (25) | 713 | 0 |
| Tantivy BM25, always top 1 | published | 979 | 454 (483) | 525 (496) | 0 | 0 |
| Tantivy BM25, always top 1 | strict | 979 | 454 (482) | 525 (497) | 0 | 0 |
| SQLite FTS5, threshold | published | 269 | 241 (245) | 28 (24) | 710 | 0 |
| SQLite FTS5, threshold | strict | 269 | 241 (245) | 28 (24) | 710 | 0 |
| SQLite FTS5, always top 1 | published | 979 | 459 (487) | 520 (492) | 0 | 0 |
| SQLite FTS5, always top 1 | strict | 979 | 459 (487) | 520 (492) | 0 | 0 |

| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |
|---|---|---|---:|---:|
| GEL RAM | 94.2% (90.0–96.8%) | 1.1% (0.6–2.0%) | 97 / 101 | 83 / 90 |
| GEL RAM, first run | 93.5% (84.6–97.5%) | 0.4% (0.2–1.0%) | 27 / 29 | 31 / 33 |
| GEL RAM, v1/v2 build | 94.5% (89.8–97.1%) | 0.9% (0.5–1.7%) | 83 / 88 | 71 / 75 |
| GEL RAM, prototype | 94.1% (89.8–96.7%) | 1.1% (0.6–2.0%) | 95 / 99 | 81 / 88 |
| Tantivy BM25, threshold | 89.5% (85.2–92.6%) | 2.9% (2.0–4.1%) | 148 / 170 | 90 / 96 |
| Tantivy BM25, always top 1 | 46.4% (43.3–49.5%) | 53.6% (50.5–56.7%) | 244 / 488 | 210 / 491 |
| SQLite FTS5, threshold | 89.6% (85.4–92.7%) | 2.9% (2.0–4.1%) | 153 / 174 | 88 / 95 |
| SQLite FTS5, always top 1 | 46.9% (43.8–50.0%) | 53.1% (50.0–56.2%) | 246 / 488 | 213 / 491 |

| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |
|---|---:|---:|
| GEL RAM beside GEL RAM, first run | 131 / 9 (p < 0.001) | 9 / 2 (p = 0.065) |
| GEL RAM beside Tantivy BM25, threshold | 37 / 95 (p < 0.001) | 9 / 26 (p = 0.006) |
| GEL RAM beside SQLite FTS5, threshold | 36 / 97 (p < 0.001) | 9 / 26 (p = 0.006) |
<!-- ANSWER-BENCH-RESULTS-END -->

After the review, every verdict is the same under the strict rules as under
the published ones; before it, one automatic verdict of Tantivy `always top 1`
differs.

- **GEL RAM answered 191 of the 979 questions (19.5%): 180 correct and 11
  wrong.** Its precision was 94.2% (95% Wilson interval 90.0–96.8%), and wrong
  answers were 1.1% of all questions.
- **The search engines with their thresholds answered 266 and 269 questions:
  238 and 241 correct, 28 wrong each.** Their precision was 89.5% and 89.6%.
  On v1 + v2 all three systems had a precision of at least 0.95.
- **Neither side is better on both counts.**
  - Question by question, the engines answered correctly 95 and 97 questions
    that GEL RAM did not, and GEL RAM answered correctly 36 and 37 that they did
    not (p < 0.001). **The engines find more answers.**
  - The engines answered wrongly 26 questions where GEL RAM did not, and GEL
    RAM answered wrongly 9 where they did not (p = 0.006). **GEL RAM gives
    fewer wrong answers.**
- **When they answer every question**, the engines give 454 and 459 correct
  answers and 520 and 525 wrong ones.
- **The change raised GEL RAM from 58 correct answers to 180** at about the same
  precision (93.5% in the first run, 94.2% now). Question by question the count
  is 131 against 9 (p < 0.001). Its wrong answers rose from 4 to 11 (9 against
  2, p = 0.065). The v1/v2 build gave 154 correct answers and 9 wrong ones.
- **By language**, GEL RAM got 97 of its 101 Polish answers right and 83 of its
  90 English answers.
- **In each of GEL RAM's 11 wrong answers, the returned passage does not state
  the asked fact.**
  - Two of these passages hold only the article title.
  - One holds only photo captions.
  - Eight are about the same or a neighbouring subject but give another fact:
    another unit, another cartridge, another match, another entry in a list.
- **The review changed 31 question–answer pairs on 30 questions, all to
  WRONG.** These are the 67 lines of
  [review.txt](recorded/with-answer/review.txt), one for each system that
  returned that passage.
  - In each changed pair, the passage contained an accepted spelling by chance
    (another person, season or object).
  - No answer was changed to CORRECT.
  - GEL RAM had one change (question 65).
  - All 1055 distinct answered pairs, out of 3096 answers, were reviewed.

## Time

Wall-clock time per question on one machine
([measurement host](../HARDWARE.md)). Each system ran all 979 questions three
times. The table gives nearest-rank percentiles, as the range over the three
runs. These times are not re-checked by `xtask verify`.

| System | How it was asked | p50 | p95 | p99 | Max |
|---|---|---:|---:|---:|---:|
| GEL RAM | one question at a time, from a separate client process: from handing over the question to reading the answer | 8.8–8.9 ms | 9.8–10.1 ms | 10.1–10.6 ms | 10.5–11.7 ms |
| Tantivy BM25 | one query at a time, one thread, inside the measuring program; ten best passages | 2.1–2.7 ms | 6.9–9.1 ms | 11.8–14.8 ms | 20.6–25.1 ms |
| SQLite FTS5 | one query at a time, one thread, inside the measuring program; ten best passages | 218–252 ms | 368–424 ms | 462–535 ms | 538–742 ms |

With 256 questions handed over at once, GEL RAM took 0.67–0.73 s for all 979
questions, or 0.69–0.75 ms per question. The engines were run only one query at
a time.

Tantivy's median time is lower than GEL RAM's. GEL RAM's slowest questions are
faster: its p99 is 10.1–10.6 ms, against 11.8–14.8 ms for Tantivy. Memory and
disk use were not measured on the same terms and are not compared here.

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
thresholds the engines also return UNKNOWN.

## Limits

- The questions and accepted spellings were written by the project's AI coding
  assistant. It saw each passage and its article title, as in v2, but no
  system's answers. This is not an independent benchmark.
- The same assistant did the review, blind to the systems, in seven separate
  parts. Every proposed change was checked against the source passage, and the
  reasons are listed.
- For the final GEL build this set is a re-test after a fix; see above.
- GEL's answers come from the separate private implementation. The final build
  is a development build of 2026-09-30 and not yet part of a release.
- Nothing here can be re-run from this checkout, because the bank of 671,416
  passages is not published. The dumps it comes from are named in the question
  file.
- The engines ran with default settings. A tuned setup, with language-specific
  stemming or other query forms, may do better.
- Each system's answers come from one run; only the times come from three.
- The scorer checks text, not truth. A returned passage is long and may contain
  an accepted spelling by chance. The review found 31 such pairs and may have
  missed others.

## Attribution

Source passages and expected answers quote Wikipedia, available under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions
paraphrase it. The engines' and GEL's answers are passages of the same bank,
reproduced as returned. Third-party material keeps its own terms, as described
in [LICENSING](../../LICENSING.md).
