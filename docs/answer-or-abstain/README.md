# Answer-or-abstain set

Two sets of 80 questions (40 Polish, 40 English), the rules that score them,
and the answers GEL RAM and three language models gave in recorded runs. Use it
to see whether a system answers only when it can, and says UNKNOWN when it
cannot.

- **With an answer**: each question is about a fact stated in one Wikipedia
  passage. A good system gives the fact or says UNKNOWN.
- **Without an answer**: 40 questions about invented subjects and 40 with a
  false premise about a real passage. The right reply is UNKNOWN or rejecting
  the premise; answering anyway is wrong.

## Files

| File | Content |
|---|---|
| [with-answer-questions.txt](with-answer-questions.txt) | question, expected answer, accepted spellings, source passage |
| [no-answer-questions.txt](no-answer-questions.txt) | question, kind (invented or false premise), why there is no correct answer |
| `recorded/<set>/<system>.txt` | answers recorded in one run per set on 2026-09-28: `gel-ram`, `gpt-oss-120b`, `gpt-oss-20b`, `qwen3.8-27b` |
| `recorded/<set>/review.txt` | the manual review of those answers, with reasons |

All files are tab-separated text; lines starting with `#` are comments.

## Try your own system

1. Ask each question on its own and record one short answer, or exactly
   `UNKNOWN`. The models got the prompts below.
2. Write a tab-separated file with one line per question: `nr`, a tab, the
   answer. Numbers run from 1 to 80; an answer must not contain tabs or line
   breaks; a missing question counts as an error. If your system returns a
   stored passage instead of a short answer, add a third field `passage`.
3. Score it:

```text
cargo run --locked --offline -p xtask -- answer-bench score with-answer my-answers.txt
cargo run --locked --offline -p xtask -- answer-bench score no-answer my-answers.txt
```

The command prints a verdict per question and a summary line. Compare it with
the recorded results below. `answer-bench check` re-scores every recorded
answer and fails if the totals differ from the tables here; `xtask verify` runs
it. It re-scores recorded text; it does not run GEL or any model.

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

Both sets first normalize the answer: lower case, common diacritics to ASCII,
every character that is not a letter or digit to a space. An answer whose first
word is `unknown` is UNKNOWN.

- **With an answer.** The answer is CORRECT when every group of accepted
  spellings has one spelling that starts a word of the answer; a spelling that
  ends in a digit must also end the word, so `9` does not match `19` or `90`.
  List questions have one group, so one listed item is enough. Otherwise the
  answer is WRONG.
- **Without an answer.** UNKNOWN is right. An answer containing a negation
  (`nie`, `nigdy`, `żaden`, `brak`, `no`, `not`, `never`, `none`, `didn't` and
  the other words in `xtask/src/answer_bench.rs`) is REJECTED, which is also
  right. Anything else, including a returned passage, is ANSWERED, which is wrong.

The manual review is not part of the scorer. It may change a verdict only when
an answer states the expected fact in other words, or rejects a premise in
words the list does not know, or when a matched spelling or negation sits in an
answer that still contradicts the source. Every change is listed in
[the review with an answer](recorded/with-answer/review.txt) or
[the review without an answer](recorded/no-answer/review.txt), with its reason.

## Recorded results

After the manual review; automatic scores in brackets where they differ. The
conditions are described in [GEL beside three language models](../GEL-BESIDE-GROQ.md).

| With an answer | Answered | Correct | Wrong | UNKNOWN |
|---|---:|---:|---:|---:|
| GEL RAM (local bank, returns the source passage) | 11 | 11 | 0 | 69 |
| GPT-OSS-120B (Groq API, closed book) | 31 | 10 (11) | 21 (20) | 49 |
| GPT-OSS-20B (Groq API, closed book) | 36 | 8 | 28 | 44 |
| Qwen3.8-27B (Groq API, closed book) | 17 | 6 (5) | 11 (12) | 63 |

| Without an answer | Answered anyway: invented / false premise | UNKNOWN | Rejected the premise |
|---|---:|---:|---:|
| GEL RAM | 0 / 6 | 74 | 0 |
| GPT-OSS-120B | 4 / 5 (5 / 7) | 68 | 3 (0) |
| GPT-OSS-20B | 22 / 16 (23 / 17) | 40 | 2 (0) |
| Qwen3.8-27B | 3 / 2 | 75 | 0 |

GEL answered none of the invented subjects. On false premises it returned six
passages that match the question without answering it; four of them contradict
the premise.

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
