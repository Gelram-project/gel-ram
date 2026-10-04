# GEL RAM v0.5.2 — evidence you can re-check

Date: 2026-09-30

v0.5.2 is a documentation and evidence release. The public libraries, file
formats, Rust pin and third-party dependencies are unchanged from
v0.5.1; the workspace version moves to
0.5.2. Every change since v0.5.1 is listed with its commit in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/ROADMAP.md).

## What the project is building

> **Correction (October 2026).** The goal statement introduced in this release was
> restated on `main`: the project works toward a text AI that answers in Polish or
> English from its sources, or says it does not know, and holds a free
> conversation in both languages. See the [README](README.md).

The README now states the goal as a goal, not as a result of this repository. The
README sections "What is different here" and "What you can check without the
private code" give each property with its evidence and its limit.

## New checks anyone can run

- **`xtask answer-bench`** re-scores every recorded answer of four frozen
  question sets under the published and the strict rules, and scores your own
  system on the same questions:
  [set v1](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain/README.md) — 160 questions, half of them without a
  correct answer; [set v2](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain-v2/README.md) — 394 new questions drawn by a
  fixed seed; [set v3](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain-v3/README.md) — 979 questions beside two BM25
  search engines; [set v4](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain-v4/README.md) — 985 questions held out for
  the latest build. For v3 and v4 it also generates and checks precision with
  95% Wilson intervals, the split by language and a question-by-question exact
  test. `xtask verify` runs `answer-bench check` for all four.
- **`xtask disclosure`**, run by `xtask verify`, fails on private paths,
  internal names (compared by hash), network addresses, credentials and e-mail
  addresses other than the published contacts
  ([gate](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/PUBLIC-DISCLOSURE-GATE.md)).
- **`xtask crash-series`** grows a collection in the public tool and kills it
  with SIGKILL at seeded random moments; every acknowledged snapshot must reload
  exactly, an unacknowledged one may be absent but never partial, and a fresh
  process must resume to the same result. Recorded run: 200 trials, 2,683
  acknowledged snapshots, **0 lost, 0 partial**, 200 of 200 resumed
  ([crash series](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/CRASH-SERIES.md)). `xtask verify` runs 5 trials on Unix hosts.
- **Hardening** ([audit](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/AUDIT-HARDENING.md)): fail-closed checks of workflow
  permissions and action pins, full benchmark answer inventories and 40,000
  bounded format mutants.

## New measured results

Author-run measurements of the separate private implementation. They cannot be
re-run from this checkout; the [claim registry](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/CLAIMS.md) lists them as
`MEASURED_LOCAL`, and the recorded answers can be re-scored with `answer-bench`.

- **Set v4, held out for the latest build** ([set v4](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain-v4/README.md)):
  985 new questions written after the tested change and its settings were
  fixed, frozen before any run, one run per system, on the same bank for all.
  The change: passages are searched together with their article title and
  section heading; the answer shown is still the stored passage.

  | Same 985 questions | Answered | Correct | Wrong | Precision (95% Wilson) |
  |:---|---:|---:|---:|---:|
  | GEL RAM, development build | 423 | 405 | **18** | 95.7% (93.4–97.3%) |
  | GEL RAM, the v3 build | 189 | 172 | 17 | 91.0% |
  | Tantivy BM25, same selection rule | 571 | 531 | 40 | 93.0% |
  | SQLite FTS5, same selection rule | 599 | 558 | 41 | 93.2% |

  Question by question, the engines find more answers and GEL RAM gives about
  half as many wrong ones. At the precise setting (precision at least 0.99 on
  v1 + v2) GEL RAM gave 281 correct and 8 wrong answers (97.2%) and the engines
  at their strict thresholds were ahead.
- **Set v3 beside two BM25 search engines** ([set v3](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/answer-or-abstain-v3/README.md)):
  979 questions, all four GEL runs on the set published; the build of that day
  answered 191 (180 correct, 11 wrong) beside 238–241 correct and 28 wrong
  answers from the engines. For that build v3 was a re-test after a fix; v4 is
  its held-out check.
- **Set v2 beside three language models** (one run, 394 frozen questions,
  closed book through the Groq API, one scoring rule for all; after the manual
  review):

  | Same 394 questions | Answered | Correct | Wrong | UNKNOWN |
  |:---|---:|---:|---:|---:|
  | GEL RAM | 63 | 59 | **4** | 331 |
  | GPT-OSS-120B | 223 | 92 | 131 | 171 |
  | GPT-OSS-20B | 283 | 50 | 233 | 111 |
  | Qwen3.8-27B | 123 | 43 | 80 | 271 |

  GEL's precision is 93.7% (95% Wilson 84.8–97.5%); it answers 16% of the
  questions. Its 4 wrong answers are passages from another article.
- **No-answer control** ([page](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/GEL-BESIDE-GROQ-NO-ANSWER.md)): 80 questions with no
  correct answer (40 invented subjects, 40 false premises). Answered anyway:
  GEL 6 (0 on invented subjects, 6 on false premises); the models 9, 38 and 5.
- **Kill during learning — the private knowledge store**
  ([measured progress](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/MEASURED-PROGRESS.md)): 207 trials in three series,
  0 confirmed records lost, every reopen check passed and every resumed run
  equal to a run without a kill.

## For people who do not write code

The README and CONTRIBUTING now point to four ways to help without writing
code — try to break it, check the evidence, improve the question sets, review a
claim — each with an issue form. Every results page marks author-run
measurements of the private implementation as such.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.5.2
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
```

Expected final marker: `GEL_VERIFY_ALL=PASS`. `SOURCE-SHA256SUMS.txt` lists
every other tracked file of the tagged tree. The release asset
GEL-RAM-v0.5.2-SOURCE.zip is `git archive` of the tagged commit; its SHA-256 is
published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in [binaries](https://github.com/Gelram-project/gel-ram/blob/v0.5.2/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, private
engine or bank, hardware-level memory computation, speed advantage over grep or
any language model, or commercial superiority is claimed. A process kill is not
a power cut; power-loss safety is not established. The side-by-side runs show
what each system does when it does not know, not which knows more, and answering
natural questions remains open: GEL answers a minority of them. On the same bank
the BM25 search engines find more answers than GEL RAM; a 99% precision is not
claimed.
