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

## Recorded run r1

[Evidence](evidence-bench-r1/summary.txt): revision dc38282 with a clean
working tree, GNU grep 3.11, GNU coreutils sha256sum 9.4, locale C.UTF-8,
network isolation verified, 69 corpus files, load average 2.8 at the start and
3.1 at the end on the owner's AMD Ryzen AI 9 HX 370 with background desktop
processes. The manifest, the answer table and all 1,452 samples (1,320 measured,
132 warm-up) are in
[evidence-bench-r1](evidence-bench-r1/manifest.txt).

Answers: 20 of 36 queries gave the same answer. 15 differ because GEL splits
words at punctuation and underscores (for example "rust 1 85 0", "sha 256",
"gel evidence", "source" in source_find); 1 differs by Unicode normalization
(a decomposed "café"). Polish and Greek queries, including final sigma, gave
the same answer.

| Phase (20 same-answer queries) | GEL p50 / p95 | Baseline p50 / p95 | GEL ÷ baseline (p50) |
|---|---|---|---|
| process-workload | 72.7 / 88.2 ms | grep: 62.8 / 72.9 ms | 1.16 |
| integrity-process | 3.13 / 3.43 ms | sha256sum: 1.95 / 2.38 ms | 1.61 |
| in-memory-search, per query | 3.32 / 4.26 ms | — | — |

On this corpus and host GEL is slower than the baseline in both compared
phases: its whole-process run of the query set takes about 16% longer than 20
grep processes, and loading a pinned snapshot, which also parses and checks its
structure, takes about 61% longer than hashing the file. The in-memory search
time is most of GEL's workload time. The GEL process's peak resident set size
was 2.8–2.9 MB in every run. These numbers describe this small corpus on this
host only; they establish no speed advantage.

## CI

Linux CI runs `--answers-only` on every revision: it proves the comparison
runs and reports the agreement, without timing a shared runner.

## Limits

One small corpus and one host per recorded run. Process start dominates
short grep runs; GEL loads and verifies its snapshot once per process. The
comparison says nothing about semantic retrieval quality.
