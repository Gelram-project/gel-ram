# GEL RAM v0.6.0 — knowledge printed, not trained: the record comes first again

Date: 2026-10-09

v0.6.0 is a documentation and evidence release. The public libraries, file
formats, Rust pin and third-party dependencies are unchanged from v0.5.3; the
workspace version moves to 0.6.0. Every change since v0.5.3 is listed with its
commit in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/ROADMAP.md).

## What the project is

GEL RAM prints knowledge into memory as fixed Q8 records, instead of training it
into model weights, and reads the records back exactly, working toward
hardware-level memory computation. The goal is a text AI that answers in Polish
and English from what it holds, or says it does not know. **The text AI is a
goal, not a result of this repository**, and hardware-level memory computation
is not established: the claim registry lists it as `NOT_ESTABLISHED`. The step
that prints text into records is private, so the Q8 records in this checkout are
synthetic.

## The first screen shows the record again

The README now opens with what the public code holds and how to check it:

- **One record, four exact views.** A card drawn from the 48 recorded comparison
  runs in `docs/evidence-q8-current`: one stored record of 1,152 bytes (1,024
  one-byte values and a 128-byte activity mask), read through four equivalent
  views, each score equal bit for bit to a separate reference read on the same
  build and platform. The builder refuses to draw the card if a run lacks
  `Q8_QUAD_EXACT=PASS` or the runs disagree on a byte count. Four views are not
  four independent memories, and 1,152 bytes is the record, not process memory.
- **Exact bytes, or a refusal.** The public collection tool quotes byte for
  byte, refuses an earlier citation after its source is replaced and refuses a
  snapshot with one changed byte under the pin you kept. A matching pin is not
  proof that the source is true.
- **A killed process loses nothing it confirmed.** In the recorded crash series
  the tool was killed at random moments in 200 trials: 0 of 2,683 acknowledged
  snapshots were lost. A process kill is not a power cut.
- **Run it now and the memory core.** Four commands, each with the line it ends
  with and its limit, and a table of the public core (the 128-byte record, the
  store, Reader16, the exact structural rebuild, the four views and F0 memory
  physics), each part with the test that checks it.

The claim registry gains `q8-one-record-four-views` as a `SEPARATE_GATE` row,
and the four rows of the language-model comparisons leave it; the GEL numbers of
set v2 keep a row of their own, `gel-answer-set-v2`.
The banner and the full-page edition carry the same line: knowledge printed, not
trained.

## Question set v7: an external check

[Question set v7](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/answer-or-abstain-v7/README.md)
has 1,581 new questions, frozen before any run and not used before:

- 982 with an answer (487 Polish, 495 English);
- 599 whose answer is not in the bank: 399 about real Wikipedia topics whose
  articles the bank does not hold, and 200 about subjects made up for the set.

Every question, recorded answer and review decision is published, and
`xtask answer-bench check` re-scores them in every verify run. The build that
answered is private and cannot be re-run from this checkout.

- The development build measured on v4 to v6, unchanged, answered 425 of the
  982 questions with an answer: 413 correct and 12 wrong; it said `UNKNOWN` to
  557. At its precise setting it gave 291 correct and 3 wrong answers.
- It answered 26 of the 599 questions without an answer and said `UNKNOWN` to
  the other 573.
- **A candidate change was rolled back.** Its prediction, frozen before the run,
  was more correct answers than the balanced setting. It gave 245 against 413,
  though with fewer wrong answers in all (11 against 38).
- **One flaw of the set is stated in its README.** A topic outside the bank is
  checked by its title, and one such topic is in the bank under another word form.

Search-engine references for this one task, and what has nothing to compare
with in it, are on a new page,
[external checks](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with),
which also lists every question set.

## Language-model pages leave the tree

The pages, films and cards that set GEL RAM beside language models are removed
from the tree; they remain in the git history. The answers those models gave in
question sets v1 and v2 stay as frozen data without links from the README, so
the identity of both sets does not change and `answer-bench check` still
re-scores them. The README no longer carries result cards from the question
sets: their numbers are on the external-checks page, and the author-run
measurements of the private implementation are in
[measured progress](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/MEASURED-PROGRESS.md).

## `answer-bench`

- A third kind of question without an answer, `absent`: a real topic whose article
  the bank does not hold. Its source fields are checked like the others.
- Each set states its own number of questions without an answer and the kinds it
  may hold. Answers given anyway are counted per kind.
- Sets with both parts get a table of all their questions. In that table a wrong
  answer and any answer to a question without one both count as wrong.
- The published tables of v1 to v6 are unchanged.

## Earlier versions

The notes of v0.5.3 leave the tree, as earlier notes did. A side-by-side
comparison of set v6 with three language models, prepared on the release branch,
was removed before release.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.6.0
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. `SOURCE-SHA256SUMS.txt` lists
every other tracked file of the tagged tree. The release asset
GEL-RAM-v0.6.0-SOURCE.zip is `git archive` of the tagged commit; its SHA-256 is
published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in
[binaries](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, measured
conversation, private engine or bank, hardware-level memory computation, speed
advantage over grep or any language model, or commercial superiority is claimed.
Four views of one record are not four independent memories, four votes or a 4×
speedup. A process kill is not a power cut; power-loss safety is not established.
Answering natural questions remains open: GEL answers a minority of them, and a
99% precision is not claimed. A topic checked as outside the bank is not proven
absent from it.
