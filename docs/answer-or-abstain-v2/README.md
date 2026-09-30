# Answer-or-abstain set v2

`answer_or_abstain_v2` · SHA-256 `c782b45054da7c4fa1648d2f746f643cef523323f47eaabd88042b35fe514ab9`

> **Private measurement, public re-scoring.** GEL RAM answered these questions
> once, on the separate private implementation; this repository cannot run GEL on
> them. You can re-score every recorded answer (`xtask answer-bench check`) and
> score your own run of the models' side (`answer-bench score v2:with-answer FILE`).

394 questions (198 Polish, 196 English), each about a fact stated in one
Wikipedia passage of the GEL bank, and the answers GEL RAM and three language
models gave in one recorded run on 2026-09-29. This is the new frozen set the
[v1 set](../answer-or-abstain/README.md) announced: nothing in GEL was tuned on
it before the run. The next set, [v3](../answer-or-abstain-v3/README.md), puts GEL
beside two BM25 search engines on the same bank.

**What changed from v1**

- **Five times larger and drawn at random**: 400 passages drawn by a fixed seed
  from the bank, disjoint from every earlier set (the 80 test and 80 calibration
  questions and the 40 false-premise passages). The draw keeps passages of
  150–600 characters, as v1 did, so that one question can ask about one fact and
  the passage can be shown whole; that filter keeps 57.5% of the Polish and
  61.6% of the English passages.
- **Questions name their subject**: the writer saw the article title with each
  passage, so a question says which person, film or place it is about.
- **Accepted spellings were written before GEL ran** (in v1 after GEL's test).
- **Passages without an honest question are listed, not dropped silently**: 6
  of the 400 were marked unaskable with a reason (a navigation bar, a bare
  table legend, a table whose cells were run together in extraction), leaving
  394 questions.

**Status: public diagnostic.** Once published, these questions are no longer
held out; tuning on them and then scoring on them proves nothing about new
questions. The identity above is the SHA-256 of a list of the hashes of every
data file of the set; `answer-bench check` recomputes it.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| `recorded/with-answer/<system>.txt` | answers recorded in one run on 2026-09-29: `gel-ram`, `gpt-oss-120b`, `gpt-oss-20b`, `qwen3.8-27b` |
| [recorded/with-answer/review.txt](recorded/with-answer/review.txt) | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments. The
provenance, prompts, scoring rules and the way we compare are the same as for
[v1](../answer-or-abstain/README.md): the same prompts, word for word, the same
three models with the same settings, one run, and the same two rule sets.

## Try your own system

Record one short answer per question, or exactly `UNKNOWN`, one tab-separated
line per question (`nr`, a tab, the answer; numbers 1 to 394), then:

```text
cargo run --locked --offline -p xtask -- answer-bench score v2:with-answer my-answers.txt
```

## Recorded results

After the manual review; automatic scores in brackets where they differ. The
table is generated from the recorded answers (`answer-bench tables v2`), and
`answer-bench check`, run by `xtask verify`, fails if it shows other numbers.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 63 | 59 | 4 | 331 | 0 |
| GEL RAM | strict | 63 | 59 | 4 | 331 | 0 |
| GPT-OSS-120B | published | 223 | 92 (81) | 131 (142) | 171 | 0 |
| GPT-OSS-120B | strict | 223 | 92 (81) | 131 (142) | 171 | 0 |
| GPT-OSS-20B | published | 283 | 50 (39) | 233 (244) | 111 | 0 |
| GPT-OSS-20B | strict | 283 | 50 (39) | 233 (244) | 111 | 0 |
| Qwen3.8-27B | published | 123 | 43 (38) | 80 (85) | 271 | 0 |
| Qwen3.8-27B | strict | 123 | 43 (38) | 80 (85) | 271 | 0 |
<!-- ANSWER-BENCH-RESULTS-END -->

Under the strict rules every recorded verdict is the same as under the
published ones.

- **GEL RAM answered 63 of the 394 questions (16%)**: 59 correct and **4
  wrong**, a precision of 93.7% (95% Wilson interval 84.8–97.5%); wrong
  answers were 1.0% of all questions (0.4–2.6%). In all four wrong answers it
  returned a passage from another article — a disambiguation page in three,
  another royal couple in one — and all four were Polish questions; in English
  it gave 29 answers, all correct. 58 of its 63 answers are the question's own
  source passage.
- **The three models answered 31–72% of the questions**, with a precision of
  17.7–41.3%; 20–59% of all questions got a wrong answer from them (80, 131 and
  233 wrong answers).
- Review: 31 changes, listed with reasons in
  [review.txt](recorded/with-answer/review.txt); one proposed change (a unit
  conversion, 627 hp for 635 metric horsepower) was rejected as not the same
  number written differently.

## Limits

- The questions and accepted spellings were written by the project's AI coding
  assistant for passages in the GEL bank, as in v1. This is not an independent
  benchmark, and the set favours a system that holds those passages.
- GEL's answers come from the separate private implementation, with the same
  configuration as for v1; they cannot be re-run from this checkout. The
  models' side can: the questions and prompts are public.
- The length filter changes which passages a question can come from; it is
  stated above with its effect.
- The scorer checks text, not truth. Accepted spellings are generous for lists
  and strict for wording.

## Attribution

Source passages and expected answers quote Wikipedia, available under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions
paraphrase it. Model answers are reproduced as returned. Third-party material
keeps its own terms, as described in [LICENSING](../../LICENSING.md).
