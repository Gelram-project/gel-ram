# Timing examples

Scope: the wall-time examples `crates/*/examples/speed_*.rs` and the identity
programs `crates/*/examples/identity_*.rs`. Timing examples measure elapsed
wall time of fixed, deterministic work and fingerprint its output bits.
Identity programs compare current code with an exact copy of earlier code and
measure no time. Neither measures answer quality. Results from different
machines, loads, pinning or thread counts are not comparable.

Both kinds print through two shared, dependency-free helpers in the workspace
root, `tools/speed_support.rs` and `tools/identity_support.rs`, included with
`#[path]` (no crate gains a dependency). The output check is a SHA-256
(FIPS 180-4) computed by the helper, never by the code being timed. The helper
checks its own digest against known answers before every timing example.

`cargo run --locked --offline -p xtask -- speed-convention` checks the file
names, that every example prints through the helpers, and that every example
is listed below. It prints `SPEED_CONVENTION=PASS` or `SPEED_CONVENTION=FAIL`;
`xtask verify` runs it. It reads source text only: it does not show that an
example measures what its documentation says.

## Output grammar of a timing example

Standard output, in this order and nothing else:

```text
SPEED_SETUP example=<stem> runs=<R>[ <key>=<value>]*     exactly one, first line
SPEED name=<case> run=<i> ns=<n>                         one per case and run
SPEED_CHECK sha256=<64 lowercase hex>                    exactly one, last line
```

- `<stem>` is the file name without `.rs`. `runs` is always the second field;
  the other fields are the inputs in the order of the file's documentation.
  Keys are `[a-z0-9_]+`, values contain no white space, lists are
  comma-separated (`threads=1,8`). A thread count is always `threads=`.
- `<case>` is `[a-z0-9_]+` and unique within the example. Sizes are part of the
  name (`_64mib`, `_8192`, `_k256`); a thread count is the suffix `_t<N>`.
- `<i>` counts from 0 to R-1. `<n>` is `Instant::elapsed().as_nanos()` of the
  timed region only, in decimal. The order of `SPEED` lines is not part of the
  grammar; a reader keys them by `name=`.
- `SPEED_CHECK` covers the outputs of every run, or of run 0 only when the
  example compares every later run with run 0 and stops on a difference.
- A failed comparison prints `SPEED_FAIL reason=<slug>` on standard error,
  prints no `SPEED_CHECK` line and exits with status 1. A bad argument prints a
  message on standard error and exits with status 2. Status 0 means the run
  completed and `SPEED_CHECK` was printed.

## Output grammar of an identity program

```text
IDENTITY name=<case> cases=<n> equal=<n>                 one per case
IDENTITY_VERDICT=PASS|FAIL                               exactly one, last line
```

PASS only when at least one case ran, every case made at least one comparison
and every comparison was equal. Exit status 0 on PASS and 1 on FAIL. Each
program exposes `run(scale) -> Vec<Tally>`; `main` runs the full scale, an
optional test wrapper `crates/<crate>/tests/identity_<topic>.rs` may run a
smaller one. The wrapper is a convenience, not the verdict.

"Exact copy of the earlier code" means the earlier function bodies kept
verbatim apart from their names, either in the program itself or in a
`src/<topic>_reference.rs` file compiled only into unit tests and, through
`#[path]`, into the program. Every result is compared bit for bit, errors
included.

## Arguments

Positional only: `[RUNS] [<extra>...]`.

- `RUNS`: repetitions per case, 1..=999, default 9. 0 is refused.
- `THREADS`: where an example takes one, 1..=999, default 8.
- Other arguments (`DIR`, `WORK_DIR`, `RECORDS`, `PHYSICS_REPEATS`,
  `BIN_DIR`) are described in each file; numeric ones are repeated in
  `SPEED_SETUP`.

## Running one example

```text
cargo run --locked --offline --release -p <crate> --example <stem> -- [RUNS] ...
```

`speed_bench_physics` starts the release binaries as child processes; build
them first with `cargo build --release -p gel-bench -p gel-physics --bins --examples`.
`speed_bench_checks` does the same for `gel-bench` alone
(`cargo build --release -p gel-bench --bins --examples`; in `xtask speed-ab`,
`--with-bins gel-bench`).
Pin every timing run to one L3 domain as described in `docs/PERFORMANCE.md`.

## Comparing two trees: A/B with an A/A control

