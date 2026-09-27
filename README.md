# GEL RAM

### Evidence Lab

**Your documents. Exact quotes. A restart you can check.**

Load UTF-8 documents, retrieve source-bound passages, reject stale citations and
reopen pinned snapshots. Local Rust tools, without an LLM.

[Quick start](#quick-start) · [See it in action](#see-it-in-action) · [Reproduce](#reproduce-the-checks) · [Documentation](#documentation) · [License](#about-and-licensing)

> **Development toward v0.5.0. Not a tagged release.**
> The instructions below target `work/v0.5.0`, not the default branch. This work
> branch still carries workspace version 0.4.0. Record the exact commit you test.
> [Publication status](CANDIDATE-STATUS.md) · [Tagged releases](https://github.com/Gelram-project/gel-ram/releases) · [Work-branch CI](https://github.com/Gelram-project/gel-ram/actions?query=branch%3Awork%2Fv0.5.0)

## See it in action

**Actual public-tool runs, presented as six edited log replays.**
[Static view](media/gifs/STATIC.md) · [Full gallery and transcripts](media/gifs/README.md) · [Source and hashes](media/gifs/MANIFEST.txt)

Each animation lasts 12 seconds. Card pacing is editorial, not execution time.
Light and dark variants match the README theme; these are not product UI screenshots.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/01-evidence-dark.gif">
  <img alt="Exact quotes. Verified restart." src="media/gifs/01-evidence-light.gif" width="1000">
</picture>

[Read the full source/restart transcript](media/gifs/01-evidence.txt)

<details>
<summary><strong>2. Changed source. Old citation refused.</strong></summary>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/02-stale-dark.gif">
  <img alt="Changed source. Old citation refused." src="media/gifs/02-stale-light.gif" width="1000">
</picture>

[Complete transcript](media/gifs/02-stale.txt) · [Full-size animation](media/gifs/02-stale-light.gif)

</details>

<details>
<summary><strong>3. Backup. Inspect. Restore to a new path.</strong></summary>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/03-backup-dark.gif">
  <img alt="Backup. Inspect. Restore to a new path." src="media/gifs/03-backup-light.gif" width="1000">
</picture>

[Complete transcript](media/gifs/03-backup.txt) · [Full-size animation](media/gifs/03-backup-light.gif)

</details>

<details>
<summary><strong>4. One command. Inspect every result.</strong></summary>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/04-reproduce-dark.gif">
  <img alt="One command. Inspect every result." src="media/gifs/04-reproduce-light.gif" width="1000">
</picture>

[Complete transcript](media/gifs/04-reproduce.txt) · [Full-size animation](media/gifs/04-reproduce-light.gif)

</details>

<details>
<summary><strong>5. One changed byte. Trusted pin rejects it.</strong></summary>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/05-integrity-dark.gif">
  <img alt="One changed byte. Trusted pin rejects it." src="media/gifs/05-integrity-light.gif" width="1000">
</picture>

[Complete transcript](media/gifs/05-integrity.txt) · [Full-size animation](media/gifs/05-integrity-light.gif)

</details>

<details>
<summary><strong>6. GEL and grep. Compare answers first.</strong></summary>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/gifs/06-compare-dark.gif">
  <img alt="GEL and grep. Compare answers first." src="media/gifs/06-compare-light.gif" width="1000">
</picture>

[Complete transcript](media/gifs/06-compare.txt) · [Full-size animation](media/gifs/06-compare-light.gif)

</details>

The earlier [70-second Evidence Lab film](media/GEL-EVIDENCE-LAB-EN.mp4),
its [original process logs](media/EVIDENCE-LAB-GUIDE.md) and
[open human review](docs/MEDIA-DECODE-REVIEW.md) remain separate historical material.
The new replays do not close that review.

### What you can inspect

| Source-bound retrieval | Explicit failure states | Save and reopen |
|:---|:---|:---|
| Original UTF-8 quotations, document identifiers and byte ranges. | UNKNOWN is different from an incomplete search or an error. | Save to a new path and reopen with an independently retained SHA-256 pin. |
| [Readout contract](docs/DOCUMENT-READOUT.md) | [Batch schema and exit codes](docs/EVIDENCE-BATCH.md) | [Collection guide](docs/EVIDENCE-LAB.md) |

A citation check establishes correspondence to the retained source bytes.
**It does not establish that the source is true.** Search is bounded token-phrase
retrieval, not unrestricted question answering. Snapshots and backups contain
plaintext; hash verification is not encryption or a signature.

### Update, restart and reject a corrupted copy

<details>
<summary><strong>Open the recorded final screen and the executable scenario</strong></summary>

[![Recorded final screen of the public update, restart and corrupted-copy scenario. This is one still image, not a film or a new run.](media/evidence-lab/03-update-restart-102s.png)](media/evidence-lab/03-update-restart-102s.png)

[Scenario, commands and checks](docs/UPDATE-RESTART-SCENARIO.md).
The still does not prove that every intermediate step was reviewed.

</details>

[All media and their scope](media/INDEX.md). Historical private previews show a
separate application and are not instructions for running this public checkout.
No generated terminal mockup or illustrative timing is used as execution evidence.

## Quick start

Use a new checkout. Install [Git](https://git-scm.com/) and [rustup](https://rustup.rs)
first. Cloning, toolchain installation and dependency fetching need a network;
the final command uses locked offline dependencies. Run each line separately,
including in Windows PowerShell.

```sh
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout work/v0.5.0
git rev-parse HEAD
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo
```

Expected completion marker: `GEL_EVIDENCE_DEMO=PASS`.
This is an expected result to check, not a claim about a run you have not performed.

For the interactive prompt, run:

```sh
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence
```

Then enter these commands inside Evidence Lab:

```text
add crates/gel-source/fixtures/evidence-lab/memory.txt
find ram is volatile
proof 1
```

For your own UTF-8 files, use `add PATH`, `list`, `find PHRASE` and `proof 1`.
Use `save NEW_PATH`, retain the printed SHA-256 independently, exit, and reopen
with `load SHA256 PATH`. `replace ID PATH` and `drop ID` invalidate earlier
results. Nothing is uploaded or automatically saved by Evidence Lab.

[Full guide and limits](docs/EVIDENCE-LAB.md) · [More examples](docs/TRY-IT.md) ·
[Binary packaging and publication conditions](docs/BINARIES.md)

## Three practical workflows

| Goal | Public interface | Contract and evidence |
|:---|:---|:---|
| Integrate with scripts or CI | `gel-evidence --batch` produces JSON Lines, with separate status and diagnostic channels. | [Versioned schema, exit codes and limits](docs/EVIDENCE-BATCH.md) |
| Keep and recover a collection | `gel-backup` creates and inspects backups, restores to a new path and distinguishes withdrawal from deletion. | [Backup and restore](docs/BACKUP.md) |
| Challenge the parser | Run the finite format-mutation campaign against pinned fixtures. | [Mutation matrix and exclusions](docs/MUTATION-MATRIX.md) |

A missing committed backup manifest is not a complete backup. Restore must not
overwrite an existing snapshot. Directory durability is platform-specific;
`clear`, `drop`, withdrawal and file deletion do not erase all existing copies.
Read the backup contract before using these operations on important data.

## Reproduce the checks

After the quick-start setup, choose a new output directory outside the checkout:

```sh
cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p xtask -- reproduce ../gel-repro-new
```

Expected markers include `GEL_VERIFY_ALL=PASS` and `REPRODUCTION=PASS` within
their declared scope. Read the report for failed, skipped and unmeasured steps;
a final marker is not proof of every feature or an independent second-host result.

Cargo's offline flag is not a system-wide network barrier. For the Linux
namespace procedure and `--require-isolation`, follow the
[reproduction guide](docs/REPRODUCE.md) and [isolation contract](docs/REPRODUCE-ISOLATED.md).
Windows and macOS limitations are recorded rather than counted as Linux checks.

[Per-platform CI evidence](docs/CI-EVIDENCE.md) · [Executable claim registry](docs/CLAIMS.md) ·
[Report an independent reproduction](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml)

Reports and test data stay local unless you share them. Review paths, diagnostics
and source content before publishing a report. Keep slower runs and failures.

## Evidence, not a universal speed claim

| Evidence | What it establishes | Where to inspect it |
|:---|:---|:---|
| Public grep / SHA-256 comparison | A scoped comparison on a declared corpus, including cases where GEL is slower. Matching rules and timed work are not identical. | [Method, disagreement cases and raw results](docs/BENCHMARK-GREP.md) |
| Collection update measurements | Declared mutation-time and memory measurements, with measurement boundaries and exclusions. | [Mutation comparison](docs/MUTATION-COMPARISON.md) |
| F32, F16 and affine Q1 to Q16 reference | Separate byte, numerical-error and ranking checks on public fixtures. Not integration with every reader or private format. | [Precision matrix](docs/PRECISION-MATRIX.md) · [Codec scope](docs/CODEC-SCOPE.md) |
| Historical Ocean measurements | Author-reported CPU/RAM full scans; the research archive is not in this checkout. | [Historical results and limitations](docs/OCEAN-SCALE.md) |
| Author-reported GEL component measurements | Separate tasks whose private implementations cannot be reproduced from this repository. Not an end-to-end speedup over Ocean. | [Method and retained public observations](docs/GEL-EXPERIMENTAL-MEASUREMENTS.md) |

Do not divide timings from different tasks to claim a speedup. Numerical checks
are not semantic recall. Use the [measurement protocol](docs/MEASUREMENT-PROTOCOL.md)
and the [roadmap](docs/ROADMAP.md) to distinguish implemented checks, local
measurements, independent acceptance and open research.

<details>
<summary><strong>Explore the public numerical readout and source formats</strong></summary>

[Public architecture](docs/ARCHITECTURE.md) · [Q8 contract](docs/Q8-QUAD.md) ·
[Source readout](docs/SOURCE-READOUT.md) · [Multipart sources](docs/SOURCE-PARTS.md) ·
[Source building](docs/SOURCE-BUILDER.md) · [Storage format](docs/FORMAT.md)

The source panel and synthetic Q8 panel are separate. Four reversible coordinate
views are not four independent datasets; a Q label alone says nothing about
natural-language quality. The public numerical demonstration is available through:

```sh
cargo run --locked --offline --release -p gel-phase-quad --example quad_playground -- --interactive
```

[Illustrated guide, English / Polish](docs/ILLUSTRATED-GUIDE.md) ·
[Live Lab](docs/LIVE-LAB.md) · [Real-source fixture and provenance](docs/REAL-SOURCE-DEMO.md)

</details>

## Documentation

| Start and use | Inspect and reproduce | Project and history |
|:---|:---|:---|
| [Project map](docs/START-HERE.md) | [Verified results and scope](docs/VERIFIED-RESULTS.md) | [Public roadmap](docs/ROADMAP.md) |
| [Evidence Lab](docs/EVIDENCE-LAB.md) | [Publication fault tests](docs/PUBLICATION-FAULT-TESTS.md) | [Publication status](CANDIDATE-STATUS.md) |
| [Batch interface](docs/EVIDENCE-BATCH.md) | [Source bundle verification](docs/SOURCE-BUNDLE.md) | [Media index](media/INDEX.md) |
| [Backup / restore](docs/BACKUP.md) | [Dependency inventory](docs/DEPENDENCY-INVENTORY.md) | [Earlier README and results](README-HISTORY-R2.md) |

The complete README immediately before this layout change is preserved unchanged
in [README-HISTORY-PRE-VISUAL.md](README-HISTORY-PRE-VISUAL.md), from commit
`ffa85b639e58ad0e70d45f322a375080ccc41258`. It is a historical snapshot, not an
additional current entry point. Existing detailed reports and raw evidence remain
in their original locations.

## About and licensing

GEL RAM is an independent hobby and research project by RR, developed with AI
assistance. This public repository is not the complete private research system.

GEL RAM-owned material uses **GEL RAM Noncommercial Reciprocal License 1.0**.
It is source-available, not an OSI-approved open-source license. Noncommercial
use and non-production Evaluation are governed by that license; Monetized Use
requires a separate signed Commercial Agreement. This README adds no rights.

[LICENSE](LICENSE) · [Licensing guide](LICENSING.md) · [Commercial terms](COMMERCIAL-LICENSE.md) ·
[Third-party notices](THIRD-PARTY-NOTICES.md) · [Media rights](media/RIGHTS.md)

Third-party material retains its own terms, including the MIT-licensed Rust Book
fixture. Historical grants are not rewritten. The historical proposal directory
identified in the licensing guide is not an additional active license.

[Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [CLA privacy](CLA-PRIVACY.md)

Public attribution: **RR, GEL RAM Project**. Contract and CLA enquiries use
`gelram.licensing@gmail.com`; completed agreements and personal records stay private.
