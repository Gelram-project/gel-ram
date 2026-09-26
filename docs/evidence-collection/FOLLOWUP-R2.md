# Follow-up R2 — preparation, not publication

> Historical record of the R2 preparation. Superseded: publication and main
> integration were later approved, PR #9 was merged (71142a2), and native
> Windows/macOS runs for the PR #9 head are pinned in [the platform record](../PLATFORM-REVIEW.md).
> Current status: [publication status](../../CANDIDATE-STATUS.md).

The previous local snapshot and ZIP are preserved. This successor adds:

- [Actual terminal recording and screenshots](../../media/EVIDENCE-LAB-GUIDE.md),
  70.35 seconds, 1920x1080, no audio, real process restart.
- [Post-freeze assessment](../ASSESSMENT-REVIEW.md): 24/24 specified cases,
  20 on documentation and 4 synthetic Unicode cases; **not blind evaluation**.
- [Windows/macOS handoff](../PLATFORM-REVIEW.md), without claiming native runs.
- A Rust typing driver and retained first-run/case evidence. No new dependency
  or collection/terminal runtime change relative to review commit 676e751.

Local Linux full verification: **305 tests passed**, zero failures. The
[log](verify-r2-linux.txt) retains stdout/stderr, replacing only the checkout
path with `<CHECKOUT>`. The original R1 logs retain their historical counts.
The first assessment output similarly redacts only that checkout path.

Main NCRL license, Commercial, CLA and dependency versions remain unchanged.
Media scope wording now distinguishes the new public-candidate demonstration
from the old private previews. The reused Rust Book excerpt remains MIT with
its own notice. Full source/asset bytes are covered by the snapshot manifest.

Status at the time of this record (superseded, see the banner above):
PUBLICATION_APPROVED=NO. WINDOWS_TESTED=NO. MACOS_TESTED=NO.
Independent blind review remains pending; user approval was then pending and was given later. Linux results and a
prepared workflow do not satisfy those external gates. No paid service was used.