1. Two separate checkouts of the baseline commit (A and A') and one of the
   candidate (B), each built in its own target directory.
2. Copy the `speed_<topic>.rs` file and `tools/speed_support.rs` into A and A'
   when the baseline does not have them. Examples marked "yes" in the last
   column below use only API that also exists on the earlier tree.
   `xtask speed-ab` (`docs/SPEED-AB.md`) does this step: it carries the example
   and every file it includes through `#[path]`, and removes them afterwards.
3. Run rounds with rotated order (B A A', A A' B, A' B A), the same `RUNS` in
   every round.
4. Per case, take the median `ns` per tree; report A/B next to A/A'. A ratio
   inside the A/A' spread is not a difference.
5. `SPEED_CHECK` must be equal in A, A' and B. A difference stops the
   comparison: the change altered output bits.
6. Record the commit, the hash of the example file and of the helper, the
   machine, the pinning and the load.

## Same-binary comparisons (`*_paired`)

A `speed_<topic>_paired.rs` example times a reference side (`<base>_ref`)
and one or more variants (`<base>_<variant>`) in the same process, and
compares their outputs in every run. The ratio is the median of `<base>_ref`
over the median of `<base>_<variant>` from the same binary. These examples
may need API that the earlier tree lacks.

## Examples and their control cases

Control cases run code that the change measured by the example does not
modify; their ratio shows the noise of the comparison itself. Every example is
also a control for changes outside its own crate.

| example | crate | cases | control cases | runs on an earlier tree |
|---|---|---|---|---|
| `crates/gel-core/examples/speed_crc.rs` | gel-core | 3 CRC-64/ECMA cases over 64 MiB | none inside; the A/A' comparison | yes |
| `crates/gel-core/examples/speed_crc_large.rs` | gel-core | 6 cases over 256 MiB, 4 MiB and 3 MiB | `blocks1mib_256mib`, `records128_256mib`, `oneshot_3mib` | yes |
| `crates/gel-core/examples/speed_crc_threads_paired.rs` | gel-core | `crc_256mib_ref`, `_t2`, `_t4`, `_t8`, `_auto` | the `_ref` side | no |
| `crates/gel-live-lab/examples/speed_witness.rs` | gel-live-lab | `witness_scores` | none inside; the A/A' comparison | yes |
| `crates/gel-live-lab/examples/speed_literal.rs` | gel-live-lab | 6 `literal::read` cases, 1 B to 64 MiB, one refused | none inside; the A/A' comparison | yes |
| `crates/gel-phase-quad/examples/speed_q8_read.rs` | gel-phase-quad | 4 single-query scan and read cases | the whole example for multi-query and mask changes | yes |
| `crates/gel-phase-quad/examples/speed_q8_queries.rs` | gel-phase-quad | `queries16_t1`, `queries16_t<T>` | the whole example (single-query scan only) | yes |
| `crates/gel-phase-quad/examples/speed_q8_queries_paired.rs` | gel-phase-quad | `queries16_t<W>_ref`, `queries16_t<W>_many` | the `_ref` sides | no |
| `crates/gel-phase-quad/examples/speed_q8_masks.rs` | gel-phase-quad | 6 fixture, record, prepare and view cases | `grid_view_4poles_8192` | yes |
| `crates/gel-reader/examples/speed_topk.rs` | gel-reader | 6 Top-K cases over 524,288 ORBs | `uniform_k8`, `ties_k256` | yes |
| `crates/gel-reader/examples/speed_topk_threads_paired.rs` | gel-reader | `<bank>_k<K>_ref`, `<bank>_k<K>_t<T>` | the `_ref` sides | no |
| `crates/gel-reader/examples/speed_top1_queries_paired.rs` | gel-reader | `top1_q16_ref`, `top1_q16_many`, `top1_q16_t<T>_ref`, `top1_q16_t<T>_many` | the `_ref` sides | no |
| `crates/gel-reader/examples/speed_views.rs` | gel-reader | 7 view cases over 65,536 ORBs | `apply_rotate_65536`, `apply_reverse_65536`, `apply_xor_mask_65536` | yes |
| `crates/gel-source/examples/speed_source_load.rs` | gel-source | 7 add, load and backup cases | `collection_load` | yes |
| `crates/gel-source/examples/speed_sha256.rs` | gel-source | `large`, `small` | none inside; the A/A' comparison | yes |
| `crates/gel-source/examples/speed_search.rs` | gel-source | 3 document cases, `collection_64x1mib` | none inside; the A/A' comparison | yes |
| `crates/gel-source/examples/speed_line_ascii.rs` | gel-source | 6 document cases dominated by the line end search and ASCII check, 512 KiB and 16 MiB | `polish_16mib`, `over_limit_16mib` | yes |
| `crates/gel-store/examples/speed_store_io.rs` | gel-store | `write_`, `verify_`, `open_<size>mib` | none inside; the A/A' comparison | yes |
| `crates/gel-structural/examples/speed_structural_encode.rs` | gel-structural | 6 residual and prototype cases | `apply_near`, `popcount_near` | yes |
| `crates/gel-history/examples/speed_history.rs` | gel-history | 10 history and sparse residual cases | `to_bytes_walk_k3_100k`; nearly `from_bytes_unrelated_20k` | yes |
| `crates/gel-bench/examples/speed_bench_physics.rs` | gel-bench | whole `gel-bench` and `gel-physics` runs | nearly `gel_bench_8192_t1` (process start dominates) | yes, after `--bins` |
| `crates/gel-bench/examples/speed_bench_checks.rs` | gel-bench | whole `gel-bench` runs over 1,310,720 orbs (`_t1`, `_t4`), 131,072 and 8,192 orbs; the check covers every non-timing output line | nearly `gel_bench_8192_t1` (process start dominates) | yes, after `--bins` |
| `crates/gel-bench/examples/speed_bench_setup_paired.rs` | gel-bench | 4 setup cases, `<base>_ref` and `<base>_new` | the `_ref` sides | no |
| `crates/gel-source/examples/speed_nfc_fast.rs` | gel-source | 8 line cases (200 searches each) and 6 document cases of phrase search over normalization inputs | `ascii_line`, `polish_nfc_line`, `doc_ascii`, `doc_polish_nfc` | yes |
| `crates/gel-source/examples/speed_corpus_load.rs` | gel-source | 5 catalog load cases, 256 KiB to 64 MiB, one refused | `small_load_256kib`, `reject_first_64mib` | yes |
| `crates/gel-source/examples/speed_corpus_load_paired.rs` | gel-source | `load_64mib_ref`, `load_64mib_t<T>`, `load_r50k_16mib_ref`, `load_r50k_16mib_t<T>` | the `_ref` sides | no |
| `crates/gel-source/examples/speed_quote_ranges.rs` | gel-source | 3 catalog load cases whose rows repeat passage ranges, one refused, and 3 with distinct ranges | `distinct_encoded_64mib`, `distinct_r50k_16mib` | yes |
| `crates/gel-source/examples/speed_load_overlap.rs` | gel-source | 8 bundle, corpus, snapshot and backup load cases, two refused | `bundle_load_512kib`, `collection_load_512kib`, `encoded_load_64mib`, `backup_inspect_64mib` | yes |
| `crates/gel-source/examples/speed_backup_restore.rs` | gel-source | 6 backup create, inspect and restore cases | `backup_create_64mib`, `backup_inspect_64mib`, `backup_restore_withdrawn_64mib`, `backup_restore_target_exists_64mib` | yes |
| `crates/gel-source/examples/speed_lowercase.rs` | gel-source | 5 scripts of 16 documents of 512 KiB, `_t1` and `_t<T>` | `ascii_16x512kib_t1`, `ascii_16x512kib_t<T>` | yes |
| `crates/gel-source/examples/speed_lowercase_paired.rs` | gel-source | 8 scripts of 4 MiB lines, `<script>_ref` and `<script>_fast` | the `_ref` sides; `dotted_i_fast` (both sides call the standard library) | no |
| `crates/gel-source/examples/speed_catalog_parse.rs` | gel-source | 6 catalog writer, reader and lookup cases over 50,000 records | `quote_k50`, `find_title_k5` | yes |
| `crates/gel-source/examples/speed_tree_gates.rs` | gel-source | 5 whole-tree `xtask` gates as child processes, optionally `recorder_lint` | `ci_policy` | yes, after building `xtask` |
| `crates/gel-store/examples/speed_store_crc.rs` | gel-store | 8 build, verify, open and CRC cases, 1 MiB to 256 MiB | `build_1mib`, `verify_1mib`, `open_1mib` | yes |
| `crates/gel-store/examples/speed_store_overlap.rs` | gel-store | 12 verify, open and CRC cases, 1 MiB to 256 MiB, one refused, three with a partial last read block | `verify_1mib`, `verify_8mib`, `open_8mib`, `crc_256mib` (the small cases control the code path, not the allocator state left by earlier cases in the same process) | yes |
| `crates/gel-reader/examples/speed_top1_many.rs` | gel-reader | `single16_<n>_t1`, `many16_<n>_t1`, `single16_<n>_t<T>`, `many16_<n>_t<T>` over two banks | the `single16_*` cases | yes |
| `crates/gel-reader/examples/speed_top1_many_paired.rs` | gel-reader | `queries16_<n>_t1_ref`, `_many`, `queries16_<n>_t<T>_ref`, `_many` over two banks | the `_ref` sides | yes |
| `crates/gel-source/examples/speed_read_regular.rs` | gel-source | `read_regular` of 64 MiB, 16 MiB, 1 MiB and 4 KiB files, `bundle_load_64mib`, `refused_64mib_plus1`, `digest_64mib` | `refused_64mib_plus1`, `digest_64mib` | yes |
| `crates/gel-source/examples/speed_bundle_memory.rs` | gel-source | `bundle_load_64mib` after a setup that needs less memory than one load, so the peak resident memory of the process (`/usr/bin/time -v`) is that of the load | none inside; the A/A' comparison | yes |

Identity programs:

| program | crate | compares |
|---|---|---|
| `crates/gel-core/examples/identity_crc_threads.rs` | gel-core | threaded and combined CRC-64/ECMA with a table copy and a bit-serial form |
| `crates/gel-phase-quad/examples/identity_q8_queries.rs` | gel-phase-quad | multi-query scan with single-query scan and pair reads; witness grid pin |
| `crates/gel-phase-quad/examples/identity_q8_masks.rs` | gel-phase-quad | word-level mask paths with the earlier per-position code |
| `crates/gel-reader/examples/identity_top1_queries.rs` | gel-reader | multi-query Top-1 with single-query Top-1 and the earlier loop |
| `crates/gel-reader/examples/identity_views.rs` | gel-reader | word-level affine views with the earlier per-bit code |
| `crates/gel-structural/examples/identity_structural_encode.rs` | gel-structural | popcount-first residual and prototype paths with the earlier code |
| `crates/gel-history/examples/identity_history_rebuild.rs` | gel-history | streaming sparse residuals and rolled history rebuild with the earlier code |
| `crates/gel-bench/examples/identity_bench_setup.rs` | gel-bench | threaded bank fill, threaded bank CRC, Top-K reference and chain order with the earlier serial code |
| `crates/gel-source/examples/identity_nfc_fast.rs` | gel-source | table-based normalization properties and reused buffers with the earlier binary-search code, on every scalar value and seeded strings |
| `crates/gel-source/examples/identity_corpus_load.rs` | gel-source | catalog loading on several threads with the earlier one-thread loader, results and first errors |
| `crates/gel-source/examples/identity_load_overlap.rs` | gel-source | bundle and snapshot loading with the whole-file digest overlapped with the part digests, which stop early under a wrong pin, with the earlier one-thread code |
| `crates/gel-source/examples/identity_backup_restore.rs` | gel-source | backup restore from the inspected bytes with the earlier restore, results and errors |
| `crates/gel-source/examples/identity_lowercase.rs` | gel-source | line lowercasing on every scalar value and document search with the earlier code |
| `crates/gel-source/examples/identity_catalog_parse.rs` | gel-source | catalog reader, writer and hex output with the earlier code, including malformed input |
| `crates/gel-source/examples/identity_tree_gates.rs` | gel-source | whole-tree `xtask` gates on one shared walk with the earlier sequential gates |
| `crates/gel-store/examples/identity_store_crc.rs` | gel-store | store payload CRC paths and read block size with the earlier code, files and errors |
| `crates/gel-reader/examples/identity_top1_many.rs` | gel-reader | blocked multi-query Top-1 with the earlier per-ORB loop and single-query Top-1 |

## Limits

- A thread gain depends on free cores; on one core the threaded paths run
  serially.
- Process-level examples include process start and exit in every run.
- `SPEED_CHECK` compares output bits between builds of the same example file;
  it is not a pin across versions of that file.
- No result of these examples is a claim about other hardware.
