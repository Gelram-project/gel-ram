# GEL RAM v0.5.1 — answer-verdict results

Date: 2026-09-28

v0.5.1 is a documentation and version release. The public file formats, Rust
pin and third-party dependencies are unchanged from
[v0.5.0](RELEASE-NOTES-v0.5.0.md); the workspace version moves to 0.5.1, and
`xtask verify` gains one documentation check: the summary of the review work
list in the [roadmap](docs/ROADMAP.md) is recounted from its table.

## New measured results

Author-run measurements of the separate private implementation, on the same 1M
PL/EN bank as the v0.5.0 ranking rows. They cannot be re-run from this checkout;
the protocol and evidence identities are in
[measured progress](docs/MEASURED-PROGRESS.md) and the
[claim registry](docs/CLAIMS.md) lists all three as `MEASURED_LOCAL`.

- **Answer verdict** (`gel-answer-verdict-self-read`): GEL answers only when its
  best passage clearly leads the runner-up, with a threshold fixed in advance.
  After storing duplicates once with all sources, leaving reference sections out
  of the answer bank and keeping numbers in the encoder, 50,000 stored-passage
  probes give 92.8% answered, 99.95% correct answers and 0.046% wrong answers
  (from 77.0%, 97.7% and 1.78% on the original bank). The probes are stored
  passages, not questions, and the answer bank changed in scope.
- **Natural questions** (`gel-natural-question-answers`): on 80 PL/EN questions
  written by the project's AI coding assistant and frozen before the run, source
  verification raises top-1 from 27 to 40 and gives 11 answers, all correct,
  with 69 UNKNOWN. This small set supports no precision rate; answering natural
  questions remains open.
- **Side by side with three language models** (`gel-beside-groq-closed-book`):
  the same 80 frozen questions in one run, with GPT-OSS-120B, GPT-OSS-20B and
  Qwen3.8-27B called closed book through the Groq API and one scoring rule for
  all. GEL: 11 correct, 0 wrong, 69 UNKNOWN; the models: 10, 8 and 6 correct
  with 21, 28 and 11 wrong answers. It is not a speed comparison or an engine
  ranking; see [GEL beside three language models](docs/GEL-BESIDE-GROQ.md).

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.5.1
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. `SOURCE-SHA256SUMS.txt` lists
every other tracked file of the tagged tree. The release asset
GEL-RAM-v0.5.1-SOURCE.zip is `git archive` of the tagged commit; its SHA-256 is
published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in [binaries](docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, private
engine or bank, hardware-level memory computation, speed advantage over grep or
any language model, or commercial superiority is claimed. The self-read
precision is not natural-question accuracy.

## Licensing and compatibility

Licensing is unchanged ([LICENSING](LICENSING.md)). Rust remains pinned to
1.85.0. The v0.3.0, v0.4.0 and v0.5.0 tags are not moved.
