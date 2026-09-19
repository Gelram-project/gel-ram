# GEL Live Lab

An interactive English-language terminal demonstration of the public Rust core.
Everything runs locally. No LLM, network service, GPU backend or subscription.
This is a laboratory, not an unrestricted chatbot.

The two existing repository films preview a separate private application, not
this public laboratory. Run the commands below to inspect this implementation.

## Start

```sh
cargo run --locked --offline --release -p gel-live-lab
```

Dependencies and the Rust toolchain must already be available for offline builds.
Use a terminal at least 100 columns by 40 rows. Colors and redraw are enabled
only on an interactive terminal; use `--plain` or NO_COLOR to keep a scrolling
transcript. No input is executed as a shell command.

## Try this first

```text
read 2
find independently trusted
phase 64
view 3
noise 128
tamper
read 4
```

The source panel shows a selected exact quotation, address, UTF-8 byte range,
hash and quote/validation elapsed time. `read N` selects a numbered source part;
`page N` reads more of a long quotation. Controls are escaped for terminal safety.
Built-in notes are short authored demonstration texts, not a private knowledge bank.

`find PHRASE` searches the entire immutable source payload, including across
storage-part boundaries. `match N` selects one of up to four original matching
lines and displays its global byte range. NFC and sigma handling follow the
[document contract](DOCUMENT-READOUT.md). A miss clears the old quote; skipped
overlong lines produce INCOMPLETE, not an exhaustive absence claim. Search is
limited to 16 MiB and 512 query bytes; larger imports remain readable with
`read`. Multi-node catalogs are refused by `find` to avoid joining unrelated
documents. No ranking, semantic segmentation or natural-language QA is implied.

The numeric panel is **separate synthetic Q8 data**, not an encoding of the text.
Four colored rows show the first 16 phase codes in four reversible coordinate
views. `view 0..3` highlights a row, `phase 0..255` shifts the body phase,
`mask 0..1024` changes active-mask stride (0 disables all body dimensions), and
`noise 0..1024` perturbs that many dimensions. All four inverses are checked.
Each color is a deterministic display of a phase code, not a measured RAM waveform.

The public Record occupies 1152 bytes (1024 phase bytes plus 128 mask bytes).
256 means the number of phase levels, **not the record's bit length**. The displayed
size is not the process's total RSS or all allocated view buffers. The shared score
is one piece of numeric evidence; four views do not create four independent facts.

`tamper` changes one byte of a newly constructed in-memory copy of the selected
quote and tests the original pins. It must display TAMPER=REJECTED. It does not
modify your document or saved bundle. Hash correspondence is not truth certification.

## Your own file and a real restart

```text
open /path/to/my notes.txt
read 1
save /path/to/new-bank.gelsrc
exit
```

Paths may contain spaces and need no quotes. Keep the separately displayed
64-character **bundle pin**, not the quote hash. Start the program again, then:

```text
load YOUR_RETAINED_BUNDLE_SHA256 /path/to/new-bank.gelsrc
find a phrase from your document
read 1
exit
```

The new process reads the file and checks it against your trusted pin. It does not
remember an unsaved conversation. Imported documents are bounded UTF-8, at most
64 MiB, split on character boundaries into byte-bounded parts, not semantic chunks.
The original text is never modified. A loaded bundle is read-only in this interface;
import a source document to save a new bundle. Existing output paths are refused.

Files are **plaintext**, not a private encrypted vault. Use directories you control;
the same path-race and filesystem-durability limits described in the
[source persistence guide](SOURCE-BUILDER.md) apply. No automatic save, scan of
personal directories, upload or background server runs. Importing is not training.

## What the times mean

- Source time covers quote retrieval and validation, not disk import or drawing.
- After `find`, the label changes to phrase-search time. Source integrity was
  established on import/load; hashing the entire file is not inside this timer.
- Q8 time covers one shared synthetic read after query preparation, not view
  construction, rendering, a benchmark campaign or proof of DRAM-refresh sync.
- Rendering/terminal output is excluded. Very short single measurements fluctuate.
- No token/s, invented ORB/s or cached language-model answers are shown.

Automated local demonstration and tests:

```sh
cargo run --locked --offline --release -p gel-live-lab -- --demo
cargo test --locked --offline -p gel-live-lab
```

The tests include fresh-process save/reopen, corrupt-load refusal without losing
the current session, no overwrite, exact Unicode, oversized commands, terminal
control escaping, all four inverse views and expected phase/mask responses.
This is not a physical power-failure test or a semantic-quality benchmark.
