# Evidence Lab compared with grep and sha256sum

```text
cargo run --locked --offline -p xtask -- bench-compare NEW_DIR_OUTSIDE_CHECKOUT
cargo run --locked --offline -p xtask -- bench-compare NEW_DIR --answers-only
```

A paired comparison on the same public corpus: the repository README and
documentation pages (this page excluded), a Rust Book excerpt (MIT) and a
synthetic Unicode file. The output directory holds the corpus manifest with
SHA-256 of every file, the tested revision, tool versions, locale, network
isolation status, host load, every timing sample and a summary.

## The tools do not follow the same rules

| | GEL `find` | `grep -n -i -w -F` |
|---|---|---|
| unit | a line split at CR or LF | a line split at LF |
| match | the query's words as consecutive words | the literal query string at word boundaries |
| words | letters and digits; punctuation and underscore separate words | letters, digits and underscore |
| case and Unicode | NFC, lowercase, final sigma equals sigma | locale case folding, no normalization |
| over-long lines | lines over 4096 bytes are skipped (INCOMPLETE) | read |
| provenance | byte range, document SHA-256, citation check | none |

So "rust 1 85 0" matches "Rust 1.85.0" in GEL and nothing in grep, and a
decomposed "café" matches in GEL only. Neither answer is "wrong": they answer
different questions.

## How answers are compared

For each of 36 fixed queries (common, rare, absent, multi-word, punctuation,
Unicode) the answer is the number of matching lines, and every line GEL shows
(at most 16 per query) must also be listed by grep. Both tools give the same
answer only if the counts are equal, GEL skipped no line and every shown line
is contained. Otherwise the answer is DIFFERENT with the most likely reason.
**Times are compared only for queries with the same answer.** The different
queries stay in the answer table.

## What is timed

| Phase | GEL | Baseline |
|---|---|---|
| process-workload | one `gel-evidence --batch` process: load the pinned snapshot, run every same-answer query | one grep process per same-answer query, summed |
| integrity-process | one process: load the snapshot with its pin (hash and structural parse) | `sha256sum` of the same file (hash only) |
| in-memory-search | `search_ns` per query from the batch output: search only, no process start or load | none |

Wall time is measured around each process with the page cache warm; no
cold-cache claim is made. Each repetition alternates which tool runs first.
Three warm-up repetitions are recorded and excluded; 30 are measured. The
summary reports the nearest-rank median and p95, labelled as small samples; no
deeper tail is claimed. The GEL process's peak resident set size comes from its
own batch summary (Linux VmHWM). grep's memory is not measured.

## CI

Linux CI runs `--answers-only` on every revision: it proves the comparison
runs and reports the agreement, without timing a shared runner.

## Limits

One small corpus and one host per recorded run. Process start dominates
short grep runs; GEL loads and verifies its snapshot once per process. The
comparison says nothing about semantic retrieval quality.
