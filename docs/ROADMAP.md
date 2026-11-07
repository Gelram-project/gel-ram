# GEL RAM public roadmap

This roadmap describes only work that is safe to discuss in the public repository.
The complete private research direction, unpublished execution mechanisms and
private performance work are intentionally outside this document.

## Version boundary
- `v0.8.0` is the current version line; its tag and release page are made by
  the owner's publication from the merge commit of its version cutover, and the
  Releases page shows whether that has happened. `v0.7.0` is the release before
  it; tagged releases, including `v0.6.1`, `v0.6.0` and `v0.5.3`, are listed on
  the Releases page. The tags `v0.5.2`, `v0.5.1`, `v0.5.0`, `v0.4.0` and `v0.3.0`
  remain without a release page. Changes merged after `v0.8.0` are listed under
  [after v0.8.0](#after-v080).
- The workspace version is 0.8.0 ([Cargo.toml](../Cargo.toml)). A
  [test](../xtask/tests/version_line.rs) ties a fixed list of files that name
  the current version to it, the first line of this section and the "After"
  heading among them; a mention outside that list is not checked.
- The sections [v0.8.0](#v080) back to [v0.5.0](#v050) list each merged change
  with its commit, the command that checks it and its remaining acceptance.
  Earlier release points are recorded in the [publication status](../CANDIDATE-STATUS.md).
- This page keeps three kinds of statement apart:
  [merged capabilities](#public-main--already-delivered), each shown by a file,
  program, test or CI job of this tree; [historical evidence](#historical-evidence),
  recorded at an earlier revision and not re-run; and gates that are still open,
  in [open review items](#open-review-items) and the
  [review work list](#review-work-list-and-next-acceptance). A merged capability
  is not independent acceptance.
- How the base arrived, as history: document readout through
  [PR #4](https://github.com/Gelram-project/gel-ram/pull/4) (exact source
  quotations, Unicode/CR regressions, runtime CI on three operating systems);
  [Live Lab](LIVE-LAB.md) and [source building](SOURCE-BUILDER.md) in PR #5;
  [Evidence Lab](EVIDENCE-LAB.md) through PR #9 (71142a2, workspace
  0.4.0-rc.1), whose review repairs followed in PR #10 (0c17f6e) and became
  version 0.4.0.
- Pull requests, required checks and squash merges are branch rules of the
  repository settings; this tree does not show them.

## Public main — already delivered

Present on `main` at v0.8.0; each entry names what shows it in this tree.

Source readout and integrity:

- Exact source-bound quotations with SHA-256 provenance, refusing stale or
  corrupt input ([source readout](SOURCE-READOUT.md),
  [tests](../crates/gel-source/tests/source.rs)); SHA-256 and Unicode NFC are
  computed by gel-source itself, NFC from UCD 16.0.0 tables that a test
  regenerates byte for byte (`xtask ucd-tables`,
  [Unicode normalization](DOCUMENT-READOUT.md#unicode-normalization)).
- Multipart readout with explicit gaps, ambiguity and UNKNOWN
  ([source parts](SOURCE-PARTS.md), [tests](../crates/gel-source/tests/parts.rs));
  bounded quote context with omission flags ([quote context](QUOTE-CONTEXT.md)).
- GELSRC01 plaintext source bundles with no-replace publication, caller-retained
  full-file SHA-256 pins and fresh-process reopen ([source bundle](SOURCE-BUNDLE.md),
  [source builder](SOURCE-BUILDER.md), [tests](../crates/gel-source/tests/persistence.rs)).

Programs:

- `gel-evidence`: multi-document collection, exact citations, stale-result
  rejection, GELSET01 snapshots and fresh-process reopen
  ([Evidence Lab](EVIDENCE-LAB.md), [tests](../crates/gel-live-lab/tests/evidence.rs)),
  and a batch mode with the schema `gel-evidence/1`
  ([batch interface](EVIDENCE-BATCH.md), [tests](../crates/gel-live-lab/tests/batch.rs)).
- `gel-backup`: backup, restore, withdraw, delete and clear
  ([backup](BACKUP.md), [tests](../crates/gel-live-lab/tests/backup_cli.rs)).
- `gel-live-lab`: an offline lab for caller-selected UTF-8 documents
  ([Live Lab](LIVE-LAB.md)); `--literal` places a file's first bytes into one Q8
  record and `--reopen` checks it under its pin
  ([your own bytes](LIVE-LAB.md#your-own-bytes-in-one-record),
  [tests](../crates/gel-live-lab/tests/literal.rs)); `--witness` compares Q8
  score bits with recorded pins, on every CI platform
  ([same bits](LIVE-LAB.md#same-bits-on-your-machine)).
- `gel-license`: `check DIR` and `use ANSWERS_FILE`; not legal advice
  ([decision guide](LICENSE-DECISION.md), [tests](../crates/gel-live-lab/tests/license_cli.rs)).
- The eight tool programs (`gel-cli`, `gel-bench`, `gel-physics`,
  `gel-live-lab`, `gel-evidence`, `gel-backup`, `gel-source` and `gel-license`)
  answer `--version` and `--license`; the `xtask` runner does not
  ([binaries](BINARIES.md#version-and-license-of-each-program)).

Records:

- Synthetic Q8 records with four reversible coordinate views, exact inverse
  checks and one frozen cosine table ([Q8 contract](Q8-QUAD.md),
  [table](../crates/gel-phase-quad/src/grid.rs)); the views are not independent information.
- Record history in the GELHIS01 format, rebuilt bit for bit
  ([record history](RECORD-HISTORY.md), [tests](../crates/gel-history/tests/history.rs)).
- An independent byte, numeric and ranking audit, run in CI on every platform
  ([data integrity](DATA-INTEGRITY.md)).

Verification and gates:

- `xtask verify` runs, among its steps, formatting, lints, the workspace tests
  and doctests, the Rust-only, licensing, workflow-policy, documentation-reference,
  disclosure, source-header and timing-example gates, the claim registry
  ([claims](CLAIMS.md)), the answer re-scoring ([external checks](EXTERNAL-CHECKS.md)),
  the recount of the review work list below, the crash series on Unix
  ([crash series](CRASH-SERIES.md)), the format mutation matrices
  ([matrices](MUTATION-MATRIX.md)) and the runtime demos ([gates](GATES.md),
  [disclosure gate](PUBLIC-DISCLOSURE-GATE.md), [recorder safety](RECORDER-SAFETY.md)).
- One command for a local report and a short witness (`xtask reproduce`,
  `xtask witness`, [reproduce](REPRODUCE.md)); a release package built twice and
  compared (`xtask rebuild-check`, [binaries](BINARIES.md#building-the-same-bytes));
  a digest that fixes a directory that is not published
  (`xtask commitment-check`, [commitments](COMMITMENTS.md)); an A/B timing tool
  with an A/A control (`xtask speed-ab`, [A/B tool](SPEED-AB.md)).
- No third-party package in [Cargo.lock](../Cargo.lock)
  ([dependency inventory](DEPENDENCY-INVENTORY.md),
  [test](../xtask/tests/dependency_inventory.rs)); Rust pinned to 1.85.0
  ([toolchain](../rust-toolchain.toml)).
- CI: `verify`, the source manifest, network isolation, a full-disk
  publication and a sanitized evidence report on Linux, and workspace tests and
  runtime checks on macOS and Windows ([CI workflow](../.github/workflows/ci.yml),
  [CI evidence](CI-EVIDENCE.md)); binary packages built and checked in CI
  ([binaries workflow](../.github/workflows/binaries.yml)); the generated README
  rechecked ([README workflow](../.github/workflows/readme-presentation.yml),
  [presentation guide](README-PRESENTATION.md)).
- A property-to-test map checked on every CI platform ([property map](PROPERTY-TESTS.md))
  and the goal stated once ([goal](GOAL.md), [test](../xtask/tests/goal_statement.rs)).

### Historical evidence

Recorded at an earlier revision or on the author's machine and kept as it was;
not re-run by the checks above unless stated.

- Ocean Scale R3 1M/10M synthetic full-scan measurements; the research archive
  was withdrawn in 0.4.0 ([results](OCEAN-SCALE.md)).
- Native CI of the PR #9 head ([platform record](PLATFORM-REVIEW.md)) and the
  per-platform reports of three later revisions ([recorded reports](evidence-ci/README.md)).
- Collection measurements r1, whose percentiles `collection_recheck` recomputes
  from the raw observations, without timing again, in `verify` and on every CI
  platform ([record](evidence-collection/README.md)).
- The comparison with grep and sha256sum, run r1 ([summary](evidence-bench-r1/summary.txt)),
  and the mutation campaign r1 ([campaign](evidence-mutation-campaign-r1/CAMPAIGN.txt)), each on one host.
- Timing records of the v0.7.0 speed work on one machine
  ([against v0.6.1](speed-results/2026-10-20-against-v0.6.1/README.md),
  [quiet machine](speed-results/2026-10-25-quiet-machine/README.md)).
- Recorded answers of private development builds to frozen question sets,
  re-scored on every verify run but not re-run ([external checks](EXTERNAL-CHECKS.md)).
- Author-reported component measurements of the private implementation
  ([experimental measurements](GEL-EXPERIMENTAL-MEASUREMENTS.md)).

## Open review items

Re-checked against the tree at v0.8.0 (`319b4e8`); the state words are those of
the [review work list](#how-to-read-the-review-work-list).

1. OPEN. Independent, non-author review of the multi-document Evidence Lab:
   source boundaries, stale citation invalidation, snapshots and independently
   recomputed raw measurements. The [assessment](ASSESSMENT-REVIEW.md) is
   developer-authored, not blind validation; no outside review is recorded.
   Single-document import/read/save/reopen belongs to the earlier published base.
2. PARTIAL. Per-platform CI evidence for every revision: every CI run prints a
   sanitized report on Linux, macOS and Windows (`xtask ci-evidence`), but runs
   and logs expire and only three revisions are kept in the tree, the latest
   79f6ac5 ([recorded reports](evidence-ci/README.md)). `xtask
   ci-evidence-recorded`, in `verify`, checks what is kept. No CI job compares
   two platforms. A unit test of xtask reads the workspace's Rust sources, so a
   test under a platform `cfg` that is not declared is found when it is added; a
   test made by a macro is found only when a revision is recorded, and a
   platform-only block inside a test or a check of the platform at run time is
   not found by either. These are sanitized reports of hosted CI runs of one
   revision each; hosted CI is not an independent second host. Unix permission
   and directory-sync checks remain platform-specific, not portable guarantees.
3. DONE_SCOPED, since v0.5.0. One-command local reporting: `xtask reproduce`
   writes into one new directory the raw output of every step, host and load
   at the start and end, each step's result and the SHA-256 of every file; a
   failed step keeps its partial logs and the later steps are NOT_RUN
   ([reproduce](REPRODUCE.md)). A run by someone outside the project is item 4.
   Never infer robust performance from a single timing.
4. OPEN. Reproduce numeric Q8 evidence on an independent second host, including
   regressions and complete memory costs (work list A20). `gel-live-lab --witness`
   gives the same score bits on the three CI platforms (PR #69), and memory per
   mutation is measured on one host (A09); hosted CI is not a second host. Keep
   addressed readout and full scan distinct; coordinate views are not
   independent information.
5. PARTIAL. Knowledge retrieval: the published question sets were frozen before
   any run and use a published abstention rule ([external checks](EXTERNAL-CHECKS.md)),
   but the measured builds and their bank are private and there is no public
   encoder, so no retrieval result can be re-run from this tree. Phrase
   matching, source integrity and synthetic numerical ranking are not semantic QA.

## After v0.8.0

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| The v0.8.0 section names the commit of PR #97 | 319b4e8 (PR #98) | none | [v0.8.0](#v080) | None |
| The version boundary, the delivered capabilities and the open review items describe v0.8.0: each capability links what shows it in this tree, historical evidence is listed apart, and each open item is re-checked against the tree; the version boundary and the security policy word the current version so that it holds at a version cutover, before the owner publishes its tag, and after; the status file records v0.8.0 as published, and the supported-versions table of the security policy names the 0.8.0 line instead of 0.7; tests require the first line of the version boundary, the heading of this section, both cells of that table and the checkout line of the quick-check guide to name the workspace version, every relative link and anchor of this file and of the status file to resolve, and the historical test count of the status file to equal its platform record | f20366c (PR #99) | `cargo test -p xtask --test version_line --test doc_links` | [version boundary](#version-boundary), [open review items](#open-review-items), [status](../CANDIDATE-STATUS.md) | An independent reading of this page against the tree; the link test covers these two files only |
| The claim registry grows from 49 to 73 rows: executable rows for the published SHA-256 vectors, the record layouts, exact structural comparison, the size of a Q8 record as the public encoder writes it, and the absence of third-party packages and of network code; separate-gate rows for the tool versions and license, the source headers, the license self-check, Unicode normalization, the generated README, the version line, the goal statement, the commitments and the identical rebuild; measured rows for the crash series, the recorded Q8 runs and two speed sets; independent reproduction as not verified and four more limits as not established; sentences of other pages that name a row are listed with that row and its state, and each must occur exactly once in its file | f8adca6 (PR #100) | `cargo run --locked --offline -p xtask -- claims` | [Claims](CLAIMS.md) | Not an exhaustive map of historical statements; the roadmap rows and the gel-store and gel-reader parts of the README have no row |
| Every `speed-ab` record states the comparison protocol (input, profile with the values of the cargo variables that change code generation, clock boundary, warm-up, order, A/A control, host metadata); `speed-ab-check` refuses a record missing a field or the A/A control, and refuses A and A2 unless both are clean trees of one full commit; `speed-campaign` runs a plan of examples through `speed-ab` and the check and ends `COMPLETE`, `INCOMPLETE` or, for a shortened trial run, `PARTIAL`; four plans re-run the recorded speed sets of 2026-10-20 and 2026-10-25 from public revisions | 2bc603e (PR #101) | `cargo run --locked --offline -p xtask -- speed-ab-check DIR` | [Campaign protocol](EVIDENCE-CAMPAIGN.md#re-running-the-recorded-speed-sets), [speed-ab](SPEED-AB.md) | No re-run under these plans is retained yet; the recorded sets stay as they were measured |
| One command, `xtask witness NEW_DIR`, runs the short public checks on any host and writes one plain-text block (`GEL_WITNESS_2`): tool version, revision and whether the tree was clean, operating system, kernel, CPU, cores, RAM and `rustc` version, and for each step its outcome, exit status, wall time and last result line; a filter replaces host and user names, home and run directories, other absolute paths, network addresses and serial numbers in the step lines, and a separate check refuses the block if this host's names or directories remain anywhere in it; a guide page, the reproduction guide and the reproduction form ask for that block | 4ffab16 (PR #102) | `cargo run --locked --offline -p xtask -- witness NEW_DIR` | [Check on your hardware](CHECK-ON-YOUR-HARDWARE.md), [reproduction form](../.github/ISSUE_TEMPLATE/reproduction.yml) | No block from a host other than the author's is retained yet; the filter removes paths and addresses of other machines only as far as it recognises them |
| The four re-run plans were run once with `xtask speed-campaign` on the author's machine: each ends `SPEED_CAMPAIGN=COMPLETE` and every one of the 33 records passes `speed-ab-check` as measured; against v0.6.1, 105 cases are faster beyond their A/A band, 24 slower and 4 inside, and 102 of the 103 cases recorded faster on 2026-10-20 have a ratio above 1 again, 100 of them beyond their band; the output digests equal the recorded ones except for one tree of the program that counts the files of its own tree | 15b1efd (PR #103) | `cargo run --locked --offline -p xtask -- speed-ab-check DIR` | [Re-runs](speed-results/2026-11-04-reruns/README.md) | One machine, not pinned to cores; a re-run on other hardware |
| Each section of the README leads with a picture drawn from the files of the checkout and one line, with the full wording folded under it; six new animated cards (status, what to look for, the re-run, 0.7 and 0.8 at a glance, the commitments, the memory core), light and dark, each standing still under reduced motion; the words in view drop from 1,695 to 537 | 294a50a (PR #105) | `rustc +1.85.0 --edition 2021 --test tools/readme_presentation.rs` | [Presentation guide](README-PRESENTATION.md) | A local rendering is not a check of the page GitHub shows; on a phone the cards scale down and their small text is hard to read |
| Every wide card of the README also comes 600 wide, with text of at least 20 px, for a phone, and the picture sends a screen up to 600 px wide to it; a row of badges under the banner, each a number read from a file when the README is built and linking to the file it comes from; a card of what one Q8 record holds, as docs/Q8-QUAD.md states it, with the separate commitment of the target quantizer; the record and citation cards play once and hold instead of going blank every 12 s; a new published commitment, the target Q8 Quad quantizer | 79f6ac5 (PR #106) | `rustc +1.85.0 --edition 2021 --test tools/readme_presentation.rs` | [Presentation guide](README-PRESENTATION.md), [Commitments](COMMITMENTS.md) | The phone card is chosen by the reader's browser from the screen width; a viewer that ignores the picture's media rule shows the wide card. The commitment shows that the quantizer's files do not change, not that it works |
| Fault and concurrency coverage of saved files: 20 tests in three new files cut the saved files of gel-history, gel-store and gel-source at every byte (a store of version 1 past its header; a collection over 1 MiB at 12 fields), damage them without changing their length, race two child processes on one path, stop or kill a save and restart, and read beside a writer; properties P36 to P40 join the map, now 40 properties and 91 tests | this change | `cargo test -p gel-history -p gel-store -p gel-source --test faults` | [Faults of saved files](PUBLICATION-FAULT-TESTS.md#faults-of-saved-files-on-disk-and-between-processes) | A power-loss test; 16 of the 20 tests compile on Windows and all 20 on macOS, and they run there only from the CI of this change on, with no recorded report yet; no timing property is covered |
| Sanitized reports of a third revision, 79f6ac5, are recorded. `xtask ci-evidence-import` rebuilds them from a job log and refuses a mismatch with the checksums printed in CI; `xtask ci-evidence-recorded`, in `verify`, refuses an altered report, a missing or extra file, a wrong run or commit, a line outside the report format, a failed strict comparison and a table that differs from the reports. The strict comparison failed on 79f6ac5: 11 platform-only tests were undeclared and one ignored child-process helper was refused, so the declarations now name a Linux condition as well and the helpers are declared | this change | `cargo run --locked --offline -p xtask -- ci-evidence-recorded` | [Recorded reports](evidence-ci/README.md), [CI evidence](CI-EVIDENCE.md), [gates](GATES.md) | Reports of hosted CI runs of one revision each, not of every revision; hosted CI is not an independent second host. The comparison runs when a revision is recorded, not in a CI job, and reads the debug workspace lists only |
| Memory accounting of public operations: `xtask memory-run` starts each operation as a child process at least five times and records payload, peak mapped and resident memory, page faults, threads and host conditions with min, median and max; `memory-check` refuses a missing field, a non-zero exit, a malformed number, too few samples, a summary that does not recompute and a peak below a resident payload | this change | `cargo run --locked --offline -p xtask -- memory-check FILE` | [Instrument and limits](MEMORY-ACCOUNTING.md) | No run is retained yet; the record of a public revision follows. Linux only; the peak is read from `/proc` during the run, not taken from `wait4`; no time is recorded, so latency is not read beside memory |
| The claim registry grows from 74 to 85 rows and from 35 to 105 tied sentences: executable rows for the gel-store file refusals, Reader16, exact progressive Top-K and the source-builder limits; separate-gate rows for the selftest, the real-source readout, the recorded CI reports and the faults of saved files; measured rows for the recorded history kill series and the Q8 R2 comparisons; the query-rate limit as not established; recorded numbers tied to their evidence files | this change | `cargo run --locked --offline -p xtask -- claims` | [Claims](CLAIMS.md) | Not an exhaustive map of historical statements; the roadmap rows, the release notes, videos and the measurement tables of other pages have no row |
| Thirteen of the fourteen phone cards drop, from 16,620 to 10,799 px at 600 wide in all, each at most 1,180 tall (the header card is unchanged): repetition and decoration are cut, the phone speed card shows totals and not one row per program, and the phone license card shows five of the seven quotes, with the limit on evaluation | this change | `rustc +1.85.0 --edition 2021 --test tools/readme_presentation.rs` | [Presentation guide](README-PRESENTATION.md) | A local rendering is not a check of the page GitHub shows; the q8 card is not shortened |

## v0.8.0

Everything merged after v0.7.0 (PR #91 to PR #97). Every program says what it
is and under which license; every GEL RAM-owned `.rs` file carries a two-line
source header naming the license and the Required Notice, which a gate checks;
`gel-license` reads a distribution tree, and a user's own answers about one
use, against the license; the license guides follow the words of the LICENSE;
the README is generated as a whole and gives one command for each claim it
makes; and one more private directory is fixed by a published commitment.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Every program prints one identity line for `--version`, and for `--license` the license name, the Required Notice, the NOTICE summary and a line saying that third-party material in the program keeps its own license; the gel-evidence batch session record names the license | 06d2959 (PR #91) | `cargo run -p gel-cli -- --version` | [binaries](BINARIES.md#version-and-license-of-each-program), [batch interface](EVIDENCE-BATCH.md) | The printed lines name the license; the LICENSE file, not this output, holds the terms |
| Every GEL RAM-owned `.rs` file begins with a two-line source header, the license name and the Required Notice, checked in `verify`; the generated Unicode tables are exempt; `speed-ab` sets an exact header aside when it compares a carried example with an older tree; the measured bytes of three examples are kept beside their manifest and the current examples must equal the header and those bytes | 06d2959 (PR #91) | `cargo run -p xtask -- source-headers` | [gates](GATES.md), [contributing](../CONTRIBUTING.md), [A/B tool](SPEED-AB.md), [evidence collection](evidence-collection/README.md) | v0.7.0 and earlier releases carry no header; source hashes in earlier speed manifests describe their measured revision, not the files with headers |
| `gel-license check DIR` reports whether a tree has the exact LICENSE, the Required Notice and intact source headers; `gel-license use ANSWERS_FILE` maps the user's own yes, no or unsure answers to the LICENSE sentences its rules cite; `verify` runs `check` on this repository; the decision guide lists worked cases | 06d2959 (PR #91) | `cargo run -p gel-live-lab --bin gel-license -- check .` | [license decision guide](LICENSE-DECISION.md) | Not legal advice and not a certification (`LEGAL_CERTIFICATION=NO`, LEGAL_APPROVED=NO); the answers are the user's own and are not checked; `check` reads only the license, notice, header and crate-name items listed; several worked cases say "ask the Licensor" |
| NOTICE and `LICENSING.md` no longer list removed files among those that quote Wikipedia; `LICENSING.md` says the LICENSE is the complete public grant | 7e836c7 (PR #92) | `cargo run -p xtask -- verify` | [NOTICE](../NOTICE), [licensing](../LICENSING.md) | None; no license term changes |
| Published commitment: private source tree 2 | bb71441 (PR #93) | `cargo run -p xtask -- commitment-check MANIFEST DIR` | [commitments](COMMITMENTS.md) | The manifest and files stay private until shown |
| The CI `verify` job has 40 minutes instead of 20 | 3d48805 (PR #94) | none | [CI workflow](../.github/workflows/ci.yml) | None |
| The README is generated as a whole; each claim at the top has one command; its counts, the crash series and the timing chart are read from their files, and a series that disagrees with its rows is refused; the license card quotes the LICENSE word for word | ed053c7 (PR #95) | the steps of the [readme-presentation job](../.github/workflows/readme-presentation.yml) | [presentation guide](README-PRESENTATION.md) | Independent reading of the README against the commands it names |
| The license guides follow the words of the LICENSE: the Evaluation purpose condition, the full Monetized Use definition, the grant verbs, the Required Notice lines and a Commercial Agreement signed by both sides | e635149 (PR #96) | none | [licensing](../LICENSING.md), [decision guide](LICENSE-DECISION.md) | None; the LICENSE is unchanged |
| Every file that names the current version is tied to the workspace version by a test, and only the notes of that version stay in the root; `docs/TRY-IT.md` follows the release instead of v0.6.0 | 5131104 (PR #97) | `cargo test -p xtask --test version_line` | [try it](TRY-IT.md) | None |

## v0.7.0

Everything merged after v0.6.1 (PR #72 to PR #89). gel-source computes SHA-256
and Unicode NFC itself, so `Cargo.lock` lists no third-party package, and NFC
follows Unicode 16.0.0, the Unicode version of the Rust 1.85.0 standard library;
answer-bench stops counting the right record as an error; the goal is stated in
one file; one command produces the reproduction witness; the release package can
be built twice and compared; a digest can fix a directory that is not published;
many paths are faster with the same output bits; and a bundle or collection load
needs about half the peak memory.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Own SHA-256 (FIPS 180-4, safe Rust) in gel-source; every published pin unchanged | 96a9ddf (PR #72) | `cargo test -p gel-source` | [source readout](SOURCE-READOUT.md) | Hashing was about 6.3 times slower than sha2 with the processor's SHA instructions on the author's machine; after PR #82 it is 1.5 times faster than at this commit and still 2.6 to 3.6 times slower than v0.6.1 (table below) |
| sha2 leaves the tree, test dependencies included: ten fewer third-party packages | 27e4d53 (PR #73) | `cargo tree -e normal,build,dev --workspace -i sha2` (no package matches) | [dependency inventory](DEPENDENCY-INVENTORY.md) | — |
| Own Unicode NFC from UCD 15.1.0 tables, regenerated byte for byte in CI; the last two third-party packages leave | 26ee1a2 (PR #74) | `cargo run -p xtask -- ucd-tables` | [document readout](DOCUMENT-READOUT.md#unicode-normalization) | Matching can differ from v0.6.1 only on characters assigned in Unicode 16.0 or 17.0; long runs of combining marks and decomposed Hangul jamo were slower than v0.6.1 in every measured round, and after PR #82 they are not (1.00 to 6.1 times the speed of v0.6.1, table below) |
| answer-bench: a returned passage that is the question's own source passage, or that answers a question the bank does answer, is counted apart, not as an error | c057033 (PR #76) | `cargo run -p xtask -- answer-bench check` | [set v1](answer-or-abstain/README.md), [set v7](answer-or-abstain-v7/README.md) | The rule was added after the recorded runs; the set pages give the earlier numbers |
| The goal stated once, in docs/GOAL.md, with four commitments in a fixed shape that the README repeats | 3fb8e1f (PR #75) | `cargo test -p xtask --test goal_statement` | [goal](GOAL.md) | The text AI is a goal, not a result |
| One command runs the reproduction steps and prints one block with its own SHA-256 | 9d749b5 (PR #77) | `cargo run -p xtask -- witness NEW_DIR` | [reproduce](REPRODUCE.md) | The history crash series inside it needs at least one of its five control kills to leave a torn file; since PR #83 those kills are drawn from inside the control run |
| The release package built twice from the same revision, compared byte for byte | 62d3cb2 (PR #78) | `cargo run -p xtask -- rebuild-check NEW_DIR` | [binaries](BINARIES.md) | Identical on each CI platform; a rebuild with a different system C toolchain is not expected to match a published package |
| A SHA-256 published now that fixes the exact contents of a directory that is not published, and the check for it | aa5602b (PR #81) | `cargo run -p xtask -- commitment-check MANIFEST DIR` | [commitments](COMMITMENTS.md) | A commitment says nothing about what the contents are or whether they are correct |
| Faster paths with the same output bits, one grammar for timing examples and identity programs, and a Rust A/B tool with an A/A control | 876247f (PR #82) | `cargo run -p xtask -- speed-ab ...` | [timing examples](SPEED.md), [A/B tool](SPEED-AB.md) | Medians on one machine; paths that hash every byte are still slower than v0.6.1 (table below) |
| CI reliability: control kills of the history crash series drawn from inside the control run; 40 minutes for portable-build | e1f2c9c (PR #83) | `cargo run -p xtask -- history-crash-series 5 20261010 5` | [reproduce](REPRODUCE.md) | Neither failure was in the code under test |
| Published commitments: private data set 1, then private data sets 2 to 5 and sealed sample 1 | 29875ec (PR #84), d0f1a62 (PR #85) | `cargo run -p xtask -- commitment-check MANIFEST DIR` | [commitments](COMMITMENTS.md) | The manifests and files stay private until shown |
| The collection-review fixture cut to the scope and limits of a withdrawn bundle; embedded tool metadata removed from one image | bb3da1b (PR #86) | `cargo run -p gel-source --example collection_review` | [fixture provenance](../crates/gel-source/fixtures/collection-review/PROVENANCE.md) | All 24 cases give the same result |
| Second speed package with the same output bits: file reads reserve from the file length, the ASCII check runs in the line-end pass, `--literal` reads in bounded memory, gel-bench checks in one pass, store reads overlap with digest and decode, repeated passage ranges hashed once | 90bd091 (PR #87) | `cargo run -p xtask -- speed-ab ...` | [timing examples](SPEED.md) | Measured with an A/A control on a loaded machine; the ratios are directions, not figures to quote |
| NFC tables from UCD 16.0.0, the Unicode version of the Rust 1.85.0 standard library, so the search path follows one Unicode version | 3cf1be5 (PR #88) | `cargo run -p xtask -- ucd-tables` | [document readout](DOCUMENT-READOUT.md#unicode-normalization) | Matching can differ from v0.6.1 only on characters assigned in Unicode 17.0 |
| Third package with the same outputs: a bundle is loaded without a second copy of its text, a collection snapshot is loaded with its document texts in one shared buffer instead of copied, the store reader digests each block on the calling thread again and decodes it beside the digest on a second thread, and the gel-bench check example pins every output key | 331f494 (PR #89) | `cargo run -p xtask -- speed-ab ...` | [quiet-machine records](speed-results/2026-10-25-quiet-machine/README.md) | Measured on a quiet machine with an A/A control: the slower v1 store check after PR #87 is back to its earlier speed; peak memory of a 64 MiB bundle load and of a 64 MiB collection load about halved; a loaded document that is replaced or removed keeps its bytes in the shared buffer while any document of that load lives |

The PR #74 row describes the tables as merged then. Since then the NFC tables
are generated with the same command from UCD 16.0.0, the Unicode version of the
Rust 1.85.0 standard library, which lowercases and splits words, so the search
path follows one Unicode version; the 15.1.0 files stay for tests that every
15.1.0 result and property is kept ([document readout](DOCUMENT-READOUT.md#unicode-normalization)).
Matching can now differ from v0.6.1 only on characters assigned in Unicode 17.0.

### Measured against v0.6.1

`xtask speed-ab` with A and A2 both at the v0.6.1 tag and B at the library code
of `main` after the changes above: the same timing example in all three trees,
3 rounds of 9 runs with the order rotated, medians, on one 24-thread Linux
machine that other work was loading. The runs were not pinned to cores, so the
A/A column is the noise to read each ratio against. The manifests, summaries
and every run's output are in
[speed-results/2026-10-20-against-v0.6.1](speed-results/2026-10-20-against-v0.6.1/README.md).
Each example printed the same output digest in A, A2 and B, except
`speed_top1_many` (its API does not exist in v0.6.1) and `speed_tree_gates` (it
counts the files of its own tree). Speed-up is the v0.6.1 time divided by the
current time; A/A is the second v0.6.1 copy divided by the first.

| Area (example) | Speed-up against v0.6.1 | A/A |
|---|---|---|
| CRC-64 (`speed_crc`, `speed_crc_large`) | 4.9 to 47.8 | 0.95 to 1.00 |
| Store build, open, verify and write (`speed_store_io`, `speed_store_crc`) | 4.8 to 46.5 | 1.01 to 1.14 |
| Structural encode (`speed_structural_encode`) | 1.4 to 103 | 0.96 to 1.02 |
| Witness scores (`speed_witness`) | 34.7 | 1.00 |
| Q8 read and scan (`speed_q8_read`, `speed_q8_queries`) | 1.3 to 5.7 | 0.94 to 1.05 |
| Q8 masks and views (`speed_q8_masks`, `speed_views`) | 0.97 to 35 | 0.98 to 1.24 |
| History (`speed_history`) | 1.1 to 18 | 0.96 to 1.08 |
| Document and collection search (`speed_search`) | 1.4 to 14.8 | 0.95 to 0.97 |
| NFC-heavy lines and documents (`speed_nfc_fast`) | 1.0 to 29 | 0.89 to 1.07 |
| Lowercasing non-ASCII lines (`speed_lowercase`) | 1.2 to 2.8 | 0.87 to 1.08 |
| Top-K (`speed_topk`) | 1.0 to 1.7 | 0.97 to 1.01 |
| Whole `gel-bench` run (`speed_bench_physics`) | 3.1 to 3.9 | 0.99 to 1.01 |
| Whole `gel-physics` run (`speed_bench_physics`) | 1.06, within the A/A spread | 0.95 |
| SHA-256 (`speed_sha256`) | **0.28 and 0.39** | 0.96 to 0.98 |
| Paths that hash every byte: bundle, collection, corpus and catalog load, backup (`speed_source_load`, `speed_load_overlap`, `speed_corpus_load`, `speed_catalog_parse`, `speed_backup_restore`) | **0.27** to 6.5 | 0.81 to 1.25 |

v0.6.1 hashed with the sha2 crate, which uses the processor's SHA instructions.
The workspace now has no third-party package and no `unsafe`, so SHA-256 is a
portable implementation; the cases in bold are slower than v0.6.1 for that
reason, and the slowest ones are those that hash the whole input.

### Measured on a quiet machine

`xtask speed-ab` on 2026-10-25 with no other measurement running: package 2
with A and A2 at `bb3da1b` (before PR #87) and B at `90bd091` (after it);
package 3 with A and A2 at `bb3da1b` and B at PR #89, for the bundle
load at `3cf1be5`, and for the collection load at `37d509d` (PR #89
before the collection change). 3 rounds of 9 runs (the bundle and the 64 MiB
collection load: 3) with the order rotated, medians, not pinned to cores. The manifests, summaries and every run's output are in
[speed-results/2026-10-25-quiet-machine](speed-results/2026-10-25-quiet-machine/README.md);
every example printed the same output digest in A, A2 and B.

| Area (example) | Speed-up | A/A |
|---|---|---|
| Package 2: `--literal` (`speed_literal`) | 1.28 to 9.47; refused input 0.99 | 0.99 to 1.02 |
| Package 2: repeated passage ranges (`speed_quote_ranges`) | repeats 8.8 to 24.0; distinct ranges 0.97 to 1.00 | 0.99 to 1.05 |
| Package 2: file reads (`speed_read_regular`) | reads 1.03 to 1.85; bundle load, digest and refused input 0.98 to 1.00 | 0.89 to 1.24 |
| Package 2: ASCII check (`speed_line_ascii`) | 1.02 to 1.61 | 0.97 to 1.03 |
| Package 2: gel-bench checks (`speed_bench_checks`) | 1.23 to 1.60; smallest input 0.99 | 0.97 to 1.03 |
| Package 2: store open and verify (`speed_store_overlap`) | open 1.11 to 1.16; verify from 16 MiB 1.16 to 1.62; **v1 verify 0.64**; verify of 1 and 8 MiB 0.93 and 0.97 | 0.99 to 1.01 |
| Package 3: store open and verify (`speed_store_overlap`) | open 1.13 to 1.18; verify from 16 MiB 1.20 to 1.65; v1 verify 1.08; verify of 1 and 8 MiB 1.00 and 0.99 | 0.99 to 1.01 |
| Package 3: bundle load (`speed_bundle_memory`) | 1.06; peak resident memory 134 072 KiB to 68 612 KiB (median of three runs) | 0.99 |
| Package 3: collection load (`speed_collection_memory`, `speed_source_load`) | 64 MiB load 0.99; 32 MiB load 1.09; the other source cases, which do not run `Collection::load`, 0.99 to 1.18, a spread the collection change does not explain; peak resident memory of the 64 MiB load 133 160 KiB to 68 640 KiB (median of three runs) | 0.96 to 1.04 |

After PR #87 the store check digested each block on a new thread. A v1 file has
one sequential digest, so its check ran at the speed of the core that thread
was given: this machine has 8 logical CPUs at up to 5.16 GHz and 16 at up to
3.29 GHz, and 3.29 / 5.16 = 0.64. In package 3 the calling thread digests again,
the read of the next block runs on a thread, and on a load the decode runs on a
second one.

## v0.6.1

An evidence release. The public Q8 reader takes its 256 cosine values from one
frozen table, so Q8 scores are the same bits on every platform, and
`gel-live-lab --witness` checks that; three checks show what a stopped save
leaves and how the record-history format refuses changed bytes. gel-source makes
`PublicationIo` and `write_file_new_with` public; the file formats and
third-party dependencies are unchanged from v0.6.0.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| README replays: the sixth scene now records `gel-live-lab --literal` saving a record and `--reopen` checking it under the pin, instead of the answer comparison with grep; all six replays were recorded again on the author's Linux host, with card lines broken between words | PR #66 | `tools/readme_media.rs` | [gallery](../media/gifs/README.md), [source and hashes](../media/gifs/MANIFEST.txt) | human visual acceptance of the replays; a recording on a host outside the project |
| Record history kill series: `xtask history-crash-series` kills a writer that saves one growing history with `write_atomic`; the file must open and equal the last acknowledged history or the next one, never a partial file, and every trial resumes to the uninterrupted bytes; an in-place control writer must be caught leaving a partial file; registered as the `SEPARATE_GATE` claim `record-history-kill-series`, while `record-history-durability` stays `NOT_ESTABLISHED` for a power cut | PR #67 | `xtask history-crash-series` (5 trials in `xtask verify` on Unix) | [crash series](CRASH-SERIES.md#the-record-history-one-file-replaced-while-it-grows) | a power-loss test; runs by independent operators |
| GELHIS01 format mutation matrix: 151 mutants of one 338-byte record-history file, with their CRC64 fields recomputed, read by the library and by a reference reader written from the format description; every row agrees, 14 valid changes are accepted with the same states, and a reader without the canonical-form rule is caught; kept in its own report so the three-format matrix stays byte for byte; registered as the `SEPARATE_GATE` claim `history-format-mutations`. `quad_evidence --generate` now writes a flushed temporary file and links it into place instead of writing the new file directly | PR #68 | `xtask history-mutation-matrix` (in `xtask verify`) | [format mutation matrix](MUTATION-MATRIX.md#gelhis01-record-history-files), [report](evidence-mutation/history-matrix-r1.txt) | an independent reader written by someone outside the project; a kill test of `--generate` |
| Same score bits on every platform: `gel-live-lab --witness` reads the 256 cosine values and 1,048,576 scores of a synthetic bank through the public `Reader` and compares them with pins recorded on the author's Linux host. Its first run found macOS (arm64) and Windows (x86_64) giving a different last bit at 13 and 7 of the 256 levels; the reader, the reference reader and the Q8 examples now use one frozen table, `grid::COS256_BITS` (the glibc 2.39 values, so Linux results are unchanged), and the platform's own cosine is shown for information only; registered as the `SEPARATE_GATE` claim `q8-score-bits-across-platforms` | PR #69 | `gel-live-lab --witness` (in `xtask runtime-examples` on every CI platform), the table test in `cargo test` | [same bits on your machine](LIVE-LAB.md#same-bits-on-your-machine), [Q8 quad](Q8-QUAD.md) | runs on other CPUs and operating systems by people outside the project |
| A Q8 record save stopped at known points: a child process ends the save of a 1,164-byte literal record at 23 known points (pieces of the temporary file, its flush, the link, the directory flush), each confirmed by a marker; the new path always holds nothing or the whole record under its pin, while a naive writer stopped at the same points leaves part of a record at 19 of 21; cut files are refused and a zeroed 4 KiB block is caught by the pin only. `gel_source::PublicationIo` and `write_file_new_with` become public for such tests; registered as the `SEPARATE_GATE` claim `q8-record-save-stopped` | PR #70 | `cargo test -p gel-live-lab --test torn_write` | [stopped at known points](PUBLICATION-FAULT-TESTS.md#a-q8-record-save-stopped-at-known-points) | a power-loss test; a review of the publisher by someone outside the project |

## v0.6.0

A documentation and evidence release with one new command mode,
`gel-live-lab --literal` (with `--save` and `--reopen`), its `literal` module in
gel-live-lab, one new function in gel-source, `write_file_new`, and one new
library crate, `gel-history`, with its own file format GELHIS01; the other public
file formats and third-party dependencies are unchanged from v0.5.3. `answer-bench`
gains question sets v6 and v7, with a third kind of question without an answer.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Answer-or-abstain set v6: 990 new questions frozen before any run, used internally before (an experiment and an analysis of its errors) without changing any build or setting; the v4 build unchanged beside two BM25 engines, the third held-out set with the same result | 8bfb3df (PR #62) | `xtask answer-bench check` | [set v6](answer-or-abstain-v6/README.md), [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with) | questions from an independent writer; a held-out check of the next change |
| Answer-or-abstain set v7: 982 new questions with an answer and 599 whose answer is not in the bank (real topics outside it and invented subjects), frozen before any run; the v4 build unchanged beside two BM25 engines and a candidate change that was rolled back; answer-bench counts a third kind of question without an answer | v0.6.0 | `xtask answer-bench check` | [set v7](answer-or-abstain-v7/README.md), [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with) | questions from an independent writer; a held-out check of the next change |
| First screen returns to the record: one Q8 record with four exact views, exact bytes or a refusal, the crash series and commands to run; question sets and search-engine references moved to docs/EXTERNAL-CHECKS.md with a section on what has nothing to compare with; language-model pages, films, side-by-side records and the four claim rows of those comparisons removed from the tree (in the git history); the GEL numbers of set v2 registered on their own as `gel-answer-set-v2` | v0.6.0 | `tools/readme_presentation.rs`, workflow `readme-presentation`, `xtask verify` | [README](../README.md), [external checks](EXTERNAL-CHECKS.md) | a human review of the rendered page |
| Your bytes in one record: `gel-live-lab --literal FILE` places the first 1,024 bytes of one file into one Q8 record, shows its four views, restores each with 0 different bits and rejects its 1,164-byte file under the pin after a 1-byte change; literal bytes, not GEL knowledge printing; registered as the executable check `q8-literal-record` | v0.6.0 | `cargo test --locked --offline -p gel-live-lab`, `xtask claims` | [live lab](LIVE-LAB.md#your-own-bytes-in-one-record) | a run on a host outside the project |
| Record history: `gel-history` stores every state appended to one 128-byte record as a literal copy or as the XOR residual from the state before it, at most two residuals from a literal, rebuilds each bit for bit, also from its reopened GELHIS01 file, and accepts only the bytes its encoder writes; registered as the executable check `record-history-exact`, with `record-history-durability` as `NOT_ESTABLISHED` | v0.6.0 | `cargo test --locked --offline -p gel-history`, `xtask claims`, `cargo run --locked --offline --release -p gel-history --example record_history` | [record history](RECORD-HISTORY.md) | a review of the format by someone outside the project; a power-loss test of `write_atomic` (its kill series came after v0.6.0) |

## v0.5.3

A documentation and evidence release; public file formats and third-party dependencies
are unchanged from v0.5.2. `answer-bench` gains question set v5; the disclosure gate no
longer carries a word list; documentation is English only, with a guard.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| The project goal restated: a text AI that answers in Polish or English from its sources, or says it does not know, and holds a free conversation in both languages; still a goal, not a result | ba1884b (PR #45) | `tools/readme_presentation.rs`, workflow `readme-presentation` | [README](../README.md), [start here](START-HERE.md) | a public measurement of conversation in both languages |
| One license set with pinned file hashes, Wikipedia text attributed under CC BY-SA 4.0, documentation in English only with a guard, private results reported by their outcome only | aec70d5 to cfec9fb (PR #46 to PR #53) | `xtask verify` (licensing, disclosure) | [licensing](../LICENSING.md), [license pins](LICENSE-PINS.md), [third-party notices](../THIRD-PARTY-NOTICES.md) | an independent legal review of the license |
| README numbers made consistent: each result card names its question set and build, exact counts for the answer verdict, earlier versions kept out of the tree | c49d2c3, 804db2f, dfb2857 (PR #54 to PR #56) | `tools/readme_presentation.rs`, `xtask claims` | [README](../README.md), [measured progress](MEASURED-PROGRESS.md) | — |
| Answer-or-abstain set v5: 987 new questions frozen before any run; the v4 build unchanged and one candidate change beside two BM25 engines; the candidate rolled back under the rule fixed in advance | b3970bb (PR #57) | `xtask answer-bench check` | [set v5](answer-or-abstain-v5/README.md), [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with) | questions from an independent writer; a further held-out set for the next change |
| Fixes from three reviews of the v0.5.3 draft: the cut of the answer-bank slots to 167,854 passages stated, a timing ratio rounded correctly, one frozen review fixture sentence reworded with its old SHA-256 recorded, the README example card credited under CC BY-SA 4.0, stale sentences and one broken link | f64596a (PR #59) | `xtask verify`, `cargo run -p gel-source --example collection_review` | [measured progress](MEASURED-PROGRESS.md), [fixture provenance](../crates/gel-source/fixtures/collection-review/PROVENANCE.md) | — |
| Fixes from a second review of the v0.5.3 draft: the passages removed from each slot stated; Polish scope change −26–27% | PR #60 | `xtask verify` | [measured progress](MEASURED-PROGRESS.md) | — |

## v0.5.2

A documentation and evidence release; public file formats and third-party dependencies
are unchanged from v0.5.1. `xtask` gains `answer-bench` (re-scoring the recorded answers
of the v1 to v4 sets), `crash-series` and the public disclosure gate `disclosure`.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Public evidence and CI hardening: fail-closed workflow permission and action-pin checks, full benchmark answer inventories, 40,000 bounded format mutants | 7a4d237 (PR #19) | `xtask verify` | [hardening audit](AUDIT-HARDENING.md) | independent second-host acceptance; human visual review; power-loss guarantees |
| Answer-or-abstain set: 160 frozen questions with the recorded answers of GEL and three models, re-scored under strict and published rules, registered as a `SEPARATE_GATE` claim | 5c6342d (PR #31) | `xtask answer-bench` | [set and method](answer-or-abstain/README.md) | the answers are recorded, not re-run; a larger, independently written question set |
| No-answer control: 80 questions with no correct answer, GEL beside three language models, one run, registered as a `MEASURED_LOCAL` claim; the registry row was withdrawn in v0.6.0 | 57acddf (PR #30) | `xtask claims` | removed from the tree; in the git history | GEL answered 6 of 40 false premises; independent reproduction |
| README "Evidence you can inspect": figures drawn from the recorded public data, goals labelled as not achieved | fd7a469 (PR #32) | `tools/readme_presentation.rs`, workflow `readme-presentation` | [README](../README.md) | a human review of the rendered page |
| The project goal stated as a goal, not a result | 0dc4c64 (PR #34) | `tools/readme_presentation.rs` | [README](../README.md) | — |
| What is different here and what can be checked without the private code; pages brought up to date after an external review | 7367313 (PR #35) | `xtask docs-refs`, `xtask answer-bench` | [README](../README.md), [verified results](VERIFIED-RESULTS.md) | an independent reading by a second reviewer |
| Crash series: a growing collection in the public tool killed at random moments; acknowledged snapshots, partial snapshots and resume checked, registered as a `SEPARATE_GATE` claim; the author-run series on the private store registered as `MEASURED_LOCAL` | 1075225 (PR #36) | `xtask crash-series` (5 trials in `xtask verify` on Unix) | [crash series](CRASH-SERIES.md) | a power-loss test; runs by independent operators |
| Answer-or-abstain set v2: 394 new frozen questions, GEL beside three language models in one run, re-scored by `answer-bench`; registered as a `MEASURED_LOCAL` claim; in v0.6.0 that row was withdrawn and GEL's numbers alone registered as `gel-answer-set-v2` | 1f27a89 (PR #37) | `xtask answer-bench` | [set v2](answer-or-abstain-v2/README.md) | a question set written independently of the project; an independent reproduction |
| Answer-or-abstain set v3: 979 new frozen questions; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank, all four GEL runs on the set published; `answer-bench` also checks precision with Wilson intervals, the split by language and paired exact tests; registered as a `MEASURED_LOCAL` claim | 4690933 (PR #42) | `xtask answer-bench` | [set v3](answer-or-abstain-v3/README.md), [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with) | a clean held-out set for the final GEL build; a tuned BM25 baseline; an independent reproduction |
| Public/private boundary: author-run measurements of the private implementation labelled as such on every results page, engine telemetry removed from public pages | 0a0c089 (PR #38) | `xtask docs-refs` | [measured progress](MEASURED-PROGRESS.md) | an independent reading of the boundary |
| Public disclosure gate in `xtask verify`: private paths, internal names (compared by hash), network addresses, credentials and e-mail addresses other than the published contacts | 2f7876a (PR #39) | `xtask disclosure` | [gate](PUBLIC-DISCLOSURE-GATE.md) | a review of meaning; the gate finds only what it is told to look for |
| Paths for people who do not write code: break it, evidence integrity, corpus and review, with issue forms and labels | 579032d (PR #40) | — | [contributing](../CONTRIBUTING.md) | first outside reports |
| Answer-or-abstain set v4: 985 new questions written after the tested change was fixed, frozen before any run; passages searched with their article title and section heading; GEL at two settings beside Tantivy BM25 and SQLite FTS5 on the same bank and on the v3 bank; registered as a `MEASURED_LOCAL` claim | 0190687 (PR #43) | `xtask answer-bench` | [set v4](answer-or-abstain-v4/README.md), [what has nothing to compare with](EXTERNAL-CHECKS.md#what-has-nothing-to-compare-with) | a released build; a tuned BM25 baseline; an independent reproduction |

## v0.5.1

A documentation and version release; public file formats and dependencies are
unchanged from v0.5.0. `xtask verify` gains one documentation check (the summary
of the review work list below).

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Answer-verdict and natural-question results of the private implementation, registered as `MEASURED_LOCAL` claims | 739c452 | `xtask claims` | [measured progress](MEASURED-PROGRESS.md) | independent reproduction; a larger, independently written question set |
| GEL beside three language models (closed book) on the same 80 frozen questions, one run, registered as a `MEASURED_LOCAL` claim; the registry row was withdrawn in v0.6.0 | b85ce20 (PR #29) | `xtask claims` | removed from the tree; in the git history | independent reproduction; a larger question set |
| Review work list summary recounted from its table on every verify run | PR #33 | `xtask roadmap` | [review work list](#review-work-list-and-next-acceptance) | none beyond the check itself |

## v0.5.0

Finishing a tool, measuring performance and independent acceptance are three
different events. This table records the first two with their commits; the last
column says what acceptance still needs. The A01–A24 list below keeps its own
states.

| Change | Commit | Command | Evidence | Remaining acceptance |
|---|---|---|---|---|
| Limits stated without naming private components | 6b3b6b9 | — | current documents | owner-approved content allowlist and manual review of meaning; history, forks and copies keep earlier text |
| Network-isolation evidence | 8693840, 9937e1a, 9555b6e, 133d225 | `xtask isolation-check` | CI fails it on the networked runner and passes it in a namespace | Linux only; macOS and Windows report NOT_VERIFIED |
| Batch mode, schema gel-evidence/1 | 40c107a; peak_rss_kb in dc38282 | `gel-evidence --batch` | [schema](EVIDENCE-BATCH.md), batch tests | review by an outside consumer of the schema |
| Backup, restore, withdraw, delete, clear | fc710d4 | `gel-backup` | [backup](BACKUP.md), unit and CLI tests, full-disk CI | Windows directory durability not confirmed; independent review |
| Format mutation matrix | 855af81 | `xtask mutation-matrix` | [matrix r1](evidence-mutation/matrix-r1.txt), checked by verify | finite matrix on small fixtures, not exhaustive |
| Comparison with grep and sha256sum | dc38282, 940f5c0 | `xtask bench-compare` | [run r1](evidence-bench-r1/summary.txt) | one host; an independent second host; does not close A09 (single-mutation costs) |
| GEL search times beside the baseline | 1738220 | — | README table, author-reported | a same-task comparison (A21) before any speed-up factor |
| Public verification suite runner | 80f91d7 to 133d225; counts and `--strict` in 4cbfff5 | `xtask reproduce` | REPRODUCTION.txt of an owner run at 133d225: 3 of 3 steps passed, isolation verified | an independent operator on a second host (A20) |
| Documentation review fixes: work branch, historical baseline, security surface | 7bcca67 | — | README, TRY-IT, REPRODUCE, SECURITY | — |
| Still of the update, restart and corrupted-copy scenario | 3c088cc | — | [screenshot](../media/evidence-lab/03-update-restart-102s.png), final screen of an AI-reviewed recording; the recording is not published | — |
| Binary packages for Linux, macOS and Windows, built only in CI | 3c088cc | `xtask package-binaries`, workflow `binaries` | [binaries](BINARIES.md): license files checked against the inventory, build-path scan, per-platform smoke run, checksums; provenance attestation on a hand-started run | first owner-started run; packages are unsigned and checked only on their build runners |
| Fresh paired timing and fresh-process RSS of one mutation (A09) | 306c7f1 | `xtask mutation-campaign` | [campaign r1](evidence-mutation-campaign-r1/CAMPAIGN.txt), [results and limits](MUTATION-COMPARISON.md) | one host, 10 processes per combination; a second host |
| Private previews described only by scope and limits; removed from the tree on 2026-10-04 | 1f7693c, ffa85b6 | — | the public Evidence Lab film keeps its full [transcript](../media/TRANSCRIPTS-EN.md) | — |
| Property-to-test map (A07) | fc5cf90 | `xtask ci-evidence` | [map](PROPERTY-TESTS.md): 35 properties, 71 tests, one `PROPERTY` line per row in each platform report | properties without a row are not claimed untested; a row shows the tests ran, not full coverage |
| Claim registry for the v0.5.0 tools and every README timing table (A19) | 118e86d | `xtask claims` | [claims](CLAIMS.md) | not an exhaustive map of historical statements |
| README visual entry: six edited log replays of recorded public workflows | 034cd33 (PR #14) | `tools/readme_media.rs` | [gallery](../media/gifs/README.md), [source and hashes](../media/gifs/MANIFEST.txt) | edited replays, not wall time; the one-off publishing workflow is not part of the release |
| Multimedia README, animated citation diagram, checked facts and workflow badges; script-free full-page edition | 9e1b63d (PR #16), b8ed840 (PR #17) and the animated-graphics pull request | `tools/readme_presentation.rs`, workflow `readme-presentation` (read-only check) | [presentation guide](README-PRESENTATION.md), [full-page edition](../README-MULTIMEDIA.html) | built locally and rechecked in CI; a human review of the rendered page in browsers |

## How to read the review work list

Implementation state and acceptance state are separate. `DONE_SCOPED` means the
change and its checks are in the repository at this revision, within the stated
scope; it is not independent acceptance. `PARTIAL` means a stated part is
implemented or documented while an acceptance condition remains open. `OPEN`
means this public inventory does not establish the implementation or evidence.
`OWNER` means the remaining step is a repository setting or owner decision, not
code. No completion percentage is calculated from unequal tasks.

The identifiers A01 to A24 retain the order of the 24-point external review.
Evidence links are contracts and entry points, not blanket PASS labels. For each
accepted item, retain the implementation commit, fixture or corpus hash, exact
command, expected result and counterexample, actual result, platform/profile,
run attempt, reviewer and remaining exclusions. A second-host reproduction must
identify its independent operator rather than invent one.

## Review work list and next acceptance

Summary: 14 DONE_SCOPED · 6 PARTIAL · 3 OPEN · 1 OWNER.

| ID | State | Work and next acceptance condition | Evidence / entry point |
|---|---|---|---|
| A01 | DONE_SCOPED | Historical source/lockfile mapping and a public R1 reproduction procedure exist; old hashes are kept and both positive and changed-source checks pass. | [Measured-source guide](evidence-collection/README.md) |
| A02 | DONE_SCOPED | CI rechecks the retained R1 dataset and rejects missing, duplicated or inconsistent evidence. This is not a new timing campaign. | [R1 negative checks](R1-NEGATIVE-CHECKS.md) |
| A03 | DONE_SCOPED | Bounded quote context and omission flags with negation, condition, unit and line-ending regressions; no claim of unrestricted context understanding. | [Quote context](QUOTE-CONTEXT.md) |
| A04 | DONE_SCOPED | Runtime-example sequencing checks each native exit status, including first/middle failures and misleading PASS output. | [Process tests](../xtask/tests/process_sequence.rs) |
| A05 | PARTIAL | Decode report and an AI review of every visually distinct frame of all three films exist. A human start-to-finish review remains open. | [Media review](MEDIA-DECODE-REVIEW.md) |
| A06 | DONE_SCOPED | The recorder fails closed on split UTF-8, EOF, failed writes, missing markers and child exit; a lint gate forbids 14 lints and rejects 10 probes. | [Recorder safety](RECORDER-SAFETY.md) |
| A07 | DONE_SCOPED | Reports sum cargo's test results per profile, list 23 declared platform-only tests (19 Unix-only, 4 Linux-only) and 3 partial branches, and a strict cross-platform diff checks them. A property-to-test map of 40 documented properties and 91 uniquely named tests is checked on every CI platform; it is not an exhaustive map of every statement. | [CI evidence](CI-EVIDENCE.md), [property map](PROPERTY-TESTS.md) |
| A08 | DONE_SCOPED | Structural, admission and OS-level publication faults (permission denial, SIGKILL, full disk on a bounded tmpfs) keep the previous snapshot; possible publication before a directory-sync error is documented. | [Publication fault tests](PUBLICATION-FAULT-TESTS.md) |
| A09 | DONE_SCOPED | Allocations, user-space copies and peak additional heap per mutation are measured locally (DHAT, heaptrack). A fresh paired timing comparison and the resident-set peak of one mutation in fresh processes, with the peak reset before the mutation, are recorded on one host; a second host remains open. | [Mutation comparison](MUTATION-COMPARISON.md), [builder](COLLECTION-BUILDER.md) |
| A10 | PARTIAL | A comparison protocol (input, profile, clock boundary, warm-up, order, A/A control, host metadata) is stated in every `speed-ab` record, and `speed-ab-check` refuses a record missing a field or the A/A control, or whose A and A2 are not clean trees of one commit; the [grep comparison](BENCHMARK-GREP.md) follows it except for the A/A control. Plans re-run the recorded speed sets of 2026-10-20 and 2026-10-25 from public revisions, and one [re-run of all four](speed-results/2026-11-04-reruns/README.md) on the author's machine is retained; other historical campaigns are not re-run under it, and no re-run on other hardware is retained. | [Campaign protocol](EVIDENCE-CAMPAIGN.md) |
| A11 | DONE_SCOPED | A public F32/F16/affine Q1–Q16 reference with a versioned container, independent oracles and retained output exists. It does not establish integration with every GEL reader/writer or with any private format. | [Precision matrix](PRECISION-MATRIX.md), [codec scope](CODEC-SCOPE.md) |
| A12 | DONE_SCOPED | Bytes, numeric error, ranking and context are separate acceptance results, not derived from transport PASS. | [Claims](CLAIMS.md), [codec scope](CODEC-SCOPE.md) |
| A13 | DONE_SCOPED | Sanitized per-platform CI reports of selected revisions are kept in the repository, because Actions artifacts and logs expire. | [CI evidence](CI-EVIDENCE.md), [recorded reports](evidence-ci/README.md) |
| A14 | DONE_SCOPED | README entry point, fresh-checkout instructions and a first-screen link to the public Evidence Lab walkthrough; main, review branches and the historical tag are distinct. | [README](../README.md) |
| A15 | OWNER | Description and topics are repository settings. A social-preview image is prepared; uploading it is a manual settings step. | [Repository](https://github.com/Gelram-project/gel-ram) |
| A16 | DONE_SCOPED | One film index lists the public walkthrough with its destination and scope label; the private previews were removed from the tree on 2026-10-04. | [Film index](../media/INDEX.md) |
| A17 | PARTIAL | A two-process update/restart scenario and tests exist. One still of the final screen of a recording of it is published; the recording itself is not. | [Update/restart scenario](UPDATE-RESTART-SCENARIO.md) |
| A18 | PARTIAL | An English descriptive transcript of the public Evidence Lab film exists; the private previews were removed from the tree on 2026-10-04. Human confirmation of the timeline remains open; translations are not new engine outputs. | [Transcript](../media/TRANSCRIPTS-EN.md) |
| A19 | PARTIAL | An executable claim registry separates executable checks, separate gates, locally measured rows and deferred rows, and rejects any other state. It covers the v0.5.0 tools, every README timing table, the published hash vectors and record layouts, the gel-store and gel-reader parts of the README, the source-builder limits, the separate gates CI runs, the recorded crash and kill series, the recorded Q8 runs and speed sets, and the limits stated as not established; 105 sentences of the README, other pages and three recorded runs that name a row are checked against its state. It is not yet an exhaustive map of historical statements. | [Claims](CLAIMS.md) |
| A20 | OPEN | Reproduction form, a [verification suite runner](REPRODUCE.md) and a [one-command hardware check](CHECK-ON-YOUR-HARDWARE.md) exist; `xtask witness` prints a filtered `GEL_WITNESS_2` block to paste into the form. Obtain and retain an independent second-host campaign and first-use feedback. Hosted CI alone does not close this item. | [Reproduction form](../.github/ISSUE_TEMPLATE/reproduction.yml) |
| A21 | PARTIAL | Scoped author-reported component measurements are published. An identical-task, end-to-end comparison with the exact baseline remains open. | [Component scope](GEL-EXPERIMENTAL-MEASUREMENTS.md) |
| A22 | OPEN | Implement a public execution-identity gate only within approved disclosure scope. Substituting the full-scan baseline must fail that gate even when results match. | [Baseline boundaries](OCEAN-SCALE.md), [claims](CLAIMS.md) |
| A23 | OPEN | Adaptive precision needs a declared error budget, independent oracle, full metadata costs and stable task decisions, including weak-signal failures. | [Codec scope](CODEC-SCOPE.md) |
| A24 | DONE_SCOPED | PR #10 was built in small commits, each with the source manifest, full verify and a leak scan; later PRs follow the same practice. A green job alone does not authorize a merge, release or disclosure. | [PR #10](https://github.com/Gelram-project/gel-ram/pull/10) |

## Execution order and dependencies

### M0. Coherent information and review scope

Update this roadmap, README, publication-status scope and PR description together
with the source manifest. Acceptance: readers can distinguish merged features,
branch-only changes, historical evidence and open gates; PR metadata matches the
actual diff. This is a maintained snapshot, not an automatic status synchronizer.

### M1. Integrity and failure-path acceptance

Review A01-A04, A06-A08, A12-A13 and A19 at one exact revision. Run configured
native checks and retain both success and expected-rejection evidence. Separate
finite fixture coverage, ignored/platform-excluded tests, repeated test
executions, doctests and debug/release profiles. Do not mark a gate PASS from
prose alone.

### M2. Reproducible user experience

After the relevant M1 checks, close the visual parts of A05 and A14-A18. Review
all three films from start to finish, check original bytes and identifiers,
record reviewed time ranges and reconcile captions with actual logs. Keep
historical films unchanged; publish a new recording only after reviewing the
A17 scenario. A decode-only report or sampled screenshots cannot close M2.

### M3. Measured scaling

After correctness acceptance, address A09-A10 and A20. Run paired baseline/new
measurements on the same task and inputs, then on independent hardware. Retain
all raw samples, slower runs and failures. Report payload, metadata, mapped
bytes, RSS/peak RSS, faults, allocations/copies where measured, worker count,
profile, host load and timing boundaries. Choose sample size and uncertainty
reporting before a tail-latency claim; do not infer deep tails from a small N.

### M4. Distinctive memory research

A21-A23 depend on an exact task contract, the M1 failure gates and approved
public disclosure. Keep the existing full scan as the comparison oracle, never
as evidence that a different execution path was used. Require differential and
execution-identity checks together. Use the public [precision reference](PRECISION-MATRIX.md)
as a scoped starting point, not as proof of private codec or application
integration. A public retrieval-quality claim additionally needs a
redistributable corpus, a documented public encoder, held-out questions and an
abstention policy. These public gates do not publish private engines, banks or
other private internals.

### M5. Separately approved release

A bounded release does not require pretending that all longer-term research is
complete. Freeze its declared scope, carry open items and exclusions into release
notes, verify the exact tree/assets/manifest and configured CI, and obtain separate
maintainer release approval. Any included media must have its stated M2 review.
M3/M4 are prerequisites only for performance or capability claims included in
the release. Do not move the historical v0.3.0 tag or infer approval from a
green job.

## Published baseline and longer-term direction
The tagged `v0.3.0` release remains an earlier immutable release point. Public
`main` has advanced since; later tagged releases are listed on the Releases page.

## Public performance boundary

The historical, author-reported 1M/10M Ocean measurements are retained as a
**CPU/RAM full-scan baseline**: CPU workers scan, score, rank and select records
resident in memory. They are valuable comparison evidence, but they are not
presented as the final GEL RAM execution model.

The private project contains newer experimental work. Author-reported component
measurements are published separately in
[GEL-EXPERIMENTAL-MEASUREMENTS.md](GEL-EXPERIMENTAL-MEASUREMENTS.md); they
measure different tasks and establish no end-to-end advantage or multiplier over
this baseline. Any such claim requires an exact public reproduction package.

## Next public acceptance gates

1. **Independent reproduction on a second host.** Re-run the public Q8 and
   source-integrity evidence on different hardware. Publish slow cases and
   failures, not only best runs.
2. **Larger statistical campaigns.** Where performance claims need tail latency,
   retain raw samples and use a sufficient N before reporting p99.9 or deeper tails.
3. **Memory-accounting evidence.** Report payload, mapped bytes, RSS/peak RSS,
   page faults, worker count and host conditions alongside latency.
4. **Exactness before speed.** Any faster public path must be differential-tested
   against the existing exact reference and must not weaken integrity,
   abstention, ordering or restart/replay checks.
5. **Fault and concurrency coverage.** Extend corruption, truncation, competing
   writer, restart, timing and observer-effect tests.
6. **Public retrieval evaluation only with a suitable corpus.** Semantic or
   knowledge-quality claims require a redistributable corpus, a documented
   public encoder, held-out evaluation and an explicit abstention policy.
7. **Next release package.** Cut a new tagged release only after its exact source
   tree, release assets, checksums, documentation and required CI are all green.

## Reproduction priorities for outside researchers

The most useful independent work is currently:

- public workspace verification on additional CPU families;
- corruption/restart/concurrency edge cases;
- Unicode/source-boundary cases for the document reader;
- complete memory-cost measurements;
- slower or negative performance results that help identify the real limits.

Start with [TRY-IT](TRY-IT.md) and
[VERIFIED RESULTS](VERIFIED-RESULTS.md). A benchmark report does not require a
code contribution or a CLA; code intended for merge follows [CONTRIBUTING](../CONTRIBUTING.md).

## Research boundaries retained

The public project does not currently establish:

- unrestricted semantic question answering;
- universal speedup on all hardware or workloads;
- stable p99.9–p99.999 from campaigns with too few observations;
- physical memory-side compute;
- production power-loss durability;
- fourfold independent information capacity from four coordinate views;
- private-system performance or architecture.

These boundaries are deliberate. Public evidence should remain reproducible
without exposing unpublished private mechanisms.

## Publication policy

Prefer a bounded, independently runnable capability over a larger unverifiable
claim. Keep source inventory, exact revision, hardware context, raw evidence,
licensing and known limitations together. A green technical gate proves only
the scope it actually tests.
