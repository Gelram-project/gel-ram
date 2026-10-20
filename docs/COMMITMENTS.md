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
COMMITMENT files=N links=M bytes=B sha256=HEX
```

where `HEX` is the SHA-256 of the manifest bytes. Only `HEX` is published here.
The manifest and the files stay with their owner, who keeps an unchanged copy.

The manifest (`GEL_COMMITMENT_1`) is plain text:

- the header line `GEL_COMMITMENT_1`;
- one `exclude NAME` line per excluded name, sorted; a file or directory with
  that exact name is skipped wherever it occurs;
- one line per regular file, `file SHA256 LENGTH PATH`, and per symbolic
  link, `link SHA256-OF-TARGET PATH` (links are recorded, never followed);
- paths relative to `DIR`, `/`-separated, in byte order.

Names that are not UTF-8 or contain a line break, and other file types
(sockets, devices), stop the command with an error instead of being skipped.
Permissions, owners and times are not recorded.

## How a commitment is checked

When the owner hands over a manifest and the files:

```sh
sha256sum MANIFEST                      # equals the published HEX
cargo run --locked --offline -p xtask -- commitment-check MANIFEST DIR
```

`COMMITMENT_CHECK=PASS sha256=HEX` means that every file and link under `DIR`,
apart from the recorded exclusions, is exactly as committed, and that nothing was
added or removed. `COMMITMENT_CHECK=FAIL first=PATH` names the first path in
byte order that differs, and the command exits with status 2. The manifest
format is simple enough to rebuild with `sha256sum` and a sort; the tests in
`xtask/src/commitment.rs` compare with such a manifest.

## Published commitments

| Date (UTC) | Label | Files | SHA-256 of the manifest |
|---|---|---|---|
| 2026-10-20 | Private source tree 1: Rust sources only (`*.rs`, `Cargo.toml`, `Cargo.lock`) | 4,054 | `26b84cb55bc97dedb97649027545da16efa969694e8593a03e5d4acf75deafe9` |
