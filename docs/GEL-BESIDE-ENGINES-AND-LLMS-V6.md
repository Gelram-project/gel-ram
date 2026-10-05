# GEL RAM beside two search engines and three language models — question set v6

> **Private measurement.** GEL RAM ran on the separate private implementation;
> this repository cannot run it. The search engines' and the models' answers are
> recorded, and the model side can be re-run by anyone with the prompts below
> (hosted models change, so the answers may differ).

400 questions of [question set v6](answer-or-abstain-v6/README.md) (200 Polish,
200 English), asked to three language models through the Groq API on
2026-10-05, beside the recorded answers of GEL RAM and of two BM25 search engines
on the same questions. A film shows 150 of them side by side; the claim registry
lists the result as `MEASURED_LOCAL`.

[![Each of the 150 questions shown, as one cell per system: green correct, red wrong, grey UNKNOWN.](../media/beside-v6/all-150-answers.png)](../media/beside-v6/GEL-BESIDE-ENGINES-AND-LLMS-V6-PART1-EN.mp4)

The film, 150 questions in two parts, 12 min 16 s in all:
[part 1, questions 1–75 and the limits, 5 min 45 s](../media/beside-v6/GEL-BESIDE-ENGINES-AND-LLMS-V6-PART1-EN.mp4) ·
[part 2, questions 76–150 and the summary, 6 min 31 s](../media/beside-v6/GEL-BESIDE-ENGINES-AND-LLMS-V6-PART2-EN.mp4) ·
[summary only, 1 min 28 s](../media/beside-v6/GEL-BESIDE-ENGINES-AND-LLMS-V6-SUMMARY-EN.mp4).
All are replays rendered from the recorded answer files, not screen captures;
the on-screen pacing is not execution time.

**What this shows.** Not which side knows more. GEL RAM and the search engines
answer from the same bank of 671,416 Wikipedia passages; the models answer from
training, closed book. Every question was written from a passage of this bank, so
the bank holds each answer by construction; the models were not given it. It
shows what each does when it cannot find or does not know the answer: say so, or
answer anyway.

## What was compared

- **The questions.** 400 of the 990 questions of set v6, drawn before any request
  by a fixed seed, 200 per language ([sample-400.txt](answer-or-abstain-v6/beside-llm/sample-400.txt)).
  The API's daily limits on the plan used were too low to ask all 990 questions in
  one run. The 150 questions in the film were drawn from the 400 by a second seed
  ([film-150.txt](answer-or-abstain-v6/beside-llm/film-150.txt)). None was picked
  by hand.
- **GEL RAM**, the development build measured in sets v4 to v6, at its balanced
  setting and its precise setting. It answers with one stored passage, quoted as
  stored, or says UNKNOWN. Its answers are those recorded for set v6.
- **Tantivy and SQLite FTS5 (BM25)** over the same bank, each at a threshold and a
  strict threshold, as recorded for set v6.
