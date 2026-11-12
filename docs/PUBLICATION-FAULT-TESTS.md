# Snapshot publication fault matrix

The public source/collection publisher has a statically dispatched I/O boundary
for deterministic tests, `gel_source::PublicationIo`, which
`write_file_new_with` accepts for one caller-encoded file. Production still uses
ordinary write_all, file sync, no-replace hard link and (on Unix)
parent-directory sync. There is no runtime option to weaken these operations, and
the release binaries call only the plain functions.

| Injected condition | Expected API result | New destination | Previous snapshot |
|---|---|---|---|
| Successful writes of at most 3 bytes | Success | Complete, reloadable | Unchanged |
| Write failure after half the bytes | Error | Absent | Unchanged |
| Permission-denied write | Error | Absent | Unchanged |
| File sync failure | Error | Absent | Unchanged |
| Hard-link publication failure | Error | Absent | Unchanged |
| Parent sync failure after linking | Error | Complete, reloadable | Unchanged |
| Existing destination | Error | Existing bytes unchanged | Unchanged |

Every case reloads the previous snapshot with its independently retained pin.
Partial temporary files are not treated as committed snapshots. Controlled
failures clean up the temporary file; a process crash can still leave one.

A failed parent sync cannot safely be interpreted as “nothing was written”.
The complete destination can already exist. Inspect it using its trusted pin;
do not blindly overwrite or delete it. Non-Unix production directory durability
remains unproven.

These are deterministic failures at the real publisher boundary. Two further
checks use the operating system itself rather than an injected error:

- **Kernel permission denial (Unix test):** publishing into a 0500 directory
  must fail with PermissionDenied, leave no destination or temporary file, and
  keep the previous snapshot reloadable with its pin. The test first proves
  that the kernel refuses an ordinary file creation there. Run it unprivileged:
  as root the kernel permits the write and the test fails by design, rather
  than reporting a pass it did not exercise.
- **Physically full filesystem (Linux CI):** `full_disk_publication` fills a
  dedicated 1 MiB tmpfs until the kernel returns ENOSPC, then publishes. It
  requires ENOSPC from the publisher, an unchanged directory listing (no
  destination, no temporary file) and a reloadable previous snapshot. After the
  filler is removed, the same publication must succeed and reload. The program
  refuses a directory unless it is the root of its own tmpfs mount (not a bind
  of a subdirectory), its device matches that mount (so a shadowed or
  over-mounted entry cannot be mistaken for it), and the tmpfs has an explicit,
  non-zero
  size of at most 64 MiB (size 0 means unlimited; read from /proc/self/mountinfo). It refuses a
  non-empty directory, caps the filler at 64 MiB and removes it on every exit
  path, including a failure while filling or while writing its own output.
  The same run then fills the device again for [backups](BACKUP.md): creating a
  backup must fail without a committed manifest (a partial directory reads as
  INCOMPLETE), restoring must leave no target, and after the filler is removed
  the restore succeeds and reopens with the same pin.

```sh
cargo run --locked --offline --release -p gel-source --example full_disk_publication -- EMPTY_SMALL_TMPFS
```

Neither is a power-cut test. Existing separate SIGKILL tests retain their
narrower process-crash scope.

## A Q8 record save stopped at known points

```sh
cargo test --locked --offline -p gel-live-lab --test torn_write
```

The file `gel-live-lab --literal FILE --save NEW_PATH` writes, one Q8DEMO01
record of 1,164 bytes, is published through `write_file_new_with` by a child
process (the test binary run again) whose I/O steps write 64 bytes at a time
and end the process at one point: nothing after that point runs, no destructor
and no cleanup. The child prints a marker just before it stops, and the test
requires the marker, so a point that was never reached cannot pass.

| Stage | Points | New path afterwards | Temporary file left |
|---|---|---|---|
| Temporary file | after 0 to 19 pieces of 64 bytes, and after its flush (21) | absent | 1, holding what was written |
| Linked | after the hard link (1) | the whole record under its pin | 1, the same bytes |
| Directory flushed | after the parent flush, before the temporary file is removed (1) | the whole record under its pin | 1, the same bytes |

At every one of the 23 points the new path holds nothing or the whole record,
reopened as `--reopen` does; never part of it. A fresh save afterwards succeeds
where the path was free and is refused where the record is already there, which
it leaves unchanged.

**Control.** A naive writer that creates the new path and writes the same
pieces straight into it is stopped at the same 21 points of the first stage:
it leaves part of a record at 19 of them (after 0 to 18 pieces) and a whole one
at the last two. The test requires those 19, so a check that cannot see a
partial file fails. Replacing the publisher with the naive writer in the child
makes the first test fail.

