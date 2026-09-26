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
mutation timings with Ocean searches, Q8 decoding or private GEL/WAVE timings.

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

The reported bytes convert Linux kB by 1024. VmHWM is a lifetime high-water mark:
it includes bank construction and is not reset at the operation boundary.
RSS includes process/runtime state and allocator-retained pages. Sampling and
reading proc also have overhead. These are not allocation counts, copied-byte
counts, or a precise isolated mutation peak. A zero HWM increase does not mean
zero allocation. The single operation latency is diagnostic, not a percentile.
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

The measured sources are the current ones: harness SHA-256
fd07ca267a7dee3beb7679c8d36fa5da05546a7839f74456560fed5ffb29263b (printed by
every run as `HARNESS_SHA256=`), current collection SHA-256
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
