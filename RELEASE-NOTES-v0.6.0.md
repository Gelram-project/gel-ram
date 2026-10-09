# GEL RAM v0.6.0 — questions whose answer is not there

Date: 2026-10-09

v0.6.0 is a documentation and evidence release. The public libraries, file
formats, Rust pin and third-party dependencies are unchanged from v0.5.3; the
workspace version moves to 0.6.0. Every change since v0.5.3 is listed with its
commit in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/ROADMAP.md).

## What the project is building

A text AI whose knowledge is written into memory rather than trained into model
weights. It answers in Polish or English from that knowledge and shows the source
it used, or says plainly that it does not know, and it holds a free conversation
in both languages. **This is the goal, not a result of this repository.** Free
conversation has not been measured yet.

## New measured result: question set v7

[Question set v7](https://github.com/Gelram-project/gel-ram/blob/v0.6.0/docs/answer-or-abstain-v7/README.md)
has 1,581 new questions, frozen before any run and not used before:

- 982 with an answer (487 Polish, 495 English);
- 599 whose answer is not in the bank: 399 about real Wikipedia topics whose
  articles the bank does not hold, and 200 about subjects made up for the set.

Every question, recorded answer and review decision is published, and
`xtask answer-bench check` re-scores them in every verify run.

- **The result of v4 to v6 held a fourth time.**
  - The development build measured on v4 to v6, unchanged, answered 425 of the
    982 questions with an answer: 413 correct and 12 wrong, a precision of 97.2%
    (95% Wilson interval 95.1–98.4%).
  - It answered 26 of the 599 questions without an answer and said `UNKNOWN` to
    the other 573.
  - Over all 1,581 questions: 413 correct and 38 wrong.
- **Beside two BM25 search engines at their strict threshold**, with about as
  many wrong answers in all (34 and 38 against 38):
  - the engines gave 380 and 384 correct answers, GEL RAM 413;
  - question by question, GEL RAM answered correctly 119 and 117 questions that
    the engines did not, the engines 86 and 88 that GEL RAM did not (p = 0.025
    and p = 0.050);
  - in English GEL RAM gave 175 correct answers, the engines 138 and 137.
  - At the plain threshold the engines found more answers (550 and 581 correct)
    and gave many more wrong ones (163 and 187 in all).
- **At the precise setting** GEL RAM gave 291 correct and 3 wrong answers (99.0%)
  and answered 3 of the 599 questions without an answer: 6 wrong in all. Across
  v4 to v7 a precision of 0.99 is still not claimed.
- **A candidate change was rolled back.** Its prediction, frozen before the run,
  was more correct answers than the balanced setting. It gave 245 against 413,
  though with fewer wrong answers (11 against 38).
- **One flaw of the set is stated in its README.** A topic outside the bank is
  checked by its title, and one such topic is in the bank under another word form.

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
was removed before release; the earlier comparison on question set v1 is unchanged.

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
A process kill is not a power cut; power-loss safety is not established.
Answering natural questions remains open: GEL answers a minority of them. At the
same rule the BM25 search engines find more answers than GEL RAM; a 99% precision
is not claimed. A topic checked as outside the bank is not proven absent from it.
