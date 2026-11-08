# Memory record of 2026-11-08

[MEMORY.txt](MEMORY.txt) is one run of `xtask memory-run --reps 9` at revision
390136f, the merge commit of PR #108, from a clean checkout: eight operations
with nine processes each, 72 samples. The instrument, the fields and the general
limits are in [Memory accounting](../../MEMORY-ACCOUNTING.md).

Re-check the record and the table below, on any system:

```sh
cargo run --locked --offline -p xtask -- memory-check docs/memory-results/2026-11-08/MEMORY.txt
cargo test --locked --offline -p xtask the_retained_record
```

The first prints `GEL_MEMORY_CHECK=PASS records=8 samples=72`. The test puts the
record through the same check, requires every table row below to be the one the
record gives, and requires no major fault in any process.

Re-run on Linux from a clean checkout of 390136f:

```sh
cargo run --locked --offline -p xtask -- memory-run \
  --out NEW_DIR_OUTSIDE_CHECKOUT --work DIR_OUTSIDE_CHECKOUT \
  --target-root BUILD_DIR --reps 9
```

## Scope

- One machine, the author's, not pinned to cores, frequency not fixed. The record
  describes it without a host name: AMD Ryzen AI 9 HX 370, 24 logical CPUs,
  about 124 GiB, Ubuntu 24.04.5, kernel 7.0.0-42-generic, rustc 1.85.0, release
  profile, no RUSTFLAGS. Nothing here says what another host would show.
- Numbers are process-wide: binary, allocator, the setup of an example and its
  input generator are inside them. Peak minus payload is not the cost of a format.
- The one-minute load average stayed between 2.18 and 2.56 at every reading, and
  available memory was about 100 GiB of about 124 GiB. The record holds both
  values from just before and just after each process. No time was recorded and
  none is claimed.
- Not measured: time, heap allocations, allocator-internal use, swap, huge-page
  use, NUMA placement, the page cache of the system, cold start, and any other host.

## Results

Payload in bytes, the same in all nine processes of an operation; memory in KiB;
faults and threads as counts. Peak resident (`peak_rss_kib`, from `VmHWM`) is
given as the median and as lowest to highest of the nine processes; peak mapped
(`mapped_peak_kib`, from `VmPeak`), minor faults (`minor_faults`) and threads
(`threads_peak`) are given lowest to highest. No operation had a major fault.

| Operation | Payload | Peak resident, median | Peak resident, range | Peak mapped, range | Minor faults, range | Threads, range |
|---|---|---|---|---|---|---|
| `control_touch_64mib` | 67108864 | 71164 | 71028 to 71320 | 76244 to 76244 | 16570 to 16573 | 1 to 1 |
| `q8_read_8192` | 9437184 | 12152 | 12104 to 12168 | 13172 to 13172 | 2551 to 2554 | 1 to 1 |
| `q8_queries_32768_t4` | 37748736 | 43872 | 43848 to 43888 | 288256 to 443904 | 10434 to 10439 | 4 to 4 |
| `evidence_fixtures` | 520 | 2440 | 2428 to 2540 | 3640 to 3640 | 103 to 105 | 1 to 1 |
| `evidence_save_64mib` | 67108864 | 149852 | 149844 to 149908 | 216652 to 216652 | 39004 to 39008 | 1 to 1 |
| `evidence_load_64mib` | 67109168 | 68428 | 68364 to 68488 | 609980 to 741004 | 16539 to 16541 | 9 to 9 |
| `snapshot_load_64mib` | 67110679 | 69232 | 69208 to 69252 | 272744 to 338256 | 17002 to 17004 | 4 to 4 |
| `bundle_load_64mib` | 67357139 | 69264 | 69248 to 69268 | 543020 to 608532 | 33189 to 33191 | 8 to 8 |

How to read them:

- The control shows that the instrument reads a high-water mark: 64 MiB is 65536
  KiB, its peak (71028 to 71320 KiB) holds that block and the rest of the
  process, and its last resident set is at most 6072 KiB.
- Between processes, peak resident varied (highest minus lowest, over the median)
  by at most 4.6% in `evidence_fixtures`, a process of about 2.4 MiB, and by at
  most 0.6% in every other operation; minor faults varied by at most 5 counts.
  Nine processes show repeatability on this machine, not a distribution. The last
  resident set in the record says little about use: it is whatever the last read
  happened to meet.
- In the four operations with more than one thread, peak mapped varies far more
  between processes than peak resident (`q8_queries_32768_t4`: 288256 to 443904
  KiB); in the four single-thread operations it does not vary at all. That fits
  reserved address space such as thread stacks and allocator arenas, which
  depends on how threads overlap. It is not memory in use.
- The three operations of about 64 MiB that load a file start threads of their
  own although `workers_requested` is 1: that field states only what was asked.
- Each peak is the `VmHWM` of the last read of `/proc/<pid>/status` before the
  process ended, not the kernel's final account that `wait4` returns; that
  account was not read for this record. In `evidence_fixtures` and
  `evidence_load_64mib` the last read still met the process at its peak in all
  nine processes, so for these two a rise after that read cannot be excluded.

## What this does not establish

Nothing here compares GEL RAM with any other system, and nothing here says that
an operation uses little memory. It records what these processes held, on this
machine, on this day.
