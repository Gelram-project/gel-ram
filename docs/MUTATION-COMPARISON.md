# Paired mutation comparison

```sh
cargo run --locked --offline --release -p gel-source --example mutation_compare -- ../new-mutation-comparison
```

Use the pinned Rust 1.85.0 toolchain after fetching dependencies. The output
directory must not exist. The example compiles the unchanged historical
collection and bundle snapshots alongside the current collection. Only add,
replace and remove are timed, not historical persistence code. Historical bundle
tests compiled with this example are repeated reference checks, not new coverage.

The deterministic synthetic corpus has 8, 64 or 256 documents of 20,400 bytes
of text each (one 51-byte UTF-8 line repeated 400 times). The added or
replacement text is 20,426 bytes. Each size/operation has one unrecorded
warm-up and 30 recorded pairs.
Variant order alternates. State cloning, input construction, result serialization
and equality checks occur outside the timer. Every pair must have identical
serialized result bytes and roots, and the root must equal SHA256(serialization).
Timing includes the entire mutation, including document hashing and allocations
inside it. It does not isolate the root hash kernel.

Generated raw.csv preserves all observations and execution order; summary.csv
uses nearest-rank percentiles. With N=30, p99 is the observed maximum, not a
robust tail guarantee. inputs.txt identifies corpus and implementation hashes.
COMPLETE means the paired correctness checks passed, not a performance win.

This first (timing) harness does **not** measure allocation counts,
allocator-internal copies or peak RSS. Both states coexist and the
allocator/cache are warm. Allocation and library-copy counts inside one
mutation were measured separately with an external profiler; see the DHAT
section below. That measurement also does not observe allocator-internal
copies or peak RSS.
The source-level removal of a temporary serialization buffer does not itself
prove a measured RSS reduction. Memory profiling and repeated isolated runs
remain necessary before a broad optimization claim. Do not compare these
mutation timings with Ocean searches, Q8 decoding or private-engine timings.

## Separate Linux process-memory experiment

```sh
cargo build --locked --offline --release -p gel-source --example mutation_compare
target/release/examples/mutation_compare --memory stream 256 replace
target/release/examples/mutation_compare --memory historical_vec 256 replace
```

Run each command in a fresh process. Supported sizes are 8, 64, 256 and operations
are add, replace, remove. This mode builds only the selected bank, samples Linux
VmRSS and VmHWM before and after one mutation, then serializes and checks its
result with the current decoder and SHA256 oracle. Compare result roots and
lengths across variants before comparing memory. Preserve every output and
repeat with alternating execution order for a performance campaign.

The reported bytes convert Linux kB by 1024. hwm_before is the peak of
building the bank. Just before the mutation the harness writes 5 to
/proc/self/clear_refs, which resets VmHWM to the current RSS (Linux 4.0 or
later); rss_reset and hwm_reset are sampled right after, and hwm_after is the
peak from the reset through the after sample. RSS counts resident pages,
including process/runtime state and allocator-retained pages; it is not heap
bytes. Sampling and reading proc also have overhead. These are not allocation
counts or copied-byte counts. A zero increase does not mean zero allocation.
The single operation latency is diagnostic, not a percentile.
No unsafe allocator hook is added; workspace safety policy is unchanged.

## Optional external allocation profiling

Build a separate release binary with debug symbols and without symbol stripping.
The memory mode contains named, non-inlined profile_current_change and
profile_historical_change boundaries around the mutation only. External stack
profiling can therefore exclude bank construction and the later oracle without
linking a profiler into GEL or changing the workspace's unsafe-code policy.

```sh
CARGO_TARGET_DIR=../profile-target CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=none cargo build --locked --offline --release -p gel-source --example mutation_compare
heaptrack --record-only -o ../new-stream-profile ../profile-target/release/examples/mutation_compare --memory stream 256 replace
heaptrack_print ../new-stream-profile.zst --filter-bt-function profile_current_change -F ../new-stream-stacks.txt
```

