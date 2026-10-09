# External checks

Frozen question sets check private development builds from the outside; each
set page names the build it measured. Every question, its expected answer and
source passage, and every recorded answer are published.
`cargo run --locked --offline -p xtask -- answer-bench check` re-scores the
recorded answers under two rules on every verify run and fails if a published
table differs. The builds and their bank are private, so their answers cannot
be re-run from this checkout.

The GEL RAM column repeats the row named GEL RAM of each set's results block,
under the published rules after the manual review.

| Set | Questions | GEL RAM | Page |
|:---|:---|:---|:---|
| v1 | 80 with an answer, 80 without one | with an answer: 11 answered, 11 correct, 0 wrong, 69 UNKNOWN; without one: 6 answered, 74 UNKNOWN | [set v1](answer-or-abstain/README.md) |
| v2 | 394 with an answer | 63 answered, 59 correct, 4 wrong, 331 UNKNOWN | [set v2](answer-or-abstain-v2/README.md) |
| v3 | 979 with an answer | 191 answered, 180 correct, 11 wrong, 788 UNKNOWN | [set v3](answer-or-abstain-v3/README.md) |
| v4 | 985 with an answer | 423 answered, 405 correct, 18 wrong, 562 UNKNOWN | [set v4](answer-or-abstain-v4/README.md) |
| v5 | 987 with an answer | 432 answered, 416 correct, 16 wrong, 555 UNKNOWN | [set v5](answer-or-abstain-v5/README.md) |
| v6 | 990 with an answer | 465 answered, 447 correct, 18 wrong, 525 UNKNOWN | [set v6](answer-or-abstain-v6/README.md) |
| v7 | 982 with an answer, 599 without one | with an answer: 425 answered, 413 correct, 12 wrong, 557 UNKNOWN; without one: 26 answered, 573 UNKNOWN | [set v7](answer-or-abstain-v7/README.md) |

## Question set v7 in brief

On the 982 questions with an answer the private build at its balanced setting
gave 413 correct answers, each with its source passage, 12 wrong and 557
UNKNOWN; at its precise setting 291 correct and 3 wrong. On the 599 questions
written to have no answer in the bank (topics outside it are checked by title,
and the set page lists one exception) it answered 26 and said UNKNOWN to 573.
Two BM25 search engines, Tantivy and SQLite FTS5, ran on the same passages
under the same rule. At their plain threshold they find more answers (550 and
581 correct, against 413) and give more wrong ones (43 each, against 12); at
their strict threshold they give fewer correct answers (380 and 384) and fewer
wrong ones (9 each), and say UNKNOWN to 574 and 570 of the 599 questions
without an answer. The authoritative numbers are the results block of the set
page, which `answer-bench check` compares with the recorded answers.

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
  ([contract](Q8-QUAD.md), [the 48 runs](evidence-q8-current/README.md)).
- **Reader16.** One fused comparison of two records returns 16 judgments; they
  are not 16 independent channels ([Reader16](READER16.md)).
- **F0 memory physics.** The cost of the memory the records live in is measured
  in nanoseconds and GiB/s instead of assumed ([method](PERFORMANCE.md)).

Hardware-level memory computation, the direction of the project, is not
established by any run on these pages or in this repository
([claim registry](CLAIMS.md)). UNKNOWN is not unique to GEL RAM: at their
thresholds the engines also return UNKNOWN, and at the strict threshold they
abstain on 574 and 570 of the 599 questions without an answer.
