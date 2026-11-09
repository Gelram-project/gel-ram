# GEL RAM v0.9.0 — one block from your hardware, the speed sets re-run, and memory on record

Date: 2026-11-09

v0.9.0 collects everything merged after v0.8.0 (pull requests #98 to #112).
The short public checks now write a filtered block with the facts of the host,
the recorded speed sets were re-run under a protocol that every new speed record
states, one memory record of public operations is retained, saved files are
tested under fault, and the CI reports of a third revision are recorded. No
library code, data file format, search result, license term or third-party
dependency changes. The records of `speed-ab` and `witness` move to a second
format; `speed-campaign` runs a plan and writes a new record, and `memory-run`
writes another; `ci-evidence-import` rebuilds recorded CI reports from a job
log; and the report of `xtask reproduce` passes the same filter and refusal as
the witness block. The Rust pin stays 1.85.0; the workspace version moves to
0.9.0.
The [roadmap](https://github.com/Gelram-project/gel-ram/blob/v0.9.0/docs/ROADMAP.md#v090)
lists each change with its commit, the last as "this change"; #104, #107 and
#109 only name the merge commits of earlier rows and have no row of their own.

## What the project is

GEL RAM prints knowledge into memory as fixed Q8 records, instead of training it
into model weights, and reads the records back exactly, working toward
hardware-level memory computation. The goal is a text AI that answers in Polish
and English from what it holds, or says it does not know. **The text AI is a
goal, not a result of this repository**, and hardware-level memory computation
is not established: the claim registry lists it as `NOT_ESTABLISHED`. The step
that prints text into records is private, so the Q8 records in this checkout
hold synthetic values or literal bytes.

## Check on your hardware

- `xtask witness NEW_DIR` has run the six short public checks since v0.7.0 (the
  two kill series only on Unix; elsewhere they are SKIPPED). Its block is now
  `GEL_WITNESS_2`. To the facts of the v0.8.0 block (revision and whether the
  tree was clean, system and architecture, available threads, CPU, RAM,
  `rustc`, and each step's outcome and marker) it adds the tool version, the
  system version, the kernel, the logical cores, the count of replacements, and
  each step's exit status and wall time; the kernel and RAM are read on Linux
  only, the system version on Linux and macOS, the CPU and logical cores on
  Linux and Windows, and elsewhere they read `unknown`. A filter replaces six
  kinds of value in the step lines (this host's names, the user's names,
  absolute paths, IP and MAC addresses, serial numbers), and a separate check
  refuses the block if this host's names, the user's names or the directories
  of the run (home, checkout, output) remain anywhere in it; a host or user
  named like the system itself (`ubuntu`, `kali`) is allowed in the fact lines
  (#102).
- `docs/CHECK-ON-YOUR-HARDWARE.md` and the reproduction form ask for that block.
  No block is retained yet, from the author's host or any other; paths and
  addresses of other machines are removed only as far as the filter recognises
  them, and the step logs are not filtered (#102).

## Speed and memory on record

- Every `speed-ab` record (`GEL_SPEED_AB_2`) states its comparison protocol, and
  `speed-ab-check` refuses one without a field or the A/A control, or whose A
  and A2 are not clean trees of one commit; `speed-campaign` runs a plan and
  ends `COMPLETE`, `INCOMPLETE` or `PARTIAL` (#101).
- Four plans re-ran the speed sets of 2026-10-20 and 2026-10-25 once, from
  public revisions, on the author's 24-thread Linux machine, not pinned to
  cores; all 33 records pass the check. In the plan against v0.6.1, whose
  candidate is 876247f, a revision before v0.7.0 and not this release, 105 of
  133 cases are faster beyond their A/A band, 24 slower beyond it and 4 inside
  it; among the slower are SHA-256, at 0.26 and 0.35 of its v0.6.1 speed (A/A
  1.00 and 0.99; 0.28 and 0.39 in the set of 2026-10-20), which no longer uses
  the processor's SHA instructions, and several file loads. The two package-3
  plans use public revisions where the recorded trees were local or not on
  main, so they are not the same comparison; the five `speed_source_load` cases
  recorded at 1.07 to 1.18 come out at 0.99 to 1.07. The recorded sets stay as
  they were measured; a re-run on other hardware remains open (#103).
- `xtask memory-run`, on Linux, starts each of eight operations, the control
  among them, as a child process at least five times and records payload, peak
  mapped and resident memory, page faults and threads; `memory-check` refuses a
  missing field, a non-zero exit, too few samples or a summary that does not
  recompute (#108). One record is retained: nine processes of each of eight
  operations at 390136f, the merge commit of #108; a test checks it and refuses
  it with one sample changed (#110). One machine on one day, not pinned to
  cores; the numbers are process-wide, not the cost of a format; the peak is
  the `VmHWM` of the last read, and the kernel's final account was not read; no
  time is recorded.

## Saved files and CI reports

- 20 tests in three new files cut the saved files of gel-history, gel-store and
  gel-source at every byte (a version 1 store past its header, a collection over
  1 MiB at 12 chosen lengths), damage them without changing their length, race
  two child processes on one path, stop or kill a save and restart, and read
  beside a writer; properties P36 to P40 join the map (#108). There is no
  power-loss test and no timing property; 16 of the 20 tests compile on Windows
  and all 20 on macOS, and they run there only from the CI of #108 on, with no
  recorded report yet.
- Sanitized CI reports of a third revision, 79f6ac5, are recorded;
  `xtask ci-evidence-recorded`, in `verify`, refuses an altered report, a
  missing or extra file and a table that differs. Their strict comparison found
  11 undeclared platform-only tests and refused one ignored child-process
  helper; both are declared since. These are hosted CI runs of one revision
  each, not an independent second host; the strict comparison covers the
  recorded revisions only, not every CI run, and reads the debug workspace
  lists only (#108).

## Claims, README and roadmap

- The claim registry grows from 49 to 86 rows, among them no third-party
  packages, no network code, the recorded CI reports, the saved-file faults, the
  speed re-run and the memory record, and the query rate as not established;
  `xtask claims` checks 105 sentences and result lines of the README, other
  pages and three record files against the rows they name. It is not an
  exhaustive map: the roadmap rows, the release notes, videos and
  measurement tables have no row (#100, #103, #108, #110).
- Most README sections lead with a picture drawn from the files and one line,
  the full wording folded under it; among the new cards are the re-run, the
  commitments, the memory core and what one Q8 record holds, as
  `docs/Q8-QUAD.md` states it; every wide card of the README also comes 600
  wide for a phone (#105, #106, #108). A viewer
  that ignores the picture's media rule shows the wide card, and a local
  rendering is not a check of the page GitHub shows.
- The roadmap lists each delivered capability with what shows it in this tree,
  historical evidence apart and each open review item re-checked; a test
  requires every relative link and anchor of the roadmap and the status file to
  resolve (#99). Before the tag, these notes and some sentences of the
  roadmap, the status file, the README and other pages were corrected against
  the tree, and two checks of the README cards read the page they follow
  (#112).

## Private work, fixed in time

One more published commitment, Target Q8 Quad quantizer (2026-11-05,
3 directories, 10 files, manifest SHA-256
`0034f6ee14b457c1ba11fbd8756c2830763d26ac262b35ca9474541f98e1861a`), in
`docs/COMMITMENTS.md` (#106). As with every commitment, the manifest and the
files stay private until shown. It shows that the files do not change, not that
the quantizer works.

## Earlier versions

The notes of v0.8.0 leave the tree, as earlier notes did; they remain in the
history, the tag and the v0.8.0 release page.

## Verify this release yourself

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout v0.9.0
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline --release -p gel-live-lab -- --witness
cargo run --locked --offline -p gel-live-lab --bin gel-license -- check .
cargo run --locked --offline -p xtask -- memory-check docs/memory-results/2026-11-08/MEMORY.txt
```

Expected markers: `GEL_VERIFY_ALL=PASS`, `WITNESS=MATCH`,
`LISTED_ITEMS_PRESENT` and `GEL_MEMORY_CHECK=PASS records=8 samples=72`.
`SOURCE-SHA256SUMS.txt` lists every other tracked file of the tagged tree. The
release asset GEL-RAM-v0.9.0-SOURCE.zip is `git archive` of the tagged commit;
its SHA-256 is published next to it. Binary packages are built in CI and carry a
build-provenance attestation, as described in
[binaries](https://github.com/Gelram-project/gel-ram/blob/v0.9.0/docs/BINARIES.md).

## Not claimed

No semantic chatbot, general AI, unrestricted question answering, measured
conversation, private engine or bank, hardware-level memory computation, speed
advantage over grep or any language model, or commercial superiority is claimed.
The timings and the memory record each come from one machine and say nothing
about another host; each speed ratio is read against its A/A band, and no time
is recorded beside memory. No result from an independent second host is
retained yet; the recorded CI reports come from hosted runners.
`gel-license` does not decide whether a use or a copy complies with the LICENSE,
and this repository gives no legal advice. The private printing step and the
private data sets are not part of this release; the commitments only fix their
contents. Four views of one record are not four independent memories, four
votes or a 4× speedup. A process kill or stop is not a power cut; power-loss
safety is not established.

RR — GEL RAM Project
