# Format mutation matrix

```text
cargo run --locked --offline -p xtask -- mutation-matrix
```

A finite, explicit matrix of byte-level mutations of the three public file
formats: GELSRC01 source bundles, GELSET01 collection snapshots and the Q8DEMO01
numeric fixture. `xtask verify` runs it and requires the regenerated report to
equal the committed [matrix-r1.txt](evidence-mutation/matrix-r1.txt) (tab-separated) byte for
byte. GELHIS01 record-history files have a matrix of their own, with its own
report, described [below](#gelhis01-record-history-files).

## Two pin modes for every mutant

| Pin | Question | Expected |
|---|---|---|
| original | Does the pin reject any changed byte before the structure is read? | always rejected |
| recomputed | With a pin that matches the changed bytes, do the structural rules decide as declared? | declared per mutant |

The original-pin rows show that a changed file is refused with the pin the owner
retained. The recomputed-pin rows exercise structural validation, because a
pin delivered alongside a changed file would match it.

## Formats, regions and operators

Each file is divided into regions: the magic, every length, count, ID and
revision field, every title, text, catalog, phase and mask area. Every region
receives the same operators: flip bit 0 of its first byte, flip bit 7 of its
last byte, set its first byte to 0x00 and to 0xFF. Integer fields additionally
get +1, −1, 0 and the maximum value. The file is also truncated before each
region and at its last byte, and extended by one byte. Mutations that would
leave the bytes unchanged are skipped. Format-specific mutants add: swapped and
duplicated GELSET01 entries, a valid letter change and a control character in a
title, and a valid Q8 phase change.

## How the expected result is fixed

- **GELSET01 and Q8DEMO01:** by a small reference reader in the matrix, written
  from the published format descriptions ([GELSET01](EVIDENCE-LAB.md),
  Q8DEMO01 in the gel-phase-quad fixture module), independent of the library
  decoder. It reads in file order, so the first rule a mutant breaks names the
  expected class. When a mutant is accepted, the content the library returns
  (revision, IDs, titles and texts; Q8 phases and masks) must equal the
  reference reading.
- **GELSRC01:** by an explicit table per region and operator, from the envelope
  description in [SOURCE-BUILDER](SOURCE-BUILDER.md): wrong magic or lengths that
  do not add up are FORMAT, an oversized length is LIMIT, a changed quoted text
  byte that stays valid UTF-8 fails its passage hash (INTEGRITY), invalid UTF-8
  is FORMAT.

A valid changed value is a different file, not an error: a changed GELSET01
revision, title or text letter and every changed Q8 phase value or mask bit are
accepted with a recomputed pin, and rejected with the original pin.

## Negative control

The matrix would be useless if it could not tell a wrong reader apart. `verify`
therefore also runs a deliberately lenient GELSET01 reader that ignores trailing
bytes and requires at least one mutant whose result differs; the summary line
reports how many. A manual check changing one declared GELSRC01 expectation
turns exactly that row into FAIL.

## What the matrix shows about the formats

- GELSRC01 reports the same INTEGRITY error for a wrong file pin and for a
  passage whose quoted bytes no longer match their hash. In the original-pin
  rows it is always the file pin that fires first, because it is checked before
  anything else is read.
- Q8DEMO01 has no checksum of its own; the pin is applied by the caller. Every
  phase value and mask bit is a valid record.
- GELSET01 accepts control characters such as NUL inside a document's text (it
  is valid UTF-8); titles reject them. Text is displayed escaped.

## GELHIS01 record-history files

```text
cargo run --locked --offline -p xtask -- history-mutation-matrix
```

A [record history](RECORD-HISTORY.md) file carries two CRC64 fields of its own,
over the header and over the payload. A mutant whose CRC64 no longer fits would
only show that the checksum works, so every GELHIS01 mutant has its CRC64 fields
recomputed, as anyone who rewrites the file can do, and the structural rules
decide. Two kinds of mutant keep a CRC64 that does not fit: those that change a
CRC64 field themselves, and one changed state byte left under the old payload
CRC64. Like Q8DEMO01, the file has no pin of its own, so the original-pin rows
are rejected by the caller's retained SHA-256 before the file is read.

- **Fixture:** four states of one record, 338 bytes: a literal, a residual
  with 3 changed bits, a residual with 1 more, and a literal again because the
  residual depth limit is reached.
- **Regions:** the seven header fields and every field of the four entries (tag,
  state; parent, depth, form, count, length and packed positions), 25 regions in
  all, with the operators above. Three mutants are added: the unsealed state
  byte, the second entry stored as a literal although its residual fits, and the
  third entry naming the first as its parent with a depth that fits that chain.
- **Expected result:** a small reference reader written from the format
  description in [record history](RECORD-HISTORY.md), with its own
  CRC-64/ECMA-182, independent of the gel-history decoder. It reads in file order,
  so the first rule a mutant breaks names the class: `MAGIC`, `HEADER_CRC`,
  `VERSION`, `HEADER`, `LIMIT`, `LENGTH`, `PAYLOAD_CRC`, `ENTRY` or `CANONICAL`.
  When a mutant is accepted, every state the library rebuilds must equal the
  reference's.
- **Negative control:** a reader without the canonical-form rule must differ
  from the reference on at least one mutant.

`xtask verify` requires the regenerated report to equal the committed
[history-matrix-r1.txt](evidence-mutation/history-matrix-r1.txt) byte for byte:

```text
HISTORY_MUTATION_MATRIX=PASS mutants=151 rows=302 accepted_valid_changes=14 failed=0 negative_control_distinguishing_mutants=2
```

The 14 accepted mutants change a byte of a stored state (8) or a packed
position that stays strictly increasing (6): each is a different valid history.
A changed padding bit after the last position is refused. With the
canonical-form rule removed from the library, the two canonical-form mutants
are accepted and those two rows fail.

## Limits

The matrix is finite: it does not enumerate every byte value at every offset,
and it uses small fixtures. It tests decoding of bytes, not publication (see
[publication fault tests](PUBLICATION-FAULT-TESTS.md)) and not search results.