Use fresh output names. Repeat for historical_vec with the historical boundary;
preserve the trace, executable/source hashes, tool version and result roots.
Compression suffix depends on the installed profiler. This is an optional Linux
diagnostic, not a new build dependency or a required service.

For heaptrack 1.5, scope counts to the exported filtered allocation stacks.
Its final report totals and allocation histogram can still describe the whole
process despite the backtrace filter; do not label them mutation-only numbers.
Check a known allocating control before interpreting an empty filtered stack.
Allocation calls, requested bytes, memory-copy volume, heap peak and RSS are
different metrics. This heaptrack procedure does not measure memory-copy
volume; the DHAT copy mode below gives a lower bound for it.

Profiling changes execution time and process memory. Never mix its latency or
RSS readings with uninstrumented benchmark results. Raw symbolized profiles can
contain local paths: review/redact before any publication. No trace or profiler
binary is added to this repository by these instructions.

## Allocation and copy volume inside one mutation (DHAT, 26 September 2026)

One external measurement used Valgrind 3.22.0 DHAT on the profile binary
built as above (debug symbols, no stripping). Every variant, size and
operation ran three times in heap mode and three times in copy mode, each run
in its own process with the `--memory` mode: 108 processes. Only program
points whose stack contains the profile_current_change or
profile_historical_change frame are counted, so bank construction, the oracle
and printing are excluded. The three repetitions gave identical counts for
every combination; the table shows that single value.

The measured sources were: harness SHA-256
fd07ca267a7dee3beb7679c8d36fa5da05546a7839f74456560fed5ffb29263b (printed by
every run as `HARNESS_SHA256=`; the harness before the HWM reset was added,
with the same mutation code), current collection SHA-256
3bbfa5cd4d3424932a2f95d0403ae17cc5831375209e9b7ab4f5df7fdb296c42, pinned
historical collection SHA-256
cf79c345e62f84ba47f149fd4d26170e46356d98416bdc94a7a2b92cd63a8280.
Host: x86-64, Ubuntu 24.04, glibc 2.39, Rust 1.85.0.

The tool was unpacked, not installed:

```sh
apt-get download valgrind          # valgrind_1%3a3.22.0-0ubuntu3_amd64.deb
dpkg-deb -x valgrind_1%3a3.22.0-0ubuntu3_amd64.deb ../vg
env -u DEBUGINFOD_URLS VALGRIND_LIB=../vg/usr/libexec/valgrind \
  ../vg/usr/bin/valgrind.bin --tool=dhat --mode=heap \
  --num-callers=200 --read-inline-info=yes --dhat-out-file=../dhat-heap-stream-256-replace.json \
  ../profile-target/release/examples/mutation_compare --memory stream 256 replace
```

Package SHA-256
744e081e5cf3d5c598b499dbb7d2250ea3f2869dde4a4d7b231fe6114f347d7d, equal to
the value in the Ubuntu 24.04 (noble/main) package index. It needs libc6
(>= 2.38) and libc6-dbg; both were already present, so nothing else was
fetched. The unpacked valgrind.bin is called directly with `VALGRIND_LIB`
because the packaged wrapper script assumes an installed tree. Removing
`DEBUGINFOD_URLS` keeps Valgrind from contacting a debug-info server. Run the
same command with `--mode=copy`, and use a fresh output name for every run.
`--num-callers=200` (default 12) keeps the boundary frame on every stack; the
deepest recorded stack had 43 frames. Totals are summed from the DHAT JSON
output: every `ftbl` entry naming a boundary function gives a frame index,
and `tb`, `tbk` are summed over the `pps` entries whose `fs` list contains
that index.

| Column | DHAT field | Meaning |
|---|---|---|
| Allocated bytes | heap mode `tb` | Sum of requested sizes of `malloc` and `realloc` calls with the boundary on the stack. A `realloc` counts as a new block of its new size. A sum of requests, not a peak. |
| Blocks | heap mode `tbk` | Number of those allocation calls. |
| Copied bytes | copy mode `tb` | Sum of the length arguments of intercepted memcpy-family calls (memcpy, memmove, mempcpy, strcpy, bcopy and similar) with the boundary on the stack. The destination may be the heap or the stack. A lower bound for user-space copies. |
| Copy calls | copy mode `tbk` | Number of those copy calls. |

