# Descriptive transcript

The public Evidence Lab film is silent and its hashes are hard to read on a
small screen, so its meaningful commands, answers and limitations are
described below; this is not a verbatim record of every cursor movement. Times
mark when each event first appears on screen; a result stays visible
afterwards. They were read from every visually distinct frame (ffmpeg
mpdecimate) in an AI frame review on 26 September 2026, see
[media review](../docs/MEDIA-DECODE-REVIEW.md). That review is not a human
sign-off. The original video is unchanged.

## 1. Public Evidence Lab — 70.35 s

[Video](GEL-EVIDENCE-LAB-EN.mp4) · [Reproduction and provenance](EVIDENCE-LAB-GUIDE.md)

| At | Description |
|---|---|
| 5.7 s | A centered Linux terminal labels scripted typing, live output, offline CPU execution and phrase retrieval, NOT semantic conversation. The operator starts gel-evidence. |
| 8.2 s, 10.1 s | add memory.txt and add unicode.txt import 169 and 139 bytes. |
| 12.2 s | find ram is volatile returns HIT: “RAM is volatile.” Document 1, bytes 0..16. |
| 20.7 s | proof 1 returns CITATION=PASS with document and collection hashes. |
| 25.1 s | find imaginary evidence returns UNKNOWN. |
| 31.1 s | save checkpoint.gelset saves revision 2 and prints its SHA256 pin. |
| 36.5 s, 39.5 s | exit ends the first process; a new process is started. |
| 45.9 s | load with the retained pin returns REOPEN=PASS, revision 2. |
| 51.3 s, 59.8 s | The same find and proof return the same quote, byte range and hashes. |
| 66.2 s | exit closes the second process. Only explicitly saved snapshots persist. |

The header, the typed `$ gel-evidence` lines, the restart separator and the
closing lines come from the scripted recorder, not from gel-evidence. Its
closing words “citation verified” mean the hash correspondence reported as
CITATION=PASS, not that the quoted content is true.

Exact command sequence and full hashes: [commands](evidence-lab/commands.txt),
[first process](evidence-lab/process-1.txt), [second process](evidence-lab/process-2.txt).

| Operation | Recorded time |
|---|---:|
| First HIT | 0.047389 ms |
| UNKNOWN | 0.045666 ms |
| Save | 12.155075 ms |
| Load | 0.074550 ms |
| HIT after restart | 0.066134 ms |

These are individual application measurements, not percentiles, ORB/s or a
million-record search. Typing, reading pauses and terminal rendering are outside
the timers. The historical film predates the newer bounded-context renderer.
