# GEL RAM v0.8.0 — every program and source file names its license, and a README you can check

Date: 2026-11-02

v0.8.0 collects everything merged after v0.7.0 (pull requests #91 to #96).
Every program says what it is and under which license, every GEL RAM-owned
Rust source file begins with its license header, and a new program,
`gel-license`, reads a copy of the tree, or a user's own answers about one use,
against the LICENSE. The license guides now follow the words of the LICENSE,
and the README is rebuilt around what a reader can check with one command each.
The LICENSE itself is unchanged. No file format, search result or third-party
dependency changes. The Rust pin stays 1.85.0; the workspace version moves to
0.8.0. Every change is listed with its pull request in the
[roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.8.0/docs/ROADMAP.md).

## What the project is

GEL RAM prints knowledge into memory as fixed Q8 records, instead of training it
into model weights, and reads the records back exactly, working toward
hardware-level memory computation. The goal is a text AI that answers in Polish
and English from what it holds, or says it does not know. **The text AI is a
goal, not a result of this repository**, and hardware-level memory computation
is not established: the claim registry lists it as `NOT_ESTABLISHED`. The step
that prints text into records is private, so the Q8 records in this checkout
hold synthetic values or literal bytes.

## Programs and files name their license

- Every tool program answers `--version` with one line,
  `GEL_TOOL=<name> VERSION=0.8.0 LICENSE=GEL-RAM-NCRL-1.0`, and `--license`
  with the license name, the `Required Notice:` line, the NOTICE summary and a
  line saying that third-party material in the program, including the
  compiled-in Unicode data, keeps its own license. Both flags work only as the
  sole argument, so every other invocation behaves as before (#91).
- The gel-evidence batch session record names the license; adding the field
  keeps the schema `gel-evidence/1` (#91).
- Every GEL RAM-owned `.rs` file begins with two lines, the license name and the
  Required Notice; `xtask source-headers` checks them in `verify`. The
  generated Unicode tables are third-party data and are exempt (#91).
- `gel-license check DIR` reports whether a tree has the exact LICENSE, the
  Required Notice and intact source headers: `LISTED_ITEMS_PRESENT`,
  `LISTED_ITEMS_MISSING` or `NO_GEL_NOTICES_FOUND`. A fixed note says this is
  not a finding of compliance (#91).
- `gel-license use ANSWERS_FILE` reads the user's own `yes`, `no` or `unsure`
  answers to questions that quote section 2 of the LICENSE, and reports
  `COMMERCIAL_AGREEMENT_REQUIRED`, `ASK_THE_LICENSOR`, `EVALUATION` or
  `NONCOMMERCIAL` with the sections and a verbatim quote. It is packaged with
  the Evidence Lab binaries (#91).

## License guides follow the LICENSE

No license term changes; the LICENSE is the complete public grant (its
section 17), and the guides describe it without adding to or changing it.

- `LICENSING.md`: Evaluation states its purpose condition, "solely to assess,
  learn about, validate, or decide whether to obtain commercial rights", next to
  the non-production condition; Monetized Use covers use that "directly or
  indirectly receives, earns, enables, supports, protects, or is intended to
  produce" a financial benefit, for the user or another person; the exclusions
  and the grant follow the words of the LICENSE (#96).
- `media/RIGHTS.md` names the `Required Notice:` lines of section 8;
  `docs/LICENSE-DECISION.md` says a Commercial Agreement is signed by both
  sides; `THIRD-PARTY-NOTICES.md` covers all three Unicode items (#96).
- NOTICE and `LICENSING.md` no longer list removed files among those that quote
  Wikipedia; the removed copies stay under CC BY-SA 4.0 in the history (#92).

## A README you can check

- The README is generated as a whole by `tools/readme_presentation.rs`, and CI
  fails when it differs from the program's output (#95).
- Each claim at the top has one command; what is private and what is not
  established are listed apart from what can be checked here (#95).
- Its numbers are counted from the files when it is built: published
  commitments from `docs/COMMITMENTS.md`, packages from `Cargo.lock`, the crash
  series and the timing records from their result files. A series that
  disagrees with its own rows is refused (#95).
- The license card quotes the LICENSE word for word; the README is not built
  when a quoted sentence is missing from the LICENSE (#95).
- `docs/TRY-IT.md` follows this release instead of v0.6.0.

## Private work, fixed in time

One more published commitment, private source tree 2 (2026-11-01), in
`docs/COMMITMENTS.md` (#93). As with every commitment, the manifest and the
files stay private until shown, and the commitment says nothing about what the
contents are or whether they are correct.

## CI

The `verify` job has 40 minutes, the limit the portable builds already had;
it had reached its 20-minute limit three times (#94).

## Earlier versions

The notes of v0.7.0 leave the tree, as earlier notes did; they remain in the
history, the tag and the v0.7.0 release page.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.8.0
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline --release -p gel-live-lab -- --witness
cargo run --locked --offline -p gel-live-lab --bin gel-license -- check .
```

Expected final markers: `GEL_VERIFY_ALL=PASS`, `WITNESS=MATCH` and
`LISTED_ITEMS_PRESENT`. `SOURCE-SHA256SUMS.txt` lists every other tracked file
of the tagged tree. The release asset GEL-RAM-v0.8.0-SOURCE.zip is `git archive`
of the tagged commit; its SHA-256 is published next to it. Binary packages are
built in CI and carry a build-provenance attestation, as described in
[binaries](https://github.com/Gelram-project/gel-ram/blob/v0.8.0/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, measured
conversation, private engine or bank, hardware-level memory computation, speed
advantage over grep or any language model, or commercial superiority is claimed.
`gel-license` does not decide whether a use or a copy complies with the LICENSE,
and this repository gives no legal advice. The private printing step and the
private data sets are not part of this release; the commitments only fix their
contents. Four views of one record are not four independent memories, four
votes or a 4× speedup. A process kill or stop is not a power cut; power-loss
safety is not established.

RR — GEL RAM Project