The format explains why the pin is needed: the decoder refuses a file cut
after 0, 8 or 12 bytes, after the 1,024 values of the record or one byte short,
and a file one byte longer, but an eight-record file whose bytes 4096 to 8191 are
zeroed, as a file system can leave a block, still decodes, because every byte is
a valid value or mask byte. Only the SHA-256 pin kept with the file tells it
apart.

Ending a process at a point is not a power cut. The test runs on Linux, macOS
and Windows in CI; the directory flush exists on Unix only.

## Faults of saved files, on disk and between processes

```sh
cargo test --locked --offline -p gel-history -p gel-store -p gel-source --test faults
```

Three test files take the files a save leaves, write the bytes a damaged disk
could leave, start a second process or restart. Where processes matter, the child is the
test binary run again.

| Fault | Record history | Store | Collection and bundle |
|---|---|---|---|
| File cut at every byte | every length, on disk through `open_verified` | a version 1 file at every length past its header (version 2 sweeps sit beside the code) | every length, under the kept pin and under a pin taken from the cut file, so the structure alone refuses it; 12 stated cuts of a file over 1 MiB, the threaded load |
| Damage that keeps the length | a splice of two versions at every byte, zeroed blocks of 64, 512 and 1,024 bytes, two blocks exchanged, one stale block | a splice of two generations, zeroed blocks of 128, 512 and 4,096 bytes, blocks exchanged, a new header over an old payload, a header sealed for a shorter file; version 1 too | a splice of two files at every byte and zeroed blocks of 16, 64 and 256 bytes, each under the kept pin |
| Two writers on one path | two processes save 40 versions each while three threads read; each read and the last file is one whole version of one writer, and no temporary file stays (Unix) | the same, with loading and streaming verification on the readers (Unix) | two processes meet at the link: one publishes, the other is refused, and the file is the winner's |
| Restart after a partial write | leftovers at the writer's own names: 127 are skipped, 128 fail closed, and the file stays | the same; a writer killed in 8 trials, each a little later into its next save, leaves the last acknowledged generation or a later whole one, and a fresh process resumes (Unix) | a save stopped after every byte, after the flush, after the link and after the directory flush; a restart reaches the bytes of an uninterrupted save and never touches the leftover. 32 leftover names are skipped, 33 fail closed |
| Reader beside a writer | saves with readers end in the same bytes as saves without; no read sees part of a file (Unix) | as in the row of two writers | a reader in step with the writer at every piece, the flush, the link and the directory flush finds nothing before the link and the whole file after it |
| What reading leaves | on a read-only file with an old time of change: opening, a refused limit and a refused file change no byte, time, permission or name | loading, verifying, and a refused `write_if_newer`, version 1 included | loading, a wrong pin and a refused save |

Limits:

- A killed or stopped process is **not a power cut**. Nothing here establishes
  power-loss durability, for any of these formats.
- Saves of a history or a store replace the file by a rename, and the last
  rename wins. Two writers leave one whole file, not both sets of changes:
  this is not a compare-and-swap, and generations can go backwards between two
  writers, because `write_if_newer` assumes one writer. Only the no-replace
  publisher of collections and bundles refuses a second writer.
- The three tests of what reading leaves compare bytes, time of change,
  permissions and directory names. The time of access can change with any read and is not compared.
- A same-length exchange of two texts is a valid collection under a pin taken
  from the changed file; only a pin kept apart from the file tells them apart.
- The tests that race a replaced file, and the kill of a store writer, are
  compiled on Unix only: replacing a file another process has open is a rename
  guarantee of that platform. One machine and one file system per run.
- 16 of the 20 tests compile on Windows and all 20 on macOS. They run there only
  in CI; the recorded reports of a9b5c6a, the v0.9.0 release, show all 20
  passing on Linux and macOS and the 16 on Windows
  ([recorded reports](evidence-ci/README.md)). No timing property is covered.

The old measured publisher remains preserved as
[bundle.measured.rs.txt](evidence-collection/bundle.measured.rs.txt).

## Valid hash does not make a valid structure

The collection tests recompute the input hash for each deliberately malformed
fixture: inconsistent next-ID/revision, missing or invented record counts,
duplicate/out-of-order IDs, ID overflow, empty/oversized/overflowing text or
title lengths, blank/control-character titles and invalid UTF-8. These must
fail structure validation even though their supplied hash matches.

Separate valid near-exhaustion snapshots exercise u64 revision and next-ID
overflow. Rejected add/replace/remove operations leave bytes, root and existing
citation validity unchanged; the resulting state still roundtrips.

An executable CLI test loads a well-hashed invalid snapshot over a live
collection. It requires a structural refusal and confirms that the previous
document remains queryable with a valid citation. No input is relabelled as
trusted merely because the caller supplied its matching hash.
