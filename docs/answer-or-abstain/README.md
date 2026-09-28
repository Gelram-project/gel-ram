# Answer-or-abstain set

`answer_or_abstain_v1` · SHA-256 `bafc32cb8ff76c28b8b51bc72da743856b1fd8e0e01b073ffc39df04b30d0dc5`

Two sets of 80 questions (40 Polish, 40 English), the rules that score them,
and the answers GEL RAM and three language models gave in recorded runs. Use it
to see whether a system answers only when it can, and says UNKNOWN when it
cannot.

- **With an answer**: each question is about a fact stated in one Wikipedia
  passage. A good system gives the fact or says UNKNOWN.
- **Without an answer**: 40 questions about synthetic subjects made up for the
  set and 40 with a false premise about a real passage. The right reply is
  UNKNOWN or rejecting the premise; answering anyway is wrong.

**Status: public diagnostic.** Once published, these 160 questions are no longer
held out. A system tuned on them and then scored on them proves nothing about
new questions. Changes to GEL will be measured on a new frozen set (v2) that is
not used for tuning. The identity above is the SHA-256 of a list of the hashes
of every data file of the set; `answer-bench check` recomputes it.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source title, URL, dump, entry, passage hash, passage |
| [no-answer-questions.txt](no-answer-questions.txt) | question, kind, why there is no correct answer, the names checked for synthetic subjects, and the source fields for false premises |
| `recorded/<set>/<system>.txt` | answers recorded in one run per set on 2026-09-28: `gel-ram`, `gpt-oss-120b`, `gpt-oss-20b`, `qwen3.8-27b` |
| `recorded/<set>/review.txt` | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments.

**Provenance.** Passages come from the Kiwix dumps
`wikipedia_pl_all_maxi_2026-05` and `wikipedia_en_all_maxi_2026-02`. Each row
gives the article title, its Wikipedia URL, the dump, the dump entry number, the
passage text as stored and the SHA-256 of that text; `answer-bench check`
verifies every hash. The dumps carry no per-article revision id, so the dump
name and entry identify the snapshot; the live article behind the URL may have
changed since.

**Synthetic subjects.** Their key names did not occur in the frozen bank, in all
Polish Wikipedia titles of the 2026-05 dump or in an English title list of the
2026-02 dump. That was checked; that they exist nowhere else was not.

## Try your own system

1. Ask each question on its own and record one short answer, or exactly
   `UNKNOWN`. The models got the prompts below.
2. Write a tab-separated file with one line per question: `nr`, a tab, the
   answer. Numbers run from 1 to 80. A line has exactly two fields, or three
   when the third is `passage` (your system returned a stored passage instead of
   a short answer); anything else is refused. A missing question counts as an
   error.
3. Score it:

```text
cargo run --locked --offline -p xtask -- answer-bench score with-answer my-answers.txt
cargo run --locked --offline -p xtask -- answer-bench score no-answer my-answers.txt
```

The strict rules are the default; add `--rules published` for the rules behind
the recorded results. The command prints a verdict per question and a summary
line. Answers marked REVIEW need a human decision before they count.

Prompt for the set with an answer (the Polish version says the same and names
Polish Wikipedia):

```text
Answer the question from your own knowledge. The question refers to a fact stated in an English Wikipedia article. Give one short answer (at most 12 words) and no explanation. If you are not sure of the answer, reply with exactly: UNKNOWN

Question: {question}
```

Prompt for the set without an answer:

```text
Answer the question from your own knowledge. Give one short answer (at most 12 words) and no explanation. If you are not sure, or the question has no correct answer, reply with exactly: UNKNOWN

Question: {question}
```

## Scoring rules

Both rule sets first normalize the answer: lower case, common diacritics to
ASCII, every character that is not a letter or digit to a space. An accepted
spelling always has to start a word of the answer; one that ends in a digit
also has to end the word, so `9` does not match `19` or `90`.

| | Published rules (frozen before the recorded runs) | Strict rules (default for new runs) |
|---|---|---|
| Accepted spelling | may be the start of a longer word (`mann` matches `Mannheim`) | must be a whole word unless marked as a stem with `*` |
| UNKNOWN | the first word is `unknown` | the whole answer is `unknown` |
| Negation in an answer to a question without an answer | REJECTED, counted as right | REVIEW, decided by a human |

A spelling is marked as a stem mechanically: when it is not a whole word of its
source passage and does not end in a digit (`lodz*` for "Łodzi", `hughes*` for
"Hughesa", `actin*` for an English alternative). List questions have one group
of spellings, so one listed item is enough; every group must match.

