# Snapshot publication fault matrix

The public source/collection publisher now has an internal, statically dispatched
I/O boundary for deterministic tests. Production still uses ordinary write_all,
file sync, no-replace hard link and (on Unix) parent-directory sync. There is no
runtime option to weaken these operations.

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

These are deterministic failures at the real publisher boundary, not a physically
full disk, a kernel permission experiment or a power-cut test. Existing separate
SIGKILL tests retain their narrower process-crash scope.
The old measured publisher remains preserved as
[bundle.measured.rs.txt](evidence-collection/bundle.measured.rs.txt).
