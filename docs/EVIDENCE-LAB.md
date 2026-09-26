# Multi-document Evidence Lab

Rust runtime, local CPU/RAM, no LLM, no server and no shell execution.
This is a new local-review candidate, not the private conversational application.

```sh
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence
```

Interactive example using the included authored fixtures:

```text
add crates/gel-source/fixtures/evidence-lab/memory.txt
add crates/gel-source/fixtures/evidence-lab/unicode.txt
list
find ram is volatile
proof 1
find zażółć gęślą
proof 1
find imaginary evidence
save /YOUR/OWN/DIRECTORY/new.gelset
exit
```

Restart and use `load RETAINED_SHA256 /YOUR/OWN/DIRECTORY/new.gelset`.
No dependency on the original files is required to reopen the snapshot.
Save refuses existing targets. Paths with spaces need no quotes. `replace ID PATH`
keeps the document ID/title but replaces its text. `drop ID` removes it from RAM;
old saved snapshots are NOT erased. Deletion is not secure forgetting.

## Source contract

- Up to 1024 documents, each nonempty UTF-8 up to 16 MiB; 64 MiB aggregate text.
- Titles: nonblank, at most 512 UTF-8 bytes, no control characters. Duplicate
  titles are permitted; document IDs, not titles, disambiguate sources.
- IDs increase and are not reused within a lineage, including across an empty
  collection save/reopen. Loading an older trusted snapshot restores that older
  lineage: this is not anti-rollback or a distributed globally unique identity.
- Every mutation increments revision and recomputes the canonical snapshot SHA256.
  Even unrelated document changes invalidate previous hits. Identical snapshots
  have identical roots; this hash is not an authentication key or account identity.
- Search independently scans each document using the [phrase contract](DOCUMENT-READOUT.md).
  It never joins two documents. Exact quote offsets are local to the named document.
- Up to 4 matching lines per document and 16 shown overall, in ascending document
  ID and source-line order, NOT relevance order. `matching_lines` counts all matches.
  Queries allow up to 512 bytes/32 words. Lines over 4096 bytes are skipped and
  produce INCOMPLETE, even if other lines match. UNKNOWN means no match in the
  fully examined bounded input, not absence of knowledge everywhere.
- `proof N` checks current root, source hash, exact byte range and quote.
  It proves correspondence to retained source bytes, not factual truth.
- No implicit learning, training, translation, paraphrasing or semantic ranking.
- Terminal controls and bidi formatting are escaped. This is not an HTML viewer.

## Canonical plaintext GELSET01

All integers little-endian. Header: 8-byte magic, u64 revision, u64 next-ID,
u64 document count. Entries in increasing ID order: u64 ID, u32 title byte length,
u64 text byte length, exact title bytes, exact text bytes. No padding/trailing data.
Total bytes = 32 + sum(20 + title UTF-8 bytes + source UTF-8 bytes).

The separately retained SHA256 covers the entire file, including IDs and revision.
Load checks the independent pin before bounded structural/UTF-8 validation.
It rejects noncanonical order, duplicates, invalid counters, lengths and limits.
A hash provided by an untrusted sender alongside the file does not authenticate it.

The [no-replace publisher](SOURCE-BUILDER.md) uses a same-directory temporary file,
file sync, hard-link installation and directory sync on Unix. Parent directories
must be owner-controlled; hostile concurrent path changes are outside this portable
contract. There is no overwrite fallback where hard links are unavailable.
Non-Unix directory power-loss durability is not established. A process kill can
leave a temporary file; no automatic cleanup scans user directories. An error
after installation can leave a complete destination, so inspect rather than overwrite.

The tests kill owned child processes after ACK and at observable publication.
The latter is scheduling-dependent, not deterministic coverage of every syscall.
No ACK permits either absent or complete new output. Old snapshots must remain intact.
No physical power cut, cache-controller flush proof or protection from root is claimed.

Search timers exclude terminal output/import/load. Mutation includes serialization
and hashing of the entire collection. RAM holds exact strings, hashes and results;
disk snapshots are plaintext. Do not use this tool as an encrypted private vault.
