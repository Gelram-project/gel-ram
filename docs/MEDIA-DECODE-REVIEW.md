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

This is an automated decode result, **not** a complete visual or privacy sign-off.
Full viewing, small-screen legibility and timeline-to-log verification remain
NOT_VERIFIED by this review. Do not convert decode PASS into those claims.
Historical films do not show subsequent context-rendering or recorder fixes.
