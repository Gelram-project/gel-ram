# Memory accounting of public operations

`xtask memory-run` starts named public operations as child processes and records
what each process held. It is an instrument: it reports no time, compares
nothing with anything else and does not say that a cost is low. One run of a
public revision is retained in
[memory-results/2026-11-08](memory-results/2026-11-08/README.md).

```text
cargo run --locked --offline -p xtask -- memory-run \
  --out NEW_DIR_OUTSIDE_CHECKOUT --work DIR_OUTSIDE_CHECKOUT \
  --target-root BUILD_DIR [--reps N] [--only OPERATION]...

cargo run --locked --offline -p xtask -- memory-check FILE
```

`--reps` is 5 to 99, default 5: a single sample is refused, so the spread of
every number shows how repeatable it is. Operations run in rounds, one process
of each operation per round, so a change of the host falls on all of them. Each
program is built once with `cargo build --locked --offline --release`. The run
needs Linux, because its numbers come from `/proc`, and stops with a refusal on
any other system. `memory-check` works anywhere. The output of the operations is
read for the payload and then deleted, so a record holds no timing line of an
example and no document text.

## Operations

| Operation | Program | Payload stated by | Workers requested |
|---|---|---|---|
| `control_touch_64mib` | `xtask memory-touch`, the control | its own `SPEED_SETUP` line | 1 |
| `q8_read_8192` | example `speed_q8_read` | its `SPEED_SETUP` line, `record_bytes` | 1 |
| `q8_queries_32768_t4` | example `speed_q8_queries` | its `SPEED_SETUP` line, `record_bytes` | 4 |
| `evidence_fixtures` | `gel-evidence --batch` on the authored fixtures, searches repeated | the sizes of the files | 1 |
| `evidence_save_64mib` | `gel-evidence --batch`, eight generated documents, then `save` | the sizes of the files | 1 |
| `evidence_load_64mib` | `gel-evidence --batch`, `load` of that snapshot, one search | the size of the file | 1 |
| `snapshot_load_64mib` | example `speed_collection_memory` | its `SPEED_SETUP` line, `snapshot_bytes` | 1 |
| `bundle_load_64mib` | example `speed_bundle_memory` | its `SPEED_SETUP` line, `bundle_bytes` | 1 |

The control writes every byte of a block of known size, gives the block back and
waits. Its peak must be at least the block, and a test checks that its last
resident set is far below it in at least one process of a run, so a run shows
that the instrument reads a high-water mark. Every input is generated from fixed
seeds or is a fixture in the tree.

## The numbers

| Field | Where it comes from |
|---|---|
| `payload_bytes` | what the operation states, or the sizes of the files it reads |
| `mapped_peak_kib` | highest `VmPeak` read from `/proc/<pid>/status` |
| `rss_last_kib` | `VmRSS` of the last read |
| `peak_rss_kib` | highest `VmHWM` read |
| `minor_faults`, `major_faults` | growth of the waited-for children's counters in `/proc/self/stat` over one wait |
| `threads_peak` | highest `Threads` read |
| `polls` | reads that found the process with its memory |
| `load1_centi_start`, `load1_centi_end` | one-minute load in hundredths, just before and just after the process |
| `mem_available_kib_start`, `mem_available_kib_end` | `MemAvailable`, just before and just after the process |

The record also names the host the way `xtask witness` does, after the same
privacy filter: no host name, user name or path. An argument that is a directory
is written as the label `WORK_DIR`.

## The record

The file MEMORY.txt in the output directory is plain text. The first line is `GEL_MEMORY_1`, then the header
lines, then one `sample` line per process, one `record` line per operation with
each number as `min,median,max`, and one `end` line with the counts. The median
of an even number of samples is the lower middle value.

`memory-check` refuses a record, naming the line and the field but never the
value, when

- the version line is not the first line, a header is missing or repeated, or a
  line or field is not of this format, which also keeps time fields out of the
  sample, record and end lines;
- a field is missing, empty, repeated or not a whole number;
- a record line names no operation of the table above, or its kind, package,
  target, arguments, payload source, `resident` or workers differ from what this
  program says of that operation;
- an exit status is not 0, or a process was never read or has a peak of 0;
- an operation has fewer than 5 samples, its repetitions are out of sequence or
  its payload differs between samples;
- the peak of an operation marked resident is below its payload;
- a summary does not recompute from its samples, or the end line does not count
  the lines, or it is absent.

`memory-run` puts its record through the same check before it writes the file, so
a refused run leaves neither the record nor the output directory.

The check concerns the record. It cannot show that a machine was idle or that a
number was not typed by hand.

## Limits

- **The peak is a read, not the kernel's final account.** The workspace forbids
  unsafe code and the standard library has no safe call for `wait4`, so
  the peak is the last `VmHWM` read while the process lived. Pages touched after
  that read are not seen. The loop pauses 100 microseconds between reads, and
  longer on a loaded host.
- **Numbers are process-wide.** They hold the binary, the allocator, the setup
  and the input generator of an example, not the data structure alone. Peak
  minus payload is therefore not the overhead of a format.
- **Mapped means address space.** No library code of these operations maps its
  payload from a file. `VmPeak` counts reserved address space such as thread
  stacks and allocator arenas, most of it never touched, and it moves with how
  threads happen to overlap.
- **The page cache is not counted.** File bytes in the cache of the system are
  not resident in the process.
- **`workers_requested` is 1 where an operation has no worker argument.** Such an
  operation may still start threads of its own, and `threads_peak` shows those it
  caught; a thread that lives between two reads is not seen.
- Not measured: time, heap allocations, allocator-internal use, swap, huge-page
  use, NUMA placement, and any other host.
