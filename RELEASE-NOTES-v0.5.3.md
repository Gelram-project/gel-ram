# GEL RAM v0.5.3 — one goal, traceable numbers, a fifth question set

Date: 2026-10-04

v0.5.3 is a documentation and evidence release. The public libraries, file
formats, Rust pin and third-party dependencies are unchanged from v0.5.2; the
workspace version moves to 0.5.3. Every change since v0.5.2 is listed with its
commit in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/docs/ROADMAP.md).

## What the project is building

A text AI whose knowledge is written into memory rather than trained into model
weights. It answers in Polish or English from that knowledge and shows the source
it used, or says plainly that it does not know, and it holds a free conversation
in both languages. **This is the goal, not a result of this repository.** Free
conversation has not been measured yet. The README states the goal first and
labels each result card with the question set and the build it comes from.

## New measured result: question set v5

[Question set v5](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/docs/answer-or-abstain-v5/README.md)
has 987 new questions (491 Polish, 496 English), frozen before any run. Every
question, recorded answer and review decision is published, and
`xtask answer-bench check` re-scores them in every verify run.

- **The v4 result held.** The development build measured on question set v4,
  unchanged, answered 432: 416 correct and 16 wrong, a precision of 96.3% (95%
  Wilson interval 94.1–97.7%), after 95.6% where its setting was chosen and
  95.7% on question set v4.
- **Beside two BM25 search engines** on the same bank and rule, the engines
  found more answers (567 and 597 correct), and GEL RAM gave about a third as
  many wrong ones (16 against 43 and 47).
- **At the precise setting** GEL RAM gave 297 correct and 1 wrong. On question
  set v4 the same setting gave 97.2%, so a precision of 0.99 is still not
  claimed; the engines at their strict thresholds found more answers.
- **A candidate change was rolled back.** It gave more correct answers, but its
  precision fell to 92.3%, below the 0.95 fixed before the run.

## Numbers you can trace

- The first screen of the README labels each card with its question set and
  build, and links the published question files.
- [Measured progress](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/docs/MEASURED-PROGRESS.md)
  gives exact counts for every answer-verdict row. One row is corrected: the
  original-bank run counted 9,998 probes and 76.9% answered, not 10,000 and
  77.0%. The page also states the bank sizes: 1M passages at first, 671,416
  after the measured changes.
- Question sets are named "question set v1" to "v5", so they are not read as
  release versions.

## One license set, English documentation

- The license text is unchanged. The license, contributor agreement, its privacy
  notice and the commercial terms are pinned by SHA-256 in
  [license pins](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/docs/LICENSE-PINS.md),
  with a test; the separate draft licensing folder was removed.
- Wikipedia passages quoted in the question sets are credited under
  CC BY-SA 4.0 in the
  [third-party notices](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/THIRD-PARTY-NOTICES.md).
- Documentation is in English only, with a guard in `xtask verify`; Polish
  remains in the question data.
- The disclosure gate checks paths, addresses and credentials without carrying
  a word list.

## Earlier versions

Earlier README versions, the notes of earlier releases and two films of a
private application preview were removed from the tree. The release pages of
v0.3.0 to v0.5.1 were removed; their tags remain.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.5.3
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. `SOURCE-SHA256SUMS.txt` lists
every other tracked file of the tagged tree. The release asset
GEL-RAM-v0.5.3-SOURCE.zip is `git archive` of the tagged commit; its SHA-256 is
published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in
[binaries](https://github.com/Gelram-project/gel-ram/blob/v0.5.3/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, measured
conversation, private engine or bank, hardware-level memory computation, speed
advantage over grep or any language model, or commercial superiority is claimed.
A process kill is not a power cut; power-loss safety is not established.
Answering natural questions remains open: GEL answers a minority of them. On the
same bank the BM25 search engines find more answers than GEL RAM; a 99%
precision is not claimed.