| Variant | Documents | Operation | Allocated bytes | Blocks | Copied bytes | Copy calls |
|---|---:|---|---:|---:|---:|---:|
| stream | 8 | add | 20,431 | 2 | 21,132 | 37 |
| stream | 8 | replace | 20,426 | 1 | 20,934 | 29 |
| stream | 8 | remove | 0 | 0 | 1,226 | 30 |
| stream | 64 | add | 20,431 | 2 | 25,390 | 248 |
| stream | 64 | replace | 20,426 | 1 | 25,184 | 239 |
| stream | 64 | remove | 0 | 0 | 5,300 | 242 |
| stream | 256 | add | 21,503 | 3 | 40,726 | 973 |
| stream | 256 | replace | 20,426 | 1 | 40,460 | 1,002 |
| stream | 256 | remove | 0 | 0 | 20,024 | 962 |
| historical_vec | 8 | add | 571,405 | 4 | 204,203 | 22 |
| historical_vec | 8 | replace | 510,200 | 3 | 183,784 | 19 |
| historical_vec | 8 | remove | 428,496 | 2 | 143,544 | 17 |
| historical_vec | 64 | add | 3,998,605 | 4 | 1,347,223 | 134 |
| historical_vec | 64 | replace | 3,937,400 | 3 | 1,326,804 | 131 |
| historical_vec | 64 | remove | 3,855,696 | 2 | 1,286,372 | 129 |
| historical_vec | 256 | add | 15,750,077 | 5 | 5,266,639 | 520 |
| historical_vec | 256 | replace | 15,687,800 | 3 | 5,245,836 | 515 |
| historical_vec | 256 | remove | 15,606,096 | 2 | 5,205,468 | 513 |

How to read it:

- stream allocates the new title and text (5 + 20,426 bytes for add, 20,426
  for replace) and, at 256 add, one 1,072-byte B-tree node. Its remove
  allocates nothing inside the boundary. Its copied bytes are the new title
  and text, small copies into the SHA-256 block buffer on the stack (about
  20,000 bytes in about 960–1,000 calls at 256 documents) and B-tree node
  shifts.
- historical_vec builds the complete serialization to compute the root. Its
  buffer is reserved for the text plus 32 bytes and then reallocated once to
  twice that capacity; DHAT counts both blocks (256 add: 5,242,858 +
  10,485,716 bytes). Serialization copies dominate its copied bytes (256 add:
  5,245,793 of 5,266,639).

Checks: for every size and operation, both variants and both modes produced
the same result length and root, and every complete run passed the harness
oracle (`MEMORY_SAMPLE=PASS`). The allocation sizes seen by a gdb probe of the
native binary (no Valgrind) sum to the allocated bytes above in 18 of 18
combinations, and the call counts equal the earlier heaptrack filtered stacks
in 18 of 18.

Limits of this measurement:

- DHAT heap mode also reports reads and writes (`rb`, `wb`), but only for
  heap blocks allocated inside the boundary. Reads of documents allocated
  before the mutation (for example hashing or serializing the whole
  collection) and stack accesses are not in them, so they are not bytes
  touched by the mutation and are not tabulated here.
- `realloc`: DHAT always models a copy of the old size and adds it to `rb`
  and `wb`. glibc may instead grow the block in place. In one native probe per
  combination (glibc 2.39, under gdb) the historical buffer moved only for add
  (a new mapping, the old one released); for replace and remove it grew in
  place and nothing was copied. The copy inside `realloc` is not visible to
  copy mode either (under Valgrind the tool's allocator performs it; natively
  glibc copies internally). When it happens it adds up to the old size, for
  example 5,242,858 bytes at 256 add.
