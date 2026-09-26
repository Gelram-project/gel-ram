# Native terminal demonstration — scope and measurement

Recorded locally on 2026-09-13. Duration 90 seconds, 1920×1080, 30 fps, no audio.
This is a private application preview, **not an included public chat feature**.

## Visible execution

- Launch the locally installed `gel-ram` demonstration.
- Show hardware and actual process/system memory information.
- Enter `/chat`, holding the visible command for three seconds before Enter.
- Ask a birthplace question, follow up about the birthdate, change subject,
  then ask an unsupported family relation and receive UNKNOWN.
- Show final memory/turn counters and close the session.

The recorded terminal runs the actual application. Typing is scripted and
labelled; no prerecorded answers replace process output. English sentences are
realized by a bounded native Rust source-frame renderer, with no external LLM.
It is not a demonstration of unrestricted language generation or general AI.

The neutral background and border are presentation framing added around the
complete capture. There are no cuts or speed changes. The original capture,
raw process replies and operator event log are retained privately; they are not
bundled because they contain local paths and source excerpts.

## Hardware and measured results

Machine / installed RAM reported by the owner: MINISFORUM AI X1 Pro, 128 GB.
Linux reported AMD Ryzen AI 9 HX 370, 12 cores / 24 logical CPUs and 93.91 GiB
OS-visible RAM. The startup CPU frequency snapshot was min 605 / mean 1443 /
max 5143 MHz across 24 readings, sampled about 558 ms after application start.
These are OS-reported snapshots, not per-answer frequency measurements.

GEL's computation route uses the CPU, with working data in RAM. The GPU may
render the desktop. Frequency alone is not proof of CPU-only execution.

| Prepared question | Response time | Observed result |
|---|---:|---|
| Where was Rami Malek born? | 1.336424 ms | Source frame: Torrance |
| And when was he born? | 0.232778 ms | Source frame: 12 May 1981 |
| Where did Erich Honecker die? | 0.435760 ms | Source frame: Santiago |
| When did his father die? | 0.139533 ms | UNKNOWN: unsupported relation |

Engine RSS: 66.61 MiB at startup, 67.00 MiB at final status. Q8 bank file:
885,776 bytes, 256 source nodes. These are separate quantities, not claims that
all data and runtime structures fit within the bank-file size.

Response timing includes IPC, engine work, English realization and reply
decoding. Scripted typing and reading pauses are excluded. Screen capture and
other system workloads may affect timings. This is one demonstration run,
not an isolated performance campaign or a statistical accuracy evaluation.

## What this does not establish

No chat ORB/s metric, 99% semantic accuracy, general reasoning, persistent
learning, P2P operation, physical DRAM-refresh synchronization or GPU attestation
is established here. The four scenarios exercise limited source relations.
The film does not grant access to the private engine or expand the public
source's implemented features. When recorded, publication was a separate
approval gate; the film has since been published under [RIGHTS.md](RIGHTS.md).
The private application itself remains unpublished.
