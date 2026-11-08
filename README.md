# GEL RAM

<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->
<picture>
  <source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/header-dark.svg">
  <source media="(max-width: 600px)" srcset="media/presentation/phone/header-light.svg">
  <source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/presentation/header-still-dark.svg">
  <source media="(prefers-reduced-motion: reduce)" srcset="media/presentation/header-still-light.svg">
  <source media="(prefers-color-scheme: dark)" srcset="media/presentation/header-dark.svg">
  <img alt="GEL RAM. Knowledge printed, not trained. One stored record, four exact views, 1,152 bytes. Read back exactly; it can say UNKNOWN; a process kill, not a power cut, loses nothing it confirmed. Logo: a glass cube of record cells around an accent core." src="media/presentation/header-light.svg" width="1200">
</picture>

<p align="center"><a href="docs/evidence-q8-current/README.md"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/views-dark.svg"><img alt="4 of 4 views score bit for bit as their written-out copies in 48 recorded runs" src="media/presentation/badges/views-light.svg" width="145"></picture></a> <a href="docs/evidence-mutation/matrix-r1.txt"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/mutants-dark.svg"><img alt="Mutation matrix: 179 format mutants, each row as expected" src="media/presentation/badges/mutants-light.svg" width="162"></picture></a> <a href="docs/CRASH-SERIES.md"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/lost-dark.svg"><img alt="Crash series: 0 of 2,683 confirmed snapshots lost in 200 process kills, not a power cut" src="media/presentation/badges/lost-light.svg" width="188"></picture></a> <a href="Cargo.lock"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/packages-dark.svg"><img alt="14 packages in Cargo.lock, none from outside this checkout" src="media/presentation/badges/packages-light.svg" width="151"></picture></a> <a href="docs/speed-results/2026-11-04-reruns/README.md"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/faster-dark.svg"><img alt="Speed: 100 of 133 cases faster beyond their A/A band in both runs, one machine" src="media/presentation/badges/faster-light.svg" width="203"></picture></a> <a href="docs/COMMITMENTS.md"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/commitments-dark.svg"><img alt="9 published commitments" src="media/presentation/badges/commitments-light.svg" width="131"></picture></a> <a href="RELEASE-NOTES-v0.9.0.md"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/version-dark.svg"><img alt="Version 0.9.0: release notes" src="media/presentation/badges/version-light.svg" width="119"></picture></a> <a href="LICENSE"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/badges/license-dark.svg"><img alt="License: noncommercial, source-available; LICENSE controls" src="media/presentation/badges/license-light.svg" width="207"></picture></a></p>

**GEL RAM prints knowledge into memory as fixed Q8 records, instead of training
it into model weights, and reads the records back exactly, working toward
hardware-level memory computation.** The goal is a text AI that holds a free
conversation in Polish and English: it answers in sentences of its own, not
pasted excerpts, from what it holds, shows the source it relied on, and says it
does not know when it holds nothing on the question.

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/status-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/status-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/status-dark.svg"><img alt="Checkable here: exact read-back through four views, a changed byte refused, nothing confirmed lost to a kill. Private: printing and answering; 9 published commitments fix the contents of private directories as of their dates; only their labels say what each holds. The text AI is a goal." src="media/presentation/status-light.svg" width="1200"></picture>

The text AI is a goal, not a result ([the goal](docs/GOAL.md)). Hardware-level memory computation and power-cut durability of snapshots and of the record history are `NOT_ESTABLISHED` ([claim registry](docs/CLAIMS.md)).

<details><summary>What is public and what is private, in words</summary>

