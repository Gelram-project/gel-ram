# Backup, restore, withdraw, delete and clear

Evidence Lab collections are saved as GELSET01 snapshots with an independently
retained SHA-256 pin. `gel-backup` copies such a snapshot into a verified backup
directory and restores it to a new place. Four operations with different reach
are kept apart:

| Operation | Tool | What changes | What stays |
|---|---|---|---|
| clear | `gel-evidence` command | the in-memory collection | every saved snapshot and backup |
| delete | `gel-backup delete` | one pinned file | other copies, backups, earlier snapshots; the storage blocks (not a secure erase) |
| withdraw | `gel-backup withdraw` | adds a WITHDRAWN marker; restore refuses | the backed-up bytes |
| drop ID | `gel-evidence` command | one document in memory | saved snapshots |

## Commands

Arguments are separate words, so paths may contain spaces.

```text
gel-backup create  TRUSTED_SHA256 SNAPSHOT NEW_BACKUP_DIR
gel-backup inspect TRUSTED_SHA256 BACKUP_DIR
gel-backup restore TRUSTED_SHA256 BACKUP_DIR NEW_SNAPSHOT_PATH
gel-backup withdraw BACKUP_DIR REASON
gel-backup delete  TRUSTED_SHA256 SNAPSHOT
```

Run them with `cargo run --locked --offline --release -p gel-live-lab --bin gel-backup -- ...`.
The pin is the SHA-256 you retained when the snapshot was saved; it must come
from your own record, not from the backup.

## A backup directory

`create` checks the snapshot against the pin and parses it, creates a new
directory (an existing one is refused), writes a byte-identical copy named
collection.gelset, and only then publishes MANIFEST. Each file is written to a
temporary file, synced, linked into place without replacing anything, and its
directory is synced. The manifest names the snapshot's length, SHA-256,
document count and revision.

**The manifest is the commit point.** A process killed at any earlier moment
leaves no manifest, and nothing is appended afterwards, so the state is read
from the files alone:

| State | Condition | Restorable |
|---|---|---|
| COMPLETE | manifest committed; snapshot matches it and the pin | yes, unless withdrawn |
| INCOMPLETE | no committed manifest | no |
| UNTRUSTED | manifest names another snapshot than the trusted pin | no |
| CORRUPT | files contradict each other or their formats, or an unexpected file is present | no |

`inspect` also reports whether the backup is withdrawn and how many temporary
files an interrupted publication left behind. Those files are never part of a
backup.

## Restore

`restore` accepts only a COMPLETE, not withdrawn backup, and writes only to a
path that does not exist yet; it never replaces a file. The restored file is the
same bytes and reopens with the same pin. The result tells three cases apart:

| Result | Meaning |
|---|---|
| RESTORE=PASS | the snapshot is in place and its directory was synced |
| RESTORE=NOT_PUBLISHED | nothing usable is at the target |
| RESTORE=PUBLISHED_UNCONFIRMED | the complete snapshot is at the target, but a later step (directory sync or temporary cleanup) failed, so the durability of its name is not confirmed |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | done, or backup restorable |
| 3 | backup not restorable but not damaged (INCOMPLETE or WITHDRAWN) |
| 4 | written or removed, but directory durability unconfirmed |
| 2 | refused, failed, CORRUPT, UNTRUSTED or invalid arguments |

stdout carries one result line; stderr carries usage and diagnostics.

## Tested

Unit tests cover the complete cycle, every state an interrupted creation can
leave (an empty directory, a temporary file, a snapshot without manifest, a
temporary manifest), tampering, a foreign pin, unexpected files, restore onto an
existing path, withdrawal and deletion by pin. The Linux full-disk check in CI
runs backup creation and restore on a physically full 1 MiB tmpfs: creation
leaves no manifest, restore leaves no target, and both succeed after space is
freed.

## Limits

- Directory sync is performed on Unix only. On Windows the durability of a new
  name is not confirmed by this tool.
- Use owner-controlled directories. Delete checks the pin and then removes the
  path; a concurrent writer could swap the file in between.
- A pin identifies bytes; it is not a signature. Backups are plaintext copies.
- Delete does not erase storage blocks, file-system journals or other copies.
