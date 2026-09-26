# Media decode review — 26 September 2026

Local checks of the three existing repository MP4s:

| File | Duration | Resolution | Frame rate | Bytes | Full video decode |
|---|---:|---|---:|---:|---|
| GEL-EVIDENCE-LAB-EN.mp4 | 70.35 s | 1920×1080 | 20 fps | 840031 | PASS |
| GEL-RAM-CONTINUOUS-CHAT-EN-60s.mp4 | 60 s | 1920×1080 | 30 fps | 1786517 | PASS |
| GEL-RAM-HARDWARE-EN-CENTERED-90s.mp4 | 90 s | 1920×1080 | 30 fps | 1394644 | PASS |

All video streams are H264. ffprobe returned metadata; ffmpeg decoded each full
video with error logging and -xerror, exiting successfully with empty error logs.
The files were read, not transcoded or replaced. Their exact hashes remain
pinned in the repository source manifest and reviewed-asset gate.

Recheck an individual file locally:

```sh
ffprobe -v error -show_format -show_streams media/GEL-EVIDENCE-LAB-EN.mp4
ffmpeg -nostdin -v error -xerror -i media/GEL-EVIDENCE-LAB-EN.mp4 -map 0:v -f null -
```

The decode check is automated and says nothing about what the frames show.

## AI review of every visually distinct frame — 26 September 2026

Each film was reduced with `ffmpeg -vf mpdecimate` to its visually distinct
frames, with their timestamps, and every one of those frames was viewed by a
separate AI reviewer (Claude). The reviewer checked privacy, content cuts,
legibility at 1080p and at 640 px width, and agreement with the recorded logs,
guides and transcripts. **This is not a human review.** Changes below the
mpdecimate threshold, such as a blinking cursor, may have been skipped.

| Film | Distinct frames viewed | Privacy | Content cuts | Logs / documentation | 1080p | 640 px |
|---|---:|---|---|---|---|---|
| Evidence Lab, 70.35 s | 161 / 161 | nothing found | none | commands, byte counts, ns values and hashes match both process logs | readable; the pointer hides one hash character at 31–67 s (the same hash is readable elsewhere) | hashes and ns values unreadable |
| Continuous chat, 60 s | 138 / 138 | nothing found; the hardware model is shown on purpose | none | questions, answers, times, sources and RAM/bank values match the guide | readable | borderline |
| Hardware / answers, 90 s | 134 / 134 | nothing found; the hardware model is shown on purpose | none | four cases, times, bank size and RSS match the guide | readable | small; digits need zoom |

Corrections made from this review: event times in the
[transcripts](../media/TRANSCRIPTS-PL-EN.md) now mark when each event first
appears (they were 3–5 s late); the film guide quotes the on-screen
“Session transcript logging ON” and states that the private-preview label is
visible only before `/chat` (0–24.5 s and 0–15.7 s). The films themselves are
unchanged. At 640 px the hash-heavy screens cannot be read in any film; the
transcripts carry those values.

```text
AI_FRAME_REVIEW=COMPLETE_FOR_DISTINCT_FRAMES
HUMAN_VISUAL_REVIEW=NOT_VERIFIED
```

A human viewing of the complete films remains
NOT_VERIFIED. Do not convert decode PASS or the AI frame review into that claim.
Historical films do not show subsequent context-rendering or recorder fixes.