| Status | What it covers |
|:---|:---|
| **You can check it here** | One record read back exactly through four views; a changed byte refused; a killed process losing nothing it confirmed. One command each, [below](#check-it-yourself). |
| **Private** | The step that prints text into records, and the private build that answers from them; both exist only on the author's machine. [9 published commitments](docs/COMMITMENTS.md) fix the exact contents of private source trees, data sets, a sealed sample and the target Q8 Quad quantizer as of their dates; only their labels say what each holds. |
| **Not established** | The text AI, hardware-level memory computation and power-cut durability of snapshots and of the record history; see the line above. |

</details>

## Check it yourself

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/terminal-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/terminal-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/terminal-dark.svg"><img alt="Drawn, what to look for: --literal: ROUNDTRIP=4/4 DIFFERENT_BITS=0, TAMPER=REJECTED; quad_compare: Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED; mutation-matrix: MUTATION_MATRIX=PASS mutants=179; crash-series 20: CRASH_SERIES=PASS with acknowledged_lost=0; verify: GEL_VERIFY_ALL=PASS." src="media/presentation/terminal-light.svg" width="1200"></picture>

With [Git](https://git-scm.com/) and [rustup](https://rustup.rs) installed, run each line separately; only `git clone` and `rustup` need a network.
No independent run on a second machine has been reported yet: [post yours in issue #20](https://github.com/Gelram-project/gel-ram/issues/20).

```sh
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab -- --literal README.md
cargo run --locked --offline --release -p gel-phase-quad --example quad_compare
cargo run --locked --offline -p xtask -- mutation-matrix
cargo run --locked --offline -p xtask -- crash-series 20
cargo run --locked --offline -p xtask -- verify
```

<details><summary>What each line shows, and its limit</summary>

No model and no private data are involved.

| Command | Line to look for | What it shows | Its limit |
|:---|:---|:---|:---|
| `--literal` | `ROUNDTRIP=4/4 DIFFERENT_BITS=0`, `TAMPER=REJECTED` | The first 1,024 bytes of any file you name, placed in one record; all four views give them back bit for bit, and a copy with one changed byte is refused | Your bytes as they are, not knowledge printing: no meaning and no search |
| `quad_compare` | `Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED` | 512 synthetic records; each view's score equals the score of a separate reference copy, bit for bit | Meaning is not measured; one plain copy can still be read faster than the shared record |
| `mutation-matrix` | `MUTATION_MATRIX=PASS mutants=179` | Each changed copy in the matrix, over the three public file formats, is refused under the original SHA-256 | A finite set of changes, not every possible one |
| `crash-series 20` | `CRASH_SERIES=PASS` with `acknowledged_lost=0` | Evidence Lab is killed 20 times at random while it writes; nothing it confirmed is lost | Recorded on Linux; `verify` runs it on Unix hosts only; a process kill, not a power cut |
| `verify` | `GEL_VERIFY_ALL=PASS` | The workspace tests and public gates, with short forms of the runs above | Crash series on Unix hosts only; the private build is not in this checkout |

</details>

## One record, four exact views

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/q8-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/q8-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/q8-dark.svg"><img alt="What a Q8 record holds: 1,024 values of one byte each plus a 128-byte mask, one bit per value, make 1,152 bytes in memory, held once. Four views read that same record, each exactly as its own full copy would; four copies would take 4,608 bytes. Not four memories, votes or a speedup." src="media/presentation/q8-light.svg" width="1200"></picture>

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/record-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/record-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/record-dark.svg"><img alt="One stored Q8 record: 1,024 one-byte values and a 128-byte activity mask, 1,152 bytes, read through four equivalent views without four copies. Four packed copies would take 4,608 bytes; the reference layout used for checking takes 8,192 bytes. 48 of 48 recorded runs end with Q8_QUAD_EXACT=PASS; semantic accuracy is not measured. Record payload only, not process memory; not four independent memories." src="media/presentation/record-light.svg" width="1200"></picture>

One record, 1,152 bytes, read four ways, each bit for bit. [Contract](docs/Q8-QUAD.md) · [the recorded runs](docs/evidence-q8-current/README.md)

<details><summary>What the four views are, and what they are not</summary>

A record is held once, in 1,152 bytes of memory on the tested ABI. A view reads
the same values in another fixed arrangement; each of the four views scores exactly as a separate, fully
written-out copy would, bit for bit, and four such copies would take 4,608 bytes.
Four views are not four independent memories, four votes or a speedup.

</details>

## Exact bytes, or a refusal

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/flow-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/flow-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/flow-dark.svg"><img alt="What happens to a citation, drawn in 3D: your document with an exact quote, a snapshot pinned by the SHA-256 you keep, a new process that reopens it with the same pin and passes, and a copy with one changed byte that is refused." src="media/presentation/flow-light.svg" width="1200"></picture>

Your quote comes back byte for byte; a changed byte is refused. [Evidence Lab guide](docs/EVIDENCE-LAB.md) · [as text](media/gifs/05-integrity.txt)

<details><summary>What Evidence Lab checks, and what a hash does not show</summary>

Evidence Lab (`gel-evidence`), the public collection tool, works on your own
UTF-8 files: each quotation comes back byte for byte with its byte range, a
replaced document's old citation is refused, and a saved snapshot with one
changed byte fails to load under the SHA-256 you kept. A matching hash shows
these are your bytes, not that the source is true or private: snapshots and
backups hold plain text ([backup contract](docs/BACKUP.md)). It is phrase lookup,
not question answering. Its demo,
`cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo`,
ends with `GEL_EVIDENCE_DEMO=PASS`.

</details>

## A killed process loses nothing it confirmed

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/crash-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/crash-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/crash-dark.svg"><img alt="One recorded crash series on the author's Linux machine (ext4, 2026-09-29): in each of 200 trials a collection growing to 24 snapshots was killed with SIGKILL at a random moment within 344 ms. 2,683 snapshots were acknowledged before the kills; 0 were lost and 0 were partial, and 200 of 200 trials resumed to the uninterrupted result. 153 kills came in the middle of a publication. A process kill, not a power cut: power-loss durability is not established." src="media/presentation/crash-light.svg" width="1200"></picture>

200 random process kills in one recorded run: 0 of 2,683 confirmed snapshots lost. [Crash series](docs/CRASH-SERIES.md)

<details><summary>The recorded series in words</summary>

In the recorded series Evidence Lab was killed at random moments in 200 trials:
0 of 2,683 acknowledged snapshots were lost, none was partial, and every trial
resumed to the uninterrupted result ([crash series](docs/CRASH-SERIES.md)).
Write failures, denied permissions and a full disk leave the previous snapshot
intact ([fault tests](docs/PUBLICATION-FAULT-TESTS.md)).

</details>

## The public checks, drawn from their files

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/bars-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/bars-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/bars-dark.svg"><img alt="Format mutation matrix: rejected and accepted rows per public format in the committed report, and how many rows matched their expected verdict." src="media/presentation/bars-light.svg" width="1200"></picture>

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/wall-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/wall-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/wall-dark.svg"><img alt="Property map across CI platforms: one cell per documented property on Linux, macOS and Windows. Every mapped test must run and pass exactly once on each platform; declared Unix-only tests are exempt on Windows." src="media/presentation/wall-light.svg" width="1200"></picture>

Every changed copy refused under its original SHA-256; CI requires every mapped property on three platforms, Unix-only tests exempt on Windows. [CI evidence](docs/CI-EVIDENCE.md)

<details><summary>Where these images come from</summary>

Both images are drawn from the committed mutation report and the property map when the README is built; the presentation check fails if either file changes without a rebuild. The map shows the rule CI enforces, not the outcome of one run ([CI evidence](docs/CI-EVIDENCE.md)).

</details>

## Speed, timed against v0.6.1

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/speed-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/speed-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/speed-dark.svg"><img alt="Timing set 2026-10-20-against-v0.6.1: v0.6.1 against main at 876247f, the first v0.7 speed package. 133 timed cases in 23 of 24 timing programs, one row per program in name order: the range from the lowest to the highest v0.6.1 median time divided by the new one over the cases of the program, and their median, on a log scale centred at 1 over the widest A/A noise band of those cases. Cases are grouped by timing program; per-case results are in docs/speed-results/2026-10-20-against-v0.6.1. 103 cases are faster beyond their A/A band, 18 slower beyond it (SHA-256 alone at 0.28 and 0.39) and 12 inside it. 22 of 23 timed programs printed the same output digest in all three trees. speed_top1_many has only a manifest: its API does not exist in v0.6.1, so the example does not build there. speed_tree_gates prints DIFFERENT_OUTPUT because it counts the files of the tree it runs in, and the two trees differ. Machine: AMD Ryzen AI 9 HX 370, 24 hardware threads, 94 GiB RAM, Linux 6.17; load average 2.7 at the start, 12.6 to 19.4 at the end, from other work on the machine; not pinned to cores. Ratios of different programs are not comparable with each other; the output digests, not the timings, are the claim." src="media/presentation/speed-light.svg" width="1200"></picture>

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/rerun-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/rerun-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/rerun-dark.svg"><img alt="The same 133 cases, timed twice against v0.6.1, each against its own A/A band: 103 faster, 12 inside, 18 slower on 2026-10-20; 105 faster, 4 inside, 24 slower on 2026-11-04; 100 of the 103 faster again (102 with a ratio above 1). A/A noise 0.04 → 0.004. One machine, not pinned to cores." src="media/presentation/rerun-light.svg" width="1200"></picture>

Each case timed against two copies of the baseline, then once more with no other work running; most are faster, some slower. [Re-runs](docs/speed-results/2026-11-04-reruns/README.md)

<details><summary>What came in 0.7, in words</summary>

- **No third-party packages:** every one of the 14 packages in `Cargo.lock` is a crate of this checkout. SHA-256 and Unicode normalization are written here; SHA-256 alone runs at 0.28 and 0.39 of the speed of v0.6.1, which used the processor's SHA instructions.
- **One Unicode version:** normalization follows Unicode 16.0.0, the version the pinned Rust 1.85.0 standard library uses for lowercasing and word splitting, and passes every line of the Unicode normalization test file.
- **The same output bits, faster:** each speed change was timed against two copies of the baseline, and every run printed the same output digest, apart from the whole-tree gates, which count the files of their own tree ([v0.7.0 in the roadmap](docs/ROADMAP.md#v070)).

</details>

## New in 0.9, and in 0.8

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/glance-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/glance-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/glance-dark.svg"><img alt="Versions 0.8 and 0.9 at a glance. New in 0.9: 20 fault tests of saved files, 4 Unix-only, no power-loss test; 72 memory samples of 8 operations, one machine, process-wide, untimed; 6 kinds of value replaced in witness step lines. Since 0.8: 8 tool programs answer --version and --license." src="media/presentation/glance-light.svg" width="1200"></picture>

<details><summary>What changed, in words</summary>

- **New in 0.9, a filtered witness block and faults of saved files:** `xtask witness NEW_DIR` has run the six short public checks since 0.7 (the two kill series only on Unix; elsewhere they are SKIPPED). It now writes one block with the facts of the host, a filter replaces six kinds of value in the step lines (host names, user names, paths, IP and MAC addresses, serial numbers), and a separate check refuses the block if this host's names or directories remain anywhere in it. The step logs beside the block are not filtered, paths and addresses of other machines are removed only as far as the filter recognises them, and no block from a host other than the author's is retained yet ([check on your hardware](docs/CHECK-ON-YOUR-HARDWARE.md)). 20 tests in three new files (four of them Unix-only) cut the saved files of gel-history, gel-store and gel-source at every byte (a store of version 1 past its header; a collection over 1 MiB at 12 fields), damage them without changing their length, race two child processes on one path, stop or kill a save and restart, and read beside a writer; none is a power-loss test ([faults of saved files](docs/PUBLICATION-FAULT-TESTS.md#faults-of-saved-files-on-disk-and-between-processes)).
- **New in 0.9, memory and CI reports on record:** `xtask memory-run`, on Linux, starts each of eight operations, the control among them, as a child process and records payload, peak mapped and resident memory, page faults and threads; one record of a public revision is kept, nine processes of each of eight operations, 72 samples, on one machine, process-wide, with no time recorded ([memory record](docs/memory-results/2026-11-08/README.md)). Sanitized CI reports of a third revision are kept and checked by `xtask ci-evidence-recorded`; they are hosted CI runs of one revision each, not an independent second host, and their strict comparison found 11 undeclared platform-only tests, declared since ([recorded reports](docs/evidence-ci/README.md), [release notes](RELEASE-NOTES-v0.9.0.md)).
- **New in 0.8, programs and files name their license:** the eight tool programs answer `--version` and `--license` (the `xtask` runner does not); every GEL RAM-owned Rust source file begins with a two-line license header (the generated Unicode tables are third-party data with their own notice); `gel-license check DIR` reports whether a copy kept the license, the Required Notice (the attribution line in NOTICE), those headers and the Unicode notice, not that every notice was kept. The license guides now follow the words of the LICENSE ([v0.8.0 in the roadmap](docs/ROADMAP.md#v080)).

</details>

## Private work, fixed in time

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/pledges-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/pledges-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/pledges-dark.svg"><img alt="9 commitments, 2026-10-20 to 2026-11-05: each the SHA-256 of a manifest of a private directory, shown by its first 8 hex characters with its directory and file counts. Checkable with xtask commitment-check once the files are handed over; a hash shows they did not change, not that they are correct." src="media/presentation/pledges-light.svg" width="1200"></picture>

Private files, hashed now; checkable once the owner hands them over. A hash shows they did not change, not that they are correct. [Commitments](docs/COMMITMENTS.md)

<details><summary>What a commitment shows, and what it does not</summary>

A [commitment](docs/COMMITMENTS.md) publishes now the SHA-256 of a manifest of a private directory; the files can be handed over later and checked with `xtask commitment-check`.
It shows the files existed and did not change, not that they are correct. The author's own measurements are in [measured progress](docs/MEASURED-PROGRESS.md); nobody else has re-run them.

</details>

## See it in action

Six edited 12-second replays of recorded public-tool runs; the pacing is not execution time.
[Static view](media/gifs/STATIC.md) · [Original source and hashes](media/gifs/MANIFEST.txt)

<table>
<tr>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/01-dark.svg"><img alt="Workflow 01 · CITE AND RESTART" src="media/presentation/chips/01-light.svg" width="320"></picture><h4>Exact quotes. A verifiable restart.</h4><a href="media/gifs/01-evidence-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/01-evidence-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/01-evidence-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/01-evidence-dark.gif"><img alt="Exact quotes. A verifiable restart." src="media/gifs/01-evidence-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/01-evidence.txt">Transcript</a> · <a href="media/gifs/01-evidence-dark.gif">Dark full size</a></p></td>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/02-dark.svg"><img alt="Workflow 02 · STALE CITATION REFUSED" src="media/presentation/chips/02-light.svg" width="320"></picture><h4>Changed source. Old citation refused.</h4><a href="media/gifs/02-stale-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/02-stale-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/02-stale-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/02-stale-dark.gif"><img alt="Changed source. Old citation refused." src="media/gifs/02-stale-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/02-stale.txt">Transcript</a> · <a href="media/gifs/02-stale-dark.gif">Dark full size</a></p></td>
</tr>
<tr>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/03-dark.svg"><img alt="Workflow 03 · BACKUP AND RESTORE" src="media/presentation/chips/03-light.svg" width="320"></picture><h4>Backup. Inspect. Restore.</h4><a href="media/gifs/03-backup-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/03-backup-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/03-backup-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/03-backup-dark.gif"><img alt="Backup. Inspect. Restore." src="media/gifs/03-backup-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/03-backup.txt">Transcript</a> · <a href="media/gifs/03-backup-dark.gif">Dark full size</a></p></td>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/04-dark.svg"><img alt="Workflow 04 · STRICT REPRODUCTION" src="media/presentation/chips/04-light.svg" width="320"></picture><h4>One command. Inspect every result.</h4><a href="media/gifs/04-reproduce-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/04-reproduce-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/04-reproduce-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/04-reproduce-dark.gif"><img alt="One command. Inspect every result." src="media/gifs/04-reproduce-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/04-reproduce.txt">Transcript</a> · <a href="media/gifs/04-reproduce-dark.gif">Dark full size</a></p></td>
</tr>
<tr>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/05-dark.svg"><img alt="Workflow 05 · CHANGED BYTE REFUSED" src="media/presentation/chips/05-light.svg" width="320"></picture><h4>Changed bytes. Retained pin. Refusal.</h4><a href="media/gifs/05-integrity-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/05-integrity-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/05-integrity-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/05-integrity-dark.gif"><img alt="Changed bytes. Retained pin. Refusal." src="media/gifs/05-integrity-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/05-integrity.txt">Transcript</a> · <a href="media/gifs/05-integrity-dark.gif">Dark full size</a></p></td>
<td width="50%" valign="top"><picture><source media="(prefers-color-scheme: dark)" srcset="media/presentation/chips/06-dark.svg"><img alt="Workflow 06 · YOUR BYTES, ONE RECORD" src="media/presentation/chips/06-light.svg" width="320"></picture><h4>Your bytes. One record. Four views back.</h4><a href="media/gifs/07-literal-light.gif"><picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/07-literal-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/07-literal-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/07-literal-dark.gif"><img alt="Your bytes. One record. Four views back." src="media/gifs/07-literal-light.gif" width="1000" loading="lazy"></picture></a><p><a href="media/gifs/07-literal.txt">Transcript</a> · <a href="media/gifs/07-literal-dark.gif">Dark full size</a></p></td>
</tr>
</table>

## Using GEL RAM

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/license-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/license-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/license-dark.svg"><img alt="GEL RAM Noncommercial Reciprocal License 1.0 at a glance, quoted from LICENSE. Noncommercial use (section 3): 'a worldwide, non-exclusive, non-transferable, non-sublicensable, royalty-free copyright license to reproduce, run, study, modify, and share the Software for Noncommercial Purposes.' 'You may keep Noncommercial Modifications private without a time limit.' Evaluation, by anyone (sections 4 and 2): 'Any person or organization may evaluate the Software and may create Evaluation Modifications without payment and without a fixed evaluation deadline.' 'in a non-production environment solely to assess, learn about, validate, or decide whether to obtain commercial rights.' Money needs an agreement (section 5): 'No Monetized Use is licensed by these public terms.' 'Before beginning a Monetized Use, You must obtain a Commercial Agreement from the Licensor.' Every copy (section 8): 'each recipient receives this license, all copyright notices, and all plain-text lines beginning with `Required Notice:` supplied by the Licensor.' Commercial Agreements: gelram.licensing@gmail.com. A summary adds no rights; LICENSE controls (section 17)." src="media/presentation/license-light.svg" width="1200"></picture>

Source-available: noncommercial use and evaluation are allowed; Monetized Use needs a written Commercial Agreement first; the [LICENSE](LICENSE) controls.

<details><summary>The license in words</summary>

GEL RAM uses the GEL RAM Noncommercial Reciprocal License 1.0: source-available,
not an OSI-approved open-source license. You may reproduce, run, study, modify
and share it for Noncommercial Purposes, keeping the LICENSE, all copyright
notices and every `Required Notice:` line with each copy; private noncommercial
changes may stay private. Anyone, including a company, may evaluate it in a
non-production environment solely to assess, learn about, validate, or decide
whether to obtain commercial rights, with no fixed time limit. Monetized Use is
use that "directly or indirectly receives, earns, enables, supports, protects, or
is intended to produce money or other material financial benefit for You or
another person", including donations, tips, crowdfunding or sponsorships received
because of a GEL RAM-based activity; it needs a written Commercial Agreement
signed by the Licensor and by you before it begins: `gelram.licensing@gmail.com`.
`gel-license use ANSWERS_FILE` shows which LICENSE sentences apply to your
answers about one use; it is not legal advice. Code contributions for merge need
a [CLA 2.0](CLA.md) accepted by both sides.
This README adds no rights; the [LICENSE](LICENSE) controls
([licensing guide](LICENSING.md) · [license decision guide](docs/LICENSE-DECISION.md) · [commercial terms](COMMERCIAL-LICENSE.md) · [license pins](docs/LICENSE-PINS.md)).

</details>

## The memory core

<picture><source media="(max-width: 600px) and (prefers-color-scheme: dark)" srcset="media/presentation/phone/core-dark.svg"><source media="(max-width: 600px)" srcset="media/presentation/phone/core-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/core-dark.svg"><img alt="The 7 parts of the memory core and the 5 dependencies their Cargo.toml files declare among them: Store, Reader and Rebuild build on Bit record; History builds on Bit record and Rebuild. Q8 record and Memory cost stand alone." src="media/presentation/core-light.svg" width="1200"></picture>

<details><summary>Each part, and the test that checks it</summary>

| Part | What it does | Check it |
|:---|:---|:---|
| Bit record (`gel-orb`) | One fixed 128-byte record of 1,024 bits, 64-byte aligned; its bytes round-trip exactly | `exact_byte_roundtrip` · [format](docs/FORMAT.md) |
| Store (`gel-store`) | `.gel` v2 files carry a CRC64 over the header and over the payload; in the tests every single header bit flip, every payload byte flip and every truncation is rejected, and so is an older or equal save counter | `every_payload_byte_flip_is_rejected` · `generation_rollback_and_equal_generation_are_rejected` |
| Reader (`gel-reader`) | One comparison of two records returns 16 judgments at once, not 16 independent measurements; a search that stops early returns exactly the same top-K matches as a full scan | `progressive_top_k_is_exactly_equal_to_full_top_k` · [Reader16](docs/READER16.md) |
| Rebuild (`gel-structural`) | With its stored parent record, the XOR of the differing bits gives back the exact record; a missing parent or an invalid depth is refused, a wrong parent is not detected | `exact_xor_roundtrip_is_bit_identical` · [contract](docs/STRUCTURAL-CODEC.md) |
| History (`gel-history`) | Each state of one record is stored as a copy or as the XOR difference from the state before, at most two differences from a copy, and is rebuilt bit for bit | `long_history_is_exact_and_residual_depth_never_exceeds_two` · [contract](docs/RECORD-HISTORY.md) |
| Q8 record (`gel-phase-quad`) | One 1,152-byte record read through four equivalent views, each score equal to the reference bit for bit | `storage_is_1152_bytes_and_every_bit_survives` · [contract](docs/Q8-QUAD.md) |
| Memory cost (`gel-physics`) | Pointer-chase latency and read bandwidth of your own RAM from 48 KiB to 256 MiB; nanoseconds and GiB/s only, compared within one machine | `cargo run --locked --offline --release -p gel-physics -- 5` · [method](docs/PERFORMANCE.md) |

`cargo test --locked --offline -p CRATE` runs the tests named here; `cargo run --locked --offline --release -p gel-cli -- selftest` checks the first four parts in one run (`GEL_SELFTEST_V2=PASS`).

</details>

## Documentation

| Start and use | Inspect and reproduce | Project |
|:---|:---|:---|
| [Project map](docs/START-HERE.md) | [Claim registry](docs/CLAIMS.md) | [Roadmap](docs/ROADMAP.md) |
| [Evidence Lab](docs/EVIDENCE-LAB.md) · [batch mode](docs/EVIDENCE-BATCH.md) | [Reproduction guide](docs/REPRODUCE.md) · [check on your hardware](docs/CHECK-ON-YOUR-HARDWARE.md) | [Release notes v0.9.0](RELEASE-NOTES-v0.9.0.md) |
| [Backup and restore](docs/BACKUP.md) | [Per-platform CI evidence](docs/CI-EVIDENCE.md) | [Publication status](CANDIDATE-STATUS.md) |
| [Your bytes in one record](docs/LIVE-LAB.md) | [Mutation matrix](docs/MUTATION-MATRIX.md) · [crash series](docs/CRASH-SERIES.md) | [Measured progress](docs/MEASURED-PROGRESS.md) |
| [License decision guide](docs/LICENSE-DECISION.md) | [External checks](docs/EXTERNAL-CHECKS.md) | [All media and their scope](media/INDEX.md) |

The [full-page edition](README-MULTIMEDIA.html) opens from your checkout in a browser; GitHub shows HTML files as source ([presentation guide](docs/README-PRESENTATION.md)).

## About

<picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/presentation/logo-still-dark.svg"><source media="(prefers-reduced-motion: reduce)" srcset="media/presentation/logo-still-light.svg"><source media="(prefers-color-scheme: dark)" srcset="media/presentation/logo-dark.svg"><img alt="GEL RAM logo: a glass cube of record cells around an accent core." src="media/presentation/logo-light.svg" width="120"></picture>

GEL RAM is an independent hobby and research project by **RR — GEL RAM Project**, developed with AI assistance; this repository is not the complete private research system.
Third-party material keeps its own terms: the Rust Book fixture (MIT), the Unicode data files and the tables derived from them (Unicode License v3) and the quoted Wikipedia passages (CC BY-SA 4.0) ([third-party notices](THIRD-PARTY-NOTICES.md)).
[Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [CLA privacy](CLA-PRIVACY.md) · [Media rights](media/RIGHTS.md)
