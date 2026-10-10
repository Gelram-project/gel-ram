# Crash series: a growing collection killed at random moments

`xtask crash-series` checks one sentence of the project goal on the public
tools: *a restart should lose nothing that was confirmed as saved*.

## What one trial does

1. `gel-evidence` (the public Evidence Lab binary) adds 24 documents of about
   64 KiB one at a time and saves a new snapshot after each, so the collection
   grows while it is being written. Every save prints an acknowledgement with the
   snapshot's SHA-256 pin.
2. The process is killed at a random moment (SIGKILL on Unix). The kill time is
   drawn from a fixed seed, uniformly up to the length of an uninterrupted run
   plus 20%, so some kills land after the last save.
3. Afterwards:
   - every **acknowledged** snapshot must exist, match its pin and hold exactly
     the documents added before it — otherwise it counts as *acknowledged lost*;
   - a snapshot that was written but **not acknowledged** may be absent or
     complete, never *partial*;
   - a **fresh process** loads the last acknowledged snapshot, adds the remaining
     documents and must reach the same final collection, byte for byte, as a run
     without a kill.

Temporary files left by a kill are counted and never loaded as snapshots.
A trial that leaves one, or a complete unacknowledged snapshot, was killed in the
middle of a publication.

## Run it

```text
cargo run --locked --offline -p xtask -- crash-series            # 200 trials, seed 20260929
cargo run --locked --offline -p xtask -- crash-series 50 12345   # trials, seed
```

The last line reads `CRASH_SERIES=PASS` only when no acknowledged snapshot was
lost, none was partial and every trial resumed to the uninterrupted result.
`xtask verify` runs 5 trials on Unix hosts.

## Recorded run

[crash-series-linux.txt](evidence-crash/crash-series-linux.txt) is one run of
200 trials on the author's Linux machine (ext4, Rust 1.85.0), with every
trial's kill time, counts and the time the fresh process needed to reopen the
last snapshot:

| Trials | Acknowledged snapshots | Acknowledged lost | Partial | Resumed to the uninterrupted result | Killed mid-publication | Reopen of the last snapshot, median / p95 / max |
|---:|---:|---:|---:|---:|---:|:---|
| 200 | 2,683 | **0** | **0** | 200 | 153 | 2.3 / 4.5 / 4.5 ms |

In 151 trials a temporary file was left, and in 74 a complete snapshot existed
that had not been acknowledged; none was partial. 28 kills came after the last
save and 6 before the first. Run it on your own machine: a different result is
useful evidence.

## What it does not show

- Killing a process is **not a power cut**: data still in the operating
  system's or the disk's cache survives a process kill but may not survive a
  power loss. Power-loss durability is not established.
- One file system per run, one machine per recorded report.
- It tests the public collection tool, not the private implementation. A
  separate author-run series on the private knowledge store is reported with its
  own scope in [measured progress](MEASURED-PROGRESS.md#kill-during-learning--the-private-knowledge-store).

## The record history: one file replaced while it grows

`xtask history-crash-series` asks the same of the public
[record history](RECORD-HISTORY.md), where every save replaces one file instead
of writing a new one.

1. A writer process (`xtask history-writer`, run from the same binary) appends
   240 states to one record, changing 1 to 300 bits per step so that residual
   and literal entries alternate, and after each state saves the whole GELHIS01
   file with `RecordHistory::write_atomic`. It prints an acknowledgement when
   the save returns.
2. The writer is killed at a random moment, drawn as above from a fixed seed up
   to the length of an uninterrupted run plus 20%.
3. Afterwards the file must open with `open_verified` and equal, byte for byte,
   the history at the last acknowledgement or the one after it (a save that
   finished just before the kill but was not acknowledged yet). A missing file,
   or one holding fewer states than were acknowledged, counts as *acknowledged
   lost*; any other content counts as *partial*. A fresh process then continues
   from the file as it is and must reach the bytes of a run without a kill.
   Temporary files left by a kill stay in the directory and are counted; only
   the history path is ever opened.
4. **Control.** Further trials kill a writer that rewrites the same file in
   place: truncated, then written in four pieces with a flush after each. The
   series passes only if at least one of them leaves a partial file, so a check
   that cannot see a partial file cannot pass.

```text
cargo run --locked --offline -p xtask -- history-crash-series              # 200 trials, seed 20261010, 20 control trials
cargo run --locked --offline -p xtask -- history-crash-series 50 12345 10  # trials, seed, control trials
```

The last line reads `HISTORY_CRASH_SERIES=PASS` only when no acknowledged state
was lost, no file was partial, every trial resumed to the uninterrupted result
and the control left a partial file. `xtask verify` runs 5 trials on Unix hosts,
without the control.

### What it does not show

- The limits above hold here too: a process kill, not a power cut, and one file
  system and one machine per report. Power-loss durability of the history file
  stays `NOT_ESTABLISHED` in the [claim registry](CLAIMS.md).
- One writer per file: two processes saving the same path are not tested.
- The writer runs in the build of xtask that `cargo run` makes; the reopen times
  in a report are for that build.
