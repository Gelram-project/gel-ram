// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Generate a script-free README presentation. Existing recordings are never rewritten.
#![forbid(unsafe_code)]
use std::{fs, io::Write, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// The release whose sources the page links to and checks out.
const RELEASE: &str = "v0.7.0";
const SCENES: &[(&str, &str, &str)] = &[
    (
        "01-evidence",
        "Exact quotes. A verifiable restart.",
        "Add a source, inspect its citation, save and reopen it in a separate process.",
    ),
    (
        "02-stale",
        "Changed source. Old citation refused.",
        "Replace the source and inspect the refusal of the previous result.",
    ),
    (
        "03-backup",
        "Backup. Inspect. Restore.",
        "Restore into a new path and check equality with the saved snapshot.",
    ),
    (
        "04-reproduce",
        "One command. Inspect every result.",
        "Read the recorded strict reproduction result and its declared scope.",
    ),
    (
        "05-integrity",
        "Changed bytes. Retained pin. Refusal.",
        "A changed-byte copy is rejected against the original trusted hash.",
    ),
    (
        "07-literal",
        "Your bytes. One record. Four views back.",
        "Place a file's first bytes in one record, restore its four views and reject a changed byte.",
    ),
];
/// Four checked facts; the counts are read from the repository by `fact_counts`.
const FACTS: &str = r####"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="128" viewBox="0 0 1200 128" role="img" aria-labelledby="t d">
<title id="t">Checked facts about this release</title>
<desc id="d">__PROPS__ documented properties mapped to __TESTS__ tests, checked on every CI platform with declared Unix-only exclusions; 3 CI platforms; __MUTANTS__ format mutants each rejected or explained; network isolation verified in the strict reproduction.</desc>
<style>
text{font-family:Arial,Helvetica,sans-serif}
.num{font-size:36px;font-weight:800}
.lab{font-size:14px;fill:__MUTED__}
.tile{fill:__PAPER__;stroke:__LINE__;stroke-width:2}
.t1,.t2,.t3,.t4{animation:rise 1.2s ease-out both;transform-box:fill-box}
.t2{animation-delay:.25s}.t3{animation-delay:.5s}.t4{animation-delay:.75s}
@keyframes rise{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:none}}
@media (prefers-reduced-motion:reduce){.t1,.t2,.t3,.t4{animation:none}}
</style>
<g class="t1"><rect class="tile" x="2" y="2" width="288" height="124" rx="16"/><rect x="2" y="2" width="8" height="124" rx="4" fill="__C1__"/>
<text class="num" x="28" y="54" fill="__C1__">__PROPS__ → __TESTS__</text><text class="lab" x="28" y="82">properties mapped to tests, checked</text><text class="lab" x="28" y="102">per CI platform, Unix-only declared</text></g>
<g class="t2"><rect class="tile" x="306" y="2" width="288" height="124" rx="16"/><rect x="306" y="2" width="8" height="124" rx="4" fill="__C2__"/>
<text class="num" x="332" y="54" fill="__C2__">3</text><text class="lab" x="332" y="82">CI platforms:</text><text class="lab" x="332" y="102">Linux · macOS · Windows</text></g>
<g class="t3"><rect class="tile" x="610" y="2" width="288" height="124" rx="16"/><rect x="610" y="2" width="8" height="124" rx="4" fill="__C3__"/>
<text class="num" x="636" y="54" fill="__C3__">__MUTANTS__</text><text class="lab" x="636" y="82">format mutants, each rejected</text><text class="lab" x="636" y="102">or its acceptance explained</text></g>
<g class="t4"><rect class="tile" x="914" y="2" width="284" height="124" rx="16"/><rect x="914" y="2" width="8" height="124" rx="4" fill="__C4__"/>
<text class="num" x="940" y="54" fill="__C4__">offline</text><text class="lab" x="940" y="82">network isolation verified</text><text class="lab" x="940" y="102">in the strict reproduction</text></g>
</svg>
"####;
/// One colour-coded badge per scene, in SCENES order: label, icon path, light
/// colour, dark colour.
const CHIPS: &[(&str, &str, &str, &str)] = &[
    (
        "01 · CITE AND RESTART",
        "M4 13c0-4 2-6 5-7v3c-1 1-2 2-2 4h2v5H4zM13 13c0-4 2-6 5-7v3c-1 1-2 2-2 4h2v5h-5z",
        "#0969da",
        "#58a6ff",
    ),
    (
        "02 · STALE CITATION REFUSED",
        "M12 3a9 9 0 1 0 .01 0zM5.6 5.6l12.8 12.8",
        "#bc4c00",
        "#f0883e",
    ),
    (
        "03 · BACKUP AND RESTORE",
        "M3 5h18v4H3zM5 9h14v11H5zM10 13h4",
        "#1a7f37",
        "#3fb950",
    ),
    (
        "04 · STRICT REPRODUCTION",
        "M3 4h18v16H3zM7 9l3 3-3 3M12 15h5",
        "#8250df",
        "#bc8cff",
    ),
    (
        "05 · CHANGED BYTE REFUSED",
        "M12 2l8 3v6c0 5-4 8-8 10-4-2-8-5-8-10V5zM9 11l6 6M15 11l-6 6",
        "#cf222e",
        "#ff7b72",
    ),
    (
        "06 · YOUR BYTES, ONE RECORD",
        "M5 3h14v18H5zM8 7h8M8 11h8M8 15h5",
        "#0b7285",
        "#39c5cf",
    ),
];
const HTML: &str = r####"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; object-src 'none'">
<title>GEL RAM | Multimedia guide</title>
<style>
:root{color-scheme:light dark;--bg:#f6f9ff;--paper:#fff;--ink:#102949;--muted:#485e77;--line:#cbdcf0;--blue:#0969da;--soft:#e9f3ff;--shadow:0 18px 48px #24497112}
*{box-sizing:border-box}body{margin:0;background:radial-gradient(ellipse at 95% 0%,#d8eaff 0,transparent 55%),var(--bg);color:var(--ink);font:16px/1.6 system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
a{color:var(--blue);text-underline-offset:3px}a:hover{text-decoration-thickness:2px}a:focus-visible,label:focus-visible{outline:3px solid var(--blue);outline-offset:5px}img{max-width:100%;height:auto;display:block}h1,h2,h3,p{margin-top:0}h1{font-size:clamp(34px,4.3vw,57px);letter-spacing:-.05em;line-height:1.07;margin-bottom:22px}h2{font-size:29px;letter-spacing:-.025em;line-height:1.2}h3{font-size:18px;line-height:1.35}code,pre{font:13px/1.6 ui-monospace,SFMono-Regular,Consolas,monospace}pre{white-space:pre-wrap;overflow-wrap:anywhere;border:1px solid var(--line);background:var(--soft);border-radius:12px;padding:20px}code{overflow-wrap:anywhere}.page{max-width:1220px;padding:32px 32px 0;margin:auto}.brand-row{display:flex;justify-content:space-between;gap:28px;align-items:center}.brand{display:flex;align-items:center;gap:18px}.brand>.motion,.brand>.poster{flex:none}.brand img{width:84px}.brand-name{font-size:33px;letter-spacing:-.03em;line-height:1.12;font-weight:780}.brand-name small{display:block;font-size:23px;letter-spacing:0}.strap{margin:10px 0 0;font-size:14px;color:var(--muted)}.eyebrow{text-transform:uppercase;letter-spacing:.14em;font-size:12px;font-weight:750;color:var(--blue);margin-bottom:14px}.header-right{text-align:right;max-width:355px}.header-right p{font-size:17px;margin:0 0 10px}.chips{display:flex;gap:7px;flex-wrap:wrap;justify-content:flex-end}.chip{border:1px solid var(--line);padding:3px 10px;border-radius:6px;font-size:12px;background:var(--paper);color:var(--muted)}nav{display:flex;gap:26px;flex-wrap:wrap;padding:18px 0;margin:23px 0;border-top:1px solid var(--line);border-bottom:1px solid var(--line);font-weight:600;font-size:14px}nav a{text-decoration:none}.hero{display:grid;grid-template-columns:1.18fr 1fr;gap:36px;align-items:center;padding:30px;background:linear-gradient(135deg,var(--soft),var(--paper));border:1px solid var(--line);border-radius:20px;box-shadow:var(--shadow)}.hero>*{min-width:0}.hero figure{margin:0}.media-link{display:block;border:1px solid var(--line);border-radius:10px;overflow:hidden;background:var(--paper);text-decoration:none;box-shadow:var(--shadow)}.hero figcaption{font-size:12px;color:var(--muted);margin-top:13px}.hero-lead{font-size:17px}.checks{list-style:none;padding:0;margin:20px 0}.checks li{margin:8px 0;font-size:14px}.checks li::before{content:'✓';font-weight:800;color:var(--blue);margin-right:10px}.buttons{display:flex;flex-wrap:wrap;gap:10px}.button{display:inline-block;background:var(--blue);color:#fff;border:1px solid var(--blue);border-radius:10px;padding:10px 17px;text-decoration:none;font-weight:650;font-size:14px}.button.secondary{background:var(--paper);color:var(--blue);border-color:var(--line)}.boundary{margin:19px 0 0;color:var(--muted);font-size:12px}.section{margin:44px 0}.section-heading{text-align:center;max-width:760px;margin:0 auto 24px}.section-heading p{color:var(--muted);font-size:14px;margin-bottom:0}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:19px}.card{padding:17px;background:var(--paper);border:1px solid var(--line);border-radius:14px;box-shadow:var(--shadow)}.card .num{font-size:11px;letter-spacing:.14em;color:var(--blue);font-weight:750;margin-bottom:8px}.card h3{min-height:49px;margin-bottom:12px}.card p{color:var(--muted);font-size:13px;margin:14px 0 12px}.card .links{font-size:12px;display:flex;gap:12px;flex-wrap:wrap}.trust{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:20px;border-block:1px solid var(--line);padding:24px 0;margin-top:34px}.trust strong{display:block;margin-bottom:5px;font-size:16px}.trust p{font-size:13px;color:var(--muted);margin:0}.columns{display:grid;grid-template-columns:1.2fr 1fr 1fr;gap:30px}.columns>section{min-width:0}.columns h2{font-size:23px}.columns p,.columns li{font-size:14px;color:var(--muted)}.doclinks{padding:0;list-style:none}.doclinks li{margin:9px 0}.scope{background:var(--soft);border:1px solid var(--line);border-radius:14px;padding:22px;font-size:13px}.scope h2{font-size:20px}.scope p:last-child{margin-bottom:0}footer{display:flex;justify-content:space-between;gap:24px;flex-wrap:wrap;padding:25px 0;border-top:1px solid var(--line);font-size:12px;color:var(--muted)}footer p{margin:0}.poster{display:none}.motion-control{position:absolute;left:-9999px}.view-tools{display:flex;gap:20px;justify-content:flex-end;align-items:center;font-size:12px;margin:0 0 18px;color:var(--muted)}.view-tools label{cursor:pointer;border:1px solid var(--line);border-radius:7px;background:var(--paper);padding:6px 10px}.motion-control:focus-visible~.page label[for=still-mode]{outline:3px solid var(--blue)}.motion-control:checked~.page .motion{display:none}.motion-control:checked~.page .poster{display:block}.motion-control:checked~.page label[for=still-mode]{background:var(--soft);outline:2px solid var(--blue)}.skip{position:absolute;left:12px;top:-80px;padding:8px 12px;background:var(--paper);z-index:5}.skip:focus{top:10px}
@media(prefers-color-scheme:dark){:root{--bg:#0d1117;--paper:#151b23;--ink:#e6edf3;--muted:#a9b9ce;--line:#303d50;--blue:#79c0ff;--soft:#16253a;--shadow:0 18px 45px #0003}body{background:radial-gradient(ellipse at 95% 0%,#1a3555 0,transparent 55%),var(--bg)}.button{color:#081322}.button.secondary{color:var(--blue)}}
@media(max-width:920px){.hero{grid-template-columns:1fr;gap:24px}.hero figure{order:2}.hero h1{max-width:680px}.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.columns{grid-template-columns:1fr 1fr}.columns>section:first-child{grid-column:1/-1}.header-right{max-width:290px}.brand-name{font-size:28px}.brand-name small{font-size:20px}.page{padding:24px 22px 0}}
@media(max-width:580px){.page{padding:20px 16px 0}.brand-row{display:block}.header-right{text-align:left;max-width:none;margin-top:18px}.header-right p{font-size:15px}.chips{justify-content:flex-start}.hero{padding:19px}.hero h1{font-size:38px}.grid,.columns{grid-template-columns:1fr}.card h3{min-height:0}.trust{grid-template-columns:1fr 1fr}.view-tools{justify-content:flex-start;flex-wrap:wrap;gap:10px}nav{gap:16px;font-size:13px}.section{margin:32px 0}}
.facts{margin-top:18px}.card .num img{width:auto;max-width:100%}
@media print{.motion{display:none!important}.poster{display:block!important}body{background:white;color:black}.view-tools{display:none}.hero,.card{box-shadow:none}}
</style>
</head>
<body>
<a class="skip" href="#content">Skip to content</a>
<input id="still-mode" class="motion-control" type="checkbox" aria-label="Show still images instead of moving previews">
<div class="page">
<header><div class="brand-row"><div><div class="brand">
__BRAND_LOGO__
<div class="brand-name">GEL RAM<small>Public tools</small></div></div><p class="strap">Knowledge printed, not trained. Exact bytes, or a refusal.</p></div>
<div class="header-right"><p>Public tools for local work<br>and independent inspection.</p><div class="chips"><span class="chip">Rust 1.85</span><span class="chip">No third-party packages</span><span class="chip">Offline</span></div></div></div>
<nav aria-label="Page navigation"><a href="#content">Overview</a><a href="#quick-start">Quick start</a><a href="#workflows">Six workflows</a><a href="#checks">Checks</a><a href="#reproduce">Reproduce</a><a href="#documentation">Documentation</a><a href="#scope">Scope &amp; license</a></nav></header>
<div class="view-tools"><label for="still-mode">Show still images</label><a href="__DOC__/media/gifs/MANIFEST.txt">Media provenance</a><span>12-second edited replays, not wall time</span></div>
<main id="content">
<section class="hero" aria-labelledby="hero-heading">
<figure><a class="media-link" href="media/gifs/01-evidence-light.gif" aria-label="Open the exact quotes and restart animation at full size">__HERO__</a><figcaption>Real public CLI output, edited into three readable cards.<br><a href="media/gifs/01-evidence.txt">Full transcript and source revision</a> · <a href="media/gifs/01-evidence-dark.gif">Dark full-size version</a></figcaption></figure>
<div><p class="eyebrow">Check it yourself</p><h1 id="hero-heading">Exact bytes,<br>or a refusal.</h1><p class="hero-lead">GEL RAM prints knowledge into memory as fixed Q8 records and reads them back exactly. The printing step is private; the public tools in this checkout let you check exact read-back, the refusal of changed bytes and crash safety on your own files and on synthetic records.</p><ul class="checks"><li>One record, four views, read back bit for bit</li><li>A changed byte refused under the SHA-256 you kept</li><li>Nothing confirmed lost when the process is killed</li><li>No third-party packages; runs offline on your CPU</li></ul><div class="buttons"><a class="button" href="#quick-start">Run it yourself →</a><a class="button secondary" href="#workflows">Explore six workflows</a></div><p class="boundary">The text AI is a goal, not a result. A process kill is not a power cut. A matching hash does not show that a source is true.</p></div>
</section>
<section class="section flow" aria-label="How a citation is checked">__FLOW__<div class="facts">__FACTS__</div></section>
<section class="section" id="workflows" aria-labelledby="workflow-heading"><div class="section-heading"><p class="eyebrow">Six workflows. Their original evidence.</p><h2 id="workflow-heading">See it in action</h2><p>These are edited replays of recorded public command output, not product screenshots or new benchmark runs. Open each image at full size to read the terminal text.</p></div><div class="grid">__CARDS__</div>
<div class="trust"><div><strong>Inspectable</strong><p>Transcripts and retained hashes accompany the previews.</p></div><div><strong>Explicit failures</strong><p>Inspect refusal and incomplete states, not only success.</p></div><div><strong>Local operation</strong><p>The CLI uses your local documents. No automatic upload.</p></div><div><strong>Reproducible scope</strong><p>Commands, sources and limitations remain linked.</p></div></div></section>
<section class="section" id="checks" aria-labelledby="checks-heading"><div class="section-heading"><p class="eyebrow">Drawn from this checkout</p><h2 id="checks-heading">What the public checks cover</h2><p>The property map, the CI exclusion list and the committed mutation report are read when the images are built; the read-only presentation check fails if they drift.</p></div>__WALL__<div class="facts">__BARS__</div></section>
<div class="columns section"><section id="quick-start"><p class="eyebrow">Try the public tool</p><h2>Start with your own source.</h2><p>In a new checkout, run each line separately. Setup needs a network; after that every command runs offline.</p><pre><code>git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git rev-parse HEAD
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab -- --literal README.md
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo</code></pre><p>Expected markers: <code>GEL_LIVE_LAB_LITERAL=PASS</code> and <code>GEL_EVIDENCE_DEMO=PASS</code>. These are results to check, not pre-approved outcomes.</p></section>
<section id="reproduce"><p class="eyebrow">Check the declared scope</p><h2>Reproduce the checks.</h2><pre><code>cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p xtask -- reproduce ../gel-repro-new</code></pre><p>Use a new output directory outside the checkout. Read failed, skipped and unmeasured steps. Cargo offline mode is not system-wide network isolation.</p><ul class="doclinks"><li><a href="__DOC__/docs/REPRODUCE.md">Reproduction instructions</a></li><li><a href="__DOC__/docs/CI-EVIDENCE.md">Per-revision CI evidence</a></li><li><a href="__DOC__/docs/CLAIMS.md">Executable claim registry</a></li></ul></section>
<section id="documentation"><p class="eyebrow">Go beyond the preview</p><h2>Documentation.</h2><ul class="doclinks"><li><a href="__DOC__/README.md">README</a></li><li><a href="__DOC__/docs/EVIDENCE-LAB.md">Evidence Lab guide</a></li><li><a href="__DOC__/docs/LIVE-LAB.md">Your bytes in one record</a></li><li><a href="__DOC__/docs/Q8-QUAD.md">Q8 record contract</a></li><li><a href="__DOC__/docs/CLAIMS.md">Claim registry</a></li><li><a href="__DOC__/docs/COMMITMENTS.md">Published commitments</a></li><li><a href="__DOC__/docs/LICENSE-DECISION.md">License decision guide</a></li><li><a href="__DOC__/docs/ROADMAP.md">Public roadmap</a></li><li><a href="__DOC__/media/INDEX.md">Films and their scope</a></li></ul><p>Documentation links open the files of this checkout. Viewing this local page loads only adjacent files, with no scripts, trackers or remote fonts.</p></section></div>
<section class="scope section" id="scope"><h2>What this page does and does not establish</h2><p>This is a documentation presentation, not the application interface. It describes this checkout; tagged releases are listed on the repository's Releases page. The older GIFs retain the source revision declared in their <a href="media/gifs/MANIFEST.txt">original manifest</a>. Presentation changes do not retime them, certify a release or close outstanding human review.</p><p>Snapshots and backups are plaintext. A hash is not encryption, a signature or proof that a source is true. Different search tasks and matching semantics are not interchangeable benchmarks.</p><p>GEL-owned material remains under <a href="__DOC__/LICENSE">GEL RAM Noncommercial Reciprocal License 1.0</a>. Read the <a href="__DOC__/LICENSING.md">licensing guide</a>, <a href="__DOC__/THIRD-PARTY-NOTICES.md">third-party notices</a> and <a href="__DOC__/media/RIGHTS.md">media rights</a>. This page grants no additional rights.</p></section>
</main>
<footer><p><strong>GEL RAM</strong><br>RR — GEL RAM Project · Public documentation</p><p><a href="__DOC__/LICENSE">License</a> · <a href="__DOC__/SECURITY.md">Security</a> · <a href="__DOC__/CLA-PRIVACY.md">CLA privacy</a> · <a href="__DOC__/CONTRIBUTING.md">Contributing</a><br>Existing evidence preserved. New presentation only.</p></footer>
</div></body></html>
"####;
const NATIVE: &str = r####"# GEL RAM

<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->
<picture>
  <source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/presentation/header-still-dark.svg">
  <source media="(prefers-reduced-motion: reduce)" srcset="media/presentation/header-still-light.svg">
  <source media="(prefers-color-scheme: dark)" srcset="media/presentation/header-dark.svg">
  <img alt="GEL RAM. Knowledge printed, not trained. One stored record, four exact views, 1,152 bytes. Read back exactly; it can say UNKNOWN; a process kill, not a power cut, loses nothing it confirmed. Logo: a glass cube of record cells around an accent core." src="media/presentation/header-light.svg" width="1200">
</picture>

**GEL RAM prints knowledge into memory as fixed Q8 records, instead of training
it into model weights, and reads the records back exactly, working toward
hardware-level memory computation.** The goal is a text AI that holds a free
conversation in Polish and English: it answers in sentences of its own, not
pasted excerpts, from what it holds, shows the source it relied on, and says it
does not know when it holds nothing on the question.

| Status | What it covers |
|:---|:---|
| **You can check it here** | One record read back exactly through four views; a changed byte refused; a killed process losing nothing it confirmed. One command each, [below](#check-it-yourself). |
| **Private** | The step that prints text into records, and the private build that answers from them; both exist only on the author's machine. [__COMMITMENTS__ published commitments](docs/COMMITMENTS.md) fix the exact contents of private source trees, data sets and a sealed sample as of their dates; which holds what is not published. |
| **Not established** | The text AI is a goal, not a result ([the goal](docs/GOAL.md)). Hardware-level memory computation and power-cut durability of the record history are `NOT_ESTABLISHED` ([claim registry](docs/CLAIMS.md)). |

## Check it yourself

With [Git](https://git-scm.com/) and [rustup](https://rustup.rs) installed, run each line separately.
Only `git clone` and `rustup` need a network; no model and no private data are involved.
No independent run on a second machine has been reported yet: [post yours in issue #20](https://github.com/Gelram-project/gel-ram/issues/20).

```sh
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab -- --literal README.md
cargo run --locked --offline --release -p gel-phase-quad --example quad_compare
cargo run --locked --offline -p xtask -- mutation-matrix
cargo run --locked --offline -p xtask -- crash-series 20
cargo run --locked --offline -p xtask -- verify
```

| Command | Line to look for | What it shows | Its limit |
|:---|:---|:---|:---|
| `--literal` | `ROUNDTRIP=4/4 DIFFERENT_BITS=0`, `TAMPER=REJECTED` | The first 1,024 bytes of any file you name, placed in one record; all four views give them back bit for bit, and a copy with one changed byte is refused | Your bytes as they are, not knowledge printing: no meaning and no search |
| `quad_compare` | `Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED` | 512 synthetic records; each view's score equals the score of a separate reference copy, bit for bit | Meaning is not measured; one plain copy can still be read faster than the shared record |
| `mutation-matrix` | `MUTATION_MATRIX=PASS mutants=179` | Each changed copy in the matrix, over the three public file formats, is refused under the original SHA-256 | A finite set of changes, not every possible one |
| `crash-series 20` | `CRASH_SERIES=PASS` with `acknowledged_lost=0` | Evidence Lab is killed 20 times at random while it writes; nothing it confirmed is lost | Recorded on Linux; `verify` runs it on Unix hosts only; a process kill, not a power cut |
| `verify` | `GEL_VERIFY_ALL=PASS` | The workspace tests and public gates, with short forms of the runs above | Crash series on Unix hosts only; the private build is not in this checkout |

## One record, four exact views

__RECORD__

A record is stored once, in 1,152 bytes. A view reads the same values in another
fixed arrangement; each of the four views scores exactly as a separate, fully
written-out copy would, bit for bit, and four such copies would take 4,608 bytes.
Four views are not four independent memories, four votes or a speedup.
[Contract](docs/Q8-QUAD.md) · [the recorded runs](docs/evidence-q8-current/README.md)

## Exact bytes, or a refusal

__FLOW__

Evidence Lab (`gel-evidence`), the public collection tool, works on your own
UTF-8 files: each quotation comes back byte for byte with its byte range, a
replaced document's old citation is refused, and a saved snapshot with one
changed byte fails to load under the SHA-256 you kept. A matching hash shows
these are your bytes, not that the source is true or private: snapshots and
backups hold plain text ([backup contract](docs/BACKUP.md)). It is phrase lookup,
not question answering. Its demo,
`cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo`,
ends with `GEL_EVIDENCE_DEMO=PASS`. [Evidence Lab guide](docs/EVIDENCE-LAB.md) · [a changed byte refused, as text](media/gifs/__HERO_ID__.txt)

## A killed process loses nothing it confirmed

__CRASH__

In the recorded series Evidence Lab was killed at random moments in 200 trials:
0 of 2,683 acknowledged snapshots were lost, none was partial, and every trial
resumed to the uninterrupted result ([crash series](docs/CRASH-SERIES.md)).
Write failures, denied permissions and a full disk leave the previous snapshot
intact ([fault tests](docs/PUBLICATION-FAULT-TESTS.md)).

## The public checks, drawn from their files

__BARS__

__WALL__

Both images are drawn from the committed mutation report and the property map when the README is built; the presentation check fails if either file changes without a rebuild. The map shows the rule CI enforces, not the outcome of one run ([CI evidence](docs/CI-EVIDENCE.md)).

## New in 0.7, and on main

- **No third-party packages:** every one of the __PACKAGES__ packages in `Cargo.lock` is a crate of this checkout. SHA-256 and Unicode normalization are written here; SHA-256 alone runs at 0.28 and 0.39 of the speed of v0.6.1, which used the processor's SHA instructions.
- **One Unicode version:** normalization follows Unicode 16.0.0, the version the pinned Rust 1.85.0 standard library uses for lowercasing and word splitting, and passes every line of the Unicode normalization test file.
- **The same output bits, faster:** each speed change was timed against two copies of the baseline, and every run printed the same output digest, apart from the whole-tree gates, which count the files of their own tree ([release notes](RELEASE-NOTES-v0.7.0.md)).

<details><summary><strong>Every timed case against v0.6.1, as an animated chart</strong> (tall; opens here)</summary>

__SPEED__

</details>

- **On `main` since v0.7.0, not yet released:** the eight tool programs answer `--version` and `--license` (the `xtask` runner does not); every GEL RAM-owned Rust source file except the generated Unicode tables begins with a two-line license header; `gel-license check DIR` reports whether a copy kept the license, the Required Notice (the attribution line in NOTICE), those headers and the Unicode notice, not that every notice was kept.

## Using GEL RAM

GEL RAM uses the GEL RAM Noncommercial Reciprocal License 1.0: source-available,
not an OSI-approved open-source license. You may use, study, change and share it
for noncommercial purposes, keeping the LICENSE, the copyright notices and every
`Required Notice:` line with each copy; private noncommercial changes may stay
private. Anyone, including a company, may evaluate it outside production, solely
to assess it, with no fixed time limit. Any use that directly or indirectly
earns, enables, supports or is meant to produce money or other material financial
benefit, for you or anyone else, needs a Commercial Agreement signed by the
Licensor before it begins: `gelram.licensing@gmail.com`. `gel-license use ANSWERS_FILE`
shows which LICENSE sentences apply to your answers about one use; it is not
legal advice. Code contributions for merge need the signed [CLA 2.0](CLA.md).
This README adds no rights; the [LICENSE](LICENSE) controls
([licensing guide](LICENSING.md) · [license decision guide](docs/LICENSE-DECISION.md) · [commercial terms](COMMERCIAL-LICENSE.md) · [license pins](docs/LICENSE-PINS.md)).

## Private work, fixed in time

A [commitment](docs/COMMITMENTS.md) publishes now the SHA-256 of a manifest of every file in a private directory; the files can be handed over later and checked with `xtask commitment-check`.
It shows the files existed and did not change, not that they are correct. The author's own measurements are in [measured progress](docs/MEASURED-PROGRESS.md); nobody else has re-run them.

## See it in action

Six edited 12-second replays of recorded public-tool runs; the pacing is not execution time.
[Static view](media/gifs/STATIC.md) · [Original source and hashes](media/gifs/MANIFEST.txt)

__GRID__
## The memory core

| Part | What it does | Check it |
|:---|:---|:---|
| Bit record (`gel-orb`) | One fixed 128-byte record of 1,024 bits, 64-byte aligned; its bytes round-trip exactly | `exact_byte_roundtrip` · [format](docs/FORMAT.md) |
| Store (`gel-store`) | `.gel` v2 files carry a CRC64 over the header and over the payload; in the tests every single header bit flip, every payload byte flip and every truncation is rejected, and so is an older or equal save counter | `every_payload_byte_flip_is_rejected` · `generation_rollback_and_equal_generation_are_rejected` |
| Reader (`gel-reader`) | One comparison of two records returns 16 judgments at once, not 16 independent measurements; a search that stops early returns exactly the same top-K matches as a full scan | `progressive_top_k_is_exactly_equal_to_full_top_k` · [Reader16](docs/READER16.md) |
| Rebuild (`gel-structural`) | With its stored parent record, the XOR of the differing bits gives back the exact record; a missing parent or an invalid depth is refused, a wrong parent is not detected | `exact_xor_roundtrip_is_bit_identical` · [contract](docs/STRUCTURAL-CODEC.md) |
| History (`gel-history`) | Each state of one record is stored as a copy or as the XOR difference from the state before, at most two differences from a copy, and is rebuilt bit for bit | `long_history_is_exact_and_residual_depth_never_exceeds_two` · [contract](docs/RECORD-HISTORY.md) |
| Q8 record (`gel-phase-quad`) | One 1,152-byte record read through four equivalent views, each score equal to the reference bit for bit | `storage_is_1152_bytes_and_every_bit_survives` · [contract](docs/Q8-QUAD.md) |
| Memory cost (`gel-physics`) | Pointer-chase latency and read bandwidth of your own RAM from 48 KiB to 256 MiB; nanoseconds and GiB/s only, compared within one machine | `cargo run --locked --offline --release -p gel-physics -- 5` · [method](docs/PERFORMANCE.md) |

`cargo test --locked --offline -p CRATE` runs the tests named here; `cargo run --locked --offline --release -p gel-cli -- selftest` checks the first four parts in one run (`GEL_SELFTEST_V2=PASS`).

## Documentation

| Start and use | Inspect and reproduce | Project |
|:---|:---|:---|
| [Project map](docs/START-HERE.md) | [Claim registry](docs/CLAIMS.md) | [Roadmap](docs/ROADMAP.md) |
| [Evidence Lab](docs/EVIDENCE-LAB.md) · [batch mode](docs/EVIDENCE-BATCH.md) | [Reproduction guide](docs/REPRODUCE.md) | [Release notes v0.7.0](RELEASE-NOTES-v0.7.0.md) |
| [Backup and restore](docs/BACKUP.md) | [Per-platform CI evidence](docs/CI-EVIDENCE.md) | [Publication status](CANDIDATE-STATUS.md) |
| [Your bytes in one record](docs/LIVE-LAB.md) | [Mutation matrix](docs/MUTATION-MATRIX.md) · [crash series](docs/CRASH-SERIES.md) | [Measured progress](docs/MEASURED-PROGRESS.md) |
| [License decision guide](docs/LICENSE-DECISION.md) | [External checks](docs/EXTERNAL-CHECKS.md) | [All media and their scope](media/INDEX.md) |

The [full-page edition](README-MULTIMEDIA.html) opens from your checkout in a browser; GitHub shows HTML files as source ([presentation guide](docs/README-PRESENTATION.md)).

## About

__LOGO__

GEL RAM is an independent hobby and research project by **RR — GEL RAM Project**, developed with AI assistance; this repository is not the complete private research system.
Third-party material keeps its own terms: the Rust Book fixture (MIT), the Unicode data files and the tables derived from them (Unicode License v3) and the quoted Wikipedia passages (CC BY-SA 4.0) ([third-party notices](THIRD-PARTY-NOTICES.md)).
[Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [CLA privacy](CLA-PRIVACY.md) · [Media rights](media/RIGHTS.md)
"####;
const GUIDE: &str = r####"# Multimedia README presentation

This presentation has two views. The root README is normal GitHub Markdown with
an SVG banner with the GEL logo as a turning 3D glass cube (a still version when
reduced motion is requested), a card of one stored Q8 record drawn from the 48
recorded comparison runs in docs/evidence-q8-current, colour-coded workflow
badges and six real GIF previews. Two of its numbers are counted from files when
the README is built: the published commitments in docs/COMMITMENTS.md and the
packages in Cargo.lock, which the builder refuses to count when one comes from
outside the checkout. The adjacent README-MULTIMEDIA.html is a script-free local
document with the full responsive panel layout, theme-aware backgrounds, a
still-image control, an animated 3D scene of the citation check, a strip of
checked facts, two 3D graphics of the public checks and links to the files of
the checkout. Open it from the checkout in a browser; GitHub's file viewer shows
it as source, not as a web page.

The animated scene illustrates the flow the six recordings show: an exact
quote with its byte range, a snapshot pinned by the SHA-256 the user keeps, a
new process that reopens it with the same pin, and a copy with one changed byte
that is refused. It is drawn, not recorded, and its timing is not a
measurement. Under reduced motion it shows its final state.

The logo in the banner and in the full-page brand is the GEL RAM cube drawn in
3D: document sheets settle into a translucent cube as three layers of source
cells around the accent core, a query lights one exact cell and a pin seal
appears. It pictures the public workflow, not how the engine works inside, and
it is not a claim of general AI; under reduced motion it shows its final state.

The builder draws the facts strip and the two check graphics from the property
map, the Unix-only exclusion list of the CI evidence collector and the committed
mutation matrix report. The property-map graphic shows the rule the collector
enforces on each CI platform, not the result of a particular run. A change to
any of these sources makes the read-only check fail until the images are rebuilt.
The record card is drawn the same way from the recorded comparison runs: the
builder refuses to draw it when a run lacks its PASS line or the runs disagree
on a byte count. A change to any of those runs makes the read-only check fail
until the card is rebuilt.

## Preservation and source

The presentation describes the checkout it is in; tagged releases are listed on
the repository's Releases page. Earlier README versions are no longer kept in
the tree; they remain in the git history. The whole README is generated; no part of it is maintained
by hand.

The original GIF files, PNG alternatives, transcripts, fixtures and
[media manifest](../media/gifs/MANIFEST.txt) are not regenerated or relabelled.
Each preview retains its recorded source. Three cards over 12 seconds are edited
log replays, not the CLI's elapsed execution time. No generated mock terminal,
unmeasured speed figure, static CI-passing badge or source-truth claim is added.

## Accessibility and scope

The page uses system fonts, local images, no JavaScript, no tracking and no remote
font requests. Its picture sources prefer PNGs when reduced motion is requested.
The standalone still-image control hides moving previews; it does not claim to
stop GIF decoding. Images have alt text, transcript links and full-size alternatives.
On narrow screens the HTML card grid becomes one column. GitHub controls the
surrounding Markdown page's appearance; only its image assets can carry this design.

The original [static gallery](../media/gifs/STATIC.md) remains available. The
local presentation and its documentation links open without a network.

## Regeneration and checks

The Rust-only presentation builder is tools/readme_presentation.rs. It refuses
unexpected input headings and pre-existing output paths, verifies the original
media pins before and after its edits, archives the actual preceding README and
adds exact pins only for the files it generates. It does not allow arbitrary SVG or
HTML files. The source manifest is then regenerated and the complete tree must
pass xtask verify. Normal three-platform CI is read separately for the final head.

The presentation was built once with Rust 1.85.0 and committed by its author;
no workflow writes to the repository. The read-only `readme-presentation`
workflow compiles and tests the builder and reruns it on every change to the
presentation or its sources. On an existing presentation the builder only
validates: any generated file that differs from the committed one fails the
check, and historic recordings are never replaced. Visual and CI results are
recorded in the pull request; this guide does not predeclare PASS.

## Review criteria

Check the rendered page at desktop and phone widths, both color schemes, all six
previews, the keyboard-accessible still control and reduced-motion image selection.
Confirm no clipped text, horizontal page overflow, remote image/font requests or
script execution; validate all local links and every preserved media hash.
A local browser rendering is not an independent check of GitHub's rendered page.

The license, CLA, historical permissions and third-party notices are unchanged.
[LICENSE](../LICENSE) and [media rights](../media/RIGHTS.md) remain authoritative.
"####;
fn picture(id: &str, title: &str, still: bool) -> String {
    let ext = if still { "png" } else { "gif" };
    format!(
        r#"<picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/{id}-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/{id}-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/{id}-dark.{ext}"><img alt="{title}" src="media/gifs/{id}-light.{ext}" width="1000" loading="lazy"></picture>"#
    )
}
fn pair(id: &str, title: &str) -> String {
    format!(
        "<div class=\"motion\">{}</div><div class=\"poster\">{}</div>",
        picture(id, title, false),
        picture(id, title, true)
    )
}
/// The opening banner: the logo as a large turning glass cube with the name and what it does.
fn header(dark: bool) -> String {
    hero(dark, true)
}
/// The banner without motion, for readers who ask for reduced motion.
fn header_still(dark: bool) -> String {
    hero(dark, false)
}
/// The logo alone, for the brand of the full-page edition: animated, or still
/// in its final state for the page's still-image control and for print.
fn logo(dark: bool, animated: bool) -> String {
    let (t, accent) = (theme(dark), if dark { "#79c0ff" } else { "#0969da" });
    let (o, a) = ((96.0, 108.0), 76.0);
    let (label, style, body) = if animated {
        (
            "GEL RAM animated logo: document sheets settle into a translucent cube, a query lights one exact cell and a pin seal appears",
            format!("<style>\n{LOGO_STYLE}\n</style>\n"),
            gel_logo(t, accent, o, a) + &gel_sheets(t, accent, o, a),
        )
    } else {
        (
            "GEL RAM logo: a translucent cube of source cells with one exact cell lit and a pin seal",
            String::new(),
            gel_logo(t, accent, o, a),
        )
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200" role="img" aria-label="{label}">
{style}{body}
</svg>
"#
    )
}
fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = template.to_string();
    for (key, val) in pairs {
        s = s.replace(&format!("__{key}__"), val);
    }
    s
}
/// Colours of the drawn 3D graphics. Box faces are listed top, left, right.
struct Theme {
    bg: [&'static str; 2],
    line: &'static str,
    ink: &'static str,
    muted: &'static str,
    accent: &'static str,
    floor: &'static str,
    good: &'static str,
    paper: [&'static str; 3],
    blue: [&'static str; 3],
    green: [&'static str; 3],
    red: [&'static str; 3],
    amber: [&'static str; 3],
}
const LIGHT: Theme = Theme {
    bg: ["#f7fbff", "#deedff"],
    line: "#cbdcf0",
    ink: "#102949",
    muted: "#486380",
    accent: "#0969da",
    floor: "#9fbce0",
    good: "#1a7f37",
    paper: ["#ffffff", "#dde7f3", "#c3d2e6"],
    blue: ["#7db4ff", "#2f7fe0", "#1f5fb0"],
    green: ["#6fdc8c", "#2da44e", "#1a7f37"],
    red: ["#ff9b93", "#e5484d", "#b3242a"],
    amber: ["#ffd46b", "#d4a72c", "#9a6700"],
};
const DARK: Theme = Theme {
    bg: ["#101d31", "#0d1117"],
    line: "#30465f",
    ink: "#e6edf3",
    muted: "#9fb3c8",
    accent: "#58a6ff",
    floor: "#26405f",
    good: "#56d364",
    paper: ["#e6edf3", "#9fb3c8", "#6e859e"],
    blue: ["#79c0ff", "#388bfd", "#1f6feb"],
    green: ["#56d364", "#2ea043", "#196c2e"],
    red: ["#ff9492", "#f85149", "#b62324"],
    amber: ["#e3b341", "#bb8009", "#845306"],
};
fn theme(dark: bool) -> &'static Theme {
    if dark {
        &DARK
    } else {
        &LIGHT
    }
}
const COS30: f64 = 0.866;
type Point = (f64, f64);
type Vec3 = (f64, f64, f64);
/// Isometric projection of the world point (x, y, z) around the origin `o`.
fn iso(o: Point, x: f64, y: f64, z: f64) -> Point {
    (o.0 + (x - y) * COS30, o.1 + (x + y) * 0.5 - z)
}
fn polygon(points: &[Point], fill: &str, extra: &str) -> String {
    let points: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect();
    format!(
        r#"<polygon points="{}" fill="{fill}"{extra}/>"#,
        points.join(" ")
    )
}
/// An isometric box at world position `p` with size `d`.
fn iso_box(o: Point, p: Vec3, d: Vec3, faces: [&str; 3], stroke: &str) -> String {
    let ((x, y, z), (dx, dy, dz)) = (p, d);
    let q = |a, b, c| iso(o, a, b, c);
    let edge = format!(r#" stroke="{stroke}" stroke-linejoin="round""#);
    let top = [
        q(x, y, z + dz),
        q(x + dx, y, z + dz),
        q(x + dx, y + dy, z + dz),
        q(x, y + dy, z + dz),
    ];
    let left = [
        q(x, y + dy, z),
        q(x + dx, y + dy, z),
        q(x + dx, y + dy, z + dz),
        q(x, y + dy, z + dz),
    ];
    let right = [
        q(x + dx, y, z),
        q(x + dx, y + dy, z),
        q(x + dx, y + dy, z + dz),
        q(x + dx, y, z + dz),
    ];
    polygon(&left, faces[1], &edge)
        + &polygon(&right, faces[2], &edge)
        + &polygon(&top, faces[0], &edge)
}
/// Transform drawing local coordinates (u right, v down) on a box's right face.
fn right_face(o: Point, p: Vec3, d: Vec3) -> String {
    let (x, y) = iso(o, p.0 + d.0, p.1 + d.1, p.2 + d.2);
    format!("matrix({COS30} -0.5 0 1 {x:.1} {y:.1})")
}
/// Transform drawing local coordinates (u along x, v along y) on a box's top face.
fn top_face(o: Point, p: Vec3, d: Vec3) -> String {
    let (x, y) = iso(o, p.0, p.1, p.2 + d.2);
    format!("matrix({COS30} 0.5 -{COS30} 0.5 {x:.1} {y:.1})")
}
/// A box in oblique projection: front face w × h at (x, y), receding by `depth`.
fn oblique(x: f64, y: f64, (w, h, depth): Vec3, faces: [&str; 3]) -> String {
    let edge = r##" stroke="#00000022" stroke-width=".6""##;
    let (xd, yd) = (x + depth, y - depth);
    polygon(
        &[(x, y), (x + w, y), (x + w, y + h), (x, y + h)],
        faces[1],
        edge,
    ) + &polygon(
        &[(x + w, y), (xd + w, yd), (xd + w, yd + h), (x + w, y + h)],
        faces[2],
        edge,
    ) + &polygon(
        &[(x, y), (xd, yd), (xd + w, yd), (x + w, y)],
        faces[0],
        edge,
    )
}
/// A themed card: `text` is the title and the description for screen readers.
fn card((w, h): (u32, u32), t: &Theme, text: (&str, &str), style: &str, body: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-labelledby="t d">
<title id="t">{title}</title>
<desc id="d">{desc}</desc>
<style>
text{{font-family:Arial,Helvetica,sans-serif;fill:{ink}}}
.mono{{font-family:ui-monospace,Consolas,"DejaVu Sans Mono",monospace;fill:{muted}}}
.kick{{font-size:13px;font-weight:700;letter-spacing:2px;fill:{accent}}}
{style}
</style>
<defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="{bg1}"/><stop offset="1" stop-color="{bg2}"/></linearGradient>
<clipPath id="card"><rect x="1" y="1" width="{iw}" height="{ih}" rx="24"/></clipPath></defs>
<rect x="1" y="1" width="{iw}" height="{ih}" rx="24" fill="url(#bg)" stroke="{line}"/>
{body}
</svg>
"##,
        title = text.0,
        desc = text.1,
        ink = t.ink,
        muted = t.muted,
        accent = t.accent,
        bg1 = t.bg[0],
        bg2 = t.bg[1],
        line = t.line,
        iw = w - 2,
        ih = h - 2
    )
}
const SCENE_STYLE: &str = r#".sheet,.snap,.ok,.bad,.b1,.b2,.b3{opacity:0;transform-box:fill-box;transform-origin:center}
.s0{animation:s0 12s infinite}.s1{animation:s1 12s infinite}.s2{animation:s2 12s infinite}
.b1{animation:b1 12s infinite}.snap{animation:snap 12s infinite}.b2{animation:b2 12s infinite}.b3{animation:b3 12s infinite}
.ok{animation:ok 12s infinite}.bad{animation:bad 12s infinite}.hot{animation:hot 1s infinite alternate}
.beam{animation-name:b1,flow;animation-duration:12s,1s;animation-iteration-count:infinite;animation-timing-function:ease,linear}
.b2.beam{animation-name:b2,flow}.b3.beam{animation-name:b3,flow}
@keyframes s0{0%{opacity:0;transform:translateY(-40px)}6%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes s1{0%,4%{opacity:0;transform:translateY(-40px)}10%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes s2{0%,8%{opacity:0;transform:translateY(-40px)}14%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes b1{0%,16%{opacity:0}22%,94%{opacity:1}100%{opacity:0}}
@keyframes snap{0%,22%{opacity:0;transform:scale(.6)}30%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes b2{0%,34%{opacity:0}40%,94%{opacity:1}100%{opacity:0}}
@keyframes b3{0%,50%{opacity:0}56%,94%{opacity:1}100%{opacity:0}}
@keyframes ok{0%,42%{opacity:0;transform:translateY(-30px)}50%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes bad{0%,56%{opacity:0;transform:translateY(-30px)}62%{opacity:1;transform:none}64%{transform:translateX(-6px)}66%{transform:translateX(6px)}68%{transform:translateX(-4px)}70%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes hot{from{opacity:1}to{opacity:.35}}
@keyframes flow{to{stroke-dashoffset:-36}}
@media (prefers-reduced-motion:reduce){*{animation:none!important}.sheet,.snap,.ok,.bad,.b1,.b2,.b3{opacity:1}}"#;
/// Animated 3D scene of the public citation check: document sheets with a
/// highlighted quote, the pinned snapshot, the reopened copy and the refused
/// copy. It illustrates the recorded workflows; it is not a recording or a
/// timing measurement.
fn scene(dark: bool) -> String {
    let t = theme(dark);
    let o = (100.0, 245.0);
    let mut floor = String::from(r#"<g clip-path="url(#card)" opacity=".55">"#);
    for k in -18..=18 {
        let v = f64::from(k) * 40.0;
        for (a, b) in [
            (iso(o, v, -800.0, 0.0), iso(o, v, 400.0, 0.0)),
            (
                iso(o, -400.0, v - 360.0, 0.0),
                iso(o, 900.0, v - 360.0, 0.0),
            ),
        ] {
            floor.push_str(&format!(
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width=".7"/>"#,
                a.0, a.1, b.0, b.1, t.floor
            ));
        }
    }
    floor.push_str("</g>");
    // Boxes stand on the floor diagonal: `at(a, d)` centres a box of side d at a.
    let at = |a: f64, d: f64| (a - d / 2.0, -a - d / 2.0, 0.0);
    let sheet = (110.0, 90.0, 5.0);
    let (x, y, _) = at(46.0, 90.0);
    let mut docs = String::new();
    for (i, z) in [0.0, 8.0, 16.0].into_iter().enumerate() {
        docs.push_str(&format!(
            r#"<g class="sheet s{i}">{}</g>"#,
            iso_box(o, (x, y, z), sheet, t.paper, t.line)
        ));
    }
    docs.push_str(&format!(
        r#"<g class="sheet s2"><g transform="{}">"#,
        top_face(o, (x, y, 16.0), sheet)
    ));
    for (k, row) in [16.0, 30.0, 44.0, 58.0, 72.0].into_iter().enumerate() {
        let (fill, h, len) = if k == 2 {
            (t.accent, 7.0, 84.0)
        } else {
            (t.floor, 4.0, 70.0 - 4.0 * k as f64)
        };
        docs.push_str(&format!(
            r#"<rect x="14" y="{row}" width="{len}" height="{h}" rx="2" fill="{fill}"/>"#
        ));
    }
    docs.push_str("</g></g>");
    let big = (96.0, 96.0, 96.0);
    let p = at(214.0, 96.0);
    let snap = format!(
        r##"<g class="snap">{}<g transform="{}"><rect x="30" y="44" width="36" height="30" rx="5" fill="#ffffff" opacity=".92"/><path d="M38 44v-9a10 10 0 0 1 20 0v9" fill="none" stroke="#ffffff" stroke-width="5" opacity=".92"/><circle cx="48" cy="57" r="4.5" fill="{}"/></g></g>"##,
        iso_box(o, p, big, t.blue, t.line),
        right_face(o, p, big),
        t.blue[2]
    );
    let small = (80.0, 80.0, 80.0);
    let p = at(404.0, 80.0);
    let ok = format!(
        r##"<g class="ok">{}<g transform="{}"><path d="M22 42l14 14 26-30" fill="none" stroke="#ffffff" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/></g></g>"##,
        iso_box(o, p, small, t.green, t.line),
        top_face(o, p, small)
    );
    let p = at(543.0, 80.0);
    let mut cells = String::new();
    for i in 0..4 {
        for j in 0..4 {
            let hot = (i, j) == (2, 1);
            cells.push_str(&format!(
                r##"<rect{} x="{}" y="{}" width="16" height="16" rx="2" fill="#ffffff" opacity="{}"/>"##,
                if hot { r#" class="hot""# } else { "" },
                8 + 17 * i,
                8 + 17 * j,
                if hot { "1" } else { ".22" }
            ));
        }
    }
    let bad = format!(
        r#"<g class="bad">{}<g transform="{}">{cells}</g></g>"#,
        iso_box(o, p, small, t.red, t.line),
        right_face(o, p, small)
    );
    let beam = |a: f64, b: f64, class: &str| {
        let ((x1, y1), (x2, y2)) = (iso(o, a, -a, 30.0), iso(o, b, -b, 30.0));
        format!(
            r#"<path class="beam {class}" d="M{x1:.1} {y1:.1}H{x2:.1}" stroke="{c}" stroke-width="4" stroke-dasharray="10 8" stroke-linecap="round" fill="none"/><path class="{class}" d="M{x2:.1} {y2:.1}l-12 -7v14z" fill="{c}"/>"#,
            c = t.accent
        )
    };
    let arc = format!(
        r#"<path class="beam b3" d="M540 96Q760 -6 985 104" stroke="{c}" stroke-width="4" stroke-dasharray="10 8" stroke-linecap="round" fill="none"/><path class="b3" d="M992 110l-15 -1 8 -12z" fill="{c}"/><text class="mono b3" x="760" y="84" text-anchor="middle" font-size="13">copy with 1 byte flipped</text>"#,
        c = t.accent
    );
    let label = |x: u32, head: &str, sub: &str, class: &str| {
        format!(
            r#"<g class="{class}"><text x="{x}" y="318" text-anchor="middle" font-size="18" font-weight="700">{head}</text><text class="mono" x="{x}" y="340" text-anchor="middle" font-size="13">{sub}</text></g>"#
        )
    };
    let body = [
        floor,
        r#"<text class="kick" x="40" y="44">WHAT HAPPENS TO A CITATION</text>"#.to_string(),
        beam(108.0, 150.0, "b1"),
        beam(282.0, 350.0, "b2"),
        arc,
        docs,
        snap,
        ok,
        bad,
        label(180, "Your document", "exact quote, bytes 67..105", "s0"),
        label(470, "Pinned snapshot", "SHA-256 you keep", "snap"),
        label(800, "Reopened · PASS", "new process, same pin", "ok"),
        label(1041, "1 byte changed · REFUSED", "copy with the old pin", "bad"),
        r#"<text class="mono" x="40" y="368" font-size="12">Drawn, not recorded: the flow the six recorded runs below show · not a timing measurement</text>"#.to_string(),
    ]
    .concat();
    card(
        (1200, 380),
        t,
        (
            "What happens to a citation",
            "Animated 3D illustration: document sheets with a highlighted exact quote become a snapshot pinned by SHA-256; a new process reopens it with the same pin and passes; a copy with one changed byte is refused. Drawn, not recorded; not a timing measurement.",
        ),
        SCENE_STYLE,
        &body,
    )
}
/// Mutation counts of one public format in the committed matrix report.
#[derive(Default)]
struct FormatCount {
    name: String,
    /// Distinct mutants; each is read with the original and a recomputed pin.
    mutants: usize,
    /// Report rows whose declared verdict is a rejection or an acceptance.
    rejected: usize,
    accepted: usize,
    /// Report rows whose observed result equals the declared one.
    matched: usize,
}
/// Public evidence the facts strip and the data graphics are drawn from, read
/// from this checkout so that no number can drift from its source.
struct Evidence {
    /// Property map rows: the ID and the tests the row names.
    rows: Vec<(String, Vec<String>)>,
    /// Tests the CI evidence collector declares Unix-only.
    unix_only: Vec<String>,
    /// Formats in name order.
    formats: Vec<FormatCount>,
}
impl Evidence {
    fn read() -> Result<Evidence> {
        let map = fs::read_to_string("docs/PROPERTY-TESTS.md")?;
        let rows: Vec<(String, Vec<String>)> = map
            .lines()
            .filter(|l| l.starts_with("| P"))
            .map(|l| {
                let id = l.split('|').nth(1).unwrap_or("").trim().to_string();
                let cell = l.rsplit('|').nth(1).unwrap_or("");
                let tests = cell.split('`').skip(1).step_by(2).map(str::to_string);
                (id, tests.collect())
            })
            .collect();
        let ci = fs::read_to_string("xtask/src/ci_evidence.rs")?;
        let list = ci
            .split_once("const EXCLUSIONS")
            .and_then(|(_, rest)| rest.split_once("];"))
            .ok_or("missing CI exclusion list")?
            .0;
        // The list holds (test, reason) pairs of string literals.
        let literals: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
        let unix_only: Vec<String> = literals.iter().step_by(2).map(|s| s.to_string()).collect();
        if literals.len() % 2 != 0 || unix_only.iter().any(|s| s.is_empty() || s.contains(' ')) {
            return Err("unexpected CI exclusion list".into());
        }
        let matrix = fs::read_to_string("docs/evidence-mutation/matrix-r1.txt")?;
        let mut lines = matrix.lines();
        if lines.next() != Some("format\tcase\tregion\toperator\tpin\texpected\tobserved\tverdict")
        {
            return Err("unexpected mutation matrix header".into());
        }
        let mut formats: Vec<FormatCount> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for line in lines {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != 8 {
                return Err(format!("unexpected mutation matrix row: {line}").into());
            }
            let i = match formats.iter().position(|c| c.name == f[0]) {
                Some(i) => i,
                None => {
                    formats.push(FormatCount {
                        name: f[0].to_string(),
                        ..FormatCount::default()
                    });
                    formats.len() - 1
                }
            };
            let c = &mut formats[i];
            c.mutants += usize::from(seen.insert((f[0], f[1])));
            if f[5] == "ACCEPT" {
                c.accepted += 1;
            } else {
                c.rejected += 1;
            }
            c.matched += usize::from(f[5] == f[6] && f[7] == "PASS");
        }
        formats.sort_by(|a, b| a.name.cmp(&b.name));
        if rows.is_empty() || rows.iter().any(|r| r.1.is_empty()) || formats.is_empty() {
            return Err("missing evidence sources".into());
        }
        Ok(Evidence {
            rows,
            unix_only,
            formats,
        })
    }
    /// Property rows that name at least one declared Unix-only test.
    fn unix_rows(&self) -> Vec<bool> {
        let unix = |test: &String| self.unix_only.contains(test);
        self.rows
            .iter()
            .map(|(_, tests)| tests.iter().any(unix))
            .collect()
    }
    /// Counts shown in the facts strip: properties, mapped tests and mutants.
    fn fact_counts(&self) -> (usize, usize, usize) {
        let tests = self.rows.iter().map(|r| r.1.len()).sum();
        let mutants = self.formats.iter().map(|f| f.mutants).sum();
        (self.rows.len(), tests, mutants)
    }
}
/// Property map across the three CI platforms, as the CI evidence collector
/// checks it. It draws the rule, not the result of a particular run.
fn wall(dark: bool, e: &Evidence) -> String {
    let t = theme(dark);
    let (side, depth, step, x0) = (19.0, 8.0, 26.0, 150.0);
    let n = e.rows.len();
    let unix = e.unix_rows();
    let partial = unix.iter().filter(|u| **u).count();
    let full = 3 * n - partial;
    let mut body = format!(
        r#"<text class="kick" x="40" y="44">PROPERTY MAP × CI PLATFORMS</text><text x="40" y="84" font-size="26" font-weight="800">{n} properties × 3 platforms</text><text class="mono" x="1160" y="84" text-anchor="end" font-size="12">the rule, not one run: read each run's CI report</text><text class="mono" x="40" y="108" font-size="13">each cell: every mapped test must run and pass exactly once on that platform, or CI evidence is not COMPLETE</text>"#
    );
    for (r, platform) in ["Linux", "macOS", "Windows"].into_iter().enumerate() {
        let back = (2 - r) as f64;
        let (row_x, row_y) = (x0 + 16.0 * back, 250.0 - 34.0 * back);
        body.push_str(&format!(
            r#"<text x="{:.0}" y="{:.0}" text-anchor="end" font-size="14" font-weight="700">{platform}</text>"#,
            row_x - 12.0,
            row_y + 14.0
        ));
        for (c, unix_only) in unix.iter().enumerate() {
            let faces = if r == 2 && *unix_only {
                t.amber
            } else {
                t.blue
            };
            body.push_str(&format!(
                r#"<g class="cube" style="animation-delay:{:.2}s">{}</g>"#,
                0.035 * c as f64 + 0.12 * r as f64,
                oblique(row_x + step * c as f64, row_y, (side, side, depth), faces)
            ));
        }
    }
    for c in (0..n).filter(|c| c % 5 == 0 || *c == n - 1) {
        body.push_str(&format!(
            r#"<text class="mono" x="{:.1}" y="300" text-anchor="middle" font-size="11">{}</text>"#,
            x0 + step * c as f64 + side / 2.0,
            e.rows[c].0
        ));
    }
    body.push_str(&format!(
        r#"<rect x="40" y="318" width="14" height="14" rx="3" fill="{}"/><text x="62" y="330" font-size="13">{full} cells: every mapped test required</text><rect x="380" y="318" width="14" height="14" rx="3" fill="{}"/><text x="402" y="330" font-size="13">{partial} Windows cells: declared Unix-only tests exempt, the rest required</text>"#,
        t.blue[1], t.amber[1]
    ));
    card(
        (1200, 350),
        t,
        (
            "Property map across CI platforms",
            &format!("{n} documented properties by 3 CI platforms, as the CI evidence collector checks them. In {full} cells every mapped test must run and pass exactly once; in {partial} Windows cells the declared Unix-only tests are exempt and the rest are required. This is the rule, not the result of a particular run."),
        ),
        ".cube{animation:rise .9s cubic-bezier(.2,.8,.2,1) both}@keyframes rise{from{opacity:0;transform:translateY(26px)}to{opacity:1;transform:none}}@media (prefers-reduced-motion:reduce){.cube{animation:none}}",
        &body,
    )
}
/// The committed format mutation report as stacked 3D bars per format.
fn bars(dark: bool, e: &Evidence) -> String {
    let t = theme(dark);
    let mutants: usize = e.formats.iter().map(|f| f.mutants).sum();
    let rejected: usize = e.formats.iter().map(|f| f.rejected).sum();
    let accepted: usize = e.formats.iter().map(|f| f.accepted).sum();
    let matched: usize = e.formats.iter().map(|f| f.matched).sum();
    let rows = rejected + accepted;
    let tallest = e
        .formats
        .iter()
        .map(|f| f.rejected + f.accepted)
        .max()
        .unwrap_or(1);
    let (base, reach, w, depth) = (300.0, 150.0, 110.0, 36.0);
    let mut body =
        String::from(r#"<text class="kick" x="40" y="44">FORMAT MUTATION MATRIX</text>"#);
    for (i, f) in e.formats.iter().enumerate() {
        let x = 110.0 + 230.0 * i as f64;
        let height = |n: usize| n as f64 / tallest as f64 * reach;
        let (hr, ha) = (height(f.rejected), height(f.accepted));
        let block = |top: f64, h: f64, faces| {
            if h > 0.0 {
                oblique(x, top, (w, h, depth), faces)
            } else {
                String::new()
            }
        };
        let (mid, top) = (x + w / 2.0, base - hr - ha - depth);
        body.push_str(&format!(
            r#"<g class="bar" style="animation-delay:{:.2}s">{}{}</g><g class="num" style="animation-delay:{:.2}s"><text x="{:.0}" y="{:.0}" text-anchor="middle" font-size="22" font-weight="800">{}</text><text class="mono" x="{:.0}" y="{:.0}" text-anchor="middle" font-size="12">mutants</text></g><text x="{mid:.0}" y="334" text-anchor="middle" font-size="15" font-weight="700">{}</text><text class="mono" x="{mid:.0}" y="354" text-anchor="middle" font-size="12">{} rejected · {} accepted</text>"#,
            0.2 * i as f64,
            block(base - hr, hr, t.red),
            block(base - hr - ha, ha, t.blue),
            0.8 + 0.2 * i as f64,
            mid + depth / 2.0,
            top - 30.0,
            f.mutants,
            mid + depth / 2.0,
            top - 14.0,
            f.name,
            f.rejected,
            f.accepted
        ));
    }
    let verdict = if matched == rows { t.good } else { t.red[1] };
    body.push_str(&format!(
        r#"<g class="num" style="animation-delay:1.4s"><text x="830" y="120" font-size="46" font-weight="800" style="fill:{verdict}">{matched}/{rows}</text><text x="830" y="148" font-size="15">rows matched their expected verdict</text><text x="830" y="196" font-size="30" font-weight="800" style="fill:{}">{mutants}</text><text x="900" y="196" font-size="15">byte-level mutants of the</text><text x="830" y="218" font-size="15">three public formats, each read with the</text><text x="830" y="238" font-size="15">original pin and with a recomputed one</text><rect x="830" y="262" width="14" height="14" rx="3" fill="{}"/><text x="852" y="274" font-size="13">rejected: {rejected}</text><rect x="960" y="262" width="14" height="14" rx="3" fill="{}"/><text x="982" y="274" font-size="13">accepted as documented: {accepted}</text><text class="mono" x="830" y="310" font-size="12">committed report; xtask verify requires</text><text class="mono" x="830" y="328" font-size="12">an identical rerun</text></g>"#,
        t.accent, t.red[1], t.blue[1]
    ));
    card(
        (1200, 380),
        t,
        (
            "Format mutation matrix",
            &format!("{mutants} byte-level mutants of the three public formats, each read with the original pin and with a recomputed one. In the committed report {matched} of {rows} rows matched their expected verdict: {rejected} rejected, {accepted} accepted as documented."),
        ),
        ".bar{animation:grow 1.1s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:bottom}.num{animation:fade .8s ease-out both}@keyframes grow{from{transform:scaleY(0)}to{transform:none}}@keyframes fade{from{opacity:0}to{opacity:1}}@media (prefers-reduced-motion:reduce){.bar,.num{animation:none}}",
        &body,
    )
}
const LOGO_STYLE: &str = r#".sheet,.l0,.l1,.l2,.hit,.ray,.seal{transform-box:fill-box;transform-origin:center}
.sa{animation:sa 12s infinite}.sb{animation:sb 12s infinite}.sc{animation:sc 12s infinite}
.l0{animation:l0 12s infinite}.l1{animation:l1 12s infinite}.l2{animation:l2 12s infinite}
.ray{animation:ray 12s infinite}.hit{animation:hit 12s infinite}.seal{animation:seal 12s infinite}
@keyframes sa{0%{opacity:0;transform:translateY(-26px)}3%{opacity:1}10%{opacity:1;transform:translateY(34px) scale(.7)}13%,100%{opacity:0;transform:translateY(40px) scale(.4)}}
@keyframes sb{0%,12%{opacity:0;transform:translateY(-26px)}15%{opacity:1}22%{opacity:1;transform:translateY(34px) scale(.7)}25%,100%{opacity:0;transform:translateY(40px) scale(.4)}}
@keyframes sc{0%,24%{opacity:0;transform:translateY(-26px)}27%{opacity:1}34%{opacity:1;transform:translateY(34px) scale(.7)}37%,100%{opacity:0;transform:translateY(40px) scale(.4)}}
@keyframes l0{0%,9%{opacity:0;transform:scale(.3)}13%,92%{opacity:1;transform:none}98%,100%{opacity:0}}
@keyframes l1{0%,21%{opacity:0;transform:scale(.3)}25%,92%{opacity:1;transform:none}98%,100%{opacity:0}}
@keyframes l2{0%,33%{opacity:0;transform:scale(.3)}37%,92%{opacity:1;transform:none}98%,100%{opacity:0}}
@keyframes ray{0%,44%{opacity:0;stroke-dashoffset:60}46%{opacity:1}52%,80%{opacity:1;stroke-dashoffset:0}86%,100%{opacity:0;stroke-dashoffset:0}}
@keyframes hit{0%,51%{opacity:0;transform:scale(.6)}55%{opacity:1;transform:scale(1.25)}59%,92%{opacity:1;transform:none}98%,100%{opacity:0}}
@keyframes seal{0%,60%{opacity:0;transform:scale(.4)}64%{opacity:1;transform:scale(1.15)}67%,92%{opacity:1;transform:none}98%,100%{opacity:0}}
@media (prefers-reduced-motion:reduce){.sheet{animation:none;opacity:0}.l0,.l1,.l2,.ray,.hit,.seal{animation:none;opacity:1}}"#;
/// The GEL RAM logo as an animated 3D drawing: a translucent cube (the
/// collection) around the logo's accent core. Document sheets settle into it
/// as three layers of source cells, a query ray lights one exact cell and a
/// pin seal appears. `o` is the cube centre and `a` its side. Drawn, not
/// recorded: it pictures the public workflow, not how the engine works inside.
fn gel_logo(t: &Theme, accent: &str, o: Point, a: f64) -> String {
    let q = |x, y, z| iso(o, x, y, z);
    let mut s = format!(
        r#"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="{}" opacity=".12"/>"#,
        o.0,
        o.1 + a + 4.0,
        0.75 * a,
        0.1 * a,
        t.ink
    );
    // Back edges, seen through the cube.
    let back = q(0.0, 0.0, 0.0);
    for e in [q(a, 0.0, 0.0), q(0.0, a, 0.0), q(0.0, 0.0, a)] {
        s.push_str(&format!(
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="2" stroke-dasharray="4 5" opacity=".45"/>"#,
            back.0, back.1, e.0, e.1, t.ink
        ));
    }
    // Three layers of source cells; the centre of the middle layer is the core.
    let step = a / 3.0;
    let (cell, core) = (a / 8.2, a / 3.2);
    let at = |n: usize, size: f64| (n as f64 + 0.5) * step - size / 2.0;
    for k in 0..3 {
        s.push_str(&format!(r#"<g class="l{k}">"#));
        for i in 0..3 {
            for j in (0..3).filter(|j| (i, *j, k) != (1, 1, 1)) {
                let p = (at(i, cell), at(j, cell), at(k, cell));
                s.push_str(&iso_box(o, p, (cell, cell, cell), t.paper, t.line));
            }
        }
        s.push_str("</g>");
        if k == 1 {
            let p = (at(1, core), at(1, core), at(1, core));
            s.push_str(&iso_box(o, p, (core, core, core), t.blue, t.ink));
        }
    }
    // The exact cell a query finds, at the upper right edge.
    let big = cell + 4.0;
    let p = (at(2, big), at(0, big), at(2, big));
    s.push_str(&format!(
        r#"<g class="hit">{}</g>"#,
        iso_box(o, p, (big, big, big), t.green, t.ink)
    ));
    // Translucent faces over the cells, then the outline and the front edges.
    for (face, opacity) in [
        (
            [q(0.0, 0.0, a), q(a, 0.0, a), q(a, a, a), q(0.0, a, a)],
            ".10",
        ),
        (
            [q(0.0, a, 0.0), q(a, a, 0.0), q(a, a, a), q(0.0, a, a)],
            ".16",
        ),
        (
            [q(a, 0.0, 0.0), q(a, a, 0.0), q(a, a, a), q(a, 0.0, a)],
            ".24",
        ),
    ] {
        s.push_str(&polygon(&face, accent, &format!(r#" opacity="{opacity}""#)));
    }
    let outline = [
        q(0.0, 0.0, a),
        q(a, 0.0, a),
        q(a, 0.0, 0.0),
        q(a, a, 0.0),
        q(0.0, a, 0.0),
        q(0.0, a, a),
    ];
    let edge = format!(
        r#" stroke="{}" stroke-width="4" stroke-linejoin="round""#,
        t.ink
    );
    s.push_str(&polygon(&outline, "none", &edge));
    let front = q(a, a, a);
    let mut d = String::new();
    for e in [q(a, 0.0, a), q(0.0, a, a), q(a, a, 0.0)] {
        d.push_str(&format!(
            "M{:.1} {:.1}L{:.1} {:.1}",
            front.0, front.1, e.0, e.1
        ));
    }
    s.push_str(&format!(
        r#"<path d="{d}" stroke="{}" stroke-width="4" stroke-linecap="round"/>"#,
        t.ink
    ));
    // Glass highlight along the upper edges.
    let (h1, h2, h3) = (q(0.0, a, a), q(0.0, 0.0, a), q(a, 0.0, a));
    s.push_str(&format!(
        r##"<path d="M{:.1} {:.1}L{:.1} {:.1}L{:.1} {:.1}" fill="none" stroke="#ffffff" stroke-width="1.5" opacity=".7"/>"##,
        h1.0 + 6.0,
        h1.1 + 1.0,
        h2.0,
        h2.1 + 6.0,
        h3.0 - 6.0,
        h3.1 + 1.0
    ));
    // The query ray from the upper right to the exact cell.
    let target = q(2.5 * step, 0.5 * step, 2.5 * step + cell / 2.0);
    s.push_str(&format!(
        r#"<path class="ray" d="M{:.1} {:.1}L{:.1} {:.1}" stroke="{accent}" stroke-width="3" stroke-dasharray="6 5" stroke-linecap="round" fill="none"/>"#,
        o.0 + 1.22 * a,
        o.1 - 1.2 * a,
        target.0 + 2.0,
        target.1 - 3.0
    ));
    // The pin seal at the lower right.
    let (x, y) = (o.0 + 0.95 * a, o.1 + 0.62 * a);
    s.push_str(&format!(
        r##"<g class="seal"><circle cx="{x:.1}" cy="{y:.1}" r="15" fill="{}" stroke="{}" stroke-width="2.5"/><path d="M{:.1} {y:.1}l5 5 9-10" fill="none" stroke="#ffffff" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/></g>"##,
        t.green[1],
        t.ink,
        x - 7.0
    ));
    s
}
/// The three document sheets that drop into the cube; only the animated logo
/// draws them, so a still logo carries no hidden elements.
fn gel_sheets(t: &Theme, accent: &str, o: Point, a: f64) -> String {
    let mut s = String::new();
    for (class, dx, dy) in [("sa", -34.0, 6.0), ("sb", 0.0, -6.0), ("sc", 32.0, 4.0)] {
        let (x, y) = (o.0 + dx - 11.0, o.1 - 1.3 * a + dy);
        s.push_str(&format!(
            r#"<g class="sheet {class}"><rect x="{x:.1}" y="{y:.1}" width="22" height="28" rx="2" fill="{}" stroke="{}" stroke-width="1.5"/><path d="M{:.1} {:.1}h13M{:.1} {:.1}h13M{:.1} {:.1}h9" stroke="{accent}" stroke-width="1.5"/></g>"#,
            t.paper[0],
            t.ink,
            x + 4.5,
            y + 8.0,
            x + 4.5,
            y + 14.0,
            x + 4.5,
            y + 20.0
        ));
    }
    s
}
fn facts(dark: bool, (props, tests, mutants): (usize, usize, usize)) -> String {
    let v = if dark {
        [
            "#9fb3c8", "#151b23", "#30465f", "#58a6ff", "#bc8cff", "#f0883e", "#3fb950",
        ]
    } else {
        [
            "#486380", "#ffffff", "#cbdcf0", "#0969da", "#8250df", "#bc4c00", "#1a7f37",
        ]
    };
    let (props, tests, mutants) = (props.to_string(), tests.to_string(), mutants.to_string());
    fill(
        FACTS,
        &[
            ("MUTED", v[0]),
            ("PAPER", v[1]),
            ("LINE", v[2]),
            ("C1", v[3]),
            ("C2", v[4]),
            ("C3", v[5]),
            ("C4", v[6]),
            ("PROPS", &props),
            ("TESTS", &tests),
            ("MUTANTS", &mutants),
        ],
    )
}
fn chip(i: usize, dark: bool) -> String {
    let (label, icon, light, dark_colour) = CHIPS[i];
    let (colour, tint) = if dark {
        (dark_colour, ".16")
    } else {
        (light, ".1")
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="40" viewBox="0 0 320 40" role="img" aria-label="Workflow {label}"><rect x="1" y="1" width="318" height="38" rx="19" fill="{colour}" fill-opacity="{tint}" stroke="{colour}" stroke-width="1.5"/><g transform="translate(12 8)" fill="none" stroke="{colour}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="{icon}"/></g><text x="46" y="26" font-family="Arial,Helvetica,sans-serif" font-size="14" font-weight="700" letter-spacing="1" fill="{colour}">{label}</text></svg>
"#
    )
}
fn chip_path(i: usize, dark: bool) -> String {
    let theme = if dark { "dark" } else { "light" };
    format!("media/presentation/chips/{:02}-{theme}.svg", i + 1)
}
/// A light/dark SVG picture. Only the width is fixed, so it keeps its proportions.
fn themed(light: &str, dark: &str, alt: &str, width: u32) -> String {
    format!(
        r#"<picture><source media="(prefers-color-scheme: dark)" srcset="{dark}"><img alt="{alt}" src="{light}" width="{width}"></picture>"#
    )
}
fn chip_picture(i: usize) -> String {
    themed(
        &chip_path(i, false),
        &chip_path(i, true),
        &format!("Workflow {}", CHIPS[i].0),
        320,
    )
}
fn flow_picture() -> String {
    themed(
        "media/presentation/flow-light.svg",
        "media/presentation/flow-dark.svg",
        "What happens to a citation, drawn in 3D: your document with an exact quote, a snapshot pinned by the SHA-256 you keep, a new process that reopens it with the same pin and passes, and a copy with one changed byte that is refused.",
        1200,
    )
}
fn facts_picture() -> String {
    themed(
        "media/presentation/facts-light.svg",
        "media/presentation/facts-dark.svg",
        "Checked facts: documented properties mapped to tests, checked on every CI platform with declared Unix-only exclusions, three CI platforms, format mutants each rejected or explained, network isolation verified in the strict reproduction.",
        1200,
    )
}
fn wall_picture() -> String {
    themed(
        "media/presentation/wall-light.svg",
        "media/presentation/wall-dark.svg",
        "Property map across CI platforms: one cell per documented property on Linux, macOS and Windows. Every mapped test must run and pass exactly once on each platform; declared Unix-only tests are exempt on Windows.",
        1200,
    )
}
/// The logo in the brand of the full-page edition, animated or still like the
/// previews, so the still-image control and print stop it too. The brand name
/// next to it carries the text, so the image is decorative.
fn logo_picture() -> String {
    let img = |name: &str| {
        themed(
            &format!("media/presentation/{name}-light.svg"),
            &format!("media/presentation/{name}-dark.svg"),
            "",
            84,
        )
    };
    format!(
        r#"<div class="motion">{}</div><div class="poster">{}</div>"#,
        img("logo"),
        img("logo-still")
    )
}
/// The logo for the README: animated, or still when reduced motion is requested,
/// in the same way as the banner.
fn readme_logo() -> String {
    let path = |name: &str, theme: &str| format!("media/presentation/{name}-{theme}.svg");
    format!(
        r#"<picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="{}"><source media="(prefers-reduced-motion: reduce)" srcset="{}"><source media="(prefers-color-scheme: dark)" srcset="{}"><img alt="GEL RAM logo: a glass cube of record cells around an accent core." src="{}" width="120"></picture>"#,
        path("logo-still", "dark"),
        path("logo-still", "light"),
        path("logo", "dark"),
        path("logo", "light")
    )
}
fn bars_picture() -> String {
    themed(
        "media/presentation/bars-light.svg",
        "media/presentation/bars-dark.svg",
        "Format mutation matrix: rejected and accepted rows per public format in the committed report, and how many rows matched their expected verdict.",
        1200,
    )
}
const RESULT_STYLE: &str = ".mut{fill:MUTED}.b{font-weight:700}";
fn result_card(dark: bool, h: u32, text: (&str, &str), body: &str) -> String {
    let t = theme(dark);
    card(
        (1200, h),
        t,
        text,
        &RESULT_STYLE.replace("MUTED", t.muted),
        body,
    )
}
/// The recorded comparison runs the record card is drawn from.
const RECORD_DIR: &str = "docs/evidence-q8-current";
/// The verdict line every recorded run must end with.
const Q8_PASS: &str = "Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED";
/// A stored record: one-byte values and an activity mask of one bit per value.
const RECORD_VALUES: usize = 1024;
const RECORD_MASK: usize = 128;
/// What the recorded comparison runs of the four views state: how many runs there
/// are, how many end with the exact verdict, and the byte counts they all agree on.
struct Q8Runs {
    n: usize,
    ok: usize,
    /// One stored record.
    record: usize,
    /// Four packed copies of it, which are not stored.
    packed: usize,
    /// The materialized reference layout used for checking.
    frames: usize,
}
impl Q8Runs {
    fn read() -> Result<Q8Runs> {
        let mut names: Vec<String> = fs::read_dir(RECORD_DIR)?
            .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<std::io::Result<_>>()?;
        names.retain(|n| n.starts_with("v2-") && n.ends_with(".txt"));
        names.sort();
        let texts = names
            .iter()
            .map(|n| fs::read_to_string(format!("{RECORD_DIR}/{n}")))
            .collect::<std::io::Result<Vec<_>>>()?;
        Q8Runs::from_texts(&texts)
    }
    /// Refuses to give numbers unless every run has exactly one verdict line, the
    /// exact one, and all runs agree on every byte count the card shows.
    fn from_texts(texts: &[String]) -> Result<Q8Runs> {
        let value = |text: &str, key: &str| -> Result<usize> {
            let found = text
                .split_whitespace()
                .find_map(|w| w.strip_prefix(key)?.strip_prefix('='))
                .ok_or(format!("a recorded run has no {key}"))?;
            Ok(found.parse::<usize>()?)
        };
        let mut sizes = Vec::new();
        let mut ok = 0;
        for text in texts {
            let verdicts: Vec<&str> = text
                .lines()
                .filter(|l| l.starts_with("Q8_QUAD_EXACT="))
                .collect();
            ok += usize::from(verdicts == [Q8_PASS]);
            sizes.push([
                value(text, "canonical_record_bytes")?,
                value(text, "packed_four_record_bytes")?,
                value(text, "materialized_four_frame_bytes")?,
            ]);
        }
        let Some(&[record, packed, frames]) = sizes.first() else {
            return Err("no recorded comparison runs".into());
        };
        if ok != texts.len() {
            return Err(format!(
                "{} of {} recorded runs lack {Q8_PASS}",
                texts.len() - ok,
                texts.len()
            )
            .into());
        }
        if sizes.iter().any(|s| *s != [record, packed, frames]) {
            return Err("the recorded runs disagree on a byte count".into());
        }
        if record != RECORD_VALUES + RECORD_MASK || packed != 4 * record {
            return Err("the recorded byte counts no longer match the card's wording".into());
        }
        Ok(Q8Runs {
            n: texts.len(),
            ok,
            record,
            packed,
            frames,
        })
    }
}
/// A count with a comma between groups of three digits: 1152 is "1,152".
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}
/// What the record card says, as its description and as the README alternative text.
fn record_text(r: &Q8Runs) -> String {
    format!("One stored Q8 record: {} one-byte values and a {RECORD_MASK}-byte activity mask, {} bytes, read through four equivalent views without four copies. Four packed copies would take {} bytes; the reference layout used for checking takes {} bytes. {} of {} recorded runs end with Q8_QUAD_EXACT=PASS; semantic accuracy is not measured. Record payload only, not process memory; not four independent memories.", grouped(RECORD_VALUES), grouped(r.record), grouped(r.packed), grouped(r.frames), r.ok, r.n)
}
/// Length of one loop of the record card's animation, in seconds.
const RECORD_LOOP: u32 = 12;
/// One looping CSS animation of the record card. Every element is timed by its
/// keyframe percentages alone, never by a delay, so all share one clock.
fn record_motion(name: &str, timing: &str, frames: &str) -> String {
    format!(".{name}{{animation:{name} {RECORD_LOOP}s {timing} infinite}}@keyframes {name}{{{frames}}}\n")
}
/// One stored record drawn to scale as values and mask; four views read that same
/// bar in turn and each lands on the score of a separate written-out copy; the
/// sizes it is compared with grow to scale; the verdict of every recorded run
/// follows. One loop of RECORD_LOOP seconds; with reduced motion, or without CSS
/// animation, the complete final picture stands still.
fn record_card(dark: bool, r: &Q8Runs) -> String {
    let t = theme(dark);
    let record = grouped(r.record);
    let mut css = RESULT_STYLE.replace("MUTED", t.muted);
    css.push_str(".a{transform-box:fill-box;transform-origin:center}.grow{transform-origin:left center}.scan,.arrow{opacity:0}\n");
    // 1. The stored record appears: one bar, values then mask, to scale.
    css += &record_motion(
        "r0",
        "ease-out",
        "0%{opacity:0}5%,97%{opacity:1}100%{opacity:0}",
    );
    css += &record_motion("g0", "ease-out", "0%{opacity:0;transform:scaleX(0)}1%{opacity:1}7%,97%{opacity:1;transform:none}100%{opacity:0;transform:none}");
    let mut b = format!(
        r#"<text class="kick" x="40" y="48">ONE STORED Q8 RECORD</text><g class="a r0"><text x="40" y="112" font-size="52" class="b" style="fill:{}">{record} B</text><text x="40" y="144" font-size="16" class="mut">{} one-byte values + {RECORD_MASK}-byte activity mask</text></g>"#,
        t.accent,
        grouped(RECORD_VALUES)
    );
    let (x0, y0, w, h) = (40.0, 196.0, 700.0, 52.0);
    let split = w * RECORD_VALUES as f64 / r.record as f64;
    b.push_str(&format!(
        r#"<defs><clipPath id="rec"><rect x="{x0}" y="{y0}" width="{w}" height="{h}" rx="8"/></clipPath></defs><g class="a grow g0"><g clip-path="url(#rec)"><rect x="{x0}" y="{y0}" width="{split:.1}" height="{h}" fill="{}"/><rect x="{:.1}" y="{y0}" width="{:.1}" height="{h}" fill="{}"/>"#,
        t.blue[0],
        x0 + split,
        w - split,
        t.amber[0]
    ));
    for k in 1..16 {
        let x = x0 + split * f64::from(k) / 16.0;
        b.push_str(&format!(
            r#"<path d="M{x:.1} {y0}v{h}" stroke="{}" stroke-width="1" opacity=".35"/>"#,
            t.blue[2]
        ));
    }
    b.push_str(&format!(
        r#"</g><rect x="{x0}" y="{y0}" width="{w}" height="{h}" rx="8" fill="none" stroke="{}" stroke-width="2"/></g><g class="a r0"><text class="mono" x="{x0}" y="268" font-size="12">{} B values</text><text class="mono" x="{}" y="268" font-size="12" text-anchor="middle">stored once; every view reads it in place</text><text class="mono" x="{}" y="268" font-size="12" text-anchor="end">{RECORD_MASK} B mask</text></g><text x="{x0}" y="294" font-size="13" class="mut">Each view reads the same bytes in another fixed arrangement.</text>"#,
        t.ink,
        grouped(RECORD_VALUES),
        x0 + w / 2.0,
        x0 + w
    ));
    // 2. and 3. Four views read the same bar in turn; each lands on the score
    // of a separate written-out copy. Every sweep is drawn alike: the picture
    // does not say how a view arranges the bytes.
    let (head, mut scans, mut arrows) = (44.0, String::new(), String::new());
    b.push_str(r#"<text class="kick" x="790" y="48">EACH VIEW, BIT FOR BIT</text><text class="mono" x="790" y="74" font-size="12">score vs a separate written-out copy</text>"#);
    for i in 0..4u32 {
        let (s, e) = (9 + 11 * i, 17 + 11 * i);
        css += &record_motion(&format!("s{i}"), "linear", &format!("0%,{}%{{opacity:0;transform:translateX(0px)}}{s}%{{opacity:1;transform:translateX(0px)}}{e}%{{opacity:1;transform:translateX({}px)}}{}%,100%{{opacity:0;transform:translateX({}px)}}", s - 1, w - head, e + 1, w - head));
        css += &record_motion(
            &format!("w{i}"),
            "ease-in-out",
            &format!(
                "0%,{}%{{opacity:0}}{}%,{}%{{opacity:1}}{}%,100%{{opacity:0}}",
                s - 2,
                s - 1,
                e + 1,
                e + 3
            ),
        );
        css += &record_motion(
            &format!("v{i}"),
            "ease-out",
            &format!(
                "0%,{}%{{opacity:.35}}{s}%,{e}%{{opacity:.7}}{}%,97%{{opacity:1}}100%{{opacity:.35}}",
                s - 1,
                e + 1
            ),
        );
        css += &record_motion(&format!("c{i}"), "ease-out", &format!("0%,{e}%{{opacity:0;transform:scale(.6)}}{}%{{opacity:1;transform:scale(1.15)}}{}%,97%{{opacity:1;transform:none}}100%{{opacity:0;transform:none}}", e + 2, e + 4));
        css += &record_motion(
            &format!("l{i}"),
            "ease-out",
            &format!(
                "0%,{}%{{opacity:0}}{e}%,{}%{{opacity:1}}{}%,100%{{opacity:0}}",
                e - 1,
                e + 2,
                e + 4
            ),
        );
        // A reading window slides over the bar; its leading edge is marked.
        scans.push_str(&format!(
            r#"<g class="a scan s{i}"><rect x="{x0}" y="{}" width="{head}" height="{}" rx="5" fill="{}" fill-opacity=".55" stroke="{c}" stroke-width="2"/><rect x="{}" y="{}" width="4" height="{}" fill="{c}"/></g>"#,
            y0 - 6.0,
            h + 12.0,
            t.paper[0],
            x0 + head - 4.0,
            y0 - 6.0,
            h + 12.0,
            c = t.blue[2]
        ));
        let (tip, tail) = (x0 + w, x0);
        arrows.push_str(&format!(
            r#"<g class="a arrow w{i}"><text x="{tail}" y="176" font-size="13" class="b" style="fill:{a}">view {} reads the same {record} B</text><path d="M{tail} 186H{}" stroke="{a}" stroke-width="2"/><polygon points="{tip},186 {},181 {},191" fill="{a}"/></g>"#,
            i + 1,
            tip - 10.0,
            tip - 10.0,
            tip - 10.0,
            a = t.accent
        ));
        let y = 96.0 + 46.0 * f64::from(i);
        // The view's score goes to its row.
        arrows.push_str(&format!(
            r#"<path class="a arrow l{i}" d="M{} {} C766 {} 764 {} 790 {}" stroke="{}" stroke-width="2" fill="none"/>"#,
            x0 + w,
            y0 + h / 2.0,
            y0 + h / 2.0,
            y + 19.0,
            y + 19.0,
            t.accent
        ));
        b.push_str(&format!(
            r#"<g class="a v{i}"><rect x="790" y="{y}" width="370" height="38" rx="10" fill="{bg}" stroke="{a}" stroke-width="2"/><text x="806" y="{}" font-size="16" class="b">view {}</text></g><g class="a c{i}"><rect x="880" y="{}" width="78" height="24" rx="6" fill="{bg}" stroke="{a}"/><text x="919" y="{}" font-size="13" text-anchor="middle">score</text><text x="975" y="{}" font-size="20" class="b" text-anchor="middle">=</text><rect x="992" y="{}" width="118" height="24" rx="6" fill="{bg}" stroke="{}" stroke-dasharray="4 3"/><text x="1051" y="{}" font-size="13" text-anchor="middle">copy's score</text><circle cx="1137" cy="{}" r="11" fill="{}"/><path d="M1131 {}l4 4 8-8" stroke="{bg}" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/></g>"#,
            y + 25.0,
            i + 1,
            y + 7.0,
            y + 24.0,
            y + 26.0,
            y + 7.0,
            t.muted,
            y + 24.0,
            y + 19.0,
            t.good,
            y + 19.0,
            bg = t.bg[0],
            a = t.accent
        ));
    }
    // Before and after the sweeps, and in the still picture, one line says the same.
    css += &record_motion(
        "id",
        "ease-out",
        "0%,53%{opacity:0}56%,97%{opacity:1}100%{opacity:0}",
    );
    b.push_str(&format!(
        r#"{scans}{arrows}<text class="a b id" x="{x0}" y="180" font-size="13" style="fill:{}">all four views read this one bar, without four copies</text>"#,
        t.accent
    ));
    // 4. The sizes it is compared with, to one scale: record, packed copies, layout.
    let longest = r.record.max(r.packed).max(r.frames) as f64;
    let (bx, reach) = (350.0, 700.0);
    b.push_str(r#"<text class="kick" x="40" y="334">SIZE, TO SCALE</text>"#);
    let rows = [
        (
            "one stored record",
            "what all four views read",
            r.record,
            1,
            true,
        ),
        (
            "four packed copies",
            "not stored; the record is kept once",
            r.packed,
            4,
            true,
        ),
        (
            "reference layout used for checking",
            "four materialized views",
            r.frames,
            4,
            false,
        ),
    ];
    for (k, (label, note, bytes, parts, values)) in rows.into_iter().enumerate() {
        let y = 346.0 + 44.0 * k as f64;
        let width = reach * bytes as f64 / longest;
        let part = width / f64::from(parts);
        let a = 54 + 5 * k;
        css += &record_motion(&format!("z{k}"), "ease-out", &format!("0%,{a}%{{opacity:1;transform:scaleX(0)}}{}%,97%{{opacity:1;transform:none}}100%{{opacity:0;transform:none}}", a + 8));
        css += &record_motion(
            &format!("n{k}"),
            "ease-out",
            &format!(
                "0%,{}%{{opacity:0}}{}%,97%{{opacity:1}}100%{{opacity:0}}",
                a + 6,
                a + 9
            ),
        );
        let mut fill = String::new();
        for p in 0..parts {
            let px = bx + part * f64::from(p);
            if values {
                let v = part * RECORD_VALUES as f64 / r.record as f64;
                fill.push_str(&format!(
                    r#"<rect x="{px:.1}" y="{y}" width="{v:.1}" height="26" fill="{}" stroke="{bg}"/><rect x="{:.1}" y="{y}" width="{:.1}" height="26" fill="{}" stroke="{bg}"/>"#,
                    t.blue[0],
                    px + v,
                    part - v,
                    t.amber[0],
                    bg = t.bg[0]
                ));
            } else {
                fill.push_str(&format!(
                    r#"<rect x="{px:.1}" y="{y}" width="{part:.1}" height="26" fill="{}" stroke="{}"/>"#,
                    t.paper[2], t.bg[0]
                ));
            }
        }
        let (look, dash) = if k == 1 {
            (".5", r#" stroke-dasharray="5 4""#)
        } else {
            ("1", "")
        };
        b.push_str(&format!(
            r#"<text x="40" y="{}" font-size="15" class="b">{label}</text><text x="40" y="{}" font-size="12" class="mut">{note}</text><g class="a grow z{k}"><g opacity="{look}">{fill}</g><rect class="size" x="{bx}" y="{y}" width="{width:.1}" height="26" rx="3" fill="none" stroke="{}" stroke-width="1.5"{dash}/></g><text class="a b n{k}" x="{:.1}" y="{}" font-size="16">{} B</text>"#,
            y + 14.0,
            y + 30.0,
            t.ink,
            bx + width + 12.0,
            y + 19.0,
            grouped(bytes)
        ));
    }
    // 5. The verdict of every recorded run; the limits stay in view throughout.
    css += &record_motion("vd", "ease-out", "0%,75%{opacity:0;transform:scale(.97)}79%,97%{opacity:1;transform:none}100%{opacity:0;transform:none}");
    css.push_str("@media (prefers-reduced-motion:reduce){.a{animation:none}}");
    b.push_str(&format!(
        r#"<g class="a vd"><rect x="40" y="488" width="1120" height="34" rx="17" fill="{}" stroke="{}"/><text class="mono" x="60" y="510" font-size="14"><tspan class="b" style="fill:{}">{} of {} recorded runs:</tspan> Q8_QUAD_EXACT=PASS · SEMANTIC_ACCURACY=NOT_MEASURED</text></g><text x="40" y="550" font-size="13" class="mut">Record payload only; process memory is additional. Four views are not four independent memories.</text>"#,
        t.bg[0],
        t.line,
        t.good,
        r.ok,
        r.n
    ));
    card(
        (1200, 576),
        t,
        ("One stored Q8 record, four exact views", &record_text(r)),
        &css,
        &b,
    )
}
fn record_picture(r: &Q8Runs) -> String {
    themed(
        "media/presentation/record-light.svg",
        "media/presentation/record-dark.svg",
        &record_text(r),
        1200,
    )
}
/// Points of a quad as SVG text.
fn quad(p: &[Point; 4]) -> String {
    p.iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
}
/// A point of the cube frame (±1 on each axis) turned by `yaw` about the vertical axis and seen from
/// above at `pitch`: screen position and depth towards the viewer.
fn turn(p: [f64; 3], yaw: f64, pitch: f64, c: Point, s: f64) -> (Point, f64) {
    let x = p[0] * yaw.cos() + p[2] * yaw.sin();
    let z = -p[0] * yaw.sin() + p[2] * yaw.cos();
    let y = p[1] * pitch.cos() - z * pitch.sin();
    let depth = p[1] * pitch.sin() + z * pitch.cos();
    ((c.0 + s * x, c.1 - s * y), depth)
}
/// Side faces and the top of a cube: corner indices (bit 0 = +x, bit 1 = +y, bit 2 = +z) and the outward normal.
const CUBE: [([usize; 4], [f64; 3]); 5] = [
    ([4, 5, 7, 6], [0.0, 0.0, 1.0]),
    ([5, 1, 3, 7], [1.0, 0.0, 0.0]),
    ([1, 0, 2, 3], [0.0, 0.0, -1.0]),
    ([0, 4, 6, 2], [-1.0, 0.0, 0.0]),
    ([6, 7, 3, 2], [0.0, 1.0, 0.0]),
];
fn corner(i: usize) -> [f64; 3] {
    let s = |bit: usize| if i & bit != 0 { 1.0 } else { -1.0 };
    [s(1), s(2), s(4)]
}
/// Face `f` of a cube of half-size `s` at every key angle: points and how much it faces the viewer.
fn face_frames(f: usize, yaws: &[f64], pitch: f64, c: Point, s: f64) -> Vec<(String, f64)> {
    let (idx, n) = CUBE[f];
    yaws.iter()
        .map(|&yaw| {
            let pts = idx.map(|k| turn(corner(k), yaw, pitch, c, s).0);
            let facing = turn(n, yaw, pitch, (0.0, 0.0), 1.0).1;
            (quad(&pts), facing)
        })
        .collect()
}
fn animate(attr: &str, values: &[String]) -> String {
    format!(
        r#"<animate attributeName="{attr}" dur="24s" repeatCount="indefinite" values="{}"/>"#,
        values.join(";")
    )
}
/// A turning glass cube: faces turned away are drawn first and faintly, faces towards the viewer on top.
/// Hidden faces fade out, so no depth sorting is needed for a convex cube.
fn glass_cube(
    yaws: &[f64],
    animated: bool,
    pitch: f64,
    c: Point,
    s: f64,
    colour: &str,
    strength: f64,
) -> (String, String) {
    let (mut back, mut front) = (String::new(), String::new());
    for f in 0..CUBE.len() {
        let frames = face_frames(f, yaws, pitch, c, s);
        let fade = |x: f64| (x * 5.0).clamp(0.0, 1.0);
        let op_front: Vec<String> = frames
            .iter()
            .map(|(_, v)| format!("{:.2}", fade(*v)))
            .collect();
        let op_back: Vec<String> = frames
            .iter()
            .map(|(_, v)| format!("{:.2}", 0.5 * fade(-*v)))
            .collect();
        let pts: Vec<String> = frames.iter().map(|(p, _)| p.clone()).collect();
        let (anim_f, anim_b) = if animated {
            (
                animate("points", &pts) + &animate("opacity", &op_front),
                animate("points", &pts) + &animate("opacity", &op_back),
            )
        } else {
            (String::new(), String::new())
        };
        back.push_str(&format!(r#"<polygon points="{}" fill="{colour}" fill-opacity="{:.2}" stroke="{colour}" stroke-width="1.5" stroke-dasharray="5 6" opacity="{}">{anim_b}</polygon>"#, pts[0], 0.05 * strength, op_back[0]));
        let fill = if f == 4 { 0.2 } else { 0.12 + 0.04 * f as f64 };
        front.push_str(&format!(r#"<polygon points="{}" fill="{colour}" fill-opacity="{:.2}" stroke="{colour}" stroke-width="2.2" stroke-linejoin="round" opacity="{}">{anim_f}</polygon>"#, pts[0], fill * strength, op_front[0]));
    }
    (back, front)
}
/// The opening logo: a large glass cube of record cells that turns slowly around the accent core.
/// Drawn, not recorded; `animated` false gives the still.
fn hero(dark: bool, animated: bool) -> String {
    let t = theme(dark);
    let (c, s, pitch) = ((320.0, 206.0), 104.0, 0.42);
    let start = 0.62;
    let yaws: Vec<f64> = (0..=24)
        .map(|k| start + k as f64 * std::f64::consts::TAU / 24.0)
        .collect();
    let yaws = if animated { yaws } else { vec![start] };
    let mut b = format!(
        r#"<defs><radialGradient id="glow"><stop offset="0" stop-color="{0}" stop-opacity=".32"/><stop offset="1" stop-color="{0}" stop-opacity="0"/></radialGradient></defs><ellipse cx="{1}" cy="{2}" rx="250" ry="210" fill="url(#glow)"/>"#,
        t.accent, c.0, c.1
    );
    let bob = if animated {
        r#"<animateTransform attributeName="transform" type="translate" dur="6s" repeatCount="indefinite" values="0 0;0 -9;0 0" calcMode="spline" keySplines=".45 0 .55 1;.45 0 .55 1"/>"#
    } else {
        ""
    };
    let (back, front) = glass_cube(&yaws, animated, pitch, c, s, t.accent, 1.0);
    b.push_str(&format!(r#"<g>{bob}{back}"#));
    // Three layers of record cells around the core; they turn with the cube.
    let step = 2.0 / 3.0;
    let cell = 0.13 * s;
    for layer in [-1.0, 0.0, 1.0] {
        for i in [-1.0, 0.0, 1.0] {
            for j in [-1.0, 0.0, 1.0] {
                if (i, layer, j) == (0.0, 0.0, 0.0) {
                    continue;
                }
                let p = [i * step, layer * step, j * step];
                let pos: Vec<Point> = yaws
                    .iter()
                    .map(|&yaw| turn(p, yaw, pitch, c, s).0)
                    .collect();
                let hit = (i, layer, j) == (1.0, 1.0, 1.0);
                let colour = if hit { t.accent } else { t.paper[0] };
                let anim = if animated {
                    let xs: Vec<String> = pos
                        .iter()
                        .map(|q| format!("{:.1}", q.0 - cell / 2.0))
                        .collect();
                    let ys: Vec<String> = pos
                        .iter()
                        .map(|q| format!("{:.1}", q.1 - cell / 2.0))
                        .collect();
                    animate("x", &xs) + &animate("y", &ys)
                } else {
                    String::new()
                };
                b.push_str(&format!(r#"<rect x="{:.1}" y="{:.1}" width="{cell:.1}" height="{cell:.1}" rx="2.5" fill="{colour}" stroke="{}" stroke-width="1.2" opacity=".85">{anim}</rect>"#, pos[0].0 - cell / 2.0, pos[0].1 - cell / 2.0, if hit { t.accent } else { t.line }));
            }
        }
    }
    let (core_back, core_front) = glass_cube(&yaws, animated, pitch, c, 0.22 * s, t.blue[1], 3.0);
    b.push_str(&core_back);
    b.push_str(&core_front);
    b.push_str(&front);
    b.push_str("</g>");
    b.push_str(&format!(r#"<text x="640" y="196" font-size="86" class="b" letter-spacing="2">GEL RAM</text><text x="644" y="248" font-size="32">Knowledge printed, not trained</text><text x="644" y="296" font-size="22" style="fill:{}">One record. Four exact views. 1,152 bytes.</text><text x="644" y="336" font-size="15" class="mut">Read back exactly · can say UNKNOWN · a kill loses nothing confirmed</text>"#, t.accent));
    let desc = if animated {
        "Animated logo: a glass cube of record cells turns slowly around an accent core."
    } else {
        "The logo, still: a glass cube of record cells around an accent core."
    };
    result_card(
        dark,
        440,
        ("GEL RAM — Knowledge printed, not trained", desc),
        &b,
    )
}
/// The recorded crash series the crash card is drawn from.
const CRASH_FILE: &str = "docs/evidence-crash/crash-series-linux.txt";
/// The columns of one trial row, in the order the series writes them.
const CRASH_COLUMNS: &str = "# trial\tkill_ms\tacknowledged\tacknowledged_lost\tunacknowledged_complete\tpartial\ttemporaries\tresumed\treopen_us";
/// One trial of the crash series: when the writer was killed and what it had
/// been told was saved by then.
struct CrashTrial {
    kill_ms: u32,
    acknowledged: usize,
    /// A temporary file or a complete unacknowledged snapshot was left.
    mid_publication: bool,
}
/// What one recorded crash series states, checked against its own rows.
struct CrashSeries {
    /// The day of the run and its file system; the host and CPU are not shown.
    date: String,
    file_system: String,
    /// Snapshots one uninterrupted run saves, and the window the kill is drawn from.
    documents: usize,
    window_ms: u32,
    trials: Vec<CrashTrial>,
    acknowledged: usize,
    lost: usize,
    partial: usize,
    resumed: usize,
    mid_publication: usize,
}
impl CrashSeries {
    fn read() -> Result<CrashSeries> {
        CrashSeries::from_text(&fs::read_to_string(CRASH_FILE)?)
    }
    /// Refuses to give numbers unless every row parses, the trials are numbered
    /// 1..=n, and the one PASS line states exactly what the rows add up to.
    fn from_text(text: &str) -> Result<CrashSeries> {
        let (mut host, mut setup, mut columns, mut summary) = (None, None, false, Vec::new());
        let mut rows: Vec<[&str; 9]> = Vec::new();
        for line in text.lines() {
            if let Some(h) = line.strip_prefix("# host: ") {
                host = host.or(Some(h));
            } else if let Some(s) = line.strip_prefix("# gel-evidence crash series: ") {
                setup = Some(s);
            } else if line == CRASH_COLUMNS {
                columns = true;
            } else if line.starts_with("CRASH_SERIES=") {
                summary.push(line);
            } else if line.trim().is_empty() || line.starts_with(' ') || line.starts_with('#') {
                // Indented lines are the build output of `cargo run`.
            } else {
                let f: Vec<&str> = line.split('\t').collect();
                rows.push(
                    f.try_into()
                        .map_err(|_| format!("unexpected crash series row: {line}"))?,
                );
            }
        }
        if !columns {
            return Err("unexpected crash series columns".into());
        }
        // The first header line: kernel · CPU · file system · rustc · revision · time.
        let host: Vec<&str> = host.ok_or("no host line")?.split(" · ").collect();
        let (date, file_system) = match host[..] {
            [kernel, _, fs, _, _, time] if kernel.starts_with("Linux ") => {
                (time.get(..10).unwrap_or(""), fs)
            }
            _ => return Err("unexpected crash series host line".into()),
        };
        let b = date.as_bytes();
        if b.len() != 10
            || !b.iter().enumerate().all(|(i, c)| match i {
                4 | 7 => *c == b'-',
                _ => c.is_ascii_digit(),
            })
            || file_system.is_empty()
            || !file_system.bytes().all(|c| c.is_ascii_alphanumeric())
        {
            return Err("unexpected date or file system in the host line".into());
        }
        let setup = setup.ok_or("no crash series setup line")?;
        let between = |from: &str, to: &str| -> Result<usize> {
            let rest = setup.split_once(from).ok_or("unexpected setup line")?.1;
            let found = rest.split_once(to).ok_or("unexpected setup line")?.0;
            Ok(found.parse()?)
        };
        let documents = between("", " documents")?;
        let window_ms = u32::try_from(between("kill after 1..=", " ms")?)?;
        let mut trials = Vec::new();
        let (mut acknowledged, mut lost, mut partial, mut resumed) = (0, 0, 0, 0);
        for (i, f) in rows.iter().enumerate() {
            let n = |k: usize| -> Result<usize> { Ok(f[k].parse::<usize>()?) };
            let kill_ms = f[1].parse::<u32>()?;
            if n(0)? != i + 1 || kill_ms > window_ms || n(2)? > documents {
                return Err(format!("unexpected crash series trial {}", f[0]).into());
            }
            if f[8] != "-" {
                f[8].parse::<u64>()?;
            }
            acknowledged += n(2)?;
            lost += n(3)?;
            partial += n(5)?;
            resumed += match f[7] {
                "true" => 1,
                "false" => 0,
                _ => return Err(format!("unexpected resumed value: {}", f[7]).into()),
            };
            trials.push(CrashTrial {
                kill_ms,
                acknowledged: n(2)?,
                mid_publication: n(4)? + n(6)? > 0,
            });
        }
        let mid_publication = trials.iter().filter(|t| t.mid_publication).count();
        let [line] = summary[..] else {
            return Err("expected exactly one CRASH_SERIES line".into());
        };
        let words: Vec<&str> = line.split(' ').collect();
        let value = |key: &str| -> Result<usize> {
            let found = words
                .iter()
                .find_map(|w| w.strip_prefix(key)?.strip_prefix('='))
                .ok_or(format!("the summary has no {key}"))?;
            Ok(found.parse::<usize>()?)
        };
        if words.first() != Some(&"CRASH_SERIES=PASS") {
            return Err("the crash series did not pass".into());
        }
        let stated = [
            value("trials")?,
            value("acknowledged")?,
            value("acknowledged_lost")?,
            value("partial")?,
            value("resumed")?,
            value("killed_mid_publication")?,
        ];
        let counted = [
            trials.len(),
            acknowledged,
            lost,
            partial,
            resumed,
            mid_publication,
        ];
        if stated != counted {
            return Err(
                format!("the summary {stated:?} disagrees with the rows {counted:?}").into(),
            );
        }
        // Every point is drawn as a survived trial, so a PASS must mean exactly that.
        if trials.is_empty() || lost != 0 || partial != 0 || resumed != trials.len() {
            return Err("a PASS line over a trial that lost, tore or did not resume".into());
        }
        Ok(CrashSeries {
            date: date.to_string(),
            file_system: file_system.to_string(),
            documents,
            window_ms,
            trials,
            acknowledged,
            lost,
            partial,
            resumed,
            mid_publication,
        })
    }
}
/// What the crash card says, as its description and as the README alternative text.
fn crash_text(c: &CrashSeries) -> String {
    format!("One recorded crash series on the author's Linux machine ({}, {}): in each of {} trials a collection growing to {} snapshots was killed with SIGKILL at a random moment within {} ms. {} snapshots were acknowledged before the kills; {} were lost and {} were partial, and {} of {} trials resumed to the uninterrupted result. {} kills came in the middle of a publication. A process kill, not a power cut: power-loss durability is not established.", c.file_system, c.date, c.trials.len(), c.documents, c.window_ms, grouped(c.acknowledged), c.lost, c.partial, c.resumed, c.trials.len(), c.mid_publication)
}
/// Every trial of the recorded crash series as one point: when the writer was
/// killed and how many snapshots it had acknowledged by then. The points appear
/// in the order of the series, then the totals the summary line states.
fn crash_card(dark: bool, c: &CrashSeries) -> String {
    let t = theme(dark);
    let n = c.trials.len();
    let mut b = format!(
        r#"<text class="kick" x="40" y="44">CRASH SERIES · KILLED WHILE SAVING</text><text x="40" y="80" font-size="25" class="b">{n} kills while saving. {} acknowledged snapshots lost.</text>"#,
        c.lost
    );
    // The plot: kill time across, acknowledged snapshots up.
    let (x0, x1, y0, y1) = (100.0, 740.0, 324.0, 120.0);
    let px = |ms: u32| x0 + (x1 - x0) * f64::from(ms) / f64::from(c.window_ms);
    let py = |k: usize| y0 - (y0 - y1) * k as f64 / c.documents as f64;
    let step = (c.documents / 4).max(1);
    for k in (0..=c.documents).step_by(step) {
        let y = py(k);
        b.push_str(&format!(
            r#"<path d="M{x0} {y:.1}H{x1}" stroke="{}" stroke-width="1"/><text class="mono" x="{}" y="{:.1}" text-anchor="end" font-size="12">{k}</text>"#,
            t.line,
            x0 - 10.0,
            y + 4.0
        ));
    }
    for ms in (0..=c.window_ms).step_by(50) {
        b.push_str(&format!(
            r#"<text class="mono" x="{:.1}" y="344" text-anchor="middle" font-size="12">{ms}</text>"#,
            px(ms)
        ));
    }
    b.push_str(&format!(
        r#"<path d="M{x0} {y0}H{x1}M{x0} {y0}V{y1}" stroke="{ink}" stroke-width="1.5" fill="none"/><path d="M{x0} {top:.1}H{x1}" stroke="{ink}" stroke-width="1.5" stroke-dasharray="5 5" fill="none"/><text x="{:.1}" y="{:.1}" font-size="13" class="mut">all {} snapshots saved</text><text x="{x1}" y="366" font-size="13" class="mut" text-anchor="end">kill time, ms after the writer started (window {} ms)</text><text x="0" y="0" font-size="13" class="mut" text-anchor="middle" transform="translate(48 {:.1}) rotate(-90)">snapshots acknowledged</text>"#,
        x0 + 8.0,
        py(c.documents) - 8.0,
        c.documents,
        c.window_ms,
        (y0 + y1) / 2.0,
        ink = t.ink,
        top = py(c.documents)
    ));
    // Legend in the empty lower right: a late kill always finds many saves.
    b.push_str(&format!(
        r#"<circle cx="492" cy="254" r="5" fill="{g}"/><text x="506" y="259" font-size="13">killed mid-publication ({})</text><circle cx="492" cy="278" r="4.2" fill="none" stroke="{g}" stroke-width="1.8"/><text x="506" y="283" font-size="13">killed between saves ({})</text><text x="486" y="307" font-size="13" class="mut">every trial: no acknowledged snapshot lost</text>"#,
        c.mid_publication,
        n - c.mid_publication,
        g = t.good
    ));
    // The parser refuses a PASS line over a lost, partial or unresumed trial, so
    // every point is drawn as survived; filled when the kill hit a publication.
    for (i, trial) in c.trials.iter().enumerate() {
        let paint = if trial.mid_publication {
            format!(r#"r="5" fill="{}""#, t.good)
        } else {
            format!(
                r#"r="4.2" fill="none" stroke="{}" stroke-width="1.8""#,
                t.good
            )
        };
        b.push_str(&format!(
            r#"<circle class="pt" style="animation-delay:{:.2}s" cx="{:.1}" cy="{:.1}" {paint}/>"#,
            0.2 + 2.4 * i as f64 / n as f64,
            px(trial.kill_ms),
            py(trial.acknowledged)
        ));
    }
    // The totals, in two columns of three tiles.
    let tiles = [
        (n.to_string(), t.accent, "trials, each", "killed once"),
        (
            grouped(c.acknowledged),
            t.accent,
            "snapshots acknowledged",
            "before the kills",
        ),
        (c.lost.to_string(), t.good, "acknowledged", "snapshots lost"),
        (
            c.partial.to_string(),
            t.good,
            "partial snapshots",
            "found after a kill",
        ),
        (
            c.resumed.to_string(),
            t.accent,
            "resumed to the",
            "uninterrupted result",
        ),
        (
            c.mid_publication.to_string(),
            t.accent,
            "killed in the middle",
            "of a publication",
        ),
    ];
    for (k, (num, colour, l1, l2)) in tiles.iter().enumerate() {
        let (x, y) = (
            790.0 + 190.0 * (k % 2) as f64,
            104.0 + 90.0 * (k / 2) as f64,
        );
        b.push_str(&format!(
            r#"<g class="cnt" style="animation-delay:{:.2}s"><rect x="{x}" y="{y}" width="176" height="80" rx="12" fill="{}" fill-opacity=".07" stroke="{}"/><text x="{}" y="{}" font-size="28" class="b" style="fill:{colour}">{num}</text><text x="{}" y="{}" font-size="13">{l1}</text><text x="{}" y="{}" font-size="13">{l2}</text></g>"#,
            2.8 + 0.15 * k as f64,
            t.accent,
            t.line,
            x + 16.0,
            y + 34.0,
            x + 16.0,
            y + 54.0,
            x + 16.0,
            y + 70.0
        ));
    }
    b.push_str(&format!(
        r#"<text x="40" y="404" font-size="13" class="mut">One recorded run on the author's Linux machine ({}, {}) · a process kill (SIGKILL), not a power cut: power-loss durability is not established</text>"#,
        c.file_system, c.date
    ));
    let t_style = format!(".pt{{animation:pop .45s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:center}}.cnt{{animation:fade .8s ease-out both}}@keyframes pop{{from{{opacity:0;transform:scale(.2)}}to{{opacity:1;transform:none}}}}@keyframes fade{{from{{opacity:0}}to{{opacity:1}}}}@media (prefers-reduced-motion:reduce){{.pt,.cnt{{animation:none}}}}{}", RESULT_STYLE.replace("MUTED", t.muted));
    card(
        (1200, 424),
        t,
        ("Crash series: killed while saving", &crash_text(c)),
        &t_style,
        &b,
    )
}
fn crash_picture(c: &CrashSeries) -> String {
    themed(
        "media/presentation/crash-light.svg",
        "media/presentation/crash-dark.svg",
        &crash_text(c),
        1200,
    )
}
/// The timing set the speed card is drawn from, the one the release notes cite
/// against v0.6.1: A and A2 are two checkouts of v0.6.1, B the new tree.
const SPEED_DIR: &str = "docs/speed-results/2026-10-20-against-v0.6.1";
/// The revision of `main` the set's B tree equals, as its README states it.
const SPEED_B: &str = "876247f";
/// One timed case: the two ratios as its summary prints them, checked against
/// the medians of A, A2 and B when it is read.
struct SpeedCase {
    name: String,
    /// A2/A: the noise of two copies of the same tree.
    aa: f64,
    /// A/B: above 1 the new tree is faster.
    gain: f64,
    runs: u32,
}
/// Where a case falls against its own A/A band.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Faster,
    Slower,
    Within,
}
impl SpeedCase {
    /// Parses the fields after `SPEED_AB `, in their fixed order, and refuses a
    /// ratio that does not round from the medians to its printed digits.
    fn parse(fields: &str) -> Result<SpeedCase> {
        const KEYS: [&str; 7] = ["case", "a_ns", "a2_ns", "b_ns", "aa", "gain", "runs"];
        let words: Vec<&str> = fields.split(' ').collect();
        let v: Vec<&str> = words
            .iter()
            .zip(KEYS)
            .filter_map(|(w, k)| w.strip_prefix(k)?.strip_prefix('='))
            .collect();
        if words.len() != KEYS.len() || v.len() != KEYS.len() {
            return Err("unexpected SPEED_AB fields".into());
        }
        if !plain_name(v[0]) {
            return Err("unexpected case name".into());
        }
        let count = |s: &str| -> Result<u64> {
            match s.parse::<u64>() {
                Ok(n) if n > 0 && s.bytes().all(|b| b.is_ascii_digit()) => Ok(n),
                _ => Err(format!("unexpected count {s}").into()),
            }
        };
        let (a, a2, b) = (count(v[1])?, count(v[2])?, count(v[3])?);
        Ok(SpeedCase {
            name: v[0].to_string(),
            aa: printed_ratio(v[4], a2, a)?,
            gain: printed_ratio(v[5], a, b)?,
            runs: u32::try_from(count(v[6])?)?,
        })
    }
    /// The A/A band: from the smaller of A2/A and A/A2 to the larger.
    fn noise(&self) -> (f64, f64) {
        let (x, y) = (self.aa, 1.0 / self.aa);
        (x.min(y), x.max(y))
    }
    fn side(&self) -> Side {
        let (lo, hi) = self.noise();
        if self.gain > hi {
            Side::Faster
        } else if self.gain < lo {
            Side::Slower
        } else {
            Side::Within
        }
    }
}
/// Example and case names: lower-case ASCII, digits and single underscores.
fn plain_name(name: &str) -> bool {
    let ok = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_';
    !name.is_empty() && name.bytes().all(ok) && !name.contains("__")
}
/// A ratio as a summary prints it, refused unless `num / den` rounds to it:
/// within half a unit of its last printed digit.
fn printed_ratio(text: &str, num: u64, den: u64) -> Result<f64> {
    let digits = text.split_once('.').map_or(0, |(_, d)| d.len());
    let value: f64 = text.parse()?;
    let half = 0.5 * 10f64.powi(-(digits as i32));
    let exact = num as f64 / den as f64;
    if digits == 0
        || !text.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        || (exact - value).abs() > half * (1.0 + 1e-9)
    {
        return Err(format!("{text} does not round from {num}/{den}").into());
    }
    Ok(value)
}
/// The cases of one summary.txt and whether its three digests agree. Refuses an
/// unknown line, a repeated case, digests that are not exactly A, A2 and B, and
/// a missing, repeated or contradicting verdict.
fn speed_summary(text: &str) -> Result<(Vec<SpeedCase>, bool)> {
    let (mut cases, mut checks, mut verdicts) = (Vec::<SpeedCase>::new(), Vec::new(), Vec::new());
    for line in text.lines() {
        if let Some(fields) = line.strip_prefix("SPEED_AB ") {
            let case = SpeedCase::parse(fields).map_err(|e| format!("{e}: {line}"))?;
            if cases.iter().any(|c| c.name == case.name) {
                return Err(format!("repeated case: {line}").into());
            }
            cases.push(case);
        } else if let Some(check) = line.strip_prefix("SPEED_AB_CHECK tree=") {
            checks.push(check);
        } else if let Some(verdict) = line.strip_prefix("SPEED_AB_VERDICT=") {
            verdicts.push(verdict);
        } else {
            return Err(format!("unexpected summary line: {line}").into());
        }
    }
    let hex = |s: &&str| s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    let mut digests = Vec::new();
    for (check, tree) in checks.iter().zip(["a", "a2", "b"]) {
        let digest = check
            .strip_prefix(tree)
            .and_then(|r| r.strip_prefix(" sha256="))
            .filter(hex)
            .ok_or(format!("unexpected digest line: tree={check}"))?;
        digests.push(digest);
    }
    if cases.is_empty() || checks.len() != 3 || digests.len() != 3 {
        return Err("a summary needs its cases and the digests of A, A2 and B".into());
    }
    let same = digests.iter().all(|d| *d == digests[0]);
    match verdicts.as_slice() {
        ["SAME_OUTPUT"] if same => Ok((cases, true)),
        ["DIFFERENT_OUTPUT"] if !same => Ok((cases, false)),
        _ => Err(format!("verdict {verdicts:?} does not match the digests").into()),
    }
}
/// One timing example of the set: its cases and whether A, A2 and B printed the
/// same output digest, or `None` when it has only a manifest. `note` is the
/// README's sentence that explains a differing digest or a missing summary.
struct SpeedExample {
    name: String,
    timed: Option<(Vec<SpeedCase>, bool)>,
    note: Option<String>,
}
impl SpeedExample {
    fn cases(&self) -> &[SpeedCase] {
        self.timed.as_ref().map_or(&[][..], |t| &t.0[..])
    }
}
/// The timing set the speed card draws: the machine and its load as the set's
/// README states them, and every example directory in name order.
struct SpeedSet {
    machine: String,
    load: String,
    examples: Vec<SpeedExample>,
}
impl SpeedSet {
    fn read() -> Result<SpeedSet> {
        let readme = fs::read_to_string(format!("{SPEED_DIR}/README.md"))?;
        let mut names = Vec::new();
        for entry in fs::read_dir(SPEED_DIR)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        let mut examples = Vec::new();
        for name in names {
            let dir = format!("{SPEED_DIR}/{name}");
            if !Path::new(&format!("{dir}/manifest.txt")).is_file() {
                return Err(format!("{dir} has no manifest").into());
            }
            let summary = format!("{dir}/summary.txt");
            let text = if Path::new(&summary).try_exists()? {
                Some(fs::read_to_string(summary)?)
            } else {
                None
            };
            examples.push((name, text));
        }
        SpeedSet::from_texts(&readme, &examples)
    }
    /// Refuses to give a set unless every summary adds up, the README states the
    /// machine, its load, no pinning and the B revision, and one README sentence
    /// explains each example without a summary or with differing digests.
    fn from_texts(readme: &str, examples: &[(String, Option<String>)]) -> Result<SpeedSet> {
        let plain = |s: &str| s.is_ascii() && !s.contains(['<', '>', '&', '"', '`']);
        let row = |key: &str| -> Result<String> {
            readme
                .lines()
                .find_map(|l| {
                    l.strip_prefix("| ")?
                        .strip_prefix(key)?
                        .strip_prefix(" | ")?
                        .strip_suffix(" |")
                })
                .filter(|v| plain(*v))
                .map(str::to_string)
                .ok_or_else(|| format!("the set README has no plain {key} row").into())
        };
        let (machine, load, pinning) = (row("Machine")?, row("Load")?, row("Pinning")?);
        if !pinning.starts_with("none") || !readme.contains(&format!("`main` at `{SPEED_B}`")) {
            return Err("the set README no longer matches the card's wording".into());
        }
        let flat = readme.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut out = Vec::new();
        for (name, summary) in examples {
            if !plain_name(name) {
                return Err(format!("unexpected example name {name}").into());
            }
            let timed = match summary {
                Some(text) => Some(speed_summary(text).map_err(|e| format!("{name}: {e}"))?),
                None => None,
            };
            let note = if matches!(timed, Some((_, true))) {
                None
            } else {
                let start = flat
                    .find(&format!("`{name}` "))
                    .ok_or(format!("the set README does not explain {name}"))?;
                let len = flat[start..]
                    .find(". ")
                    .map_or(flat.len() - start, |e| e + 1);
                let sentence = flat[start..start + len].replace('`', "");
                if !plain(&sentence) || !sentence.ends_with('.') {
                    return Err(format!("unexpected note on {name}").into());
                }
                Some(sentence)
            };
            out.push(SpeedExample {
                name: name.clone(),
                timed,
                note,
            });
        }
        if out.iter().all(|e| e.timed.is_none()) {
            return Err("no timed example in the set".into());
        }
        Ok(SpeedSet {
            machine,
            load,
            examples: out,
        })
    }
    fn cases(&self) -> impl Iterator<Item = &SpeedCase> {
        self.examples.iter().flat_map(|e| e.cases())
    }
    /// Cases faster beyond their A/A band, slower beyond it, and inside it.
    fn sides(&self) -> [usize; 3] {
        let n = |side| self.cases().filter(|c| c.side() == side).count();
        [n(Side::Faster), n(Side::Slower), n(Side::Within)]
    }
    /// Timed examples, and those that printed the same digest in all three trees.
    fn verdicts(&self) -> (usize, usize) {
        let timed = self.examples.iter().filter(|e| e.timed.is_some());
        let same = timed.clone().filter(|e| matches!(e.timed, Some((_, true))));
        (timed.count(), same.count())
    }
}
/// A ratio as the card prints it: one decimal from 10 up, two below.
fn ratio(v: f64) -> String {
    if v >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}
/// What the speed card says, as its description and as the README alternative text.
fn speed_text(s: &SpeedSet) -> String {
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let sha = s
        .examples
        .iter()
        .find(|e| e.name == "speed_sha256")
        .map_or(String::new(), |e| {
            let gains: Vec<String> = e.cases().iter().map(|c| ratio(c.gain)).collect();
            format!(" (SHA-256 alone at {})", gains.join(" and "))
        });
    let notes: Vec<&str> = s
        .examples
        .iter()
        .filter_map(|e| e.note.as_deref())
        .collect();
    let set = SPEED_DIR.rsplit('/').next().unwrap_or(SPEED_DIR);
    format!("Timing set {set}: v0.6.1 against main at {SPEED_B}, the first v0.7 speed package. {} timed cases in {timed} timing examples, each drawn as the v0.6.1 median time divided by the new one on a log scale centred at 1, over the band of its A/A noise. {faster} are faster beyond their A/A band, {slower} slower beyond it{sha} and {within} inside it. {same} of {timed} timed examples printed the same output digest in all three trees. {} Machine: {}; {}; not pinned to cores. Ratios of different cases are not comparable with each other; the output digests, not the timings, are the claim.", s.cases().count(), notes.join(" "), s.machine, s.load)
}
const SPEED_STYLE: &str = ".mut{fill:MUTED}.b{font-weight:700}.bar{animation:grow .9s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box}.gr{transform-origin:left}.gl{transform-origin:right}.fd{animation:fade .6s ease-out both}@keyframes grow{from{transform:scaleX(0)}to{transform:none}}@keyframes fade{from{opacity:0}to{opacity:1}}@media (prefers-reduced-motion:reduce){.bar,.fd{animation:none}}";
/// Every timed case of the set as one bar on a log scale centred at 1: to the
/// right where the new tree is faster, to the left where it is slower, over the
/// grey band of its A/A noise. Examples in name order fill two columns, each
/// marked with its output verdict; nothing is aggregated or left out.
fn speed_card(dark: bool, s: &SpeedSet) -> String {
    let t = theme(dark);
    let (row, head, top) = (14.0, 20.0, 232.0);
    let colour = |side| match side {
        Side::Faster => t.blue[1],
        Side::Slower => t.red[1],
        Side::Within => t.muted,
    };
    let tall = |e: &SpeedExample| head + row * e.cases().len().max(1) as f64;
    // Two columns in name order, split where the taller column is shortest.
    let total: f64 = s.examples.iter().map(tall).sum();
    let (mut split, mut left) = ((0, f64::INFINITY), 0.0_f64);
    for k in 0..=s.examples.len() {
        if left.max(total - left) < split.1 {
            split = (k, left.max(total - left));
        }
        left += s.examples.get(k).map_or(0.0, tall);
    }
    // A log2 axis in whole octaves, wide enough for every ratio and every band.
    let (mut lo, mut hi) = (0.5_f64, 2.0_f64);
    for c in s.cases() {
        let (a, b) = c.noise();
        lo = lo.min(a).min(c.gain);
        hi = hi.max(b).max(c.gain);
    }
    let (lo, hi) = (lo.log2().floor() as i32, hi.log2().ceil() as i32);
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let span = |runs: &mut dyn Iterator<Item = u32>| {
        let (a, b) = runs.fold((u32::MAX, 0), |(a, b), r| (a.min(r), b.max(r)));
        if a == b {
            a.to_string()
        } else {
            format!("{a} to {b}")
        }
    };
    let check = |x: f64, y: f64| {
        format!(
            r#"<path d="M{x:.1} {y:.1}l3.5 3.5 6.5-7.5" fill="none" stroke="{}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
            t.good
        )
    };
    let set = SPEED_DIR.rsplit('/').next().unwrap_or(SPEED_DIR);
    let mut b = format!(
        r#"<text class="kick" x="40" y="46">SPEED · v0.6.1 AGAINST main AT {SPEED_B} · SET {set}</text><text x="40" y="82" font-size="24" class="b">{} timed cases in {timed} timing examples: v0.6.1 time divided by the new time</text><text class="mono" x="40" y="108" font-size="12.5">A, A2: two checkouts of v0.6.1 · B: main at {SPEED_B}, the first v0.7 speed package · medians of {} runs per case</text><text class="mono" x="40" y="128" font-size="12.5">Machine: {} · not pinned to cores</text><text class="mono" x="40" y="148" font-size="12.5">Load: {}</text>"#,
        s.cases().count(),
        span(&mut s.cases().map(|c| c.runs)),
        s.machine,
        s.load
    );
    b.push_str(&format!(
        r#"<rect x="40" y="173" width="24" height="6" rx="1.5" fill="{}"/><text x="72" y="181" font-size="13">faster beyond its A/A band: {faster}</text><rect x="300" y="173" width="24" height="6" rx="1.5" fill="{}"/><text x="332" y="181" font-size="13">slower beyond it: {slower}</text><rect x="476" y="173" width="24" height="6" rx="1.5" fill="{}"/><text x="508" y="181" font-size="13">inside it: {within}</text><rect x="604" y="171" width="30" height="10" rx="2" fill="{}" opacity=".3"/><path d="M604 171v10M634 171v10" stroke="{}" stroke-opacity=".5" stroke-width="1.2"/><text x="642" y="181" font-size="13">A/A noise band</text>{}<text x="788" y="181" font-size="13">same output digest in A, A2 and B: {same} of {timed}</text>"#,
        t.blue[1],
        t.red[1],
        t.muted,
        t.muted,
        t.ink,
        check(770.0, 176.0)
    ));
    let mut notes: Vec<&str> = Vec::new();
    let mut bottom = top;
    for (col, examples) in [&s.examples[..split.0], &s.examples[split.0..]]
        .into_iter()
        .enumerate()
    {
        let x0 = 32.0 + 584.0 * col as f64;
        let (p0, p1) = (x0 + 218.0, x0 + 462.0);
        let x = |v: f64| p0 + (v.log2() - f64::from(lo)) / f64::from(hi - lo) * (p1 - p0);
        let end = top + examples.iter().map(tall).sum::<f64>();
        bottom = bottom.max(end);
        b.push_str(&format!(
            r#"<g class="mut b" font-size="11"><text x="{:.0}" y="208">case</text><text x="{:.1}" y="208" text-anchor="end">slower</text><text x="{:.1}" y="208">faster</text><text x="{:.0}" y="208" text-anchor="end">gain</text><text x="{:.0}" y="208" text-anchor="end">A/A</text></g>"#,
            x0 + 16.0,
            x(1.0) - 6.0,
            x(1.0) + 6.0,
            x0 + 510.0,
            x0 + 552.0
        ));
        for k in lo..=hi {
            let v = 2f64.powi(k);
            let (stroke, width) = if k == 0 { (t.ink, 1.4) } else { (t.line, 1.0) };
            b.push_str(&format!(
                r#"<path d="M{gx:.1} 228V{end:.0}" stroke="{stroke}" stroke-width="{width}"/><text class="mono" x="{gx:.1}" y="222" text-anchor="middle" font-size="10">{v}</text>"#,
                gx = x(v)
            ));
        }
        // Rows of one column appear one after another, both columns at once.
        let (mut y, mut k) = (top, 0.0);
        for e in examples {
            let note = e.note.as_deref().map_or(String::new(), |n| {
                notes.push(n);
                format!(" · note {}", notes.len())
            });
            let (icon, right) = match &e.timed {
                Some((cases, true)) => (
                    check(x0, y + 11.0),
                    format!("{} runs per case", span(&mut cases.iter().map(|c| c.runs))),
                ),
                Some((cases, false)) => (
                    format!(
                        r##"<circle cx="{:.1}" cy="{:.1}" r="6" fill="{}"/><text x="{:.1}" y="{:.1}" text-anchor="middle" font-size="10" class="b" style="fill:#1b1f24">!</text>"##,
                        x0 + 5.0,
                        y + 10.0,
                        t.amber[1],
                        x0 + 5.0,
                        y + 13.6
                    ),
                    format!(
                        "{} runs per case · output digests differ{note}",
                        span(&mut cases.iter().map(|c| c.runs))
                    ),
                ),
                None => (
                    format!(
                        r#"<circle cx="{:.1}" cy="{:.1}" r="5.5" fill="none" stroke="{m}" stroke-width="1.6"/><path d="M{:.1} {:.1}h6" stroke="{m}" stroke-width="1.6"/>"#,
                        x0 + 5.0,
                        y + 10.0,
                        x0 + 2.0,
                        y + 10.0,
                        m = t.muted
                    ),
                    format!("not built in v0.6.1{note}"),
                ),
            };
            b.push_str(&format!(
                r#"<g class="fd" style="animation-delay:{:.2}s"><path d="M{x0:.0} {:.0}H{:.0}" stroke="{}"/>{icon}<text x="{:.0}" y="{:.0}" font-size="12.5" class="b">{}</text><text x="{:.0}" y="{:.0}" font-size="10.5" class="mut" text-anchor="end">{right}</text></g>"#,
                0.1 + 0.014 * k,
                y + 1.0,
                x0 + 552.0,
                t.line,
                x0 + 16.0,
                y + 15.0,
                e.name,
                x0 + 552.0,
                y + 15.0
            ));
            y += head;
            if e.timed.is_none() {
                b.push_str(&format!(
                    r#"<text class="fd mut" style="animation-delay:{:.2}s" x="{:.0}" y="{:.1}" font-size="10.5" font-style="italic">no timing: the example does not build in v0.6.1</text>"#,
                    0.1 + 0.014 * k,
                    x0 + 16.0,
                    y + row / 2.0 + 3.8
                ));
                (y, k) = (y + row, k + 1.0);
            }
            for c in e.cases() {
                // The A/A band with a cap at each end, the bar from 1 over it,
                // then the case name and its two ratios as printed.
                let (cy, fade, grow) = (y + row / 2.0, 0.1 + 0.014 * k, 0.35 + 0.014 * k);
                let ((n0, n1), one, to) = (c.noise(), x(1.0), x(c.gain));
                let (dir, from) = if c.gain >= 1.0 {
                    ("gr", one)
                } else {
                    ("gl", to)
                };
                let (band0, band1, band_y) = (x(n0), x(n1), cy - 5.0);
                let (ink, muted, fill) = (t.ink, t.muted, colour(c.side()));
                let (name, gain, aa) = (&c.name, ratio(c.gain), ratio(c.aa));
                let (tx, ty, gx, ax) = (x0 + 16.0, cy + 3.8, x0 + 510.0, x0 + 552.0);
                b.push_str(&format!(
                    r#"<g class="fd" style="animation-delay:{fade:.2}s"><rect x="{band0:.1}" y="{band_y:.1}" width="{:.1}" height="10" rx="2" fill="{muted}" opacity=".3"/><text x="{tx:.0}" y="{ty:.1}" font-size="11">{name}</text><text x="{gx:.0}" y="{ty:.1}" font-size="11" class="b" text-anchor="end">{gain}</text><text x="{ax:.0}" y="{ty:.1}" font-size="11" class="mut" text-anchor="end">{aa}</text></g><rect class="bar {dir}" style="animation-delay:{grow:.2}s" x="{from:.1}" y="{:.1}" width="{:.1}" height="6" rx="1.5" fill="{fill}"/><path class="fd" style="animation-delay:{fade:.2}s" d="M{band0:.1} {band_y:.1}v10M{band1:.1} {band_y:.1}v10" stroke="{ink}" stroke-opacity=".5" stroke-width="1.2"/>"#,
                    band1 - band0,
                    cy - 3.0,
                    (to - one).abs(),
                ));
                (y, k) = (y + row, k + 1.0);
            }
        }
    }
    let mut y = bottom + 30.0;
    for (i, note) in notes.iter().enumerate() {
        b.push_str(&format!(
            r#"<text x="40" y="{y:.0}" font-size="12.5"><tspan class="b">Note {}</tspan> · {note}</text>"#,
            i + 1
        ));
        y += 20.0;
    }
    for line in [
        "The machine was loaded and the runs were not pinned to cores, so each grey band, one A/A ratio of two copies of v0.6.1, is the noise to read its",
        "bar against: a single sample of that noise, not a confidence interval. A case counts as faster or slower only beyond its band. Bars of different",
        "cases are not comparable with each other: each times its own work. The output digests, not the timings, are the claim.",
    ] {
        y += 18.0;
        b.push_str(&format!(
            r#"<text x="40" y="{y:.0}" font-size="12.5" class="mut">{line}</text>"#
        ));
    }
    card(
        (1200, (y + 26.0) as u32),
        t,
        ("Speed against v0.6.1, every timed case", &speed_text(s)),
        &SPEED_STYLE.replace("MUTED", t.muted),
        &b,
    )
}
fn speed_picture(s: &SpeedSet) -> String {
    themed(
        "media/presentation/speed-light.svg",
        "media/presentation/speed-dark.svg",
        &speed_text(s),
        1200,
    )
}
/// Every generated SVG asset with its body.
fn assets() -> Result<Vec<(String, String)>> {
    let evidence = Evidence::read()?;
    let counts = evidence.fact_counts();
    let mut out = vec![
        (
            "media/presentation/header-light.svg".to_string(),
            header(false),
        ),
        (
            "media/presentation/header-dark.svg".to_string(),
            header(true),
        ),
        (
            "media/presentation/header-still-light.svg".to_string(),
            header_still(false),
        ),
        (
            "media/presentation/header-still-dark.svg".to_string(),
            header_still(true),
        ),
    ];
    let runs = Q8Runs::read()?;
    let crash = CrashSeries::read()?;
    let speed = SpeedSet::read()?;
    for dark in [false, true] {
        let theme = if dark { "dark" } else { "light" };
        out.push((
            format!("media/presentation/record-{theme}.svg"),
            record_card(dark, &runs),
        ));
        out.push((
            format!("media/presentation/crash-{theme}.svg"),
            crash_card(dark, &crash),
        ));
        out.push((
            format!("media/presentation/speed-{theme}.svg"),
            speed_card(dark, &speed),
        ));
    }
    for dark in [false, true] {
        let theme = if dark { "dark" } else { "light" };
        let path = |name: &str| format!("media/presentation/{name}-{theme}.svg");
        out.push((path("flow"), scene(dark)));
        out.push((path("facts"), facts(dark, counts)));
        out.push((path("wall"), wall(dark, &evidence)));
        out.push((path("bars"), bars(dark, &evidence)));
        out.push((path("logo"), logo(dark, true)));
        out.push((path("logo-still"), logo(dark, false)));
    }
    for i in 0..CHIPS.len() {
        for dark in [false, true] {
            out.push((chip_path(i, dark), chip(i, dark)));
        }
    }
    Ok(out)
}
fn page() -> String {
    let mut cards = String::new();
    for (i, (id, title, desc)) in SCENES.iter().enumerate() {
        if i > 0 {
            cards.push('\n');
        }
        cards.push_str(&format!(r#"<article class="card"><div class="num">{}</div><h3>{title}</h3><a class="media-link" href="media/gifs/{id}-light.gif" aria-label="Open {title} at full size">{}</a><p>{desc}</p><div class="links"><a href="media/gifs/{id}.txt">Full transcript</a><a href="media/gifs/{id}-dark.gif">Dark full size</a></div></article>"#,chip_picture(i),pair(id,title)));
    }
    HTML.replace("__HERO__", &pair(SCENES[0].0, SCENES[0].1))
        .replace("__BRAND_LOGO__", &logo_picture())
        .replace("__CARDS__", &cards)
        .replace("__FLOW__", &flow_picture())
        .replace("__FACTS__", &facts_picture())
        .replace("__WALL__", &wall_picture())
        .replace("__BARS__", &bars_picture())
        // The page is opened from a checkout, so its links name that checkout's files.
        .replace("__DOC__", ".")
        .replace("__RELEASE__", RELEASE)
}
fn native_intro() -> String {
    let mut grid = String::from("<table>\n");
    for (r, row) in SCENES.chunks(2).enumerate() {
        grid.push_str("<tr>\n");
        for (c, (id, title, desc)) in row.iter().enumerate() {
            grid.push_str(&format!("<td width=\"50%\" valign=\"top\">{}<h4>{title}</h4><a href=\"media/gifs/{id}-light.gif\">{}</a><p>{desc}</p><p><a href=\"media/gifs/{id}.txt\">Transcript</a> · <a href=\"media/gifs/{id}-dark.gif\">Dark full size</a></p></td>\n",chip_picture(r*2+c),picture(id,title,false)));
        }
        grid.push_str("</tr>\n");
    }
    grid.push_str("</table>\n");
    // The opening preview is the most telling one: a changed byte is refused.
    let (hero_id, hero_title, _) = SCENES[4];
    // Without its data the card is not drawn; the placeholder then stays and fails the checks.
    let record = Q8Runs::read()
        .map(|r| record_picture(&r))
        .unwrap_or_else(|e| format!("__RECORD__ {e}"));
    let commitments = commitments()
        .map(|n| n.to_string())
        .unwrap_or_else(|e| format!("__COMMITMENTS__ {e}"));
    let packages = packages()
        .map(|n| n.to_string())
        .unwrap_or_else(|e| format!("__PACKAGES__ {e}"));
    let crash = CrashSeries::read()
        .map(|c| crash_picture(&c))
        .unwrap_or_else(|e| format!("__CRASH__ {e}"));
    let speed = SpeedSet::read()
        .map(|s| speed_picture(&s))
        .unwrap_or_else(|e| format!("__SPEED__ {e}"));
    NATIVE
        .replace("__RECORD__", &record)
        .replace("__COMMITMENTS__", &commitments)
        .replace("__PACKAGES__", &packages)
        .replace("__FLOW__", &flow_picture())
        .replace("__BARS__", &bars_picture())
        .replace("__WALL__", &wall_picture())
        .replace("__LOGO__", &readme_logo())
        .replace("__CRASH__", &crash)
        .replace("__SPEED__", &speed)
        .replace("__HERO__", &picture(hero_id, hero_title, false))
        .replace("__HERO_ID__", hero_id)
        .replace("__GRID__", &grid)
}
/// The dated rows of the published-commitments table, as the README counts them.
fn commitments() -> Result<usize> {
    let text = fs::read_to_string("docs/COMMITMENTS.md")?;
    let table = text
        .split_once("## Published commitments\n")
        .ok_or("no published-commitments table")?
        .1;
    let rows = table
        .lines()
        .take_while(|l| l.is_empty() || l.starts_with('|'))
        .filter(|l| l.starts_with("| 20"))
        .count();
    if rows == 0 {
        return Err("no published commitment".into());
    }
    Ok(rows)
}
/// The packages in Cargo.lock; refused when one comes from outside this checkout.
fn packages() -> Result<usize> {
    let lock = fs::read_to_string("Cargo.lock")?;
    if lock.lines().any(|l| l.starts_with("source = ")) {
        return Err("Cargo.lock lists a package from outside this checkout".into());
    }
    Ok(lock.lines().filter(|l| *l == "[[package]]").count())
}
fn rewrite_readme(before: &str) -> Result<String> {
    let marker = "### What you can inspect\n";
    if !before.starts_with("# GEL RAM\n\n### Evidence Lab\n") || before.matches(marker).count() != 1
    {
        return Err("unexpected README structure; no guessed replacement".into());
    }
    let tail = before
        .split_once(marker)
        .ok_or("missing content boundary")?
        .1;

    Ok(format!("{}{marker}{tail}", native_intro()))
}
fn write_new(name: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = Path::new(name).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(name)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn hash(name: &str) -> Result<String> {
    let out = Command::new("sha256sum").args(["--", name]).output()?;
    if !out.status.success() {
        return Err(format!("cannot hash {name}").into());
    }
    let s = String::from_utf8(out.stdout)?;
    let h = s.split_whitespace().next().ok_or("missing hash")?;
    if h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hash".into());
    }
    Ok(h.to_string())
}
fn check_media() -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string("media/gifs/MANIFEST.txt")?;
    let mut checked = Vec::new();
    for line in text.lines().filter(|l| l.starts_with("sha256 ")) {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() != 3 || !words[2].starts_with("media/gifs/") || words[2].contains("..") {
            return Err("unexpected original media pin".into());
        }
        if hash(words[2])? != words[1] {
            return Err(format!("changed original media: {}", words[2]).into());
        }
        checked.push((words[2].to_string(), words[1].to_string()));
    }
    if checked.len() != 24 {
        return Err("expected 24 original media assets".into());
    }
    Ok(checked)
}
fn check_page(text: &str) -> Result<()> {
    let low = text.to_ascii_lowercase();
    for bad in [
        "<script",
        "<iframe",
        "<object",
        "<embed",
        "<form",
        "javascript:",
        "onerror=",
        "onload=",
        "@import",
        "url(",
        "src=\"http",
        "srcset=\"http",
        "__hero__",
        "__cards__",
        "__doc__",
        "__release__",
        "__base__",
        "__branch__",
        "__flow__",
        "__facts__",
        "__wall__",
        "__bars__",
        "__brand_logo__",
        "__hero_chip__",
        "__hero_id__",
        "__record__",
    ] {
        if low.contains(bad) {
            return Err(format!("unexpected active/remote/unresolved markup: {bad}").into());
        }
    }
    for (id, _, _) in SCENES {
        for ending in ["-dark.gif", "-light.gif", "-dark.png", "-light.png", ".txt"] {
            if !text.contains(&format!("media/gifs/{id}{ending}")) {
                return Err(format!("missing {id}{ending}").into());
            }
        }
    }
    let mut needed = Vec::new();
    for name in ["flow", "facts", "wall", "bars", "logo", "logo-still"] {
        for theme in ["light", "dark"] {
            needed.push(format!("media/presentation/{name}-{theme}.svg"));
        }
    }
    for i in 0..CHIPS.len() {
        needed.push(chip_path(i, false));
        needed.push(chip_path(i, true));
    }
    for need in needed {
        if !text.contains(&need) {
            return Err(format!("missing {need}").into());
        }
    }
    Ok(())
}
fn run() -> Result<()> {
    if std::env::args().len() != 1 {
        return Err("run the builder without arguments from the repository root".into());
    }
    let media = check_media()?;
    let immutable = [
        "LICENSE",
        "NOTICE",
        "CLA.md",
        "CLA-PRIVACY.md",
        "LICENSING.md",
        "media/RIGHTS.md",
        "media/gifs/MANIFEST.txt",
        "media/gifs/01-evidence.txt",
        "media/gifs/02-stale.txt",
        "media/gifs/03-backup.txt",
        "media/gifs/04-reproduce.txt",
        "media/gifs/05-integrity.txt",
        "media/gifs/07-literal.txt",
    ];
    let saved: Vec<_> = immutable
        .iter()
        .map(|s| Ok((*s, hash(s)?)))
        .collect::<Result<_>>()?;
    let before = fs::read_to_string("README.md")?;
    if before.contains("<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->") {
        let mut same = before.starts_with(&native_intro())
            && fs::read_to_string("docs/README-PRESENTATION.md")?
                == GUIDE.replace("__RELEASE__", RELEASE)
            && fs::read_to_string("README-MULTIMEDIA.html")? == page();
        for (name, body) in assets()? {
            same &= fs::read_to_string(&name).is_ok_and(|found| found == body);
        }
        if !same {
            return Err("existing presentation differs; refuse to overwrite it".into());
        }
        check_page(&page())?;
        println!("README_PRESENTATION=UNCHANGED original media pins verified");
        return Ok(());
    }
    let new = rewrite_readme(&before)?;
    let html = page();
    check_page(&html)?;
    let mut outputs = vec![("README-MULTIMEDIA.html".to_string(), html)];
    outputs.extend(assets()?);
    for name in outputs.iter().map(|(n, _)| n.as_str()).chain([
        "README-HISTORY-PRE-MULTIMEDIA.md",
        "docs/README-PRESENTATION.md",
    ]) {
        if Path::new(name).try_exists()? {
            return Err(format!("new path already exists: {name}").into());
        }
    }
    let main = fs::read_to_string("xtask/src/main.rs")?;
    let mark = "const REVIEWED_ASSETS: &[(&str, &str)] = &[\n";
    if main.matches(mark).count() != 1 {
        return Err("asset pin insertion point is not unique".into());
    }
    write_new("README-HISTORY-PRE-MULTIMEDIA.md", before.as_bytes())?;
    for (name, body) in &outputs {
        write_new(name, body.as_bytes())?;
    }
    let mut pins = mark.to_string();
    pins.push_str(
        "    // Exact script-free documentation presentation assets; no HTML/SVG wildcard.\n",
    );
    for (name, _) in &outputs {
        pins.push_str(&format!(
            "    (\n        \"{name}\",\n        \"{}\",\n    ),\n",
            hash(name)?
        ));
    }
    fs::write("xtask/src/main.rs", main.replacen(mark, &pins, 1))?;
    fs::write("README.md", new)?;
    write_new(
        "docs/README-PRESENTATION.md",
        GUIDE.replace("__RELEASE__", RELEASE).as_bytes(),
    )?;
    let index = fs::read_to_string("media/INDEX.md")?;
    fs::write("media/INDEX.md",format!("[Multimedia README and local full-page edition](../docs/README-PRESENTATION.md). Existing recordings below retain their original scope.\n\n{index}"))?;
    if fs::read("README-HISTORY-PRE-MULTIMEDIA.md")? != before.as_bytes() {
        return Err("README archive differs".into());
    }
    if check_media()? != media {
        return Err("media inventory changed".into());
    }
    for (name, pin) in saved {
        if hash(name)? != pin {
            return Err(format!("protected file changed: {name}").into());
        }
    }
    println!("README_PRESENTATION=BUILT six existing previews, exact archive, unchanged licenses and recordings");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("README_PRESENTATION=FAIL {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn six_real_scenarios() {
        assert_eq!(SCENES.len(), 6);
        let p = page();
        for (id, _, _) in SCENES {
            assert!(p.contains(&format!("media/gifs/{id}.txt")));
        }
    }
    #[test]
    fn no_active_or_remote_assets() {
        check_page(&page()).unwrap();
        for s in [
            "<script>x</script>",
            "<img src=\"https://remote.test/a.png\">",
            "__HERO__",
        ] {
            assert!(check_page(&(page() + s)).is_err());
        }
    }
    #[test]
    fn themes_have_no_placeholders() {
        let all = assets().unwrap();
        assert_eq!(all.len(), 22 + 2 * CHIPS.len());
        for (name, s) in all {
            assert!(!s.contains("__"), "{name}");
            assert!(!s.contains("<script"), "{name}");
            assert!(s.contains("<title") || s.contains("aria-label="), "{name}");
        }
        for s in [native_intro(), page()] {
            for bad in [
                "__FLOW__",
                "__FACTS__",
                "__WALL__",
                "__BARS__",
                "__BRAND_LOGO__",
                "__HERO_CHIP__",
                "__HERO_ID__",
                "__HERO__",
                "__GRID__",
                "__RESULTS__",
                "__RECORD__",
                "__COMMITMENTS__",
                "__PACKAGES__",
                "__LOGO__",
                "__CRASH__",
                "__SPEED__",
            ] {
                assert!(!s.contains(bad), "{bad}");
            }
        }
    }
    #[test]
    fn readme_counts_are_read_from_their_files() {
        let s = native_intro();
        // Counted another way: every table row in the file whose first cell is a date.
        let dated = fs::read_to_string("docs/COMMITMENTS.md")
            .unwrap()
            .lines()
            .filter(|l| {
                let b = l.as_bytes();
                b.len() > 14
                    && l.starts_with("| ")
                    && b[2..6].iter().all(u8::is_ascii_digit)
                    && b[6] == b'-'
                    && b[9] == b'-'
                    && l[12..].starts_with(" |")
            })
            .count();
        assert_eq!(commitments().unwrap(), dated);
        assert!(s.contains(&format!(
            "[{dated} published commitments](docs/COMMITMENTS.md)"
        )));
        let packages = packages().unwrap();
        assert!(s.contains(&format!(
            "every one of the {packages} packages in `Cargo.lock`"
        )));
        let lock = fs::read_to_string("Cargo.lock").unwrap();
        assert_eq!(
            packages,
            lock.matches("\n[[package]]\n").count(),
            "every package entry is counted"
        );
    }
    #[test]
    fn readme_is_short_and_names_no_question_set() {
        let s = native_intro();
        // Each image is one line; the text itself stays near 160 lines.
        assert!(s.lines().count() <= 200, "{} lines", s.lines().count());
        let lower = s.to_lowercase();
        for named in [
            "answer-or-abstain",
            "question set",
            "frozen question",
            "unknown to",
        ] {
            assert!(!lower.contains(named), "{named}");
        }
        // One place to start: a single clone, no tag the visitor cannot check out.
        assert_eq!(s.matches("git clone ").count(), 1);
        assert!(!s.contains("git checkout"));
    }
    #[test]
    fn facts_are_read_from_the_repository() {
        let (props, tests, mutants) = Evidence::read().unwrap().fact_counts();
        assert!(props > 0 && tests >= props && mutants > 0);
        let s = facts(false, (props, tests, mutants));
        assert!(s.contains(&format!("{props} → {tests}")));
        assert!(s.contains(&format!(">{mutants}<")));
    }
    #[test]
    fn check_graphics_follow_the_evidence() {
        let e = Evidence::read().unwrap();
        let (props, _, mutants) = e.fact_counts();
        let partial = e.unix_rows().iter().filter(|u| **u).count();
        let rows: usize = e.formats.iter().map(|f| f.rejected + f.accepted).sum();
        assert_eq!(rows, 2 * mutants, "each mutant is read with two pins");
        for dark in [false, true] {
            let w = wall(dark, &e);
            assert!(w.contains(&format!(">{props} properties × 3 platforms<")));
            assert!(w.contains(&format!(">{} cells: ", 3 * props - partial)));
            assert!(w.contains(&format!(">{partial} Windows cells: ")));
            let b = bars(dark, &e);
            assert!(b.contains(&format!(">{mutants}<")));
            for f in &e.formats {
                assert!(b.contains(&format!(">{}<", f.name)));
                assert!(b.contains(&format!(
                    "{} rejected · {} accepted",
                    f.rejected, f.accepted
                )));
            }
        }
    }
    #[test]
    fn wall_draws_the_rule_not_a_result() {
        let e = Evidence::read().unwrap();
        for dark in [false, true] {
            let w = wall(dark, &e);
            assert!(w.contains("must run and pass exactly once"));
            assert!(!w.contains("passed"), "a drawn cell is not a CI result");
        }
    }
    #[test]
    fn record_card_follows_the_recorded_runs() {
        let r = Q8Runs::read().unwrap();
        assert_eq!((r.n, r.ok), (48, 48));
        assert_eq!((r.record, r.packed, r.frames), (1152, 4608, 8192));
        assert_eq!(RECORD_VALUES + RECORD_MASK, r.record);
        for dark in [false, true] {
            let c = record_card(dark, &r);
            for want in [
                "1,152 B",
                "4,608 B",
                "8,192 B",
                "48 of 48 recorded runs",
                "view 1",
                "view 2",
                "view 3",
                "view 4",
                "SEMANTIC_ACCURACY=NOT_MEASURED",
            ] {
                assert!(c.contains(want), "{want}");
            }
            assert!(!c.contains("P0") && !c.contains("view 5"));
            // The banner states the record size the runs record.
            assert!(header(dark).contains(&format!(" {} bytes.<", grouped(r.record))));
        }
        assert!(native_intro().contains(&record_text(&r)));
        // The card draws the numbers it is given, not those of the recorded runs.
        let other = Q8Runs {
            n: 3,
            ok: 2,
            record: 2000,
            packed: 8000,
            frames: 9000,
        };
        for dark in [false, true] {
            let c = record_card(dark, &other);
            for want in ["2 of 3 recorded runs", "2,000 B", "8,000 B", "9,000 B"] {
                assert!(c.contains(want), "{want}");
            }
            for stale in ["48 of 48", "1,152 B", "4,608 B", "8,192 B"] {
                assert!(!c.contains(stale), "{stale}");
            }
        }
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1152), "1,152");
        assert_eq!(grouped(1234567), "1,234,567");
        // The card is not drawn from runs that are not all exact or that disagree.
        let run = |record: usize, verdict: &str| {
            format!(
                "canonical_record_bytes={record} materialized_four_frame_bytes=8192 packed_four_record_bytes={}\n{verdict}\n",
                4 * record
            )
        };
        let pass = run(1152, Q8_PASS);
        assert!(Q8Runs::from_texts(&[pass.clone(), pass.clone()]).is_ok());
        let fail = run(1152, "Q8_QUAD_EXACT=FAIL SEMANTIC_ACCURACY=NOT_MEASURED");
        for bad in [
            vec![pass.clone(), fail],
            vec![pass.clone(), run(1152, "")],
            vec![format!("{pass}{Q8_PASS}\n")],
            vec![pass.clone(), pass.replace("=8192", "=8193")],
            vec![pass.clone(), run(1153, Q8_PASS)],
            vec![run(1153, Q8_PASS), run(1153, Q8_PASS)],
            vec![pass.replace("packed_four_record_bytes=4608", "")],
            Vec::new(),
        ] {
            assert!(Q8Runs::from_texts(&bad).is_err(), "{bad:?}");
        }
    }
    #[test]
    fn banner_draws_the_cube_only() {
        for dark in [false, true] {
            for banner in [header(dark), header_still(dark)] {
                assert!(!banner.contains(r#"attributeName="rx""#));
                assert!(!banner.contains(r#"dur="4.2s""#));
            }
        }
    }
    #[test]
    fn first_screen_prints_first() {
        let s = native_intro();
        let at = |needle: &str| s.find(needle).unwrap_or_else(|| panic!("missing {needle}"));
        let print = at("**GEL RAM prints knowledge into memory as fixed Q8 records");
        let goal = at("The goal is a text AI");
        let limit = at("`NOT_ESTABLISHED`");
        let core = at("## The memory core");
        assert!(print < goal && goal < core);
        // The limit of the opening sentence stays on the first screen, before the record.
        assert!(goal < limit && limit < at("## One record, four exact views"));
    }
    #[test]
    fn bars_show_a_mismatch() {
        let e = Evidence {
            rows: vec![("P01".to_string(), vec!["t".to_string()])],
            unix_only: Vec::new(),
            formats: vec![FormatCount {
                name: "FMT".to_string(),
                mutants: 1,
                rejected: 1,
                accepted: 1,
                matched: 1,
            }],
        };
        let b = bars(false, &e);
        assert!(b.contains(&format!("fill:{}\">1/2<", LIGHT.red[1])));
        assert!(!b.contains(LIGHT.good));
    }
    #[test]
    fn motion_can_be_reduced() {
        let e = Evidence::read().unwrap();
        for dark in [false, true] {
            // The banner moves with SMIL, which CSS cannot stop: README shows the still file instead.
            assert!(header(dark).contains("<animate") && !header_still(dark).contains("<animate"));
            for s in [
                scene(dark),
                logo(dark, true),
                facts(dark, (1, 1, 1)),
                wall(dark, &e),
                bars(dark, &e),
            ] {
                assert!(s.contains("prefers-reduced-motion"));
            }
        }
        assert!(native_intro().contains("05-integrity"));
        assert!(native_intro().contains(
            r#"<source media="(prefers-reduced-motion: reduce)" srcset="media/presentation/header-still-light.svg">"#
        ));
    }
    #[test]
    fn logo_keeps_its_core_and_a_final_state() {
        for dark in [false, true] {
            let s = logo(dark, true);
            // The still variant draws the same logo without any animation.
            let still = logo(dark, false);
            assert!(!still.contains("animation") && !still.contains("<style"));
            assert!(!still.contains("class=\"sheet"));
            assert!(still.contains(&gel_logo(
                theme(dark),
                if dark { "#79c0ff" } else { "#0969da" },
                (96.0, 108.0),
                76.0
            )));
            // The accent core is drawn outside every animated group.
            let core = iso_box(
                (96.0, 108.0),
                (
                    1.5 * 76.0 / 3.0 - 76.0 / 6.4,
                    1.5 * 76.0 / 3.0 - 76.0 / 6.4,
                    1.5 * 76.0 / 3.0 - 76.0 / 6.4,
                ),
                (76.0 / 3.2, 76.0 / 3.2, 76.0 / 3.2),
                theme(dark).blue,
                theme(dark).ink,
            );
            let at = s.find(&core).expect("core drawn");
            let before = &s[..at];
            assert_eq!(
                before.matches("<g class=").count(),
                before.matches("</g>").count()
            );
            assert!(s.contains(".l0,.l1,.l2,.ray,.hit,.seal{animation:none;opacity:1}"));
            for banner in [header(dark), header_still(dark)] {
                assert!(
                    banner.contains(">GEL RAM<")
                        && banner.contains(">Knowledge printed, not trained<")
                );
            }
        }
    }
    #[test]
    fn keeps_technical_tail() {
        let s = "# GEL RAM\n\n### Evidence Lab\nold\n### What you can inspect\nkeep these bytes\n";
        assert!(rewrite_readme(s)
            .unwrap()
            .ends_with("### What you can inspect\nkeep these bytes\n"));
    }
    #[test]
    fn unknown_readme_refused() {
        assert!(rewrite_readme("unrelated").is_err());
    }
    #[test]
    fn no_false_status_badges() {
        for bad in ["CI passing", "No hallucinations", "platforms 3/3"] {
            assert!(!page().contains(bad));
            assert!(!native_intro().contains(bad));
        }
    }
    #[test]
    fn movement_has_alternatives() {
        assert!(page().contains("prefers-reduced-motion"));
        assert!(page().contains("id=\"still-mode\""));
        assert_eq!(page().matches("<article class=\"card\"").count(), 6);
        // The still-image control also stops the animated brand logo.
        let brand = page()
            .split("<div class=\"brand\">")
            .nth(1)
            .and_then(|s| s.split("<div class=\"brand-name\">").next())
            .unwrap()
            .to_string();
        assert!(brand.trim_start().starts_with("<div class=\"motion\">"));
        assert!(brand.contains("logo-light.svg") && brand.contains("logo-dark.svg"));
        assert!(brand.contains("<div class=\"poster\">"));
        assert!(brand.contains("logo-still-light.svg") && brand.contains("logo-still-dark.svg"));
    }
    #[test]
    fn native_is_not_one_big_image() {
        let s = native_intro();
        assert!(s.contains("<table>"));
        assert!(s.contains("README-MULTIMEDIA.html"));
        assert_eq!(s.matches("<td width=\"50%\"").count(), 6);
    }
    #[test]
    fn images_keep_their_aspect_ratio() {
        // A fixed height with a width the page narrows stretches the image.
        for text in [native_intro(), page()] {
            for img in text.split("<img ").skip(1) {
                let tag = img.split('>').next().unwrap_or("");
                assert!(!tag.contains(" height="), "fixed image height: {tag}");
            }
        }
    }
    #[test]
    fn record_card_moves_and_draws_sizes_to_scale() {
        let recorded = Q8Runs::read().unwrap();
        let other = Q8Runs {
            n: 3,
            ok: 2,
            record: 2000,
            packed: 8000,
            frames: 9000,
        };
        for r in [&recorded, &other] {
            for dark in [false, true] {
                let c = record_card(dark, r);
                // The loop: the bar grows, four views sweep it and land, the sizes
                // grow and the verdict fades in, all on one clock.
                for name in [
                    "g0", "s0", "s1", "s2", "s3", "c0", "c1", "c2", "c3", "z0", "z1", "z2", "vd",
                ] {
                    assert!(
                        c.contains(&format!(".{name}{{animation:{name} {RECORD_LOOP}s "))
                            && c.contains(&format!("@keyframes {name}{{")),
                        "{name}"
                    );
                    // ... and an element carries it.
                    assert!(c.contains(&format!(" {name}\">")), "{name}");
                }
                assert!(!c.contains("animation-delay"));
                // Reduced motion stops every moving element on the final picture.
                assert!(c.contains("@media (prefers-reduced-motion:reduce){.a{animation:none}}"));
                // The compared sizes are drawn to one scale.
                let widths: Vec<f64> = c
                    .split("<rect class=\"size\" ")
                    .skip(1)
                    .map(|s| {
                        s.split("width=\"")
                            .nth(1)
                            .unwrap()
                            .split('"')
                            .next()
                            .unwrap()
                            .parse()
                            .unwrap()
                    })
                    .collect();
                assert_eq!(widths.len(), 3);
                let longest = r.record.max(r.packed).max(r.frames) as f64;
                let reach = widths.iter().copied().fold(0.0, f64::max);
                assert!((reach - 700.0).abs() < 0.05, "{reach}");
                for (drawn, bytes) in widths.iter().zip([r.record, r.packed, r.frames]) {
                    let want = reach * bytes as f64 / longest;
                    assert!((drawn - want).abs() < 0.06, "{drawn} vs {want}");
                }
            }
        }
    }
    #[test]
    fn the_first_page_does_not_say_how_a_view_arranges_bytes() {
        let r = Q8Runs::read().unwrap();
        // Gradient stops carry an `offset` attribute, so the phrase is matched whole.
        for text in [
            native_intro(),
            record_card(false, &r),
            record_card(true, &r),
        ] {
            let lower = text.to_lowercase();
            for word in ["reversal", "additive offset", "modulo", "seed"] {
                assert!(!lower.contains(word), "{word}");
            }
        }
    }
    #[test]
    fn crash_card_follows_the_recorded_series() {
        let c = CrashSeries::read().unwrap();
        let text = fs::read_to_string(CRASH_FILE).unwrap();
        let summary = text.lines().last().unwrap();
        assert_eq!(
            summary,
            format!(
                "CRASH_SERIES=PASS trials={} acknowledged={} acknowledged_lost={} partial={} resumed={} killed_mid_publication={}",
                c.trials.len(),
                c.acknowledged,
                c.lost,
                c.partial,
                c.resumed,
                c.mid_publication
            )
        );
        assert_eq!((c.documents, c.window_ms), (24, 344));
        assert_eq!(
            (c.file_system.as_str(), c.date.as_str()),
            ("ext4", "2026-09-29")
        );
        for dark in [false, true] {
            let s = crash_card(dark, &c);
            for want in [
                format!(">{}<", c.trials.len()),
                format!(">{}<", grouped(c.acknowledged)),
                format!(">{}<", c.mid_publication),
                format!(
                    "{} kills while saving. 0 acknowledged snapshots lost.",
                    c.trials.len()
                ),
                "not a power cut".to_string(),
                "the author's Linux machine".to_string(),
                "SIGKILL".to_string(),
            ] {
                assert!(s.contains(&want), "{want}");
            }
            // The host name and the CPU of the recording machine are not shown.
            for hidden in ["Ryzen", "Radeon", "oem", "x86_64", "ebe5360"] {
                assert!(!s.contains(hidden), "{hidden}");
            }
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.contains("<title id=\"t\">") && s.contains("<desc id=\"d\">"));
            assert!(s.contains("@media (prefers-reduced-motion:reduce){.pt,.cnt{animation:none}}"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—".contains(ch)));
        }
        assert!(crash_picture(&c).contains(&crash_text(&c)));
        assert!(crash_text(&c).contains("2,683 snapshots were acknowledged"));
    }
    #[test]
    fn crash_card_draws_every_trial() {
        let c = CrashSeries::read().unwrap();
        let s = crash_card(false, &c);
        assert_eq!(s.matches("<circle class=\"pt\"").count(), c.trials.len());
        let filled = format!("r=\"5\" fill=\"{}\"/>", LIGHT.good);
        assert_eq!(
            s.matches(&filled).count(),
            c.mid_publication + 1,
            "points and the legend"
        );
        // A series of its own is drawn from its own rows, not from the recorded one.
        let mini = "# host: Linux k · cpu · btrfs · rustc · rev · 2027-01-02T00:00:00Z\n# gel-evidence crash series: 4 documents of about 64 KiB, a new snapshot after each; kill after 1..=100 ms (uninterrupted run + 20%), seed 1\n".to_string()
            + CRASH_COLUMNS
            + "\n1\t0\t0\t0\t0\t0\t1\ttrue\t-\n2\t100\t4\t0\t0\t0\t0\ttrue\t9\n3\t50\t2\t0\t1\t0\t0\ttrue\t5\nCRASH_SERIES=PASS trials=3 acknowledged=6 acknowledged_lost=0 partial=0 resumed=3 killed_mid_publication=2\n";
        let m = CrashSeries::from_text(&mini).unwrap();
        let s = crash_card(true, &m);
        assert_eq!(s.matches("<circle class=\"pt\"").count(), 3);
        // Kill at 0 ms with nothing saved, at the window end with all saved, and halfway.
        for (cx, cy) in [("100.0", "324.0"), ("740.0", "120.0"), ("420.0", "222.0")] {
            assert!(s.contains(&format!("cx=\"{cx}\" cy=\"{cy}\"")), "{cx} {cy}");
        }
        assert!(s.contains("(btrfs, 2027-01-02)") && s.contains(">all 4 snapshots saved<"));
        assert!(!s.contains("2,683") && !s.contains(">200<"));
    }
    #[test]
    fn crash_series_disagreeing_with_its_rows_is_refused() {
        let text = fs::read_to_string(CRASH_FILE).unwrap();
        assert!(CrashSeries::from_text(&text).is_ok());
        let row = "\n10\t311\t24\t0\t0\t0\t0\ttrue\t4439\n";
        assert!(text.contains(row));
        let summary = text.lines().last().unwrap();
        for bad in [
            text.replace(row, "\n10\t311\t24\t1\t0\t0\t0\ttrue\t4439\n"),
            text.replace(row, "\n10\t311\t24\t0\t0\t1\t0\ttrue\t4439\n"),
            text.replace(row, "\n10\t311\t24\t0\t0\t0\t0\tfalse\t4439\n"),
            text.replace(row, "\n10\t311\t23\t0\t0\t0\t0\ttrue\t4439\n"),
            text.replace(row, "\n10\t311\t24\t0\t0\t0\t1\ttrue\t4439\n"),
            text.replace(row, "\n"),
            text.replace(row, "\n10\t311\t24\t0\t0\t0\ttrue\t4439\n"),
            text.replace(row, "\n10\t311\tmany\t0\t0\t0\t0\ttrue\t4439\n"),
            text.replace(row, "\n10\t999\t24\t0\t0\t0\t0\ttrue\t4439\n"),
            text.replace("CRASH_SERIES=PASS", "CRASH_SERIES=FAIL"),
            text.replace(summary, ""),
            format!("{text}{summary}\n"),
            text.replace("killed_mid_publication=153", ""),
            text.replace(CRASH_COLUMNS, "# trial\tkill_ms"),
            text.replace("# host: Linux ", "# host: "),
        ] {
            assert!(CrashSeries::from_text(&bad).is_err());
        }
    }
    #[test]
    fn speed_card_follows_the_timing_records() {
        let s = SpeedSet::read().unwrap();
        // Counted another way: example directories, summaries and their case lines.
        let (mut dirs, mut summaries, mut lines) = (0, 0, 0);
        for entry in fs::read_dir(SPEED_DIR).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs += 1;
                if let Ok(text) = fs::read_to_string(path.join("summary.txt")) {
                    summaries += 1;
                    lines += text
                        .lines()
                        .filter(|l| l.starts_with("SPEED_AB case="))
                        .count();
                }
            }
        }
        let cases = s.cases().count();
        let (timed, same) = s.verdicts();
        assert_eq!((s.examples.len(), timed, cases), (dirs, summaries, lines));
        assert_eq!((dirs, summaries, lines), (24, 23, 133));
        assert_eq!(s.sides(), [103, 18, 12]);
        assert_eq!(same, 22);
        let text = speed_text(&s);
        for want in [
            "133 timed cases in 23 timing examples",
            "103 are faster beyond their A/A band, 18 slower beyond it (SHA-256 alone at 0.28 and 0.39) and 12 inside it",
            "22 of 23 timed examples",
            "speed_tree_gates prints DIFFERENT_OUTPUT because it counts the files of the tree it runs in",
            "speed_top1_many has only a manifest",
            "not pinned to cores",
        ] {
            assert!(text.contains(want), "{want}");
        }
        assert!(speed_picture(&s).contains(&format!(r#"alt="{text}""#)));
        for dark in [false, true] {
            let c = speed_card(dark, &s);
            // One bar per timed case: nothing aggregated, nothing dropped.
            assert_eq!(c.matches(r#"<rect class="bar "#).count(), cases);
            for e in &s.examples {
                assert!(c.contains(&format!(">{}<", e.name)), "{}", e.name);
                for case in e.cases() {
                    assert!(c.contains(&format!(">{}<", case.name)), "{}", case.name);
                }
            }
            // The SHA-256 slowdown is drawn to the left, in red.
            assert!(c.contains(">0.28<") && c.contains(">0.39<"));
            assert!(c.contains(&format!(r#"fill="{}"/>"#, theme(dark).red[1])));
            assert!(c.contains(r#"<rect class="bar gl""#));
            for want in [
                ">faster beyond its A/A band: 103<",
                ">slower beyond it: 18<",
                ">inside it: 12<",
                ">same output digest in A, A2 and B: 22 of 23<",
                // Notes are numbered in name order.
                "not built in v0.6.1 · note 1",
                "output digests differ · note 2",
                "AMD Ryzen AI 9 HX 370",
                "not a confidence interval",
                "not comparable with each other",
                "The output digests, not the timings, are the claim.",
            ] {
                assert!(c.contains(want), "{want}");
            }
            assert!(c.contains("@media (prefers-reduced-motion:reduce){.bar,.fd{animation:none}}"));
            // Only the keyframes hide or shrink anything, so with the animation
            // removed every element is in its final, visible state.
            let style = c.split("<style>").nth(1).unwrap();
            let css = style.split("@keyframes").next().unwrap();
            assert!(!css.contains("opacity") && !css.contains("transform:"));
            assert!(!c.contains("__") && !c.contains("<script") && !c.contains("href"));
            assert!(c.chars().all(|ch| ch.is_ascii() || ch == '·' || ch == '—'));
        }
    }
    #[test]
    fn speed_summaries_that_do_not_add_up_are_refused() {
        let readme = format!("| Machine | one machine |\n| Load | load average 1 |\n| Pinning | none: not pinned |\nB is `main` at `{SPEED_B}`. `speed_new` has only a manifest: it does not build there. `speed_tree` prints `DIFFERENT_OUTPUT` because it counts its own tree.\n");
        let line = "SPEED_AB case=large a_ns=38298255 a2_ns=37391469 b_ns=138871537 aa=0.976 gain=0.276 runs=27";
        let digest = |c: char| c.to_string().repeat(64);
        let summary = |lines: &str, b: char, verdict: &str| {
            format!(
                "{lines}\nSPEED_AB_CHECK tree=a sha256={}\nSPEED_AB_CHECK tree=a2 sha256={}\nSPEED_AB_CHECK tree=b sha256={}\nSPEED_AB_VERDICT={verdict}\n",
                digest('a'),
                digest('a'),
                digest(b)
            )
        };
        let one = |name: &str, text: Option<String>| {
            SpeedSet::from_texts(&readme, &[(name.to_string(), text)])
        };
        let good = summary(line, 'a', "SAME_OUTPUT");
        let s = one("speed_sha256", Some(good.clone())).unwrap();
        let case = &s.examples[0].cases()[0];
        assert_eq!((case.gain, case.aa, case.runs), (0.276, 0.976, 27));
        assert!(s.examples[0].note.is_none());
        // A differing digest and a missing summary are kept, with the README's sentence.
        let differs = summary(line, 'b', "DIFFERENT_OUTPUT");
        let s = one("speed_tree", Some(differs.clone())).unwrap();
        assert_eq!(
            s.examples[0].note.as_deref(),
            Some("speed_tree prints DIFFERENT_OUTPUT because it counts its own tree.")
        );
        let s = SpeedSet::from_texts(
            &readme,
            &[
                ("speed_new".to_string(), None),
                ("speed_sha256".to_string(), Some(good.clone())),
            ],
        )
        .unwrap();
        assert!(s.examples[0].timed.is_none() && s.examples[0].note.is_some());
        assert_eq!(s.cases().count(), 1);
        let last_digest = format!("SPEED_AB_CHECK tree=b sha256={}\n", digest('a'));
        for bad in [
            good.replace("gain=0.276", "gain=0.286"),
            good.replace("aa=0.976", "aa=0.967"),
            good.replace("gain=0.276", "gain=0.27"),
            good.replace("gain=0.276", "gain=0"),
            good.replace("b_ns=138871537", "b_ns=128871537"),
            good.replace("a_ns=38298255", "a_ns=0"),
            good.replace(" runs=27", ""),
            good.replace("runs=27", "runs=27 extra=1"),
            good.replace("case=large", "case=Large"),
            good.replace("SPEED_AB_VERDICT=SAME_OUTPUT\n", ""),
            good.replace("SAME_OUTPUT", "DIFFERENT_OUTPUT"),
            good.replace("tree=a2", "tree=b"),
            good.replace(&last_digest, ""),
            format!("{line}\n{good}"),
            format!("{good}SPEED_AB_VERDICT=SAME_OUTPUT\n"),
            format!("an unexpected line\n{good}"),
            summary(line, 'b', "SAME_OUTPUT"),
            summary(line, 'g', "DIFFERENT_OUTPUT"),
        ] {
            assert!(one("speed_sha256", Some(bad.clone())).is_err(), "{bad}");
        }
        // The medians give 0.2758: a print to two digits must read 0.28.
        let shorter = good.replace("gain=0.276", "gain=0.28");
        assert!(one("speed_sha256", Some(shorter)).is_ok());
        // Unexplained differences or gaps, and a README that no longer states the setup.
        assert!(one("speed_sha256", Some(differs)).is_err());
        assert!(one("speed_new", None).is_err());
        let gap = [
            ("speed_sha256".to_string(), Some(good.clone())),
            ("speed_gone".to_string(), None),
        ];
        assert!(SpeedSet::from_texts(&readme, &gap).is_err());
        for changed in [
            readme.replace("| Machine |", "| Host |"),
            readme.replace("none:", "cores 0 to 7:"),
            readme.replace(SPEED_B, "0000000"),
            readme.replace("one machine", "one <b>machine</b>"),
        ] {
            let set = [("speed_sha256".to_string(), Some(good.clone()))];
            assert!(SpeedSet::from_texts(&changed, &set).is_err(), "{changed}");
        }
    }
    #[test]
    fn speed_card_draws_the_numbers_it_is_given() {
        let case = |name: &str, aa: f64, gain: f64| SpeedCase {
            name: name.to_string(),
            aa,
            gain,
            runs: 9,
        };
        let s = SpeedSet {
            machine: "one machine".to_string(),
            load: "load average 1".to_string(),
            examples: vec![SpeedExample {
                name: "speed_one".to_string(),
                timed: Some((
                    vec![
                        case("quarter", 1.0, 0.25),
                        case("four", 1.0, 4.0),
                        case("same", 1.25, 1.1),
                    ],
                    true,
                )),
                note: None,
            }],
        };
        assert_eq!(s.sides(), [1, 1, 1]);
        for dark in [false, true] {
            let c = speed_card(dark, &s);
            assert_eq!(c.matches(r#"<rect class="bar "#).count(), 3);
            for want in [
                ">quarter<",
                ">four<",
                ">same<",
                ">0.25<",
                ">4.00<",
                ">1.10<",
            ] {
                assert!(c.contains(want), "{want}");
            }
            assert!(c.contains(">faster beyond its A/A band: 1<") && c.contains(">inside it: 1<"));
            assert!(c.contains(">9 runs per case<") && c.contains(">1.25<"));
            assert!(!c.contains(">0.28<") && !c.contains("AMD") && !c.contains("note 1"));
            // On a log scale a quarter and four are equally long, on either side of 1.
            let bars: Vec<(&str, &str)> = c
                .split(r#"<rect class="bar "#)
                .skip(1)
                .map(|r| {
                    let width = r.split(" width=\"").nth(1).unwrap();
                    (
                        r.split('"').next().unwrap(),
                        width.split('"').next().unwrap(),
                    )
                })
                .collect();
            assert_eq!((bars[0].0, bars[1].0), ("gl", "gr"));
            assert_eq!(bars[0].1, bars[1].1);
            let colours = [theme(dark).red[1], theme(dark).blue[1], theme(dark).muted];
            for (bar, colour) in c.split(r#"<rect class="bar "#).skip(1).zip(colours) {
                assert!(bar.contains(&format!(r#"fill="{colour}"/>"#)));
            }
        }
    }
}
