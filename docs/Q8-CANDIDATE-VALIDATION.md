# Q8 candidate validation and publication boundary

2026-09-11. Local Linux verification, Rust1.85.0. No remote CI or release
has been triggered for this candidate. Base main:339f3649684683da3d8ee54d6fdcc1c624bdfabe.

Revision2 adds input hardening and a complete repeat campaign; see the
[R2 audit](Q8-CANDIDATE-R2-AUDIT.md). The counts below describe the initial
campaign; R2 increases workspace test executions to145 per profile.

## Initial completed local checks

- Full `xtask verify`: PASS, including format, Clippy with warnings denied,
  documentation, release build, selftest, existing benchmarks and the new demo.
- 142 test executions in debug and142 in release; all passed. Counts include
  repeated reference tests in different harnesses, not142 independent features.
- Four new checked-view tests, including deliberate relabelling to document
  the lack of authentication; four new fixture/baseline tests.
- Independent integrity audit:262400/262400 structural cases and256/256
  ranking queries; `GEL_DATA_INTEGRITY_ALL=PASS`.
- Final existing V2 campaign:48 runs, all8,355,840 view comparisons passed.
- New canonical-baseline campaign:24 runs, all2,509,056 view comparisons passed.
- Benchmark source and binary hashes were checked after the final campaign.
- cargo-audit0.22.2: no known vulnerability findings with the local advisory
  database commit b50980aad8b8f14f77e25a97b32dd94bf008b0af. No database fetch
  and no yanked-package check were performed (`--no-fetch --no-yanked`).
  This is not a statement about advisories published after that snapshot.

## Scope review

Reviewed the new descriptor API, numeric parser/generator, benchmark and Rust
campaign helpers. No dynamic loading, runtime network call, LLM invocation,
executable media or private encoder is introduced. The campaign helper runs
the explicitly supplied local benchmark binaries. The example reads a bounded
caller-selected file and only creates new fixture files; it never interprets
their bytes as code. It does not provide adversarial filesystem isolation.

Cargo.lock and dependencies are unchanged from the inspected public base.
No donor source or private licensing file was copied into this candidate.
Existing module low-level APIs remain available; checked views are opt-in.
Compatibility labels do not protect against deliberately false metadata.
Input caps do not guarantee immunity to resource exhaustion or allocation failure.

Only source, public historical evidence, numerical current logs and documentation
belong in the archive. Private fixtures, benchmark executables, build directories,
local transcripts, TLS credentials and Git metadata are excluded. The final
delivery audit separately records the file allow-list, checksum verification,
secret-scan result and clean-unpack verification. No audit guarantees absence
of all malicious code or vulnerabilities.

## Memory observation

Separate, untimed-campaign Linux `/usr/bin/time -v` probes on the final binary:
768 real records:10,488 KiB maximum RSS;8192 synthetic records:86,220 KiB.
Both had exit0. These are single process high-water observations, not memory
requirements for an optimized deployed reader. The diagnostic deliberately
holds both canonical and four-view banks plus input bytes and output buffers.
No GPU, PUF, physical refresh measurement, energy or sustained full-CPU result.

## Remaining release decisions

Only Linux execution is established here. Existing main's Windows/macOS jobs
compile code; they do not qualify this unpublished candidate or its runtime.
The maintainer must review the selected export, run new CI if publishing a PR,
and decide whether to tag a release. The private real fixture is not included,
so that exact external-data experiment is not publicly reproducible.
Semantic accuracy and readiness of the complete private system remain outside this package.