- Copy mode does not see copies the compiler emits inline (fixed-size
  to_le_bytes, short fixed-length copy_from_slice, loops and vector moves) or
  copies in the kernel. Copied bytes are therefore a lower bound for
  user-space copies, not the total volume of moved bytes.
- Times under Valgrind are not performance. The latency_ns, VmRSS and VmHWM
  values the harness prints in these runs are invalid and were not used; do
  not compare anything here with the timing tables.
- Valgrind replaces the allocator. The request sequence and sizes are the
  same as natively (checked with the probe), but placement, alignment, pages
  and RSS differ. DHAT measures no RSS, page, cache or DRAM traffic.
- The profile binary keeps debug information and symbols; its optimization
  settings equal the release profile, but its machine code was not compared
  with the stripped release binary.
- One synthetic corpus, one machine and one mutation per process. B-tree node
  splits depend on the document count.

The raw DHAT profiles contain local paths and are not part of this repository.

For constructing an entirely new bank, see the separate
[collection builder experiment](COLLECTION-BUILDER.md). It avoids intermediate
whole-bank root computations without changing the final serialized format.
Its construction times must not be compared as though they were single live
add/replace/remove timings from this page.

### Peak additional heap inside one mutation

The peak additional heap is the largest value, between entry to and return from the
boundary function, of the requested bytes of all live malloc-family blocks minus the value
at entry; frees of blocks that existed before the mutation lower it. It is not RSS, allocator
footprint or the allocated-bytes total above. It was computed from the complete native event
sequence recorded by heaptrack 1.5 (three runs per combination, identical results, same machine
code as the DHAT binary); the call sequence matches a gdb probe of the uninstrumented binary and
the heap levels match DHAT's process totals to the byte. glibc moved the historical buffer
during `realloc` only for add, so old and new buffers coexisted during the copy; the atomic
column omits that overlap, as DHAT does. DHAT's per-point `mb` cannot bound this peak: in
Valgrind 3.22 it changes only when the whole process reaches a new heap maximum.

| Variant | Documents | Operation | Peak additional heap, native (bytes) | Atomic-realloc model (bytes) | Net change at return (bytes) | `realloc` of the historical buffer |
|---|---:|---|---:|---:|---:|---|
| stream | 8 | add | 20,431 | 20,431 | +20,431 | none |
| stream | 8 | replace | 20,426 | 20,426 | +26 | none |
| stream | 8 | remove | 0 | 0 | −20,410 | none |
| stream | 64 | add | 20,431 | 20,431 | +20,431 | none |
| stream | 64 | replace | 20,426 | 20,426 | +26 | none |
| stream | 64 | remove | 0 | 0 | −20,410 | none |
| stream | 256 | add | 21,503 | 21,503 | +21,503 | none |
| stream | 256 | replace | 20,426 | 20,426 | +26 | none |
| stream | 256 | remove | 0 | 0 | −20,410 | none |
| historical_vec | 8 | add | 571,405 | 387,747 | +20,431 | moved |
| historical_vec | 8 | replace | 326,542 | 326,542 | +26 | in place |
| historical_vec | 8 | remove | 265,254 | 265,254 | −20,410 | in place |
| historical_vec | 64 | add | 3,998,605 | 2,672,547 | +20,431 | moved |
| historical_vec | 64 | replace | 2,611,342 | 2,611,342 | +26 | in place |
| historical_vec | 64 | remove | 2,550,054 | 2,550,054 | −20,410 | in place |
| historical_vec | 256 | add | 15,750,077 | 10,507,219 | +21,503 | moved |
| historical_vec | 256 | replace | 10,444,942 | 10,444,942 | +26 | in place |
| historical_vec | 256 | remove | 10,383,654 | 10,383,654 | −20,410 | in place |

## Fresh paired timing and fresh-process RSS, 27 September 2026

```sh
cargo run --locked --offline -p xtask -- mutation-campaign ../new-mutation-campaign
```

