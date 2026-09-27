# Format mutation matrix

```text
cargo run --locked --offline -p xtask -- mutation-matrix
```

A finite, explicit matrix of byte-level mutations of the three public file
formats: GELSRC01 source bundles, GELSET01 collection snapshots and the Q8DEMO01
numeric fixture. `xtask verify` runs it and requires the regenerated report to
equal the committed [matrix-r1.txt](evidence-mutation/matrix-r1.txt) (tab-separated) byte for
byte.

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

## Limits

The matrix is finite: it does not enumerate every byte value at every offset,
and it uses small fixtures. It tests decoding of bytes, not publication (see
[publication fault tests](PUBLICATION-FAULT-TESTS.md)) and not search results.