- **One rule for choosing settings**, on earlier questions: precision at least
  0.95 (GEL RAM balanced, the engines' threshold settings) or at least 0.99
  (GEL RAM precise, the engines' strict settings).
- **GPT-OSS-120B, GPT-OSS-20B and Qwen3.8-27B** (`openai/gpt-oss-120b`,
  `openai/gpt-oss-20b`, `qwen/qwen3.8-27b`) through the Groq API, **closed book**:
  no passages and no tools. Temperature 0, at most 4096 output tokens, reasoning
  effort `low` for the GPT-OSS models and the API default for Qwen3.8-27B, one
  question per request, 4 s apart. Two rate-limit replies (HTTP 429), both for
  Qwen3.8-27B, were retried as allowed; every request completed. The three models
  ran at the same time, one run each.
- **One scoring rule for everyone**, as for the set: an answer is correct when it
  states the fact of the question's source passage; UNKNOWN is counted apart,
  never as wrong. The models' answers were reviewed blind to the systems: 583
  distinct question-answer pairs, 17 changes proposed and each checked against
  its row, 16 accepted (14 to CORRECT, mostly English or Latin names and
  transliterations of Polish forms; 2 to WRONG) and 1 rejected. Two models gave
  one of the accepted answers, so [review.txt](answer-or-abstain-v6/beside-llm/recorded/review.txt)
  has 17 rows; the rejected change is listed after them. GEL RAM's and the
  engines' verdicts are those reviewed for set v6.

The English prompt, verbatim (SHA-256 `5880eedb…`; the Polish one,
`73204ceb…`, says the same and names Polish Wikipedia):

```text
Answer the question from your own knowledge. The question refers to a fact stated in an English Wikipedia article. Give one short answer (at most 12 words) and no explanation. If you are not sure of the answer, reply with exactly: UNKNOWN

Question: {PYTANIE}
```

## Results — the 400 questions

| System | Conditions | Answered | Correct | Wrong | UNKNOWN | Correct among answered (95% Wilson) |
|---|---|---:|---:|---:|---:|---|
| GEL RAM, balanced | local bank, one stored passage | 170 | 160 | 10 | 230 | 94.1% (89.5–96.8%) |
| Tantivy (BM25), threshold | same bank, threshold | 239 | 226 | 13 | 161 | 94.6% (90.9–96.8%) |
| SQLite FTS5 (BM25), threshold | same bank, threshold | 253 | 239 | 14 | 147 | 94.5% (90.9–96.7%) |
| GEL RAM, precise | the same, stricter setting | 114 | 112 | 2 | 286 | 98.2% (93.8–99.5%) |
| Tantivy (BM25), strict | same bank, strict threshold | 157 | 156 | 1 | 243 | 99.4% (96.5–99.9%) |
| SQLite FTS5 (BM25), strict | same bank, strict threshold | 161 | 159 | 2 | 239 | 98.8% (95.6–99.7%) |
| GPT-OSS-120B | Groq API, closed book | 218 | 89 | 129 | 182 | 40.8% (34.5–47.5%) |
| GPT-OSS-20B | Groq API, closed book | 276 | 46 | 230 | 124 | 16.7% (12.7–21.5%) |
| Qwen3.8-27B | Groq API, closed book | 130 | 33 | 97 | 270 | 25.4% (18.7–33.5%) |

Before the review the models had 82, 43 and 30 correct answers (136, 233 and 100
wrong).

[![The scoreboard frame of the film: the table above.](../media/beside-v6/scoreboard-400.png)](../media/beside-v6/GEL-BESIDE-ENGINES-AND-LLMS-V6-PART2-EN.mp4)

| System | Polish: correct / wrong / UNKNOWN | English: correct / wrong / UNKNOWN |
|---|---|---|
| GEL RAM, balanced | 94 / 7 / 99 | 66 / 3 / 131 |
| Tantivy (BM25), threshold | 131 / 8 / 61 | 95 / 5 / 100 |
| SQLite FTS5 (BM25), threshold | 135 / 10 / 55 | 104 / 4 / 92 |
| GEL RAM, precise | 68 / 2 / 130 | 44 / 0 / 156 |
| Tantivy (BM25), strict | 103 / 1 / 96 | 53 / 0 / 147 |
| SQLite FTS5 (BM25), strict | 104 / 2 / 94 | 55 / 0 / 145 |
| GPT-OSS-120B | 48 / 58 / 94 | 41 / 71 / 88 |
| GPT-OSS-20B | 25 / 118 / 57 | 21 / 112 / 67 |
| Qwen3.8-27B | 24 / 70 / 106 | 9 / 27 / 164 |

Question by question: GEL RAM at its balanced setting beside the engines'
threshold settings and each model, and at its precise setting beside the engines'
strict settings. p is the
two-sided exact sign test on the questions only one of the two answered
correctly (or wrongly):

| GEL RAM setting / other system | Correct only in GEL RAM / only in the other | Wrong only in GEL RAM / only in the other |
|---|---:|---:|
| balanced / Tantivy (BM25), threshold | 16 / 82 (p < 0.001) | 6 / 9 (p = 0.607) |
| balanced / SQLite FTS5 (BM25), threshold | 14 / 93 (p < 0.001) | 6 / 10 (p = 0.454) |
| precise / Tantivy (BM25), strict | 16 / 60 (p < 0.001) | 1 / 0 (p = 1.000) |
| precise / SQLite FTS5 (BM25), strict | 14 / 61 (p < 0.001) | 1 / 1 (p = 1.000) |
| balanced / GPT-OSS-120B | 133 / 62 (p < 0.001) | 6 / 125 (p < 0.001) |
| balanced / GPT-OSS-20B | 141 / 27 (p < 0.001) | 4 / 224 (p < 0.001) |
| balanced / Qwen3.8-27B | 149 / 22 (p < 0.001) | 7 / 94 (p < 0.001) |

What stands out:

- **Closed book, the models were wrong more often than right.** Every question
  was written from a passage of the bank, which they were not given. They were
  allowed to say UNKNOWN, yet 97–230 of their answers were wrong: 59–83% of what
  each answered. GEL RAM was wrong 10 times (twice at its precise setting), the
  engines 13 and 14 times (once and twice at their strict settings).
- **The search engines find more answers than GEL RAM** from the same bank
  (82 and 93 questions only they answered correctly, against 16 and 14; at the
  strict settings 60 and 61 against 16 and 14). On these 400 questions their
  wrong answers do not differ from GEL RAM's beyond chance, at either setting.
  On all 990 questions of the set, GEL RAM was wrong where they were not on 11 and
  10 questions, they where it was not on 26 and 28 (p = 0.020 and p = 0.005;
  beside Tantivy 12 against 24, p = 0.065, before the review), while they
  answered more questions (621 and 652 against 465). At the strict settings it
  was 1 against 3 and 1 against 4 (p = 0.625 and p = 0.375); see
  [set v6](answer-or-abstain-v6/README.md).
- **The two kinds of system know different questions.** GEL RAM answered
  correctly 133–149 questions that a given model did not; each model answered
  correctly 22–62 that GEL RAM did not.

## Limits

- The questions and both reviews were made by the project's AI coding assistant;
  this is not an independent benchmark.
- GEL RAM's answers come from a private development build, not a release. The
  set was used internally before its recorded runs without changing any build or
  setting, so GEL RAM's results on it were known ([set v6](answer-or-abstain-v6/README.md)).
- The models are closed book: this compares conditions, not models. A model
  given the bank's passages would be a different comparison.
- The engines run with default settings; a tuned setup may do better.
- 400 questions, one run per system. Times are not compared: GEL RAM and the
  engines ran locally, the models over the network. Hosted models change; a
  re-run of the same prompts may give other answers.
- GPT-OSS (OpenAI), Qwen (Alibaba Cloud), Groq, Tantivy and SQLite are named only
  to identify the systems compared; no affiliation or endorsement.

## Re-scoring

The recorded model answers are
[gpt-oss-120b.txt](answer-or-abstain-v6/beside-llm/recorded/gpt-oss-120b.txt),
[gpt-oss-20b.txt](answer-or-abstain-v6/beside-llm/recorded/gpt-oss-20b.txt) and
[qwen3.8-27b.txt](answer-or-abstain-v6/beside-llm/recorded/qwen3.8-27b.txt), one
line per question: the question number in set v6, the answer as returned. To
score one with the set's rules, write a file of 990 lines with `UNKNOWN` for the
questions outside the sample and run:

```text
cargo run --locked --offline -p xtask -- answer-bench score v6:with-answer my-answers.txt --rules published
```

The tables above use the published rules; with `--rules published` the counts
printed are those of the sample before the review (82, 43 and 30 correct; 136,
233 and 100 wrong).
Without it the stricter default rules apply; they count one more of Qwen3.8-27B's
answers as wrong (29 correct, 101 wrong). Source passages and expected answers
quote Wikipedia under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/);
the models' answers are reproduced as returned.