The command builds the harness above, runs its paired timing mode (30 pairs
per size and operation after one warm-up), then runs its Linux memory mode 10
times per variant, size and operation, each in a fresh process, alternating
which variant of a pair runs first. Both variants of every pair must report the
same result length and root, or the campaign fails. It refuses to start when
the one-minute load average is above a sixth of the logical CPUs (at least 1).
Retained output, with the harness SHA-256 and host load at start and end:
[campaign r1](evidence-mutation-campaign-r1/CAMPAIGN.txt),
[memory raw](evidence-mutation-campaign-r1/memory-raw.txt) and
[summary](evidence-mutation-campaign-r1/memory-summary.txt),
[timing raw](evidence-mutation-campaign-r1/timing-raw.txt),
[summary](evidence-mutation-campaign-r1/timing-summary.txt) and
[paired ratios](evidence-mutation-campaign-r1/timing-paired-ratios.txt).

Timing: median over 30 pairs of historical_vec time divided by stream time,
with the smallest and largest pair. Every pair produced identical bytes and root.

| Documents | add | replace | remove |
|---:|---|---|---|
| 8 | 1.08 (0.76–1.25) | 1.03 (0.98–1.07) | 1.03 (0.96–1.71) |
| 64 | 1.08 (1.02–1.33) | 1.04 (0.98–1.45) | 1.04 (1.02–1.35) |
| 256 | 1.26 (0.91–1.52) | 1.14 (1.05–1.56) | 1.18 (0.96–1.55) |

Stream p50 at 256 documents is 4.85–4.90 ms per mutation; both variants hash
the whole collection, which dominates. An
[earlier run the same day](evidence-mutation-campaign-r1/earlier-run-timing-paired-ratios.txt)
with the same timing code gave 1.12–1.13 at 256 documents and 1.03–1.07 at 8
and 64, so the ratio moves by about 0.1 between runs.

Memory: resident-set peak of one mutation above the RSS at the reset
(hwm_after − rss_reset), median of 10 fresh processes, bytes, with the range
where it varied.

| Variant | Documents | add | replace | remove |
|---|---:|---:|---:|---:|
| stream | 8 | 90,112 | 90,112 | 69,632 |
| stream | 64 | 90,112 | 90,112 | 69,632 |
| stream | 256 | 90,112 | 90,112 | 73,728 |
| historical_vec | 8 | 114,688 (114,688–176,128) | 118,784 | 81,920 (81,920–110,592) |
| historical_vec | 64 | 872,448 (598,016–1,249,280) | 114,688 | 77,824 |
| historical_vec | 256 | 4,489,216 (3,665,920–5,050,368) | 114,688 | 77,824 |

Peak while building the bank by repeated add (hwm_before, medians across the
three operations): stream
2.42–2.46, 3.58 and 7.54–7.55 MB; historical_vec 2.58–2.61, 5.69–5.81 and
17.19–17.28 MB at 8, 64 and
256 documents. Each historical add serializes the whole collection.

How to read it:

- A stream mutation raises the resident set by about 68–88 KiB at every size:
  the new 20 KB text plus page and allocator granularity.
- historical_vec replace and remove request about 10.4 MB of temporary heap at
  256 documents (DHAT above), yet their resident peak is about 112 KiB: the
  temporary serialization buffer reused pages already resident after building
  the bank. Only add, whose buffer grows to twice its capacity, needed new
  pages: 0.87 MB at 64 and 4.49 MB at 256 documents.
- Heap bytes and resident pages are different metrics. The resident cost of
  the historical variant depends on what the allocator already holds, so a
  process that did not just build a bank could see a larger peak.

Limits: one host (the one in the campaign file), 10 processes per combination,
glibc 2.39 allocator. The load average was 3.1 at the start and 4.2 at the end
because of unrelated processes. The host mixes two CPU core types and no
affinity is set: at 256 documents single mutations took 2.1–8.0 ms, so only
the paired ratio, not an absolute time, is compared. After the reset, HWM
exceeded RSS by up to 110,592 bytes (historical_vec, 8 documents; 0 elsewhere),
which bounds the error of those rows. Reading proc allocates. Warm process,
not cold start. The memory mode reads Linux proc and runs only on Linux.
