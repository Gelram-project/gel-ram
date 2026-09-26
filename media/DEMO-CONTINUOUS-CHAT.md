# Continuous English chat — native source summaries

Recorded locally on 2026-09-13. 60 seconds, 1920x1080, 30 fps, silent.
This is a preview of a private application, **not an included public chat feature**.

The whole terminal is centered. Questions follow one another without clearing
the screen. The Rust operator holds each response for about ten seconds before
the next transition, and holds the typed /chat and /exit commands for two seconds
before Enter. Typing is scripted and labelled; application output and timings
are produced live. There are no cuts, answer substitutions or speed changes.
The neutral border is presentation framing, not a physical desktop capture.

## Actual results in this recording

| Prepared question | Response time | Observed result |
|---|---:|---|
| What do we know about Erich Honecker? | 5.107357 ms | Four source-owned biographical fields |
| What do we know about Rami Malek? | 16.973663 ms | Birth fields; death fields left unresolved |
| What do we know about John Lennon? | 23.706502 ms | UNKNOWN: no unambiguous supported summary |

Each successful summary has eight sentences: an introduction, four field results
and three limitations. That is not eight independent facts or a complete biography.
English wording uses fixed grammatical templates around checked source frames.
This route uses exact-title navigation, not a semantic Q8 retrieval benchmark.
It does not demonstrate general reasoning or unrestricted language generation.

John Lennon is present as a catalog subject, but the current route cannot produce
an unambiguous supported summary. UNKNOWN is retained on screen; it does not mean
that the corpus has no information about him. An initial diagnostic expectation
of a successful summary failed; that failed test is retained privately. A subsequent
negative regression test reproduced the unresolved response. This is a disclosed
limitation, not an accuracy improvement or proof of perfect abstention.

The short source identifier displayed for each summary is its anchor. Full field
evidence and byte references are retained in local diagnostic logs, not this bundle.
Source consistency is not independent verification of truth.

## Hardware and timing scope

Owner-reported machine: MINISFORUM AI X1 Pro, 128 GB installed RAM.
OS-reported CPU: AMD Ryzen AI 9 HX 370, 12 cores / 24 logical CPUs.
Linux-visible RAM: 93.91 GiB; available RAM at startup: 19.98 GiB.
Engine RSS: 66.57 MiB initially and 66.96 MiB at final status.
Loaded Q8 bank file: 885,776 bytes; 256 source nodes.

Startup CPU MHz snapshot: min 2931 / mean 3152 / max 3585, sampled about
969 ms after application start. This is not per-answer frequency or proof that
the GPU is idle. The selected computation backend is native CPU; desktop rendering
may use the GPU. The backend runs with an isolated network namespace, no LLM and
no external API.

Latency includes request IPC, native processing, English realization and decoding.
It excludes typing, reading holds and final display rendering. Recording and
other workloads can affect it. The readings are from this run, not reused from
the first movie or selected from multiple runs to claim peak performance.
Differences between the two movies are not a controlled speed comparison.
No chat ORB/s, token throughput, general accuracy, persistent learning, P2P
operation or physical DRAM-refresh synchronization is established here.

The application, private bank, speaker, encoder, executable and recording scripts
remain excluded. The public source does not provide this chat command.
When recorded, publication and licence approval were separate, outstanding
gates; the film itself has since been published under [RIGHTS.md](RIGHTS.md),
which does not license the excluded components above.
