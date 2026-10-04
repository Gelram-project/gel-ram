# Evidence Lab batch mode — schema gel-evidence/1

```text
gel-evidence --batch < commands.txt > results.jsonl
```

Batch mode reads the same commands as the interactive prompt, one per line
(add, replace, drop, clear, list, find, proof, save, load, help, exit). It prints no
prompt and no banner.

- **stdout carries only data:** one JSON object per line (JSON Lines).
- **stderr carries only diagnostics** for a human reader.
- Blank lines are skipped. A command line, including its newline, may be at
  most 4096 bytes and must be UTF-8, as at the interactive prompt.
- The first ERROR stops the batch. Later commands are not executed; they are
  counted as not_run in the summary. `exit` also ends the batch.

## Status and exit code

| Status | Meaning |
|---|---|
| OK | The command completed (add, replace, drop, clear, list, proof, save, load, help, exit). |
| HIT | find matched at least one line and skipped no line. |
| UNKNOWN | find examined every line and matched none. This is a valid answer, not a failure. |
| INCOMPLETE | find skipped at least one over-long line, so a match may be missing. |
| ERROR | The command was refused or failed; the batch stops here. |

| Exit code | Condition |
|---|---|
| 0 | No ERROR and no INCOMPLETE. |
| 3 | At least one INCOMPLETE and no ERROR. |
| 2 | An ERROR, including an unreadable command line or a failed write to stdout. |

## Records

Every record starts with `"schema":"gel-evidence/1"` and `"record"`. Fields
keep the order shown below.

The first record is the session:

```json
{"schema":"gel-evidence/1","record":"session","tool":"gel-evidence","version":"0.5.3"}
```

Each command gives one result record, with `seq` (1-based, counting non-blank
command lines), `command` (the first word of the line) and `status`, followed by
fields that depend on the command:

| Command | Fields after status |
|---|---|
| add | id, bytes |
| replace | id, previous_citations_invalidated |
| drop | id, old_snapshots_remain_on_disk |
| clear | documents (count removed from memory), files_unchanged |
| list | documents: [{id, title, bytes, sha256}] |
| find | documents_examined, matching_lines, shown, skipped_long_lines, search_ns, results |
| proof | document_sha256, collection_sha256, correspondence_not_truth |
| save | revision, save_ns, bundle_sha256 |
| load | revision, load_ns |
| help | help |
| exit | — |
| (ERROR) | error |

Each find result is
`{n, doc, title, start, end, quote, quote_sha256, context: {start, end, omitted_before, omitted_after, text}}`.
start and end are byte offsets in the document. quote is the whole matching
line, and quote_sha256 is the SHA-256 of its exact UTF-8 bytes. JSON escapes
only what it must (quotation mark, backslash and control characters), so every
character of the source survives. The context is bounded and not necessarily a
complete sentence. Timing fields (…_ns) measure only the named operation and
vary between runs.

The last record is the summary:

```json
{"schema":"gel-evidence/1","record":"summary","executed":6,"not_run":0,"statuses":{"OK":3,"HIT":2,"UNKNOWN":1,"INCOMPLETE":0,"ERROR":0},"exit_code":0,"peak_rss_kb":4312}
```

peak_rss_kb is the process's peak resident set size when the summary is written
(Linux VmHWM), or null where it is not measured. It describes this run on this
host, not a property of the format.

## Compatibility

Consumers must ignore fields they do not know. Adding a field keeps
gel-evidence/1. Removing a field, renaming it or changing its meaning requires a
new schema name. The human interactive output is a separate interface and is
not a stable format.

This is phrase retrieval from plaintext sources: a HIT means the phrase occurs
in the retained source, not that the source is true.
