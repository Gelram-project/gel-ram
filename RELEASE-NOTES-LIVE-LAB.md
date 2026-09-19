# GEL Live Lab — public core integration

Base: `c0058a1ed5df758f2cd118ad15db81845d4cbd46` (document readout, PR #4).
The owner requested completion of this bounded public integration. Publication
is conditional on the final source/privacy checks and exact-revision CI.
No new tag or Release is created; the existing v0.3.0 asset remains unchanged.

## Runnable scope

- Rust terminal Live Lab with authored demo notes and caller-selected UTF-8 files.
- A source-catalog builder and GELSRC01 plaintext envelope, not the private ORB encoder.
- No-replace publication, retained independent SHA256 pin and verified fresh-process reopen.
- Whole-source phrase search across storage parts, exact original byte ranges,
  selectable matching lines and explicit UNKNOWN/INCOMPLETE states.
- Separate synthetic Q8 controls: phase, mask, noise and four checked inverse views.
- Corruption, truncation, concurrent-write, Unicode and subprocess regression tests.
- Updated roadmap, historical-license labels and unambiguous audit-status wording.

Start: `cargo run --locked --offline --release -p gel-live-lab`.
Use `--demo` for scripted runtime checks or `--plain` for a scrolling transcript.
[Commands, limits and persistence](docs/LIVE-LAB.md).

## Verification

Run `cargo run --locked --offline -p xtask -- verify` for the full local gate,
or `cargo run --locked --offline -p xtask -- report ../new-report` for a retained
local report. Initial toolchain/dependency preparation needs network; runtime
and subsequent cached builds do not. Never publish your report without checking
paths and any inputs you chose to import.

Final test counts and the exact revision belong to [GitHub Actions](https://github.com/Gelram-project/gel-ram/actions).
Historical source and Ocean measurements are not timing results of this UI.
No semantic accuracy, speedup, full-Ocean conversation or physical DRAM-refresh
synchronization is claimed by the new package.

## Unchanged/excluded

Active root NCRL1.0, CLA and commercial terms, external dependency versions,
existing films, Ocean research archives and private implementation remain
unchanged. Historical licensing-proposal notices do not activate that proposal.
No private speaker, personal corpus, keys, conversation, encrypted vault,
network daemon, telemetry or paid service is added. Do not use plaintext source
bundles as a private encrypted memory store. Parent directories must be trusted;
portable path checks do not defend against hostile concurrent path replacement.