Without an answer, UNKNOWN is right, a rejection of the premise is right, and
anything else is wrong, including a returned passage. The negation words are
listed in `xtask/src/answer_bench.rs`.

The manual review is not part of the scorer. It may change a verdict only when
an answer states the expected fact in other words, or rejects a premise in words
the list does not know, or when a matched spelling or negation sits in an answer
that still contradicts the source. Every change is listed in
[the review with an answer](recorded/with-answer/review.txt) or
[the review without an answer](recorded/no-answer/review.txt), with its reason.

## Recorded results

After the manual review; automatic scores in brackets where they differ. The
tables are generated from the recorded answers (`answer-bench tables`), and
`answer-bench check`, run by `xtask verify`, fails if this README or the
[side-by-side comparison](../GEL-BESIDE-GROQ.md) shows other numbers.

<!-- ANSWER-BENCH-RESULTS-BEGIN -->
| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 11 | 11 | 0 | 69 | 0 |
| GEL RAM | strict | 11 | 11 | 0 | 69 | 0 |
| GPT-OSS-120B | published | 31 | 10 (11) | 21 (20) | 49 | 0 |
| GPT-OSS-120B | strict | 31 | 10 (11) | 21 (20) | 49 | 0 |
| GPT-OSS-20B | published | 36 | 8 | 28 | 44 | 0 |
| GPT-OSS-20B | strict | 36 | 8 | 28 | 44 | 0 |
| Qwen3.8-27B | published | 17 | 6 (5) | 11 (12) | 63 | 0 |
| Qwen3.8-27B | strict | 17 | 6 (5) | 11 (12) | 63 | 0 |

| Without an answer | Rules | Answered anyway: invented / false premise | UNKNOWN | Rejected the premise | Needs review | Errors |
|---|---|---:|---:|---:|---:|---:|
| GEL RAM | published | 0 / 6 | 74 | 0 | 0 | 0 |
| GEL RAM | strict | 0 / 6 | 74 | 0 | 0 | 0 |
| GPT-OSS-120B | published | 4 (5) / 5 (7) | 68 | 3 (0) | 0 | 0 |
| GPT-OSS-120B | strict | 4 (5) / 5 (7) | 68 | 3 (0) | 0 | 0 |
| GPT-OSS-20B | published | 22 (23) / 16 (17) | 40 | 2 (0) | 0 | 0 |
| GPT-OSS-20B | strict | 22 (23) / 16 (17) | 40 | 2 (0) | 0 | 0 |
| Qwen3.8-27B | published | 3 / 2 | 75 | 0 | 0 | 0 |
| Qwen3.8-27B | strict | 3 / 2 | 75 | 0 | 0 | 0 |
<!-- ANSWER-BENCH-RESULTS-END -->

Under the strict rules every recorded verdict is the same as under the
published ones. GEL answered none of the synthetic subjects. On false premises
it returned six passages that match the question without answering it; four of
them contradict the premise.

## How we compare

1. **Freeze first.** Questions, expected answers and accepted spellings,
   prompts, the scoring program and the protocol are fixed and identified by
   SHA-256 before any system is asked.
2. **One run.** No reruns and no prompt, model or limit changes after seeing
   answers. A changed setup is a new, separately named run.
3. **Same questions, same rule, for every system.** UNKNOWN is never counted as
   wrong, and never as right on a question that has an answer.
4. **Review by fixed rules only.** The manual review is counted separately and
   every change is listed with the answer it concerns.
5. **Publish every answer**, including the ones that make GEL look bad. Raw
   requests and responses are kept and identified by SHA-256.
6. **Times are records, not races,** unless the conditions are the same.
7. **Say what is not shown.**
8. **A published set is spent.** It stays as a public diagnostic; new claims
   need a new frozen set.

## Limits

- The questions and expected answers were written by the project's AI coding
  assistant for passages in the GEL bank. This is not an independent benchmark,
  and the set with an answer favours a system that holds those passages: some
  questions are underspecified without them.
- Accepted spellings are generous for lists and strict for wording: a true but
  less specific answer counts as wrong.
- The false premises contradict their source passages; whether each is false
  everywhere else was not checked outside the bank.
- 80 questions per set give wide intervals. The scorer checks text, not truth.

## Attribution

Source passages, expected answers and the reasons for false premises quote
Wikipedia, available under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the questions
paraphrase it. Model answers are reproduced as returned. Third-party material
keeps its own terms, as described in [LICENSING](../../LICENSING.md).
