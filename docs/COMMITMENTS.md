# Commitments

A commitment is a SHA-256 published now that fixes the exact contents of a
directory that is not published. It shows that those contents existed by the
date of the commit that added the line, and that they cannot be changed later
without the check failing. It says nothing about what the contents are or
whether they are correct.

## How a commitment is made

```sh
cargo run --locked --offline -p xtask -- commitment DIR MANIFEST_OUT [--exclude NAME]...
```

The command walks `DIR` and writes a manifest to `MANIFEST_OUT`, which must be
outside `DIR` and must not exist yet. It prints

```text
COMMITMENT files=N links=M dirs=D bytes=B sha256=HEX
```

where `HEX` is the SHA-256 of the manifest bytes. Only `HEX` is published here.
The manifest and the files stay with their owner, who keeps an unchanged copy.

The manifest (`GEL_COMMITMENT_2`) is plain text:

- the header line `GEL_COMMITMENT_2`;
- one `exclude NAME` line per excluded name, sorted; a file or directory with
  that exact name is skipped wherever it occurs;
- one line per directory, `dir PATH`, so an empty directory is part of the
  contents;
- one line per regular file, `file SHA256 LENGTH PATH`, and per symbolic
  link, `link SHA256-OF-TARGET PATH` (links are recorded, never followed);
- paths relative to `DIR`, `/`-separated, in byte order.

Names that are not UTF-8 or contain a line break, and other file types
(sockets, devices), stop the command with an error instead of being skipped.
Permissions, owners and times are not recorded.

The manifest describes one state of the directory. A file whose length or
modification time differs before and after it is read, or a second walk that
sees a different set of entries, sizes or times than the first, stops the
command with an error and no manifest is written. Make the commitment from a
copy that nothing else writes to.

## How a commitment is checked

When the owner hands over a manifest and the files:

```sh
sha256sum MANIFEST                      # equals the published HEX
cargo run --locked --offline -p xtask -- commitment-check MANIFEST DIR
```

`COMMITMENT_CHECK=PASS sha256=HEX` means that every directory, file and link
under `DIR`, apart from the recorded exclusions, is exactly as committed, and
that nothing was added or removed. `COMMITMENT_CHECK=FAIL first=PATH` names the
first path in byte order that differs, and the command exits with status 2. The
manifest format is simple enough to rebuild with `sha256sum`, `find` and a sort;
the tests in `xtask/src/commitment.rs` compare with such a manifest.

## Published commitments

| Date (UTC) | Label | Directories | Files | SHA-256 of the manifest |
|---|---|---|---|---|
| 2026-10-20 | Private source tree 1: Rust sources only (`*.rs`, `Cargo.toml`, `Cargo.lock`) | 1,436 | 4,054 | `69c10bd58f5173e11a30b1f27072566d1b4dcbeee504882d7359fd0f1f92fefd` |
| 2026-10-21 | Private data set 1 | 6 | 44 | `ab5e8b71d8f6d8202065714ceb6c0aa8837dd7fbfb805ef33dcb5022f75da92f` |

The draft of PR #81 first listed `26b84cb5…deafe9` for Private source tree 1 in
an earlier manifest format that did not record directories. That digest is
withdrawn; the 2026-10-20 line for Private source tree 1 replaces it.
