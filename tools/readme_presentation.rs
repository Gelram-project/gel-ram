// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Generate a script-free README presentation. Existing recordings are never rewritten.
#![forbid(unsafe_code)]
use std::{fs, io::Write, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// The release whose sources the page links to and checks out.
const RELEASE: &str = "v0.9.0";
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
<section class="scope section" id="scope"><h2>What this page does and does not establish</h2><p>This is a documentation presentation, not the application interface. It describes this checkout; tagged releases are listed on the repository's Releases page. The older GIFs retain the source revision declared in their <a href="media/gifs/MANIFEST.txt">original manifest</a>. Presentation changes do not retime them, certify a release or close outstanding human review.</p><p>Snapshots and backups are plaintext. A hash is not encryption, a signature or proof that a source is true. Different search tasks and matching semantics are not interchangeable benchmarks.</p><p>GEL RAM-owned material remains under <a href="__DOC__/LICENSE">GEL RAM Noncommercial Reciprocal License 1.0</a>. Read the <a href="__DOC__/LICENSING.md">licensing guide</a>, <a href="__DOC__/THIRD-PARTY-NOTICES.md">third-party notices</a> and <a href="__DOC__/media/RIGHTS.md">media rights</a>. This page grants no additional rights.</p></section>
</main>
<footer><p><strong>GEL RAM</strong><br>RR — GEL RAM Project · Public documentation</p><p><a href="__DOC__/LICENSE">License</a> · <a href="__DOC__/SECURITY.md">Security</a> · <a href="__DOC__/CLA-PRIVACY.md">CLA privacy</a> · <a href="__DOC__/CONTRIBUTING.md">Contributing</a><br>Existing evidence preserved. New presentation only.</p></footer>
</div></body></html>
"####;
const NATIVE: &str = r####"# GEL RAM

<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->
__HEADER__
__BADGES__

**GEL RAM prints knowledge into memory as fixed Q8 records, instead of training
it into model weights, and reads the records back exactly, working toward
hardware-level memory computation.** The goal is a text AI that holds a free
conversation in Polish and English: it answers in sentences of its own, not
pasted excerpts, from what it holds, shows the source it relied on, and says it
does not know when it holds nothing on the question.

__STATUS__

The text AI is a goal, not a result ([the goal](docs/GOAL.md)). Hardware-level memory computation and power-cut durability of snapshots and of the record history are `NOT_ESTABLISHED` ([claim registry](docs/CLAIMS.md)).

<details><summary>What is public and what is private, in words</summary>

| Status | What it covers |
|:---|:---|
| **You can check it here** | One record read back exactly through four views; a changed byte refused; a killed process losing nothing it confirmed. One command each, [below](#check-it-yourself). |
| **Private** | The step that prints text into records, and the private build that answers from them; both exist only on the author's machine. [__COMMITMENTS__ published commitments](docs/COMMITMENTS.md) fix the exact contents of private source trees, data sets, a sealed sample and the target Q8 Quad quantizer as of their dates; only their labels say what each holds. |
| **Not established** | The text AI, hardware-level memory computation and power-cut durability of snapshots and of the record history; see the line above. |

</details>

## Check it yourself

__TERMINAL__

With [Git](https://git-scm.com/) and [rustup](https://rustup.rs) installed, run each line separately; only `git clone` and `rustup` need a network.
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

<details><summary>What each line shows, and its limit</summary>

No model and no private data are involved.

| Command | Line to look for | What it shows | Its limit |
|:---|:---|:---|:---|
| `--literal` | `ROUNDTRIP=4/4 DIFFERENT_BITS=0`, `TAMPER=REJECTED` | The first 1,024 bytes of any file you name, placed in one record; all four views give them back bit for bit, and a copy with one changed byte is refused | Your bytes as they are, not knowledge printing: no meaning and no search |
| `quad_compare` | `Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED` | 512 synthetic records; each view's score equals the score of a separate reference copy, bit for bit | Meaning is not measured; one plain copy can still be read faster than the shared record |
| `mutation-matrix` | `MUTATION_MATRIX=PASS mutants=179` | Each changed copy in the matrix, over the three public file formats, is refused under the original SHA-256 | A finite set of changes, not every possible one |
| `crash-series 20` | `CRASH_SERIES=PASS` with `acknowledged_lost=0` | Evidence Lab is killed 20 times at random while it writes; nothing it confirmed is lost | Recorded on Linux; `verify` runs it on Unix hosts only; a process kill, not a power cut |
| `verify` | `GEL_VERIFY_ALL=PASS` | The workspace tests and public gates, with short forms of the runs above | Crash series on Unix hosts only; the private build is not in this checkout |

</details>

## One record, four exact views

__Q8__

__RECORD__

One record, 1,152 bytes, read four ways, each bit for bit. [Contract](docs/Q8-QUAD.md) · [the recorded runs](docs/evidence-q8-current/README.md)

<details><summary>What the four views are, and what they are not</summary>

A record is held once, in 1,152 bytes of memory on the tested ABI. A view reads
the same values in another fixed arrangement; each of the four views scores exactly as a separate, fully
written-out copy would, bit for bit, and four such copies would take 4,608 bytes.
Four views are not four independent memories, four votes or a speedup.

</details>

## Exact bytes, or a refusal

__FLOW__

Your quote comes back byte for byte; a changed byte is refused. [Evidence Lab guide](docs/EVIDENCE-LAB.md) · [as text](media/gifs/__HERO_ID__.txt)

<details><summary>What Evidence Lab checks, and what a hash does not show</summary>

Evidence Lab (`gel-evidence`), the public collection tool, works on your own
UTF-8 files: each quotation comes back byte for byte with its byte range, a
replaced document's old citation is refused, and a saved snapshot with one
changed byte fails to load under the SHA-256 you kept. A matching hash shows
these are your bytes, not that the source is true or private: snapshots and
backups hold plain text ([backup contract](docs/BACKUP.md)). It is phrase lookup,
not question answering. Its demo,
`cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo`,
ends with `GEL_EVIDENCE_DEMO=PASS`.

</details>

## A killed process loses nothing it confirmed

__CRASH__

200 random process kills in one recorded run: 0 of 2,683 confirmed snapshots lost. [Crash series](docs/CRASH-SERIES.md)

<details><summary>The recorded series in words</summary>

In the recorded series Evidence Lab was killed at random moments in 200 trials:
0 of 2,683 acknowledged snapshots were lost, none was partial, and every trial
resumed to the uninterrupted result ([crash series](docs/CRASH-SERIES.md)).
Write failures, denied permissions and a full disk leave the previous snapshot
intact ([fault tests](docs/PUBLICATION-FAULT-TESTS.md)).

</details>

## The public checks, drawn from their files

__BARS__

__WALL__

Every changed copy refused under its original SHA-256; CI requires every mapped property on three platforms, Unix-only tests exempt on Windows. [CI evidence](docs/CI-EVIDENCE.md)

<details><summary>Where these images come from</summary>

Both images are drawn from the committed mutation report and the property map when the README is built; the presentation check fails if either file changes without a rebuild. The map shows the rule CI enforces, not the outcome of one run ([CI evidence](docs/CI-EVIDENCE.md)).

</details>

## Speed, timed against v0.6.1

__SPEED__

__RERUN__

Each case timed against two copies of the baseline, then once more with no other work running; most are faster, some slower. [Re-runs](docs/speed-results/2026-11-04-reruns/README.md)

<details><summary>What came in 0.7, in words</summary>

- **No third-party packages:** every one of the __PACKAGES__ packages in `Cargo.lock` is a crate of this checkout. SHA-256 and Unicode normalization are written here; SHA-256 alone runs at 0.28 and 0.39 of the speed of v0.6.1, which used the processor's SHA instructions.
- **One Unicode version:** normalization follows Unicode __UNICODE__, the version the pinned Rust 1.85.0 standard library uses for lowercasing and word splitting, and passes every line of the Unicode normalization test file.
- **The same output bits, faster:** each speed change was timed against two copies of the baseline, and every run printed the same output digest, apart from the whole-tree gates, which count the files of their own tree ([v0.7.0 in the roadmap](docs/ROADMAP.md#v070)).

</details>

## New in 0.9, and in 0.8

__GLANCE__

<details><summary>What changed, in words</summary>

- **New in 0.9, checks on your own host and faults of saved files:** `xtask witness NEW_DIR` runs the six short public checks on any host and writes one plain-text block; a filter replaces host and user names, paths, network addresses and serial numbers in the step lines, and a separate check refuses the block if this host's names or directories remain anywhere in it. No block from a host other than the author's is retained yet ([check on your hardware](docs/CHECK-ON-YOUR-HARDWARE.md)). 20 tests in three new files cut the saved files of gel-history, gel-store and gel-source at every byte, damage them without changing their length, race two child processes on one path, stop or kill a save and restart, and read beside a writer; none is a power-loss test ([faults of saved files](docs/PUBLICATION-FAULT-TESTS.md#faults-of-saved-files-on-disk-and-between-processes)).
- **New in 0.9, memory and CI reports on record:** `xtask memory-run` starts each public operation as a child process and records payload, peak mapped and resident memory, page faults and threads; one record of a public revision is kept, nine processes of each of eight operations, 72 samples, on one machine, process-wide, with no time recorded ([memory record](docs/memory-results/2026-11-08/README.md)). Sanitized CI reports of a third revision are kept and checked by `xtask ci-evidence-recorded`; they are hosted CI runs of one revision each, not an independent second host ([recorded reports](docs/evidence-ci/README.md), [release notes](RELEASE-NOTES-v0.9.0.md)).
- **New in 0.8, programs and files name their license:** the eight tool programs answer `--version` and `--license` (the `xtask` runner does not); every GEL RAM-owned Rust source file begins with a two-line license header (the generated Unicode tables are third-party data with their own notice); `gel-license check DIR` reports whether a copy kept the license, the Required Notice (the attribution line in NOTICE), those headers and the Unicode notice, not that every notice was kept. The license guides now follow the words of the LICENSE ([v0.8.0 in the roadmap](docs/ROADMAP.md#v080)).

</details>

## Private work, fixed in time

__PLEDGES__

Private files, hashed now; checkable once the owner hands them over. A hash shows they did not change, not that they are correct. [Commitments](docs/COMMITMENTS.md)

<details><summary>What a commitment shows, and what it does not</summary>

A [commitment](docs/COMMITMENTS.md) publishes now the SHA-256 of a manifest of a private directory; the files can be handed over later and checked with `xtask commitment-check`.
It shows the files existed and did not change, not that they are correct. The author's own measurements are in [measured progress](docs/MEASURED-PROGRESS.md); nobody else has re-run them.

</details>

## See it in action

Six edited 12-second replays of recorded public-tool runs; the pacing is not execution time.
[Static view](media/gifs/STATIC.md) · [Original source and hashes](media/gifs/MANIFEST.txt)

__GRID__
## Using GEL RAM

__LICENSE__

Source-available: noncommercial use and evaluation are allowed; Monetized Use needs a written Commercial Agreement first; the [LICENSE](LICENSE) controls.

<details><summary>The license in words</summary>

GEL RAM uses the GEL RAM Noncommercial Reciprocal License 1.0: source-available,
not an OSI-approved open-source license. You may reproduce, run, study, modify
and share it for Noncommercial Purposes, keeping the LICENSE, all copyright
notices and every `Required Notice:` line with each copy; private noncommercial
changes may stay private. Anyone, including a company, may evaluate it in a
non-production environment solely to assess, learn about, validate, or decide
whether to obtain commercial rights, with no fixed time limit. Monetized Use is
use that "directly or indirectly receives, earns, enables, supports, protects, or
is intended to produce money or other material financial benefit for You or
another person", including donations, tips, crowdfunding or sponsorships received
because of a GEL RAM-based activity; it needs a written Commercial Agreement
signed by the Licensor and by you before it begins: `gelram.licensing@gmail.com`.
`gel-license use ANSWERS_FILE` shows which LICENSE sentences apply to your
answers about one use; it is not legal advice. Code contributions for merge need
a [CLA 2.0](CLA.md) accepted by both sides.
This README adds no rights; the [LICENSE](LICENSE) controls
([licensing guide](LICENSING.md) · [license decision guide](docs/LICENSE-DECISION.md) · [commercial terms](COMMERCIAL-LICENSE.md) · [license pins](docs/LICENSE-PINS.md)).

</details>

## The memory core

__CORE__

<details><summary>Each part, and the test that checks it</summary>

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

</details>

## Documentation

| Start and use | Inspect and reproduce | Project |
|:---|:---|:---|
| [Project map](docs/START-HERE.md) | [Claim registry](docs/CLAIMS.md) | [Roadmap](docs/ROADMAP.md) |
| [Evidence Lab](docs/EVIDENCE-LAB.md) · [batch mode](docs/EVIDENCE-BATCH.md) | [Reproduction guide](docs/REPRODUCE.md) · [check on your hardware](docs/CHECK-ON-YOUR-HARDWARE.md) | [Release notes v0.9.0](RELEASE-NOTES-v0.9.0.md) |
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
badges and six real GIF previews. Each section leads with a picture and one
line; the full wording, with its limits, is folded under the picture in a
`<details>` block and stays in the file. The pictures added with that layout are
drawn from files too: what is checked here, private or not established; the
README's own commands with the line each prints, drawn and not a recording; the
timing set of 2026-10-20 beside its re-run; versions 0.8 and 0.9 at a glance; the
published commitments on a time line; the parts of the memory core; and what one
Q8 record holds, as docs/Q8-QUAD.md states it: 1,024 one-byte values and a
128-byte activity mask in one 1,152-byte record that four views read, and what
the views are not. Under the banner a row of badges shows numbers read from
files when the README is built, each linking to the file its number comes from;
none is a CI badge. Every wide card also comes 600 wide, with text of at least 20 px, for
a phone: the picture sends a screen up to 600 px wide to it. A phone card says
less than its wide card: the speed card shows the totals, the license card a
subset of the quotes. Three
values in the text are read from files when the README is built: the
published commitments in docs/COMMITMENTS.md, the packages in Cargo.lock,
which the builder refuses to count when one comes from outside the checkout or
is not one of its crates, and the Unicode version of the normalization tables,
refused when their header and their version constant disagree. The adjacent README-MULTIMEDIA.html is a script-free local
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
map, the Unix and Linux exclusion list of the CI evidence collector and the
committed mutation matrix report. The property-map graphic shows the rule the collector
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
log replays, not the CLI's elapsed execution time. No generated mock terminal
output, unmeasured speed figure, static CI-passing badge or source-truth claim
is added: the card of commands shows only the README's command lines and the
line to look for, and says it is drawn.

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
/// The flow card's motion: one entrance on a 12-second clock, played once and
/// held, so nothing that has appeared goes away again. Start states live only
/// in the keyframes (filled backwards), so without animation the whole flow
/// shows. Only the dashes of the beams (d1..d3) keep moving; the changed cell
/// pulses six times once its copy has landed, then holds.
const SCENE_STYLE: &str = r#".sheet,.snap,.ok,.bad{transform-box:fill-box;transform-origin:center}
.s0{animation:sc-s0 12s ease both}.s1{animation:sc-s1 12s ease both}.s2{animation:sc-s2 12s ease both}
.snap{animation:sc-snap 12s ease both}.ok{animation:sc-ok 12s ease both}.bad{animation:sc-bad 12s ease both}
.b1{animation:sc-b1 12s ease both}.b2{animation:sc-b2 12s ease both}.b3{animation:sc-b3 12s ease both}
.d1{animation:sc-b1 12s ease both,sc-flow 1s linear infinite}.d2{animation:sc-b2 12s ease both,sc-flow 1s linear infinite}.d3{animation:sc-b3 12s ease both,sc-flow 1s linear infinite}
.hot{animation:sc-hot 1s ease 8.4s 6 alternate both}
@keyframes sc-s0{0%{opacity:0;transform:translateY(-40px)}6%,100%{opacity:1;transform:none}}
@keyframes sc-s1{0%,4%{opacity:0;transform:translateY(-40px)}10%,100%{opacity:1;transform:none}}
@keyframes sc-s2{0%,8%{opacity:0;transform:translateY(-40px)}14%,100%{opacity:1;transform:none}}
@keyframes sc-b1{0%,16%{opacity:0}22%,100%{opacity:1}}
@keyframes sc-snap{0%,22%{opacity:0;transform:scale(.6)}30%,100%{opacity:1;transform:none}}
@keyframes sc-b2{0%,34%{opacity:0}40%,100%{opacity:1}}
@keyframes sc-b3{0%,50%{opacity:0}56%,100%{opacity:1}}
@keyframes sc-ok{0%,42%{opacity:0;transform:translateY(-30px)}50%,100%{opacity:1;transform:none}}
@keyframes sc-bad{0%,56%{opacity:0;transform:translateY(-30px)}62%{opacity:1;transform:none}64%{transform:translateX(-6px)}66%{transform:translateX(6px)}68%{transform:translateX(-4px)}70%,100%{opacity:1;transform:none}}
@keyframes sc-hot{from{opacity:1}to{opacity:.35}}
@keyframes sc-flow{to{stroke-dashoffset:-36}}
@media (prefers-reduced-motion:reduce){*{animation:none!important}}"#;
/// Animated 3D scene of the public citation check: document sheets with a
/// highlighted quote, the pinned snapshot, the reopened copy and the refused
/// copy. It illustrates the recorded workflows; it is not a recording or a
/// timing measurement. The entrance plays once and holds.
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
    // A beam's dashed line (d1..d3) appears with its arrowhead (b1..b3) and
    // its dashes keep running.
    let beam = |a: f64, b: f64, class: &str| {
        let ((x1, y1), (x2, y2)) = (iso(o, a, -a, 30.0), iso(o, b, -b, 30.0));
        let dash = class.replacen('b', "d", 1);
        format!(
            r#"<path class="{dash}" d="M{x1:.1} {y1:.1}H{x2:.1}" stroke="{c}" stroke-width="4" stroke-dasharray="10 8" stroke-linecap="round" fill="none"/><path class="{class}" d="M{x2:.1} {y2:.1}l-12 -7v14z" fill="{c}"/>"#,
            c = t.accent
        )
    };
    let arc = format!(
        r#"<path class="d3" d="M540 96Q760 -6 985 104" stroke="{c}" stroke-width="4" stroke-dasharray="10 8" stroke-linecap="round" fill="none"/><path class="b3" d="M992 110l-15 -1 8 -12z" fill="{c}"/><text class="mono b3" x="760" y="84" text-anchor="middle" font-size="13">copy with 1 byte flipped</text>"#,
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
/// The Unix tests of the collector's exclusion list, which holds (test,
/// condition, reason) triples of string literals. A mapped test that only runs
/// on Linux is refused: the property-map cards would draw its macOS cell as
/// required.
fn evidence_exclusions(list: &str, rows: &[(String, Vec<String>)]) -> Result<Vec<String>> {
    let literals: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
    if literals.len() % 3 != 0 {
        return Err("unexpected CI exclusion list".into());
    }
    let mut unix_only = Vec::new();
    for triple in literals.chunks(3) {
        let (test, condition) = (triple[0], triple[1]);
        let mapped = rows.iter().any(|r| r.1.iter().any(|t| t == test));
        match condition {
            _ if test.is_empty() || test.contains(' ') => {
                return Err("unexpected CI exclusion list".into());
            }
            "unix" => unix_only.push(test.to_string()),
            "linux" if mapped => {
                return Err(format!("{test} runs on Linux only but a property maps it").into());
            }
            "linux" => {}
            _ => return Err("unexpected CI exclusion list".into()),
        }
    }
    Ok(unix_only)
}
/// Public evidence the facts strip and the data graphics are drawn from, read
/// from this checkout so that no number can drift from its source.
struct Evidence {
    /// Property map rows: the ID and the tests the row names.
    rows: Vec<(String, Vec<String>)>,
    /// Tests the CI evidence collector declares for Unix. Its list also holds
    /// Linux-only tests, which `evidence_exclusions` refuses in the map.
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
        let unix_only = evidence_exclusions(list, &rows)?;
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
/// Text made safe for SVG and HTML text and attribute values.
fn xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}
fn themed(light: &str, dark: &str, alt: &str, width: u32) -> String {
    let alt = xml(alt);
    let phone = light
        .strip_prefix("media/presentation/")
        .and_then(|p| p.strip_suffix("-light.svg"))
        .map(phone_sources)
        .unwrap_or_default();
    format!(
        r#"<picture>{phone}<source media="(prefers-color-scheme: dark)" srcset="{dark}"><img alt="{alt}" src="{light}" width="{width}"></picture>"#
    )
}
/// The README header: a phone card on a narrow screen, a still card with reduced
/// motion, otherwise the moving card in the reader's theme.
fn header_picture() -> String {
    let phone = phone_sources("header").replace("><", ">\n  <");
    let phone = if phone.is_empty() {
        phone
    } else {
        format!("\n  {phone}")
    };
    format!(
        r#"<picture>{phone}
  <source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/presentation/header-still-dark.svg">
  <source media="(prefers-reduced-motion: reduce)" srcset="media/presentation/header-still-light.svg">
  <source media="(prefers-color-scheme: dark)" srcset="media/presentation/header-dark.svg">
  <img alt="GEL RAM. Knowledge printed, not trained. One stored record, four exact views, 1,152 bytes. Read back exactly; it can say UNKNOWN; a process kill, not a power cut, loses nothing it confirmed. Logo: a glass cube of record cells around an accent core." src="media/presentation/header-light.svg" width="1200">
</picture>"#
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
/// Length of the record card's entrance, in seconds. It plays once and holds
/// its last frame, so the card never goes blank.
const RECORD_ENTRANCE: u32 = 12;
/// One CSS animation of the record card's entrance, played once and held. Every
/// element is timed by its keyframe percentages alone, never by a delay, so all
/// share one clock.
fn record_motion(name: &str, timing: &str, frames: &str) -> String {
    format!(".{name}{{animation:{name} {RECORD_ENTRANCE}s {timing} both}}@keyframes {name}{{{frames}}}\n")
}
/// One stored record drawn to scale as values and mask; four views read that same
/// bar in turn and each lands on the score of a separate written-out copy; the
/// sizes it is compared with grow to scale; the verdict of every recorded run
/// follows. The entrance takes RECORD_ENTRANCE seconds and then holds: what has
/// appeared stays, only the reading windows and arrows pass. With reduced motion,
/// or without CSS animation, the complete final picture stands still.
fn record_card(dark: bool, r: &Q8Runs) -> String {
    let t = theme(dark);
    let record = grouped(r.record);
    let mut css = RESULT_STYLE.replace("MUTED", t.muted);
    css.push_str(".a{transform-box:fill-box;transform-origin:center}.grow{transform-origin:left center}.scan,.arrow{opacity:0}\n");
    // 1. The stored record appears: one bar, values then mask, to scale.
    css += &record_motion("r0", "ease-out", "0%{opacity:0}5%,100%{opacity:1}");
    css += &record_motion(
        "g0",
        "ease-out",
        "0%{opacity:0;transform:scaleX(0)}1%{opacity:1}7%,100%{opacity:1;transform:none}",
    );
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
                "0%,{}%{{opacity:.35}}{s}%,{e}%{{opacity:.7}}{}%,100%{{opacity:1}}",
                s - 1,
                e + 1
            ),
        );
        css += &record_motion(&format!("c{i}"), "ease-out", &format!("0%,{e}%{{opacity:0;transform:scale(.6)}}{}%{{opacity:1;transform:scale(1.15)}}{}%,100%{{opacity:1;transform:none}}", e + 2, e + 4));
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
    // After the sweeps, and in the still picture, one line says the same.
    css += &record_motion("id", "ease-out", "0%,53%{opacity:0}56%,100%{opacity:1}");
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
        css += &record_motion(
            &format!("z{k}"),
            "ease-out",
            &format!(
                "0%,{a}%{{opacity:1;transform:scaleX(0)}}{}%,100%{{opacity:1;transform:none}}",
                a + 8
            ),
        );
        css += &record_motion(
            &format!("n{k}"),
            "ease-out",
            &format!("0%,{}%{{opacity:0}}{}%,100%{{opacity:1}}", a + 6, a + 9),
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
    css += &record_motion(
        "vd",
        "ease-out",
        "0%,75%{opacity:0;transform:scale(.97)}79%,100%{opacity:1;transform:none}",
    );
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
/// The license card quotes LICENSE word for word and adds no words of its own to
/// the quotes; it is not drawn when a quoted sentence is no longer in the file.
const LICENSE_FILE: &str = "LICENSE";
/// One column per subject: the LICENSE sections quoted, heading, quoted sentences.
const LICENSE_COLUMNS: [(&str, &str, [&str; 2]); 3] = [
    (
        "section 3",
        "Noncommercial use",
        [
            "a worldwide, non-exclusive, non-transferable, non-sublicensable, royalty-free copyright license to reproduce, run, study, modify, and share the Software for Noncommercial Purposes.",
            "You may keep Noncommercial Modifications private without a time limit.",
        ],
    ),
    (
        "sections 4 and 2",
        "Evaluation, by anyone",
        [
            "Any person or organization may evaluate the Software and may create Evaluation Modifications without payment and without a fixed evaluation deadline.",
            "in a non-production environment solely to assess, learn about, validate, or decide whether to obtain commercial rights.",
        ],
    ),
    (
        "section 5",
        "Money needs an agreement",
        [
            "No Monetized Use is licensed by these public terms.",
            "Before beginning a Monetized Use, You must obtain a Commercial Agreement from the Licensor.",
        ],
    ),
];
/// What section 8 asks every copy to carry.
const LICENSE_NOTICE_QUOTE: &str = "each recipient receives this license, all copyright notices, and all plain-text lines beginning with `Required Notice:` supplied by the Licensor.";
/// Every quoted sentence, checked against LICENSE with its line breaks folded.
fn license_quotes(license: &str) -> Result<Vec<&'static str>> {
    let text = license.split_whitespace().collect::<Vec<_>>().join(" ");
    let quotes: Vec<&'static str> = LICENSE_COLUMNS
        .iter()
        .flat_map(|(_, _, q)| q.iter().copied())
        .chain([LICENSE_NOTICE_QUOTE])
        .collect();
    for q in &quotes {
        if !text.contains(q) {
            return Err(format!("LICENSE no longer says: {q}").into());
        }
    }
    Ok(quotes)
}
/// Words in lines of at most `max` characters.
fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= max => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}
/// What the license card says, as its description and as the README alternative text.
fn license_text() -> String {
    let mut s = String::from(
        "GEL RAM Noncommercial Reciprocal License 1.0 at a glance, quoted from LICENSE.",
    );
    for (section, heading, quotes) in LICENSE_COLUMNS {
        // Single quotes: the text is also an HTML attribute value.
        s.push_str(&format!(
            " {heading} ({section}): '{}' '{}'",
            quotes[0], quotes[1]
        ));
    }
    s.push_str(&format!(" Every copy (section 8): '{LICENSE_NOTICE_QUOTE}' Commercial Agreements: gelram.licensing@gmail.com. A summary adds no rights; LICENSE controls (section 17)."));
    s
}
/// Three columns quoted from LICENSE: what is free, what anyone may evaluate,
/// and what needs a signed agreement; then what every copy must carry.
fn license_card(dark: bool) -> Result<String> {
    license_quotes(&fs::read_to_string(LICENSE_FILE)?)?;
    let t = theme(dark);
    let mut b = String::from(
        r#"<text class="kick" x="40" y="44">LICENSE AT A GLANCE</text><text x="40" y="78" font-size="24" class="b">GEL RAM Noncommercial Reciprocal License 1.0</text><text x="40" y="102" font-size="14" class="mut">Source-available, not an OSI-approved open-source license. Each quoted line below is from LICENSE.</text>"#,
    );
    let colours = [t.green, t.blue, t.amber];
    for (i, (section, heading, quotes)) in LICENSE_COLUMNS.iter().enumerate() {
        let x = 40.0 + 380.0 * i as f64;
        let c = colours[i];
        // A check for what is granted; a signature line for what needs an agreement.
        let icon = if i < 2 {
            format!(
                r#"<circle cx="{}" cy="146" r="15" fill="{}"/><path d="M{} 146l5 5 10-10" stroke="{}" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round"/>"#,
                x + 34.0,
                c[1],
                x + 26.0,
                t.bg[0]
            )
        } else {
            format!(
                r#"<rect x="{}" y="131" width="24" height="30" rx="3" fill="{}"/><path d="M{} 141h12M{} 147h12M{} 155c3-4 5 2 8-2" stroke="{}" stroke-width="2" fill="none" stroke-linecap="round"/>"#,
                x + 22.0,
                c[1],
                x + 28.0,
                x + 28.0,
                x + 28.0,
                t.bg[0]
            )
        };
        let mut text = String::new();
        let mut y = 200.0;
        for q in quotes {
            let lines = wrap(q, 44);
            for (k, line) in lines.iter().enumerate() {
                let open = if k == 0 { "\"" } else { "" };
                let close = if k + 1 == lines.len() { "\"" } else { "" };
                text.push_str(&format!(
                    r#"<text x="{}" y="{y}" font-size="13">{open}{line}{close}</text>"#,
                    x + 28.0
                ));
                y += 18.0;
            }
            y += 10.0;
        }
        b.push_str(&format!(
            r#"<g class="col" style="animation-delay:{:.2}s"><rect x="{x}" y="120" width="360" height="252" rx="16" fill="{}" fill-opacity=".10" stroke="{}" stroke-width="2"/><rect x="{}" y="190" width="3" height="{:.0}" rx="1.5" fill="{}"/>{icon}<text x="{}" y="152" font-size="18" class="b">{heading}</text><text class="mono" x="{}" y="172" font-size="12">LICENSE {section}</text>{text}</g>"#,
            0.2 + 0.3 * i as f64,
            c[1],
            c[1],
            x + 16.0,
            y - 200.0,
            c[1],
            x + 62.0,
            x + 62.0
        ));
    }
    let mut notice = String::new();
    let lines = wrap(LICENSE_NOTICE_QUOTE, 120);
    for (k, line) in lines.iter().enumerate() {
        let open = if k == 0 { "\"" } else { "" };
        let close = if k + 1 == lines.len() { "\"" } else { "" };
        notice.push_str(&format!(
            r#"<text x="40" y="{}" font-size="13">{open}{line}{close}</text>"#,
            412 + 18 * k
        ));
    }
    b.push_str(&format!(
        r#"<g class="col" style="animation-delay:1.2s"><text class="kick" x="40" y="394">EVERY COPY · SECTION 8</text>{notice}<text x="40" y="462" font-size="14" class="b" style="fill:{}">Commercial Agreements: gelram.licensing@gmail.com</text><text x="1160" y="462" font-size="13" class="mut" text-anchor="end">A summary adds no rights; LICENSE controls (section 17).</text></g>"#,
        t.accent
    ));
    let style = format!(".col{{animation:rise .7s cubic-bezier(.2,.8,.2,1) both}}@keyframes rise{{from{{opacity:0;transform:translateY(14px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{.col{{animation:none}}}}{}", RESULT_STYLE.replace("MUTED", t.muted));
    Ok(card(
        (1200, 484),
        t,
        ("License at a glance", &license_text()),
        &style,
        &b,
    ))
}
fn license_picture() -> String {
    themed(
        "media/presentation/license-light.svg",
        "media/presentation/license-dark.svg",
        &license_text(),
        1200,
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
/// The name of the timing set: the last part of its directory.
fn speed_set_name() -> &'static str {
    SPEED_DIR.rsplit('/').next().unwrap_or(SPEED_DIR)
}
/// How the set's results are arranged, as the description of either card says it.
const SPEED_GROUPED: &str = "Cases are grouped by timing program";
/// The gains of the SHA-256 program in the words of the description, if the
/// set has that program timed.
fn speed_sha_gains(s: &SpeedSet) -> Option<String> {
    let e = s.examples.iter().find(|e| e.name == "speed_sha256")?;
    let gains: Vec<String> = e.cases().iter().map(|c| ratio(c.gain)).collect();
    Some(format!("SHA-256 alone at {}", gains.join(" and ")))
}
/// What the speed card says, as its description and as the README alternative
/// text. Like the card it names timing programs, never a case.
fn speed_text(s: &SpeedSet) -> String {
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let sha = speed_sha_gains(s).map_or(String::new(), |g| format!(" ({g})"));
    let notes: Vec<&str> = s
        .examples
        .iter()
        .filter_map(|e| e.note.as_deref())
        .collect();
    let set = speed_set_name();
    format!("Timing set {set}: v0.6.1 against main at {SPEED_B}, the first v0.7 speed package. {} timed cases in {timed} of {} timing programs, one row per program in name order: the range from the lowest to the highest v0.6.1 median time divided by the new one over the cases of the program, and their median, on a log scale centred at 1 over the widest A/A noise band of those cases. {SPEED_GROUPED}; per-case results are in {SPEED_DIR}. {faster} cases are faster beyond their A/A band, {slower} slower beyond it{sha} and {within} inside it. {same} of {timed} timed programs printed the same output digest in all three trees. {} Machine: {}; {}; not pinned to cores. Ratios of different programs are not comparable with each other; the output digests, not the timings, are the claim.", s.cases().count(), s.examples.len(), notes.join(" "), s.machine, s.load)
}
/// One row of the speed card: what the cases of one timing program add up to.
/// The card names no case; the per-case results stay in the timing set.
struct SpeedRow {
    min: f64,
    median: f64,
    max: f64,
    /// The widest A/A band of its cases: the lowest low end to the highest high end.
    band: (f64, f64),
    /// Its cases faster beyond their own A/A band, slower beyond it, inside it.
    sides: [usize; 3],
}
impl SpeedRow {
    /// The row of one example, or `None` when it has no timed case. The median
    /// of an even number of gains is the mean of the middle two.
    fn of(e: &SpeedExample) -> Option<SpeedRow> {
        let cases = e.cases();
        let mut gains: Vec<f64> = cases.iter().map(|c| c.gain).collect();
        gains.sort_by(f64::total_cmp);
        let (min, max) = (*gains.first()?, *gains.last()?);
        let mid = gains.len() / 2;
        let median = if gains.len() % 2 == 1 {
            gains[mid]
        } else {
            (gains[mid - 1] + gains[mid]) / 2.0
        };
        let band = cases
            .iter()
            .map(SpeedCase::noise)
            .fold((f64::INFINITY, 0.0_f64), |(a, b), (lo, hi)| {
                (a.min(lo), b.max(hi))
            });
        let count = |side| cases.iter().filter(|c| c.side() == side).count();
        Some(SpeedRow {
            min,
            median,
            max,
            band,
            sides: [
                count(Side::Faster),
                count(Side::Slower),
                count(Side::Within),
            ],
        })
    }
}
/// `ONE` is the x of ratio 1: every span grows out of that axis.
const SPEED_STYLE: &str = ".mut{fill:MUTED}.b{font-weight:700}.sp{animation:grow .9s cubic-bezier(.2,.8,.2,1) both;transform-box:view-box;transform-origin:ONEpx 0}.fd{animation:fade .6s ease-out both}@keyframes grow{from{transform:scaleX(0)}to{transform:none}}@keyframes fade{from{opacity:0}to{opacity:1}}@media (prefers-reduced-motion:reduce){.sp,.fd{animation:none}}";
/// One row per timing program in name order, on a log scale centred at 1: the
/// span from its lowest to its highest gain, red below 1 and blue above, its
/// median, the widest A/A band of its cases behind them, how many cases fall
/// beyond or inside their own band, and its output verdict. No case is named.
fn speed_card(dark: bool, s: &SpeedSet) -> String {
    let t = theme(dark);
    let (row, top) = (15.0, 234.0);
    // A log2 axis in whole octaves, wide enough for every ratio and every band.
    let (mut lo, mut hi) = (0.5_f64, 2.0_f64);
    for c in s.cases() {
        let (a, b) = c.noise();
        lo = lo.min(a).min(c.gain);
        hi = hi.max(b).max(c.gain);
    }
    let (lo, hi) = (lo.log2().floor() as i32, hi.log2().ceil() as i32);
    let (p0, p1) = (424.0, 890.0);
    let x = |v: f64| p0 + (v.log2() - f64::from(lo)) / f64::from(hi - lo) * (p1 - p0);
    let one = x(1.0);
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let (least, most) = s
        .cases()
        .fold((u32::MAX, 0), |(a, b), c| (a.min(c.runs), b.max(c.runs)));
    let runs = if least == most {
        least.to_string()
    } else {
        format!("{least} to {most}")
    };
    let check = |x: f64, y: f64| {
        format!(
            r#"<path d="M{x:.1} {y:.1}l3.5 3.5 6.5-7.5" fill="none" stroke="{}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
            t.good
        )
    };
    let set = speed_set_name();
    let mut b = format!(
        r#"<text class="kick" x="40" y="42">SPEED · v0.6.1 AGAINST main AT {SPEED_B} · SET {set}</text><text x="40" y="72" font-size="22" class="b">{} timed cases in {timed} of {} timing programs: v0.6.1 time divided by the new time</text><text x="40" y="95" font-size="13.5">One row per timing program: the range of its cases from min to max, and their median · cases grouped by timing program; per-case results are in docs/speed-results</text><text class="mono" x="40" y="116" font-size="12">A, A2: two checkouts of v0.6.1 · B: main at {SPEED_B}, the first v0.7 speed package · medians of {runs} runs per case</text><text class="mono" x="40" y="133" font-size="12">Machine: {} · not pinned to cores</text><text class="mono" x="40" y="150" font-size="12">Load: {}</text>"#,
        s.cases().count(),
        s.examples.len(),
        s.machine,
        s.load
    );
    // The totals, then what each mark means.
    b.push_str(&format!(
        r#"<text x="40" y="177" font-size="13" class="b">{} cases:</text><rect x="122" y="169" width="22" height="6" rx="1.5" fill="{}"/><text x="152" y="177" font-size="13">faster beyond its A/A band: {faster}</text><rect x="374" y="169" width="22" height="6" rx="1.5" fill="{}"/><text x="404" y="177" font-size="13">slower beyond it: {slower}</text><rect x="548" y="169" width="22" height="6" rx="1.5" fill="{}"/><text x="578" y="177" font-size="13">inside it: {within}</text>{}<text x="704" y="177" font-size="13">same output digest in A, A2 and B: {same} of {timed}</text>"#,
        s.cases().count(),
        t.blue[1],
        t.red[1],
        t.muted,
        check(686.0, 172.0)
    ));
    b.push_str(&format!(
        r#"<rect x="40" y="191" width="20" height="6" rx="1.5" fill="{}"/><rect x="60" y="191" width="20" height="6" rx="1.5" fill="{}"/><path d="M60 187v14" stroke="{}" stroke-width="1.4"/><text x="88" y="198" font-size="12.5">range of its cases, min to max: red below 1, blue above 1</text><circle cx="436" cy="194" r="3.6" fill="{}" stroke="{}" stroke-width="1.2"/><text x="446" y="198" font-size="12.5">median of its cases</text><rect x="578" y="189" width="30" height="10" rx="2" fill="{}" fill-opacity=".2"/><text x="616" y="198" font-size="12.5">widest A/A band of its cases: the noise to read it against</text>"#,
        t.red[1],
        t.blue[1],
        t.ink,
        t.ink,
        t.bg[0],
        t.muted
    ));
    let bottom = top + row * s.examples.len() as f64;
    b.push_str(&format!(
        r#"<g class="mut b" font-size="11"><text x="{:.1}" y="214" text-anchor="end">slower</text><text x="{:.1}" y="214">faster</text><text x="40" y="228">timing program</text><text x="262" y="228" text-anchor="end">cases</text><text x="306" y="228" text-anchor="end">faster</text><text x="350" y="228" text-anchor="end">slower</text><text x="394" y="228" text-anchor="end">inside</text><text x="944" y="228" text-anchor="end">min</text><text x="1000" y="228" text-anchor="end">median</text><text x="1054" y="228" text-anchor="end">max</text><text x="1070" y="228">output</text></g>"#,
        one - 6.0,
        one + 6.0
    ));
    for k in lo..=hi {
        let v = 2f64.powi(k);
        let (stroke, width) = if k == 0 { (t.ink, 1.4) } else { (t.line, 1.0) };
        b.push_str(&format!(
            r#"<path d="M{gx:.1} {:.0}V{bottom:.0}" stroke="{stroke}" stroke-width="{width}"/><text class="mono" x="{gx:.1}" y="228" text-anchor="middle" font-size="10">{v}</text>"#,
            top - 2.0,
            gx = x(v)
        ));
    }
    let mut notes: Vec<&str> = Vec::new();
    for (k, e) in s.examples.iter().enumerate() {
        // Rows appear one after another; a span grows out of 1, then its median shows.
        let (y0, kf) = (top + row * k as f64, k as f64);
        let (cy, ty) = (y0 + row / 2.0, y0 + row / 2.0 + 4.0);
        let (fade, grow, mark) = (0.1 + 0.04 * kf, 0.3 + 0.04 * kf, 1.0 + 0.04 * kf);
        if k % 2 == 0 {
            b.push_str(&format!(
                r#"<rect x="32" y="{y0:.1}" width="1136" height="{row:.0}" fill="{}" fill-opacity=".35"/>"#,
                t.line
            ));
        }
        let note = e.note.as_deref().map(|n| {
            notes.push(n);
            notes.len()
        });
        let label = |text: String| {
            format!(r#"<text x="1086" y="{ty:.1}" font-size="11" class="mut">{text}</text>"#)
        };
        let output = match (&e.timed, note) {
            (Some((_, true)), _) => check(1072.0, cy),
            (Some((_, false)), n) => format!(
                r##"<circle cx="1077" cy="{cy:.1}" r="5.5" fill="{}"/><text x="1077" y="{:.1}" text-anchor="middle" font-size="9.5" class="b" style="fill:#1b1f24">!</text>{}"##,
                t.amber[1],
                cy + 3.4,
                label(n.map_or("differs".to_string(), |n| format!("differs · note {n}")))
            ),
            (None, n) => format!(
                r#"<circle cx="1077" cy="{cy:.1}" r="5" fill="none" stroke="{m}" stroke-width="1.5"/><path d="M1074.3 {cy:.1}h5.4" stroke="{m}" stroke-width="1.5"/>{}"#,
                label(n.map_or(String::new(), |n| format!("note {n}"))),
                m = t.muted
            ),
        };
        let name = format!(
            r#"<text class="pn b" x="40" y="{ty:.1}" font-size="12">{}</text>"#,
            e.name
        );
        let Some(r) = SpeedRow::of(e) else {
            b.push_str(&format!(
                r#"<g class="fd" style="animation-delay:{fade:.2}s">{name}<text x="{:.1}" y="{ty:.1}" font-size="11" class="mut" font-style="italic">not built in v0.6.1: no timing</text>{output}</g>"#,
                one + 8.0
            ));
            continue;
        };
        let cell = |cx: f64, n: usize, colour: &str| {
            if n == 0 {
                format!(
                    r#"<text x="{cx:.0}" y="{ty:.1}" font-size="11.5" class="mut" text-anchor="end">0</text>"#
                )
            } else {
                format!(
                    r#"<text x="{cx:.0}" y="{ty:.1}" font-size="11.5" class="b" text-anchor="end" style="fill:{colour}">{n}</text>"#
                )
            }
        };
        let [f, sl, w] = r.sides;
        b.push_str(&format!(
            r#"<g class="fd" style="animation-delay:{fade:.2}s">{name}<text x="262" y="{ty:.1}" font-size="11.5" text-anchor="end">{}</text>{}{}{}<text x="944" y="{ty:.1}" font-size="11.5" text-anchor="end">{}</text><text x="1000" y="{ty:.1}" font-size="11.5" class="b" text-anchor="end">{}</text><text x="1054" y="{ty:.1}" font-size="11.5" text-anchor="end">{}</text>{output}</g><rect class="fd" style="animation-delay:{fade:.2}s" x="{:.1}" y="{:.1}" width="{:.1}" height="10" rx="2" fill="{}" fill-opacity=".2"/>"#,
            f + sl + w,
            cell(306.0, f, t.blue[1]),
            cell(350.0, sl, t.red[1]),
            cell(394.0, w, t.ink),
            ratio(r.min),
            ratio(r.median),
            ratio(r.max),
            x(r.band.0),
            cy - 5.0,
            x(r.band.1) - x(r.band.0),
            t.muted
        ));
        // The span, cut at 1: the part below in red, the part above in blue. A
        // program of one case has no span, only its median mark.
        for (class, from, to, fill) in [
            ("lo", r.min, r.max.min(1.0), t.red[1]),
            ("hi", r.min.max(1.0), r.max, t.blue[1]),
        ] {
            if from < to {
                b.push_str(&format!(
                    r#"<rect class="sp {class}" style="animation-delay:{grow:.2}s" x="{:.1}" y="{:.1}" width="{:.1}" height="6" rx="1.5" fill="{fill}"/>"#,
                    x(from),
                    cy - 3.0,
                    x(to) - x(from)
                ));
            }
        }
        b.push_str(&format!(
            r#"<circle class="fd" style="animation-delay:{mark:.2}s" cx="{:.1}" cy="{cy:.1}" r="3.6" fill="{}" stroke="{}" stroke-width="1.2"/>"#,
            x(r.median),
            t.ink,
            t.bg[0]
        ));
    }
    let mut y = bottom + 4.0;
    for (i, note) in notes.iter().enumerate() {
        y += 16.0;
        b.push_str(&format!(
            r#"<text x="40" y="{y:.0}" font-size="12"><tspan class="b">Note {}</tspan> · {note}</text>"#,
            i + 1
        ));
    }
    y += 6.0;
    for line in [
        "The machine was loaded and the runs were not pinned to cores, so each grey band, the A/A ratios of two copies of v0.6.1, is the noise to read a span against: samples of that noise, not a confidence interval.",
        "A case counts as faster or slower only beyond its own band. Ratios of different programs are not comparable with each other: each program times its own work. The output digests, not the timings, are the claim.",
    ] {
        y += 16.0;
        b.push_str(&format!(
            r#"<text x="40" y="{y:.0}" font-size="12" class="mut">{line}</text>"#
        ));
    }
    card(
        (1200, (y + 18.0) as u32),
        t,
        (
            "Speed against v0.6.1, one row per timing program",
            &speed_text(s),
        ),
        &SPEED_STYLE
            .replace("MUTED", t.muted)
            .replace("ONE", &format!("{one:.1}")),
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
/// The README's own pictures added with the visual layout: one card per
/// section, each drawn from the files it shows. A card is (name, drawing,
/// alternative text); the drawing comes in a light and a dark theme.
type Visual = (
    &'static str,
    fn(bool) -> Result<String>,
    fn() -> Result<String>,
);
const VISUALS: [Visual; 7] = [
    ("status", status_card, status_alt),
    ("q8", q8_card, q8_alt),
    ("terminal", terminal_card, terminal_alt),
    ("rerun", rerun_card, rerun_alt),
    ("glance", glance_card, glance_alt),
    ("pledges", pledges_card, pledges_alt),
    ("core", core_card, core_alt),
];
fn visual_path(name: &str, dark: bool) -> String {
    let theme = if dark { "dark" } else { "light" };
    format!("media/presentation/{name}-{theme}.svg")
}
/// A visual for the README; without its data the placeholder stays and fails the checks.
fn visual_picture(name: &str) -> String {
    let Some((_, draw, alt)) = VISUALS.iter().find(|v| v.0 == name) else {
        return format!("__{}__ unknown visual", name.to_ascii_uppercase());
    };
    match (draw(false), alt()) {
        (Ok(_), Ok(alt)) => themed(
            &visual_path(name, false),
            &visual_path(name, true),
            &alt,
            1200,
        ),
        (Err(e), _) | (_, Err(e)) => format!("__{}__ {e}", name.to_ascii_uppercase()),
    }
}
/// Phone versions of the wide README cards. A phone shows a 1200-wide card about
/// 358 px wide, so each wide card also comes 600 wide with larger text, and the
/// picture sends a narrow screen to it. A phone card is (name of the wide card,
/// drawing in a light or dark theme).
type Phone = (&'static str, fn(bool) -> Result<String>);
const PHONES: [Phone; 15] = [
    ("q8", q8_phone),
    ("header", header_phone),
    ("status", status_phone),
    ("glance", glance_phone),
    ("terminal", terminal_phone),
    ("record", record_phone),
    ("flow", flow_phone),
    ("speed", speed_phone),
    ("rerun", rerun_phone),
    ("bars", bars_phone),
    ("wall", wall_phone),
    ("pledges", pledges_phone),
    ("crash", crash_phone),
    ("license", license_phone),
    ("core", core_phone),
];
/// A phone card is 600 wide; its smallest text, 20 px, is about 12 px on a
/// 358-px screen.
const PHONE_WIDTH: u32 = 600;
#[cfg(test)]
const PHONE_MIN_FONT: f64 = 20.0;
const PHONE_MEDIA: &str = "(max-width: 600px)";
fn phone_path(name: &str, dark: bool) -> String {
    let theme = if dark { "dark" } else { "light" };
    format!("media/presentation/phone/{name}-{theme}.svg")
}
/// The picture sources that send a narrow screen to the phone card of `name`,
/// dark first; none while that phone card cannot be drawn.
fn phone_sources(name: &str) -> String {
    let Some((_, draw)) = PHONES.iter().find(|p| p.0 == name) else {
        return String::new();
    };
    if draw(false).is_err() || draw(true).is_err() {
        return String::new();
    }
    format!(
        r#"<source media="{PHONE_MEDIA} and (prefers-color-scheme: dark)" srcset="{}"><source media="{PHONE_MEDIA}" srcset="{}">"#,
        phone_path(name, true),
        phone_path(name, false)
    )
}
/// The shared card frame at phone width, with its small kicker raised to the
/// phone minimum. The card's own size is the phone minimum too, so a text that
/// states no size inherits 20 px, not a viewer's 16; a size the text states
/// still wins, as inheritance never overrides it.
fn phone_card(dark: bool, h: u32, text: (&str, &str), style: &str, body: &str) -> String {
    card((PHONE_WIDTH, h), theme(dark), text, style, body)
        .replacen(".kick{font-size:13px;", ".kick{font-size:20px;", 1)
        .replacen("<style>\n", "<style>\nsvg{font-size:20px}\n", 1)
}
/// Every font size a card states: `font-size` attributes and declarations, and
/// pixel sizes in `font` shorthands.
#[cfg(test)]
fn font_sizes(svg: &str) -> Vec<f64> {
    let number = |s: &str| -> Option<f64> {
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(s.len());
        s[..end].parse().ok()
    };
    let mut out = Vec::new();
    for key in ["font-size=\"", "font-size:"] {
        for part in svg.split(key).skip(1) {
            out.extend(number(part.trim_start()));
        }
    }
    for part in svg.split("font:").skip(1) {
        let decl = part
            .split(|c| c == ';' || c == '}' || c == '"')
            .next()
            .unwrap_or("");
        for word in decl.split_whitespace() {
            if let Some(px) = word.strip_suffix("px") {
                out.extend(number(px));
            }
        }
    }
    out
}

// ===== PHONE a1 BEGIN =====
/// The words of the wide header in short lines for a narrow screen; joined,
/// each group is said by the header's alternative text.
const HEADER_PHONE_TAG: [&str; 2] = ["Knowledge printed,", "not trained"];
const HEADER_PHONE_RECORD: [&str; 2] = ["One stored record,", "four exact views"];
/// The checked claims of the wide header, one box each: read back exactly
/// (drawn in the box of the record), and the limit of the crash series.
const HEADER_PHONE_CLAIMS: [&[&str]; 2] = [
    &["Read back exactly"],
    &[
        "A process kill,",
        "not a power cut,",
        "loses nothing it confirmed",
    ],
];
/// The phone banner, short enough that the first screen of a phone reaches
/// the text: the logo beside the name and the line under it, then the record
/// with its size, read back exactly, and the limit of the crash series. The
/// byte count of the record is `record`, which `header_phone` reads from the
/// recorded comparison runs.
fn header_phone_draw(dark: bool, record: usize) -> Result<String> {
    let t = theme(dark);
    let accent = if dark { "#79c0ff" } else { "#0969da" };
    let (x0, w) = (24.0, 552.0);
    let mut roll = GlanceRoll::new();
    let mut b = format!(
        r#"<defs><radialGradient id="hpglow"><stop offset="0" stop-color="{0}" stop-opacity=".28"/><stop offset="1" stop-color="{0}" stop-opacity="0"/></radialGradient></defs><ellipse cx="118" cy="140" rx="120" ry="116" fill="url(#hpglow)"/><g class="hl">{1}</g>"#,
        t.accent,
        gel_logo(t, accent, (108.0, 142.0), 84.0)
    );
    // The name and the line under it, beside the logo.
    let (nx, nroom) = (232.0, f64::from(PHONE_WIDTH) - 24.0 - 232.0);
    status_fit("GEL RAM", 62.0, 2.0, nroom)?;
    b.push_str(&format!(
        r#"<text class="hn b" x="{nx}" y="122" font-size="62" letter-spacing="2">GEL RAM</text>"#
    ));
    for (k, line) in HEADER_PHONE_TAG.iter().enumerate() {
        status_fit(line, 28.0, 0.0, nroom)?;
        b.push_str(&format!(
            r#"<text class="ht" x="{nx}" y="{}" font-size="28" style="animation-delay:{:.2}s">{}</text>"#,
            168 + 36 * k,
            0.5 + 0.15 * k as f64,
            xml(line)
        ));
    }
    // One box per claim, each with a check mark drawn in.
    let check = |cy: f64, c: &str, delay: f64| {
        format!(
            r#"<g transform="translate({} {cy:.1})"><circle r="20" fill="{c}" fill-opacity=".15" stroke="{c}" stroke-width="2.5"/><path class="hc" style="animation-delay:{delay:.2}s" pathLength="100" stroke-dasharray="100" d="M-9 1L-3 7L10 -7" fill="none" stroke="{c}" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round"/></g>"#,
            x0 + 34.0
        )
    };
    let (tx, room) = (x0 + 72.0, w - 72.0 - 16.0);
    let mut y = 256.0;
    // The record: read back exactly, its two lines, then its size rolling
    // into place.
    let size = grouped(record);
    let wide: f64 = size.chars().map(|c| GlanceRoll::advance(c, 60.0)).sum();
    status_fit("bytes", 30.0, 0.0, room - wide - 14.0)?;
    let h = 196.0;
    b.push_str(&format!(
        r#"<g class="hb" style="animation-delay:.9s"><rect x="{x0}" y="{y}" width="{w}" height="{h}" rx="18" fill="{c}" fill-opacity=".08" stroke="{c}" stroke-opacity=".45"/>{}"#,
        check(y + 34.0, t.accent, 1.3),
        c = t.accent
    ));
    for line in HEADER_PHONE_CLAIMS[0] {
        status_fit(line, 26.0, 0.0, room)?;
        b.push_str(&format!(
            r#"<text x="{tx}" y="{:.1}" font-size="26" class="b">{}</text>"#,
            y + 43.0,
            xml(line)
        ));
    }
    for (k, line) in HEADER_PHONE_RECORD.iter().enumerate() {
        status_fit(line, 24.0, 0.0, room)?;
        b.push_str(&format!(
            r#"<text x="{tx}" y="{:.1}" font-size="24">{}</text>"#,
            y + 79.0 + 30.0 * k as f64,
            xml(line)
        ));
    }
    roll.put((tx, y + 172.0), 60.0, &size, 1.2, t.accent);
    b.push_str(&std::mem::take(&mut roll.body));
    b.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="30" class="b mut">bytes</text></g>"#,
        tx + wide + 14.0,
        y + 172.0
    ));
    y += h + 14.0;
    for (k, lines) in HEADER_PHONE_CLAIMS.iter().enumerate().skip(1) {
        let delay = 1.2 + 0.3 * k as f64;
        let h = 40.0 + 32.0 * lines.len() as f64;
        b.push_str(&format!(
            r#"<g class="hb" style="animation-delay:{delay:.2}s"><rect x="{x0}" y="{y}" width="{w}" height="{h}" rx="18" fill="{c}" fill-opacity=".08" stroke="{c}" stroke-opacity=".45"/>{}"#,
            check(y + 36.0, t.good, delay + 0.4),
            c = t.good
        ));
        for (i, line) in lines.iter().enumerate() {
            status_fit(line, 26.0, 0.0, room)?;
            b.push_str(&format!(
                r#"<text x="{tx}" y="{:.1}" font-size="26">{}</text>"#,
                y + 45.0 + 32.0 * i as f64,
                xml(line)
            ));
        }
        b.push_str("</g>");
        y += h + 14.0;
    }
    let (roll_rules, stilled) = roll.motion();
    let style = format!(
        ".l0,.l1,.l2,.hit,.seal{{transform-box:fill-box;transform-origin:center}}.hl{{animation:hpfade .8s ease-out both}}.l0{{animation:hppop .5s cubic-bezier(.2,.8,.2,1) .2s both}}.l1{{animation:hppop .5s cubic-bezier(.2,.8,.2,1) .4s both}}.l2{{animation:hppop .5s cubic-bezier(.2,.8,.2,1) .6s both}}.ray{{animation:hpray .7s ease-out .9s both}}.hit{{animation:hphit .6s ease-out 1.4s both}}.seal{{animation:hphit .6s ease-out 1.8s both}}.hn{{animation:hprise .7s cubic-bezier(.2,.8,.2,1) .3s both}}.ht,.hb{{animation:hprise .7s cubic-bezier(.2,.8,.2,1) both}}.hc{{animation:hpdraw .6s ease-out both}}{roll_rules}@keyframes hpfade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes hppop{{from{{opacity:0;transform:scale(.3)}}to{{opacity:1;transform:none}}}}@keyframes hpray{{from{{opacity:0;stroke-dashoffset:60}}to{{opacity:1;stroke-dashoffset:0}}}}@keyframes hphit{{0%{{opacity:0;transform:scale(.6)}}60%{{opacity:1;transform:scale(1.2)}}to{{opacity:1;transform:none}}}}@keyframes hprise{{from{{opacity:0;transform:translateY(24px)}}to{{opacity:1;transform:none}}}}@keyframes hpdraw{{from{{stroke-dashoffset:100}}to{{stroke-dashoffset:0}}}}{}@media (prefers-reduced-motion:reduce){{.hl,.l0,.l1,.l2,.ray,.hit,.seal,.hn,.ht,.hb,.hc,{stilled}{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    let desc = format!(
        "The logo: a translucent cube of source cells with one exact cell lit and a pin seal. GEL RAM. Knowledge printed, not trained. One stored record, four exact views, {size} bytes. Read back exactly; a process kill, not a power cut, loses nothing it confirmed."
    );
    Ok(phone_card(
        dark,
        (y + 2.0).ceil() as u32,
        (
            &xml("GEL RAM — Knowledge printed, not trained"),
            &xml(&desc),
        ),
        &style,
        &b,
    ))
}
/// The phone banner; the size of the record comes from the recorded runs the
/// record card is drawn from.
fn header_phone(dark: bool) -> Result<String> {
    header_phone_draw(dark, Q8Runs::read()?.record)
}
/// The longest line of an item on the status phone card, in characters.
const STATUS_PHONE_WRAP: usize = 36;
/// The status card stacked: what can be checked here, what is private, what is
/// not established, one block under the other, each foot on the heading's line;
/// each icon drawn in and the count of published commitments rolling into
/// place. A line that does not fit its room is refused, not clipped.
fn status_phone_draw(dark: bool, f: &StatusFacts) -> Result<String> {
    f.check()?;
    let t = theme(dark);
    let colours = [t.good, t.accent, if dark { t.amber[0] } else { t.amber[2] }];
    let (x0, w) = (24.0, 552.0);
    let kicker = "WHERE GEL RAM STANDS";
    status_fit(kicker, 20.0, 2.0, w)?;
    let mut b = format!(r#"<text class="kick" x="32" y="44">{}</text>"#, xml(kicker));
    let mut roll = GlanceRoll::new();
    let mut y = 58.0;
    for (i, (heading, _, items)) in STATUS_COLUMNS.iter().enumerate() {
        let (c, delay) = (colours[i], 0.1 + 0.5 * i as f64);
        let (hx, right) = (x0 + 72.0, x0 + w - 16.0);
        // The foot of a column stands at the end of its heading's line.
        let foot = match i {
            0 => Some((STATUS_FEET[0], "")),
            2 => Some((STATUS_FEET[1], "mono ")),
            _ => None,
        };
        let foot_w = foot.map_or(0.0, |(words, _)| {
            words.chars().count() as f64 * 0.62 * 20.0 / 0.92 + 8.0
        });
        status_fit(heading, 24.0, 0.0, right - hx - foot_w)?;
        let mut s = format!(
            r#"<g transform="translate({} {})"><g class="sq" style="animation-delay:{:.2}s"><circle r="22" fill="{c}" fill-opacity=".14" stroke="{c}" stroke-width="2.5"/>{}</g></g><text x="{hx}" y="{}" font-size="24" class="b" style="fill:{c}">{}</text><path d="M{} {}H{}" stroke="{c}" stroke-opacity=".35" stroke-width="1.5"/>"#,
            x0 + 40.0,
            y + 34.0,
            delay + 0.6,
            status_icon(i, c, t.bg[0]),
            y + 43.0,
            xml(heading),
            x0 + 16.0,
            y + 62.0,
            right
        );
        if let Some((words, mono)) = foot {
            s.push_str(&format!(
                r#"<text class="{mono}b" x="{right}" y="{:.1}" font-size="20" text-anchor="end" style="fill:{c}">{}</text>"#,
                y + 43.0,
                xml(words)
            ));
        }
        let mut ly = y + 86.0;
        for (k, item) in items.iter().enumerate() {
            s.push_str(&format!(
                r#"<g class="si" style="animation-delay:{:.2}s"><circle cx="{}" cy="{:.1}" r="5" fill="{c}"/>"#,
                delay + 0.3 + 0.15 * k as f64,
                x0 + 26.0,
                ly - 7.0
            ));
            for line in wrap(item, STATUS_PHONE_WRAP) {
                status_fit(&line, 20.0, 0.0, right - x0 - 42.0)?;
                s.push_str(&format!(
                    r#"<text x="{}" y="{ly:.1}" font-size="20">{}</text>"#,
                    x0 + 42.0,
                    xml(&line)
                ));
                ly += 26.0;
            }
            s.push_str("</g>");
            ly += 4.0;
        }
        let mut bottom = ly - 14.0;
        if i == 1 {
            // The count rolls into place; three short lines beside it.
            let (foot, box_h) = (ly - 10.0, 84.0);
            s.push_str(&format!(
                r#"<rect x="{}" y="{foot:.1}" width="{}" height="{box_h}" rx="12" fill="{c}" fill-opacity=".13"/>"#,
                x0 + 16.0,
                w - 32.0
            ));
            let n = f.commitments.to_string();
            let wide: f64 = n.chars().map(|ch| GlanceRoll::advance(ch, 52.0)).sum();
            let tx = x0 + 36.0 + wide + 18.0;
            for (k, words) in STATUS_PLEDGE.iter().enumerate() {
                status_fit(words, 20.0, 0.0, x0 + w - 28.0 - tx)?;
                let bold = if k == 0 { r#" class="b""# } else { "" };
                s.push_str(&format!(
                    r#"<text{bold} x="{tx:.1}" y="{:.1}" font-size="20">{}</text>"#,
                    foot + 22.0 + 24.0 * k as f64,
                    xml(words)
                ));
            }
            roll.put((x0 + 36.0, foot + 60.0), 52.0, &n, delay + 0.6, c);
            s.push_str(&std::mem::take(&mut roll.body));
            bottom = foot + box_h + 14.0;
        }
        let h = bottom - y;
        b.push_str(&format!(
            r#"<g class="sc" style="animation-delay:{delay:.2}s"><clipPath id="spcol{i}"><rect x="{x0}" y="{y}" width="{w}" height="{h:.1}" rx="20"/></clipPath><rect x="{x0}" y="{y}" width="{w}" height="{h:.1}" rx="20" fill="{c}" fill-opacity=".07" stroke="{c}" stroke-opacity=".45"/><rect x="{x0}" y="{y}" width="{w}" height="6" fill="{c}" clip-path="url(#spcol{i})"/>{s}</g>"#
        ));
        y += h + 10.0;
    }
    let (roll_rules, stilled) = roll.motion();
    let style = format!(
        ".sc{{animation:sprise .75s cubic-bezier(.2,.8,.2,1) both}}.si{{animation:spfade .6s ease-out both}}.sd{{animation:spdraw .9s ease-out both}}.sq{{animation:sppulse .7s ease-in-out both;transform-box:fill-box;transform-origin:center}}{roll_rules}@keyframes sprise{{from{{opacity:0;transform:translateY(34px)}}to{{opacity:1;transform:none}}}}@keyframes spfade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes spdraw{{from{{stroke-dashoffset:100}}to{{stroke-dashoffset:0}}}}@keyframes sppulse{{0%,100%{{transform:none}}50%{{transform:scale(1.16)}}}}{}@media (prefers-reduced-motion:reduce){{.sc,.si,.sd,.sq,{stilled}{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    Ok(phone_card(
        dark,
        (y + 8.0).ceil() as u32,
        (&xml("Where GEL RAM stands"), &xml(&status_text(f))),
        &style,
        &b,
    ))
}
fn status_phone(dark: bool) -> Result<String> {
    status_phone_draw(dark, &StatusFacts::read()?)
}
/// The glance card stacked: four low tiles one under the other, each tagged
/// with the version that brought it, as on the wide card; numbers roll into place.
fn glance_phone_draw(dark: bool, f: &GlanceFacts) -> Result<String> {
    f.check()?;
    let t = theme(dark);
    let (x0, w) = (24.0, 552.0);
    let mut roll = GlanceRoll::new();
    let kicker = format!("{} AND {} AT A GLANCE", f.since, f.release);
    glance_fit(&kicker, 20.0, 2.0, w)?;
    let mut b = format!(
        r#"<text class="kick" x="32" y="44">{}</text>"#,
        xml(&kicker)
    );
    let new = format!("new in {}", f.release);
    let tiles = glance_tiles(t, f);
    let (h, gap) = (126.0, 8.0);
    let mut y = 58.0;
    for (k, (big, tag, colour, l1, l2, c2)) in tiles.iter().enumerate() {
        let delay = 0.15 + 0.3 * k as f64;
        b.push_str(&format!(
            r#"<g class="gt" style="animation-delay:{delay:.2}s"><clipPath id="gptile{k}"><rect x="{x0}" y="{y}" width="{w}" height="{h}" rx="18"/></clipPath><rect x="{x0}" y="{y}" width="{w}" height="{h}" rx="18" fill="{colour}" fill-opacity=".07" stroke="{line}"/><rect x="{x0}" y="{y}" width="{w}" height="6" fill="{colour}" clip-path="url(#gptile{k})"/>"#,
            line = t.line
        ));
        // The version tag in the upper right: filled when new in this release.
        let pw = tag.chars().count() as f64 * 0.62 * 20.0 + 32.0;
        let px = x0 + w - 24.0 - pw;
        let (tag_fill, tag_ink) = if *tag == new {
            (format!(r#"fill="{}""#, t.accent), t.bg[0])
        } else {
            (
                format!(r#"fill="none" stroke="{}" stroke-width="2""#, t.muted),
                t.muted,
            )
        };
        b.push_str(&format!(
            r#"<rect x="{px:.1}" y="{}" width="{pw:.1}" height="32" rx="16" {tag_fill}/><text x="{:.1}" y="{}" font-size="20" class="b" text-anchor="middle" style="fill:{tag_ink}">{}</text>"#,
            y + 14.0,
            px + pw / 2.0,
            y + 37.0,
            xml(tag)
        ));
        let room = px - x0 - 40.0;
        if big.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
            let wide: f64 = big.chars().map(|c| GlanceRoll::advance(c, 52.0)).sum();
            if wide > 0.92 * room {
                return Err(format!("{big} does not fit its tile").into());
            }
            roll.put((x0 + 24.0, y + 56.0), 52.0, big, delay + 0.3, colour);
            b.push_str(&std::mem::take(&mut roll.body));
        } else {
            glance_fit(big, 46.0, 0.0, room)?;
            b.push_str(&format!(
                r#"<text class="gp" style="animation-delay:{:.2}s;fill:{colour}" x="{}" y="{}" font-size="46" font-weight="700">{}</text>"#,
                delay + 0.4,
                x0 + 24.0,
                y + 54.0,
                xml(big)
            ));
        }
        glance_fit(l1, 22.0, 0.0, w - 48.0)?;
        glance_fit(l2, 20.0, 0.0, w - 48.0)?;
        let mono = if l2.starts_with("--") { "mono " } else { "" };
        b.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="22">{}</text><text class="{mono}b" x="{}" y="{}" font-size="20" style="fill:{c2}">{}</text></g>"#,
            x0 + 24.0,
            y + 87.0,
            xml(l1),
            x0 + 24.0,
            y + 113.0,
            xml(l2)
        ));
        y += h + gap;
    }
    let (roll_rules, stilled) = roll.motion();
    let style = format!(
        ".gt{{animation:gprise .7s cubic-bezier(.2,.8,.2,1) both}}.gp{{animation:gppop .8s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:left bottom}}{roll_rules}@keyframes gprise{{from{{opacity:0;transform:translateY(26px)}}to{{opacity:1;transform:none}}}}@keyframes gppop{{0%{{opacity:0;transform:scale(.6)}}70%{{opacity:1;transform:scale(1.06)}}to{{opacity:1;transform:none}}}}{}@media (prefers-reduced-motion:reduce){{.gt,.gp,{stilled}{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    Ok(phone_card(
        dark,
        (y - gap + 18.0).ceil() as u32,
        (
            &xml(&format!(
                "Versions {} and {} at a glance",
                f.since, f.release
            )),
            &xml(&glance_text(f)),
        ),
        &style,
        &b,
    ))
}
fn glance_phone(dark: bool) -> Result<String> {
    glance_phone_draw(dark, &GlanceFacts::read()?)
}
// ===== PHONE a1 END =====

// ===== PHONE a2 BEGIN =====
/// Text of the three phone cards of this block keeps 24 px from the card edges.
const TERMINAL_PHONE_SIDE: f64 = 24.0;
/// Whether `text` at `size` px, `spacing` px between letters, fits `room` px
/// with the conservative advance of the fit checks and 8 % to spare.
fn terminal_phone_fits(text: &str, size: f64, spacing: f64, room: f64) -> bool {
    (TERMINAL_WIDE * size + spacing) * text.chars().count() as f64 * TERMINAL_SLACK <= room
}
/// One line of phone text, refused unless it fits `room` px.
fn terminal_phone_line(
    (x, y): Point,
    size: f64,
    room: f64,
    attrs: &str,
    text: &str,
) -> Result<String> {
    if !terminal_phone_fits(text, size, 0.0, room) {
        return Err(format!("does not fit {room:.0} px at {size} px: {text}").into());
    }
    Ok(format!(
        r#"<text x="{x:.0}" y="{y:.0}" font-size="{size:.0}"{attrs}>{}</text>"#,
        xml(text)
    ))
}
/// A kicker line (20 px, 2 px between letters), refused unless it fits `room` px.
fn terminal_phone_kick((x, y): Point, room: f64, attrs: &str, text: &str) -> Result<String> {
    if !terminal_phone_fits(text, PHONE_KICK, 2.0, room) {
        return Err(format!("the kicker does not fit: {text}").into());
    }
    Ok(format!(
        r#"<text class="kick" x="{x:.0}" y="{y:.0}"{attrs}>{}</text>"#,
        xml(text)
    ))
}
/// The size the shared phone frame gives its kicker.
const PHONE_KICK: f64 = 20.0;
/// The words of `text` in lines that fit `room` px at `size` px; a word that
/// does not fit alone is refused, never shrunk.
fn terminal_phone_wrap(text: &str, size: f64, room: f64) -> Result<Vec<String>> {
    let max = (room / (TERMINAL_WIDE * size * TERMINAL_SLACK)).floor() as usize;
    let lines = wrap(text, max.max(1));
    if lines.is_empty() {
        return Err("nothing to draw".into());
    }
    for line in &lines {
        if !terminal_phone_fits(line, size, 0.0, room) {
            return Err(format!("a word does not fit {room:.0} px at {size} px: {line}").into());
        }
    }
    Ok(lines)
}
/// The entrance of a phone card: every moving element gets its own class that
/// plays once after its delay and holds its end, so the card never goes blank
/// again; the reduced-motion rule stops every class it hands out.
struct TerminalPhoneMotion {
    prefix: &'static str,
    css: String,
    names: Vec<String>,
}
impl TerminalPhoneMotion {
    fn new(prefix: &'static str) -> TerminalPhoneMotion {
        TerminalPhoneMotion {
            prefix,
            css: String::new(),
            names: Vec::new(),
        }
    }
    /// A class playing `animations` (comma-separated `name duration timing
    /// delay`) once each, held at their end, unless one says otherwise.
    fn add(&mut self, animations: &str) -> String {
        let name = format!("{}{}", self.prefix, self.names.len());
        // Split at the commas between animations, not those inside a timing function.
        let (mut parts, mut depth, mut from) = (Vec::new(), 0, 0);
        for (i, c) in animations.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(&animations[from..i]);
                    from = i + 1;
                }
                _ => {}
            }
        }
        parts.push(&animations[from..]);
        let list: Vec<String> = parts
            .into_iter()
            .map(|a| {
                if a.contains("infinite") {
                    a.trim().to_string()
                } else {
                    format!("{} both", a.trim())
                }
            })
            .collect();
        self.css
            .push_str(&format!(".{name}{{animation:{}}}", list.join(",")));
        self.names.push(name.clone());
        name
    }
    fn css(&self) -> String {
        let all: Vec<String> = self.names.iter().map(|n| format!(".{n}")).collect();
        format!(
            "{}@media (prefers-reduced-motion:reduce){{{}{{animation:none}}}}",
            self.css,
            all.join(",")
        )
    }
}
/// Phone version of the terminal card: one tile per command, stacked; the short
/// name of the command (its table row) is typed, then a check mark and the
/// lines to look for as green badges, a long line broken at its spaces into
/// short mono lines. A drawing of the README's own lines, not a recording.
fn terminal_phone_draw(dark: bool, checks: &[TerminalCheck]) -> Result<String> {
    if checks.is_empty() {
        return Err("no checks to draw".into());
    }
    let t = theme(dark);
    let tile = if dark { "#0a1322" } else { "#0f2240" };
    let (code, soft, blue, green) = ("#e6edf3", DARK.muted, DARK.accent, DARK.good);
    let side = TERMINAL_PHONE_SIDE;
    let (x, w) = (side, f64::from(PHONE_WIDTH) - 2.0 * side);
    let room = w - 16.0;
    let (name_px, line_px) = (26.0, 20.0);
    let n = checks.len();
    let mut m = TerminalPhoneMotion::new("tq");
    let mut b = terminal_phone_kick((x + 8.0, 42.0), room, "", "RUN IT YOURSELF")?;
    b += &terminal_phone_line(
        (x + 8.0, 76.0),
        28.0,
        room,
        r#" class="b""#,
        "Drawn, not a recording",
    )?;
    b += &terminal_phone_kick(
        (x + 8.0, 104.0),
        room,
        &format!(r#" style="fill:{}""#, t.muted),
        &format!("{n} COMMANDS · WHAT TO LOOK FOR"),
    )?;
    let mut y = 116.0;
    for (k, c) in checks.iter().enumerate() {
        let at = 0.3 + 0.7 * k as f64;
        let len = c.key.chars().count();
        let (xn, yn) = (x + 66.0, y + 35.0);
        // The name, then its check mark, in the first row of the tile.
        let name = terminal_phone_line(
            (xn, yn),
            name_px,
            w - 66.0 - 46.0,
            &format!(r#" class="mono" style="fill:{code}""#),
            &c.key,
        )?;
        let typed = TERMINAL_WIDE * name_px * len as f64;
        let typing = 0.05 * len as f64;
        let (shown, cover, done) = (
            m.add(&format!("tq-in .4s ease-out {at:.2}s")),
            m.add(&format!(
                "tq-type {typing:.2}s steps({len}) {:.2}s",
                at + 0.15
            )),
            m.add(&format!("tq-in .4s ease-out {:.2}s", at + 0.2 + typing)),
        );
        // The badges, each in its own row, after the joining word when it fits.
        let mut rows = String::new();
        let mut yb = y + 48.0;
        for (join, line) in &c.look {
            let mut xb = x + 16.0;
            let parts = terminal_phone_wrap(line, line_px, room - 32.0 - 28.0)?;
            let longest = parts.iter().map(|p| p.chars().count()).max().unwrap_or(0);
            let bw = TERMINAL_WIDE * line_px * longest as f64 + 28.0;
            if join.chars().any(char::is_alphanumeric) {
                let jw = TERMINAL_WIDE * line_px * join.chars().count() as f64 + 12.0;
                let inline = xb + jw + bw <= x + w - 16.0;
                let jy = if inline { yb + 24.0 } else { yb + 18.0 };
                rows += &terminal_phone_line(
                    (xb, jy),
                    line_px,
                    room - 32.0,
                    &format!(r#" style="fill:{soft}""#),
                    join,
                )?;
                if inline {
                    xb += jw;
                } else {
                    yb += 26.0;
                }
            }
            let bh = 26.0 * parts.len() as f64 + 10.0;
            rows.push_str(&format!(
                r#"<rect x="{xb:.1}" y="{yb:.0}" width="{bw:.1}" height="{bh:.0}" rx="14" fill="{green}" fill-opacity=".14" stroke="{green}"/>"#
            ));
            for (i, part) in parts.iter().enumerate() {
                rows += &terminal_phone_line(
                    (xb + 14.0, yb + 25.0 + 26.0 * i as f64),
                    line_px,
                    room - 32.0 - 28.0,
                    &format!(r#" class="mono b" style="fill:{green}""#),
                    part,
                )?;
            }
            yb += bh + 6.0;
        }
        let h = yb - y + 4.0;
        let xc = xn + typed + 12.0;
        // The name and the cover typing it out belong to the tile's group, so
        // neither shows before the tile does.
        b.push_str(&format!(
            r#"<g class="{shown}"><rect x="{x:.0}" y="{y:.0}" width="{w:.0}" height="{h:.0}" rx="16" fill="{tile}" stroke="{line}"/><circle cx="{:.0}" cy="{:.0}" r="18" fill="{blue}" fill-opacity=".16" stroke="{blue}"/><text x="{:.0}" y="{:.0}" font-size="20" class="b" text-anchor="middle" style="fill:{blue}">{}</text>{name}<rect class="tq-c {cover}" x="{:.0}" y="{:.0}" width="{:.1}" height="38" fill="{tile}"/></g><g class="{done}"><path d="M{xc:.1} {:.0}l7 7 13-15" fill="none" stroke="{green}" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>{}</g>"#,
            x + 36.0,
            y + 26.0,
            x + 36.0,
            y + 33.0,
            k + 1,
            xn - 2.0,
            yn - 30.0,
            typed + 10.0,
            yn - 10.0,
            rows,
            line = t.line
        ));
        y += h + 6.0;
    }
    // The cover's base rule holds the final look; only the keyframes hold the
    // start, so a renderer without animation shows every name and line.
    let note = [
        "Every name and line is read from",
        "this README's code block and table",
    ];
    for (i, line) in note.iter().enumerate() {
        b += &terminal_phone_line(
            (x + 8.0, y + 18.0 + 26.0 * i as f64),
            20.0,
            room,
            r#" class="mut""#,
            line,
        )?;
    }
    let h = (y + 18.0 + 26.0 + 18.0).ceil() as u32;
    let style = format!(
        "{}.tq-c{{transform-box:fill-box;transform-origin:right;transform:scaleX(0)}}{}@keyframes tq-in{{from{{opacity:0;transform:translateX(-12px)}}to{{opacity:1;transform:none}}}}@keyframes tq-type{{from{{transform:scaleX(1)}}to{{transform:scaleX(0)}}}}",
        RESULT_STYLE.replace("MUTED", t.muted),
        m.css()
    );
    Ok(phone_card(
        dark,
        h,
        (
            "Run it yourself — drawn, not a recording",
            &xml(&terminal_text(checks)),
        ),
        &style,
        &b,
    ))
}
fn terminal_phone(dark: bool) -> Result<String> {
    terminal_phone_draw(dark, &terminal_checks(NATIVE)?)
}
/// Phone version of the record card. On a phone it sits right under the Q8
/// card, which already shows the record, its sizes and what four views are
/// not; so it shows only its own subject: the verdict of every recorded run,
/// what each view's score was compared with, the reference layout used for
/// checking, and the limits. The entrance plays once and holds.
fn record_phone_draw(dark: bool, r: &Q8Runs) -> Result<String> {
    let t = theme(dark);
    let side = TERMINAL_PHONE_SIDE + 16.0;
    let (x0, w) = (side, f64::from(PHONE_WIDTH) - 2.0 * side);
    let right = x0 + w;
    let mut m = TerminalPhoneMotion::new("rq");
    let line = |at: Point, size: f64, attrs: &str, text: &str| {
        terminal_phone_line(at, size, w, attrs, text)
    };
    // 1. The verdict of every recorded run, in the words the runs print.
    let mut b = terminal_phone_kick((x0, 48.0), w, "", "THE RECORDED RUNS")?;
    let a = m.add("rq-pop .6s cubic-bezier(.2,.8,.2,1) .1s");
    b += &format!(r#"<g class="rq-c {a}">"#);
    let big = format!("{} of {}", r.ok, r.n);
    b += &line(
        (x0, 112.0),
        60.0,
        &format!(r#" class="b" style="fill:{}""#, t.good),
        &big,
    )?;
    b += "</g>";
    let a = m.add("rq-in .5s ease-out .5s");
    b += &format!(r#"<g class="{a}">"#);
    // The words stand beside the count, or under it when it is wide.
    let wide = TERMINAL_WIDE * 60.0 * big.chars().count() as f64 * TERMINAL_SLACK + 16.0;
    let words = ["recorded runs", "end with"];
    let (wx, wy, top) = if terminal_phone_fits(words[0], 20.0, 0.0, w - wide) {
        (x0 + wide, 88.0, 130.0)
    } else {
        (x0, 138.0, 188.0)
    };
    for (k, word) in words.iter().enumerate() {
        b += &terminal_phone_line((wx, wy + 26.0 * k as f64), 20.0, right - wx, "", word)?;
    }
    b.push_str(&format!(
        r#"<rect x="{x0}" y="{top}" width="{w}" height="84" rx="14" fill="{}" stroke="{}"/>"#,
        t.bg[0], t.line
    ));
    for (k, verdict) in Q8_PASS.split(' ').enumerate() {
        b += &terminal_phone_line(
            (x0 + 18.0, top + 34.0 + 32.0 * k as f64),
            20.0,
            w - 36.0,
            r#" class="mono b""#,
            verdict,
        )?;
    }
    b += "</g>";
    // 2. What every run compared: each view's score with a separate copy's.
    let view = top + 126.0;
    b += &terminal_phone_kick((x0, view), w, "", "EACH VIEW, BIT FOR BIT")?;
    b += &line(
        (x0, view + 30.0),
        20.0,
        r#" class="mono""#,
        "score vs a separate written-out copy",
    )?;
    // One chip per view; the words of the comparison once, under them.
    let (chip, gap) = (122.0, 10.0);
    for i in 0..4u32 {
        let (x, y) = (x0 + (chip + gap) * f64::from(i), view + 46.0);
        let row = m.add(&format!(
            "rq-in .4s ease-out {:.2}s",
            0.9 + 0.1 * f64::from(i)
        ));
        let pop = m.add(&format!(
            "rq-pop .5s cubic-bezier(.2,.8,.2,1) {:.2}s",
            1.4 + 0.3 * f64::from(i)
        ));
        b.push_str(&format!(
            r#"<g class="{row}"><rect x="{x}" y="{y}" width="{chip}" height="48" rx="12" fill="{bg}" stroke="{a}" stroke-width="2"/>{}</g><g class="rq-c {pop}"><circle cx="{:.0}" cy="{:.0}" r="12" fill="{}"/><path d="M{:.0} {:.0}l4 4 8-8" stroke="{bg}" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round"/></g>"#,
            terminal_phone_line(
                (x + 8.0, y + 32.0),
                20.0,
                chip - 34.0,
                r#" class="b""#,
                &format!("view {}", i + 1)
            )?,
            x + chip - 17.0,
            y + 24.0,
            t.good,
            x + chip - 23.0,
            y + 24.0,
            bg = t.bg[0],
            a = t.accent
        ));
    }
    let a = m.add("rq-in .5s ease-out 2.6s");
    b += &format!(r#"<g class="{a}">"#);
    b += &line(
        (x0, view + 122.0),
        20.0,
        r#" class="b""#,
        "score = copy's score",
    )?;
    // 3. The layout the separate copies were written out in, for checking.
    let (label, size) = ("reference layout", grouped(r.frames) + " B");
    if !terminal_phone_fits(&format!("{label}  {size}"), 22.0, 0.0, w) {
        return Err("the reference layout row does not fit".into());
    }
    let (rule, end) = (view + 164.0, view + 192.0);
    b += &line((x0, rule), 22.0, r#" class="b""#, label)?;
    b += &line(
        (right, rule),
        22.0,
        r#" class="b" text-anchor="end""#,
        &size,
    )?;
    b += &line((x0, end), 20.0, r#" class="mut""#, "used for checking")?;
    b += "</g>";
    // 4. The limits stay in view throughout.
    let limits = terminal_phone_wrap(
        "Record payload only; process memory is additional.",
        20.0,
        w,
    )?;
    let first = end + 40.0;
    for (k, text) in limits.iter().enumerate() {
        b += &line((x0, first + 26.0 * k as f64), 20.0, r#" class="mut""#, text)?;
    }
    let h = (first + 26.0 * (limits.len() - 1) as f64 + 24.0).ceil() as u32;
    let style = format!(
        "{}.rq-c{{transform-box:fill-box;transform-origin:center}}{}@keyframes rq-in{{from{{opacity:0;transform:translateY(12px)}}to{{opacity:1;transform:none}}}}@keyframes rq-pop{{0%{{opacity:0;transform:scale(.6)}}70%{{opacity:1;transform:scale(1.08)}}100%{{opacity:1;transform:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted),
        m.css()
    );
    Ok(phone_card(
        dark,
        h,
        ("One stored Q8 record, four exact views", &record_text(r)),
        &style,
        &b,
    ))
}
fn record_phone(dark: bool) -> Result<String> {
    record_phone_draw(dark, &Q8Runs::read()?)
}
/// What the flow card says, in its words: the four steps (head and the line
/// under it) and the note under the picture.
#[derive(Clone, Debug, PartialEq)]
struct FlowPhone {
    steps: Vec<(String, String)>,
    note: String,
}
/// Reads the words of the wide flow card, so the phone card says what it says:
/// each step is a bold 18-px head with the 13-px mono line after it; the note
/// is the line that starts "Drawn, not recorded".
fn flow_phone_read(scene: &str) -> Result<FlowPhone> {
    let head = r#"font-size="18" font-weight="700">"#;
    let mut steps = Vec::new();
    for part in scene.split(head).skip(1) {
        let (h, rest) = part
            .split_once("</text>")
            .ok_or("a step head is not closed")?;
        let sub = rest
            .split_once(r#"font-size="13">"#)
            .and_then(|(_, s)| s.split_once("</text>"))
            .ok_or("a step has no line under it")?
            .0;
        steps.push((h.to_string(), sub.to_string()));
    }
    let note = scene
        .split_once(">Drawn, not recorded")
        .and_then(|(_, s)| s.split_once("</text>"))
        .map(|(s, _)| format!("Drawn, not recorded{s}"))
        .ok_or("the flow card has no note")?;
    if steps.len() != 4 {
        return Err(format!("{} steps in the flow card, not 4", steps.len()).into());
    }
    let plain = |w: &String| !w.contains(['<', '&', '"']);
    if !(steps.iter().all(|(h, s)| plain(h) && plain(s)) && plain(&note)) {
        return Err("a step of the flow card is not plain text".into());
    }
    Ok(FlowPhone { steps, note })
}
/// Phone version of the flow card: the document with its exact quote, the
/// snapshot pinned by the SHA-256 you keep, the reopened copy that passes and
/// the copy with one changed byte that is refused, one under the other. Drawn,
/// not recorded; the entrance plays once and holds, only the dashes keep running.
fn flow_phone_draw(dark: bool, f: &FlowPhone) -> Result<String> {
    let t = theme(dark);
    let side = TERMINAL_PHONE_SIDE;
    let (tx, room) = (164.0, f64::from(PHONE_WIDTH) - 164.0 - side);
    let mut m = TerminalPhoneMotion::new("fq");
    let mut b = terminal_phone_kick(
        (side + 16.0, 48.0),
        f64::from(PHONE_WIDTH) - 2.0 * side - 16.0,
        "",
        "WHAT HAPPENS TO A CITATION",
    )?;
    let (cx, step, top0) = (100.0, 106.0, 62.0);
    let top = |k: usize| top0 + step * k as f64;
    // The pictures of the four steps, each in a box about 76 px high.
    let mut sheets = String::new();
    for (i, d) in [12.0, 6.0, 0.0].into_iter().enumerate() {
        sheets += &oblique(cx - 40.0 + d, top(0) + 12.0 + d, (60.0, 62.0, 5.0), t.paper);
        if i == 2 {
            for (k, row) in [12.0, 23.0, 33.0, 43.0, 53.0].into_iter().enumerate() {
                let (fill, hh, len) = if k == 1 {
                    (t.accent, 7.0, 48.0)
                } else {
                    (t.floor, 3.0, 40.0 - 3.0 * k as f64)
                };
                sheets.push_str(&format!(
                    r#"<rect x="{:.0}" y="{:.0}" width="{len}" height="{hh}" rx="2" fill="{fill}"/>"#,
                    cx - 34.0 + d,
                    top(0) + 12.0 + d + row - hh / 2.0
                ));
            }
        }
    }
    let (bx, by) = (cx - 38.0, top(1) + 10.0);
    let snap = format!(
        r##"{}<rect x="{:.0}" y="{:.0}" width="28" height="23" rx="4" fill="#ffffff" opacity=".92"/><path d="M{:.0} {:.0}v-7a7 7 0 0 1 14 0v7" fill="none" stroke="#ffffff" stroke-width="4" opacity=".92"/><circle cx="{:.0}" cy="{:.0}" r="3.5" fill="{}"/>"##,
        oblique(bx, by, (66.0, 66.0, 10.0), t.blue),
        bx + 19.0,
        by + 29.0,
        bx + 26.0,
        by + 29.0,
        bx + 33.0,
        by + 41.0,
        t.blue[2]
    );
    let (gx, gy) = (cx - 36.0, top(2) + 12.0);
    let ok = format!(
        r##"{}<path d="M{:.0} {:.0}l10 10 19-22" fill="none" stroke="#ffffff" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/>"##,
        oblique(gx, gy, (62.0, 62.0, 10.0), t.green),
        gx + 15.0,
        gy + 31.0
    );
    let (rx, ry) = (cx - 36.0, top(3) + 12.0);
    // The changed cell pulses six times once its copy has landed, then holds.
    let hot = m.add("fq-hot 1s ease-in-out 4.6s 6 alternate");
    let mut cells = String::new();
    for i in 0..4 {
        for j in 0..4 {
            let is_hot = (i, j) == (2, 1);
            cells.push_str(&format!(
                r##"<rect{} x="{:.0}" y="{:.0}" width="12" height="12" rx="2" fill="#ffffff" opacity="{}"/>"##,
                if is_hot {
                    format!(r#" class="{hot}""#)
                } else {
                    String::new()
                },
                rx + 5.0 + 13.5 * f64::from(i),
                ry + 5.0 + 13.5 * f64::from(j),
                if is_hot { "1" } else { ".22" }
            ));
        }
    }
    let bad = oblique(rx, ry, (62.0, 62.0, 10.0), t.red) + &cells;
    // The dashed beams: down from the document to the snapshot, down to the
    // reopened copy, and around it on the left to the changed copy.
    let flow = m.add("fq-flow 1s linear infinite");
    let beam = |d: &str, tip: &str| {
        format!(
            r#"<path class="{flow}" d="{d}" stroke="{c}" stroke-width="4" stroke-dasharray="10 8" stroke-linecap="round" fill="none"/><path d="{tip}" fill="{c}"/>"#,
            c = t.accent
        )
    };
    let down = |k: usize| {
        let (y1, y2) = (top(k) + 82.0, top(k + 1) + 1.0);
        beam(
            &format!("M{cx} {y1:.0}V{:.0}", y2 - 8.0),
            &format!("M{cx} {y2:.0}l-7 -11h14z"),
        )
    };
    let (ya, yz) = (top(1) + 44.0, top(3) + 44.0);
    let around = beam(
        &format!(
            "M{} {ya:.0}C14 {ya:.0} 14 {yz:.0} {} {yz:.0}",
            bx - 4.0,
            rx - 18.0
        ),
        &format!("M{} {yz:.0}l-13 -8v16z", rx - 4.0),
    );
    let pictures = [sheets, snap, ok, bad];
    let enter = [
        "fq-drop .6s cubic-bezier(.2,.8,.2,1) .2s".to_string(),
        "fq-pop .6s cubic-bezier(.2,.8,.2,1) 1.3s".to_string(),
        "fq-drop .6s cubic-bezier(.2,.8,.2,1) 2.4s".to_string(),
        "fq-drop .6s cubic-bezier(.2,.8,.2,1) 3.4s,fq-shake .5s ease-in-out 4.1s".to_string(),
    ];
    let beams = [(down(0), ".9s"), (down(1), "2s"), (around, "3s")];
    for (beam, at) in beams {
        let a = m.add(&format!("fq-in .5s ease-out {at}"));
        b.push_str(&format!(r#"<g class="{a}">{beam}</g>"#));
    }
    // The verdicts stand in one column, after the longest name that has one.
    let wide_of = |lines: &[String]| {
        let used = lines.last().map_or(0, |l| l.chars().count()) as f64;
        TERMINAL_WIDE * 24.0 * used * TERMINAL_SLACK + 10.0
    };
    let mut verdict_at: f64 = 0.0;
    for (head, _) in &f.steps {
        if let Some((name, _)) = head.split_once(" · ") {
            verdict_at = verdict_at.max(wide_of(&terminal_phone_wrap(name, 24.0, room)?));
        }
    }
    for (k, ((head, sub), picture)) in f.steps.iter().zip(pictures).enumerate() {
        let a = m.add(&enter[k]);
        // A head with a verdict after " · " puts the verdict after the name,
        // green for the reopened copy, red for the changed one.
        let (name, verdict) = head.split_once(" · ").unwrap_or((head, ""));
        let heads = terminal_phone_wrap(name, 24.0, room)?;
        let subs = terminal_phone_wrap(sub, 20.0, room)?;
        let mut y = top(k) + 36.0;
        let mut words = String::new();
        for line in &heads {
            words += &terminal_phone_line((tx, y), 24.0, room, r#" class="b""#, line)?;
            y += 28.0;
        }
        if !verdict.is_empty() {
            let colour = match (k == 3, dark) {
                (false, _) => t.good,
                (true, true) => t.red[0],
                (true, false) => t.red[2],
            };
            let vx = tx + verdict_at.max(wide_of(&heads));
            words += &terminal_phone_line(
                (vx, y - 28.0),
                24.0,
                tx + room - vx,
                &format!(r#" class="b" style="fill:{colour}""#),
                verdict,
            )?;
        }
        for line in &subs {
            words += &terminal_phone_line((tx, y), 20.0, room, r#" class="mono""#, line)?;
            y += 26.0;
        }
        if y > top(k) + step {
            return Err(format!("the words of a step do not fit its row: {head}").into());
        }
        b.push_str(&format!(
            r#"<g class="fq-c {a}"><g>{picture}</g>{words}</g>"#
        ));
    }
    // The note: drawn, not recorded, and not a timing measurement.
    let wide = f64::from(PHONE_WIDTH) - 2.0 * side - 16.0;
    let mut y = top(4) + 8.0;
    for part in f.note.split(" · ") {
        for line in terminal_phone_wrap(part, 20.0, wide)? {
            b += &terminal_phone_line((side + 16.0, y), 20.0, wide, r#" class="mut""#, &line)?;
            y += 26.0;
        }
    }
    let h = (y + 2.0).ceil() as u32;
    let style = format!(
        "{}.fq-c{{transform-box:fill-box;transform-origin:center}}{}@keyframes fq-in{{from{{opacity:0}}to{{opacity:1}}}}@keyframes fq-drop{{from{{opacity:0;transform:translateY(-30px)}}to{{opacity:1;transform:none}}}}@keyframes fq-pop{{0%{{opacity:0;transform:scale(.6)}}70%{{opacity:1;transform:scale(1.06)}}100%{{opacity:1;transform:none}}}}@keyframes fq-shake{{0%,100%{{transform:none}}25%{{transform:translateX(-6px)}}50%{{transform:translateX(6px)}}75%{{transform:translateX(-4px)}}}}@keyframes fq-hot{{from{{opacity:1}}to{{opacity:.35}}}}@keyframes fq-flow{{to{{stroke-dashoffset:-36}}}}",
        RESULT_STYLE.replace("MUTED", t.muted),
        m.css()
    );
    Ok(phone_card(
        dark,
        h,
        (
            "What happens to a citation",
            "Animated illustration: a document with a highlighted exact quote becomes a snapshot pinned by SHA-256; a new process reopens it with the same pin and passes; a copy with one changed byte is refused. Drawn, not recorded; not a timing measurement.",
        ),
        &style,
        &b,
    ))
}
fn flow_phone(dark: bool) -> Result<String> {
    flow_phone_draw(dark, &flow_phone_read(&scene(dark))?)
}
// ===== PHONE a2 END =====

// ===== PHONE a3 BEGIN =====
/// The width a phone line may take: 0.62 × font size per character, with 8 %
/// slack.
fn speed_phone_width(text: &str, size: f64) -> f64 {
    0.62 * size * text.chars().count() as f64 * 1.08
}
/// `text` in lines that fit `width` px at `size` px.
fn speed_phone_wrap(text: &str, size: f64, width: f64) -> Vec<String> {
    let max = (width / (0.62 * size * 1.08)).floor() as usize;
    wrap(text, max.max(1))
}
/// Lines of text from `y` down, `step` apart, each through `xml`; returns the
/// drawing and the baseline of the last line.
fn speed_phone_lines(
    (x, y): Point,
    (size, step): (f64, f64),
    attrs: &str,
    lines: &[String],
) -> (String, f64) {
    let mut s = String::new();
    let mut last = y - step;
    for (i, line) in lines.iter().enumerate() {
        last = y + step * i as f64;
        s.push_str(&format!(
            r#"<text x="{x:.0}" y="{last:.0}" font-size="{size}"{attrs}>{}</text>"#,
            xml(line)
        ));
    }
    (s, last)
}
/// The motion of the four phone cards of this block, with class prefix `p`:
/// fade in (`f`), grow from the left (`g`), pop (`p`) and rise (`u`). Only
/// keyframes start from nothing; reduced motion stills every one of them.
fn speed_phone_style(p: &str, muted: &str) -> String {
    format!(
        ".mut{{fill:{muted}}}.b{{font-weight:700}}.{p}-f{{animation:{p}-fade .7s ease-out both}}.{p}-g{{animation:{p}-grow 1s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:left}}.{p}-p{{animation:{p}-pop .5s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:center}}.{p}-u{{animation:{p}-rise .8s cubic-bezier(.2,.8,.2,1) both}}@keyframes {p}-fade{{from{{opacity:0;transform:translateY(14px)}}to{{opacity:1;transform:none}}}}@keyframes {p}-grow{{from{{transform:scaleX(0)}}to{{transform:none}}}}@keyframes {p}-pop{{from{{opacity:0;transform:scale(.6)}}to{{opacity:1;transform:none}}}}@keyframes {p}-rise{{from{{opacity:0;transform:translateY(22px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{.{p}-f,.{p}-g,.{p}-p,.{p}-u{{animation:none}}}}"
    )
}
/// The neutral grey of cases inside their A/A band, as the re-run card has it.
fn speed_phone_grey(dark: bool) -> &'static str {
    if dark {
        "#8b949e"
    } else {
        "#6e7781"
    }
}
/// The gains of the SHA-256 program as a phone sentence: the phone card draws
/// no axis, so the sentence says what the numbers divide.
fn speed_phone_sha(s: &SpeedSet) -> Option<String> {
    speed_sha_gains(s).map(|g| format!("{g}: v0.6.1 time divided by the new time."))
}
/// What the speed phone card says, as its description: the wide card's words
/// without its rows, which only the wide card and the folded text carry.
fn speed_phone_text(s: &SpeedSet) -> String {
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let notes: Vec<&str> = s
        .examples
        .iter()
        .filter_map(|e| e.note.as_deref())
        .collect();
    let sha = speed_phone_sha(s).map_or(String::new(), |g| format!(" {g}"));
    format!(
        "Speed against v0.6.1, timing set {}: main at {SPEED_B}. {} timed cases in {timed} of {} timing programs: {faster} faster beyond their own A/A band, {slower} slower beyond it and {within} inside it.{sha} {SPEED_GROUPED}; per-case results are in {SPEED_DIR}. {same} of {timed} timed programs printed the same output digest in all three trees. {} Machine: {}; {}; not pinned to cores. Ratios of different programs are not comparable with each other; the output digests, not the timings, are the claim.",
        speed_set_name(),
        s.cases().count(),
        s.examples.len(),
        notes.join(" "),
        s.machine,
        s.load
    )
}
/// The speed card on a phone: the totals of the set as one bar of its cases,
/// faster / inside / slower each against its own A/A band, the output digests,
/// and the limits the wide card states. The rows per program are the wide
/// card's.
fn speed_phone_draw(dark: bool, s: &SpeedSet) -> String {
    let t = theme(dark);
    let grey = speed_phone_grey(dark);
    let (x0, w) = (30.0, 540.0);
    let [faster, slower, within] = s.sides();
    let (timed, same) = s.verdicts();
    let cases = s.cases().count();
    let mut b = format!(
        r#"<text class="kick" x="{x0}" y="44">SPEED AGAINST v0.6.1</text><g class="sph-f"><text x="{x0}" y="96" font-size="44" class="b">{cases} timed cases</text><text x="{x0}" y="130" font-size="24">in {timed} of {} timing programs</text><text class="mono" x="{x0}" y="160" font-size="20">B: main at {}</text><text class="mono" x="{x0}" y="186" font-size="20">set {}</text></g>"#,
        s.examples.len(),
        xml(SPEED_B),
        xml(speed_set_name())
    );
    // One bar of every case: faster, inside, slower, each against its own band.
    let (top, bar) = (202.0, 36.0);
    let u = w / cases.max(1) as f64;
    let parts = [
        (faster, t.blue[1], "faster"),
        (within, grey, "inside"),
        (slower, t.red[1], "slower"),
    ];
    b.push_str(r#"<g class="sph-g" style="animation-delay:.3s">"#);
    let mut x = x0;
    for (n, fill, _) in parts {
        if n > 0 {
            b.push_str(&format!(
                r#"<rect x="{x:.1}" y="{top:.0}" width="{:.1}" height="{bar:.0}" fill="{fill}"/>"#,
                n as f64 * u
            ));
        }
        x += n as f64 * u;
    }
    b.push_str(&format!(
        r#"<rect x="{x0:.0}" y="{top:.0}" width="{w:.0}" height="{bar:.0}" rx="3" fill="none" stroke="{}"/></g>"#,
        t.line
    ));
    for (k, (n, fill, word)) in parts.iter().enumerate() {
        let cx = x0 + 180.0 * k as f64;
        b.push_str(&format!(
            r#"<g class="sph-p" style="animation-delay:{:.1}s"><text x="{cx:.0}" y="280" font-size="40" class="b" style="fill:{fill}">{n}</text><text x="{cx:.0}" y="308" font-size="22">{word}</text></g>"#,
            1.0 + 0.15 * k as f64
        ));
    }
    let (rule, y) = speed_phone_lines(
        (x0, 338.0),
        (20.0, 26.0),
        r#" class="mut""#,
        &speed_phone_wrap(
            "A case counts as faster or slower only beyond its own A/A band.",
            20.0,
            w,
        ),
    );
    b.push_str(&format!(
        r#"<g class="sph-f" style="animation-delay:1.3s">{rule}</g>"#
    ));
    // The output digests.
    let y = y + 48.0;
    b.push_str(&format!(
        r#"<g class="sph-p" style="animation-delay:1.5s"><path d="M{:.0} {:.0}l9 9 17-20" fill="none" stroke="{g}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/><text x="{:.0}" y="{y:.0}" font-size="34" class="b" style="fill:{g}">{same} of {timed}</text></g><text class="sph-f" style="animation-delay:1.6s" x="{x0}" y="{:.0}" font-size="20">same output digest in A, A2 and B</text>"#,
        x0 + 2.0,
        y - 14.0,
        x0 + 44.0,
        y + 30.0,
        g = t.good
    ));
    // Why a digest differs, in the words of the set's README, as the wide
    // card's note says it; the phone names no program, so the reason starts
    // after "because".
    let (mut why, mut last) = (String::new(), None);
    let mut at = y + 56.0;
    for e in &s.examples {
        if !matches!(e.timed, Some((_, false))) {
            continue;
        }
        let text = match e.note.as_deref() {
            Some(n) => format!(
                "1 differs: {}",
                n.split_once(" because ").map_or(n, |(_, r)| r)
            ),
            None => "1 differs".to_string(),
        };
        let (lines, end) = speed_phone_lines(
            (x0, at),
            (20.0, 26.0),
            r#" class="mut""#,
            &speed_phone_wrap(&text, 20.0, w),
        );
        why.push_str(&lines);
        last = Some(end);
        at = end + 32.0;
    }
    if !why.is_empty() {
        b.push_str(&format!(
            r#"<g class="sph-f" style="animation-delay:1.7s">{why}</g>"#
        ));
    }
    let y = last.map_or(y + 30.0, |end| end);
    // What the totals leave out, then the limits the wide card states.
    let mut tail = Vec::new();
    let untimed = s.examples.len() - timed;
    if untimed > 0 {
        tail.push(format!("{untimed} not built in v0.6.1: no timing."));
    }
    tail.extend(speed_phone_sha(s));
    tail.push("Ratios of different programs are not comparable with each other.".to_string());
    tail.push("The output digests, not the timings, are the claim.".to_string());
    let (limits, y) = speed_phone_lines(
        (x0, y + 36.0),
        (20.0, 26.0),
        "",
        &speed_phone_wrap(&tail.join(" "), 20.0, w),
    );
    let (run, y) = speed_phone_lines(
        (x0, y + 36.0),
        (20.0, 26.0),
        r#" class="mut""#,
        &speed_phone_wrap(
            &format!(
                "Machine: {}. Load: {}. Not pinned to cores.",
                s.machine, s.load
            ),
            20.0,
            w,
        ),
    );
    let (path, y) = speed_phone_lines(
        (x0, y + 26.0),
        (20.0, 26.0),
        r#" class="mono""#,
        &[
            format!("{SPEED_GROUPED}."),
            "Per-case results: docs/speed-results".to_string(),
        ],
    );
    b.push_str(&format!(
        r#"<g class="sph-f" style="animation-delay:1.9s">{limits}</g><g class="sph-f" style="animation-delay:2.2s">{run}{path}</g>"#
    ));
    phone_card(
        dark,
        (y + 24.0) as u32,
        ("Speed against v0.6.1", &xml(&speed_phone_text(s))),
        &speed_phone_style("sph", t.muted),
        &b,
    )
}
fn speed_phone(dark: bool) -> Result<String> {
    Ok(speed_phone_draw(dark, &SpeedSet::read()?))
}
/// The re-run card on a phone: the two runs as two bars of the same cases, the
/// cases faster beyond their band in both between them, the counts as a
/// table, then the A/A noise of each run and the wide card's limits.
fn rerun_phone_draw(dark: bool, d: &RerunData) -> Result<String> {
    rerun_check(d)?;
    let t = theme(dark);
    let grey = speed_phone_grey(dark);
    let (x0, w, bar) = (40.0, 520.0, 36.0);
    let u = w / d.cases as f64;
    let fills = [t.blue[1], grey, t.red[1]];
    let (d0, d1) = (&d.runs[0].date, &d.runs[1].date);
    let mut b = format!(
        r#"<text class="kick" x="40" y="48">{}</text><g class="rph-f"><text x="40" y="96" font-size="36" class="b">{}</text><text x="40" y="128" font-size="22">timed twice against {}</text><text x="40" y="156" font-size="20" class="mut">each case against its own A/A band</text></g>"#,
        xml(&format!("RE-RUN · {d0} → {d1}")),
        xml(&format!("The same {} cases", d.cases)),
        xml(&d.base)
    );
    let tops = [200.0, 288.0];
    for (r, run) in d.runs.iter().enumerate() {
        let (top, rf) = (tops[r], r as f64);
        let role = if r == 0 { "recorded" } else { "re-run" };
        let label_y = if r == 0 { top - 12.0 } else { top + bar + 28.0 };
        b.push_str(&format!(
            r#"<text class="rph-f" style="animation-delay:{:.1}s" x="40" y="{label_y:.0}" font-size="22"><tspan class="b">{}</tspan><tspan class="mut"> · {role}</tspan></text><g class="rph-g" style="animation-delay:{:.1}s">"#,
            0.3 + 0.4 * rf,
            xml(&run.date),
            0.3 + 0.4 * rf
        ));
        let mut x = x0;
        for (k, fill) in fills.iter().enumerate() {
            let n = run.sides[k] as f64 * u;
            if n > 0.0 {
                b.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{top:.0}" width="{n:.1}" height="{bar:.0}" fill="{fill}"/>"#
                ));
            }
            x += n;
        }
        b.push_str(&format!(
            r#"<rect x="{x0:.0}" y="{top:.0}" width="{w:.0}" height="{bar:.0}" rx="3" fill="none" stroke="{}"/></g>"#,
            t.line
        ));
    }
    // The cases faster beyond their band in both runs: a band from bar to bar.
    let (gap0, gap1) = (tops[0] + bar, tops[1]);
    let span = d.again as f64 * u;
    let pill_text = format!("{} of {} faster again", d.again, d.runs[0].sides[0]);
    let pill = speed_phone_width(&pill_text, 22.0) + 28.0;
    let px = (x0 + span / 2.0)
        .max(x0 + pill / 2.0)
        .min(560.0 - pill / 2.0);
    let mid = (gap0 + gap1) / 2.0;
    b.push_str(&format!(
        r#"<rect class="rph-f" style="animation-delay:1.3s" x="{x0:.0}" y="{gap0:.0}" width="{span:.1}" height="{:.0}" fill="{a}" fill-opacity=".18"/><path class="rph-f" style="animation-delay:1.3s" d="M{x0:.0} {gap0:.0}V{gap1:.0}M{:.1} {gap0:.0}V{gap1:.0}" stroke="{a}" stroke-width="2.5" stroke-dasharray="6 5"/><g class="rph-p" style="animation-delay:1.6s"><rect x="{:.1}" y="{:.0}" width="{pill:.0}" height="38" rx="19" fill="{pf}"/><text x="{px:.1}" y="{:.0}" font-size="22" class="b" text-anchor="middle" style="fill:#ffffff">{}</text></g>"#,
        gap1 - gap0,
        x0 + span,
        px - pill / 2.0,
        mid - 19.0,
        mid + 7.5,
        xml(&pill_text),
        a = t.accent,
        pf = if dark { t.blue[2] } else { t.accent }
    ));
    // The counts as a table: one row per run, one column per side.
    let cols = [330.0, 445.0, 560.0];
    let mut table = String::new();
    for (k, word) in ["faster", "inside", "slower"].iter().enumerate() {
        table.push_str(&format!(
            r#"<text x="{:.0}" y="392" font-size="22" class="b" text-anchor="end" style="fill:{}">{word}</text>"#,
            cols[k],
            if k == 1 { t.muted } else { fills[k] }
        ));
    }
    for (r, run) in d.runs.iter().enumerate() {
        let y = 438.0 + 38.0 * r as f64;
        table.push_str(&format!(
            r#"<text class="mono" x="40" y="{y:.0}" font-size="22">{}</text>"#,
            xml(&run.date)
        ));
        for (k, n) in run.sides.iter().enumerate() {
            table.push_str(&format!(
                r#"<text x="{:.0}" y="{y:.0}" font-size="30" class="b" text-anchor="end">{n}</text>"#,
                cols[k]
            ));
        }
    }
    b.push_str(&format!(
        r#"<path class="rph-f" style="animation-delay:1.8s" d="M40 404H560" stroke="{}" stroke-width="1.5"/><g class="rph-f" style="animation-delay:1.8s">{table}</g><text class="rph-f mut" style="animation-delay:2.0s" x="40" y="510" font-size="20">{}</text>"#,
        t.line,
        xml(&format!(
            "{} of the {} with a ratio above 1",
            d.above_one, d.runs[0].sides[0]
        ))
    ));
    // The A/A noise of the two runs, the older first, then the limits.
    let noise = format!(
        "{} → {}",
        rerun_number(d.runs[0].noise),
        rerun_number(d.runs[1].noise)
    );
    b.push_str(&format!(
        r#"<g class="rph-f" style="animation-delay:2.2s"><text x="40" y="556" font-size="26" class="b">A/A noise</text><text x="40" y="584" font-size="20" class="mut">median |aa - 1| of each run</text></g><text class="rph-p b" style="animation-delay:2.4s" x="560" y="562" font-size="36" text-anchor="end">{}</text><g class="rph-f" style="animation-delay:2.7s"><text x="40" y="622" font-size="20" class="mut">One machine, not pinned to cores</text><text x="40" y="648" font-size="20" class="mut">{}</text><text x="40" y="674" font-size="20" class="mut">{}</text></g>"#,
        xml(&noise),
        xml(&format!("A, A2: two checkouts of {}", d.base)),
        xml(&format!("B: main at {SPEED_B}"))
    ));
    Ok(phone_card(
        dark,
        696,
        (&xml(&rerun_title(d)), &xml(&rerun_text(d)?)),
        &speed_phone_style("rph", t.muted),
        &b,
    ))
}
fn rerun_phone(dark: bool) -> Result<String> {
    rerun_phone_draw(dark, &rerun_read()?)
}
/// The format mutation matrix on a phone: the rows that matched their expected
/// verdict, one horizontal bar per format of its rejected and accepted rows,
/// then what the mutants are and where the report lives.
fn bars_phone_draw(dark: bool, e: &Evidence) -> String {
    let t = theme(dark);
    let w = 520.0;
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
        .unwrap_or(1)
        .max(1);
    let verdict = if matched == rows { t.good } else { t.red[1] };
    let mut b = format!(
        r#"<text class="kick" x="40" y="48">FORMAT MUTATION MATRIX</text><g class="bph-p" style="animation-delay:.2s"><text x="40" y="114" font-size="60" class="b" style="fill:{verdict}">{matched}/{rows}</text></g><g class="bph-f" style="animation-delay:.4s"><text x="40" y="148" font-size="22">rows matched their expected verdict</text></g>"#
    );
    let mut y = 194.0;
    for (i, f) in e.formats.iter().enumerate() {
        let delay = 0.6 + 0.3 * i as f64;
        let (wr, wa) = (
            f.rejected as f64 / tallest as f64 * w,
            f.accepted as f64 / tallest as f64 * w,
        );
        b.push_str(&format!(
            r#"<g class="bph-f" style="animation-delay:{delay:.1}s"><text x="40" y="{y:.0}" font-size="22" class="b">{}</text><text x="560" y="{y:.0}" font-size="22" class="b" text-anchor="end" style="fill:{}">{} mutants</text></g><g class="bph-g" style="animation-delay:{delay:.1}s">"#,
            xml(&f.name),
            t.accent,
            f.mutants
        ));
        for (x, bw, fill) in [(40.0, wr, t.red[1]), (40.0 + wr, wa, t.blue[1])] {
            if bw > 0.0 {
                b.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{:.0}" width="{bw:.1}" height="24" fill="{fill}"/>"#,
                    y + 10.0
                ));
            }
        }
        b.push_str(&format!(
            r#"<rect x="40" y="{:.0}" width="{:.1}" height="24" rx="3" fill="none" stroke="{}"/></g><text class="bph-f mono" style="animation-delay:{:.1}s" x="40" y="{:.0}" font-size="20">{} rejected · {} accepted</text>"#,
            y + 10.0,
            wr + wa,
            t.line,
            delay + 0.6,
            y + 58.0,
            f.rejected,
            f.accepted
        ));
        y += 92.0;
    }
    // The totals, with what each colour means.
    b.push_str(&format!(
        r#"<g class="bph-f" style="animation-delay:1.8s"><rect x="40" y="{:.0}" width="20" height="20" rx="4" fill="{}"/><text x="72" y="{y:.0}" font-size="20">rejected: {rejected}</text><rect x="40" y="{:.0}" width="20" height="20" rx="4" fill="{}"/><text x="72" y="{:.0}" font-size="20">accepted as documented: {accepted}</text></g>"#,
        y - 17.0,
        t.red[1],
        y + 13.0,
        t.blue[1],
        y + 30.0
    ));
    // The mutants: the number, the words of what they are beside it.
    let y = y + 76.0;
    let tx = 40.0 + speed_phone_width(&mutants.to_string(), 44.0) + 16.0;
    let (what, last) = speed_phone_lines(
        (tx, y - 14.0),
        (20.0, 26.0),
        "",
        &speed_phone_wrap(
            &format!(
                "byte-level mutants of the {} public formats, each read with the original pin and with a recomputed one",
                e.formats.len()
            ),
            20.0,
            560.0 - tx,
        ),
    );
    let (foot, last) = speed_phone_lines(
        (40.0, last + 42.0),
        (20.0, 26.0),
        r#" class="mono""#,
        &speed_phone_wrap(
            "committed report; xtask verify requires an identical rerun",
            20.0,
            w,
        ),
    );
    b.push_str(&format!(
        r#"<g class="bph-p" style="animation-delay:2.1s"><text x="40" y="{:.0}" font-size="44" class="b" style="fill:{}">{mutants}</text></g><g class="bph-f" style="animation-delay:2.3s">{what}</g><g class="bph-f" style="animation-delay:2.6s">{foot}</g>"#,
        y + 12.0,
        t.accent
    ));
    phone_card(
        dark,
        (last + 24.0) as u32,
        (
            "Format mutation matrix",
            &format!("{mutants} byte-level mutants of the {} public formats, each read with the original pin and with a recomputed one. In the committed report {matched} of {rows} rows matched their expected verdict: {rejected} rejected, {accepted} accepted as documented.", e.formats.len()),
        ),
        &speed_phone_style("bph", t.muted),
        &b,
    )
}
fn bars_phone(dark: bool) -> Result<String> {
    Ok(bars_phone_draw(dark, &Evidence::read()?))
}
/// The property map on a phone: one block of cubes per CI platform, one cube
/// per documented property in order, amber where declared Unix-only tests are
/// exempt on Windows. Like the wide card it draws the rule, not one run.
fn wall_phone_draw(dark: bool, e: &Evidence) -> String {
    let t = theme(dark);
    let (x0, w) = (30.0, 540.0);
    let n = e.rows.len();
    let unix = e.unix_rows();
    let partial = unix.iter().filter(|u| **u).count();
    let full = 3 * n - partial;
    let (rule, last) = speed_phone_lines(
        (x0, 130.0),
        (20.0, 26.0),
        "",
        &speed_phone_wrap(
            "each cell: every mapped test must run and pass exactly once on that platform, or CI evidence is not COMPLETE",
            20.0,
            w,
        ),
    );
    let title = format!("{n} properties");
    let mut b = format!(
        r#"<text class="kick" x="{x0}" y="44">PROPERTY MAP × CI PLATFORMS</text><g class="wph-f"><text x="{x0}" y="92" font-size="34" class="b">{title}</text><text x="{:.0}" y="92" font-size="24">× 3 platforms</text></g><g class="wph-f" style="animation-delay:.3s">{rule}</g>"#,
        x0 + speed_phone_width(&title, 34.0) + 12.0
    );
    // Twenty cubes to a line: a block per platform, in property order.
    let (per, step, side, depth) = (20, 27.0, 20.0, 4.0);
    let lines = n.div_ceil(per).max(1);
    let mut y = last + 42.0;
    for (r, platform) in ["Linux", "macOS", "Windows"].into_iter().enumerate() {
        b.push_str(&format!(
            r#"<text class="wph-f b" style="animation-delay:{:.2}s" x="{x0}" y="{y:.0}" font-size="22">{platform}</text>"#,
            0.4 + 0.5 * r as f64
        ));
        for (c, unix_only) in unix.iter().enumerate() {
            let faces = if r == 2 && *unix_only {
                t.amber
            } else {
                t.blue
            };
            let (cx, cy) = (
                x0 + step * (c % per) as f64,
                y + 14.0 + depth + step * (c / per) as f64,
            );
            b.push_str(&format!(
                r#"<g class="wph-u" style="animation-delay:{:.2}s">{}</g>"#,
                0.5 + 0.5 * r as f64 + 0.02 * c as f64,
                oblique(cx, cy, (side, side, depth), faces)
            ));
        }
        y += 14.0 + depth + step * lines as f64 + 34.0;
    }
    let first = e.rows.first().map_or("", |r| r.0.as_str());
    let last_id = e.rows.last().map_or("", |r| r.0.as_str());
    let (order, y) = speed_phone_lines(
        (x0, y - 10.0),
        (20.0, 26.0),
        r#" class="mut""#,
        &[format!("one cube per property, {first} to {last_id}")],
    );
    let (blue, yb) = speed_phone_lines(
        (x0 + 26.0, y + 34.0),
        (20.0, 26.0),
        "",
        &speed_phone_wrap(
            &format!("{full} cells: every mapped test required"),
            20.0,
            w - 26.0,
        ),
    );
    let (amber, ya) = speed_phone_lines(
        (x0 + 26.0, yb + 32.0),
        (20.0, 26.0),
        "",
        &speed_phone_wrap(
            &format!("{partial} Windows cells: declared Unix-only tests exempt, the rest required"),
            20.0,
            w - 26.0,
        ),
    );
    let (foot, end) = speed_phone_lines(
        (x0, ya + 36.0),
        (20.0, 26.0),
        r#" class="mono""#,
        &speed_phone_wrap("the rule, not one run: read each run's CI report", 20.0, w),
    );
    b.push_str(&format!(
        r#"<g class="wph-f" style="animation-delay:2.4s">{order}<rect x="{x0}" y="{:.0}" width="16" height="16" rx="3" fill="{}"/>{blue}<rect x="{x0}" y="{:.0}" width="16" height="16" rx="3" fill="{}"/>{amber}</g><g class="wph-f" style="animation-delay:2.7s">{foot}</g>"#,
        y + 19.0,
        t.blue[1],
        yb + 17.0,
        t.amber[1]
    ));
    phone_card(
        dark,
        (end + 24.0) as u32,
        (
            "Property map across CI platforms",
            &format!("{n} documented properties by 3 CI platforms, as the CI evidence collector checks them. In {full} cells every mapped test must run and pass exactly once; in {partial} Windows cells the declared Unix-only tests are exempt and the rest are required. This is the rule, not the result of a particular run."),
        ),
        &speed_phone_style("wph", t.muted),
        &b,
    )
}
fn wall_phone(dark: bool) -> Result<String> {
    Ok(wall_phone_draw(dark, &Evidence::read()?))
}
// ===== PHONE a3 END =====

// ===== PHONE a4 BEGIN =====
/// The side room of the commitments, crash and license phone cards, and the
/// share of its box a line may fill.
const PLEDGES_PHONE_SIDE: f64 = 30.0;
const PLEDGES_PHONE_SLACK: f64 = 0.92;
/// `text` in lines that fill at most the slack share of `room` px at `size` px.
fn pledges_phone_fit(text: &str, size: f64, room: f64) -> Vec<String> {
    pledges_fit(text, size, room * PLEDGES_PHONE_SLACK)
}
/// One `<text>` line per entry, `step` px apart from `y`; returns the next free baseline.
fn pledges_phone_lines(
    out: &mut String,
    lines: &[String],
    (x, y, step): (f64, f64, f64),
    attrs: &str,
) -> f64 {
    let mut y = y;
    for line in lines {
        out.push_str(&format!(
            r#"<text x="{x:.1}" y="{y:.1}" {attrs}>{}</text>"#,
            xml(line)
        ));
        y += step;
    }
    y
}
/// The published commitments on a phone: their number, then one compact row per
/// commitment (name and the first 8 hex of the hash, date and counts), then how
/// a commitment is checked.
fn pledges_phone_draw(dark: bool, p: &Pledges) -> Result<String> {
    let t = theme(dark);
    let n = p.rows.len();
    if n == 0 {
        return Err("no published commitment".into());
    }
    let side = PLEDGES_PHONE_SIDE;
    let right = f64::from(PHONE_WIDTH) - side;
    let mut b = String::from(r#"<text class="kick" x="30" y="44">PUBLISHED COMMITMENTS</text>"#);
    let count = n.to_string();
    let word = if n == 1 { "commitment" } else { "commitments" };
    let span = if p.rows[0].date == p.rows[n - 1].date {
        p.rows[0].date.clone()
    } else {
        format!("{} → {}", p.rows[0].date, p.rows[n - 1].date)
    };
    let nx = side + pledges_width(&count, 56.0) / PLEDGES_PHONE_SLACK + 14.0;
    b.push_str(&format!(
        r#"<g class="pp-big"><text x="{side}" y="106" font-size="56" class="b" style="fill:{acc}">{count}</text><text x="{nx:.1}" y="84" font-size="26" class="b">{word}</text></g><text x="{nx:.1}" y="110" font-size="20" class="mut">{}</text>"#,
        xml(&span),
        acc = t.accent
    ));
    let (x, w, pad) = (side, right - side, 14.0);
    let hash_w = pledges_width("00000000", 20.0) / PLEDGES_PHONE_SLACK + 12.0;
    let mut y = 124.0;
    for (i, r) in p.rows.iter().enumerate() {
        let delay = 0.3 + 0.15 * i as f64;
        let tag = pledges_tag(p, &r.date);
        // The name beside the hash, the date beside the counts.
        let name = pledges_phone_fit(pledges_name(&r.label), 20.0, w - 2.0 * pad - hash_w);
        let cx = x + pad + pledges_width(&tag, 20.0) / PLEDGES_PHONE_SLACK + 12.0;
        let room = x + w - pad - cx;
        let [c0, c1] = pledges_counts(p, r);
        let both = format!("{c0} · {c1}");
        let counts = if pledges_width(&both, 20.0) <= room * PLEDGES_PHONE_SLACK {
            vec![both]
        } else {
            [c0, c1]
                .iter()
                .flat_map(|c| pledges_phone_fit(c, 20.0, room))
                .collect()
        };
        let mut text = String::new();
        let next = pledges_phone_lines(
            &mut text,
            &name,
            (x + pad, y + 24.0, 24.0),
            r#"font-size="20" class="b""#,
        );
        text.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" font-size="20" text-anchor="end" class="mono pp-hx">{}</text><text x="{:.1}" y="{next:.1}" font-size="20" class="b" style="fill:{}">{}</text>"#,
            x + w - pad,
            y + 24.0,
            xml(&r.hash[..8]),
            x + pad,
            t.accent,
            xml(&tag)
        ));
        let end = pledges_phone_lines(&mut text, &counts, (cx, next, 24.0), r#"font-size="20""#);
        let h = end - 24.0 + 12.0 - y;
        b.push_str(&format!(
            r#"<g class="pp-cd" style="animation-delay:{delay:.2}s"><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="10" fill="{acc}" fill-opacity=".08" stroke="{}"/>{text}</g>"#,
            t.line,
            acc = t.accent
        ));
        y += h + 4.0;
    }
    // The two footer sentences of the wide card, one thought per line group,
    // after the last row has come in.
    let mut y = y + 20.0;
    let mut foot = String::new();
    for (k, line) in PLEDGES_FOOT.iter().enumerate() {
        let attrs = if k == 0 {
            r#"font-size="20" class="b""#
        } else {
            r#"font-size="20" class="mut""#
        };
        for part in line.split(" · ") {
            let lines = pledges_phone_fit(part, 20.0, right - side);
            y = pledges_phone_lines(&mut foot, &lines, (side, y, 26.0), attrs) + 2.0;
        }
    }
    b.push_str(&format!(
        r#"<g class="pp-ft" style="animation-delay:{:.2}s">{foot}</g>"#,
        0.5 + 0.15 * n as f64
    ));
    let height = y as u32;
    let style = format!(
        ".pp-hx{{fill:{acc};font-weight:700}}.pp-big{{transform-box:fill-box;transform-origin:0 100%;animation:pp-pop .6s cubic-bezier(.2,.8,.2,1) .1s both}}.pp-cd{{animation:pp-in .55s cubic-bezier(.2,.8,.2,1) both}}.pp-ft{{animation:pp-fade .8s ease-out both}}@keyframes pp-fade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes pp-pop{{from{{opacity:0;transform:scale(.2)}}to{{opacity:1;transform:none}}}}@keyframes pp-in{{from{{opacity:0;transform:translateX(36px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{.pp-big,.pp-cd,.pp-ft{{animation:none}}}}{}",
        RESULT_STYLE.replace("MUTED", t.muted),
        acc = t.accent
    );
    Ok(phone_card(
        dark,
        height,
        ("Published commitments", &xml(&pledges_text(p))),
        &style,
        &b,
    ))
}
fn pledges_phone(dark: bool) -> Result<String> {
    pledges_phone_draw(dark, &Pledges::read()?)
}
/// The crash series on a phone: the headline, every trial as a point, the
/// totals the headline and the legend do not state, and the limit of the run.
fn crash_phone_draw(dark: bool, c: &CrashSeries) -> String {
    let t = theme(dark);
    let n = c.trials.len();
    let side = PLEDGES_PHONE_SIDE;
    let right = f64::from(PHONE_WIDTH) - side;
    let mut b = String::from(
        r#"<text class="kick" x="30" y="44">CRASH SERIES · KILLED WHILE SAVING</text>"#,
    );
    let mut head = String::new();
    let mut y = 82.0;
    for sentence in [
        format!("{n} kills while saving."),
        format!("{} acknowledged snapshots lost.", c.lost),
    ] {
        let lines = pledges_phone_fit(&sentence, 26.0, right - side);
        y = pledges_phone_lines(
            &mut head,
            &lines,
            (side, y, 32.0),
            r#"font-size="26" class="b""#,
        );
    }
    b.push_str(&format!(r#"<g class="cp-in">{head}</g>"#));
    // The plot: kill time across, acknowledged snapshots up.
    let cap = y + 2.0;
    let top = cap + 44.0;
    let bottom = top + 140.0;
    let tick_w = pledges_width(&c.documents.to_string(), 20.0);
    let (x0, x1) = (side + tick_w / PLEDGES_PHONE_SLACK + 14.0, right - 6.0);
    let px = |ms: u32| x0 + (x1 - x0) * f64::from(ms) / f64::from(c.window_ms.max(1));
    let py = |k: usize| bottom - (bottom - top) * k as f64 / c.documents.max(1) as f64;
    b.push_str(&format!(
        r#"<text x="{side}" y="{cap:.1}" font-size="20" class="mut">snapshots acknowledged</text><text x="{x1:.1}" y="{:.1}" font-size="20" text-anchor="end" class="mut">all {} snapshots saved</text>"#,
        top - 12.0,
        c.documents
    ));
    for k in (0..=c.documents).step_by((c.documents / 2).max(1)) {
        let yk = py(k);
        b.push_str(&format!(
            r#"<path d="M{x0:.1} {yk:.1}H{x1:.1}" stroke="{}" stroke-width="1.5"/><text class="mono" x="{:.1}" y="{:.1}" text-anchor="end" font-size="20">{k}</text>"#,
            t.line,
            x0 - 12.0,
            yk + 7.0
        ));
    }
    // Ticks at least 90 px apart, each label inside the card.
    let step = [10, 20, 50, 100, 200, 500, 1000, 2000, 5000]
        .into_iter()
        .find(|s| (x1 - x0) * f64::from(*s) / f64::from(c.window_ms.max(1)) >= 90.0)
        .unwrap_or(c.window_ms.max(1));
    for ms in (0..=c.window_ms).step_by(step as usize) {
        let half = pledges_width(&ms.to_string(), 20.0) / 2.0 / PLEDGES_PHONE_SLACK;
        if px(ms) + half <= right + 6.0 {
            b.push_str(&format!(
                r#"<text class="mono" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="20">{ms}</text>"#,
                px(ms),
                bottom + 28.0
            ));
        }
    }
    b.push_str(&format!(
        r#"<path d="M{x0:.1} {bottom:.1}H{x1:.1}M{x0:.1} {bottom:.1}V{top:.1}" stroke="{ink}" stroke-width="2" fill="none"/><path d="M{x0:.1} {top:.1}H{x1:.1}" stroke="{ink}" stroke-width="2" stroke-dasharray="6 6" fill="none"/>"#,
        ink = t.ink
    ));
    let axis = pledges_phone_fit(
        &format!(
            "kill time, ms after the writer started (window {} ms)",
            c.window_ms
        ),
        20.0,
        right - side,
    );
    let mut y = pledges_phone_lines(
        &mut b,
        &axis,
        (side, bottom + 56.0, 26.0),
        r#"font-size="20" class="mut""#,
    );
    // Every trial survived (the reader refuses a PASS line over a lost, partial
    // or unresumed trial); filled when the kill hit a publication.
    for (i, trial) in c.trials.iter().enumerate() {
        let paint = if trial.mid_publication {
            format!(r#"r="5" fill="{}""#, t.good)
        } else {
            format!(
                r#"r="4.4" fill="none" stroke="{}" stroke-width="2""#,
                t.good
            )
        };
        b.push_str(&format!(
            r#"<circle class="cp-pt" style="animation-delay:{:.2}s" cx="{:.1}" cy="{:.1}" {paint}/>"#,
            0.4 + 2.2 * i as f64 / n.max(1) as f64,
            px(trial.kill_ms),
            py(trial.acknowledged)
        ));
    }
    y += 4.0;
    for (filled, label) in [
        (
            true,
            format!("killed mid-publication ({})", c.mid_publication),
        ),
        (
            false,
            format!("killed between saves ({})", n - c.mid_publication),
        ),
    ] {
        let mark = if filled {
            format!(r#"r="7" fill="{}""#, t.good)
        } else {
            format!(
                r#"r="6" fill="none" stroke="{}" stroke-width="2.4""#,
                t.good
            )
        };
        b.push_str(&format!(
            r#"<circle cx="{:.1}" cy="{:.1}" {mark}/><text x="{:.1}" y="{y:.1}" font-size="20">{}</text>"#,
            side + 8.0,
            y - 7.0,
            side + 26.0,
            xml(&label)
        ));
        y += 28.0;
    }
    // The totals of the wide card that the headline and the legend do not state.
    let rows = [
        (
            grouped(c.acknowledged),
            t.accent,
            "snapshots acknowledged",
            "before the kills",
        ),
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
    ];
    let num_w = rows
        .iter()
        .map(|r| pledges_width(&r.0, 32.0))
        .fold(0.0, f64::max);
    let lx = side + 14.0 + num_w / PLEDGES_PHONE_SLACK + 14.0;
    y -= 10.0;
    for (k, (num, colour, l1, l2)) in rows.iter().enumerate() {
        let label: Vec<String> = [l1, l2]
            .iter()
            .flat_map(|l| pledges_phone_fit(l, 20.0, right - 14.0 - lx))
            .collect();
        let h = 12.0 + 24.0 * label.len() as f64;
        let mut text = String::new();
        pledges_phone_lines(&mut text, &label, (lx, y + 24.0, 24.0), r#"font-size="20""#);
        b.push_str(&format!(
            r#"<g class="cp-in" style="animation-delay:{:.2}s"><rect x="{side}" y="{y:.1}" width="{:.1}" height="{h:.1}" rx="10" fill="{}" fill-opacity=".07" stroke="{}"/><text x="{:.1}" y="{:.1}" font-size="32" class="b" style="fill:{colour}">{num}</text>{text}</g>"#,
            2.6 + 0.15 * k as f64,
            right - side,
            t.accent,
            t.line,
            side + 14.0,
            y + h / 2.0 + 11.0
        ));
        y += h + 6.0;
    }
    // The limit of the run, as the wide card states it.
    y += 22.0;
    let mut foot = String::new();
    let machine = pledges_phone_fit(
        &format!(
            "One recorded run on the author's Linux machine ({}, {})",
            c.file_system, c.date
        ),
        20.0,
        right - side,
    );
    y = pledges_phone_lines(
        &mut foot,
        &machine,
        (side, y, 26.0),
        r#"font-size="20" class="mut""#,
    );
    let box_y = y - 12.0;
    let mut ly = box_y + 30.0;
    for (part, attrs) in [
        ("a process kill (SIGKILL),", r#"font-size="20" class="b""#),
        ("not a power cut:", r#"font-size="20" class="b""#),
        (
            "power-loss durability is not established",
            r#"font-size="20""#,
        ),
    ] {
        let lines = pledges_phone_fit(part, 20.0, right - side - 36.0);
        ly = pledges_phone_lines(&mut foot, &lines, (side + 18.0, ly, 26.0), attrs);
    }
    let box_h = ly - 26.0 + 14.0 - box_y;
    b.push_str(&format!(
        r#"<g class="cp-in" style="animation-delay:3.6s"><rect x="{side}" y="{box_y:.1}" width="{:.1}" height="{box_h:.1}" rx="12" fill="{a}" fill-opacity=".10" stroke="{a}" stroke-width="2"/>{foot}</g>"#,
        right - side,
        a = t.amber[1]
    ));
    let height = (box_y + box_h + 16.0) as u32;
    let style = format!(".cp-pt{{animation:cp-pop .45s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:center}}.cp-in{{animation:cp-rise .6s cubic-bezier(.2,.8,.2,1) both}}@keyframes cp-pop{{from{{opacity:0;transform:scale(.2)}}to{{opacity:1;transform:none}}}}@keyframes cp-rise{{from{{opacity:0;transform:translateY(16px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{.cp-pt,.cp-in{{animation:none}}}}{}", RESULT_STYLE.replace("MUTED", t.muted));
    phone_card(
        dark,
        height,
        ("Crash series: killed while saving", &crash_text(c)),
        &style,
        &b,
    )
}
fn crash_phone(dark: bool) -> Result<String> {
    Ok(crash_phone_draw(dark, &CrashSeries::read()?))
}
/// One quoted subject on the license phone card: its LICENSE sections, its
/// heading and the sentences quoted.
type LicensePhoneColumn<'a> = (&'a str, &'a str, Vec<&'a str>);
/// Which quotes of each column of the wide card the phone card shows: what the
/// public terms give, who may evaluate and within what limit, and what money
/// needs. The folded README text and the LICENSE hold the rest.
const LICENSE_PHONE_SHOWN: [&[usize]; 3] = [&[0], &[0, 1], &[1]];
/// The quote lines of `quote` in its own group, each line exact words of it,
/// with quotation marks on the first and the last line.
fn license_phone_quote(out: &mut String, quote: &str, (x, y): (f64, f64), room: f64) -> f64 {
    // Room for the two quotation marks.
    let lines = pledges_phone_fit(quote, 20.0, room - 2.0 * 0.62 * 20.0 / PLEDGES_PHONE_SLACK);
    out.push_str(r#"<g class="lp-q">"#);
    let mut y = y;
    for (k, line) in lines.iter().enumerate() {
        let open = if k == 0 { "\"" } else { "" };
        let close = if k + 1 == lines.len() { "\"" } else { "" };
        out.push_str(&format!(
            r#"<text x="{x:.1}" y="{y:.1}" font-size="20">{open}{}{close}</text>"#,
            xml(line)
        ));
        y += 25.0;
    }
    out.push_str("</g>");
    y
}
/// The license on a phone: one tinted box per subject with the quotes it is
/// given, word for word, then what every copy carries and that LICENSE controls.
fn license_phone_draw(dark: bool, columns: &[LicensePhoneColumn], notice: &str) -> String {
    let t = theme(dark);
    let side = PLEDGES_PHONE_SIDE;
    let right = f64::from(PHONE_WIDTH) - side;
    let mut b = String::from(r#"<text class="kick" x="30" y="44">LICENSE AT A GLANCE</text>"#);
    let mut head = String::new();
    let title = pledges_phone_fit(
        "GEL RAM Noncommercial Reciprocal License 1.0",
        24.0,
        right - side,
    );
    let y = pledges_phone_lines(
        &mut head,
        &title,
        (side, 78.0, 29.0),
        r#"font-size="24" class="b""#,
    );
    let sub = pledges_phone_fit(
        "Source-available, not an OSI-approved open-source license.",
        20.0,
        right - side,
    );
    let mut y = pledges_phone_lines(
        &mut head,
        &sub,
        (side, y + 2.0, 25.0),
        r#"font-size="20" class="mut""#,
    );
    b.push_str(&format!(r#"<g class="lp-col">{head}</g>"#));
    y -= 12.0;
    let colours = [t.green, t.blue, t.amber];
    for (i, (section, heading, quotes)) in columns.iter().enumerate() {
        let c = colours[i % colours.len()];
        let (x, room) = (side + 14.0, right - side - 28.0);
        // Under its heading as on the wide card: beside it, two do not fit.
        let mut text = format!(
            r#"<text x="{x:.1}" y="{:.1}" font-size="20" class="b">{}</text><text x="{x:.1}" y="{:.1}" font-size="20" class="mono">LICENSE {}</text>"#,
            y + 28.0,
            xml(heading),
            y + 53.0,
            xml(section)
        );
        let mut qy = y + 81.0;
        for q in quotes {
            qy = license_phone_quote(&mut text, q, (x, qy), room) + 4.0;
        }
        let h = qy - 4.0 - 25.0 + 14.0 - y;
        b.push_str(&format!(
            r#"<g class="lp-col" style="animation-delay:{:.2}s"><rect x="{side}" y="{y:.1}" width="{:.1}" height="{h:.1}" rx="12" fill="{c1}" fill-opacity=".10" stroke="{c1}" stroke-width="2"/>{text}</g>"#,
            0.3 + 0.35 * i as f64,
            right - side,
            c1 = c[1]
        ));
        y += h + 8.0;
    }
    // What every copy carries, then who to write to and what controls.
    let delay = 0.3 + 0.35 * columns.len() as f64;
    let mut text = format!(
        r#"<text class="kick" x="{side}" y="{:.1}">EVERY COPY · SECTION 8</text>"#,
        y + 26.0
    );
    let ny = license_phone_quote(&mut text, notice, (side, y + 54.0), right - side);
    let ey = pledges_phone_lines(
        &mut text,
        &[
            "Commercial Agreements:".to_string(),
            "gelram.licensing@gmail.com".to_string(),
        ],
        (side, ny + 8.0, 26.0),
        &format!(r#"font-size="20" class="b" style="fill:{}""#, t.accent),
    );
    let mut controls = Vec::new();
    for part in [
        "A summary adds no rights;",
        "LICENSE controls (section 17).",
    ] {
        controls.extend(pledges_phone_fit(part, 20.0, right - side));
    }
    let end = pledges_phone_lines(
        &mut text,
        &controls,
        (side, ey + 4.0, 25.0),
        r#"font-size="20" class="mut""#,
    );
    b.push_str(&format!(
        r#"<g class="lp-col" style="animation-delay:{delay:.2}s">{text}</g>"#
    ));
    let height = (end - 25.0 + 22.0) as u32;
    let style = format!(".lp-col{{animation:lp-rise .7s cubic-bezier(.2,.8,.2,1) both}}@keyframes lp-rise{{from{{opacity:0;transform:translateY(18px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{.lp-col{{animation:none}}}}{}", RESULT_STYLE.replace("MUTED", t.muted));
    phone_card(
        dark,
        height,
        (
            "License at a glance",
            &xml(&license_phone_text(columns, notice)),
        ),
        &style,
        &b,
    )
}
/// The description of the license phone card: the quotes it draws, in the
/// words of the wide card's description.
fn license_phone_text(columns: &[LicensePhoneColumn], notice: &str) -> String {
    let mut s = String::from(
        "GEL RAM Noncommercial Reciprocal License 1.0 at a glance, quoted from LICENSE.",
    );
    for (section, heading, quotes) in columns {
        s.push_str(&format!(" {heading} ({section}):"));
        for q in quotes {
            s.push_str(&format!(" '{q}'"));
        }
    }
    s.push_str(&format!(" Every copy (section 8): '{notice}' Commercial Agreements: gelram.licensing@gmail.com. A summary adds no rights; LICENSE controls (section 17)."));
    s
}
/// The phone license card of `license`, refused when it no longer says a quote.
fn license_phone_of(dark: bool, license: &str) -> Result<String> {
    license_quotes(license)?;
    let columns: Vec<LicensePhoneColumn> = LICENSE_COLUMNS
        .iter()
        .zip(LICENSE_PHONE_SHOWN)
        .map(|((s, h, q), shown)| (*s, *h, shown.iter().map(|i| q[*i]).collect()))
        .collect();
    Ok(license_phone_draw(dark, &columns, LICENSE_NOTICE_QUOTE))
}
fn license_phone(dark: bool) -> Result<String> {
    license_phone_of(dark, &fs::read_to_string(LICENSE_FILE)?)
}
// ===== PHONE a4 END =====

// ===== PHONE a5 BEGIN =====
/// The blocks of the phone core card: left edge, width, height, depth of the
/// oblique face, the step from one block to the next and the top of the first.
const CORE_PHONE_X: f64 = 104.0;
const CORE_PHONE_W: f64 = 456.0;
const CORE_PHONE_H: f64 = 78.0;
const CORE_PHONE_D: f64 = 5.0;
const CORE_PHONE_STEP: f64 = 84.0;
const CORE_PHONE_TOP: f64 = 186.0;
/// The two lines under the counts, the wide card's legend cut for a phone.
const CORE_PHONE_LEGEND: [&str; 2] = ["Arrow A → B: B builds on A,", "as its Cargo.toml says"];
/// A conservative width of a line: 0.62 × font size per character.
fn core_phone_width(text: &str, size: f64) -> f64 {
    text.chars().count() as f64 * 0.62 * size
}
/// Whether a width fits its room with 8 % to spare.
fn core_phone_fits(width: f64, room: f64) -> bool {
    width <= room * 0.92
}
/// The parts in the order the phone card stacks them: the parts joined by an
/// arrow, by the longest way of arrows that leads to them and then in table
/// order, so every arrow runs down; then the parts that stand alone.
fn core_phone_order(m: &CoreMap) -> (Vec<usize>, Vec<usize>) {
    let alone = m.alone();
    let depth = m.depths();
    let mut joined: Vec<usize> = (0..m.parts.len()).filter(|i| !alone.contains(i)).collect();
    joined.sort_by_key(|&i| (depth[i], i));
    (joined, alone)
}
/// Where the phone card puts things: the stacking row and the top of the block
/// of each part, the frame (top, bottom) around the parts that stand alone, and
/// the height of the card.
struct CorePhoneLayout {
    row: Vec<usize>,
    top: Vec<f64>,
    frame: Option<(f64, f64)>,
    height: f64,
}
fn core_phone_layout(m: &CoreMap) -> CorePhoneLayout {
    let (joined, alone) = core_phone_order(m);
    let (mut row, mut top) = (vec![0; m.parts.len()], vec![0.0; m.parts.len()]);
    let mut bottom = CORE_PHONE_TOP;
    for (k, &i) in joined.iter().enumerate() {
        (row[i], top[i]) = (k, CORE_PHONE_TOP + CORE_PHONE_STEP * k as f64);
        bottom = top[i] + CORE_PHONE_H;
    }
    let mut frame = None;
    if !alone.is_empty() {
        let f = if joined.is_empty() {
            CORE_PHONE_TOP - 14.0
        } else {
            bottom + 14.0
        };
        for (k, &i) in alone.iter().enumerate() {
            row[i] = joined.len() + k;
            top[i] = f + 44.0 + CORE_PHONE_STEP * k as f64;
            bottom = top[i] + CORE_PHONE_H;
        }
        frame = Some((f, bottom + 12.0));
        bottom += 12.0;
    }
    CorePhoneLayout {
        row,
        top,
        frame,
        height: bottom + 18.0,
    }
}
/// Refuses a part or a count whose text would not fit its box on the phone card.
fn core_phone_check(m: &CoreMap) -> Result<()> {
    let (n, e) = (m.parts.len(), m.links.len());
    let deps = if e == 1 { "dependency" } else { "dependencies" };
    let room = CORE_PHONE_W - 38.0;
    // A count and its word share a line: the first stops before the second starts.
    let count = |v: usize, word: &str| {
        core_phone_width(&v.to_string(), 52.0) + 10.0 + core_phone_width(word, 24.0)
    };
    if !core_phone_fits(count(n, "parts"), 190.0) || !core_phone_fits(count(e, deps), 312.0) {
        return Err("the counts do not fit the phone core card".into());
    }
    let mut lines: Vec<(String, f64, f64)> = vec![
        (CORE_PHONE_LEGEND[0].to_string(), 20.0, 520.0),
        (CORE_PHONE_LEGEND[1].to_string(), 20.0, 520.0),
    ];
    for p in &m.parts {
        lines.push((p.tag[0].clone(), 20.0, room));
        lines.push((p.tag[1].clone(), 20.0, room));
        // The name and the crate share the first line of a block.
        let both = core_phone_width(&p.name, 24.0) + 16.0 + core_phone_width(&p.krate, 20.0);
        if !core_phone_fits(both, room) {
            return Err(format!(
                "{} and {} do not fit one line of a phone block",
                p.name, p.krate
            )
            .into());
        }
    }
    for (text, size, room) in &lines {
        if !core_phone_fits(core_phone_width(text, *size), *room) {
            return Err(format!("{text} does not fit the phone core card").into());
        }
    }
    Ok(())
}
/// The phone card of the memory core: the counts, the legend, then one block
/// per part stacked down the card, the arrows as arcs left of the blocks, and
/// the parts that stand alone framed below. The blocks rise in, the arcs draw
/// in, one pulse runs along them once, and the picture holds.
fn core_phone_draw(dark: bool, m: &CoreMap) -> Result<String> {
    core_phone_check(m)?;
    let t = theme(dark);
    let (n, e) = (m.parts.len(), m.links.len());
    let deps = if e == 1 { "dependency" } else { "dependencies" };
    let lay = core_phone_layout(m);
    let purple = CORE_PURPLE[usize::from(dark)];
    let faces = |h: CoreHue| match h {
        CoreHue::Blue => t.blue,
        CoreHue::Green => t.green,
        CoreHue::Amber => t.amber,
        CoreHue::Purple => purple,
    };
    let face = if dark { t.bg[1] } else { t.paper[0] };
    let enter = |row: usize| 0.45 + 0.16 * row as f64;
    let after = enter(n) + 0.5;
    let (x, w, h, d) = (CORE_PHONE_X, CORE_PHONE_W, CORE_PHONE_H, CORE_PHONE_D);
    // Each count with its word beside it, on one line.
    let beside = |x: f64, v: usize| x + core_phone_width(&v.to_string(), 52.0) + 10.0;
    let mut b = format!(
        r#"<text class="kick" x="40" y="44">THE MEMORY CORE</text><g class="pn" style="animation-delay:.05s"><text x="40" y="104" font-size="52" class="big">{n}</text><text x="{:.1}" y="104" font-size="24" class="mut">parts</text></g><g class="pn" style="animation-delay:.2s"><text x="240" y="104" font-size="52" class="big">{e}</text><text x="{:.1}" y="104" font-size="24" class="mut">{deps}</text></g><text x="40" y="140" font-size="20" class="mut">{}</text><text x="40" y="166" font-size="20" class="mut">{}</text>"#,
        beside(40.0, n),
        beside(240.0, e),
        xml(CORE_PHONE_LEGEND[0]),
        xml(CORE_PHONE_LEGEND[1])
    );
    if let Some((top, bottom)) = lay.frame {
        b.push_str(&format!(
            r#"<g class="px" style="animation-delay:{:.2}s"><rect x="{:.1}" y="{top:.1}" width="{:.1}" height="{:.1}" rx="16" fill="{}" fill-opacity=".05" stroke="{}" stroke-width="2" stroke-dasharray="7 6"/><text x="{:.1}" y="{:.1}" font-size="20" class="b mut">stand alone</text></g>"#,
            enter(n - m.alone().len()),
            x - 16.0,
            w + d + 28.0,
            bottom - top,
            t.accent,
            t.muted,
            x + 6.0,
            top + 28.0
        ));
    }
    for (i, p) in m.parts.iter().enumerate() {
        let y = lay.top[i];
        let hue = faces(p.hue);
        b.push_str(&format!(
            r#"<g class="pb" style="animation-delay:{:.2}s">{}<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{face}" stroke="{}"/><rect x="{x}" y="{y}" width="8" height="{h}" fill="{}"/><text x="{:.1}" y="{:.1}" font-size="24" class="b">{}</text><text class="mono" x="{:.1}" y="{:.1}" font-size="20" text-anchor="end">{}</text><text x="{:.1}" y="{:.1}" font-size="20">{}</text><text x="{:.1}" y="{:.1}" font-size="20">{}</text></g>"#,
            enter(lay.row[i]),
            oblique(x, y, (w, h, d), [hue[0], face, hue[2]]),
            t.line,
            hue[1],
            x + 22.0,
            y + 25.0,
            xml(&p.name),
            x + w - 16.0,
            y + 25.0,
            xml(&p.krate),
            x + 22.0,
            y + 49.0,
            xml(&p.tag[0]),
            x + 22.0,
            y + 71.0,
            xml(&p.tag[1])
        ));
    }
    // Each arrow leaves and enters at the left edge of a block. Along one edge
    // the arcs to blocks above come first, nearest first, then the arcs to
    // blocks below, farthest first, so that nested arcs do not cross.
    let mut ends: Vec<Vec<((bool, std::cmp::Reverse<usize>), usize, bool)>> =
        vec![Vec::new(); m.parts.len()];
    for (l, &(a, c)) in m.links.iter().enumerate() {
        let (ra, rc) = (lay.row[a], lay.row[c]);
        ends[a].push(((rc > ra, std::cmp::Reverse(rc)), l, true));
        ends[c].push(((ra > rc, std::cmp::Reverse(ra)), l, false));
    }
    let mut at = vec![(0.0, 0.0); e];
    for (i, list) in ends.iter_mut().enumerate() {
        list.sort();
        for (k, (_, l, start)) in list.iter().enumerate() {
            let y = lay.top[i] + h * (k + 1) as f64 / (list.len() + 1) as f64;
            if *start {
                at[*l].0 = y;
            } else {
                at[*l].1 = y;
            }
        }
    }
    let depth = m.depths();
    let mut pulses = String::new();
    for (l, &(a, c)) in m.links.iter().enumerate() {
        let (ya, yc) = at[l];
        let span = lay.row[a].abs_diff(lay.row[c]) as f64;
        let bend = x - (16.0 + 17.0 * span).min(x - 18.0);
        let path = format!(
            "M{:.1} {ya:.1}C{bend:.1} {ya:.1} {bend:.1} {yc:.1} {:.1} {yc:.1}",
            x - 1.0,
            x - 15.0
        );
        let draw = enter(lay.row[a].max(lay.row[c])) + 0.45;
        b.push_str(&format!(
            r#"<path class="pa" style="animation-delay:{draw:.2}s" d="{path}" pathLength="100" fill="none" stroke="{}" stroke-width="3"/><polygon class="ph" style="animation-delay:{:.2}s" points="{:.1},{:.1} {:.1},{yc:.1} {:.1},{:.1}" fill="{}"/>"#,
            t.muted,
            draw + 0.4,
            x - 16.0,
            yc - 7.0,
            x - 1.0,
            x - 16.0,
            yc + 7.0,
            t.muted
        ));
        pulses.push_str(&format!(
            r#"<path class="pp" style="animation-delay:{:.2}s" d="{path}" pathLength="100" fill="none" stroke-width="6" stroke-linecap="round"/>"#,
            after + 0.7 * depth[a] as f64
        ));
    }
    b.push_str(&pulses);
    // The pulse has no stroke outside its keyframes, so a picture that does
    // not move shows the arcs and the blocks alone.
    let style = format!(
        ".big{{fill:{a};font-weight:700}}.pb{{animation:cph-in .55s cubic-bezier(.2,.8,.2,1) both}}.pn{{animation:cph-in .6s ease-out both}}.pa{{animation:cph-draw .5s ease-out both}}.ph,.px{{animation:cph-fade .5s ease-out both}}.pp{{animation:cph-run .9s ease-in-out both}}@keyframes cph-in{{from{{opacity:0;transform:translateY(16px)}}to{{opacity:1;transform:none}}}}@keyframes cph-draw{{from{{stroke-dasharray:100 100;stroke-dashoffset:100}}to{{stroke-dasharray:100 100;stroke-dashoffset:0}}}}@keyframes cph-fade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes cph-run{{from{{stroke:{a};stroke-dasharray:12 200;stroke-dashoffset:20}}to{{stroke:{a};stroke-dasharray:12 200;stroke-dashoffset:-110}}}}@media (prefers-reduced-motion:reduce){{.pb,.pn,.pa,.ph,.px,.pp{{animation:none}}}}{}",
        RESULT_STYLE.replace("MUTED", t.muted),
        a = t.accent
    );
    Ok(phone_card(
        dark,
        lay.height as u32,
        ("The memory core", &xml(&core_text(m))),
        &style,
        &b,
    ))
}
fn core_phone(dark: bool) -> Result<String> {
    core_phone_draw(dark, &CoreMap::read()?)
}

/// The colour of a badge's number.
#[derive(Clone, Copy)]
enum BadgeHue {
    Blue,
    Green,
    Amber,
}
/// The numbers of the badge row, each read from a file of this checkout by
/// the reader of the card that shows it.
struct BadgeFacts {
    /// Views of the Q8 record, and the recorded comparison runs whose verdict
    /// passes for all of them, out of the runs (`Q8Runs`).
    views: usize,
    q8_runs: usize,
    q8_all: usize,
    /// Distinct format mutants in the committed matrix report (`Evidence`).
    mutants: usize,
    /// Process kills of the recorded crash series, the snapshots they
    /// acknowledged and those lost (`CrashSeries`).
    kills: usize,
    acknowledged: usize,
    lost: usize,
    /// Cases faster beyond their A/A band in the recorded set and in its
    /// re-run, out of the cases (`rerun_read`).
    faster_twice: usize,
    cases: usize,
    /// Published commitments (`commitments`).
    commitments: usize,
    /// Packages of Cargo.lock, every one a crate of this checkout (`packages`).
    packages: usize,
    /// The workspace version of Cargo.toml, which must be `RELEASE`.
    version: String,
    /// The kind of license, from the first line of LICENSE.
    license: String,
}
/// The version under `[workspace.package]` of a Cargo.toml.
fn badge_version(toml: &str) -> Result<String> {
    let mut on = false;
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            on = line == "[workspace.package]";
        } else if let Some(v) = line.strip_prefix("version = \"").filter(|_| on) {
            let v = v.strip_suffix('"').ok_or("unexpected version line")?;
            if v.is_empty() || !v.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
                return Err(format!("unexpected workspace version {v}").into());
            }
            return Ok(v.to_string());
        }
    }
    Err("Cargo.toml has no workspace version".into())
}
/// The kind of the license, from its first line.
fn badge_license(license: &str) -> Result<String> {
    match license.lines().next() {
        Some("GEL RAM Noncommercial Reciprocal License 1.0") => Ok("noncommercial".to_string()),
        _ => Err("LICENSE no longer begins with the license the badge names".into()),
    }
}
impl BadgeFacts {
    fn read() -> Result<BadgeFacts> {
        let q8 = Q8Runs::read()?;
        let evidence = Evidence::read()?;
        let (rows, matched): (usize, usize) = evidence.formats.iter().fold((0, 0), |(r, m), f| {
            (r + f.rejected + f.accepted, m + f.matched)
        });
        if matched != rows {
            return Err("a row of the mutation matrix did not match its verdict".into());
        }
        let crash = CrashSeries::read()?;
        let rerun = rerun_read()?;
        rerun_check(&rerun)?;
        let version = badge_version(&fs::read_to_string("Cargo.toml")?)?;
        if RELEASE.strip_prefix('v') != Some(version.as_str()) {
            return Err(format!("Cargo.toml says {version}, the README {RELEASE}").into());
        }
        Ok(BadgeFacts {
            // Every run ends with the exact verdict, or `Q8Runs` refuses; the
            // views are the packed copies one record stands for.
            views: q8.packed / q8.record,
            q8_runs: q8.ok,
            q8_all: q8.n,
            mutants: evidence.fact_counts().2,
            kills: crash.trials.len(),
            acknowledged: crash.acknowledged,
            lost: crash.lost,
            faster_twice: rerun.again,
            cases: rerun.cases,
            commitments: commitments()?,
            packages: packages()?,
            version,
            license: badge_license(&fs::read_to_string(LICENSE_FILE)?)?,
        })
    }
}
/// One badge: its file name, label, number, the repository file it links to,
/// its alternative text and its colour.
struct Badge {
    name: &'static str,
    label: String,
    value: String,
    link: String,
    alt: String,
    hue: BadgeHue,
}
/// The badges, two by two in the pairs a phone shows side by side; each links
/// to the file its number is read from or that states it.
fn badges_of(f: &BadgeFacts) -> Vec<Badge> {
    let badge = |name, label: &str, value: String, link: &str, alt: String, hue| Badge {
        name,
        label: label.to_string(),
        value,
        link: link.to_string(),
        alt,
        hue,
    };
    let runs = if f.q8_runs == f.q8_all {
        format!("{}", f.q8_all)
    } else {
        format!("{} of {}", f.q8_runs, f.q8_all)
    };
    vec![
        badge(
            "views",
            "exact runs",
            format!("{}/{}", f.q8_runs, f.q8_all),
            "docs/evidence-q8-current/README.md",
            format!(
                "{} of {} views score bit for bit as their written-out copies in {runs} recorded runs",
                f.views, f.views
            ),
            BadgeHue::Green,
        ),
        badge(
            "mutants",
            "format mutants",
            grouped(f.mutants),
            "docs/evidence-mutation/matrix-r1.txt",
            format!(
                "Mutation matrix: {} format mutants, each row as expected",
                grouped(f.mutants)
            ),
            BadgeHue::Green,
        ),
        badge(
            "lost",
            &format!("lost in {} kills", grouped(f.kills)),
            format!("{}/{}", grouped(f.lost), grouped(f.acknowledged)),
            "docs/CRASH-SERIES.md",
            format!(
                "Crash series: {} of {} confirmed snapshots lost in {} process kills, not a power cut",
                grouped(f.lost),
                grouped(f.acknowledged),
                grouped(f.kills)
            ),
            BadgeHue::Green,
        ),
        badge(
            "packages",
            "local packages",
            grouped(f.packages),
            "Cargo.lock",
            format!(
                "{} packages in Cargo.lock, none from outside this checkout",
                grouped(f.packages)
            ),
            BadgeHue::Blue,
        ),
        badge(
            "faster",
            "faster, both runs",
            format!("{}/{}", f.faster_twice, f.cases),
            "docs/speed-results/2026-11-04-reruns/README.md",
            format!(
                "Speed: {} of {} cases faster beyond their A/A band in both runs, one machine",
                f.faster_twice, f.cases
            ),
            BadgeHue::Blue,
        ),
        badge(
            "commitments",
            "commitments",
            grouped(f.commitments),
            "docs/COMMITMENTS.md",
            format!("{} published commitments", grouped(f.commitments)),
            BadgeHue::Amber,
        ),
        badge(
            "version",
            "version",
            f.version.clone(),
            &format!("RELEASE-NOTES-v{}.md", f.version),
            format!("Version {}: release notes", f.version),
            BadgeHue::Blue,
        ),
        badge(
            "license",
            "license",
            f.license.clone(),
            LICENSE_FILE,
            format!("License: {}, source-available; LICENSE controls", f.license),
            BadgeHue::Amber,
        ),
    ]
}
/// Badge geometry: height, label and number sizes, and the padding of a part.
/// Two badges stand side by side in a 358-px phone column.
const BADGE_H: f64 = 32.0;
const BADGE_LABEL_PX: f64 = 14.0;
const BADGE_VALUE_PX: f64 = 16.0;
const BADGE_PAD: f64 = 7.0;
/// The clear edge around the pill, so that badges wrapped onto two lines of a
/// narrow screen do not touch.
const BADGE_EDGE: f64 = 1.0;
/// Advance widths, in thousandths of the font size, of the lower-case letters
/// a..z of Arial (and of Helvetica and Liberation Sans, which share them),
/// regular then bold.
const BADGE_LOWER: [[u16; 26]; 2] = [
    [
        556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333,
        500, 278, 556, 500, 722, 500, 500, 500,
    ],
    [
        556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389,
        556, 333, 611, 556, 778, 556, 556, 500,
    ],
];
/// The width of a badge text in Arial: digits are 0.556 of the size, space
/// and the marks between numbers 0.278 (0.333 for a bold colon); any other
/// character counts as wide as the widest capital, 0.944.
fn badge_text_width(text: &str, size: f64, bold: bool) -> f64 {
    let per_mille: u32 = text
        .chars()
        .map(|c| match c {
            'a'..='z' => u32::from(BADGE_LOWER[usize::from(bold)][c as usize - 'a' as usize]),
            '0'..='9' => 556,
            ':' if bold => 333,
            ' ' | ',' | '.' | '/' | ':' => 278,
            _ => 944,
        })
        .sum();
    f64::from(per_mille) * size / 1000.0
}
/// The widths of the label part and of the number part: the text in Arial
/// with 8 % to spare, for a fallback font, and the padding on both sides.
fn badge_parts(b: &Badge) -> (f64, f64) {
    let part = |text: &str, size: f64, bold: bool| {
        (badge_text_width(text, size, bold) / 0.92).ceil() + 2.0 * BADGE_PAD
    };
    (
        part(&b.label, BADGE_LABEL_PX, false),
        part(&b.value, BADGE_VALUE_PX, true),
    )
}
/// The size of a badge picture: the pill and its clear edge.
fn badge_size(b: &Badge) -> (f64, f64) {
    let (lw, vw) = badge_parts(b);
    (lw + vw + 2.0 * BADGE_EDGE, BADGE_H + 2.0 * BADGE_EDGE)
}
fn badge_path(name: &str, dark: bool) -> String {
    let theme = if dark { "dark" } else { "light" };
    format!("media/presentation/badges/{name}-{theme}.svg")
}
/// One badge as a two-part pill, label | number. A light band sweeps across
/// it once and the number rises in; the `i`-th badge starts a little later, so
/// the row lights up from left to right. Still, it shows the final state.
fn badge_svg(dark: bool, i: usize, b: &Badge) -> String {
    let t = theme(dark);
    let (lw, vw) = badge_parts(b);
    let (w, h) = (lw + vw, BADGE_H);
    let (sw, sh) = badge_size(b);
    let hue = match b.hue {
        BadgeHue::Blue => t.blue,
        BadgeHue::Green => t.green,
        BadgeHue::Amber => t.amber,
    };
    // Light: a deep label and a deep number with white text; dark: a muted
    // label and a bright number with dark text.
    let (label_fill, label_ink, value_fill, value_ink) = if dark {
        (t.line, t.ink, hue[0], t.bg[1])
    } else {
        (t.ink, t.paper[0], hue[2], t.paper[0])
    };
    let delay = 0.15 * i as f64;
    let r = h / 2.0;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{sw}" height="{sh}" viewBox="-{BADGE_EDGE} -{BADGE_EDGE} {sw} {sh}" role="img" aria-labelledby="t">
<title id="t">{title}</title>
<style>
text{{font-family:Arial,Helvetica,sans-serif}}
.bl{{font-size:{BADGE_LABEL_PX}px;fill:{label_ink}}}
.bv{{font-size:{BADGE_VALUE_PX}px;font-weight:700;fill:{value_ink};animation:bg-rise .6s ease-out {rise:.2}s both}}
.bs{{animation:bg-shine 1.4s ease-in-out {delay:.2}s both}}
@keyframes bg-rise{{from{{opacity:0;transform:translateY(8px)}}to{{opacity:1;transform:none}}}}
@keyframes bg-shine{{from{{transform:translateX(0)}}to{{transform:translateX({sweep}px)}}}}
@media (prefers-reduced-motion:reduce){{.bv,.bs{{animation:none}}}}
</style>
<defs><clipPath id="pill"><rect width="{w}" height="{h}" rx="{r}"/></clipPath><linearGradient id="shine" x2="1"><stop stop-color="#fff" stop-opacity="0"/><stop offset=".5" stop-color="#fff" stop-opacity=".45"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient></defs>
<g clip-path="url(#pill)"><rect width="{lw}" height="{h}" fill="{label_fill}"/><rect x="{lw}" width="{vw}" height="{h}" fill="{value_fill}"/><rect class="bs" x="-70" width="50" height="{h}" fill="url(#shine)"/></g>
<rect x=".5" y=".5" width="{iw}" height="{ih}" rx="{ir}" fill="none" stroke="{line}"/>
<text class="bl" x="{lx}" y="21" text-anchor="middle">{label}</text><text class="bv" x="{vx}" y="22" text-anchor="middle">{value}</text>
</svg>
"##,
        title = xml(&format!("{}: {}", b.label, b.value)),
        rise = delay + 0.3,
        sweep = w + 140.0,
        iw = w - 1.0,
        ih = h - 1.0,
        ir = r - 0.5,
        line = t.line,
        lx = lw / 2.0,
        vx = lw + vw / 2.0,
        label = xml(&b.label),
        value = xml(&b.value),
    )
}
/// The README row: every badge links to its file and shows the reader's theme.
/// The width is the badge's own, so it is shown at its size; the README fixes
/// no image height, which a narrowed width would stretch.
fn badges_html(badges: &[Badge]) -> String {
    let cells: Vec<String> = badges
        .iter()
        .map(|b| {
            let (w, _) = badge_size(b);
            format!(
                r#"<a href="{}"><picture><source media="(prefers-color-scheme: dark)" srcset="{}"><img alt="{}" src="{}" width="{}"></picture></a>"#,
                xml(&b.link),
                badge_path(b.name, true),
                xml(&b.alt),
                badge_path(b.name, false),
                w
            )
        })
        .collect();
    format!(r#"<p align="center">{}</p>"#, cells.join(" "))
}
/// The row of clickable badges under the header: each a number read from a file
/// of this repository, each linking to a file of it.
fn badges_row() -> Result<String> {
    Ok(badges_html(&badges_checked(&BadgeFacts::read()?)?))
}
/// The badges of `f`, refused when one links to a file this checkout lacks.
fn badges_checked(f: &BadgeFacts) -> Result<Vec<Badge>> {
    let badges = badges_of(f);
    for b in &badges {
        if !Path::new(&b.link).is_file() {
            return Err(format!("the {} badge links to a missing file {}", b.name, b.link).into());
        }
    }
    Ok(badges)
}
/// The badge pictures, (path, svg), for `assets()`.
fn badge_assets() -> Result<Vec<(String, String)>> {
    let badges = badges_checked(&BadgeFacts::read()?)?;
    let mut out = Vec::new();
    for (i, b) in badges.iter().enumerate() {
        for dark in [false, true] {
            out.push((badge_path(b.name, dark), badge_svg(dark, i, b)));
        }
    }
    Ok(out)
}
// ===== PHONE a5 END =====

// ===== VISUAL status BEGIN =====
/// The three columns: heading, the start of the README status-table row the
/// items are quoted from, and the items. `status_rows` holds every item to its
/// own row (the last column also to the line under the card).
const STATUS_COLUMNS: [(&str, &str, [&str; 3]); 3] = [
    (
        "Check it here",
        "| **You can check it here** |",
        [
            "One record read back exactly through four views",
            "A changed byte refused",
            "A killed process losing nothing it confirmed",
        ],
    ),
    (
        "Private",
        "| **Private** |",
        [
            "The step that prints text into records",
            "The private build that answers from them",
            "Both exist only on the author's machine",
        ],
    ),
    (
        "Not established",
        "| **Not established** |",
        [
            "The text AI is a goal, not a result",
            "Hardware-level memory computation",
            "Power-cut durability of snapshots and of the record history",
        ],
    ),
];
/// The start of the line under the card, which names what is not established.
const STATUS_LINE: &str = "The text AI is a goal, not a result (";
/// The foot of the first and of the last column; each is quoted like the items.
const STATUS_FEET: [&str; 2] = ["One command each", "NOT_ESTABLISHED"];
/// The foot of the middle column, beside the number of published commitments.
const STATUS_PLEDGE: [&str; 3] = [
    "published commitments",
    "fix private files",
    "as of their dates",
];
/// Refuses unless every item and foot of a column is quoted from its own row of
/// the README's status table; the last column may also quote the line under the card.
fn status_rows(readme: &str) -> Result<()> {
    let low = readme.to_lowercase();
    let line = low
        .lines()
        .find(|l| l.starts_with(&STATUS_LINE.to_lowercase()))
        .ok_or("the README has no line under the status card")?;
    for (i, (heading, row, items)) in STATUS_COLUMNS.iter().enumerate() {
        let mut words = low
            .lines()
            .find(|l| l.starts_with(&row.to_lowercase()))
            .ok_or(format!("the README has no status row {row}"))?
            .to_string();
        let mut quoted: Vec<&str> = items.to_vec();
        match i {
            0 => quoted.push(STATUS_FEET[0]),
            2 => {
                words.push_str(line);
                quoted.push(STATUS_FEET[1]);
            }
            _ => {}
        }
        for item in quoted {
            if !words.contains(&item.to_lowercase()) {
                return Err(format!("{heading}: the README row does not say {item}").into());
            }
        }
    }
    Ok(())
}
/// A conservative width test: 0.62 of the font size per character, plus any
/// letter spacing, must leave 8 % of the room free, so a wider fallback font fits.
fn status_fit(text: &str, size: f64, spacing: f64, room: f64) -> Result<()> {
    let need = text.chars().count() as f64 * (0.62 * size + spacing);
    if need > 0.92 * room {
        return Err(format!("{text} does not fit {room:.0} px at {size} px").into());
    }
    Ok(())
}
/// What the status card draws from files: the dated rows of the published commitments.
struct StatusFacts {
    commitments: usize,
}
impl StatusFacts {
    fn read() -> Result<StatusFacts> {
        status_rows(NATIVE)?;
        let f = StatusFacts {
            commitments: commitments()?,
        };
        f.check()?;
        Ok(f)
    }
    fn check(&self) -> Result<()> {
        if self.commitments == 0 {
            return Err("no published commitment to draw".into());
        }
        Ok(())
    }
}
/// What the status card says, as its description and as the README alternative text.
fn status_text(f: &StatusFacts) -> String {
    format!("Checkable here: exact read-back through four views, a changed byte refused, nothing confirmed lost to a kill. Private: printing and answering; {} published commitments fix the contents of private directories as of their dates; only their labels say what each holds. The text AI is a goal.", f.commitments)
}
/// The drawn icon of a column, centred on (0, 0): a check mark, a closed lock, an hourglass.
fn status_icon(i: usize, colour: &str, hole: &str) -> String {
    let draw = r#"class="sd" pathLength="100" stroke-dasharray="100" fill="none" stroke-linecap="round" stroke-linejoin="round""#;
    match i {
        0 => {
            format!(r#"<path {draw} d="M-13 1L-4 10L14 -10" stroke="{colour}" stroke-width="6"/>"#)
        }
        1 => format!(
            r#"<path {draw} d="M-8.5 -3V-9A8.5 8.5 0 0 1 8.5 -9V-3" stroke="{colour}" stroke-width="4.5"/><rect x="-14" y="-3" width="28" height="21" rx="4" fill="{colour}"/><circle cx="0" cy="5.5" r="3" fill="{hole}"/><rect x="-1.4" y="6" width="2.8" height="6" fill="{hole}"/>"#
        ),
        _ => format!(
            r#"<path {draw} d="M-11 -15H11M-11 15H11M-8.5 -15C-8.5 -3 8.5 3 8.5 15M8.5 -15C8.5 -3 -8.5 3 -8.5 15" stroke="{colour}" stroke-width="3.5"/><path d="M-5.5 12H5.5L0 5Z" fill="{colour}"/>"#
        ),
    }
}
/// Three columns rising in one after another, each icon drawn and pulsed once;
/// the count of published commitments pops in. Every line is tested against
/// the room it has, so a long count or a wide font is refused, not clipped.
fn status_draw(dark: bool, f: &StatusFacts) -> Result<String> {
    f.check()?;
    let t = theme(dark);
    let colours = [t.good, t.accent, if dark { t.amber[0] } else { t.amber[2] }];
    let kicker = "WHERE GEL RAM STANDS";
    status_fit(kicker, 16.0, 2.0, 1120.0)?;
    let mut b = format!(r#"<text class="kick" x="40" y="40">{}</text>"#, xml(kicker));
    let (y0, w, step) = (58.0, 360.0, 24.0);
    // The items of every column, wrapped; the foot boxes line up below the longest.
    let mut last = 0.0_f64;
    let mut items = Vec::new();
    for (i, (_, _, list)) in STATUS_COLUMNS.iter().enumerate() {
        let x = 40.0 + 380.0 * i as f64;
        let mut y = y0 + 100.0;
        let mut s = String::new();
        for (k, item) in list.iter().enumerate() {
            s.push_str(&format!(
                r#"<g class="si" style="animation-delay:{:.2}s"><circle cx="{}" cy="{:.1}" r="4.5" fill="{}"/>"#,
                0.55 + 0.4 * i as f64 + 0.15 * k as f64,
                x + 26.0,
                y - 7.0,
                colours[i]
            ));
            for line in wrap(item, 22) {
                status_fit(&line, 20.0, 0.0, w - 60.0)?;
                s.push_str(&format!(
                    r#"<text x="{}" y="{y:.1}" font-size="20">{}</text>"#,
                    x + 40.0,
                    xml(&line)
                ));
                last = last.max(y);
                y += step;
            }
            s.push_str("</g>");
            y += 10.0;
        }
        items.push(s);
    }
    let (foot, box_h) = (last + 18.0, 80.0);
    let h_col = foot + box_h + 14.0 - y0;
    for (i, (heading, _, _)) in STATUS_COLUMNS.iter().enumerate() {
        let (x, c) = (40.0 + 380.0 * i as f64, colours[i]);
        status_fit(heading, 26.0, 0.0, w - 96.0)?;
        // The coloured bar follows the rounded top corners of its column.
        b.push_str(&format!(
            r#"<g class="sc" style="animation-delay:{:.2}s"><clipPath id="stcol{i}"><rect x="{x}" y="{y0}" width="{w}" height="{h_col:.1}" rx="18"/></clipPath><rect x="{x}" y="{y0}" width="{w}" height="{h_col:.1}" rx="18" fill="{c}" fill-opacity=".07" stroke="{c}" stroke-opacity=".45"/><rect x="{x}" y="{y0}" width="{w}" height="8" fill="{c}" clip-path="url(#stcol{i})"/>"#,
            0.1 + 0.4 * i as f64
        ));
        b.push_str(&format!(
            r#"<g transform="translate({} {})"><g class="sp" style="animation-delay:{:.2}s"><circle r="26" fill="{c}" fill-opacity=".14" stroke="{c}" stroke-width="2.5"/>{}</g></g><text x="{}" y="{}" font-size="26" class="b" style="fill:{c}">{}</text><path d="M{} {}H{}" stroke="{c}" stroke-opacity=".35" stroke-width="1.5"/>"#,
            x + 46.0,
            y0 + 46.0,
            1.3 + 0.4 * i as f64,
            status_icon(i, c, t.bg[0]),
            x + 82.0,
            y0 + 55.0,
            xml(heading),
            x + 24.0,
            y0 + 76.0,
            x + w - 24.0
        ));
        b.push_str(&items[i]);
        b.push_str(&format!(
            r#"<rect x="{}" y="{foot:.1}" width="{}" height="{box_h}" rx="12" fill="{c}" fill-opacity=".13"/>"#,
            x + 16.0,
            w - 32.0
        ));
        if i == 1 {
            // The count, then three short lines beside it.
            let n = f.commitments.to_string();
            let tx = x + 32.0 + 0.7 * 40.0 * n.len() as f64 + 10.0;
            b.push_str(&format!(
                r#"<text class="sn b" x="{}" y="{:.1}" font-size="40" style="fill:{c}">{}</text>"#,
                x + 32.0,
                foot + 54.0,
                xml(&n)
            ));
            for (k, words) in STATUS_PLEDGE.iter().enumerate() {
                status_fit(words, 16.0, 0.0, x + w - 24.0 - tx)?;
                let bold = if k == 0 { r#" class="b""# } else { "" };
                b.push_str(&format!(
                    r#"<text{bold} x="{tx:.1}" y="{:.1}" font-size="16">{}</text>"#,
                    foot + 24.0 + 22.0 * k as f64,
                    xml(words)
                ));
            }
        } else {
            let (words, mono) = if i == 0 {
                (STATUS_FEET[0], "")
            } else {
                (STATUS_FEET[1], "mono ")
            };
            status_fit(words, 20.0, 0.0, w - 32.0)?;
            b.push_str(&format!(
                r#"<text class="{mono}b" x="{}" y="{:.1}" font-size="20" text-anchor="middle" style="fill:{c}">{}</text>"#,
                x + w / 2.0,
                foot + box_h / 2.0 + 7.0,
                xml(words)
            ));
        }
        b.push_str("</g>");
    }
    let style = format!(
        ".kick{{font-size:16px}}.sc{{animation:strise .75s cubic-bezier(.2,.8,.2,1) both}}.si{{animation:stfade .6s ease-out both}}.sd{{animation:stdraw .9s ease-out both}}.sp{{animation:stpulse .7s ease-in-out both;transform-box:fill-box;transform-origin:center}}.sn{{animation:stpop .7s cubic-bezier(.2,.8,.2,1) 1.8s both;transform-box:fill-box;transform-origin:center}}@keyframes strise{{from{{opacity:0;transform:translateY(34px)}}to{{opacity:1;transform:none}}}}@keyframes stfade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes stdraw{{from{{stroke-dashoffset:100}}to{{stroke-dashoffset:0}}}}@keyframes stpulse{{0%,100%{{transform:none}}50%{{transform:scale(1.16)}}}}@keyframes stpop{{from{{opacity:0;transform:scale(.3)}}to{{opacity:1;transform:none}}}}{}@media (prefers-reduced-motion:reduce){{.sc,.si,.sd,.sp,.sn{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    let h = (y0 + h_col + 16.0).ceil() as u32;
    Ok(card(
        (1200, h),
        t,
        (&xml("Where GEL RAM stands"), &xml(&status_text(f))),
        &style,
        &b,
    ))
}
/// Three columns: what can be checked here, what is private, what is not established.
fn status_card(dark: bool) -> Result<String> {
    status_draw(dark, &StatusFacts::read()?)
}
fn status_alt() -> Result<String> {
    Ok(status_text(&StatusFacts::read()?))
}
// ===== VISUAL status END =====

// ===== VISUAL terminal BEGIN =====
/// One check of the README: a `cargo run` line of its code block and the
/// line to look for from the table under it. `look` holds the code spans of
/// that table cell, each with the plain words written before it.
#[derive(Clone, Debug, PartialEq)]
struct TerminalCheck {
    key: String,
    command: String,
    look: Vec<(String, String)>,
}
/// The code spans of a table cell; any text outside them other than the words
/// joining two spans is refused, so the cell holds lines to look for only.
fn terminal_spans(cell: &str) -> Result<Vec<(String, String)>> {
    let parts: Vec<&str> = cell.split('`').collect();
    if parts.len() < 3 || parts.len() % 2 == 0 {
        return Err(format!("no line to look for in the cell: {cell}").into());
    }
    if !parts[0].trim().is_empty() || !parts[parts.len() - 1].trim().is_empty() {
        return Err(format!("text outside the code of the cell: {cell}").into());
    }
    let mut spans = Vec::new();
    for pair in parts[..parts.len() - 1].chunks(2) {
        let code = pair[1].trim();
        if code.is_empty() {
            return Err(format!("an empty code span in the cell: {cell}").into());
        }
        spans.push((pair[0].trim().to_string(), code.to_string()));
    }
    Ok(spans)
}
/// Reads the checks from a README template: the `cargo run` lines after
/// `cargo fetch --locked` in its `sh` block, each matched to exactly one row of
/// the table under "What each line shows, and its limit" by the words of the
/// row's first cell.
fn terminal_checks(template: &str) -> Result<Vec<TerminalCheck>> {
    let lines: Vec<&str> = template.lines().collect();
    let fetch = lines
        .iter()
        .position(|l| l.trim() == "cargo fetch --locked")
        .ok_or("no cargo fetch --locked line")?;
    let fence = lines[..fetch]
        .iter()
        .rposition(|l| l.starts_with("```"))
        .ok_or("cargo fetch --locked is outside a code block")?;
    if lines[fence].trim() != "```sh" {
        return Err("cargo fetch --locked is not in the sh block".into());
    }
    let mut commands = Vec::new();
    let mut closed = false;
    for line in &lines[fetch + 1..] {
        if line.starts_with("```") {
            closed = true;
            break;
        }
        let line = line.trim();
        if !line.starts_with("cargo run ") {
            return Err(format!("not a cargo run line in the check block: {line}").into());
        }
        commands.push(line.to_string());
    }
    if !closed || commands.is_empty() {
        return Err("the check block has no cargo run line or is not closed".into());
    }
    let head = lines
        .iter()
        .position(|l| l.contains("What each line shows, and its limit"))
        .ok_or("no table of the lines to look for")?;
    let top = lines[head..]
        .iter()
        .position(|l| l.starts_with("| Command | Line to look for |"))
        .ok_or("no Command / Line to look for header")?
        + head;
    if !lines.get(top + 1).is_some_and(|l| l.starts_with("|:")) {
        return Err("the table has no separator line".into());
    }
    let mut rows = Vec::new();
    for line in lines[top + 2..].iter().take_while(|l| l.starts_with('|')) {
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
        if cells.len() < 2 {
            return Err(format!("a table row without a line to look for: {line}").into());
        }
        let key = cells[0].replace('`', "").trim().to_string();
        if key.is_empty() {
            return Err(format!("a table row without a command: {line}").into());
        }
        rows.push((key, terminal_spans(cells[1])?));
    }
    if rows.len() != commands.len() {
        return Err(format!(
            "{} cargo run lines but {} table rows",
            commands.len(),
            rows.len()
        )
        .into());
    }
    let names = |row: &str| {
        row.split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    let mut used = vec![false; rows.len()];
    let mut checks = Vec::new();
    for command in commands {
        let words = names(&command);
        let hits: Vec<usize> = (0..rows.len())
            .filter(|&r| {
                let key = names(&rows[r].0);
                words.windows(key.len()).any(|w| w == key.as_slice())
            })
            .collect();
        let [r] = hits[..] else {
            return Err(format!("{} table rows name the command: {command}", hits.len()).into());
        };
        if used[r] {
            return Err(format!("the table row {} names two commands", rows[r].0).into());
        }
        used[r] = true;
        checks.push(TerminalCheck {
            key: rows[r].0.clone(),
            command,
            look: rows[r].1.clone(),
        });
    }
    Ok(checks)
}
/// The lines to look for of one check, joined as the table cell joins them.
fn terminal_look(c: &TerminalCheck) -> String {
    let mut s = String::new();
    for (join, code) in &c.look {
        match join.as_str() {
            "" => {}
            "," => s.push_str(", "),
            w => s.push_str(&format!(" {w} ")),
        }
        s.push_str(code);
    }
    s
}
fn terminal_text(checks: &[TerminalCheck]) -> String {
    let rows: Vec<String> = checks
        .iter()
        .map(|c| format!("{}: {}", c.key, terminal_look(c)))
        .collect();
    format!("Drawn, what to look for: {}.", rows.join("; "))
}
/// Monospace advance (0.6 em): the names and lines are drawn with exactly this
/// length, so the typing cover and the caret land on the last letter.
const TERMINAL_MONO: f64 = 0.6;
/// A conservative advance for the fit checks: a wider fallback font still fits
/// with 8 % to spare.
const TERMINAL_WIDE: f64 = 0.62;
const TERMINAL_SLACK: f64 = 1.08;
/// Font sizes: the typed command name, the lines to look for, the joining word.
const TERMINAL_NAME_PX: f64 = 24.0;
const TERMINAL_LINE_PX: f64 = 16.0;
/// Width of the badges of one check, conservatively estimated: each badge is
/// its text plus 16 px, each joining word its text, 10 px between items.
fn terminal_badges_width(c: &TerminalCheck) -> f64 {
    let px = TERMINAL_WIDE * TERMINAL_LINE_PX;
    let mut w = -10.0;
    for (join, line) in &c.look {
        if join.chars().any(char::is_alphanumeric) {
            w += px * join.chars().count() as f64 + 10.0;
        }
        w += px * line.chars().count() as f64 + 16.0 + 10.0;
    }
    w
}
/// What success looks like, in two columns: each cell types the short name
/// of one command (its table row), then shows a check mark and the line or
/// lines to look for as green badges. A drawing of the README's own lines,
/// not the output of a run; the full commands are only checked, not drawn.
fn terminal_draw(dark: bool, checks: &[TerminalCheck]) -> Result<String> {
    if checks.is_empty() {
        return Err("no checks to draw".into());
    }
    let t = theme(dark);
    let tile = if dark { "#0a1322" } else { "#0f2240" };
    let (code, soft, blue, green) = ("#e6edf3", DARK.muted, DARK.accent, DARK.good);
    let n = checks.len();
    let (left, top, cell_w, cell_h, gap) = (24.0, 66.0, 571.0, 86.0, 10.0);
    let rows = n.div_ceil(2);
    let grid_end = top + rows as f64 * (cell_h + gap) - gap;
    // The note goes into the free cell when there is one, else under the grid.
    let free = n % 2 == 1;
    let h = if free {
        grid_end + 20.0
    } else {
        grid_end + 46.0
    }
    .ceil() as u32;
    let mut b = format!(
        r#"<text x="40" y="46" font-size="24" class="b">Run it yourself — drawn, not a recording</text><text class="kick" x="1160" y="44" text-anchor="end" style="font-size:16px">{n} COMMANDS · WHAT TO LOOK FOR</text>"#
    );
    let mut style = String::new();
    let mut quiet = Vec::new();
    // The whole sequence ends within six seconds, however many cells.
    let slot = (5.2 / n as f64).min(1.2);
    for (k, c) in checks.iter().enumerate() {
        let (x, y) = (
            left + (k % 2) as f64 * (cell_w + gap),
            top + (k / 2) as f64 * (cell_h + gap),
        );
        let len = c.key.chars().count();
        let width = TERMINAL_MONO * TERMINAL_NAME_PX * len as f64;
        let wide = TERMINAL_WIDE * TERMINAL_NAME_PX * len as f64;
        if (58.0 + wide + 44.0) * TERMINAL_SLACK > cell_w {
            return Err(format!("the command name does not fit its cell: {}", c.key).into());
        }
        if terminal_badges_width(c) * TERMINAL_SLACK > cell_w - 22.0 {
            return Err(format!("the lines to look for do not fit: {}", terminal_look(c)).into());
        }
        let at = 0.3 + slot * k as f64;
        let typing = slot * 0.55;
        let shown = at + slot * 0.6;
        style.push_str(&format!(
            ".n{k}{{animation:tlf .3s ease-out {at:.2}s both}}.w{k}{{animation:ttw {typing:.2}s steps({len}) {at:.2}s both}}.k{k}{{animation:tcm{k} {typing:.2}s steps({len}) {at:.2}s both,tcs {:.2}s linear {at:.2}s both}}.p{k}{{animation:tlf .4s ease-out {shown:.2}s both}}@keyframes tcm{k}{{to{{transform:translateX({width:.1}px)}}}}",
            typing + 0.05
        ));
        quiet.extend([
            format!(".n{k}"),
            format!(".w{k}"),
            format!(".k{k}"),
            format!(".p{k}"),
        ]);
        let (xn, yn) = (x + 58.0, y + 36.0);
        b.push_str(&format!(
            r#"<rect x="{x:.0}" y="{y:.0}" width="{cell_w:.0}" height="{cell_h:.0}" rx="14" fill="{tile}" stroke="{line}"/><g class="n{k}"><circle cx="{:.0}" cy="{:.0}" r="16" fill="{blue}" fill-opacity=".16" stroke="{blue}"/><text x="{:.0}" y="{:.0}" font-size="17" class="b" text-anchor="middle" style="fill:{blue}">{}</text></g><text class="tc" x="{xn:.0}" y="{yn:.0}" font-size="{TERMINAL_NAME_PX:.0}" textLength="{width:.1}" lengthAdjust="spacing">{}</text><rect class="tv w{k}" x="{:.0}" y="{:.0}" width="{:.1}" height="32" fill="{tile}"/><rect class="tk k{k}" x="{xn:.0}" y="{:.0}" width="13" height="25" fill="{blue}"/>"#,
            x + 30.0,
            y + 28.0,
            x + 30.0,
            y + 34.0,
            k + 1,
            xml(&c.key),
            xn - 2.0,
            yn - 24.0,
            width + 18.0,
            yn - 21.0,
            line = t.line
        ));
        // The check mark after the name, then the badges on the line below.
        let xc = xn + width + 16.0;
        let ly = y + 68.0;
        let mut g = format!(
            r#"<g class="p{k}"><path d="M{xc:.1} {:.0}l7 7 13-15" fill="none" stroke="{green}" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>"#,
            yn - 9.0
        );
        let mut xb = x + 14.0;
        for (join, line) in &c.look {
            if join.chars().any(char::is_alphanumeric) {
                g.push_str(&format!(
                    r#"<text x="{xb:.1}" y="{ly:.0}" font-size="{TERMINAL_LINE_PX:.0}" style="fill:{soft}">{}</text>"#,
                    xml(join)
                ));
                xb += TERMINAL_WIDE * TERMINAL_LINE_PX * join.chars().count() as f64 + 10.0;
            }
            let w = TERMINAL_MONO * TERMINAL_LINE_PX * line.chars().count() as f64;
            g.push_str(&format!(
                r#"<rect x="{xb:.1}" y="{:.0}" width="{:.1}" height="28" rx="14" fill="{green}" fill-opacity=".14" stroke="{green}"/><text class="tp" x="{:.1}" y="{ly:.0}" font-size="{TERMINAL_LINE_PX:.0}" textLength="{w:.1}" lengthAdjust="spacing">{}</text>"#,
                ly - 20.0,
                w + 16.0,
                xb + 8.0,
                xml(line)
            ));
            xb += TERMINAL_WIDE * TERMINAL_LINE_PX * line.chars().count() as f64 + 16.0 + 10.0;
        }
        g.push_str("</g>");
        b.push_str(&g);
    }
    let note = [
        "Every name and line is read from",
        "this README's code block and table",
    ];
    let (nx, ny) = if free {
        (left + cell_w + gap + 24.0, grid_end - cell_h + 36.0)
    } else {
        (40.0, grid_end + 30.0)
    };
    if free {
        b.push_str(&format!(
            r#"<text x="{nx:.0}" y="{ny:.0}" font-size="18" class="mut">{}</text><text x="{nx:.0}" y="{:.0}" font-size="18" class="mut">{}</text>"#,
            note[0],
            ny + 26.0,
            note[1]
        ));
    } else {
        b.push_str(&format!(
            r#"<text x="{nx:.0}" y="{ny:.0}" font-size="18" class="mut">{} {}</text>"#,
            note[0], note[1]
        ));
    }
    // Base rules hold the final look; only the keyframes hold the start, so a
    // renderer without animation shows every name and line.
    let style = format!(
        ".tc,.tp{{font-family:ui-monospace,Consolas,\"DejaVu Sans Mono\",monospace}}.tc{{fill:{code}}}.tp{{fill:{green};font-weight:700}}.tv{{transform-box:fill-box;transform-origin:right;transform:scaleX(0)}}.tk{{opacity:0}}{style}@keyframes ttw{{from{{transform:scaleX(1)}}to{{transform:scaleX(0)}}}}@keyframes tcs{{0%,100%{{opacity:0}}1%,99%{{opacity:1}}}}@keyframes tlf{{from{{opacity:0;transform:translateX(-10px)}}to{{opacity:1;transform:none}}}}@media (prefers-reduced-motion:reduce){{{}{{animation:none}}}}{}",
        quiet.join(","),
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    Ok(card(
        (1200, h),
        t,
        (
            "Run it yourself — drawn, not a recording",
            &xml(&terminal_text(checks)),
        ),
        &style,
        &b,
    ))
}
/// The commands of the README's code block, each followed by the line to look for.
fn terminal_card(dark: bool) -> Result<String> {
    terminal_draw(dark, &terminal_checks(NATIVE)?)
}
fn terminal_alt() -> Result<String> {
    Ok(terminal_text(&terminal_checks(NATIVE)?))
}
// ===== VISUAL terminal END =====

// ===== VISUAL rerun BEGIN =====
/// The re-run of the speed card's timing set: the same plan, the same trees,
/// timed again by `xtask speed-campaign`. Only this plan's re-run is compared.
const RERUN_DIR: &str = "docs/speed-results/2026-11-04-reruns/rerun-2026-10-20-against-v0.6.1";
/// One run of the cases as the card draws it.
struct RerunRun {
    date: String,
    /// Cases faster beyond their own A/A band, inside it, slower beyond it.
    sides: [usize; 3],
    /// The median over the cases of |aa - 1|: the A/A noise of the run.
    noise: f64,
}
/// The recorded set and its re-run, case by case.
struct RerunData {
    /// The tree A and A2 check out, as the recorded set names it.
    base: String,
    runs: [RerunRun; 2],
    cases: usize,
    /// Cases faster beyond their band in the record and beyond it again in the re-run.
    again: usize,
    /// Cases faster beyond their band in the record whose re-run ratio is above 1.
    above_one: usize,
}
/// One timing program of a run: its name, its cases, and whether A, A2 and B
/// printed the same output digest.
type RerunProgram<'a> = (&'a str, &'a [SpeedCase], bool);
/// Where the two runs come from: the recorded set's directory, the tree it
/// times against and that tree's revision, and the date of each run.
struct RerunOrigin {
    set_dir: String,
    base: String,
    base_rev: String,
    dates: (String, String),
}
/// The fixed order of the card: faster, inside, slower.
fn rerun_sides<'a>(cases: impl Iterator<Item = &'a SpeedCase>) -> [usize; 3] {
    let mut n = [0; 3];
    for c in cases {
        n[match c.side() {
            Side::Faster => 0,
            Side::Within => 1,
            Side::Slower => 2,
        }] += 1;
    }
    n
}
/// The median of |aa - 1|; the mean of the middle two for an even count.
fn rerun_noise(cases: &[&SpeedCase]) -> Result<f64> {
    let mut v: Vec<f64> = cases.iter().map(|c| (c.aa - 1.0).abs()).collect();
    v.sort_by(f64::total_cmp);
    let mid = v.len() / 2;
    match v.len() {
        0 => Err("no case to take the A/A noise of".into()),
        n if n % 2 == 1 => Ok(v[mid]),
        _ => Ok((v[mid - 1] + v[mid]) / 2.0),
    }
}
/// The cases of one run keyed by program and case name, in key order. Refuses
/// an empty run and a repeated case.
fn rerun_keyed<'a>(run: &[RerunProgram<'a>]) -> Result<Vec<(String, &'a SpeedCase)>> {
    let mut v: Vec<(String, &SpeedCase)> = Vec::new();
    for (program, cases, _) in run {
        for c in cases.iter() {
            v.push((format!("{program}/{}", c.name), c));
        }
    }
    v.sort_by(|a, b| a.0.cmp(&b.0));
    if v.is_empty() || v.windows(2).any(|w| w[0].0 == w[1].0) {
        return Err("a run with no case or a repeated case".into());
    }
    Ok(v)
}
/// Pairs the cases of two runs by program and case name. Refuses an empty run,
/// a repeated case, two runs that do not hold exactly the same cases, and a
/// program whose output verdict (same digest in A, A2 and B, or not) changed.
fn rerun_compare(
    base: &str,
    dates: (&str, &str),
    recorded: &[RerunProgram],
    rerun: &[RerunProgram],
) -> Result<RerunData> {
    let (old, new) = (rerun_keyed(recorded)?, rerun_keyed(rerun)?);
    if old.len() != new.len() || old.iter().zip(&new).any(|(a, b)| a.0 != b.0) {
        return Err("the re-run does not hold the same cases as the record".into());
    }
    for (name, _, same) in recorded {
        if rerun.iter().any(|(n, _, s)| n == name && s != same) {
            return Err(format!("{name}: the output verdict differs between the runs").into());
        }
    }
    let pairs: Vec<(&SpeedCase, &SpeedCase)> =
        old.iter().zip(&new).map(|(a, b)| (a.1, b.1)).collect();
    let faster_before = || pairs.iter().filter(|(a, _)| a.side() == Side::Faster);
    let run = |date: &str, cases: Vec<&SpeedCase>| -> Result<RerunRun> {
        Ok(RerunRun {
            date: date.to_string(),
            sides: rerun_sides(cases.iter().copied()),
            noise: rerun_noise(&cases)?,
        })
    };
    Ok(RerunData {
        base: base.to_string(),
        runs: [
            run(dates.0, pairs.iter().map(|p| p.0).collect())?,
            run(dates.1, pairs.iter().map(|p| p.1).collect())?,
        ],
        cases: pairs.len(),
        again: faster_before()
            .filter(|(_, b)| b.side() == Side::Faster)
            .count(),
        above_one: faster_before().filter(|(_, b)| b.gain > 1.0).count(),
    })
}
/// The example names of a CAMPAIGN.txt, in file order. Refuses it unless it is
/// a complete campaign of measured timings that re-runs the recorded set, with
/// A and A2 clean checkouts of the set's base revision, B a clean checkout of
/// the revision the speed card names, and every example checked and as expected.
fn rerun_campaign(text: &str, o: &RerunOrigin) -> Result<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let has = |want: &str| lines.iter().any(|l| *l == want);
    let rev = &o.base_rev;
    let b_clean = |l: &&str| {
        l.starts_with(&format!("tree=b revision={SPEED_B}")) && l.ends_with(" working_tree=clean")
    };
    let refusal = if lines.first() != Some(&"FORMAT=GEL_SPEED_CAMPAIGN_1") {
        Some("not a GEL_SPEED_CAMPAIGN_1 file".to_string())
    } else if lines.last() != Some(&"SPEED_CAMPAIGN=COMPLETE") {
        Some("the campaign is not complete".to_string())
    } else if !has(&format!("reruns={}", o.set_dir)) {
        Some(format!("it does not re-run {}", o.set_dir))
    } else if !has("timing_validity=AS_MEASURED") {
        Some("its timings are not as measured".to_string())
    } else if !has(&format!("tree=a revision={rev} working_tree=clean"))
        || !has(&format!("tree=a2 revision={rev} working_tree=clean"))
    {
        Some(format!("A and A2 are not clean checkouts of {rev}"))
    } else if !lines.iter().any(b_clean) {
        Some(format!("B is not a clean checkout of {SPEED_B}"))
    } else {
        None
    };
    if let Some(why) = refusal {
        return Err(format!("CAMPAIGN.txt: {why}").into());
    }
    let mut names: Vec<String> = Vec::new();
    for line in lines.iter().filter(|l| l.starts_with("example=")) {
        let words: Vec<&str> = line.split(' ').collect();
        let name = words[0].trim_start_matches("example=");
        if !words.contains(&"check=PASS") || !words.contains(&"met=yes") {
            return Err(format!("CAMPAIGN.txt: {name} is not checked or not as expected").into());
        }
        if names.iter().any(|n| n == name) {
            return Err(format!("CAMPAIGN.txt: {name} is repeated").into());
        }
        names.push(name.to_string());
    }
    if names.is_empty() {
        return Err("CAMPAIGN.txt: no example".into());
    }
    Ok(names)
}
/// Refuses a manifest that is not a `GEL_SPEED_AB_2` record of measured timings.
fn rerun_manifest(text: &str) -> Result<()> {
    let mut lines = text.lines();
    if lines.next() != Some("FORMAT=GEL_SPEED_AB_2") {
        return Err("not a GEL_SPEED_AB_2 manifest".into());
    }
    if !lines.any(|l| l == "timing_validity=AS_MEASURED") {
        return Err("its timings are not as measured".into());
    }
    Ok(())
}
/// One program of the re-run as read: its name, cases and output verdict.
type RerunOwned = (String, Vec<SpeedCase>, bool);
/// The programs of the re-run from the texts of its files: CAMPAIGN.txt, then
/// per program directory its name, manifest.txt and summary.txt. The program
/// directories must be exactly the examples of the campaign. An error starts
/// with the file it comes from, relative to the re-run directory.
fn rerun_programs_from(
    campaign: &str,
    programs: &[(String, String, String)],
    o: &RerunOrigin,
) -> Result<Vec<RerunOwned>> {
    let mut examples = rerun_campaign(campaign, o)?;
    examples.sort();
    let mut names: Vec<&str> = programs.iter().map(|p| p.0.as_str()).collect();
    names.sort();
    if names != examples {
        return Err("CAMPAIGN.txt: its examples are not the program directories".into());
    }
    let mut out = Vec::new();
    for (name, manifest, summary) in programs {
        rerun_manifest(manifest).map_err(|e| format!("{name}/manifest.txt: {e}"))?;
        let (cases, same) =
            speed_summary(summary).map_err(|e| format!("{name}/summary.txt: {e}"))?;
        out.push((name.clone(), cases, same));
    }
    Ok(out)
}
/// Reads the re-run directory. A directory whose name is not a plain program
/// name is refused before any file in it is read. Only CAMPAIGN.txt and each
/// program's manifest.txt and summary.txt are read: the round files, warm-up
/// included, are not.
fn rerun_programs(dir: &str, o: &RerunOrigin) -> Result<Vec<RerunOwned>> {
    let read = |p: String| fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"));
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !plain_name(&name) {
                return Err(format!("{dir}/{name}: unexpected program directory").into());
            }
            names.push(name);
        }
    }
    names.sort();
    let campaign = read(format!("{dir}/CAMPAIGN.txt"))?;
    let mut programs = Vec::new();
    for name in names {
        let (m, s) = (
            read(format!("{dir}/{name}/manifest.txt"))?,
            read(format!("{dir}/{name}/summary.txt"))?,
        );
        programs.push((name, m, s));
    }
    rerun_programs_from(&campaign, &programs, o).map_err(|e| format!("{dir}/{e}").into())
}
/// The date a timing directory's name starts with.
fn rerun_date(name: &str) -> Result<String> {
    let date = name.get(..10).unwrap_or("");
    let shape = date.len() == 10
        && date.bytes().enumerate().all(|(i, b)| match i {
            4 | 7 => b == b'-',
            _ => b.is_ascii_digit(),
        });
    if !shape {
        return Err(format!("{name} does not start with a date").into());
    }
    Ok(date.to_string())
}
/// Where the runs come from, from the two directory paths and the recorded
/// set's README. The set is named `DATE-against-BASE`; the re-run directory is
/// `DATE-reruns/rerun-` and the set's name. The README's title names the same
/// base and date, and its A and A2 row the base's tag and full revision.
fn rerun_origin(set_dir: &str, rerun_dir: &str, readme: &str) -> Result<RerunOrigin> {
    let set_name = set_dir.rsplit('/').next().unwrap_or("");
    let mut parts = rerun_dir.rsplit('/');
    let (plan, campaign) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    if plan != format!("rerun-{set_name}") || !campaign.ends_with("-reruns") {
        return Err(format!("{rerun_dir} is not the re-run of {set_name}").into());
    }
    let dates = (rerun_date(set_name)?, rerun_date(campaign)?);
    let base = set_name
        .strip_prefix(&format!("{}-against-", dates.0))
        .filter(|b| !b.is_empty() && b.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'.'))
        .ok_or(format!("{set_name} does not name its base"))?;
    if readme.lines().next() != Some(&format!("# Timing against {base}, {}", dates.0)) {
        return Err(format!("{set_dir}/README.md: its title does not name {base}").into());
    }
    let base_rev = readme
        .lines()
        .find_map(|l| {
            l.strip_prefix(&format!(
                "| A and A2 | two clean checkouts of the {base} tag, `"
            ))?
            .strip_suffix("` |")
        })
        .filter(|r| r.len() == 40 && r.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
        .ok_or(format!(
            "{set_dir}/README.md: no full revision of {base} for A and A2"
        ))?;
    Ok(RerunOrigin {
        set_dir: set_dir.to_string(),
        base: base.to_string(),
        base_rev: base_rev.to_string(),
        dates,
    })
}
/// Reads the speed card's set and its re-run and pairs their cases.
fn rerun_read() -> Result<RerunData> {
    let path = format!("{SPEED_DIR}/README.md");
    let readme = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
    let o = rerun_origin(SPEED_DIR, RERUN_DIR, &readme)?;
    let set = SpeedSet::read()?;
    let recorded: Vec<RerunProgram> = set
        .examples
        .iter()
        .filter_map(|e| {
            Some((
                e.name.as_str(),
                &e.timed.as_ref()?.0[..],
                e.timed.as_ref()?.1,
            ))
        })
        .collect();
    let programs = rerun_programs(RERUN_DIR, &o)?;
    let rerun: Vec<RerunProgram> = programs
        .iter()
        .map(|(n, c, s)| (n.as_str(), &c[..], *s))
        .collect();
    rerun_compare(&o.base, (&o.dates.0, &o.dates.1), &recorded, &rerun)
}
/// Refuses data that does not add up: every run's counts sum to the cases, the
/// cases faster again are among those faster before and faster now, those with
/// a ratio above 1 include them, the noise is a number.
fn rerun_check(d: &RerunData) -> Result<()> {
    let adds_up = d.cases > 0
        && !d.base.is_empty()
        && d.runs.iter().all(|r| {
            r.sides.iter().sum::<usize>() == d.cases && r.noise.is_finite() && r.noise >= 0.0
        })
        && d.again <= d.above_one
        && d.above_one <= d.runs[0].sides[0]
        && d.again <= d.runs[1].sides[0];
    if !adds_up {
        return Err("the re-run counts do not add up".into());
    }
    Ok(())
}
/// A noise value as the card prints it: up to four decimals, no trailing zero.
fn rerun_number(v: f64) -> String {
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}
/// The card's title: what the picture is, with no verdict on the set as a whole.
fn rerun_title(d: &RerunData) -> String {
    format!("The same {} cases, timed twice", d.cases)
}
/// The alternative text, also the card's description.
fn rerun_text(d: &RerunData) -> Result<String> {
    rerun_check(d)?;
    let ([f0, i0, s0], [f1, i1, s1]) = (d.runs[0].sides, d.runs[1].sides);
    let (d0, d1) = (&d.runs[0].date, &d.runs[1].date);
    Ok(format!(
        "{} against {}, each against its own A/A band: {f0} faster, {i0} inside, {s0} slower on {d0}; {f1} faster, {i1} inside, {s1} slower on {d1}; {} of the {f0} faster again ({} with a ratio above 1). A/A noise {} → {}. One machine, not pinned to cores.",
        rerun_title(d),
        d.base,
        d.again,
        d.above_one,
        rerun_number(d.runs[0].noise),
        rerun_number(d.runs[1].noise),
    ))
}
/// A conservative width of a line of text: 0.62 × font size per character.
fn rerun_width(text: &str, size: f64) -> f64 {
    0.62 * size * text.chars().count() as f64
}
const RERUN_STYLE: &str = ".mut{fill:MUTED}.b{font-weight:700}.rn-g{animation:rn-grow 1s cubic-bezier(.2,.8,.2,1) both;transform-box:view-box;transform-origin:X0px 0}.rn-f{animation:rn-fade .6s ease-out both}.rn-p{animation:rn-pop .5s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:center}.rn-d{animation:rn-drop .6s ease-out 1.7s both,rn-glow 1.8s ease-in-out 2.4s 3 both;transform-box:fill-box;transform-origin:top}@keyframes rn-grow{from{transform:scaleX(0)}to{transform:none}}@keyframes rn-fade{from{opacity:0}to{opacity:1}}@keyframes rn-pop{from{opacity:0;transform:scale(.6)}to{opacity:1;transform:none}}@keyframes rn-drop{from{opacity:0;transform:scaleY(0)}to{opacity:1;transform:none}}@keyframes rn-glow{0%,100%{fill-opacity:.18}50%{fill-opacity:.34}}@media (prefers-reduced-motion:reduce){.rn-g,.rn-f,.rn-p,.rn-d{animation:none}}";
/// Two stacked bars, one per run, of faster / inside / slower cases; between
/// them the cases faster beyond their band in both; under them the A/A noise
/// of each run. No program and no case is named.
fn rerun_draw(dark: bool, d: &RerunData) -> Result<String> {
    rerun_check(d)?;
    let t = theme(dark);
    let (x0, width, bar) = (236.0, 700.0, 56.0);
    let u = width / d.cases as f64;
    let tops = [196.0, 344.0];
    // Inside the band: a neutral grey that stands out from the card on both themes.
    let grey = if dark { "#8b949e" } else { "#6e7781" };
    let fills = [t.blue[1], grey, t.red[1]];
    let inner = [
        "#ffffff",
        if dark { "#0d1117" } else { "#ffffff" },
        "#ffffff",
    ];
    let outer = if dark {
        [t.blue[0], t.muted, t.red[0]]
    } else {
        [t.blue[2], t.muted, t.red[2]]
    };
    let (d0, d1) = (xml(&d.runs[0].date), xml(&d.runs[1].date));
    let base = xml(&d.base);
    let mut b = format!(
        r#"<g class="rn-f"><text class="kick" style="font-size:16px" x="40" y="46">RE-RUN · {d0} → {d1}</text><text x="40" y="90" font-size="30" class="b">{}</text><text x="40" y="124" font-size="20" class="mut">{base} time / new time, each case against its own A/A band</text></g>"#,
        xml(&rerun_title(d))
    );
    // The legend.
    let mut lx = 40.0;
    for (k, word) in [
        "faster beyond its A/A band",
        "inside it",
        "slower beyond it",
    ]
    .iter()
    .enumerate()
    {
        b.push_str(&format!(
            r#"<g class="rn-f" style="animation-delay:.2s"><rect x="{lx:.0}" y="148" width="18" height="18" rx="4" fill="{}"/><text x="{:.0}" y="164" font-size="20">{word}</text></g>"#,
            fills[k],
            lx + 28.0
        ));
        lx += 28.0 + rerun_width(word, 20.0) + 32.0;
    }
    for (r, run) in d.runs.iter().enumerate() {
        let (top, rf) = (tops[r], r as f64);
        // A bar grows for 1 s; its counts pop in as it settles.
        let (grow, count) = (0.3 + 0.4 * rf, 1.2 + 0.4 * rf);
        let role = if r == 0 { "recorded" } else { "re-run" };
        b.push_str(&format!(
            r#"<g class="rn-f" style="animation-delay:{grow:.1}s"><text x="40" y="{:.0}" font-size="22" class="b">{}</text><text x="40" y="{:.0}" font-size="18" class="mut">{role}</text></g><g class="rn-g" style="animation-delay:{grow:.1}s">"#,
            top + 24.0,
            xml(&run.date),
            top + 48.0,
        ));
        let mut x = x0;
        let mut labels = String::new();
        for k in 0..3 {
            let n = run.sides[k];
            let w = n as f64 * u;
            if n > 0 {
                b.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{top:.0}" width="{w:.1}" height="{bar:.0}" fill="{}"/>"#,
                    fills[k]
                ));
            }
            // A wide segment holds its count; a narrow one has it above or below.
            let cx = x + w / 2.0;
            labels.push_str(&if w >= rerun_width(&n.to_string(), 24.0) * 1.08 + 8.0 {
                format!(
                    r#"<text class="rn-p b" style="animation-delay:{count:.1}s;fill:{}" x="{cx:.1}" y="{:.0}" font-size="24" text-anchor="middle">{n}</text>"#,
                    inner[k],
                    top + bar / 2.0 + 8.5
                )
            } else {
                let y = if r == 0 { top - 6.0 } else { top + bar + 26.0 };
                format!(
                    r#"<text class="rn-p b" style="animation-delay:{count:.1}s;fill:{}" x="{cx:.1}" y="{y:.0}" font-size="20" text-anchor="middle">{n}</text>"#,
                    outer[k]
                )
            });
            x += w;
        }
        b.push_str(&format!(
            r#"<rect x="{x0:.0}" y="{top:.0}" width="{width:.0}" height="{bar:.0}" rx="3" fill="none" stroke="{}"/></g>{labels}"#,
            t.line
        ));
    }
    // Right of the bars: the size of the set, then how the slower count moved.
    let tag = format!("slower: {} → {}", d.runs[0].sides[2], d.runs[1].sides[2]);
    let tag_w = rerun_width(&tag, 18.0) * 1.08 + 24.0;
    let right = x0 + width + 16.0;
    b.push_str(&format!(
        r#"<text class="rn-f" style="animation-delay:.7s" x="{right:.0}" y="{:.0}" font-size="18">{} cases</text><g class="rn-p" style="animation-delay:2.0s"><rect x="{right:.0}" y="{:.0}" width="{tag_w:.0}" height="32" rx="16" fill="{}" fill-opacity=".14" stroke="{}" stroke-width="1.5"/><text x="{:.0}" y="{:.0}" font-size="18" class="b" text-anchor="middle" style="fill:{}">{}</text></g>"#,
        tops[0] + bar / 2.0 + 6.0,
        d.cases,
        tops[1] + bar / 2.0 - 16.0,
        t.red[1],
        t.red[1],
        right + tag_w / 2.0,
        tops[1] + bar / 2.0 + 6.0,
        outer[2],
        xml(&tag)
    ));
    // The cases faster beyond their band in both runs: a band from bar to bar.
    let (gap0, gap1) = (tops[0] + bar, tops[1]);
    let span = d.again as f64 * u;
    let mid = (gap0 + gap1) / 2.0;
    let pill_text = format!("{} of {} faster again", d.again, d.runs[0].sides[0]);
    let note = format!("{} with a ratio above 1", d.above_one);
    let pill = rerun_width(&pill_text, 22.0) * 1.08 + 28.0;
    let px = (x0 + span / 2.0).max(x0 + pill / 2.0);
    let side = (x0 + span + 16.0)
        .max(px + pill / 2.0 + 16.0)
        .min(1176.0 - rerun_width(&note, 18.0) * 1.08);
    b.push_str(&format!(
        r#"<rect class="rn-d" x="{x0:.0}" y="{gap0:.0}" width="{span:.1}" height="{:.0}" fill="{a}" style="fill-opacity:.18"/><path class="rn-f" style="animation-delay:1.7s" d="M{x0:.0} {gap0:.0}V{gap1:.0}M{:.1} {gap0:.0}V{gap1:.0}" stroke="{a}" stroke-width="2" stroke-dasharray="5 4"/><g class="rn-p" style="animation-delay:2.1s"><rect x="{:.1}" y="{:.0}" width="{pill:.0}" height="40" rx="20" fill="{pf}"/><text x="{px:.1}" y="{:.0}" font-size="22" class="b" text-anchor="middle" style="fill:#ffffff">{}</text></g><text class="rn-f mut" style="animation-delay:2.3s" x="{side:.1}" y="{:.0}" font-size="18">{}</text>"#,
        gap1 - gap0,
        x0 + span,
        px - pill / 2.0,
        mid - 20.0,
        mid + 8.0,
        xml(&pill_text),
        mid + 6.0,
        xml(&note),
        a = t.accent,
        // White text needs the deeper blue on the dark theme.
        pf = if dark { t.blue[2] } else { t.accent }
    ));
    // The A/A noise of each run, on one linear scale.
    let most = d.runs[0].noise.max(d.runs[1].noise);
    let meter = 300.0;
    b.push_str(r#"<g class="rn-f" style="animation-delay:2.4s"><text x="40" y="476" font-size="22" class="b">A/A noise</text><text x="40" y="500" font-size="18" class="mut">median |aa - 1|</text></g>"#);
    for (r, run) in d.runs.iter().enumerate() {
        let y = 458.0 + 32.0 * r as f64;
        let w = if most > 0.0 {
            (run.noise / most * meter).max(if run.noise > 0.0 { 3.0 } else { 0.0 })
        } else {
            0.0
        };
        b.push_str(&format!(
            r#"<rect class="rn-g" style="animation-delay:{:.1}s" x="{x0:.0}" y="{y:.0}" width="{w:.1}" height="18" rx="3" fill="{}"/><text class="rn-f" style="animation-delay:2.9s" x="{:.1}" y="{:.0}" font-size="20"><tspan class="b">{}</tspan><tspan class="mut"> · {}</tspan></text>"#,
            2.4 + 0.2 * r as f64,
            if r == 0 { grey } else { t.accent },
            x0 + w + 12.0,
            y + 16.0,
            xml(&rerun_number(run.noise)),
            xml(&run.date)
        ));
    }
    b.push_str(&format!(
        r#"<text class="rn-p b" style="animation-delay:3.1s" x="1160" y="500" font-size="36" text-anchor="end">{} → {}</text><text class="rn-f mut" style="animation-delay:3.1s" x="40" y="548" font-size="18">One machine, not pinned to cores · A, A2: two checkouts of {base} · B: main at {}</text>"#,
        xml(&rerun_number(d.runs[0].noise)),
        xml(&rerun_number(d.runs[1].noise)),
        xml(SPEED_B)
    ));
    Ok(card(
        (1200, 574),
        t,
        (&xml(&rerun_title(d)), &xml(&rerun_text(d)?)),
        &RERUN_STYLE
            .replace("MUTED", t.muted)
            .replace("X0", &format!("{x0:.0}")),
        &b,
    ))
}
/// The timing set of 2026-10-20 beside its re-run of 2026-11-04.
fn rerun_card(dark: bool) -> Result<String> {
    rerun_draw(dark, &rerun_read()?)
}
fn rerun_alt() -> Result<String> {
    rerun_text(&rerun_read()?)
}
// ===== VISUAL rerun END =====

// ===== VISUAL glance BEGIN =====
/// The binaries guide lists the tool programs that answer `--version` and `--license`.
const GLANCE_BINARIES: &str = "docs/BINARIES.md";
const GLANCE_NUMBER_WORDS: [&str; 13] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve",
];
/// The roadmap, whose section of the earlier version states the older fact.
const GLANCE_ROADMAP: &str = "docs/ROADMAP.md";
/// What the roadmap section of the earlier version says of the programs.
const GLANCE_SINCE_FACT: &str =
    "Every program prints one identity line for `--version`, and for `--license`";
/// The kept memory record of a public revision: its MEMORY.txt is counted, and
/// the README links its README.
const GLANCE_MEMORY: &str = "docs/memory-results/2026-11-08";
/// The fault tests of saved files, one file per crate.
const GLANCE_FAULTS: [&str; 3] = [
    "crates/gel-history/tests/faults.rs",
    "crates/gel-store/tests/faults.rs",
    "crates/gel-source/tests/faults.rs",
];
/// The reproduction guide, whose section of the short witness lists its steps.
const GLANCE_REPRODUCE: &str = "docs/REPRODUCE.md";
/// The texts the glance card is read from; `GlanceFacts::read` fills them from the files.
struct GlanceTexts<'a> {
    /// `RELEASE` (e.g. v0.9.0) and the release notes of that version.
    release: &'a str,
    notes: &'a str,
    roadmap: &'a str,
    binaries: &'a str,
    /// MEMORY.txt of the kept memory record.
    memory: &'a str,
    /// The fault test files, in the order of `GLANCE_FAULTS`.
    faults: &'a [&'a str],
    reproduce: &'a str,
    readme: &'a str,
}
/// What the glance card shows, each read from the file that states it.
struct GlanceFacts {
    /// The release as major.minor, from `RELEASE`; its release notes name
    /// what the card says is new in it.
    release: String,
    /// The version before it, from the README heading; its roadmap section
    /// names the programs that answer `--license`.
    since: String,
    /// New in the release: the fault tests of saved files, without the
    /// ignored child processes they start.
    faults: usize,
    /// New in the release: the operations of the kept memory record, the
    /// processes of each and the samples, one per process.
    operations: usize,
    processes: usize,
    samples: usize,
    /// New in the release: the steps of `xtask witness`.
    steps: usize,
    /// Since the version before: the tool programs that answer `--version` and `--license`.
    tools: Vec<String>,
}
/// A file of the checkout; an error names the file.
fn glance_file(path: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|e| format!("{path}: {e}").into())
}
/// The test functions of a test file, without the ignored ones: a `#[test]`
/// among the attributes right above a function.
fn glance_tests(text: &str) -> usize {
    let (mut test, mut ignored, mut n) = (false, false, 0);
    for line in text.lines().map(str::trim) {
        if line.starts_with("#[") {
            test |= line == "#[test]";
            ignored |= line.starts_with("#[ignore");
            continue;
        }
        if test && !ignored && line.starts_with("fn ") {
            n += 1;
        }
        (test, ignored) = (false, false);
    }
    n
}
/// The operations of a memory record, the processes of each and its samples:
/// the end line must count the record and sample lines, and every operation
/// must have run as many processes, which add up to the samples.
fn glance_memory(text: &str) -> Result<(usize, usize, usize)> {
    let number = |v: &str| -> Option<usize> {
        if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        v.parse().ok()
    };
    let end = text
        .lines()
        .last()
        .and_then(|l| l.strip_prefix("end records="))
        .and_then(|l| l.split_once(" samples="))
        .and_then(|(r, s)| Some((number(r)?, number(s.split(' ').next()?)?)));
    let (Some((records, samples)), Some("GEL_MEMORY_1")) = (end, text.lines().next()) else {
        return Err(
            format!("{GLANCE_MEMORY}/MEMORY.txt: not a memory record with an end line").into(),
        );
    };
    let reps: Vec<Option<usize>> = text
        .lines()
        .filter(|l| l.starts_with("record "))
        .map(|l| {
            l.split(' ')
                .find_map(|f| f.strip_prefix("reps="))
                .and_then(number)
        })
        .collect();
    let counted = text.lines().filter(|l| l.starts_with("sample ")).count();
    let processes = reps.first().copied().flatten().unwrap_or(0);
    if records == 0
        || reps.len() != records
        || counted != samples
        || reps.iter().any(|r| *r != Some(processes))
        || records.checked_mul(processes) != Some(samples)
    {
        return Err(format!(
            "{GLANCE_MEMORY}/MEMORY.txt: the end line says {records} records and {samples} samples, the file holds {} and {counted}",
            reps.len()
        )
        .into());
    }
    Ok((records, processes, samples))
}
/// The steps of `xtask witness`: the rows of the step table in the section of
/// the short witness, which must name the command and give their number in words.
fn glance_witness(text: &str) -> Result<usize> {
    let section = text
        .split_once("\n## A short witness\n")
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(""))
        .unwrap_or("");
    let names: Vec<&str> = section
        .lines()
        .skip_while(|l| !l.starts_with("| Step |"))
        .skip(2)
        .take_while(|l| l.starts_with("| "))
        .map(|l| l[2..].split(" |").next().unwrap_or(""))
        .collect();
    let plain = |n: &&str| !n.is_empty() && n.bytes().all(|b| b.is_ascii_lowercase() || b == b'-');
    let distinct = names
        .iter()
        .enumerate()
        .all(|(i, n)| !names[..i].contains(n));
    let said = GLANCE_NUMBER_WORDS
        .get(names.len())
        .is_some_and(|w| section.contains(&format!("It runs {w} steps")));
    if names.is_empty()
        || !names.iter().all(plain)
        || !distinct
        || !said
        || !section.contains("-p xtask -- witness ")
    {
        return Err(format!(
            "{GLANCE_REPRODUCE}: the short witness does not list the steps it counts"
        )
        .into());
    }
    Ok(names.len())
}
impl GlanceFacts {
    fn read() -> Result<GlanceFacts> {
        let notes = format!("RELEASE-NOTES-{RELEASE}.md");
        let faults = GLANCE_FAULTS
            .iter()
            .map(|f| glance_file(f))
            .collect::<Result<Vec<String>>>()?;
        let faults: Vec<&str> = faults.iter().map(String::as_str).collect();
        GlanceFacts::from_texts(&GlanceTexts {
            release: RELEASE,
            notes: &glance_file(&notes)?,
            roadmap: &glance_file(GLANCE_ROADMAP)?,
            binaries: &glance_file(GLANCE_BINARIES)?,
            memory: &glance_file(&format!("{GLANCE_MEMORY}/MEMORY.txt"))?,
            faults: &faults,
            reproduce: &glance_file(GLANCE_REPRODUCE)?,
            readme: NATIVE,
        })
    }
    /// Refuses unless the README names the earlier version, the roadmap section
    /// of that version names the programs, the binaries guide lists distinct
    /// programs, as many as the README says, and the memory record, the fault
    /// test files and the guide of the short witness give numbers that the
    /// release notes of `RELEASE` and the README both state.
    fn from_texts(s: &GlanceTexts) -> Result<GlanceFacts> {
        let parts: Vec<&str> = s
            .release
            .strip_prefix('v')
            .unwrap_or("")
            .split('.')
            .collect();
        let numeric = |p: &&str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
        if parts.len() != 3 || !parts.iter().all(numeric) {
            return Err(format!("unexpected release {}", s.release).into());
        }
        let release = format!("{}.{}", parts[0], parts[1]);
        // Line breaks and runs of spaces count as one space.
        let fold = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
        let (notes, readme) = (fold(s.notes), fold(s.readme));
        if !s.notes.starts_with(&format!("# GEL RAM {} ", s.release)) {
            return Err(format!("RELEASE-NOTES-{}.md is not the notes of it", s.release).into());
        }
        let since = s
            .readme
            .lines()
            .find_map(|l| l.strip_prefix(&format!("## New in {release}, and in ")))
            .unwrap_or("");
        let p: Vec<&str> = since.split('.').collect();
        if p.len() != 2 || !p.iter().all(numeric) || since == release {
            return Err(format!("the README names no earlier version beside {release}").into());
        }
        let since = since.to_string();
        // A new release must not relabel the fact of the earlier one: the
        // section of the earlier version states it.
        let section = s
            .roadmap
            .split_once(&format!("\n## v{since}.0\n"))
            .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(""))
            .unwrap_or("");
        if !fold(section).contains(GLANCE_SINCE_FACT) {
            return Err(
                format!("{GLANCE_ROADMAP}: v{since}.0 does not say {GLANCE_SINCE_FACT}").into(),
            );
        }
        let paragraph = s.binaries.lines().collect::<Vec<_>>().join(" ");
        let list = paragraph
            .split_once("Every program of the workspace (")
            .and_then(|(_, rest)| rest.split_once(')'))
            .ok_or(format!("{GLANCE_BINARIES}: no list of programs"))?
            .0;
        let tools: Vec<String> = list
            .split('`')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect();
        let valid = |n: &String| {
            !n.is_empty()
                && n != "xtask"
                && n.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        let distinct = tools
            .iter()
            .enumerate()
            .all(|(i, n)| !tools[..i].contains(n));
        if tools.is_empty() || !tools.iter().all(valid) || !distinct {
            return Err(format!("{GLANCE_BINARIES}: unexpected program list: {list}").into());
        }
        // New in the release: the fault tests, the memory record and the steps
        // of the short witness, each counted in its own file.
        let mut faults = 0;
        for (k, text) in s.faults.iter().enumerate() {
            let n = glance_tests(text);
            if n == 0 {
                let file = GLANCE_FAULTS.get(k).unwrap_or(&"a fault test file");
                return Err(format!("{file}: no test").into());
            }
            faults += n;
        }
        let (operations, processes, samples) = glance_memory(s.memory)?;
        let steps = glance_witness(s.reproduce)?;
        let word = |n: usize| {
            GLANCE_NUMBER_WORDS
                .get(n)
                .ok_or(format!("{n} is too many to name in words"))
        };
        let witness = format!(
            "`xtask witness NEW_DIR` runs the {} short public checks",
            word(steps)?
        );
        let tests = format!("{faults} tests in {} new files", word(s.faults.len())?);
        let memory = format!(
            "{} processes of each of {} operations",
            word(processes)?,
            word(operations)?
        );
        // A new release must not relabel what came later either: its own notes
        // name the numbers the card says are new in it.
        for stated in [
            witness.clone(),
            tests.clone(),
            memory.clone(),
            format!("records={operations} samples={samples}"),
        ] {
            if !notes.contains(&stated) {
                return Err(format!("RELEASE-NOTES-{}.md does not say {stated}", s.release).into());
            }
        }
        for (stated, what) in [
            (
                format!(
                    "the {} tool programs answer `--version` and `--license`",
                    word(tools.len())?
                ),
                "number of tool programs",
            ),
            (witness, "number of witness steps"),
            (tests, "number of fault tests"),
            (format!("{memory}, {samples} samples"), "memory record"),
            (format!("({GLANCE_MEMORY}/README.md)"), "memory record"),
        ] {
            if !readme.contains(&stated) {
                return Err(format!("the README states another {what}: {stated}").into());
            }
        }
        let facts = GlanceFacts {
            release,
            since,
            faults,
            operations,
            processes,
            samples,
            steps,
            tools,
        };
        facts.check()?;
        Ok(facts)
    }
    /// What a drawing needs, also for facts made by hand.
    fn check(&self) -> Result<()> {
        let version = |v: &str, n: usize| {
            let p: Vec<&str> = v.split('.').collect();
            p.len() == n
                && p.iter()
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        };
        if self.tools.is_empty()
            || self.faults == 0
            || self.steps == 0
            || self.operations == 0
            || self.operations.checked_mul(self.processes) != Some(self.samples)
            || !version(&self.release, 2)
            || !version(&self.since, 2)
            || self.since == self.release
        {
            return Err("the glance facts are empty or malformed".into());
        }
        Ok(())
    }
}
/// What the glance card says, as its description and as the README alternative text.
fn glance_text(f: &GlanceFacts) -> String {
    format!("Versions {} and {} at a glance. New in {}: {} fault tests of saved files, no power-loss test; {} memory samples of {} operations, one machine, process-wide, no time; {} steps of xtask witness on any host, one block. Since {}: {} tool programs answer --version and --license.", f.since, f.release, f.release, f.faults, f.samples, f.operations, f.steps, f.since, f.tools.len())
}
/// One tile of a glance card: the big number, the version tag, the colour of
/// the tile, the two lines under the number and the colour of the second.
type GlanceTile = (
    String,
    String,
    &'static str,
    String,
    &'static str,
    &'static str,
);
/// The four tiles, the same on the wide and on the phone card: what is new in
/// the release first, then what came in the version before it.
fn glance_tiles(t: &'static Theme, f: &GlanceFacts) -> [GlanceTile; 4] {
    let (new, since) = (
        format!("new in {}", f.release),
        format!("since {}", f.since),
    );
    [
        (
            f.faults.to_string(),
            new.clone(),
            t.accent,
            "fault tests of saved files".to_string(),
            "no power-loss test among them",
            t.ink,
        ),
        (
            f.samples.to_string(),
            new.clone(),
            t.accent,
            format!("memory samples of {} operations", f.operations),
            "one machine, process-wide, no time",
            t.ink,
        ),
        (
            f.steps.to_string(),
            new,
            t.good,
            "steps of xtask witness, any host".to_string(),
            "one block, no host or user names",
            t.ink,
        ),
        (
            f.tools.len().to_string(),
            since,
            t.accent,
            "tool programs answer".to_string(),
            "--version and --license",
            t.muted,
        ),
    ]
}
/// A conservative width test: 0.62 of the font size per character, plus any
/// letter spacing, must leave 8 % of the room free, so a wider fallback font fits.
fn glance_fit(text: &str, size: f64, spacing: f64, room: f64) -> Result<()> {
    let need = text.chars().count() as f64 * (0.62 * size + spacing);
    if need > 0.92 * room {
        return Err(format!("{text} does not fit {room:.0} px at {size} px").into());
    }
    Ok(())
}
/// Big characters whose digits roll up into place like a counter, with the
/// keyframes and classes they need; the final characters carry class `rf`.
struct GlanceRoll {
    body: String,
    names: Vec<String>,
    style: String,
    clips: usize,
}
impl GlanceRoll {
    fn new() -> GlanceRoll {
        GlanceRoll {
            body: String::new(),
            names: Vec::new(),
            style: String::new(),
            clips: 0,
        }
    }
    /// The advance of one character: wide enough for a wider fallback font,
    /// since every character is placed on its own.
    fn advance(c: char, size: f64) -> f64 {
        size * match c {
            '0'..='9' => 0.64,
            '.' | ',' => 0.34,
            _ => 0.72,
        }
    }
    /// Draws `text` with its baseline at `y`, left-aligned at `x`.
    fn put(&mut self, (x, y): Point, size: f64, text: &str, delay: f64, colour: &str) {
        let line = 1.1 * size;
        self.body.push_str(&format!(
            r#"<g class="roll" aria-label="{}" font-size="{size}" font-weight="700" style="fill:{colour}">"#,
            xml(text)
        ));
        let mut cx = x;
        for (i, c) in text.chars().enumerate() {
            let width = GlanceRoll::advance(c, size);
            let Some(d) = c.to_digit(10) else {
                self.body.push_str(&format!(
                    r#"<text class="rf" x="{cx:.1}" y="{y:.1}" style="fill:{colour}">{}</text>"#,
                    xml(&c.to_string())
                ));
                cx += width;
                continue;
            };
            // A zero rolls a full turn, so every digit moves.
            let steps = if d == 0 { 10 } else { d };
            let name = format!("ro{}s{steps}", size.round() as u32);
            if !self.names.contains(&name) {
                self.style.push_str(&format!(
                    ".{name}{{animation-name:{name}}}@keyframes {name}{{from{{transform:translateY({:.1}px)}}}}",
                    f64::from(steps) * line
                ));
                self.names.push(name.clone());
            }
            let id = format!("glroll{}", self.clips);
            self.clips += 1;
            self.body.push_str(&format!(
                r#"<clipPath id="{id}"><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}"/></clipPath><g clip-path="url(#{id})"><g class="rl {name}" style="animation-delay:{:.2}s">"#,
                cx - 0.04 * size,
                y - 0.92 * size,
                width + 0.08 * size,
                1.16 * size,
                delay + 0.08 * i as f64
            ));
            for k in 0..=steps {
                let class = if k == steps { r#" class="rf""# } else { "" };
                self.body.push_str(&format!(
                    r#"<text{class} x="{cx:.1}" y="{:.1}" style="fill:{colour}">{}</text>"#,
                    y - f64::from(steps - k) * line,
                    k % 10
                ));
            }
            self.body.push_str("</g></g>");
            cx += width;
        }
        self.body.push_str("</g>");
    }
    /// The rules for the rolling digits, and the classes to still under reduced motion.
    fn motion(&self) -> (String, String) {
        let mut stilled = String::from(".rl");
        for n in &self.names {
            stilled.push_str(&format!(",.{n}"));
        }
        (
            format!(
                ".rl{{animation-duration:1.4s;animation-timing-function:cubic-bezier(.2,.8,.2,1);animation-fill-mode:both}}{}",
                self.style
            ),
            stilled,
        )
    }
}
/// The headline of the glance card, in two lines.
const GLANCE_HEADLINE: [&str; 2] = [
    "Checks on your own host · fault tests of saved files ·",
    "a kept memory record · programs that name their license",
];
/// Four tiles in two rows, each tagged with the version that brought it: the
/// fault tests of saved files, the samples of the kept memory record and the
/// steps of the short witness (new in the release), then the programs that
/// name their license (since the version before); numbers roll into place.
/// A line that does not fit its tile is refused.
fn glance_draw(dark: bool, f: &GlanceFacts) -> Result<String> {
    f.check()?;
    let t = theme(dark);
    let mut roll = GlanceRoll::new();
    let kicker = format!("{} AND {} AT A GLANCE", f.since, f.release);
    glance_fit(&kicker, 16.0, 2.0, 1120.0)?;
    let mut b = format!(
        r#"<text class="kick" x="40" y="40">{}</text>"#,
        xml(&kicker)
    );
    for (k, line) in GLANCE_HEADLINE.iter().enumerate() {
        glance_fit(line, 26.0, 0.0, 1120.0)?;
        b.push_str(&format!(
            r#"<text x="40" y="{}" font-size="26" class="b">{}</text>"#,
            78 + 32 * k,
            xml(line)
        ));
    }
    let new = format!("new in {}", f.release);
    let tiles = glance_tiles(t, f);
    let (w, h) = (550.0, 158.0);
    for (k, (big, tag, colour, l1, l2, c2)) in tiles.iter().enumerate() {
        let (x, y) = (
            40.0 + 570.0 * (k % 2) as f64,
            132.0 + 174.0 * (k / 2) as f64,
        );
        let delay = 0.15 + 0.3 * k as f64;
        b.push_str(&format!(
            r#"<g class="gt" style="animation-delay:{delay:.2}s"><clipPath id="gtile{k}"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16"/></clipPath><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16" fill="{colour}" fill-opacity=".07" stroke="{line}"/><rect x="{x}" y="{y}" width="{w}" height="8" fill="{colour}" clip-path="url(#gtile{k})"/>"#,
            line = t.line
        ));
        // The version tag in the upper right: filled when new in this release.
        let pw = tag.chars().count() as f64 * 0.62 * 16.0 + 28.0;
        let px = x + w - 24.0 - pw;
        let (tag_fill, tag_ink) = if *tag == new {
            (format!(r#"fill="{}""#, t.accent), t.bg[0])
        } else {
            (
                format!(r#"fill="none" stroke="{}" stroke-width="1.5""#, t.muted),
                t.muted,
            )
        };
        b.push_str(&format!(
            r#"<rect x="{px:.1}" y="{}" width="{pw:.1}" height="28" rx="14" {tag_fill}/><text x="{:.1}" y="{}" font-size="16" class="b" text-anchor="middle" style="fill:{tag_ink}">{}</text>"#,
            y + 22.0,
            px + pw / 2.0,
            y + 42.0,
            xml(tag)
        ));
        let room = px - x - 40.0;
        if big.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
            let wide: f64 = big.chars().map(|c| GlanceRoll::advance(c, 60.0)).sum();
            if wide > 0.92 * room {
                return Err(format!("{big} does not fit its tile").into());
            }
            roll.put((x + 28.0, y + 80.0), 60.0, big, delay + 0.3, colour);
            b.push_str(&std::mem::take(&mut roll.body));
        } else {
            glance_fit(big, 52.0, 0.0, room)?;
            b.push_str(&format!(
                r#"<text class="gp" style="animation-delay:{:.2}s;fill:{colour}" x="{}" y="{}" font-size="52" font-weight="700">{}</text>"#,
                delay + 0.4,
                x + 28.0,
                y + 78.0,
                xml(big)
            ));
        }
        glance_fit(l1, 20.0, 0.0, w - 56.0)?;
        glance_fit(l2, 18.0, 0.0, w - 56.0)?;
        let mono = if l2.starts_with("--") { "mono " } else { "" };
        b.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="20">{}</text><text class="{mono}b" x="{}" y="{}" font-size="18" style="fill:{c2}">{}</text></g>"#,
            x + 28.0,
            y + 116.0,
            xml(l1),
            x + 28.0,
            y + 142.0,
            xml(l2)
        ));
    }
    let (roll_rules, stilled) = roll.motion();
    let style = format!(
        ".kick{{font-size:16px}}.gt{{animation:glrise .7s cubic-bezier(.2,.8,.2,1) both}}.gp{{animation:glpop .8s cubic-bezier(.2,.8,.2,1) both;transform-box:fill-box;transform-origin:left bottom}}{roll_rules}@keyframes glrise{{from{{opacity:0;transform:translateY(26px)}}to{{opacity:1;transform:none}}}}@keyframes glpop{{0%{{opacity:0;transform:scale(.6)}}70%{{opacity:1;transform:scale(1.06)}}to{{opacity:1;transform:none}}}}{}@media (prefers-reduced-motion:reduce){{.gt,.gp,{stilled}{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    );
    Ok(card(
        (1200, 484),
        t,
        (
            &xml(&format!(
                "Versions {} and {} at a glance",
                f.since, f.release
            )),
            &xml(&glance_text(f)),
        ),
        &style,
        &b,
    ))
}
/// The release at a glance beside the version before it, drawn from the files.
fn glance_card(dark: bool) -> Result<String> {
    glance_draw(dark, &GlanceFacts::read()?)
}
fn glance_alt() -> Result<String> {
    Ok(glance_text(&GlanceFacts::read()?))
}
// ===== VISUAL glance END =====

// ===== VISUAL pledges BEGIN =====
/// The file whose published-commitments table the time line is drawn from.
const PLEDGES_FILE: &str = "docs/COMMITMENTS.md";
/// The count columns a commitments table may have: header, one, many.
const PLEDGES_COUNTED: [(&str, &str, &str); 2] = [
    ("Directories", "directory", "directories"),
    ("Files", "file", "files"),
];
/// One published commitment: only what the table states about it.
#[derive(Clone, Debug, PartialEq)]
struct PledgesRow {
    date: String,
    /// Days since 1970-01-01, to space the dates on the line.
    day: i64,
    label: String,
    counts: [usize; 2],
    hash: String,
}
/// The published-commitments table: what its two count columns count, and its rows in date order.
#[derive(Clone, Debug, PartialEq)]
struct Pledges {
    /// What each count column counts, in the singular and the plural.
    counted: [(&'static str, &'static str); 2],
    rows: Vec<PledgesRow>,
}
impl Pledges {
    /// The table of the repository, refused unless the README's own count agrees.
    fn read() -> Result<Pledges> {
        let text = fs::read_to_string(PLEDGES_FILE).map_err(|e| format!("{PLEDGES_FILE}: {e}"))?;
        pledges_checked(&text, commitments()?)
    }
}
/// The parsed table, refused when its rows are not the `counted` rows the README counts.
fn pledges_checked(text: &str, counted: usize) -> Result<Pledges> {
    let p = pledges_parse(text).map_err(|e| format!("{PLEDGES_FILE}: {e}"))?;
    if p.rows.len() != counted {
        return Err(format!(
            "{PLEDGES_FILE}: {} commitment rows parsed, but the README counts {counted}",
            p.rows.len()
        )
        .into());
    }
    Ok(p)
}
/// The trimmed cells of a table line `| a | b |`; None when it is no table line.
fn pledges_cells(line: &str) -> Option<Vec<&str>> {
    let inner = line.trim_end().strip_prefix('|')?.strip_suffix('|')?;
    Some(inner.split('|').map(str::trim).collect())
}
/// A YYYY-MM-DD calendar date as days since 1970-01-01.
fn pledges_day(date: &str) -> Result<i64> {
    let b = date.as_bytes();
    let shape = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    if !shape {
        return Err(format!("commitment date {date:?} is not YYYY-MM-DD").into());
    }
    let (y, m, d): (i64, i64, i64) = (date[..4].parse()?, date[5..7].parse()?, date[8..].parse()?);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month = [
        31,
        28 + i64::from(leap),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&m) || d < 1 || d > month[(m - 1) as usize] {
        return Err(format!("commitment date {date} is not a calendar day").into());
    }
    // Days from the civil calendar, counted from a March-based year.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    Ok(era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468)
}
/// A count as the table writes it, with commas between groups of three digits.
fn pledges_count(text: &str) -> Result<usize> {
    let n: usize = text
        .replace(',', "")
        .parse()
        .map_err(|_| format!("commitment count {text:?} is not a number"))?;
    if grouped(n) != text {
        return Err(format!("commitment count {text:?} is not written as {}", grouped(n)).into());
    }
    Ok(n)
}
/// The rows under "## Published commitments": date | label | two counts | `SHA-256`.
/// Refused: no table, a header that names other columns, a row of another shape,
/// a date that is no calendar day, dates out of order, a hash of other than 64
/// hex characters, and a table without rows.
fn pledges_parse(text: &str) -> Result<Pledges> {
    let table = text
        .split_once("## Published commitments\n")
        .ok_or("no published-commitments table")?
        .1;
    let mut lines = table
        .lines()
        .take_while(|l| l.is_empty() || l.starts_with('|'))
        .filter(|l| !l.is_empty());
    let head = lines
        .next()
        .and_then(pledges_cells)
        .ok_or("no header line in the commitments table")?;
    if head.len() != 5
        || !head[0].starts_with("Date")
        || head[1] != "Label"
        || !head[4].starts_with("SHA-256")
    {
        return Err(format!("unexpected commitments header: {head:?}").into());
    }
    let words = |cell: &str| {
        PLEDGES_COUNTED
            .iter()
            .find(|c| c.0 == cell)
            .map(|c| (c.1, c.2))
            .ok_or_else(|| format!("unknown count column {cell:?} in the commitments header"))
    };
    let counted = [words(head[2])?, words(head[3])?];
    if counted[0] == counted[1] {
        return Err(format!("the commitments header counts {} twice", head[2]).into());
    }
    let rule = lines
        .next()
        .and_then(pledges_cells)
        .ok_or("no rule line under the commitments header")?;
    if rule.len() != 5
        || rule
            .iter()
            .any(|c| c.is_empty() || !c.chars().all(|ch| ch == '-' || ch == ':'))
    {
        return Err("the commitments header is not followed by its rule line".into());
    }
    let mut rows: Vec<PledgesRow> = Vec::new();
    for line in lines {
        let c = pledges_cells(line)
            .filter(|c| c.len() == 5)
            .ok_or_else(|| format!("commitment row of another shape: {line}"))?;
        let day = pledges_day(c[0])?;
        if let Some(prev) = rows.last() {
            if day < prev.day {
                return Err(format!("commitment {} comes after {}", c[0], prev.date).into());
            }
        }
        let label = c[1].replace('`', "");
        if label.trim().is_empty() {
            return Err(format!("commitment {} has no label", c[0]).into());
        }
        let hash = c[4]
            .strip_prefix('`')
            .and_then(|h| h.strip_suffix('`'))
            .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(|| format!("commitment hash {:?} is not 64 hex characters", c[4]))?;
        rows.push(PledgesRow {
            date: c[0].to_string(),
            day,
            label,
            counts: [pledges_count(c[2])?, pledges_count(c[3])?],
            hash: hash.to_string(),
        });
    }
    if rows.is_empty() {
        return Err("no published commitment".into());
    }
    Ok(Pledges { counted, rows })
}
/// "1 directory", "1,436 directories": a count with the word of its column.
fn pledges_words(n: usize, (one, many): (&str, &str)) -> String {
    format!("{} {}", grouped(n), if n == 1 { one } else { many })
}
/// Both counts of a row under the names of their columns.
fn pledges_counts(p: &Pledges, row: &PledgesRow) -> [String; 2] {
    [
        pledges_words(row.counts[0], p.counted[0]),
        pledges_words(row.counts[1], p.counted[1]),
    ]
}
/// "8 commitments" or "1 commitment".
fn pledges_many(n: usize) -> String {
    format!("{n} commitment{}", if n == 1 { "" } else { "s" })
}
/// What the time line shows, as its description and as the README alternative text.
fn pledges_text(p: &Pledges) -> String {
    format!(
        "{}, {} to {}: each the SHA-256 of a manifest of a private directory, shown by its first 8 hex characters with its {} and {} counts. Checkable with xtask commitment-check once the files are handed over; a hash shows they did not change, not that they are correct.",
        pledges_many(p.rows.len()),
        p.rows[0].date,
        p.rows[p.rows.len() - 1].date,
        p.counted[0].0,
        p.counted[1].0
    )
}
/// The commitment's name: its label up to a colon, the rest is in the table.
fn pledges_name(label: &str) -> &str {
    match label.split_once(": ") {
        Some((name, _)) if !name.trim().is_empty() => name,
        _ => label,
    }
}
/// Every date as MM-DD when all share one year (the title names it), else in full.
fn pledges_tag(p: &Pledges, date: &str) -> String {
    let year = &p.rows[0].date[..4];
    if p.rows.iter().all(|r| r.date.starts_with(year)) {
        date[5..].to_string()
    } else {
        date.to_string()
    }
}
/// A cautious width of `text` at `size` px: 0.62 em per character, wide enough
/// for a fallback font such as DejaVu Sans.
fn pledges_width(text: &str, size: f64) -> f64 {
    text.chars().count() as f64 * 0.62 * size
}
/// `text` in lines no wider than `room` px at `size` px; a word too long is cut.
fn pledges_fit(text: &str, size: f64, room: f64) -> Vec<String> {
    let max = ((room / (0.62 * size)) as usize).max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let chars: Vec<char> = word.chars().collect();
        for piece in chars.chunks(max) {
            let piece: String = piece.iter().collect();
            let len = line.chars().count();
            if len > 0 && len + 1 + piece.chars().count() <= max {
                line.push(' ');
                line.push_str(&piece);
            } else {
                if len > 0 {
                    lines.push(std::mem::take(&mut line));
                }
                line = piece;
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
/// Sizes and places: side padding, the gap between chips, the chip columns, the
/// padding inside a chip and the room its lock takes, and the share of a box a
/// line may fill.
const PLEDGES_SIDE: f64 = 40.0;
const PLEDGES_GAP: f64 = 12.0;
const PLEDGES_COLUMNS: usize = 3;
const PLEDGES_PAD: f64 = 14.0;
const PLEDGES_LOCK: f64 = 34.0;
const PLEDGES_SLACK: f64 = 0.92;
/// Font sizes: the title, a commitment's name, everything else.
const PLEDGES_TITLE: f64 = 26.0;
const PLEDGES_NAME: f64 = 20.0;
const PLEDGES_SMALL: f64 = 16.0;
/// A date on the line (or dates too close for two dots): its dot, its label,
/// placed within the card on a label row above the line, and how many
/// commitments it holds.
struct PledgesDot {
    tag: String,
    x: f64,
    label_x: f64,
    label_row: usize,
    count: usize,
}
/// The least distance between two dots on the line: their 13 px radius twice and a little.
const PLEDGES_DOT_ROOM: f64 = 30.0;
/// One sealed chip: one commitment's name lines, count lines and date tag.
struct PledgesChip {
    row: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    name: Vec<String>,
    counts: Vec<String>,
    tag: String,
}
/// Where everything goes.
struct PledgesLayout {
    x0: f64,
    x1: f64,
    first: i64,
    span: i64,
    ay: f64,
    dots: Vec<PledgesDot>,
    chips: Vec<PledgesChip>,
    foot: f64,
    height: u32,
}
impl PledgesLayout {
    /// The x of a day on the line: dates are spaced by real date.
    fn px(&self, day: i64) -> f64 {
        if self.span == 0 {
            (self.x0 + self.x1) / 2.0
        } else {
            self.x0 + (self.x1 - self.x0) * (day - self.first) as f64 / self.span as f64
        }
    }
}
/// The room a chip line has: the name line leaves room for the lock.
fn pledges_room(w: f64, name: bool) -> f64 {
    let lock = if name { PLEDGES_LOCK } else { 0.0 };
    (w - 2.0 * PLEDGES_PAD - lock) * PLEDGES_SLACK
}
/// The time line on top, one dot per date with the number of its commitments
/// and its label (labels that would touch go to further rows), then one chip
/// per commitment in date order, three to a row.
fn pledges_layout(p: &Pledges) -> PledgesLayout {
    let n = p.rows.len();
    let tags: Vec<String> = p.rows.iter().map(|r| pledges_tag(p, &r.date)).collect();
    let half = (pledges_width(&tags[0], PLEDGES_SMALL) / 2.0).max(13.0);
    let mut l = PledgesLayout {
        x0: PLEDGES_SIDE + half,
        x1: 1200.0 - PLEDGES_SIDE - half,
        first: p.rows[0].day,
        span: p.rows[n - 1].day - p.rows[0].day,
        ay: 0.0,
        dots: Vec::new(),
        chips: Vec::new(),
        foot: 0.0,
        height: 0,
    };
    // Dates closer on the line than one dot share the dot of the first, which
    // is labelled "first → last" and holds the commitments of all of them.
    let mut groups: Vec<(f64, usize, usize)> = Vec::new();
    for (i, r) in p.rows.iter().enumerate() {
        let x = l.px(r.day);
        match groups.last_mut() {
            Some(g) if x - g.0 < PLEDGES_DOT_ROOM => g.2 = i,
            _ => groups.push((x, i, i)),
        }
    }
    let mut ends: Vec<f64> = Vec::new();
    for (x, a, z) in groups {
        let tag = if tags[a] == tags[z] {
            tags[a].clone()
        } else {
            format!("{} → {}", tags[a], tags[z])
        };
        let half = pledges_width(&tag, PLEDGES_SMALL) / 2.0;
        let label_x = x.clamp(PLEDGES_SIDE + half, 1200.0 - PLEDGES_SIDE - half);
        let row = match ends.iter().position(|end| end + 10.0 <= label_x - half) {
            Some(row) => row,
            None => {
                ends.push(f64::MIN);
                ends.len() - 1
            }
        };
        ends[row] = label_x + half;
        l.dots.push(PledgesDot {
            tag,
            x,
            label_x,
            label_row: row,
            count: z + 1 - a,
        });
    }
    l.ay = 112.0 + 20.0 * (ends.len() - 1) as f64;
    let w = (1200.0 - 2.0 * PLEDGES_SIDE - PLEDGES_GAP * (PLEDGES_COLUMNS - 1) as f64)
        / PLEDGES_COLUMNS as f64;
    let mut y = l.ay + 28.0;
    for (k, rows) in p.rows.chunks(PLEDGES_COLUMNS).enumerate() {
        let start = l.chips.len();
        for (j, r) in rows.iter().enumerate() {
            let [a, b] = pledges_counts(p, r);
            let both = format!("{a} · {b}");
            let counts = if pledges_width(&both, PLEDGES_SMALL) <= pledges_room(w, false) {
                vec![both]
            } else {
                [a, b]
                    .iter()
                    .flat_map(|c| pledges_fit(c, PLEDGES_SMALL, pledges_room(w, false)))
                    .collect()
            };
            let name = pledges_fit(pledges_name(&r.label), PLEDGES_NAME, pledges_room(w, true));
            let h = 28.0 + 24.0 * (name.len() - 1) as f64 + 22.0 * counts.len() as f64 + 33.0;
            l.chips.push(PledgesChip {
                row: k * PLEDGES_COLUMNS + j,
                x: PLEDGES_SIDE + (w + PLEDGES_GAP) * j as f64,
                y,
                w,
                h,
                name,
                counts,
                tag: tags[k * PLEDGES_COLUMNS + j].clone(),
            });
        }
        // The chips of one row share the height of the tallest.
        let tall = l.chips[start..].iter().map(|c| c.h).fold(0.0, f64::max);
        for c in &mut l.chips[start..] {
            c.h = tall;
        }
        y += tall + 10.0;
    }
    l.foot = y - 10.0 + 30.0;
    l.height = (l.foot + 24.0 + 16.0) as u32;
    l
}
/// The two footer lines: how a commitment is checked, and what it shows.
const PLEDGES_FOOT: [&str; 2] = [
    "Hashed now · checkable with xtask commitment-check once the files are handed over",
    "A hash shows they did not change, not that they are correct · each code: first 8 hex of its SHA-256",
];
const PLEDGES_STYLE: &str = ".hx{fill:ACCENT;font-weight:700}.ax{transform-box:view-box;transform-origin:X0px 0;animation:pl-draw 1.4s cubic-bezier(.4,0,.2,1) .2s both}.dt{transform-box:fill-box;transform-origin:center;animation:pl-pop .45s cubic-bezier(.2,.8,.2,1) both}.cd{animation:pl-drop .55s cubic-bezier(.2,.8,.2,1) both}.sh{animation:pl-shut .3s cubic-bezier(.5,0,.6,1.6) both}.lk{transform-box:fill-box;transform-origin:center;animation:pl-click .4s ease-out both}@keyframes pl-draw{from{transform:scaleX(0)}to{transform:none}}@keyframes pl-pop{from{opacity:0;transform:scale(.2)}to{opacity:1;transform:none}}@keyframes pl-drop{from{opacity:0;transform:translateY(-24px)}to{opacity:1;transform:none}}@keyframes pl-shut{from{transform:translateY(-6px)}to{transform:none}}@keyframes pl-click{0%,55%{transform:none}75%{transform:scale(1.3)}100%{transform:none}}@media (prefers-reduced-motion:reduce){.ax,.dt,.cd,.sh,.lk{animation:none}}";
/// When the i-th commitment's chip drops in, in seconds.
fn pledges_delay(i: usize) -> f64 {
    1.6 + 0.22 * i as f64
}
/// The commitments on a line of real dates: a dot per date with the number of
/// commitments it holds, and below it one sealed chip per commitment with its
/// name, its two counts, its date and the first 8 hex characters of its hash.
/// The line draws from the left, the dots appear as it reaches them, the chips
/// drop in date order and their locks click shut.
fn pledges_draw(dark: bool, p: &Pledges) -> Result<String> {
    let t = theme(dark);
    let n = p.rows.len();
    if n == 0 {
        return Err("no published commitment".into());
    }
    let l = pledges_layout(p);
    let (x0, x1, ay) = (l.x0, l.x1, l.ay);
    let mut b = format!(
        r#"<text x="{PLEDGES_SIDE}" y="56" font-size="{PLEDGES_TITLE}" class="b">{} · {} → {}</text>"#,
        pledges_many(n),
        xml(&p.rows[0].date),
        xml(&p.rows[n - 1].date)
    );
    // The line with a tick per day, drawn from the left.
    let mut ticks = String::new();
    if l.span > 0 && l.span <= 120 {
        for d in 0..=l.span {
            ticks.push_str(&format!("M{:.1} {:.1}v12", l.px(l.first + d), ay - 6.0));
        }
    }
    b.push_str(&format!(
        r#"<g class="ax"><path d="M{:.1} {ay:.1}H{:.1}" stroke="{}" stroke-width="3" stroke-linecap="round"/><path d="{ticks}" stroke="{}" stroke-width="1.5"/></g>"#,
        x0 - 20.0,
        x1 + 20.0,
        t.floor,
        t.floor
    ));
    // A dot per date, once the line reaches it, with its number of commitments.
    for d in &l.dots {
        b.push_str(&format!(
            r#"<g class="dt" style="animation-delay:{:.2}s"><circle cx="{x:.1}" cy="{ay:.1}" r="13" fill="{acc}" stroke="{bg}" stroke-width="3"/><text x="{x:.1}" y="{:.1}" font-size="{PLEDGES_SMALL}" text-anchor="middle" class="b" style="fill:{bg}">{}</text><text x="{:.1}" y="{:.1}" font-size="{PLEDGES_SMALL}" text-anchor="middle" class="b">{}</text></g>"#,
            0.2 + 1.4 * (d.x - x0) / (x1 - x0).max(1.0),
            ay + 5.5,
            d.count,
            d.label_x,
            ay - 22.0 - 20.0 * d.label_row as f64,
            xml(&d.tag),
            x = d.x,
            acc = t.accent,
            bg = t.bg[0]
        ));
    }
    for c in &l.chips {
        let (r, x, y, w, h) = (&p.rows[c.row], c.x, c.y, c.w, c.h);
        let delay = pledges_delay(c.row);
        let mut lines = String::new();
        let mut base = y + 28.0;
        for (j, line) in c.name.iter().enumerate() {
            if j > 0 {
                base += 24.0;
            }
            lines.push_str(&format!(
                r#"<text x="{:.1}" y="{base:.1}" font-size="{PLEDGES_NAME}" class="b">{}</text>"#,
                x + PLEDGES_PAD,
                xml(line)
            ));
        }
        for line in &c.counts {
            base += 22.0;
            lines.push_str(&format!(
                r#"<text x="{:.1}" y="{base:.1}" font-size="{PLEDGES_SMALL}">{}</text>"#,
                x + PLEDGES_PAD,
                xml(line)
            ));
        }
        let (lx, ly) = (x + w - PLEDGES_PAD - 20.0, y + 12.0);
        b.push_str(&format!(
            r#"<g class="cd" style="animation-delay:{delay:.2}s"><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="12" fill="{acc}" fill-opacity=".07" stroke="{}"/>{lines}<text x="{:.1}" y="{:.1}" font-size="{PLEDGES_SMALL}"><tspan class="b" style="fill:{acc}">{}</tspan><tspan class="mono hx" dx="12">{}</tspan></text><path class="sh" style="animation-delay:{:.2}s" d="M{:.1} {:.1}v-5a6 6 0 0 1 12 0v5" fill="none" stroke="{acc}" stroke-width="2.6"/><rect class="lk" style="animation-delay:{:.2}s" x="{lx:.1}" y="{:.1}" width="20" height="15" rx="3" fill="{acc}"/></g>"#,
            t.line,
            x + PLEDGES_PAD,
            base + 22.0,
            xml(&c.tag),
            xml(&r.hash[..8]),
            delay + 0.45,
            lx + 4.0,
            ly + 7.0,
            delay + 0.45,
            ly + 7.0,
            acc = t.accent
        ));
    }
    b.push_str(&format!(
        r#"<text x="{PLEDGES_SIDE}" y="{:.1}" font-size="{PLEDGES_SMALL}" class="b">{}</text><text x="{PLEDGES_SIDE}" y="{:.1}" font-size="{PLEDGES_SMALL}" class="mut">{}</text>"#,
        l.foot,
        xml(PLEDGES_FOOT[0]),
        l.foot + 24.0,
        xml(PLEDGES_FOOT[1])
    ));
    let style = PLEDGES_STYLE
        .replace("ACCENT", t.accent)
        .replace("X0", &format!("{:.1}", x0 - 20.0))
        + &RESULT_STYLE.replace("MUTED", t.muted);
    Ok(card(
        (1200, l.height),
        t,
        (
            "Published commitments on a time line",
            &xml(&pledges_text(p)),
        ),
        &style,
        &b,
    ))
}
/// The published commitments on a time line.
fn pledges_card(dark: bool) -> Result<String> {
    pledges_draw(dark, &Pledges::read()?)
}
fn pledges_alt() -> Result<String> {
    Ok(xml(&pledges_text(&Pledges::read()?)))
}
// ===== VISUAL pledges END =====

// ===== VISUAL core BEGIN =====
/// One row of the README table of the memory core: the part, its crate and what it does.
struct CoreRow {
    name: String,
    krate: String,
    does: String,
}
/// The colour of a block.
#[derive(Clone, Copy)]
enum CoreHue {
    Blue,
    Green,
    Amber,
    Purple,
}
/// Purple faces (top, side bar, right) in light and dark; the themes have none.
const CORE_PURPLE: [[&str; 3]; 2] = [
    ["#c297ff", "#8250df", "#6639ba"],
    ["#d2a8ff", "#a371f7", "#8957e5"],
];
/// What the card adds to a row of the table: a tag of two short lines that
/// restates the row in the row's own words, the grid slot of its block
/// (column, row) and its colour. Where the blocks stand is chosen here; which
/// arrows join them is read from the crates.
struct CoreTag {
    krate: &'static str,
    tag: [&'static str; 2],
    slot: (f64, f64),
    hue: CoreHue,
}
const CORE_TAGS: [CoreTag; 7] = [
    CoreTag {
        krate: "gel-orb",
        tag: ["128 bytes, 1,024 bits", "round-trip exactly"],
        slot: (0.0, 1.0),
        hue: CoreHue::Blue,
    },
    CoreTag {
        krate: "gel-store",
        tag: ["CRC64 over header", "and over payload"],
        slot: (1.0, 0.0),
        hue: CoreHue::Blue,
    },
    CoreTag {
        krate: "gel-reader",
        tag: ["16 judgments at once,", "not 16 measurements"],
        slot: (1.0, 1.0),
        hue: CoreHue::Blue,
    },
    CoreTag {
        krate: "gel-structural",
        tag: ["XOR; a wrong parent", "is not detected"],
        slot: (1.0, 2.0),
        hue: CoreHue::Green,
    },
    CoreTag {
        krate: "gel-history",
        tag: ["copy or XOR difference", "rebuilt bit for bit"],
        slot: (2.0, 2.0),
        hue: CoreHue::Green,
    },
    CoreTag {
        krate: "gel-phase-quad",
        tag: ["one 1,152-byte record", "four equivalent views"],
        slot: (2.0, 0.0),
        hue: CoreHue::Amber,
    },
    CoreTag {
        krate: "gel-physics",
        tag: ["latency and bandwidth", "of your own RAM"],
        slot: (2.0, 1.0),
        hue: CoreHue::Purple,
    },
];
/// One drawn block: a table row with its tag.
struct CorePart {
    name: String,
    krate: String,
    tag: [String; 2],
    slot: (f64, f64),
    hue: CoreHue,
}
/// The parts in the order of the table, and the dependencies among them as
/// (a, b): part b builds on part a, as b's Cargo.toml declares.
struct CoreMap {
    parts: Vec<CorePart>,
    links: Vec<(usize, usize)>,
}
/// The rows of the table "Each part, and the test that checks it" under
/// "## The memory core" of a README template, in order.
fn core_rows(template: &str) -> Result<Vec<CoreRow>> {
    let mut lines = template
        .lines()
        .skip_while(|l| *l != "## The memory core")
        .skip_while(|l| *l != "| Part | What it does | Check it |");
    if lines.next().is_none() || !lines.next().is_some_and(|l| l.starts_with("|:---")) {
        return Err("no table of the memory core in the template".into());
    }
    let mut rows: Vec<CoreRow> = Vec::new();
    for line in lines.take_while(|l| l.starts_with("| ")) {
        let cells: Vec<&str> = line
            .strip_prefix("| ")
            .and_then(|l| l.strip_suffix(" |"))
            .ok_or(format!("unexpected core row: {line}"))?
            .split(" | ")
            .collect();
        let [part, does, _check] = cells[..] else {
            return Err(format!("unexpected core row: {line}").into());
        };
        let (name, krate) = part
            .strip_suffix("`)")
            .and_then(|p| p.split_once(" (`"))
            .ok_or(format!("no crate in the core row: {line}"))?;
        if name.is_empty()
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ')
            || krate.is_empty()
            || !krate
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!("unexpected part or crate name: {part}").into());
        }
        if rows.iter().any(|r| r.krate == krate) {
            return Err(format!("{krate} is named twice in the core table").into());
        }
        rows.push(CoreRow {
            name: name.to_string(),
            krate: krate.to_string(),
            does: does.to_string(),
        });
    }
    if rows.is_empty() {
        return Err("the core table has no rows".into());
    }
    Ok(rows)
}
const CORE_W: f64 = 310.0;
const CORE_H: f64 = 130.0;
const CORE_D: f64 = 14.0;
/// The top left corner of the front face of a block in grid slot (column, row).
fn core_at((col, row): (f64, f64)) -> Point {
    (40.0 + 398.0 * col, 150.0 + 156.0 * row)
}
/// Whether a line fits inside a block at `size` px, counting `wide` × size per
/// character and leaving 8 % to spare for a wider fallback font.
fn core_fits(text: &str, size: f64, wide: f64) -> bool {
    text.chars().count() as f64 * wide * size <= (CORE_W - 26.0) * 0.92
}
/// The words of a tag that its row must contain: every word of three or more
/// characters and every number, such as `1,024` or `CRC64`.
fn core_words(tag: &str) -> Vec<String> {
    tag.split(|c: char| !(c.is_ascii_alphanumeric() || c == ','))
        .map(|w| w.trim_matches(',').to_ascii_lowercase())
        .filter(|w| w.len() >= 3 || w.chars().any(|c| c.is_ascii_digit()))
        .collect()
}
/// The crates a Cargo.toml names in its `[dependencies]` table. Development
/// and build dependencies are not drawn; any other dependency table, or a line
/// this reader does not understand, is refused rather than skipped.
fn core_deps(toml: &str) -> Result<Vec<String>> {
    let (mut on, mut out) = (false, Vec::new());
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            on = line == "[dependencies]";
            if !on
                && line.contains("dependencies")
                && !line.starts_with("[dev-dependencies]")
                && !line.starts_with("[build-dependencies]")
            {
                return Err(format!("unsupported dependency table {line}").into());
            }
        } else if on && !line.is_empty() && !line.starts_with('#') {
            let name = line.split(['=', '.']).next().unwrap_or("").trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(format!("unexpected dependency line {line}").into());
            }
            out.push(name.to_string());
        }
    }
    Ok(out)
}
/// "a", "a and b", "a, b and c".
fn core_list(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
/// Joins the table rows to their tags and reads each crate's dependencies from
/// its Cargo.toml (through `cargo`, which gives the file's text). Refuses a
/// crate whose Cargo.toml cannot be read, a row without a tag, a tag without a
/// row, a tag word its row does not contain, a line too long for its block,
/// two blocks in one slot, a slot off the grid, an arrow that would cross a
/// block, and parts that stand alone but cannot be framed apart.
fn core_map(
    rows: &[CoreRow],
    tags: &[CoreTag],
    cargo: &dyn Fn(&str) -> Result<String>,
) -> Result<CoreMap> {
    let (mut parts, mut deps): (Vec<CorePart>, Vec<Vec<String>>) = (Vec::new(), Vec::new());
    for row in rows {
        let k = row.krate.as_str();
        deps.push(core_deps(&cargo(k)?).map_err(|e| format!("crates/{k}/Cargo.toml: {e}"))?);
        let tag = tags
            .iter()
            .find(|t| t.krate == k)
            .ok_or(format!("the core card has no tag for {k}"))?;
        let lines = [
            (row.name.as_str(), 30.0, 0.66),
            (k, 19.0, 0.62),
            (tag.tag[0], 19.0, 0.62),
            (tag.tag[1], 19.0, 0.62),
        ];
        if let Some((text, ..)) = lines.iter().find(|(s, size, w)| !core_fits(s, *size, *w)) {
            return Err(format!("{text} does not fit its block").into());
        }
        let does = row.does.to_ascii_lowercase();
        for word in tag.tag.iter().flat_map(|l| core_words(l)) {
            if !does.contains(&word) {
                return Err(format!("the tag of {k} says {word}, its row does not").into());
            }
        }
        let (col, line) = tag.slot;
        if ![0.0, 1.0, 2.0].contains(&col) || ![0.0, 1.0, 2.0].contains(&line) {
            return Err(format!("the slot of {k} is off the grid").into());
        }
        if let Some(p) = parts.iter().find(|p| p.slot == tag.slot) {
            return Err(format!("{k} and {} share a slot", p.krate).into());
        }
        parts.push(CorePart {
            name: row.name.clone(),
            krate: row.krate.clone(),
            tag: tag.tag.map(str::to_string),
            slot: tag.slot,
            hue: tag.hue,
        });
    }
    if let Some(t) = tags
        .iter()
        .find(|t| !rows.iter().any(|r| r.krate == t.krate))
    {
        return Err(format!("the tag of {} has no row in the core table", t.krate).into());
    }
    let mut links = Vec::new();
    for (b, names) in deps.iter().enumerate() {
        for name in names {
            if let Some(a) = parts.iter().position(|p| p.krate == *name) {
                links.push((a, b));
            }
        }
    }
    let m = CoreMap { parts, links };
    m.routes()?;
    m.frame()?;
    Ok(m)
}
impl CoreMap {
    fn read() -> Result<CoreMap> {
        core_map(&core_rows(NATIVE)?, &CORE_TAGS, &|k: &str| {
            let p = format!("crates/{k}/Cargo.toml");
            Ok(fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"))?)
        })
    }
    /// The lowest edge of the blocks.
    fn floor(&self) -> f64 {
        self.parts
            .iter()
            .map(|p| core_at(p.slot).1 + CORE_H)
            .fold(0.0, f64::max)
    }
    /// Each arrow as its path and its head. Between neighbouring columns it
    /// runs through the gap; across a column it runs below every block, which
    /// needs no block under either end. Anything else would cross a block.
    fn routes(&self) -> Result<Vec<(String, String)>> {
        let below = |p: &CorePart| {
            self.parts
                .iter()
                .any(|q| q.slot.0 == p.slot.0 && q.slot.1 > p.slot.1)
        };
        let (mut lane, mut out) = (0.0, Vec::new());
        for &(a, b) in &self.links {
            let (pa, pb) = (&self.parts[a], &self.parts[b]);
            let ((ax, ay), (bx, by)) = (core_at(pa.slot), core_at(pb.slot));
            let step = pb.slot.0 - pa.slot.0;
            if step == 1.0 {
                let (x1, y1, x2, y2) = (
                    ax + CORE_W + CORE_D + 2.0,
                    ay + CORE_H / 2.0,
                    bx - 12.0,
                    by + CORE_H / 2.0,
                );
                let mx = (x1 + x2) / 2.0;
                let d = if y1 == y2 {
                    format!("M{x1:.1} {y1:.1}H{x2:.1}")
                } else {
                    format!("M{x1:.1} {y1:.1}C{mx:.1} {y1:.1} {mx:.1} {y2:.1} {x2:.1} {y2:.1}")
                };
                let head = format!(
                    "{x2:.1},{:.1} {:.1},{y2:.1} {x2:.1},{:.1}",
                    y2 - 7.0,
                    x2 + 10.0,
                    y2 + 7.0
                );
                out.push((d, head));
            } else if step > 1.0 && !below(pa) && !below(pb) {
                let y = self.floor() + 30.0 + 16.0 * lane;
                lane += 1.0;
                let (x1, x2, y2) = (ax + CORE_W / 2.0, bx + CORE_W / 2.0, by + CORE_H + 12.0);
                let d = format!("M{x1:.1} {:.1}V{y:.1}H{x2:.1}V{y2:.1}", ay + CORE_H + 2.0);
                let head = format!(
                    "{:.1},{y2:.1} {x2:.1},{:.1} {:.1},{y2:.1}",
                    x2 - 7.0,
                    y2 - 10.0,
                    x2 + 7.0
                );
                out.push((d, head));
            } else {
                return Err(format!(
                    "{} builds on {}, but an arrow between their slots would cross a block",
                    pb.krate, pa.krate
                )
                .into());
            }
        }
        Ok(out)
    }
    /// How many arrows run below the blocks.
    fn lanes(&self) -> usize {
        self.links
            .iter()
            .filter(|&&(a, b)| self.parts[b].slot.0 - self.parts[a].slot.0 > 1.0)
            .count()
    }
    /// The parts with no dependency either way among the parts.
    fn alone(&self) -> Vec<usize> {
        (0..self.parts.len())
            .filter(|&i| !self.links.iter().any(|&(a, b)| a == i || b == i))
            .collect()
    }
    /// The frame (left, top, right, bottom) around the parts that stand alone,
    /// with room for its caption; refused when another block stands inside it.
    fn frame(&self) -> Result<Option<(f64, f64, f64, f64)>> {
        let alone = self.alone();
        if alone.is_empty() {
            return Ok(None);
        }
        let (mut l, mut t, mut r, mut b) = (f64::MAX, f64::MAX, 0.0_f64, 0.0_f64);
        for &i in &alone {
            let (x, y) = core_at(self.parts[i].slot);
            (l, t) = (l.min(x - 14.0), t.min(y - CORE_D - 40.0));
            (r, b) = (r.max(x + CORE_W + CORE_D + 12.0), b.max(y + CORE_H + 6.0));
        }
        for (i, p) in self.parts.iter().enumerate() {
            let (x, y) = core_at(p.slot);
            if !alone.contains(&i)
                && x < r
                && x + CORE_W + CORE_D > l
                && y - CORE_D < b
                && y + CORE_H > t
            {
                return Err(format!(
                    "the parts that stand alone cannot be framed apart: {} stands among them",
                    p.krate
                )
                .into());
            }
        }
        Ok(Some((l, t, r, b)))
    }
    /// How many arrows lead to each part along the longest way.
    fn depths(&self) -> Vec<usize> {
        let mut depth = vec![0; self.parts.len()];
        for _ in 0..self.parts.len() {
            for &(a, b) in &self.links {
                depth[b] = depth[b].max(depth[a] + 1);
            }
        }
        depth
    }
}
/// What the core card says, as its description and as the README alternative text.
fn core_text(m: &CoreMap) -> String {
    // Parts that build on the same parts are said together.
    let mut groups: Vec<(Vec<usize>, Vec<&str>)> = Vec::new();
    for (b, p) in m.parts.iter().enumerate() {
        let on: Vec<usize> = m.links.iter().filter(|l| l.1 == b).map(|l| l.0).collect();
        if on.is_empty() {
            continue;
        }
        match groups.iter_mut().find(|g| g.0 == on) {
            Some(g) => g.1.push(&p.name),
            None => groups.push((on, vec![&p.name])),
        }
    }
    let said: Vec<String> = groups
        .iter()
        .map(|(on, who)| {
            let on: Vec<&str> = on.iter().map(|&a| m.parts[a].name.as_str()).collect();
            let s = if who.len() == 1 { "s" } else { "" };
            format!("{} build{s} on {}", core_list(who), core_list(&on))
        })
        .collect();
    let e = m.links.len();
    let mut s = format!(
        "The {} parts of the memory core and the {e} {} their Cargo.toml files declare among them",
        m.parts.len(),
        if e == 1 { "dependency" } else { "dependencies" }
    );
    if said.is_empty() {
        s.push('.');
    } else {
        s.push_str(&format!(": {}.", said.join("; ")));
    }
    let alone: Vec<&str> = m
        .alone()
        .iter()
        .map(|&i| m.parts[i].name.as_str())
        .collect();
    if !alone.is_empty() {
        let s1 = if alone.len() == 1 { "s" } else { "" };
        s.push_str(&format!(" {} stand{s1} alone.", core_list(&alone)));
    }
    s
}
/// The parts as blocks with the part name, the crate and the tag, and an arrow
/// for each dependency among them; the parts that stand alone are framed
/// apart. The blocks appear in the order of the table, the arrows draw in, one
/// pulse runs along them once, and the picture holds.
fn core_draw(dark: bool, m: &CoreMap) -> Result<String> {
    let t = theme(dark);
    let (n, e) = (m.parts.len(), m.links.len());
    let routes = m.routes()?;
    let purple = CORE_PURPLE[usize::from(dark)];
    let faces = |h: CoreHue| match h {
        CoreHue::Blue => t.blue,
        CoreHue::Green => t.green,
        CoreHue::Amber => t.amber,
        CoreHue::Purple => purple,
    };
    let front = if dark { t.bg[1] } else { t.paper[0] };
    let enter = |i: usize| 0.2 + 0.22 * i as f64;
    let after = enter(n) + 0.6;
    let title = format!(
        "{n} parts, {e} {} between them",
        if e == 1 { "dependency" } else { "dependencies" }
    );
    let mut b = format!(
        r#"<text class="kick" x="40" y="42" style="font-size:16px">THE MEMORY CORE</text><text x="40" y="80" font-size="30" class="b">{}</text><text x="40" y="114" font-size="20" class="mut">Arrow A → B: B builds on A, as its Cargo.toml says</text>"#,
        xml(&title)
    );
    if let Some((l, top, r, bottom)) = m.frame()? {
        b.push_str(&format!(
            r#"<g class="cx" style="animation-delay:{:.2}s"><rect x="{l:.1}" y="{top:.1}" width="{:.1}" height="{:.1}" rx="18" fill="{}" fill-opacity=".05" stroke="{}" stroke-width="2" stroke-dasharray="7 6"/><text x="{:.1}" y="{:.1}" font-size="18" class="b mut">stand alone</text></g>"#,
            enter(n),
            r - l,
            bottom - top,
            t.accent,
            t.muted,
            l + 18.0,
            top + 27.0
        ));
    }
    for (i, p) in m.parts.iter().enumerate() {
        let (x, y) = core_at(p.slot);
        let hue = faces(p.hue);
        b.push_str(&format!(
            r#"<g class="cb" style="animation-delay:{:.2}s">{}<rect x="{x}" y="{y}" width="{CORE_W}" height="{CORE_H}" fill="{front}" stroke="{}"/><rect x="{x}" y="{y}" width="8" height="{CORE_H}" fill="{}"/><text x="{}" y="{}" font-size="30" class="b">{}</text><text class="mono" x="{}" y="{}" font-size="19">{}</text><text x="{}" y="{}" font-size="19">{}</text><text x="{}" y="{}" font-size="19">{}</text></g>"#,
            enter(i),
            oblique(x, y, (CORE_W, CORE_H, CORE_D), [hue[0], front, hue[2]]),
            t.line,
            hue[1],
            x + 24.0,
            y + 42.0,
            xml(&p.name),
            x + 24.0,
            y + 70.0,
            xml(&p.krate),
            x + 24.0,
            y + 98.0,
            xml(&p.tag[0]),
            x + 24.0,
            y + 121.0,
            xml(&p.tag[1])
        ));
    }
    let depth = m.depths();
    let mut pulses = String::new();
    for (&(a, c), (d, head)) in m.links.iter().zip(&routes) {
        let (cx, cy) = core_at(m.parts[c].slot);
        let draw = enter(a.max(c)) + 0.45;
        let pulse = after + 0.8 * depth[a] as f64;
        b.push_str(&format!(
            r#"<path class="cf" style="animation-delay:{draw:.2}s" d="{d}" pathLength="100" fill="none" stroke="{}" stroke-width="2.6"/><polygon class="ch" style="animation-delay:{:.2}s" points="{head}" fill="{}"/>"#,
            t.muted,
            draw + 0.4,
            t.muted
        ));
        pulses.push_str(&format!(
            r#"<path class="cp" style="animation-delay:{pulse:.2}s" d="{d}" pathLength="100" fill="none" stroke-width="7" stroke-linecap="round"/><rect class="cg" style="animation-delay:{:.2}s" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="8" fill="none" stroke-width="3"/>"#,
            pulse + 0.65,
            cx - 6.0,
            cy - CORE_D - 6.0,
            CORE_W + CORE_D + 12.0,
            CORE_H + CORE_D + 12.0
        ));
    }
    b.push_str(&pulses);
    let lanes = m.lanes() as f64;
    let height = m.floor()
        + if lanes > 0.0 {
            44.0 + 16.0 * lanes
        } else {
            40.0
        };
    // The pulse and the glow have no stroke outside their keyframes, so a
    // picture that does not move shows the arrows and blocks alone.
    let style = format!(
        ".cb{{animation:core-in .55s cubic-bezier(.2,.8,.2,1) both}}.cf{{animation:core-draw .5s ease-out both}}.ch,.cx{{animation:core-fade .5s ease-out both}}.cp{{animation:core-run .8s ease-in-out both}}.cg{{animation:core-glow .9s ease-out both}}@keyframes core-in{{from{{opacity:0;transform:translateY(16px)}}to{{opacity:1;transform:none}}}}@keyframes core-draw{{from{{stroke-dasharray:100 100;stroke-dashoffset:100}}to{{stroke-dasharray:100 100;stroke-dashoffset:0}}}}@keyframes core-fade{{from{{opacity:0}}to{{opacity:1}}}}@keyframes core-run{{from{{stroke:{a};stroke-dasharray:12 200;stroke-dashoffset:20}}to{{stroke:{a};stroke-dasharray:12 200;stroke-dashoffset:-110}}}}@keyframes core-glow{{0%,100%{{stroke:{a};stroke-opacity:0}}35%{{stroke:{a};stroke-opacity:1}}}}@media (prefers-reduced-motion:reduce){{.cb,.cf,.ch,.cx,.cp,.cg{{animation:none}}}}{}",
        RESULT_STYLE.replace("MUTED", t.muted),
        a = t.accent
    );
    Ok(card(
        (1200, height as u32),
        t,
        ("The memory core", &xml(&core_text(m))),
        &style,
        &b,
    ))
}
/// The parts of the memory core and the dependencies among them.
fn core_card(dark: bool) -> Result<String> {
    core_draw(dark, &CoreMap::read()?)
}
fn core_alt() -> Result<String> {
    Ok(core_text(&CoreMap::read()?))
}
// ===== VISUAL core END =====

// ===== VISUAL q8 BEGIN =====
/// The contract the Q8 card is drawn from.
const Q8_DOC: &str = "docs/Q8-QUAD.md";
/// What the contract says the record is, right after its size: the card says
/// "in memory", not "stored".
const Q8_IN_MEMORY: &str =
    "This is an in-memory representation, not a new persistence/serialization format.";
/// The title of both Q8 cards: what the record holds, as its contract states
/// the layout; how records are printed stays private.
const Q8_TITLE: &str = "What one Q8 record holds";
/// The kickers over the record and over the four views, on both cards.
const Q8_KICKS: [&str; 2] = [
    "WHAT ONE Q8 RECORD HOLDS",
    "FOUR VIEWS, ONE RECORD IN MEMORY",
];
/// The kicker that sets the commitment apart from the record.
const Q8_SEPARATE: &str = "SEPARATE: A PUBLISHED COMMITMENT";
/// Where the commitment's strip starts on the wide and on the phone card.
const Q8_STRIP_WIDE: f64 = 656.0;
const Q8_STRIP_PHONE: f64 = 1524.0;
/// The sizes of one Q8 record, as its contract states them.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Q8Record {
    /// One-byte values.
    values: usize,
    /// Bytes of the activity mask: one bit per value.
    mask: usize,
    /// The record in memory: the values and the mask.
    record: usize,
    /// Four written-out copies of the record, which are not held.
    copies: usize,
}
impl Q8Record {
    fn read() -> Result<Q8Record> {
        Q8Record::from_doc(&fs::read_to_string(Q8_DOC)?, RECORD_VALUES, RECORD_MASK)
    }
    /// The sizes the contract states, refused unless they add up and agree with
    /// the generator's own record constants, and unless the contract still says
    /// the limits the card draws.
    fn from_doc(doc: &str, values: usize, mask: usize) -> Result<Q8Record> {
        let text = doc.split_whitespace().collect::<Vec<_>>().join(" ");
        let after = |key: &str| -> Result<String> {
            Ok(text
                .split_once(key)
                .ok_or(format!("{Q8_DOC} no longer says {key:?}"))?
                .1
                .to_string())
        };
        // The number a phrase of the contract starts with.
        let lead = |s: &str| -> Result<usize> {
            let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            Ok(s[..end].parse::<usize>()?)
        };
        let held = lead(&after("A Record holds ")?)?;
        let bits = lead(&after(" in Z256 and ")?)?;
        let sum = after("explicit activity bits: ")?;
        let sum = sum
            .split_once(" bytes")
            .ok_or("the contract has no byte sum")?
            .0;
        let (parts, total) = sum
            .split_once(" = ")
            .ok_or("the contract has no byte sum")?;
        let (a, b) = parts
            .split_once(" + ")
            .ok_or("the contract has no byte sum")?;
        let (a, b, record) = (
            a.parse::<usize>()?,
            b.parse::<usize>()?,
            total.parse::<usize>()?,
        );
        let packed = after("Four packed canonical records would occupy ")?;
        let copies = lead(&packed)?;
        let versus = lead(
            packed
                .split_once(" bytes versus ")
                .ok_or("the contract does not compare four copies with one record")?
                .1,
        )?;
        if held != values || bits != values || a != values {
            return Err(format!(
                "{Q8_DOC} gives {held} values, {bits} activity bits and {a} value bytes; the card has {values}"
            )
            .into());
        }
        if b != mask || mask * 8 != values {
            return Err(format!(
                "{Q8_DOC} gives a {b}-byte mask; the card has {mask} bytes for {values} bits"
            )
            .into());
        }
        if a + b != record || versus != record || copies != 4 * record {
            return Err(format!(
                "{Q8_DOC} sizes do not add up: {a} + {b} = {record}, four copies {copies} versus {versus}"
            )
            .into());
        }
        let title = doc.lines().next().unwrap_or("");
        if !title.contains("not a knowledge encoder") {
            return Err(format!("{Q8_DOC} title no longer says not a knowledge encoder").into());
        }
        let pct = 100 - 100 * record / copies;
        let limit = format!("{pct}% less record material, not {pct}% less process RAM");
        if !text.contains(&limit) {
            return Err(format!("{Q8_DOC} no longer says {limit}").into());
        }
        // The card says the record is held in memory, once: the contract's own limit.
        let memory = format!("{record} bytes on the tested ABI. {Q8_IN_MEMORY}");
        if !text.contains(&memory) {
            return Err(format!("{Q8_DOC} no longer says {memory}").into());
        }
        Ok(Q8Record {
            values,
            mask,
            record,
            copies,
        })
    }
    /// How much less record material one record is than four copies.
    fn saved(&self) -> usize {
        100 - 100 * self.record / self.copies
    }
}
/// What the Q8 card says, as its description and as the README alternative text.
fn q8_text(r: &Q8Record) -> String {
    format!(
        "What a Q8 record holds: {} values of one byte each plus a {}-byte mask, one bit per value, make {} bytes in memory, held once. Four views read that same record, each exactly as its own full copy would; four copies would take {} bytes. Not four memories, votes or a speedup.",
        grouped(r.values),
        grouped(r.mask),
        grouped(r.record),
        grouped(r.copies)
    )
}
/// The entrance of the Q8 cards: everything starts from nothing in its keyframes
/// only and holds its final state; reduced motion stills every class.
fn q8_style(t: &Theme) -> String {
    format!(
        "{}.q8g{{transform-box:fill-box;transform-origin:left center}}.q8p{{transform-box:fill-box;transform-origin:center}}.q8u{{animation:q8u .6s ease-out both}}.q8g{{animation:q8g .35s ease-out both}}.q8p{{animation:q8p .5s cubic-bezier(.2,.8,.2,1) both}}.q8d{{animation:q8d .7s ease-out both}}@keyframes q8u{{from{{opacity:0;transform:translateY(10px)}}to{{opacity:1;transform:none}}}}@keyframes q8g{{from{{opacity:0;transform:scaleX(0)}}to{{opacity:1;transform:none}}}}@keyframes q8p{{from{{opacity:0;transform:scale(.6)}}to{{opacity:1;transform:none}}}}@keyframes q8d{{from{{stroke-dasharray:100 100;stroke-dashoffset:100}}to{{stroke-dasharray:100 100;stroke-dashoffset:0}}}}@media (prefers-reduced-motion:reduce){{.q8u,.q8g,.q8p,.q8d{{animation:none}}}}",
        RESULT_STYLE.replace("MUTED", t.muted)
    )
}
/// `inner` entering with the animation `class` after `delay` seconds.
fn q8_in(class: &str, delay: f64, inner: &str) -> String {
    format!(r#"<g class="{class}" style="animation-delay:{delay:.2}s">{inner}</g>"#)
}
/// One line of text; `attrs` adds a class, an anchor or a fill.
fn q8_line(x: f64, y: f64, size: u32, attrs: &str, text: &str) -> String {
    format!(
        r#"<text x="{x:.0}" y="{y:.0}" font-size="{size}"{attrs}>{}</text>"#,
        xml(text)
    )
}
/// The stored record drawn to scale, one cell per byte: rows of values, then
/// rows of mask bytes, inside a box of at most `w` × `h` at (`x`, `y`). The rows
/// grow in turn from `start` seconds, `step` apart. Gives the drawing and the
/// drawn width and height.
fn q8_block(
    t: &Theme,
    r: &Q8Record,
    (x, y): Point,
    (w, h): Point,
    (start, step): Point,
) -> (String, f64, f64) {
    let cols = (1..).find(|c| c * c >= r.values).unwrap_or(1);
    let rows = [r.values.div_ceil(cols), r.mask.div_ceil(cols)];
    let pitch = (w / cols as f64).min(h / (rows[0] + rows[1]) as f64);
    let cell = pitch * 0.86;
    let mut b = String::from("<defs>");
    for (id, fill) in [("q8v", t.blue[1]), ("q8m", t.amber[1])] {
        b.push_str(&format!(
            r#"<pattern id="{id}" x="{x:.2}" y="{y:.2}" width="{pitch:.4}" height="{pitch:.4}" patternUnits="userSpaceOnUse"><rect width="{cell:.3}" height="{cell:.3}" rx="{:.2}" fill="{fill}"/></pattern>"#,
            cell * 0.18
        ));
    }
    b.push_str("</defs>");
    let mut row = 0;
    for (id, bytes, n) in [("q8v", r.values, rows[0]), ("q8m", r.mask, rows[1])] {
        for k in 0..n {
            let cells = (bytes - k * cols).min(cols);
            b.push_str(&q8_in(
                "q8g",
                start + step * row as f64,
                &format!(
                    r#"<rect x="{x:.2}" y="{:.2}" width="{:.4}" height="{pitch:.4}" fill="url(#{id})"/>"#,
                    y + pitch * row as f64,
                    pitch * cells as f64
                ),
            ));
            row += 1;
        }
    }
    let (bw, bh) = (pitch * cols as f64, pitch * row as f64);
    b.push_str(&format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="5" fill="none" stroke="{}" stroke-width="2"/>"#,
        x - 4.0,
        y - 4.0,
        bw - (pitch - cell) + 8.0,
        bh - (pitch - cell) + 8.0,
        t.ink
    ));
    (b, bw, bh)
}
/// The same record as one small solid block, values over mask, to scale by bytes.
fn q8_mini(t: &Theme, r: &Q8Record, (x, y): Point, (w, h): Point) -> String {
    let split = h * r.values as f64 / r.record as f64;
    format!(
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{split:.1}" fill="{}"/><rect x="{x:.1}" y="{:.1}" width="{w:.1}" height="{:.1}" fill="{}"/><rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="none" stroke="{}" stroke-width="2"/>"#,
        t.blue[1],
        y + split,
        h - split,
        t.amber[1],
        t.ink
    )
}
/// A round lens: the picture of a view, which shows nothing of how it reads.
fn q8_lens(t: &Theme, (cx, cy): Point, radius: f64) -> String {
    format!(
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{radius:.1}" fill="{}" fill-opacity=".18" stroke="{}" stroke-width="3"/><path d="M{:.1} {:.1}l{:.1} {:.1}" stroke="{}" stroke-width="4" stroke-linecap="round"/>"#,
        t.blue[0],
        t.accent,
        cx + radius * 0.72,
        cy + radius * 0.72,
        radius * 0.55,
        radius * 0.55,
        t.accent
    )
}
/// The width a line of `chars` characters at `size` px is allowed: 0.62 em per
/// character and 8 % slack.
fn q8_width(chars: usize, size: f64) -> f64 {
    chars as f64 * 0.62 * size * 1.08
}
/// The wide card: values become bytes and fill one record to scale; four lenses
/// read that same record, each bit for bit; four copies it does not need; what
/// it is not. One entrance of about seven seconds that holds its final state.
fn q8_draw(dark: bool, r: &Q8Record, c: &Q8Pledge) -> String {
    let t = theme(dark);
    let record = grouped(r.record);
    let mut b = q8_in(
        "q8u",
        0.0,
        &format!(
            r#"<text class="kick" x="40" y="48">{}</text><text class="kick" x="640" y="48">{}</text>"#,
            Q8_KICKS[0], Q8_KICKS[1]
        ),
    );
    // 1. Values come in and each becomes one byte.
    b += &q8_in(
        "q8u",
        0.1,
        &q8_line(
            40.0,
            84.0,
            17,
            r#" class="b""#,
            &format!("{} values, one byte each", grouped(r.values)),
        ),
    );
    for i in 0..14 {
        b += &q8_in(
            "q8p",
            0.25 + 0.07 * f64::from(i),
            &format!(
                r#"<rect x="{}" y="100" width="22" height="22" rx="5" fill="{}" stroke="{}" stroke-width="1.5"/>"#,
                40 + 28 * i,
                t.blue[0],
                t.blue[2]
            ),
        );
    }
    let (grid, gw, gh) = q8_block(t, r, (40.0, 150.0), (256.0, 288.0), (1.3, 0.045));
    b += &q8_in(
        "q8u",
        1.1,
        &format!(
            r#"<path d="M{0:.1} 128v12m-7 -7l7 7l7 -7" fill="none" stroke="{1}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>"#,
            40.0 + gw / 2.0,
            t.accent
        ),
    );
    b += &grid;
    // 2. What the block holds.
    let (lx, sw) = (40.0 + gw + 28.0, 14.0);
    b += &q8_in(
        "q8u",
        2.4,
        &(q8_line(
            lx,
            200.0,
            52,
            &format!(r#" class="b" style="fill:{}""#, t.accent),
            &format!("{record} B"),
        ) + &q8_line(lx, 228.0, 16, "", "in memory, held once")),
    );
    for (k, (fill, bytes, what, note)) in [
        (t.blue[1], r.values, "values", "one cell per byte"),
        (t.amber[1], r.mask, "activity mask", "one bit per value"),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 262.0 + 60.0 * k as f64;
        b += &q8_in(
            "q8u",
            2.6 + 0.2 * k as f64,
            &format!(
                r#"<rect x="{lx:.0}" y="{y:.0}" width="{sw}" height="{sw}" rx="3" fill="{fill}"/>{}{}"#,
                q8_line(
                    lx + 22.0,
                    y + 13.0,
                    16,
                    r#" class="b""#,
                    &format!("{} B {what}", grouped(bytes))
                ),
                q8_line(lx + 22.0, y + 34.0, 14, r#" class="mut""#, note)
            ),
        );
    }
    b += &q8_in(
        "q8u",
        3.0,
        &q8_line(
            40.0,
            150.0 + gh + 38.0,
            17,
            r#" class="b""#,
            &format!(
                "{} B + {} B = {record} B",
                grouped(r.values),
                grouped(r.mask)
            ),
        ),
    );
    // 3. Four lenses over the same record, each bit for bit.
    let (mx, my, mw, mh) = (868.0, 214.0, 64.0, 72.0);
    b += &q8_in("q8u", 3.6, &q8_mini(t, r, (mx, my), (mw, mh)));
    b += &q8_in(
        "q8u",
        3.8,
        &(q8_line(
            mx + mw + 16.0,
            my + 30.0,
            16,
            r#" class="b""#,
            &format!("the same {record} B"),
        ) + &q8_line(
            mx + mw + 16.0,
            my + 50.0,
            13,
            r#" class="mut""#,
            "read by all four views",
        )),
    );
    for i in 0..4 {
        let x = 640.0 + 400.0 * f64::from(i) / 3.0;
        let cx = x + 60.0;
        b += &q8_in(
            "q8p",
            3.3 + 0.15 * f64::from(i),
            &format!(
                r#"<rect x="{x:.1}" y="72" width="120" height="108" rx="14" fill="none" stroke="{}" stroke-width="1.5"/>{}{}"#,
                t.line,
                q8_lens(t, (cx, 102.0), 15.0),
                q8_line(
                    cx,
                    148.0,
                    16,
                    r#" class="b" text-anchor="middle""#,
                    &format!("view {}", i + 1)
                )
            ),
        );
        b += &q8_in(
            "q8d",
            3.9 + 0.1 * f64::from(i),
            &format!(
                r#"<path d="M{cx:.1} 182L{:.1} {my}" pathLength="100" fill="none" stroke="{}" stroke-width="2"/>"#,
                mx + mw * (f64::from(i) + 0.5) / 4.0,
                t.accent
            ),
        );
        b += &q8_in(
            "q8p",
            4.5 + 0.15 * f64::from(i),
            &q8_line(
                cx,
                170.0,
                13,
                &format!(r#" class="b" text-anchor="middle" style="fill:{}""#, t.good),
                "✓ bit for bit",
            ),
        );
    }
    b += &q8_in(
        "q8u",
        5.0,
        &(q8_line(
            640.0,
            318.0,
            15,
            "",
            "Each view reads exactly as a separate, fully",
        ) + &q8_line(640.0, 338.0, 15, "", "written-out copy would, bit for bit.")),
    );
    // 4. Four such copies are not held: the sizes to one scale.
    let unit = 520.0 / r.copies as f64;
    let stored = unit * r.record as f64;
    b += &q8_in(
        "q8u",
        5.3,
        &(q8_line(640.0, 378.0, 14, r#" class="mut""#, "held once")
            + &q8_line(
                640.0 + stored + 12.0,
                407.0,
                15,
                r#" class="b""#,
                &format!("{record} B"),
            )),
    );
    b += &q8_in(
        "q8g",
        5.3,
        &format!(
            r#"<rect class="q8size" x="640" y="388" width="{stored:.2}" height="26" rx="5" fill="{}"/>"#,
            t.accent
        ),
    );
    b += &q8_in(
        "q8u",
        5.6,
        &(q8_line(
            640.0,
            446.0,
            14,
            r#" class="mut""#,
            "four written-out copies would take",
        ) + &q8_line(
            1160.0,
            446.0,
            15,
            r#" class="b" text-anchor="end""#,
            &format!("{} B", grouped(r.copies)),
        )),
    );
    for i in 0..4 {
        b += &q8_in(
            "q8p",
            5.7 + 0.12 * f64::from(i),
            &format!(
                r#"<rect class="q8size" x="{:.2}" y="456" width="{:.2}" height="26" rx="5" fill="none" stroke="{}" stroke-width="2" stroke-dasharray="6 5"/>"#,
                640.0 + stored * f64::from(i) + 2.0,
                stored - 4.0,
                t.muted
            ),
        );
    }
    b += &q8_in(
        "q8u",
        6.2,
        &q8_line(
            640.0,
            510.0,
            14,
            r#" class="mut""#,
            &format!(
                "{0}% less record material, not {0}% less process RAM",
                r.saved()
            ),
        ),
    );
    // 5. What it is not.
    b += &q8_in(
        "q8u",
        6.4,
        r#"<text class="kick" x="40" y="550">WHAT IT IS NOT</text>"#,
    );
    let mut x = 40.0;
    for (i, not) in [
        "not four independent memories",
        "not four votes",
        "not a speedup",
        "not a knowledge encoder",
    ]
    .into_iter()
    .enumerate()
    {
        let w = q8_width(not.chars().count(), 15.0) + 28.0;
        b += &q8_in(
            "q8p",
            6.5 + 0.12 * i as f64,
            &format!(
                r#"<rect x="{x:.1}" y="566" width="{w:.1}" height="38" rx="19" fill="none" stroke="{}" stroke-width="1.5"/>{}"#,
                t.line,
                q8_line(x + w / 2.0, 590.0, 15, r#" text-anchor="middle""#, not)
            ),
        );
        x += w + 16.0;
    }
    // 6. Apart from the record: the target quantizer is private, and its
    // published commitment fixes its files.
    b += &q8_in(
        "q8u",
        6.9,
        &format!(r#"<text class="kick" x="40" y="642">{Q8_SEPARATE}</text>"#),
    );
    b += &q8_pledge_strip(t, c, Q8_STRIP_WIDE, 7.0);
    card(
        (1200, 768),
        t,
        (Q8_TITLE, &xml(&q8_text(r))),
        &q8_style(t),
        &b,
    )
}
/// The phone card: the same story stacked for a narrow screen, with the same
/// numbers and limits, and text of at least 20 px.
fn q8_phone_draw(dark: bool, r: &Q8Record, c: &Q8Pledge) -> String {
    let t = theme(dark);
    let record = grouped(r.record);
    let mut b = q8_in(
        "q8u",
        0.0,
        &format!(r#"<text class="kick" x="40" y="56">{}</text>"#, Q8_KICKS[0]),
    );
    b += &q8_in(
        "q8u",
        0.1,
        &(q8_line(
            40.0,
            114.0,
            44,
            r#" class="b""#,
            &format!("{} values", grouped(r.values)),
        ) + &q8_line(40.0, 148.0, 22, r#" class="mut""#, "one byte each")),
    );
    for i in 0..14 {
        b += &q8_in(
            "q8p",
            0.3 + 0.07 * f64::from(i),
            &format!(
                r#"<rect x="{}" y="170" width="28" height="28" rx="6" fill="{}" stroke="{}" stroke-width="2"/>"#,
                40 + 36 * i,
                t.blue[0],
                t.blue[2]
            ),
        );
    }
    let (grid, gw, gh) = q8_block(t, r, (40.0, 244.0), (256.0, 288.0), (1.3, 0.045));
    b += &q8_in(
        "q8u",
        1.1,
        &format!(
            r#"<path d="M{0:.1} 210v16m-8 -8l8 8l8 -8" fill="none" stroke="{1}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>"#,
            40.0 + gw / 2.0,
            t.accent
        ),
    );
    b += &grid;
    let lx = 40.0 + gw + 28.0;
    b += &q8_in(
        "q8u",
        2.4,
        &(q8_line(
            lx,
            290.0,
            48,
            &format!(r#" class="b" style="fill:{}""#, t.accent),
            &format!("{record} B"),
        ) + &q8_line(lx, 324.0, 22, "", "in memory,")
            + &q8_line(lx, 350.0, 22, "", "held once")),
    );
    for (k, (fill, bytes, lines)) in [
        (t.blue[1], r.values, &["values"][..]),
        (t.amber[1], r.mask, &["mask, 1 bit", "per value"][..]),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 384.0 + 82.0 * k as f64;
        let mut notes = String::new();
        for (j, line) in lines.iter().enumerate() {
            notes += &q8_line(
                lx + 32.0,
                y + 46.0 + 24.0 * j as f64,
                20,
                r#" class="mut""#,
                line,
            );
        }
        b += &q8_in(
            "q8u",
            2.6 + 0.2 * k as f64,
            &format!(
                r#"<rect x="{lx:.0}" y="{y:.0}" width="22" height="22" rx="4" fill="{fill}"/>{}{notes}"#,
                q8_line(
                    lx + 32.0,
                    y + 20.0,
                    24,
                    r#" class="b""#,
                    &format!("{} B", grouped(bytes))
                )
            ),
        );
    }
    b += &q8_in(
        "q8u",
        3.0,
        &q8_line(
            40.0,
            244.0 + gh + 52.0,
            26,
            r#" class="b""#,
            &format!(
                "{} B + {} B = {record} B",
                grouped(r.values),
                grouped(r.mask)
            ),
        ),
    );
    // Four lenses over the same record.
    b += &q8_in(
        "q8u",
        3.3,
        &format!(
            r#"<text class="kick" x="40" y="664">{}</text>"#,
            Q8_KICKS[1]
        ),
    );
    let (mx, my, mw, mh) = (268.0, 840.0, 64.0, 72.0);
    for i in 0..4 {
        let cx = 105.0 + 130.0 * f64::from(i);
        b += &q8_in(
            "q8p",
            3.4 + 0.15 * f64::from(i),
            &(q8_line(
                cx,
                708.0,
                20,
                r#" class="mut" text-anchor="middle""#,
                &format!("view {}", i + 1),
            ) + &q8_lens(t, (cx, 756.0), 30.0)),
        );
        b += &q8_in(
            "q8d",
            4.0 + 0.1 * f64::from(i),
            &format!(
                r#"<path d="M{cx:.1} 800L{:.1} {my}" pathLength="100" fill="none" stroke="{}" stroke-width="3"/>"#,
                mx + mw * (f64::from(i) + 0.5) / 4.0,
                t.accent
            ),
        );
        b += &q8_in(
            "q8p",
            4.6 + 0.15 * f64::from(i),
            &q8_line(
                cx,
                767.0,
                30,
                &format!(r#" class="b" text-anchor="middle" style="fill:{}""#, t.good),
                "✓",
            ),
        );
    }
    b += &q8_in("q8u", 3.8, &q8_mini(t, r, (mx, my), (mw, mh)));
    b += &q8_in(
        "q8u",
        5.0,
        &(q8_line(
            300.0,
            958.0,
            26,
            r#" class="b" text-anchor="middle""#,
            &format!("the same {record} B"),
        ) + &q8_line(
            300.0,
            992.0,
            22,
            r#" text-anchor="middle""#,
            "each view: bit for bit",
        ) + &q8_line(
            300.0,
            1020.0,
            20,
            r#" class="mut" text-anchor="middle""#,
            "as a separate full copy would",
        )),
    );
    // Four copies are not held: the sizes to one scale.
    let unit = 520.0 / r.copies as f64;
    let stored = unit * r.record as f64;
    b += &q8_in(
        "q8u",
        5.3,
        &(q8_line(40.0, 1078.0, 22, r#" class="mut""#, "held once")
            + &q8_line(
                40.0 + stored + 16.0,
                1112.0,
                28,
                r#" class="b""#,
                &format!("{record} B"),
            )),
    );
    b += &q8_in(
        "q8g",
        5.3,
        &format!(
            r#"<rect class="q8size" x="40" y="1090" width="{stored:.2}" height="34" rx="6" fill="{}"/>"#,
            t.accent
        ),
    );
    b += &q8_in(
        "q8u",
        5.6,
        &(q8_line(40.0, 1166.0, 22, r#" class="mut""#, "4 copies would take")
            + &q8_line(
                560.0,
                1166.0,
                28,
                r#" class="b" text-anchor="end""#,
                &format!("{} B", grouped(r.copies)),
            )),
    );
    for i in 0..4 {
        b += &q8_in(
            "q8p",
            5.7 + 0.12 * f64::from(i),
            &format!(
                r#"<rect class="q8size" x="{:.2}" y="1178" width="{:.2}" height="34" rx="6" fill="none" stroke="{}" stroke-width="2.5" stroke-dasharray="7 6"/>"#,
                40.0 + stored * f64::from(i) + 2.0,
                stored - 4.0,
                t.muted
            ),
        );
    }
    b += &q8_in(
        "q8u",
        6.2,
        &(q8_line(
            40.0,
            1252.0,
            20,
            r#" class="mut""#,
            &format!("{}% less record material,", r.saved()),
        ) + &q8_line(
            40.0,
            1278.0,
            20,
            r#" class="mut""#,
            &format!("not {}% less process RAM", r.saved()),
        )),
    );
    b += &q8_in(
        "q8u",
        6.4,
        r#"<text class="kick" x="40" y="1338">WHAT IT IS NOT</text>"#,
    );
    for (i, not) in [
        "not 4 independent memories",
        "not 4 votes · not a speedup",
        "not a knowledge encoder",
    ]
    .into_iter()
    .enumerate()
    {
        let y = 1380.0 + 38.0 * i as f64;
        b += &q8_in(
            "q8u",
            6.5 + 0.12 * i as f64,
            &format!(
                r#"<rect x="40" y="{:.0}" width="16" height="4" rx="2" fill="{}"/>{}"#,
                y - 9.0,
                t.red[1],
                q8_line(70.0, y, 22, "", not)
            ),
        );
    }
    // Apart from the record: the commitment of the target quantizer.
    b += &q8_in(
        "q8u",
        6.9,
        &format!(r#"<text class="kick" x="40" y="1504">{Q8_SEPARATE}</text>"#),
    );
    let (strip, end) = q8_pledge_phone(t, c, Q8_STRIP_PHONE, 7.0);
    b += &strip;
    phone_card(
        dark,
        (end + 32.0) as u32,
        (Q8_TITLE, &xml(&q8_text(r))),
        &q8_style(t),
        &b,
    )
}
/// The label of the published commitment of the target quantizer.
const Q8_TARGET: &str = "Target Q8 Quad quantizer";
/// What the commitments table states about the target quantizer: only its
/// label, date, counts and hash.
#[derive(Clone, Debug, PartialEq)]
struct Q8Pledge {
    label: String,
    date: String,
    /// Both counts under the names of their columns.
    counts: [String; 2],
    hash: String,
}
impl Q8Pledge {
    fn read() -> Result<Q8Pledge> {
        Q8Pledge::from_table(&Pledges::read()?)
    }
    /// The one row labelled Q8_TARGET; refused when there is none or more than one.
    fn from_table(p: &Pledges) -> Result<Q8Pledge> {
        let rows: Vec<&PledgesRow> = p.rows.iter().filter(|r| r.label == Q8_TARGET).collect();
        let [row] = rows[..] else {
            return Err(format!(
                "{PLEDGES_FILE} has {} commitment rows labelled {Q8_TARGET}, not one",
                rows.len()
            )
            .into());
        };
        Ok(Q8Pledge {
            label: row.label.clone(),
            date: row.date.clone(),
            counts: pledges_counts(p, row),
            hash: row.hash.clone(),
        })
    }
    /// The first 8 hex characters, as the commitments card shows a hash.
    fn short(&self) -> &str {
        &self.hash[..8]
    }
}
/// What every commitment shows and what it does not, in the commitments card's words.
const Q8_PLEDGE_LIMIT: &str = "A hash shows the files did not change, not that they are correct.";
/// The wide card's strip of the commitment: label, date, counts, hash, limit.
fn q8_pledge_strip(t: &Theme, c: &Q8Pledge, y: f64, delay: f64) -> String {
    let hash = format!(
        r#"<tspan class="mono b" style="fill:{}">{}</tspan>"#,
        t.accent,
        xml(c.short())
    );
    q8_in(
        "q8u",
        delay,
        &format!(
            r#"<rect x="40" y="{y:.0}" width="1120" height="94" rx="14" fill="{acc}" fill-opacity=".07" stroke="{line}"/>{lock}{}<text x="100" y="{:.0}" font-size="15">{} · SHA-256 {hash}</text>{}"#,
            q8_line(
                100.0,
                y + 30.0,
                17,
                r#" class="b""#,
                &format!("{}: private", c.label)
            ),
            y + 56.0,
            xml(&format!(
                "fixed {} · {}, {}",
                c.date, c.counts[0], c.counts[1]
            )),
            q8_line(100.0, y + 80.0, 14, r#" class="mut""#, Q8_PLEDGE_LIMIT),
            acc = t.accent,
            line = t.line,
            lock = q8_lock(t, (60.0, y + 22.0)),
        ),
    )
}
/// A closed lock, as the commitments card draws one.
fn q8_lock(t: &Theme, (x, y): Point) -> String {
    format!(
        r#"<path d="M{:.1} {:.1}v-5a6 6 0 0 1 12 0v5" fill="none" stroke="{acc}" stroke-width="2.6"/><rect x="{x:.1}" y="{:.1}" width="20" height="15" rx="3" fill="{acc}"/>"#,
        x + 4.0,
        y + 7.0,
        y + 7.0,
        acc = t.accent
    )
}
/// The phone card's strip of the commitment, from `y`; gives its drawing and
/// where it ends.
fn q8_pledge_phone(t: &Theme, c: &Q8Pledge, y: f64, delay: f64) -> (String, f64) {
    let (x, step) = (96.0, 30.0);
    let mut text = String::new();
    let mut base = y + 40.0;
    for line in wrap(&c.label, 26) {
        text += &q8_line(x, base, 22, r#" class="b""#, &line);
        base += step;
    }
    text += &q8_line(x, base, 20, "", &format!("private · fixed {}", c.date));
    base += step;
    text += &q8_line(
        x,
        base,
        20,
        r#" class="mut""#,
        &format!("{}, {}", c.counts[0], c.counts[1]),
    );
    base += step + 2.0;
    text += &format!(
        r#"<text x="{x:.0}" y="{base:.0}" font-size="20">SHA-256 <tspan class="mono b" font-size="24" style="fill:{}">{}</tspan></text>"#,
        t.accent,
        xml(c.short())
    );
    base += step + 4.0;
    for line in wrap(Q8_PLEDGE_LIMIT, 28) {
        text += &q8_line(x, base, 20, r#" class="mut""#, &line);
        base += step - 4.0;
    }
    let end = base + 4.0;
    (
        q8_in(
            "q8u",
            delay,
            &format!(
                r#"<rect x="40" y="{y:.0}" width="520" height="{:.0}" rx="16" fill="{acc}" fill-opacity=".07" stroke="{}"/>{}{text}"#,
                end - y,
                t.line,
                q8_lock(t, (56.0, y + 22.0)),
                acc = t.accent
            ),
        ),
        end,
    )
}
/// What one Q8 record holds, drawn from its contract, and apart from it the
/// published commitment of the target quantizer.
fn q8_card(dark: bool) -> Result<String> {
    Ok(q8_draw(dark, &Q8Record::read()?, &Q8Pledge::read()?))
}
/// Refused, like the card, while the commitment of the target quantizer is missing.
fn q8_alt() -> Result<String> {
    Q8Pledge::read()?;
    Ok(q8_text(&Q8Record::read()?))
}
fn q8_phone(dark: bool) -> Result<String> {
    Ok(q8_phone_draw(dark, &Q8Record::read()?, &Q8Pledge::read()?))
}
// ===== VISUAL q8 END =====

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
        out.push((
            format!("media/presentation/license-{theme}.svg"),
            license_card(dark)?,
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
    for (name, draw, _) in VISUALS {
        for dark in [false, true] {
            out.push((visual_path(name, dark), draw(dark)?));
        }
    }
    // A phone card that cannot be drawn is left out, and so are its sources.
    for (name, draw) in PHONES {
        if let (Ok(light), Ok(dark)) = (draw(false), draw(true)) {
            out.push((phone_path(name, false), light));
            out.push((phone_path(name, true), dark));
        }
    }
    out.extend(badge_assets()?);
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
        // The picture and its title say it; the longer description stays on the full page.
        for (c, (id, title, _)) in row.iter().enumerate() {
            grid.push_str(&format!("<td width=\"50%\" valign=\"top\">{}<h4>{title}</h4><a href=\"media/gifs/{id}-light.gif\">{}</a><p><a href=\"media/gifs/{id}.txt\">Transcript</a> · <a href=\"media/gifs/{id}-dark.gif\">Dark full size</a></p></td>\n",chip_picture(r*2+c),picture(id,title,false)));
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
    let unicode = unicode_version().unwrap_or_else(|e| format!("__UNICODE__ {e}"));
    let crash = CrashSeries::read()
        .map(|c| crash_picture(&c))
        .unwrap_or_else(|e| format!("__CRASH__ {e}"));
    let speed = SpeedSet::read()
        .map(|s| speed_picture(&s))
        .unwrap_or_else(|e| format!("__SPEED__ {e}"));
    let license = license_card(false)
        .map(|_| license_picture())
        .unwrap_or_else(|e| format!("__LICENSE__ {e}"));
    let badges = match badges_row() {
        Ok(b) if b.is_empty() => b,
        Ok(b) => format!("\n\n{b}"),
        Err(e) => format!("\n\n__BADGES__ {e}"),
    };
    NATIVE
        .replace("__HEADER__", &header_picture())
        .replace("\n__BADGES__", &badges)
        .replace("__RECORD__", &record)
        .replace("__COMMITMENTS__", &commitments)
        .replace("__PACKAGES__", &packages)
        .replace("__UNICODE__", &unicode)
        .replace("__FLOW__", &flow_picture())
        .replace("__BARS__", &bars_picture())
        .replace("__WALL__", &wall_picture())
        .replace("__LOGO__", &readme_logo())
        .replace("__CRASH__", &crash)
        .replace("__SPEED__", &speed)
        .replace("__LICENSE__", &license)
        .replace("__HERO__", &picture(hero_id, hero_title, false))
        .replace("__HERO_ID__", hero_id)
        .replace("__GRID__", &grid)
        .replace("__STATUS__", &visual_picture("status"))
        .replace("__TERMINAL__", &visual_picture("terminal"))
        .replace("__RERUN__", &visual_picture("rerun"))
        .replace("__GLANCE__", &visual_picture("glance"))
        .replace("__PLEDGES__", &visual_picture("pledges"))
        .replace("__CORE__", &visual_picture("core"))
        .replace("__Q8__", &visual_picture("q8"))
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
    lock_packages(&fs::read_to_string("Cargo.lock")?, &|dir| {
        Path::new(dir).join("Cargo.toml").is_file()
    })
}
/// The packages of a Cargo.lock, refused when one comes from outside the
/// checkout, has no plain name or is not a crate directory of it (`has_crate`).
fn lock_packages(lock: &str, has_crate: &dyn Fn(&str) -> bool) -> Result<usize> {
    if lock.lines().any(|l| l.starts_with("source = ")) {
        return Err("Cargo.lock lists a package from outside this checkout".into());
    }
    let lines: Vec<&str> = lock.lines().collect();
    let mut packages = 0;
    for (i, l) in lines.iter().enumerate() {
        if *l != "[[package]]" {
            continue;
        }
        let name = lines
            .get(i + 1)
            .and_then(|n| n.strip_prefix("name = \"")?.strip_suffix('"'))
            .filter(|n| {
                !n.is_empty()
                    && n.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            })
            .ok_or("Cargo.lock: a package without a plain name")?;
        let dir = if name == "xtask" {
            name.to_string()
        } else {
            format!("crates/{name}")
        };
        if !has_crate(&dir) {
            return Err(format!("Cargo.lock: {name} is not a crate of this checkout").into());
        }
        packages += 1;
    }
    if packages == 0 {
        return Err("Cargo.lock lists no package".into());
    }
    Ok(packages)
}
/// The generated Unicode tables: their header and their version constant name the Unicode version.
const UNICODE_TABLES: &str = "crates/gel-source/src/nfc_tables.rs";
/// The release whose roadmap section states what the README folds under the
/// speed cards: no third-party package, one Unicode version, SHA-256 written here.
const UNICODE_RELEASE: &str = "v0.7.0";
/// The Unicode version of the normalization tables, the same in their header
/// and in their version constant.
fn unicode_of(tables: &str) -> Result<String> {
    let header = tables
        .lines()
        .find_map(|l| {
            l.split_once("Character Database ")?
                .1
                .strip_suffix("; do not edit. Inputs (SHA-256):")
        })
        .ok_or(format!(
            "{UNICODE_TABLES}: no Unicode version in the header"
        ))?;
    let constant = tables
        .lines()
        .find_map(|l| l.strip_prefix("pub(crate) const UNICODE_VERSION: (u8, u8, u8) = ("))
        .and_then(|l| l.strip_suffix(");"))
        .ok_or(format!("{UNICODE_TABLES}: no version constant"))?;
    let constant: Vec<&str> = constant.split(", ").collect();
    let unicode = constant.join(".");
    let numeric = |p: &&str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    if constant.len() != 3 || !constant.iter().all(numeric) || header != unicode {
        return Err(
            format!("{UNICODE_TABLES}: the header says {header}, the constant {unicode}").into(),
        );
    }
    Ok(unicode)
}
/// Refuses unless the roadmap section of `UNICODE_RELEASE` states the three
/// facts the README gives under that release; one stated only in another
/// section does not count.
fn unicode_release_check(roadmap: &str, unicode: &str) -> Result<()> {
    let section = roadmap
        .split_once(&format!("\n## {UNICODE_RELEASE}\n"))
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(""))
        .unwrap_or("");
    for fact in [
        "lists no third-party package".to_string(),
        format!("follows Unicode {unicode}"),
        "SHA-256".to_string(),
    ] {
        if !section.contains(&fact) {
            return Err(format!("{GLANCE_ROADMAP}: {UNICODE_RELEASE} does not say {fact}").into());
        }
    }
    Ok(())
}
/// The Unicode version the README states, read from the tables; the README
/// links the roadmap section of the release that brought it, which says so.
fn unicode_version() -> Result<String> {
    let unicode = unicode_of(&fs::read_to_string(UNICODE_TABLES)?)?;
    unicode_release_check(&fs::read_to_string(GLANCE_ROADMAP)?, &unicode)?;
    let link = format!(
        "([{UNICODE_RELEASE} in the roadmap](docs/ROADMAP.md#{}))",
        UNICODE_RELEASE.replace('.', "")
    );
    if NATIVE.matches(&link).count() != 1 {
        return Err(format!("the README does not link {link} once").into());
    }
    Ok(unicode)
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
        // Every differing file is named, so a change in a source shows which picture it moves.
        let mut differ = Vec::new();
        if !before.starts_with(&native_intro()) {
            differ.push("README.md".to_string());
        }
        if fs::read_to_string("docs/README-PRESENTATION.md")?
            != GUIDE.replace("__RELEASE__", RELEASE)
        {
            differ.push("docs/README-PRESENTATION.md".to_string());
        }
        if fs::read_to_string("README-MULTIMEDIA.html")? != page() {
            differ.push("README-MULTIMEDIA.html".to_string());
        }
        for (name, body) in assets()? {
            if !fs::read_to_string(&name).is_ok_and(|found| found == body) {
                differ.push(name);
            }
        }
        if !differ.is_empty() {
            return Err(format!(
                "existing presentation differs ({}); refuse to overwrite it",
                differ.join(", ")
            )
            .into());
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
        assert_eq!(
            all.len(),
            24 + 2 * CHIPS.len()
                + 2 * VISUALS.len()
                + 2 * PHONES.len()
                + badge_assets().unwrap().len()
        );
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
                "__LICENSE__",
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
        // Each image is one line; folded details add lines a reader does not see,
        // so the words in view are limited separately.
        assert!(s.lines().count() <= 270, "{} lines", s.lines().count());
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
                // The entrance: the bar grows, four views sweep it and land, the
                // sizes grow and the verdict fades in, all on one clock, once.
                for name in [
                    "g0", "s0", "s1", "s2", "s3", "c0", "c1", "c2", "c3", "z0", "z1", "z2", "vd",
                ] {
                    assert!(
                        c.contains(&format!(".{name}{{animation:{name} {RECORD_ENTRANCE}s "))
                            && c.contains(&format!("@keyframes {name}{{")),
                        "{name}"
                    );
                    // ... and an element carries it.
                    assert!(c.contains(&format!(" {name}\">")), "{name}");
                }
                assert!(!c.contains("animation-delay") && !c.contains("infinite"));
                // What has appeared stays: the card never goes blank.
                assert_eq!(record_blanks(&c), Vec::<String>::new());
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
        let mut texts = vec![
            native_intro(),
            record_card(false, &r),
            record_card(true, &r),
        ];
        for (_, draw, _) in VISUALS {
            texts.push(draw(false).unwrap());
            texts.push(draw(true).unwrap());
        }
        for (_, draw) in PHONES {
            texts.extend(draw(false).ok());
            texts.extend(draw(true).ok());
        }
        texts.extend(badge_assets().unwrap().into_iter().map(|a| a.1));
        texts.push(badges_row().unwrap());
        for text in texts {
            let lower = text.to_lowercase();
            for word in ["reversal", "additive offset", "modulo", "seed"] {
                assert!(!lower.contains(word), "{word}");
            }
        }
    }
    #[test]
    fn license_card_quotes_the_license_word_for_word() {
        let license = fs::read_to_string(LICENSE_FILE).unwrap();
        let folded = license.split_whitespace().collect::<Vec<_>>().join(" ");
        let quotes = license_quotes(&license).unwrap();
        assert_eq!(quotes.len(), 7);
        for dark in [false, true] {
            let s = license_card(dark).unwrap();
            // Every quote is in LICENSE and drawn whole, line by line.
            for q in &quotes {
                assert!(folded.contains(q), "{q}");
                for line in wrap(q, if *q == LICENSE_NOTICE_QUOTE { 120 } else { 44 }) {
                    assert!(s.contains(&line), "{line}");
                }
            }
            for want in [
                "GEL RAM Noncommercial Reciprocal License 1.0",
                "not an OSI-approved open-source license",
                "gelram.licensing@gmail.com",
                "A summary adds no rights; LICENSE controls (section 17).",
                "@media (prefers-reduced-motion:reduce){.col{animation:none}}",
            ] {
                assert!(s.contains(want), "{want}");
            }
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—".contains(ch)));
        }
        // The alternative text is an attribute value: no double quote may end it early.
        assert!(!license_text().contains('"'));
        // The README paragraph quotes the two definitions word for word.
        let readme = native_intro()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for quoted in [
            "directly or indirectly receives, earns, enables, supports, protects, or is intended to produce money or other material financial benefit for You or another person",
            "solely to assess, learn about, validate, or decide whether to obtain commercial rights",
        ] {
            assert!(folded.contains(quoted) && readme.contains(quoted), "{quoted}");
        }
        assert!(native_intro().contains(&license_picture()));
    }
    #[test]
    fn a_license_that_no_longer_says_a_quoted_sentence_is_refused() {
        let license = fs::read_to_string(LICENSE_FILE).unwrap();
        for (from, to) in [
            ("non-production environment", "production environment"),
            ("No Monetized Use is licensed", "Monetized Use is licensed"),
            ("without a time limit", "for one year"),
            ("`Required Notice:`", "`Notice:`"),
        ] {
            assert!(license.contains(from), "{from}");
            assert!(
                license_quotes(&license.replacen(from, to, 1)).is_err(),
                "{from}"
            );
        }
        // Line breaks inside a sentence do not matter.
        let rewrapped = license.replace(
            "Licensor grants You a worldwide,",
            "Licensor grants You a\nworldwide,",
        );
        assert!(license_quotes(&rewrapped).is_ok());
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
        // The rows add up to the set: every case is counted in exactly one row.
        let rows: Vec<SpeedRow> = s.examples.iter().filter_map(SpeedRow::of).collect();
        let mut sides = [0; 3];
        for r in &rows {
            for (sum, n) in sides.iter_mut().zip(r.sides) {
                *sum += n;
            }
        }
        assert_eq!((rows.len(), sides), (timed, s.sides()));
        let text = speed_text(&s);
        for want in [
            "133 timed cases in 23 of 24 timing programs",
            "103 cases are faster beyond their A/A band, 18 slower beyond it (SHA-256 alone at 0.28 and 0.39) and 12 inside it",
            "22 of 23 timed programs",
            "Cases are grouped by timing program; per-case results are in docs/speed-results/2026-10-20-against-v0.6.1.",
            "speed_tree_gates prints DIFFERENT_OUTPUT because it counts the files of the tree it runs in",
            "speed_top1_many has only a manifest",
            "not pinned to cores",
            "Ratios of different programs are not comparable",
        ] {
            assert!(text.contains(want), "{want}");
        }
        assert!(speed_picture(&s).contains(&format!(r#"alt="{text}""#)));
        for dark in [false, true] {
            let (c, t) = (speed_card(dark, &s), theme(dark));
            // Short enough to stand in the README without a collapsed block.
            let height = c.split(" height=\"").nth(1).unwrap().split('"').next();
            assert!(height.unwrap().parse::<u32>().unwrap() <= 700);
            // SHA-256 alone is slower in every case: a red span only, left of 1.
            let sha = speed_row(&c, "speed_sha256");
            assert_eq!(speed_spans(sha).len(), 1);
            assert_eq!(speed_spans(sha)[0].0, "lo");
            for want in [
                ">0.28<".to_string(),
                ">0.33<".to_string(),
                ">0.39<".to_string(),
                format!(r#"style="fill:{}">2<"#, t.red[1]),
                format!(r#"fill="{}"/>"#, t.red[1]),
            ] {
                assert!(sha.contains(&want), "{want}");
            }
            // The overlap loads are slower in five cases: their span reaches left of 1 in red.
            let overlap = speed_row(&c, "speed_load_overlap");
            let kinds: Vec<&str> = speed_spans(overlap).iter().map(|s| s.0).collect();
            assert_eq!(kinds, ["lo", "hi"]);
            assert!(overlap.contains(&format!(r#"style="fill:{}">5<"#, t.red[1])));
            // The two exceptions keep their footnotes, numbered in name order.
            let top1 = speed_row(&c, "speed_top1_many");
            assert!(top1.contains(">not built in v0.6.1: no timing<") && top1.contains(">note 1<"));
            assert!(!top1.contains(r#"class="sp "#));
            let gates = speed_row(&c, "speed_tree_gates");
            assert!(gates.contains(">differs · note 2<") && gates.contains(t.amber[1]));
            // One check per program with the same output, and one in the legend.
            assert_eq!(c.matches("l3.5 3.5 6.5-7.5").count(), same + 1);
            for want in [
                ">133 cases:<",
                ">faster beyond its A/A band: 103<",
                ">slower beyond it: 18<",
                ">inside it: 12<",
                ">same output digest in A, A2 and B: 22 of 23<",
                "cases grouped by timing program; per-case results are in docs/speed-results",
                "medians of 9 to 27 runs per case",
                "AMD Ryzen AI 9 HX 370",
                "not pinned to cores",
                "not a confidence interval",
                "Ratios of different programs are not comparable with each other",
                "The output digests, not the timings, are the claim.",
            ] {
                assert!(c.contains(want), "{want}");
            }
            assert!(c.contains("@media (prefers-reduced-motion:reduce){.sp,.fd{animation:none}}"));
            // Spans grow out of the axis at 1, one row after another, and stop.
            let tick = r#"" y="228" text-anchor="middle" font-size="10">1<"#;
            let one = c.split(tick).next().unwrap().rsplit("x=\"").next().unwrap();
            assert!(
                c.contains(&format!("transform-origin:{one}px 0}}")),
                "{one}"
            );
            assert_eq!(c.matches(" both").count(), 2);
            assert!(!c.contains("infinite"));
            // Only the keyframes hide or shrink anything, so with the animation
            // removed every element is in its final, visible state.
            let style = c.split("<style>").nth(1).unwrap();
            let css = style.split("@keyframes").next().unwrap();
            assert!(!css.contains("opacity") && !css.contains("transform:"));
            // The fade ends at opacity 1 and holds it, so an animated element must
            // not carry its own opacity: the moving card would end darker than the still one.
            for tag in c.split('<') {
                if tag.contains(r#"class="fd"#) || tag.contains(r#"class="sp "#) {
                    assert!(!tag.contains(" opacity="), "{tag}");
                }
            }
            assert!(!c.contains("__") && !c.contains("<script") && !c.contains("href"));
            assert!(c.chars().all(|ch| ch.is_ascii() || ch == '·' || ch == '—'));
        }
    }
    /// The part of a speed card from a program's name up to the next program's.
    fn speed_row<'a>(c: &'a str, name: &str) -> &'a str {
        c.split(r#"<text class="pn b""#)
            .find(|r| {
                r.split_once('>')
                    .is_some_and(|(_, t)| t.starts_with(&format!("{name}<")))
            })
            .unwrap()
    }
    /// The spans of one row: below 1 (`lo`) or above it (`hi`), with x and width.
    fn speed_spans(row: &str) -> Vec<(&str, f64, f64)> {
        let attr = |r: &str, key: &str| -> f64 {
            let v = r.split(&format!(" {key}=\"")).nth(1).unwrap();
            v.split('"').next().unwrap().parse().unwrap()
        };
        row.split(r#"<rect class="sp "#)
            .skip(1)
            .map(|r| (r.split('"').next().unwrap(), attr(r, "x"), attr(r, "width")))
            .collect()
    }
    /// The x of the median mark of one row.
    fn speed_median(row: &str) -> f64 {
        let r = row.split(r#"<circle class="fd""#).nth(1).unwrap();
        let v = r.split(" cx=\"").nth(1).unwrap();
        v.split('"').next().unwrap().parse().unwrap()
    }
    #[test]
    fn speed_card_names_no_case() {
        let s = SpeedSet::read().unwrap();
        // Every case= value, read from the summaries here, not through the parser.
        let mut names = Vec::new();
        for entry in fs::read_dir(SPEED_DIR).unwrap() {
            let path = entry.unwrap().path();
            if let Ok(text) = fs::read_to_string(path.join("summary.txt")) {
                for line in text.lines() {
                    if let Some(rest) = line.strip_prefix("SPEED_AB case=") {
                        names.push(rest.split(' ').next().unwrap().to_string());
                    }
                }
            }
        }
        assert_eq!(names.len(), 133);
        let programs: Vec<&str> = s.examples.iter().map(|e| e.name.as_str()).collect();
        // Words of a text: runs of lower-case letters, digits and underscores.
        let words = |text: &str| -> Vec<String> {
            text.split(|ch: char| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'))
                .filter(|w| !w.is_empty())
                .map(str::to_string)
                .collect()
        };
        let alt = speed_text(&s);
        for dark in [false, true] {
            let c = speed_card(dark, &s);
            let mut drawn = words(&alt);
            for node in c.split('>').filter_map(|p| p.split_once('<')) {
                drawn.extend(words(node.0));
            }
            for name in &names {
                // A program may share a name with a case; it is drawn as the program.
                if programs.contains(&name.as_str()) {
                    continue;
                }
                // A whole text node: catches every name, even a common word such as "large".
                assert!(!c.contains(&format!(">{name}<")), "{name}");
                // Inside longer text, only a name with an underscore or a digit, which
                // no ordinary word has, so "small" in a sentence is not a case.
                if name.contains('_') || name.bytes().any(|b| b.is_ascii_digit()) {
                    assert!(!drawn.contains(name), "{name}");
                }
            }
        }
    }
    #[test]
    fn speed_card_has_one_row_per_program() {
        let s = SpeedSet::read().unwrap();
        let mut dirs = Vec::new();
        for entry in fs::read_dir(SPEED_DIR).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                dirs.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        dirs.sort();
        assert_eq!(dirs.len(), 24);
        for dark in [false, true] {
            let c = speed_card(dark, &s);
            let drawn: Vec<&str> = c
                .split(r#"<text class="pn b""#)
                .skip(1)
                .map(|r| r.split_once('>').unwrap().1.split('<').next().unwrap())
                .collect();
            assert_eq!(drawn, dirs);
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
        let timed = |name: &str, cases: Vec<SpeedCase>| SpeedExample {
            name: name.to_string(),
            timed: Some((cases, true)),
            note: None,
        };
        let s = SpeedSet {
            machine: "one machine".to_string(),
            load: "load average 1".to_string(),
            examples: vec![
                timed(
                    "speed_one",
                    vec![
                        case("quarter", 1.0, 0.25),
                        case("four", 1.0, 4.0),
                        case("same", 1.25, 1.1),
                    ],
                ),
                SpeedExample {
                    name: "speed_new".to_string(),
                    timed: None,
                    note: Some("speed_new has only a manifest.".to_string()),
                },
                timed(
                    "speed_slow",
                    vec![case("half_a", 1.0, 0.5), case("quarter_a", 1.0, 0.25)],
                ),
                timed(
                    "speed_two",
                    vec![case("double", 1.0, 2.0), case("eight", 1.0, 8.0)],
                ),
            ],
        };
        assert_eq!(s.sides(), [3, 3, 1]);
        let near = |a: f64, b: f64| (a - b).abs() < 0.16;
        for dark in [false, true] {
            let (c, t) = (speed_card(dark, &s), theme(dark));
            assert_eq!(c.matches(r#"<text class="pn b""#).count(), 4);
            for name in [
                "quarter",
                "four",
                "same",
                "half_a",
                "quarter_a",
                "double",
                "eight",
            ] {
                assert!(!c.contains(&format!(">{name}<")), "{name}");
            }
            for want in [
                ">7 cases:<",
                ">faster beyond its A/A band: 3<",
                ">slower beyond it: 3<",
                ">inside it: 1<",
                ">same output digest in A, A2 and B: 3 of 3<",
                "medians of 9 runs per case",
                ">Note 1<",
                "speed_new has only a manifest.",
            ] {
                assert!(c.contains(want), "{want}");
            }
            assert!(!c.contains(">0.28<") && !c.contains("AMD") && !c.contains("note 2"));
            // The row of three cases: one faster, one slower, one inside its band;
            // lowest, median and highest gain as printed.
            let row = speed_row(&c, "speed_one");
            for want in [
                format!(r#"style="fill:{}">1<"#, t.blue[1]),
                format!(r#"style="fill:{}">1<"#, t.red[1]),
                format!(r#"style="fill:{}">1<"#, t.ink),
                ">3<".to_string(),
                ">0.25<".to_string(),
                ">1.10<".to_string(),
                ">4.00<".to_string(),
            ] {
                assert!(row.contains(&want), "{want}");
            }
            // On a log scale a quarter and four are equally far from 1: the red
            // part and the blue part of the span are equally long and meet at 1.
            let one = speed_spans(row);
            assert_eq!((one[0].0, one[1].0), ("lo", "hi"));
            assert!(near(one[0].2, one[1].2) && near(one[0].1 + one[0].2, one[1].1));
            assert!(row.contains(r#"class="sp lo""#) && row.contains(t.red[1]));
            let (axis, octave) = (one[1].1, one[1].2 / 2.0);
            let at = |v: f64| axis + v.log2() * octave;
            assert!(near(speed_median(row), at(1.1)));
            // Slower in every case: a red span only, from a quarter to a half.
            let slow = speed_spans(speed_row(&c, "speed_slow"));
            assert_eq!(slow.len(), 1);
            assert_eq!(slow[0].0, "lo");
            assert!(near(slow[0].1, at(0.25)) && near(slow[0].2, octave));
            assert!(near(speed_median(speed_row(&c, "speed_slow")), at(0.375)));
            // Faster in every case: a blue span from 2 to 8, its median the mean of the two.
            let two = speed_row(&c, "speed_two");
            let span = speed_spans(two);
            assert_eq!(span.len(), 1);
            assert_eq!(span[0].0, "hi");
            assert!(near(span[0].1, at(2.0)) && near(span[0].2, 2.0 * octave));
            assert!(two.contains(">5.00<") && near(speed_median(two), at(5.0)));
            for (kind, colour) in [("lo", t.red[1]), ("hi", t.blue[1])] {
                for r in c.split(&format!(r#"<rect class="sp {kind}""#)).skip(1) {
                    let tag = r.split('>').next().unwrap();
                    assert!(tag.contains(&format!(r#"fill="{colour}"/"#)), "{tag}");
                }
            }
            // A program without timing draws no span, only its footnote.
            let new = speed_row(&c, "speed_new");
            assert!(new.contains(">not built in v0.6.1: no timing<") && new.contains(">note 1<"));
            assert!(!new.contains(r#"class="sp "#) && !new.contains("<circle class=\"fd\""));
        }
    }
    /// The words a reader sees without opening anything: outside code blocks,
    /// outside folded `<details>`, outside HTML tags (alternative text
    /// included) and table rules; a word has at least one ASCII letter or digit.
    fn visible_words(readme: &str) -> usize {
        let (mut code, mut folded, mut text) = (false, false, String::new());
        for line in readme.lines() {
            if line.starts_with("```") {
                code = !code;
                continue;
            }
            if code {
                continue;
            }
            if line.contains("<details>") {
                folded = true;
            }
            if folded {
                folded = !line.contains("</details>");
                continue;
            }
            let rule = line.starts_with('|') && line.chars().all(|c| "|:- ".contains(c));
            if rule {
                continue;
            }
            let mut tag = false;
            for c in line.chars() {
                match c {
                    '<' => tag = true,
                    '>' if tag => tag = false,
                    _ if !tag => text.push(c),
                    _ => {}
                }
            }
            text.push(' ');
        }
        text.split_whitespace()
            .filter(|w| w.bytes().any(|b| b.is_ascii_alphanumeric()))
            .count()
    }
    #[test]
    fn the_readme_shows_pictures_more_than_words() {
        let s = native_intro();
        let words = visible_words(&s);
        assert!(words <= 600, "{words} visible words");
        // The layout before the pictures, kept as the archive of this change, fails the limit.
        let before = include_str!("readme_before_visual.txt");
        assert!(visible_words(before) > 1600, "{}", visible_words(before));
        assert!(s.matches("<img ").count() >= before.matches("<img ").count() + VISUALS.len());
        // Every fold opens with a summary and closes, so no text is lost from view by accident.
        assert_eq!(
            s.matches("<details>").count(),
            s.matches("</details>").count()
        );
        assert_eq!(
            s.matches("<details><summary>").count(),
            s.matches("<details>").count()
        );
    }
    #[test]
    fn every_sentence_the_registry_cites_occurs_once() {
        // `xtask claims` refuses a cited sentence that occurs twice; an
        // alternative text that repeats one would fail there, so it fails here first.
        let claims = fs::read_to_string("xtask/src/claims.rs").unwrap();
        let fold = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let readme = fold(&native_intro());
        let mut cited = 0;
        for entry in claims.split("Citation {").skip(1) {
            if !entry.contains("file: \"README.md\"") {
                continue;
            }
            let words = entry
                .split_once("words: \"")
                .and_then(|(_, rest)| rest.split_once("\",\n"))
                .map(|(w, _)| w.replace("\\\"", "\""))
                .unwrap();
            assert_eq!(readme.matches(&fold(&words)).count(), 1, "{words}");
            cited += 1;
        }
        assert!(cited >= 10, "{cited} cited README sentences found");
    }
    #[test]
    fn folding_counts_what_it_hides() {
        let open = "# T\n\nOne two three.\n";
        assert_eq!(visible_words(open), 4);
        let folded = "# T\n\n<details><summary>s</summary>\n\nOne two three.\n\n</details>\n";
        assert_eq!(visible_words(folded), 1);
        let code = "```sh\ncargo run\n```\n| a | b |\n|:---|:---|\n<img alt=\"many words here\" src=\"x\">\n";
        assert_eq!(visible_words(code), 2);
    }
    #[test]
    fn every_visual_is_drawn_from_data_and_can_stand_still() {
        for (name, draw, alt) in VISUALS {
            let alt = alt().unwrap();
            assert!(!alt.contains("STUB") && !alt.contains('"'), "{name}");
            assert!(native_intro().contains(&visual_picture(name)), "{name}");
            for dark in [false, true] {
                let s = draw(dark).unwrap();
                assert!(!s.contains("STUB"), "{name}");
                assert!(s.contains("<title"), "{name}");
                assert!(s.contains("prefers-reduced-motion"), "{name}");
                assert!(s.contains("@keyframes"), "{name}: a visual moves");
                assert!(!s.contains("<script") && !s.contains("href"), "{name}");
            }
            assert_ne!(draw(false).unwrap(), draw(true).unwrap(), "{name}");
        }
    }
    /// The names of the 1200-wide pictures of the README.
    fn wide_pictures(readme: &str) -> Vec<String> {
        let mut wide = Vec::new();
        for part in readme.split(r#"src="media/presentation/"#).skip(1) {
            let (file, rest) = part.split_once('"').unwrap();
            if rest.starts_with(r#" width="1200""#) {
                wide.push(file.strip_suffix("-light.svg").unwrap().to_string());
            }
        }
        wide
    }
    #[test]
    fn every_wide_card_has_a_phone_card() {
        let readme = native_intro();
        let wide = wide_pictures(&readme);
        assert!(wide.len() >= 14, "{wide:?}");
        for name in &wide {
            let sources = phone_sources(name);
            assert!(!sources.is_empty(), "{name}: no phone card");
            // The narrow-screen sources come first, so a phone never falls through
            // to the wide card.
            let picture = readme
                .split(&format!(r#"src="media/presentation/{name}-light.svg""#))
                .next()
                .unwrap()
                .rsplit("<picture>")
                .next()
                .unwrap();
            let at = |needle: &str| picture.find(needle).unwrap_or_else(|| panic!("{name}"));
            assert!(
                at(&phone_path(name, true)) < at(&phone_path(name, false)),
                "{name}"
            );
            assert!(
                picture
                    .trim_start()
                    .starts_with("<source media=\"(max-width"),
                "{name}"
            );
        }
        // Every phone card belongs to a wide card the README shows.
        for (name, _) in PHONES {
            assert!(wide.iter().any(|w| w == name), "{name}");
        }
    }
    /// The checks every phone card passes: drawn, 600 wide, no text under the
    /// phone minimum, a title, motion that can stand still, no script or link,
    /// and two themes that differ.
    fn check_phone(name: &str) {
        let (_, draw) = PHONES.iter().find(|p| p.0 == name).unwrap();
        for dark in [false, true] {
            let s = draw(dark).unwrap_or_else(|e| panic!("{name}: {e}"));
            check_phone_svg(name, &s);
        }
        assert_ne!(draw(false).unwrap(), draw(true).unwrap(), "{name}");
    }
    /// The checks of one drawn phone card: the root `<svg>` tag is 600 wide,
    /// the card's own size is the phone minimum, no text is under it, and it
    /// has a title, motion that can stand still, no script or link.
    fn check_phone_svg(name: &str, s: &str) {
        assert!(!s.contains("STUB"), "{name}");
        let root = s
            .split_once("<svg")
            .and_then(|(_, r)| r.split_once('>'))
            .unwrap_or_else(|| panic!("{name}: no svg tag"))
            .0;
        assert!(
            root.contains(&format!(r#" width="{PHONE_WIDTH}""#))
                && root.contains(&format!(r#" viewBox="0 0 {PHONE_WIDTH} "#)),
            "{name}: {PHONE_WIDTH} wide"
        );
        assert!(
            s.contains(&format!("<style>\nsvg{{font-size:{PHONE_MIN_FONT}px}}")),
            "{name}: a text without a size inherits the phone minimum"
        );
        let sizes = font_sizes(s);
        assert!(!sizes.is_empty(), "{name}");
        for size in sizes {
            assert!(size >= PHONE_MIN_FONT, "{name}: {size} px text");
        }
        assert!(s.contains("<title"), "{name}");
        assert!(s.contains("prefers-reduced-motion"), "{name}");
        assert!(s.contains("@keyframes"), "{name}: a phone card moves");
        assert!(!s.contains("<script") && !s.contains("href"), "{name}");
    }
    #[test]
    fn phone_cards_are_readable_on_a_phone() {
        for (name, _) in PHONES {
            check_phone(name);
        }
    }
    /// A frame that passes every other check of a phone card.
    const CHECK_PHONE_STYLE: &str =
        ".m{animation:m 1s both}@keyframes m{from{opacity:0}}@media (prefers-reduced-motion:reduce){.m{animation:none}}";
    #[test]
    fn check_phone_passes_the_raised_frame() {
        check_phone_svg(
            "frame",
            &phone_card(false, 100, ("t", "d"), CHECK_PHONE_STYLE, ""),
        );
    }
    #[test]
    #[should_panic(expected = "px text")]
    fn check_phone_refuses_small_text() {
        // The shared frame at phone width, its kicker not raised: under the minimum.
        let s = phone_card(false, 100, ("t", "d"), CHECK_PHONE_STYLE, "").replacen(
            ".kick{font-size:20px;",
            ".kick{font-size:13px;",
            1,
        );
        check_phone_svg("frame", &s);
    }
    #[test]
    #[should_panic(expected = "wide")]
    fn check_phone_reads_the_width_of_the_root_tag_only() {
        // A 600-wide inner element does not make a narrower card a phone card;
        // otherwise the frame passes every check.
        let s = phone_card(false, 100, ("t", "d"), CHECK_PHONE_STYLE, "")
            .replacen(
                &format!(r#"width="{PHONE_WIDTH}" height="100" viewBox="0 0 {PHONE_WIDTH} "#),
                r#"width="500" height="100" viewBox="0 0 500 "#,
                1,
            )
            .replacen(
                "</style>",
                "</style><svg width=\"600\" viewBox=\"0 0 600 10\"/>",
                1,
            );
        assert!(s.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="500""#));
        check_phone_svg("frame", &s);
    }
    #[test]
    #[should_panic(expected = "inherits the phone minimum")]
    fn check_phone_refuses_a_card_without_the_size_floor() {
        let s = card(
            (PHONE_WIDTH, 100),
            theme(false),
            ("t", "d"),
            CHECK_PHONE_STYLE,
            "",
        )
        .replacen(".kick{font-size:13px;", ".kick{font-size:20px;", 1);
        check_phone_svg("frame", &s);
    }
    #[test]
    fn font_sizes_reads_attributes_declarations_and_shorthands() {
        let s = r#"<style>.a{font-size:13px}.b{font:700 22px Arial}</style><text font-size="16.5">x</text>"#;
        assert_eq!(font_sizes(s), vec![16.5, 13.0, 22.0]);
        assert!(phone_card(false, 100, ("t", "d"), "", "").contains(".kick{font-size:20px;"));
    }
    #[test]
    fn the_badges_row_is_drawn_and_links_to_files() {
        let row = badges_row().unwrap();
        assert!(row.matches("<a href=\"").count() >= 6, "{row}");
        assert!(native_intro().contains(&row));
        for part in row.split("<a href=\"").skip(1) {
            let target = part.split_once('"').unwrap().0;
            let file = target.split_once('#').map_or(target, |t| t.0);
            assert!(fs::metadata(file).is_ok(), "{target}: no such file");
        }
    }
    // ===== PHONE TEST a1 BEGIN =====
    /// Every text of a card that states its own x and size (the kicker by its
    /// class): the text and its left and right edge, at 0.62 of the font size
    /// per character plus letter spacing, with 8 % of the width spare.
    fn header_phone_spans(svg: &str) -> Vec<(String, f64, f64)> {
        let mut out = Vec::new();
        for part in svg.split("<text").skip(1) {
            let (attrs, rest) = part.split_once('>').unwrap();
            let attr = |k: &str| {
                attrs
                    .split_once(&format!(" {k}=\""))
                    .map(|(_, v)| v.split_once('"').unwrap().0.to_string())
            };
            let (size, spacing) = if attrs.contains(r#"class="kick""#) {
                (Some("20".to_string()), 2.0)
            } else {
                (
                    attr("font-size"),
                    attr("letter-spacing").map_or(0.0, |s| s.parse().unwrap()),
                )
            };
            let (Some(x), Some(size)) = (attr("x"), size) else {
                continue;
            };
            let (x, size): (f64, f64) = (x.parse().unwrap(), size.parse().unwrap());
            let mut text = String::new();
            let mut tag = false;
            for c in rest.split_once("</text>").unwrap().0.chars() {
                match c {
                    '<' => tag = true,
                    '>' => tag = false,
                    _ if !tag => text.push(c),
                    _ => {}
                }
            }
            let text = text
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&amp;", "&");
            let width = text.chars().count() as f64 * (0.62 * size + spacing) / 0.92;
            let (l, r) = if attrs.contains(r#"text-anchor="middle""#) {
                (x - width / 2.0, x + width / 2.0)
            } else if attrs.contains(r#"text-anchor="end""#) {
                (x - width, x)
            } else {
                assert!(!attrs.contains("text-anchor"), "{attrs}");
                (x, x + width)
            };
            out.push((text, l, r));
        }
        out
    }
    /// The texts of a card drawn whole between its framed sides (24 px each).
    fn header_phone_inside(svg: &str) -> std::result::Result<usize, String> {
        let spans = header_phone_spans(svg);
        for (text, l, r) in &spans {
            if *l < 24.0 || *r > f64::from(PHONE_WIDTH) - 24.0 {
                return Err(format!("{text}: {l:.0}..{r:.0}"));
            }
        }
        Ok(spans.len())
    }
    /// Every animation shorthand keeps its end state, and every animated class
    /// stands still under reduced motion with nothing hidden by a base rule.
    fn header_phone_motion_ok(svg: &str) {
        assert_eq!(status_motion(svg), (Vec::new(), Vec::new()));
        let style = svg.split_once("<style>").unwrap().1;
        let style = style.split_once("@media").unwrap().0;
        for decl in style.split("animation:").skip(1) {
            let decl = decl.split(['}', ';']).next().unwrap();
            assert!(decl.contains("both"), "{decl}");
        }
        assert!(style.contains("animation-fill-mode:both"));
        assert!(
            !svg.contains("<animate"),
            "CSS motion only, so it can stand still"
        );
        assert!(!svg.contains("textLength") && !svg.contains("scale(0"));
    }
    /// The byte count every recorded run states, read without `Q8Runs`.
    fn header_phone_record_bytes() -> usize {
        let mut seen = Vec::new();
        for e in fs::read_dir(RECORD_DIR).unwrap() {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            if !(name.starts_with("v2-") && name.ends_with(".txt")) {
                continue;
            }
            let text = fs::read_to_string(format!("{RECORD_DIR}/{name}")).unwrap();
            for part in text.split("canonical_record_bytes=").skip(1) {
                let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
                seen.push(digits.parse::<usize>().unwrap());
            }
        }
        assert!(!seen.is_empty());
        assert!(seen.iter().all(|v| *v == seen[0]), "{seen:?}");
        seen[0]
    }
    #[test]
    fn header_status_and_glance_phones_pass_the_phone_checks() {
        for name in ["header", "status", "glance"] {
            check_phone(name);
            for dark in [false, true] {
                let s = PHONES.iter().find(|p| p.0 == name).unwrap().1(dark).unwrap();
                header_phone_motion_ok(&s);
                assert!(header_phone_inside(&s).unwrap() >= 6, "{name}");
                assert!(!s.contains("__") && !s.contains("/home"));
                assert!(s.chars().all(|ch| ch.is_ascii() || "·—→×✓".contains(ch)));
                let h: f64 = s
                    .split_once(" height=\"")
                    .unwrap()
                    .1
                    .split_once('"')
                    .unwrap()
                    .0
                    .parse()
                    .unwrap();
                // The banner leaves room on a phone's first screen for the
                // badges and the first paragraph.
                let tall = if name == "header" {
                    560.0..=820.0
                } else {
                    500.0..=1050.0
                };
                assert!(tall.contains(&h), "{name}: {h}");
            }
        }
    }
    #[test]
    fn header_phone_shows_the_record_and_the_words_of_the_header() {
        let bytes = header_phone_record_bytes();
        let alt = header_picture().to_lowercase();
        let mut groups: Vec<&[&str]> = vec![&HEADER_PHONE_TAG, &HEADER_PHONE_RECORD];
        groups.extend(HEADER_PHONE_CLAIMS);
        for dark in [false, true] {
            let s = header_phone(dark).unwrap();
            assert_eq!(glance_rolled(&s), [grouped(bytes)]);
            assert!(s.contains(">GEL RAM<") && s.contains(">bytes<"));
            // The logo of the README, still: cube, layers, the exact cell and the seal.
            for class in ["l0", "l1", "l2", "hit", "ray", "seal"] {
                assert!(s.contains(&format!("class=\"{class}\"")), "{class}");
            }
            for lines in &groups {
                for line in lines.iter() {
                    assert!(s.contains(&format!(">{}<", xml(line))), "{line}");
                }
                // Each group is one sentence of the wide header, word for word.
                let joined = lines.join(" ").to_lowercase();
                assert!(alt.contains(&joined), "{joined}");
            }
            // Every box is a checked claim; the banner does not put UNKNOWN,
            // a behaviour, beside them with a check of the same weight.
            assert!(!s.contains("UNKNOWN"));
            assert_eq!(
                s.matches("<g class=\"hb\"").count(),
                HEADER_PHONE_CLAIMS.len()
            );
        }
        // The banner sends a phone to this card before the still banner, so its
        // own reduced-motion rule is what keeps it still.
        let picture = header_picture();
        let phone = picture.find(&phone_path("header", false)).unwrap();
        assert!(phone < picture.find("header-still-light.svg").unwrap());
    }
    #[test]
    fn header_phone_draws_another_size_and_refuses_one_too_wide() {
        let s = header_phone_draw(true, 1280).unwrap();
        assert_eq!(glance_rolled(&s), ["1,280"]);
        assert!(s.contains("four exact views, 1,280 bytes."));
        assert_ne!(s, header_phone(true).unwrap());
        let e = header_phone_draw(false, usize::MAX).unwrap_err();
        assert!(e.to_string().contains("does not fit"), "{e}");
    }
    /// The dated rows of the published commitments, counted by their SHA-256 cells.
    fn status_phone_hashes() -> usize {
        let text = fs::read_to_string("docs/COMMITMENTS.md").unwrap();
        let table = text.split_once("## Published commitments\n").unwrap().1;
        let table = table.split("\n## ").next().unwrap();
        table
            .lines()
            .filter(|l| {
                l.split('`')
                    .skip(1)
                    .step_by(2)
                    .any(|cell| cell.len() == 64 && cell.bytes().all(|b| b.is_ascii_hexdigit()))
            })
            .count()
    }
    #[test]
    fn status_phone_counts_the_published_commitments() {
        let n = status_phone_hashes();
        assert!(n > 0);
        for dark in [false, true] {
            let s = status_phone(dark).unwrap();
            assert_eq!(glance_rolled(&s), [n.to_string()]);
            let mut wants: Vec<String> = Vec::new();
            for (heading, _, items) in STATUS_COLUMNS {
                wants.push(heading.to_string());
                for item in items {
                    wants.extend(wrap(item, STATUS_PHONE_WRAP));
                }
            }
            wants.extend(STATUS_FEET.map(str::to_string));
            wants.extend(STATUS_PLEDGE.map(str::to_string));
            for want in wants {
                assert!(s.contains(&format!(">{}<", xml(&want))), "{want}");
            }
            // Each foot stands on its heading's line, at the far end, clear of it.
            let lines = header_phone_spans(&s);
            let line = |words: &str| lines.iter().position(|t| t.0 == words).unwrap();
            let y_of = |words: &str| {
                let at = s.find(&format!(">{words}<")).unwrap();
                let tag = s[..at].rsplit("<text").next().unwrap();
                let y = tag.split_once(" y=\"").unwrap().1;
                y.split('"').next().unwrap().parse::<f64>().unwrap()
            };
            for (heading, foot) in [
                (STATUS_COLUMNS[0].0, STATUS_FEET[0]),
                (STATUS_COLUMNS[2].0, STATUS_FEET[1]),
            ] {
                assert_eq!(y_of(heading), y_of(foot), "{foot}");
                assert!(lines[line(heading)].2 <= lines[line(foot)].1, "{foot}");
            }
            assert!(s.contains(&xml(&status_alt().unwrap())));
            // One block per column, in order, each with its icon drawn in.
            assert_eq!(s.matches(r#"<g class="sc""#).count(), STATUS_COLUMNS.len());
            assert_eq!(s.matches(r#"class="sd""#).count(), STATUS_COLUMNS.len());
            let at: Vec<usize> = STATUS_COLUMNS
                .iter()
                .map(|c| s.find(&format!(">{}<", c.0)).unwrap())
                .collect();
            assert!(at.windows(2).all(|p| p[0] < p[1]));
        }
    }
    #[test]
    fn status_phone_draws_another_count_and_refuses_one_too_wide() {
        let s = status_phone_draw(true, &StatusFacts { commitments: 23 }).unwrap();
        assert_eq!(glance_rolled(&s), ["23"]);
        assert!(s.contains("23 published commitments fix the contents"));
        assert_ne!(s, status_phone(true).unwrap());
        assert!(status_phone_draw(false, &StatusFacts { commitments: 0 }).is_err());
        let e = status_phone_draw(
            false,
            &StatusFacts {
                commitments: usize::MAX,
            },
        )
        .unwrap_err();
        assert!(e.to_string().contains("does not fit"), "{e}");
    }
    /// The tiles of a glance card: the tag and the rolled number of each.
    fn glance_phone_tiles(svg: &str) -> Vec<(String, Vec<String>)> {
        svg.split(r#"<g class="gt""#)
            .skip(1)
            .map(|tile| {
                let tag = tile
                    .split_once(r#"text-anchor="middle""#)
                    .unwrap()
                    .1
                    .split_once('>')
                    .unwrap()
                    .1
                    .split_once('<')
                    .unwrap()
                    .0
                    .to_string();
                (tag, glance_rolled(tile))
            })
            .collect()
    }
    #[test]
    fn glance_phone_shows_the_numbers_of_the_files() {
        let f = GlanceFacts::read().unwrap();
        // The version before, from the README heading; the release from RELEASE.
        let since = NATIVE
            .lines()
            .find_map(|l| l.strip_prefix("## New in "))
            .unwrap()
            .rsplit(' ')
            .next()
            .unwrap();
        assert_eq!(since, f.since);
        assert!(RELEASE.starts_with(&format!("v{}.", f.release)));
        for dark in [false, true] {
            let s = glance_phone(dark).unwrap();
            let (new, old) = (
                format!("new in {}", f.release),
                format!("since {}", f.since),
            );
            // Three tiles are new in the release; the programs came before it.
            assert_eq!(
                glance_phone_tiles(&s),
                [
                    (new.clone(), vec![f.faults.to_string()]),
                    (new.clone(), vec![f.samples.to_string()]),
                    (new, vec![f.steps.to_string()]),
                    (old, vec![f.tools.len().to_string()]),
                ]
            );
            for want in [
                format!("{} AND {} AT A GLANCE", f.since, f.release),
                format!(">memory samples of {} operations<", f.operations),
                ">no power-loss test among them<".to_string(),
                ">one machine, process-wide, no time<".to_string(),
                ">one block, no host or user names<".to_string(),
                ">--version and --license<".to_string(),
                xml(&glance_alt().unwrap()),
            ] {
                assert!(s.contains(&want), "{want}");
            }
        }
    }
    #[test]
    fn glance_phone_draws_other_facts_and_refuses_one_too_wide() {
        let mut f = glance_other_facts();
        let s = glance_phone_draw(false, &f).unwrap();
        assert_eq!(
            glance_phone_tiles(&s),
            [
                ("new in 0.9".to_string(), vec!["3".to_string()]),
                ("new in 0.9".to_string(), vec!["4".to_string()]),
                ("new in 0.9".to_string(), vec!["2".to_string()]),
                ("since 0.8".to_string(), vec!["3".to_string()]),
            ]
        );
        assert!(s.contains("0.8 AND 0.9 AT A GLANCE"));
        assert!(s.contains(">memory samples of 2 operations<"));
        assert_ne!(s, glance_phone(false).unwrap());
        f.faults = usize::MAX;
        let e = glance_phone_draw(false, &f).unwrap_err();
        assert!(e.to_string().contains("does not fit"), "{e}");
        f.faults = 3;
        f.samples = 5;
        assert!(glance_phone_draw(false, &f).is_err());
    }
    #[test]
    fn header_phone_fit_checker_catches_a_wide_line() {
        let s = status_phone(false).unwrap();
        let item = STATUS_COLUMNS[0].2[1];
        let wide = s.replace(
            &format!(">{}<", xml(item)),
            &format!(">{} {} {}<", item, item, item),
        );
        assert!(header_phone_inside(&wide).is_err());
        let left = s.replace(
            r#"<text class="kick" x="32""#,
            r#"<text class="kick" x="8""#,
        );
        assert!(header_phone_inside(&left).is_err());
        let hidden = s.replace(".sc{animation", ".sc{opacity:0;animation");
        assert!(std::panic::catch_unwind(|| header_phone_motion_ok(&hidden)).is_err());
        let open = s.replace("ease-out both}", "ease-out}");
        assert!(std::panic::catch_unwind(|| header_phone_motion_ok(&open)).is_err());
    }
    // ===== PHONE TEST a1 END =====

    // ===== PHONE TEST a2 BEGIN =====
    /// The top-level `selector{body}` pairs of a style sheet; nested bodies are kept whole.
    fn record_blocks(css: &str) -> Vec<(String, String)> {
        let (mut out, mut depth, mut open, mut from) = (Vec::new(), 0, 0, 0);
        for (i, c) in css.char_indices() {
            match c {
                '{' => {
                    if depth == 0 {
                        open = i;
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        out.push((
                            css[from..open].trim().to_string(),
                            css[open + 1..i].to_string(),
                        ));
                        from = i + 1;
                    }
                }
                _ => {}
            }
        }
        out
    }
    /// Splits at the commas that are not inside parentheses.
    fn record_list(value: &str) -> Vec<String> {
        let (mut out, mut depth, mut from) = (Vec::new(), 0, 0);
        for (i, c) in value.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(value[from..i].trim().to_string());
                    from = i + 1;
                }
                _ => {}
            }
        }
        out.push(value[from..].trim().to_string());
        out
    }
    /// The declarations of a rule as (property, value) pairs.
    fn record_decls(body: &str) -> Vec<(String, String)> {
        body.split(';')
            .filter_map(|d| d.split_once(':'))
            .map(|(p, v)| {
                let v = v.trim().trim_end_matches("!important").trim();
                (p.trim().to_string(), v.to_string())
            })
            .collect()
    }
    /// Whether one declaration leaves nothing to see: opacity 0, display none or
    /// a zero scale. Returns the property it is about, and whether it hides.
    fn record_hides(prop: &str, value: &str) -> Option<(&'static str, bool)> {
        match prop {
            "opacity" => Some(("opacity", value.parse::<f64>().is_ok_and(|o| o == 0.0))),
            "display" => Some(("display", value == "none")),
            "transform" => {
                let zero = ["scale(", "scaleX(", "scaleY("].iter().any(|f| {
                    value.split(f).skip(1).any(|rest| {
                        rest.split([')', ','])
                            .next()
                            .and_then(|n| n.trim().parse::<f64>().ok())
                            == Some(0.0)
                    })
                });
                Some(("transform", zero))
            }
            _ => None,
        }
    }
    /// The animated elements of a card that go blank again after they have
    /// appeared. An element counts unless a base rule keeps it hidden (a passing
    /// pointer or a cover, never part of the final picture). For each property
    /// that can hide it, a keyframe that hides it after one that shows it, or an
    /// animation that repeats and starts hidden, makes it go blank again.
    fn record_blanks(svg: &str) -> Vec<String> {
        let css = svg
            .split_once("<style>")
            .and_then(|(_, s)| s.split_once("</style>"))
            .map_or("", |(s, _)| s);
        let body = svg.split_once("</style>").map_or(svg, |(_, b)| b);
        let blocks = record_blocks(css);
        let mut frames: Vec<(String, Vec<(f64, String)>)> = Vec::new();
        for (sel, inner) in &blocks {
            if let Some(name) = sel.strip_prefix("@keyframes ") {
                let mut stops: Vec<(f64, String)> = Vec::new();
                for (at, decls) in record_blocks(inner) {
                    for p in at.split(',') {
                        let pct = match p.trim() {
                            "from" => 0.0,
                            "to" => 100.0,
                            q => q.trim_end_matches('%').parse().unwrap(),
                        };
                        stops.push((pct, decls.clone()));
                    }
                }
                stops.sort_by(|a, b| a.0.total_cmp(&b.0));
                frames.push((name.trim().to_string(), stops));
            }
        }
        let known = |n: &str| frames.iter().any(|f| f.0 == n);
        // Class -> (keyframes, repeats), and the classes a base rule hides.
        let (mut moves, mut hidden) = (Vec::<(String, String, bool)>::new(), Vec::new());
        for (sel, inner) in blocks.iter().filter(|b| !b.0.starts_with('@')) {
            let decls = record_decls(inner);
            let classes: Vec<&str> = sel
                .split(',')
                .filter_map(|s| s.trim().strip_prefix('.'))
                .filter(|c| !c.contains(['.', ' ', ':', '>']))
                .collect();
            if decls
                .iter()
                .any(|(p, v)| record_hides(p, v).is_some_and(|h| h.1))
            {
                hidden.extend(classes.iter().map(|c| c.to_string()));
            }
            let mut names = Vec::new();
            let mut counts = Vec::new();
            for (p, v) in &decls {
                match p.as_str() {
                    "animation" => {
                        for item in record_list(v) {
                            let words: Vec<&str> = item.split_whitespace().collect();
                            if let Some(n) = words.iter().find(|w| known(w)) {
                                names.push(n.to_string());
                                counts.push(words.iter().any(|w| {
                                    *w == "infinite" || w.parse::<f64>().is_ok_and(|k| k > 1.0)
                                }));
                            }
                        }
                    }
                    "animation-name" => names.extend(record_list(v)),
                    "animation-iteration-count" => counts.extend(
                        record_list(v)
                            .iter()
                            .map(|w| w == "infinite" || w.parse::<f64>().is_ok_and(|k| k > 1.0)),
                    ),
                    _ => {}
                }
            }
            for c in &classes {
                for (i, n) in names.iter().enumerate() {
                    let repeats = counts.get(i).or(counts.first()).copied().unwrap_or(false);
                    moves.push((c.to_string(), n.clone(), repeats));
                }
            }
        }
        let mut out = Vec::new();
        for part in body.split("class=\"").skip(1) {
            let classes: Vec<&str> = part.split('"').next().unwrap().split_whitespace().collect();
            if classes.iter().any(|c| hidden.iter().any(|h| h == c)) {
                continue;
            }
            for (class, name, repeats) in moves.iter().filter(|m| classes.contains(&m.0.as_str())) {
                let stops = &frames.iter().find(|f| &f.0 == name).unwrap().1;
                for prop in ["opacity", "display", "transform"] {
                    let seq: Vec<bool> = stops
                        .iter()
                        .flat_map(|(_, d)| record_decls(d))
                        .filter_map(|(p, v)| record_hides(&p, &v))
                        .filter(|h| h.0 == prop)
                        .map(|h| h.1)
                        .collect();
                    let again = seq.iter().skip_while(|h| **h).any(|h| *h);
                    let restarts = *repeats && seq.first() == Some(&true);
                    let what = format!("{class}: {name} ({prop})");
                    if (again || restarts) && !out.contains(&what) {
                        out.push(what);
                    }
                }
            }
        }
        out
    }
    #[test]
    fn record_blanks_finds_a_card_that_goes_blank_again() {
        let card = |css: &str, classes: &str| {
            format!("<svg><style>{css}</style><g class=\"{classes}\"><text>x</text></g></svg>")
        };
        let old = ".r0{animation:r0 12s ease-out infinite}@keyframes r0{0%{opacity:0}5%,97%{opacity:1}100%{opacity:0}}";
        assert_eq!(record_blanks(&card(old, "a r0")), ["r0: r0 (opacity)"]);
        for blank in [
            // Played once, but its last frame is empty.
            ".r0{animation:r0 12s ease-out both}@keyframes r0{0%{opacity:0}5%,97%{opacity:1}100%{opacity:0}}",
            // Endless, and every turn starts from nothing.
            ".r0{animation:r0 1s cubic-bezier(.2,.8,.2,1) infinite}@keyframes r0{0%{transform:scaleX(0)}100%{transform:none}}",
            ".r0{animation:r0 1s ease 3 both}@keyframes r0{from{opacity:0}to{opacity:1}}",
            ".r0{animation:r0 2s both}@keyframes r0{0%{display:none}10%{display:block}90%{display:none}}",
            ".r0{animation-name:r0;animation-duration:2s;animation-iteration-count:infinite}@keyframes r0{0%{opacity:0}10%{opacity:1}}",
        ] {
            assert_eq!(record_blanks(&card(blank, "r0")).len(), 1, "{blank}");
        }
        for still in [
            // Appears once and holds.
            ".r0{animation:r0 1s ease-out both}@keyframes r0{0%{opacity:0}100%{opacity:1}}",
            // A pulse that never reaches nothing.
            ".r0{animation:r0 1s ease-in-out infinite alternate}@keyframes r0{from{opacity:1}to{opacity:.35}}",
            // A dash that runs for ever.
            ".r0{animation:r0 1s linear infinite}@keyframes r0{to{stroke-dashoffset:-36}}",
        ] {
            assert!(record_blanks(&card(still, "r0")).is_empty(), "{still}");
        }
        // A passing pointer, hidden by its base rule, may come and go.
        let pointer = ".scan{opacity:0}.s0{animation:s0 12s linear infinite}@keyframes s0{0%{opacity:0}50%{opacity:1}100%{opacity:0}}";
        assert!(record_blanks(&card(pointer, "a scan s0")).is_empty());
        assert_eq!(record_blanks(&card(pointer, "a s0")).len(), 1);
    }
    #[test]
    fn record_and_phone_cards_never_go_blank_after_they_appear() {
        let recorded = Q8Runs::read().unwrap();
        let other = Q8Runs {
            n: 3,
            ok: 2,
            record: 2000,
            packed: 8000,
            frames: 9000,
        };
        for dark in [false, true] {
            for r in [&recorded, &other] {
                let wide = record_card(dark, r);
                assert_eq!(record_blanks(&wide), Vec::<String>::new());
                // The checker sees the elements: the wide card has content that moves.
                assert!(wide.matches("class=\"a ").count() >= 20);
                assert!(!wide.contains("infinite"));
            }
            for name in ["terminal", "record", "flow"] {
                let (_, draw) = PHONES.iter().find(|p| p.0 == name).unwrap();
                let s = draw(dark).unwrap();
                assert_eq!(record_blanks(&s), Vec::<String>::new(), "{name}");
            }
        }
    }
    #[test]
    fn the_wide_flow_card_appears_once_and_holds() {
        for dark in [false, true] {
            let s = scene(dark);
            assert_eq!(record_blanks(&s), Vec::<String>::new());
            let css = s
                .split_once("<style>")
                .unwrap()
                .1
                .split("</style>")
                .next()
                .unwrap();
            let blocks = record_blocks(css);
            let (mut moving, mut pulsing) = (0, 0);
            for (sel, body) in blocks.iter().filter(|b| !b.0.starts_with('@')) {
                let decls = record_decls(body);
                // No base rule hides anything: without animation the whole flow shows.
                assert!(
                    !decls
                        .iter()
                        .any(|(p, v)| record_hides(p, v).is_some_and(|h| h.1)),
                    "{sel}"
                );
                for (p, v) in &decls {
                    if p == "animation" {
                        moving += 1;
                        // Only the dashes run for ever; the changed cell pulses a
                        // few times; every entrance plays once and holds its last frame.
                        for item in record_list(v) {
                            let endless = item.contains("infinite");
                            assert_eq!(endless, item.starts_with("sc-flow "), "{sel}: {item}");
                            assert!(endless || item.ends_with(" both"), "{sel}: {item}");
                            if item.starts_with("sc-hot ") {
                                assert_eq!(flow_pulses(&item), Some(6), "{item}");
                                pulsing += 1;
                            }
                        }
                    }
                }
            }
            assert_eq!(pulsing, 1);
            assert!(moving >= 10, "{moving}");
            // Every moving class is an element of the card, and reduced motion stops all.
            for (sel, _) in blocks
                .iter()
                .filter(|b| !b.0.starts_with('@') && b.1.contains("animation"))
            {
                let class = sel.trim_start_matches('.');
                assert!(
                    s.contains(&format!(" {class}\""))
                        || s.contains(&format!("\"{class}\""))
                        || s.contains(&format!("\"{class} ")),
                    "{sel}"
                );
            }
            assert!(
                css.contains("@media (prefers-reduced-motion:reduce){*{animation:none!important}}")
            );
            // The phone card still reads the wide card's words.
            assert_eq!(flow_phone_read(&s).unwrap().steps.len(), 4);
        }
    }
    /// The whole of every group that opens with `<g class="{class}">`, nested
    /// groups included.
    fn terminal_phone_groups<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
        let open = format!("<g class=\"{class}\">");
        let mut out = Vec::new();
        for (at, _) in svg.match_indices(&open) {
            let (mut depth, mut i) = (0usize, at);
            loop {
                let rest = &svg[i..];
                let (o, c) = (rest.find("<g"), rest.find("</g>"));
                match (o, c) {
                    (Some(o), Some(c)) if o < c => {
                        depth += 1;
                        i += o + 2;
                    }
                    (_, Some(c)) => {
                        depth -= 1;
                        i += c + 4;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => panic!("an open group"),
                }
            }
            out.push(&svg[at..i]);
        }
        out
    }
    #[test]
    fn terminal_phone_types_each_name_inside_its_tile() {
        let checks = terminal_checks(NATIVE).unwrap();
        for dark in [false, true] {
            let s = terminal_phone(dark).unwrap();
            let covers = s.matches("class=\"tq-c ").count();
            assert_eq!(covers, checks.len());
            // Each tile's group holds its tile, its number, its name and the
            // cover that types the name, so nothing floats before the tile.
            let mut inside = 0;
            for (k, c) in checks.iter().enumerate() {
                let tile = (0..3 * checks.len())
                    .map(|i| format!("tq{i}"))
                    .flat_map(|class| terminal_phone_groups(&s, &class))
                    .find(|g| g.contains(&format!(">{}</text>", k + 1)) && g.contains(" r=\"18\""))
                    .unwrap_or_else(|| panic!("no tile {}", k + 1));
                assert!(
                    tile.contains(&format!(">{}</text>", xml(&c.key))),
                    "{}",
                    c.key
                );
                assert_eq!(tile.matches("class=\"tq-c ").count(), 1, "{}", c.key);
                inside += 1;
            }
            assert_eq!(inside, covers);
        }
        assert_eq!(
            terminal_phone_groups(
                "<g class=\"a\"><g>x</g><rect/></g><g class=\"a\">y</g>",
                "a"
            ),
            ["<g class=\"a\"><g>x</g><rect/></g>", "<g class=\"a\">y</g>"]
        );
    }
    /// The iteration count of one item of an `animation` list: its bare whole
    /// number, if it states one.
    fn flow_pulses(item: &str) -> Option<u32> {
        item.split_whitespace().find_map(|w| w.parse().ok())
    }
    #[test]
    fn the_phone_flow_card_pulses_the_changed_cell_a_few_times() {
        assert_eq!(flow_pulses("sc-hot 1s ease 8.4s 6 alternate both"), Some(6));
        assert_eq!(flow_pulses("sc-hot 1s ease infinite alternate"), None);
        for dark in [false, true] {
            let s = flow_phone(dark).unwrap();
            let css = s.split_once("<style>").unwrap().1;
            let css = css.split("</style>").next().unwrap();
            let (mut pulsing, mut endless) = (0, 0);
            for (sel, body) in record_blocks(css).iter().filter(|b| !b.0.starts_with('@')) {
                for (p, v) in record_decls(body) {
                    if p != "animation" {
                        continue;
                    }
                    for item in record_list(&v) {
                        if item.starts_with("fq-hot ") {
                            assert_eq!(flow_pulses(&item), Some(6), "{sel}: {item}");
                            pulsing += 1;
                        }
                        if item.contains("infinite") {
                            // Only the dashes of the beams run for ever.
                            assert!(item.starts_with("fq-flow "), "{sel}: {item}");
                            endless += 1;
                        }
                    }
                }
            }
            assert_eq!((pulsing, endless), (1, 1));
        }
    }
    /// The texts of a drawn card in order: (x, y, size, letter spacing, anchor, words).
    fn terminal_phone_texts(svg: &str) -> Vec<(f64, f64, f64, f64, String, String)> {
        let attr = |tag: &str, key: &str| -> Option<String> {
            tag.split(&format!(" {key}=\""))
                .nth(1)
                .map(|v| v.split('"').next().unwrap().to_string())
        };
        let body = svg.split_once("</style>").unwrap().1;
        let mut out = Vec::new();
        for part in body.split("<text").skip(1) {
            let (tag, rest) = part.split_once('>').unwrap();
            let words = rest.split("</text>").next().unwrap();
            assert!(!words.contains('<'), "a text with markup inside: {words}");
            let words = words
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"");
            let kick = attr(tag, "class").is_some_and(|c| c.split(' ').any(|c| c == "kick"));
            let size = match attr(tag, "font-size") {
                Some(s) => s.parse().unwrap(),
                None if kick => PHONE_KICK,
                None => panic!("a text without a size: {tag}"),
            };
            out.push((
                attr(tag, "x").unwrap().parse().unwrap(),
                attr(tag, "y").unwrap().parse().unwrap(),
                size,
                if kick { 2.0 } else { 0.0 },
                attr(tag, "text-anchor").unwrap_or_else(|| "start".into()),
                words,
            ));
        }
        out
    }
    /// The height a card states.
    fn terminal_phone_height(svg: &str) -> f64 {
        svg.split(" height=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }
    /// Every line of the card, measured with 0.62 em per character and 8 % to
    /// spare, stays inside the card with 20 px on each side.
    fn terminal_phone_inside(svg: &str) {
        let (w, h) = (f64::from(PHONE_WIDTH), terminal_phone_height(svg));
        let texts = terminal_phone_texts(svg);
        assert!(texts.len() >= 10);
        for (x, y, size, spacing, anchor, words) in texts {
            assert!(size >= PHONE_MIN_FONT, "{words}");
            let len = (0.62 * size + spacing) * words.chars().count() as f64 * 1.08;
            let left = match anchor.as_str() {
                "end" => x - len,
                "middle" => x - len / 2.0,
                _ => x,
            };
            assert!(
                left >= 20.0 && left + len <= w - 20.0,
                "{words}: {left}..{}",
                left + len
            );
            assert!(y - size >= 0.0 && y + 8.0 <= h, "{words}: y {y} of {h}");
        }
    }
    #[test]
    fn a2_phone_cards_pass_the_phone_checks_and_fit() {
        for name in ["terminal", "record", "flow"] {
            check_phone(name);
            let (_, draw) = PHONES.iter().find(|p| p.0 == name).unwrap();
            for dark in [false, true] {
                let s = draw(dark).unwrap();
                terminal_phone_inside(&s);
                // Text is never squeezed, and no static transform scales it down.
                assert!(!s.contains("textLength"), "{name}");
                assert!(!s.contains("transform=\"scale"), "{name}");
                assert!(!s.contains("<script") && !s.contains("href") && !s.contains("<image"));
                assert!(
                    s.chars().all(|c| c.is_ascii() || "·—→×✓".contains(c)),
                    "{name}"
                );
                let h = terminal_phone_height(&s);
                assert!((500.0..=1050.0).contains(&h), "{name}: {h}");
                // Only the passing reading window and the typing cover are hidden
                // by a base rule; every class that moves stops with reduced motion.
                let css = s
                    .split_once("<style>")
                    .unwrap()
                    .1
                    .split("</style>")
                    .next()
                    .unwrap();
                let blocks = record_blocks(css);
                let still = blocks
                    .iter()
                    .find(|b| b.0.contains("prefers-reduced-motion"))
                    .map(|b| b.1.clone())
                    .unwrap();
                for (sel, body) in blocks.iter().filter(|b| !b.0.starts_with('@')) {
                    let decls = record_decls(body);
                    if decls
                        .iter()
                        .any(|(p, v)| record_hides(p, v).is_some_and(|h| h.1))
                    {
                        assert!([".rq-s", ".tq-c"].contains(&sel.as_str()), "{name}: {sel}");
                    }
                    if decls.iter().any(|(p, _)| p.starts_with("animation")) {
                        assert!(
                            still.contains(&format!("{sel},"))
                                || still.contains(&format!("{sel}{{")),
                            "{name}: {sel}"
                        );
                    }
                }
            }
        }
        // The fit checks refuse instead of shrinking.
        assert!(terminal_phone_fits(
            "x".repeat(40).as_str(),
            20.0,
            0.0,
            536.0
        ));
        assert!(!terminal_phone_fits(
            "x".repeat(41).as_str(),
            20.0,
            0.0,
            536.0
        ));
        assert!(
            terminal_phone_line((0.0, 0.0), 20.0, 100.0, "", "a line that is too long").is_err()
        );
        assert!(terminal_phone_kick((0.0, 0.0), 100.0, "", "A KICKER TOO LONG").is_err());
        assert_eq!(
            terminal_phone_wrap("one two three four", 20.0, 150.0).unwrap(),
            ["one two", "three four"]
        );
        assert!(terminal_phone_wrap("unbreakable_line_of_words", 20.0, 130.0).is_err());
    }
    /// The README's commands and lines to look for, read another way: the code
    /// block lines after the fetch, and the code spans of the table's second cells.
    fn terminal_phone_readme() -> (Vec<String>, Vec<Vec<String>>) {
        let runs: Vec<String> = NATIVE
            .lines()
            .skip_while(|l| *l != "cargo fetch --locked")
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .map(str::to_string)
            .collect();
        let looks = NATIVE
            .lines()
            .skip_while(|l| !l.starts_with("| Command | Line to look for |"))
            .skip(2)
            .take_while(|l| l.starts_with('|'))
            .map(|row| {
                let cell = row.split('|').nth(2).unwrap();
                cell.split('`')
                    .skip(1)
                    .step_by(2)
                    .map(str::to_string)
                    .collect()
            })
            .collect();
        (runs, looks)
    }
    /// The words of the badges of a drawn terminal card, one badge line each.
    fn terminal_phone_badges(svg: &str) -> Vec<String> {
        svg.split("class=\"mono b\"")
            .skip(1)
            .map(|p| {
                p.split_once('>')
                    .unwrap()
                    .1
                    .split("</text>")
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }
    #[test]
    fn terminal_phone_card_shows_the_readme_commands_and_lines() {
        let (runs, looks) = terminal_phone_readme();
        assert!(runs.len() >= 3 && runs.iter().all(|r| r.starts_with("cargo run ")));
        assert_eq!(looks.len(), runs.len());
        let checks = terminal_checks(NATIVE).unwrap();
        for dark in [false, true] {
            let s = terminal_phone(dark).unwrap();
            assert!(s.contains(&format!(">{} COMMANDS · WHAT TO LOOK FOR<", runs.len())));
            assert!(
                s.contains(">Drawn, not a recording<")
                    && s.contains("<title id=\"t\">Run it yourself — drawn, not a recording<")
            );
            assert!(s.contains(">this README's code block and table<"));
            // Every line to look for, in the README's order, broken only at spaces.
            let drawn = terminal_phone_badges(&s).join(" ");
            let want: Vec<String> = looks.iter().flatten().cloned().collect();
            assert_eq!(drawn, want.join(" "));
            let long = want.iter().map(|l| l.len()).max().unwrap();
            assert!(long > 40, "a line long enough to need breaking");
            assert!(terminal_phone_badges(&s).iter().all(|l| l.len() <= 36));
            for (c, run) in checks.iter().zip(&runs) {
                assert!(run.contains(c.key.as_str()));
                assert!(s.contains(&format!(">{}</text>", xml(&c.key))), "{run}");
                // The full command is only checked, never drawn.
                assert!(!s.contains(run.as_str()));
            }
            assert!(s.contains(">with</text>") && s.matches("l7 7 13-15").count() == runs.len());
            assert!(!s.contains(">$") && !s.contains("cargo run"));
        }
        // Drawn from other checks, it shows them and none of the README's.
        let other = vec![
            TerminalCheck {
                key: "--other".into(),
                command: "cargo run -- --other".into(),
                look: vec![(
                    String::new(),
                    "ALPHA_LINE=PASS BETA_LINE=NOT_MEASURED GAMMA=7".into(),
                )],
            },
            TerminalCheck {
                key: "second".into(),
                command: "cargo run -- second".into(),
                look: vec![
                    (String::new(), "ONE=1".into()),
                    ("with".into(), "TWO=2".into()),
                ],
            },
        ];
        let s = terminal_phone_draw(false, &other).unwrap();
        assert!(s.contains(">--other</text>") && s.contains(">second</text>"));
        assert!(s.contains(">2 COMMANDS · WHAT TO LOOK FOR<"));
        assert_eq!(
            terminal_phone_badges(&s).join(" "),
            "ALPHA_LINE=PASS BETA_LINE=NOT_MEASURED GAMMA=7 ONE=1 TWO=2"
        );
        for stale in ["--literal", "GEL_VERIFY_ALL", "5 COMMANDS"] {
            assert!(!s.contains(stale), "{stale}");
        }
        terminal_phone_inside(&s);
        // A line that cannot be broken to fit, or a name too long, is refused.
        let mut bad = other.clone();
        bad[0].look[0].1 = "X".repeat(60);
        assert!(terminal_phone_draw(false, &bad).is_err());
        let mut bad = other.clone();
        bad[1].key = "a-command-name-far-too-long-to-fit".into();
        assert!(terminal_phone_draw(false, &bad).is_err());
        assert!(terminal_phone_draw(false, &[]).is_err());
    }
    /// The recorded runs counted another way: every v2 run file, how many hold
    /// the exact verdict line, and the record size the first one states.
    fn record_phone_counted() -> (usize, usize, usize, usize) {
        let (mut n, mut ok, mut record, mut frames) = (0, 0, 0, 0);
        let number = |text: &str, key: &str| -> usize {
            let at = text.find(key).unwrap() + key.len();
            text[at..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };
        for e in fs::read_dir(RECORD_DIR).unwrap() {
            let name = e.unwrap().file_name().into_string().unwrap();
            if !(name.starts_with("v2-") && name.ends_with(".txt")) {
                continue;
            }
            let text = fs::read_to_string(format!("{RECORD_DIR}/{name}")).unwrap();
            n += 1;
            ok += usize::from(text.lines().any(|l| l.trim() == Q8_PASS));
            record = number(&text, "canonical_record_bytes=");
            frames = number(&text, "materialized_four_frame_bytes=");
        }
        (n, ok, record, frames)
    }
    #[test]
    fn record_phone_card_shows_the_recorded_runs() {
        let (n, ok, record, frames) = record_phone_counted();
        assert!(n >= 1 && ok == n);
        for dark in [false, true] {
            let s = record_phone(dark).unwrap();
            for want in [
                format!(">{ok} of {n}<"),
                ">recorded runs<".to_string(),
                ">end with<".to_string(),
                format!(">{} B<", grouped(frames)),
                ">reference layout<".to_string(),
                ">used for checking<".to_string(),
                ">Q8_QUAD_EXACT=PASS<".to_string(),
                ">SEMANTIC_ACCURACY=NOT_MEASURED<".to_string(),
                ">score vs a separate written-out copy<".to_string(),
            ] {
                assert!(s.contains(&want), "{want}");
            }
            for i in 1..=4 {
                assert!(s.contains(&format!(">view {i}<")));
            }
            assert!(!s.contains("view 5"));
            // Every view has its check; what the check means is said once.
            assert_eq!(s.matches("l4 4 8-8").count(), 4);
            assert_eq!(s.matches(">score = copy's score<").count(), 1);
            // The verdict comes first, then the views, then the layout.
            let at = |needle: &str| s.find(needle).unwrap();
            assert!(at(">Q8_QUAD_EXACT=PASS<") < at(">view 1<"));
            assert!(at(">view 4<") < at(">score = copy's score<"));
            assert!(at(">score = copy's score<") < at(">reference layout<"));
            let texts = terminal_phone_texts(&s);
            let limits: Vec<String> = texts
                .iter()
                .skip_while(|t| t.5 != "used for checking")
                .skip(1)
                .map(|t| t.5.clone())
                .collect();
            assert_eq!(
                limits.join(" "),
                "Record payload only; process memory is additional."
            );
            // On a phone the Q8 card sits right above and already shows the
            // record, its sizes and what four views are not: none of it again.
            let drawn: Vec<String> = texts.into_iter().map(|t| t.5).collect();
            let drawn = drawn.join(" | ");
            for twice in [
                format!("{} B", grouped(record)),
                format!("{} B", grouped(4 * record)),
                "one-byte values".to_string(),
                "activity mask".to_string(),
                "independent memories".to_string(),
                "stored".to_string(),
                "SIZE, TO SCALE".to_string(),
            ] {
                assert!(!drawn.contains(&twice), "{twice}");
            }
            assert!(!s.contains("<rect class=\"size\""));
            assert!(s.contains(&record_text(&Q8Runs::read().unwrap())));
            terminal_phone_inside(&s);
        }
        // Drawn from other runs, it shows their numbers.
        let other = Q8Runs {
            n: 3,
            ok: 2,
            record: 2000,
            packed: 8000,
            frames: 9000,
        };
        let s = record_phone_draw(false, &other).unwrap();
        terminal_phone_inside(&s);
        for want in [">2 of 3<", ">9,000 B<"] {
            assert!(s.contains(want), "{want}");
        }
        for stale in ["48 of 48", "8,192 B"] {
            assert!(!s.contains(stale), "{stale}");
        }
        // A count too wide to share its line puts the words under it.
        let wide = Q8Runs {
            n: 1000,
            ok: 1000,
            ..other
        };
        let s = record_phone_draw(false, &wide).unwrap();
        terminal_phone_inside(&s);
        let texts = terminal_phone_texts(&s);
        let y_of = |w: &str| texts.iter().find(|t| t.5 == w).unwrap().1;
        assert!(y_of("recorded runs") > y_of("1000 of 1000"));
        assert!(y_of("end with") < y_of("Q8_QUAD_EXACT=PASS") - 20.0);
    }
    #[test]
    fn flow_phone_card_says_what_the_flow_card_says() {
        for dark in [false, true] {
            let wide = scene(dark);
            // The wide card's words, read another way: by their rows.
            let row = |y: &str| -> Vec<String> {
                wide.split(&format!(" y=\"{y}\" text-anchor=\"middle\""))
                    .skip(1)
                    .map(|p| {
                        p.split_once('>')
                            .unwrap()
                            .1
                            .split("</text>")
                            .next()
                            .unwrap()
                            .to_string()
                    })
                    .collect()
            };
            let (heads, subs) = (row("318"), row("340"));
            assert_eq!((heads.len(), subs.len()), (4, 4));
            let f = flow_phone_read(&wide).unwrap();
            assert_eq!(
                f.steps.iter().map(|s| s.0.clone()).collect::<Vec<_>>(),
                heads
            );
            assert_eq!(
                f.steps.iter().map(|s| s.1.clone()).collect::<Vec<_>>(),
                subs
            );
            let s = flow_phone(dark).unwrap();
            let texts: Vec<String> = terminal_phone_texts(&s).into_iter().map(|t| t.5).collect();
            let all = texts.join(" ");
            // Every word of the wide card, in order; a verdict after " · " on its own line.
            for words in heads.iter().chain(&subs) {
                let words = words.replace(" · ", " ");
                assert!(all.contains(words.as_str()), "{words}");
            }
            assert!(texts.iter().any(|t| t == "PASS") && texts.iter().any(|t| t == "REFUSED"));
            // A verdict shares its name's line; the verdicts stand in one column.
            let drawn = terminal_phone_texts(&s);
            let of = |w: &str| drawn.iter().find(|t| t.5 == w).unwrap();
            assert_eq!(of("PASS").1, of("Reopened").1);
            assert_eq!(of("REFUSED").1, of("1 byte changed").1);
            assert_eq!(of("PASS").0, of("REFUSED").0);
            for want in [
                "Drawn, not recorded",
                "not a timing measurement",
                "bytes 67..105",
                "SHA-256 you keep",
                "REFUSED",
                "PASS",
            ] {
                assert!(all.contains(want), "{want}");
            }
            // The two branches: one beam down to the reopened copy, one around it.
            assert_eq!(s.matches("stroke-dasharray=\"10 8\"").count(), 3);
        }
        // Drawn from other words, it shows them.
        let other = FlowPhone {
            steps: vec![
                ("First step".into(), "line one".into()),
                ("Second step".into(), "line two".into()),
                ("Third · OK".into(), "line three".into()),
                ("Fourth · NO".into(), "line four".into()),
            ],
            note: "Drawn: a note · and its limit".into(),
        };
        let s = flow_phone_draw(false, &other).unwrap();
        for want in [
            ">First step<",
            ">line four<",
            ">Fourth<",
            ">NO<",
            ">Drawn: a note<",
            ">and its limit<",
        ] {
            assert!(s.contains(want), "{want}");
        }
        assert!(!s.contains("67..105") && !s.contains(">Your document<"));
        terminal_phone_inside(&s);
        // A wide card that lost a step, or words too long for a row, are refused.
        let wide = scene(false);
        let cut = wide.replacen(
            r#"font-size="18" font-weight="700">"#,
            r#"font-size="18">"#,
            1,
        );
        assert!(flow_phone_read(&cut).is_err());
        assert!(flow_phone_read(&wide.replace(">Drawn, not recorded", ">Drawn")).is_err());
        let mut long = other.clone();
        long.steps[1].1 =
            "a line under the step that runs on and on and on, and on and on and on, and on and on and on, and on".into();
        assert!(flow_phone_draw(false, &long).is_err());
    }
    // ===== PHONE TEST a2 END =====

    // ===== PHONE TEST a3 BEGIN =====
    /// Every text line of a phone card: its characters, left and right edge as
    /// 0.62 × font size per character with 8 % slack (the kicker's letter
    /// spacing added), its baseline and its size.
    fn speed_phone_texts(svg: &str) -> Vec<(String, f64, f64, f64, f64)> {
        let attr = |head: &str, key: &str| -> Option<String> {
            let rest = head.split(&format!(" {key}=\"")).nth(1)?;
            Some(rest.split('"').next()?.to_string())
        };
        let body = svg.split_once("</defs>").unwrap().1;
        let mut out = Vec::new();
        for part in body.split("<text").skip(1) {
            let (head, rest) = part.split_once('>').unwrap();
            let raw = rest.split("</text>").next().unwrap();
            let mut text = String::new();
            let mut inside = false;
            for ch in raw.chars() {
                match ch {
                    '<' => inside = true,
                    '>' => inside = false,
                    _ if !inside => text.push(ch),
                    _ => {}
                }
            }
            for (entity, ch) in [
                ("&lt;", "<"),
                ("&gt;", ">"),
                ("&quot;", "\""),
                ("&amp;", "&"),
            ] {
                text = text.replace(entity, ch);
            }
            let kick = attr(head, "class").is_some_and(|c| c.split(' ').any(|c| c == "kick"));
            let size: f64 = match attr(head, "font-size") {
                Some(s) => s.parse().unwrap(),
                None if kick => 20.0,
                None => panic!("{text}: no font size"),
            };
            let spacing = if kick { 2.0 } else { 0.0 };
            let w = speed_phone_width(&text, size) + spacing * text.chars().count() as f64 * 1.08;
            let x: f64 = attr(head, "x").unwrap().parse().unwrap();
            let y: f64 = attr(head, "y").unwrap().parse().unwrap();
            let (l, r) = match attr(head, "text-anchor").as_deref() {
                Some("middle") => (x - w / 2.0, x + w / 2.0),
                Some("end") => (x - w, x),
                _ => (x, x + w),
            };
            out.push((text, l, r, y, size));
        }
        out
    }
    /// All the drawn words of a card, line after line, one space apart.
    fn speed_phone_said(svg: &str) -> String {
        let lines: Vec<String> = speed_phone_texts(svg).into_iter().map(|t| t.0).collect();
        format!(" {} ", lines.join(" "))
    }
    /// Every line fits the card: inside 20..580 across, its baseline at least
    /// its size from the top and 12 px above the bottom; no squeezing and no
    /// static shrinking.
    fn speed_phone_fits(svg: &str) {
        let height: f64 = svg
            .split_once(r#"viewBox="0 0 600 "#)
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let lines = speed_phone_texts(svg);
        assert!(lines.len() >= 8);
        for (text, l, r, y, size) in lines {
            assert!(l >= 20.0 && r <= 580.0, "{text}: {l:.0} to {r:.0}");
            assert!(
                y >= size && y <= height - 12.0,
                "{text}: at {y} of {height}"
            );
        }
        assert!(!svg.contains("textLength") && !svg.contains("transform=\"scale"));
    }
    /// Motion that can stand still: every animated class is stilled under
    /// reduced motion, and no base rule hides or shrinks a thing.
    fn speed_phone_motion(svg: &str) {
        let style = svg.split_once("<style>").unwrap().1;
        let style = style.split_once("</style>").unwrap().0;
        let (main, reduced) = style
            .split_once("@media (prefers-reduced-motion:reduce){")
            .unwrap();
        let (stilled, _) = reduced.split_once("{animation:none}").unwrap();
        let stilled: Vec<&str> = stilled.split(',').collect();
        let mut animated = 0;
        for rule in main.split('}') {
            if rule.trim_start().starts_with("@keyframes") || rule.contains("{from") {
                continue;
            }
            let Some((selector, decl)) = rule.split_once('{') else {
                continue;
            };
            if decl.starts_with("from") || decl.starts_with("to") || selector.starts_with("to") {
                continue;
            }
            assert!(
                !decl.contains("opacity") && !decl.contains("scale") && !decl.contains("hidden"),
                "{rule}"
            );
            if decl.contains("animation") {
                animated += 1;
                assert!(stilled.contains(&selector.trim()), "{selector} not stilled");
            }
        }
        assert!(animated >= 3, "{animated}");
        // Every class the body uses with the card's prefix is a styled one.
        let body = svg.split_once("</defs>").unwrap().1;
        for part in body.split("class=\"").skip(1) {
            for class in part.split('"').next().unwrap().split(' ') {
                if class.contains("ph-") {
                    assert!(stilled.contains(&format!(".{class}").as_str()), "{class}");
                }
            }
        }
    }
    #[test]
    fn speed_rerun_bars_wall_phone_cards_pass_the_phone_checks() {
        for name in ["speed", "rerun", "bars", "wall"] {
            check_phone(name);
            let (_, draw) = PHONES.iter().find(|p| p.0 == name).unwrap();
            for dark in [false, true] {
                let s = draw(dark).unwrap();
                speed_phone_fits(&s);
                speed_phone_motion(&s);
            }
        }
    }
    /// The timing set read a second way: per program directory its SPEED_AB
    /// lines and its verdict, sides from the printed aa and gain alone.
    fn speed_phone_second_count() -> (usize, usize, usize, usize, [usize; 3]) {
        let (mut programs, mut timed, mut same, mut cases) = (0, 0, 0, 0);
        let mut sides = [0; 3];
        for entry in fs::read_dir(SPEED_DIR).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_dir() {
                continue;
            }
            programs += 1;
            let Ok(text) = fs::read_to_string(path.join("summary.txt")) else {
                continue;
            };
            timed += 1;
            same += usize::from(text.contains("SPEED_AB_VERDICT=SAME_OUTPUT"));
            for line in text.lines().filter(|l| l.starts_with("SPEED_AB case=")) {
                let field = |k: &str| -> f64 {
                    line.split(' ')
                        .find_map(|w| w.strip_prefix(k))
                        .unwrap()
                        .parse()
                        .unwrap()
                };
                let (aa, gain) = (field("aa="), field("gain="));
                cases += 1;
                let (lo, hi) = (aa.min(1.0 / aa), aa.max(1.0 / aa));
                sides[if gain > hi {
                    0
                } else if gain < lo {
                    2
                } else {
                    1
                }] += 1;
            }
        }
        (programs, timed, same, cases, sides)
    }
    #[test]
    fn speed_phone_shows_the_timing_set() {
        let (programs, timed, same, cases, [f, i, s]) = speed_phone_second_count();
        assert!(cases > 0 && timed > 0);
        let svg = speed_phone(false).unwrap();
        let said = speed_phone_said(&svg);
        for want in [
            format!(" {cases} timed cases "),
            format!(" in {timed} of {programs} timing programs "),
            format!(" {f} faster {i} inside {s} slower "),
            format!(" {same} of {timed} same output digest in A, A2 and B "),
            format!(" {} not built in v0.6.1: no timing. ", programs - timed),
            " A case counts as faster or slower only beyond its own A/A band. ".to_string(),
            " Ratios of different programs are not comparable with each other. ".to_string(),
            " The output digests, not the timings, are the claim. ".to_string(),
            " Not pinned to cores. ".to_string(),
            format!(" B: main at {SPEED_B} "),
            " set 2026-10-20-against-v0.6.1 ".to_string(),
            " Cases are grouped by timing program. Per-case results: docs/speed-results "
                .to_string(),
        ] {
            assert!(said.contains(&want), "{want} not in {said}");
        }
        // The gains of the SHA-256 program read a second way, from its lines.
        // Two digits each: the same rounding as the wide card's description.
        let summary = fs::read_to_string(format!("{SPEED_DIR}/speed_sha256/summary.txt")).unwrap();
        let gains: Vec<String> = summary
            .lines()
            .filter(|l| l.starts_with("SPEED_AB case="))
            .map(|l| l.split(" gain=").nth(1).unwrap().split(' ').next().unwrap())
            .map(|g| format!("{:.2}", g.parse::<f64>().unwrap()))
            .collect();
        let sha = format!(
            " SHA-256 alone at {}: v0.6.1 time divided by the new time. ",
            gains.join(" and ")
        );
        assert!(said.contains(&sha), "{sha} not in {said}");
        let desc = svg.split_once("<desc id=\"d\">").unwrap().1;
        let desc = desc.split_once("</desc>").unwrap().0;
        for want in [
            "timing set 2026-10-20-against-v0.6.1",
            "Cases are grouped by timing program; per-case results are in docs/speed-results/2026-10-20-against-v0.6.1.",
            sha.trim(),
        ] {
            assert!(desc.contains(want), "{want} not in {desc}");
        }
        // Each differing digest with its reason, read a second way: the set
        // README's sentence on that program, from "because" to its full stop.
        let readme = fs::read_to_string(format!("{SPEED_DIR}/README.md")).unwrap();
        let flat = readme.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut differ = 0;
        for entry in fs::read_dir(SPEED_DIR).unwrap() {
            let path = entry.unwrap().path();
            let Ok(text) = fs::read_to_string(path.join("summary.txt")) else {
                continue;
            };
            if !text.contains("SPEED_AB_VERDICT=DIFFERENT_OUTPUT") {
                continue;
            }
            differ += 1;
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let from = format!("`{name}` prints `DIFFERENT_OUTPUT` because ");
            let reason = flat.split_once(&from).unwrap().1;
            let reason = &reason[..reason.find(". ").unwrap() + 1];
            let want = format!(" 1 differs: {reason} ");
            assert!(said.contains(&want), "{want} not in {said}");
            assert!(!said.contains(&name), "{name}");
        }
        assert_eq!(differ, timed - same);
        assert!(differ > 0 && said.contains(" same output digest in A, A2 and B 1 differs: "));
        assert_eq!(said.matches(" 1 differs").count(), differ);
        // The machine and the load as the set states them, then the limit.
        let set = SpeedSet::read().unwrap();
        let run = format!(
            " Machine: {}. Load: {}. Not pinned to cores. ",
            set.machine, set.load
        );
        assert!(said.contains(&run), "{run} not in {said}");
        // The rows are the wide card's: neither the picture nor its description
        // speaks of them.
        for rows in ["One row", "one row", "log scale", "median"] {
            assert!(!svg.contains(rows), "{rows}");
        }
        assert!(svg.contains(&xml(&speed_phone_text(&set))));
    }
    /// The widths of the segments of the speed card's bar, by fill.
    fn speed_phone_bar(svg: &str, fill: &str) -> Vec<f64> {
        svg.split("<rect ")
            .skip(1)
            .map(|r| r.split("/>").next().unwrap())
            .filter(|r| r.contains(&format!("fill=\"{fill}\"")) && r.contains("height=\"36\""))
            .map(|r| {
                let w = r.split_once("width=\"").unwrap().1;
                w.split('"').next().unwrap().parse().unwrap()
            })
            .collect()
    }
    fn speed_phone_case(name: &str, aa: f64, gain: f64) -> SpeedCase {
        SpeedCase {
            name: name.to_string(),
            aa,
            gain,
            runs: 5,
        }
    }
    #[test]
    fn speed_phone_draws_the_set_it_is_given() {
        let example = |name: &str, cases: Vec<SpeedCase>, same: bool| SpeedExample {
            name: name.to_string(),
            timed: Some((cases, same)),
            note: None,
        };
        let s = SpeedSet {
            machine: "Box 7".to_string(),
            load: "light".to_string(),
            examples: vec![
                example(
                    "p",
                    vec![
                        speed_phone_case("a", 1.01, 3.0),
                        speed_phone_case("b", 1.02, 5.0),
                        speed_phone_case("c", 1.2, 1.1),
                    ],
                    true,
                ),
                example("o", vec![speed_phone_case("o", 1.01, 2.0)], true),
                example(
                    "q",
                    vec![
                        speed_phone_case("d", 1.05, 0.3),
                        speed_phone_case("e", 1.05, 0.5),
                    ],
                    false,
                ),
                SpeedExample {
                    name: "r".to_string(),
                    timed: None,
                    note: Some("r has only a manifest.".to_string()),
                },
                SpeedExample {
                    name: "s".to_string(),
                    timed: None,
                    note: Some("s has only a manifest.".to_string()),
                },
            ],
        };
        for dark in [false, true] {
            let svg = speed_phone_draw(dark, &s);
            speed_phone_fits(&svg);
            let said = speed_phone_said(&svg);
            for want in [
                " 6 timed cases ",
                " in 3 of 5 timing programs ",
                " 3 faster 1 inside 2 slower ",
                " 2 of 3 same output digest ",
                " 2 not built in v0.6.1: no timing. ",
                " Machine: Box 7. Load: light. Not pinned to cores. ",
            ] {
                assert!(said.contains(want), "{want} not in {said}");
            }
            // q differs and has no note: the card says so and gives no reason.
            assert!(said.contains(" same output digest in A, A2 and B 1 differs 2 not built "));
            // One bar of the six cases: each side as wide as its count.
            let t = theme(dark);
            for (fill, n) in [
                (t.blue[1], 3.0),
                (speed_phone_grey(dark), 1.0),
                (t.red[1], 2.0),
            ] {
                assert_eq!(speed_phone_bar(&svg, fill), [90.0 * n], "{fill}");
            }
        }
        // With the README's note, its reason follows; the program is not named.
        let mut s = s;
        s.examples[2].note = Some("q prints DIFFERENT_OUTPUT because it reads the clock.".into());
        let svg = speed_phone_draw(false, &s);
        speed_phone_fits(&svg);
        let said = speed_phone_said(&svg);
        assert!(
            said.contains(
                " same output digest in A, A2 and B 1 differs: it reads the clock. 2 not built "
            ),
            "{said}"
        );
        assert!(!said.contains("DIFFERENT_OUTPUT"));
        // No SHA-256 program, no sentence; with one, the numbers it is given.
        assert!(!said.contains("SHA-256"));
        s.examples.push(example(
            "speed_sha256",
            vec![
                speed_phone_case("x", 1.01, 0.51),
                speed_phone_case("y", 1.01, 0.62),
            ],
            true,
        ));
        let svg = speed_phone_draw(false, &s);
        speed_phone_fits(&svg);
        let sha = " SHA-256 alone at 0.51 and 0.62: v0.6.1 time divided by the new time. ";
        assert!(speed_phone_said(&svg).contains(sha));
        assert!(svg.contains(sha.trim()));
    }
    /// The recorded set and its re-run read a second way: case by case from
    /// the printed lines, keyed by program and case.
    fn rerun_phone_second_count() -> ([usize; 3], [usize; 3], usize, usize, [f64; 2]) {
        let read = |dir: &str| {
            let mut v: Vec<(String, f64, f64)> = Vec::new();
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let Ok(text) = fs::read_to_string(path.join("summary.txt")) else {
                    continue;
                };
                let program = path.file_name().unwrap().to_string_lossy().into_owned();
                for line in text.lines().filter(|l| l.starts_with("SPEED_AB case=")) {
                    let field = |k: &str| line.split(' ').find_map(|w| w.strip_prefix(k)).unwrap();
                    v.push((
                        format!("{program}/{}", field("case=")),
                        field("aa=").parse().unwrap(),
                        field("gain=").parse().unwrap(),
                    ));
                }
            }
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        };
        let (old, new) = (read(SPEED_DIR), read(RERUN_DIR));
        assert_eq!(old.len(), new.len());
        let side = |aa: f64, gain: f64| {
            if gain > aa.max(1.0 / aa) {
                0
            } else if gain < aa.min(1.0 / aa) {
                2
            } else {
                1
            }
        };
        let (mut s0, mut s1, mut again, mut above) = ([0; 3], [0; 3], 0, 0);
        for (a, b) in old.iter().zip(&new) {
            assert_eq!(a.0, b.0);
            s0[side(a.1, a.2)] += 1;
            s1[side(b.1, b.2)] += 1;
            if side(a.1, a.2) == 0 {
                again += usize::from(side(b.1, b.2) == 0);
                above += usize::from(b.2 > 1.0);
            }
        }
        let median = |v: &[(String, f64, f64)]| {
            let mut n: Vec<f64> = v.iter().map(|c| (c.1 - 1.0).abs()).collect();
            n.sort_by(f64::total_cmp);
            let m = n.len() / 2;
            if n.len() % 2 == 1 {
                n[m]
            } else {
                (n[m - 1] + n[m]) / 2.0
            }
        };
        (s0, s1, again, above, [median(&old), median(&new)])
    }
    #[test]
    fn rerun_phone_shows_the_two_timing_records() {
        let (s0, s1, again, above, noise) = rerun_phone_second_count();
        let cases: usize = s0.iter().sum();
        let svg = rerun_phone(true).unwrap();
        let said = speed_phone_said(&svg);
        // The dates the two directory names start with.
        let date = |dir: &str| dir.rsplit('/').next().unwrap()[..10].to_string();
        let (d0, d1) = (date(SPEED_DIR), date(RERUN_DIR.rsplit_once('/').unwrap().0));
        for want in [
            format!(" RE-RUN · {d0} → {d1} "),
            format!(" The same {cases} cases timed twice against v0.6.1 "),
            " each case against its own A/A band ".to_string(),
            format!(" {again} of {} faster again ", s0[0]),
            format!(" faster inside slower {d0} {} {} {} ", s0[0], s0[1], s0[2]),
            format!(" {d1} {} {} {} ", s1[0], s1[1], s1[2]),
            format!(" {above} of the {} with a ratio above 1 ", s0[0]),
            format!(" {} → {} ", rerun_number(noise[0]), rerun_number(noise[1])),
            " One machine, not pinned to cores ".to_string(),
            " A, A2: two checkouts of v0.6.1 ".to_string(),
        ] {
            assert!(said.contains(&want), "{want} not in {said}");
        }
    }
    fn rerun_phone_mutant() -> RerunData {
        RerunData {
            base: "v9.9".to_string(),
            runs: [
                RerunRun {
                    date: "2031-01-02".to_string(),
                    sides: [7, 1, 3],
                    noise: 0.25,
                },
                RerunRun {
                    date: "2031-02-03".to_string(),
                    sides: [2, 6, 3],
                    noise: 0.0125,
                },
            ],
            cases: 11,
            again: 2,
            above_one: 5,
        }
    }
    #[test]
    fn rerun_phone_draws_the_runs_it_is_given() {
        let d = rerun_phone_mutant();
        let mut lopsided = rerun_phone_mutant();
        lopsided.runs[0].sides = [1, 1, 9];
        lopsided.runs[1].sides = [9, 1, 1];
        lopsided.again = 1;
        lopsided.above_one = 1;
        for (dark, data) in [(false, &d), (true, &lopsided)] {
            speed_phone_fits(&rerun_phone_draw(dark, data).unwrap());
        }
        let said = speed_phone_said(&rerun_phone_draw(false, &d).unwrap());
        for want in [
            " RE-RUN · 2031-01-02 → 2031-02-03 ",
            " The same 11 cases timed twice against v9.9 ",
            " 2 of 7 faster again ",
            " faster inside slower 2031-01-02 7 1 3 2031-02-03 2 6 3 ",
            " 5 of the 7 with a ratio above 1 ",
            " 0.25 → 0.0125 ",
            " A, A2: two checkouts of v9.9 ",
        ] {
            assert!(said.contains(want), "{want} not in {said}");
        }
        // Data that does not add up is refused, not drawn.
        let mut bad = rerun_phone_mutant();
        bad.again = 8;
        assert!(rerun_phone_draw(false, &bad).is_err());
    }
    /// The mutation matrix read a second way: per format its rows, rejections,
    /// acceptances, matches and distinct cases.
    fn bars_phone_second_count() -> Vec<(String, usize, usize, usize, usize)> {
        let text = fs::read_to_string("docs/evidence-mutation/matrix-r1.txt").unwrap();
        let mut out: std::collections::BTreeMap<String, (Vec<String>, usize, usize, usize)> =
            Default::default();
        for line in text.lines().skip(1) {
            let f: Vec<&str> = line.split('\t').collect();
            let e = out.entry(f[0].to_string()).or_default();
            if !e.0.iter().any(|c| c == f[1]) {
                e.0.push(f[1].to_string());
            }
            match f[5] {
                "ACCEPT" => e.2 += 1,
                _ => e.1 += 1,
            }
            e.3 += usize::from(f[5] == f[6] && f[7] == "PASS");
        }
        out.into_iter()
            .map(|(k, v)| (k, v.0.len(), v.1, v.2, v.3))
            .collect()
    }
    #[test]
    fn bars_phone_shows_the_committed_matrix() {
        let formats = bars_phone_second_count();
        let total = |i: usize| -> usize { formats.iter().map(|f| [f.1, f.2, f.3, f.4][i]).sum() };
        let (mutants, rejected, accepted, matched) = (total(0), total(1), total(2), total(3));
        let svg = bars_phone(false).unwrap();
        let said = speed_phone_said(&svg);
        let mut wants = vec![
            format!(" {matched}/{} rows matched their expected verdict ", rejected + accepted),
            format!(" rejected: {rejected} accepted as documented: {accepted} "),
            format!(
                " {mutants} byte-level mutants of the {} public formats, each read with the original pin and with a recomputed one ",
                formats.len()
            ),
            " committed report; xtask verify requires an identical rerun ".to_string(),
        ];
        for (name, m, r, a, _) in &formats {
            wants.push(format!(" {name} {m} mutants {r} rejected · {a} accepted "));
        }
        for want in wants {
            assert!(said.contains(&want), "{want} not in {said}");
        }
    }
    fn bars_phone_mutant() -> Evidence {
        let format = |name: &str, mutants, rejected, accepted, matched| FormatCount {
            name: name.to_string(),
            mutants,
            rejected,
            accepted,
            matched,
        };
        Evidence {
            rows: vec![
                ("Q1".to_string(), vec!["t_a".to_string()]),
                ("Q2".to_string(), vec!["t_b".to_string(), "t_c".to_string()]),
                ("Q3".to_string(), vec!["t_d".to_string()]),
            ],
            unix_only: vec!["t_c".to_string(), "t_d".to_string()],
            formats: vec![format("AAA", 5, 9, 1, 10), format("BB", 2, 3, 0, 2)],
        }
    }
    #[test]
    fn bars_phone_draws_the_counts_it_is_given() {
        let e = bars_phone_mutant();
        for dark in [false, true] {
            let svg = bars_phone_draw(dark, &e);
            speed_phone_fits(&svg);
            let said = speed_phone_said(&svg);
            for want in [
                " 12/13 rows matched ",
                " AAA 5 mutants 9 rejected · 1 accepted ",
                " BB 2 mutants 3 rejected · 0 accepted ",
                " rejected: 12 accepted as documented: 1 ",
                " 7 byte-level mutants of the 2 public formats, ",
            ] {
                assert!(said.contains(want), "{want} not in {said}");
            }
            // A miss is drawn in red, not in the colour of a pass.
            let t = theme(dark);
            assert!(
                svg.contains(&format!("fill:{}\">12/13", t.red[1])),
                "{dark}"
            );
        }
    }
    #[test]
    fn wall_phone_shows_the_property_map() {
        // Rows of the map a second way: a first cell of P and digits.
        let map = fs::read_to_string("docs/PROPERTY-TESTS.md").unwrap();
        let ci = fs::read_to_string("xtask/src/ci_evidence.rs").unwrap();
        let exclusions = ci
            .split("const EXCLUSIONS")
            .nth(1)
            .unwrap()
            .split("];")
            .next()
            .unwrap();
        let (mut n, mut partial) = (0, 0);
        let mut ids = Vec::new();
        for line in map.lines() {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let id = cells.get(1).copied().unwrap_or("");
            if id.len() > 1 && id.starts_with('P') && id[1..].bytes().all(|b| b.is_ascii_digit()) {
                n += 1;
                ids.push(id.to_string());
                let tests = cells[cells.len() - 2];
                let unix = tests
                    .split('`')
                    .skip(1)
                    .step_by(2)
                    .any(|t| exclusions.contains(&format!("\"{t}\"")));
                partial += usize::from(unix);
            }
        }
        assert!(n > 0);
        let svg = wall_phone(false).unwrap();
        let said = speed_phone_said(&svg);
        for want in [
            format!(" PROPERTY MAP × CI PLATFORMS {n} properties × 3 platforms "),
            format!(" one cube per property, {} to {} ", ids[0], ids[n - 1]),
            format!(" {} cells: every mapped test required ", 3 * n - partial),
            format!(
                " {partial} Windows cells: declared Unix-only tests exempt, the rest required "
            ),
            " the rule, not one run: read each run's CI report ".to_string(),
        ] {
            assert!(said.contains(&want), "{want} not in {said}");
        }
        assert_eq!(svg.matches(r#"<g class="wph-u""#).count(), 3 * n);
        let amber = format!(r#"fill="{}""#, theme(false).amber[1]);
        // The legend swatch and one facing side per exempt cell.
        assert_eq!(svg.matches(&amber).count(), partial + 1);
    }
    #[test]
    fn wall_phone_draws_the_map_it_is_given() {
        let e = bars_phone_mutant();
        for dark in [false, true] {
            let svg = wall_phone_draw(dark, &e);
            speed_phone_fits(&svg);
            let said = speed_phone_said(&svg);
            for want in [
                " 3 properties × 3 platforms ",
                " one cube per property, Q1 to Q3 ",
                " 7 cells: every mapped test required ",
                " 2 Windows cells: declared ",
            ] {
                assert!(said.contains(want), "{want} not in {said}");
            }
            assert_eq!(svg.matches(r#"<g class="wph-u""#).count(), 9);
        }
    }
    #[test]
    fn a_linux_only_test_in_the_property_map_is_refused() {
        let rows = vec![("P1".to_string(), vec!["mapped".to_string()])];
        // Unix tests come back; a Linux test outside the map is left out, as
        // the cards draw only Windows cells as exempt.
        let list = r#"("mapped", "unix", "why"), ("free", "linux", "why")"#;
        assert_eq!(evidence_exclusions(list, &rows).unwrap(), ["mapped"]);
        // In the map, its macOS cell would be drawn as required.
        let list = r#"("free", "unix", "why"), ("mapped", "linux", "why")"#;
        let e = evidence_exclusions(list, &rows).unwrap_err().to_string();
        assert!(e.contains("mapped runs on Linux only"), "{e}");
        // What the list says today passes: the cards draw it as it is.
        Evidence::read().unwrap();
        // A list of another shape is refused.
        for bad in [
            r#"("t", "unix")"#,
            r#"("t", "windows", "why")"#,
            r#"("a b", "unix", "why")"#,
            r#"("", "linux", "why")"#,
        ] {
            assert!(evidence_exclusions(bad, &rows).is_err(), "{bad}");
        }
    }
    // ===== PHONE TEST a3 END =====

    // ===== PHONE TEST a4 BEGIN =====
    /// A drawn line of a phone card: where it stands, its size, its anchor, the
    /// letter spacing of its class and its text with the entities read back.
    struct PledgesPhoneText {
        x: f64,
        y: f64,
        size: f64,
        anchor: String,
        spacing: f64,
        text: String,
    }
    fn pledges_phone_attr(tag: &str, key: &str) -> Option<String> {
        let rest = tag.split_once(&format!(" {key}=\""))?.1;
        Some(rest.split_once('"')?.0.to_string())
    }
    fn pledges_phone_num(tag: &str, key: &str) -> f64 {
        pledges_phone_attr(tag, key)
            .unwrap_or_else(|| panic!("no {key} in {tag}"))
            .parse()
            .unwrap()
    }
    fn pledges_phone_unescape(s: &str) -> String {
        s.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
    }
    fn pledges_phone_texts(svg: &str) -> Vec<PledgesPhoneText> {
        let body = svg.split_once("</style>").unwrap().1;
        body.split("<text")
            .skip(1)
            .map(|part| {
                let (tag, rest) = part.split_once('>').unwrap();
                let text = rest.split_once("</text>").unwrap().0;
                assert!(!text.contains('<'), "{text}");
                let kick = pledges_phone_attr(tag, "class").is_some_and(|c| c.contains("kick"));
                PledgesPhoneText {
                    x: pledges_phone_num(tag, "x"),
                    y: pledges_phone_num(tag, "y"),
                    size: if kick {
                        20.0
                    } else {
                        pledges_phone_num(tag, "font-size")
                    },
                    anchor: pledges_phone_attr(tag, "text-anchor").unwrap_or("start".into()),
                    spacing: if kick { 2.0 } else { 0.0 },
                    text: pledges_phone_unescape(text),
                }
            })
            .collect()
    }
    /// Every drawn rectangle but the card itself, as (x, y, width, height).
    fn pledges_phone_rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
        let body = svg.split_once("</defs>").unwrap().1;
        body.split("<rect")
            .skip(1)
            .map(|p| {
                let tag = p.split_once('>').unwrap().0;
                let n = |k| pledges_phone_num(tag, k);
                (n("x"), n("y"), n("width"), n("height"))
            })
            .filter(|r| r.2 < f64::from(PHONE_WIDTH) - 10.0)
            .collect()
    }
    /// All drawn text, line after line, joined by spaces.
    fn pledges_phone_words(svg: &str) -> String {
        pledges_phone_texts(svg)
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
    /// Every line fits its box by 0.62 em a character with 8 % to spare: the
    /// smallest framed box it stands in, else the card; no two lines touch, no
    /// line runs under an icon, and everything ends inside the card.
    fn pledges_phone_fits(svg: &str) {
        let height: f64 = svg
            .split_once("height=\"")
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0
            .parse()
            .unwrap();
        let rects = pledges_phone_rects(svg);
        let texts = pledges_phone_texts(svg);
        let card = (24.0, 0.0, f64::from(PHONE_WIDTH) - 48.0, height);
        let mut boxes = Vec::new();
        for t in &texts {
            let chars = t.text.chars().count() as f64;
            let w = chars * (0.62 * t.size + t.spacing);
            let need = w / 0.92;
            let (top, low) = (t.y - 0.75 * t.size, t.y + 0.22 * t.size);
            let frame = rects
                .iter()
                .filter(|r| r.2 >= 60.0)
                .filter(|r| r.0 <= t.x && t.x <= r.0 + r.2 && r.1 <= t.y && t.y <= r.1 + r.3)
                .min_by(|a, b| (a.2 * a.3).total_cmp(&(b.2 * b.3)))
                .copied()
                .unwrap_or(card);
            let (l, r) = (frame.0, frame.0 + frame.2);
            let ok = match t.anchor.as_str() {
                "start" => t.x >= l && t.x + need <= r,
                "end" => t.x - need >= l && t.x <= r,
                "middle" => t.x - need / 2.0 >= l && t.x + need / 2.0 <= r,
                a => panic!("anchor {a}"),
            };
            assert!(ok, "{:?} does not fit {l}..{r}", t.text);
            assert!(
                top >= frame.1 && low <= frame.1 + frame.3,
                "{:?} leaves its box",
                t.text
            );
            assert!(low <= height - 6.0, "{:?} below the card", t.text);
            let left = match t.anchor.as_str() {
                "start" => t.x,
                "end" => t.x - w,
                _ => t.x - w / 2.0,
            };
            let bx = (left, top, left + w, low);
            for icon in rects.iter().filter(|r| r.2 < 60.0) {
                let apart = bx.2 <= icon.0
                    || icon.0 + icon.2 <= bx.0
                    || bx.3 <= icon.1
                    || icon.1 + icon.3 <= bx.1;
                assert!(apart, "{:?} runs under an icon", t.text);
            }
            boxes.push((bx, &t.text));
        }
        for (i, (a, ta)) in boxes.iter().enumerate() {
            for (b, tb) in &boxes[i + 1..] {
                let apart = a.2 <= b.0 || b.2 <= a.0 || a.3 <= b.1 || b.3 <= a.1;
                assert!(apart, "{ta:?} touches {tb:?}");
            }
        }
    }
    /// The phone checks of the frame, motion that stands still, nothing hidden
    /// outside a keyframe, and only the allowed characters.
    fn pledges_phone_common(svg: &str) {
        let (missing, hidden) = status_motion(svg);
        assert!(missing.is_empty(), "not stilled: {missing:?}");
        assert!(hidden.is_empty(), "hidden: {hidden:?}");
        assert!(svg.contains("animation-fill-mode") || svg.contains(" both"));
        for word in [
            "<script",
            "href",
            "url(http",
            "<image",
            "STUB",
            "__",
            "textLength",
        ] {
            assert!(!svg.contains(word), "{word}");
        }
        assert!(svg.chars().all(|ch| ch.is_ascii() || "·—→×✓".contains(ch)));
        pledges_phone_fits(svg);
    }
    #[test]
    fn pledges_phone_shows_every_published_commitment() {
        check_phone("pledges");
        // The rows read a second way: the section's lines that hold a 64-hex hash.
        let text = fs::read_to_string(PLEDGES_FILE).unwrap();
        let section = text.split("## Published commitments").nth(1).unwrap();
        let rows: Vec<Vec<String>> = section
            .split("\n## ")
            .next()
            .unwrap()
            .lines()
            .filter(|l| {
                l.split('`')
                    .any(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            })
            .map(|l| l.split('|').map(|c| c.trim().to_string()).collect())
            .collect();
        assert!(!rows.is_empty());
        assert_eq!(commitments().unwrap(), rows.len());
        for dark in [false, true] {
            let s = pledges_phone(dark).unwrap();
            pledges_phone_common(&s);
            // One compact row per commitment: name, hash, date, counts, no more.
            assert_eq!(s.matches(r#"<g class="pp-cd""#).count(), rows.len());
            for row in s.split(r#"<g class="pp-cd""#).skip(1) {
                let row = row.split("</g>").next().unwrap();
                assert_eq!(row.matches("<text ").count(), 4, "{row}");
            }
            assert!(s.contains(&format!(">{}</text><text", rows.len())));
            assert!(s.contains(&format!(">{} → {}<", rows[0][1], rows[rows.len() - 1][1])));
            for r in &rows {
                let hash = r[5].trim_matches('`');
                assert!(s.contains(&format!(">{}<", &hash[..8])), "{hash}");
                assert!(!s.contains(&hash[..9]), "{hash}");
                let name = r[2].split(": ").next().unwrap();
                assert!(s.contains(&format!(">{}<", xml(name))), "{name}");
                let one = |n: &str, word: &str| {
                    if n == "1" {
                        format!("1 {word}")
                    } else if word == "directory" {
                        format!("{n} directories")
                    } else {
                        format!("{n} {word}s")
                    }
                };
                let counts = format!(">{} · {}<", one(&r[3], "directory"), one(&r[4], "file"));
                assert!(s.contains(&counts), "{counts}");
                assert!(s.contains(&format!(">{}<", &r[1][5..])), "{}", r[1]);
            }
            let words = pledges_phone_words(&s);
            for want in [
                "Hashed now",
                "checkable with xtask commitment-check once the files are handed over",
                "A hash shows they did not change, not that they are correct",
                "each code: first 8 hex of its SHA-256",
            ] {
                assert!(words.contains(want), "{want}");
            }
            // Only date, label, counts and hash: the rest of a label stays in the table.
            assert!(!s.contains("Rust sources only"));
        }
    }
    #[test]
    fn pledges_phone_draws_the_table_it_is_given() {
        let row = |date: &str, label: &str, counts: [usize; 2], hash: &str| PledgesRow {
            date: date.to_string(),
            day: pledges_day(date).unwrap(),
            label: label.to_string(),
            counts,
            hash: hash.repeat(8),
        };
        let p = Pledges {
            counted: [("file", "files"), ("directory", "directories")],
            rows: vec![
                row(
                    "2027-03-01",
                    "Hand set & co: kept out",
                    [1, 23_456],
                    "0123abcd",
                ),
                row("2027-03-01", "Second", [0, 1], "fedc9876"),
                row(
                    "2027-04-15",
                    "A commitment whose name is far too long for a single phone line",
                    [1_234_567_890, 7],
                    "aaaabbbb",
                ),
            ],
        };
        let real = Pledges::read().unwrap();
        for dark in [false, true] {
            let s = pledges_phone_draw(dark, &p).unwrap();
            pledges_phone_common(&s);
            assert_eq!(s.matches(r#"<g class="pp-cd""#).count(), 3);
            for want in [
                ">3</text><text",
                ">commitments<",
                ">2027-03-01 → 2027-04-15<",
                ">03-01<",
                ">04-15<",
                ">Hand set &amp; co<",
                ">Second<",
                ">1 file · 23,456 directories<",
                ">0 files · 1 directory<",
                ">0123abcd<",
                ">fedc9876<",
                ">aaaabbbb<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            // Too long for one line: the name wraps and the counts split.
            assert!(!s.contains(">A commitment whose name is far too long"));
            assert!(pledges_phone_words(&s)
                .contains("A commitment whose name is far too long for a single phone line"));
            assert!(s.contains(">1,234,567,890 files<") && s.contains(">7 directories<"));
            assert!(!s.contains("kept out"));
            for r in &real.rows {
                assert!(!s.contains(&r.hash[..8]), "{}", r.date);
            }
        }
        // One commitment: the word in the singular, one date, no arrow.
        let one = Pledges {
            counted: p.counted,
            rows: vec![p.rows[1].clone()],
        };
        let s = pledges_phone_draw(false, &one).unwrap();
        pledges_phone_common(&s);
        assert!(s.contains(">commitment<") && s.contains(">2027-03-01<") && !s.contains('→'));
        // Dates of two years are drawn in full and the cards make room for them.
        let mut years = p.clone();
        years.rows[0] = row("2026-12-31", "Early", [5, 5], "99998888");
        let s = pledges_phone_draw(true, &years).unwrap();
        pledges_phone_common(&s);
        assert!(s.contains(">2026-12-31<") && s.contains(">2027-04-15<"));
        let empty = Pledges {
            counted: p.counted,
            rows: Vec::new(),
        };
        assert!(pledges_phone_draw(false, &empty).is_err());
    }
    #[test]
    fn crash_phone_follows_the_recorded_series() {
        check_phone("crash");
        // The series read a second way: rows of nine tab-separated fields that
        // start with a number, and the host line split by hand.
        let text = fs::read_to_string(CRASH_FILE).unwrap();
        let rows: Vec<Vec<usize>> = text
            .lines()
            .map(|l| l.split('\t').collect::<Vec<_>>())
            .filter(|f| f.len() == 9 && f[0].parse::<usize>().is_ok())
            .map(|f| f[..7].iter().map(|v| v.parse().unwrap()).collect())
            .collect();
        let acknowledged: usize = rows.iter().map(|r| r[2]).sum();
        let lost: usize = rows.iter().map(|r| r[3]).sum();
        let partial: usize = rows.iter().map(|r| r[5]).sum();
        let mid = rows.iter().filter(|r| r[4] + r[6] > 0).count();
        assert!(!rows.is_empty() && acknowledged > 0 && mid > 0);
        let host: Vec<&str> = text
            .lines()
            .find_map(|l| l.strip_prefix("# host: "))
            .unwrap()
            .split(" · ")
            .collect();
        let machine = format!("({}, {})", host[2], &host[5][..10]);
        for dark in [false, true] {
            let s = crash_phone(dark).unwrap();
            pledges_phone_common(&s);
            assert_eq!(s.matches(r#"<circle class="cp-pt""#).count(), rows.len());
            for want in [
                format!(">{} kills while saving.<", rows.len()),
                format!(">{lost} acknowledged snapshots lost.<"),
                format!(">{}<", rows.len()),
                format!(">{}<", grouped(acknowledged)),
                format!(">{partial}<"),
                format!(">killed mid-publication ({mid})<"),
                format!(">killed between saves ({})<", rows.len() - mid),
                ">not a power cut:<".to_string(),
                ">a process kill (SIGKILL),<".to_string(),
            ] {
                assert!(s.contains(&want), "{want}");
            }
            let words = pledges_phone_words(&s);
            for want in [
                format!("One recorded run on the author's Linux machine {machine}"),
                "power-loss durability is not established".to_string(),
            ] {
                assert!(words.contains(&want), "{want}");
            }
            for hidden in ["Ryzen", "Radeon", "oem", "x86_64", "ebe5360"] {
                assert!(!s.contains(hidden), "{hidden}");
            }
        }
    }
    #[test]
    fn crash_phone_draws_the_series_it_is_given() {
        let trial = |kill_ms, acknowledged, mid_publication| CrashTrial {
            kill_ms,
            acknowledged,
            mid_publication,
        };
        let c = CrashSeries {
            date: "2027-01-02".to_string(),
            file_system: "btrfs".to_string(),
            documents: 4,
            window_ms: 100,
            trials: vec![trial(0, 0, false), trial(100, 4, true), trial(50, 2, true)],
            acknowledged: 6,
            lost: 0,
            partial: 0,
            resumed: 3,
            mid_publication: 2,
        };
        for dark in [false, true] {
            let s = crash_phone_draw(dark, &c);
            pledges_phone_common(&s);
            assert_eq!(s.matches(r#"<circle class="cp-pt""#).count(), 3);
            for want in [
                ">3 kills while saving.<",
                ">6<",
                ">killed mid-publication (2)<",
                ">killed between saves (1)<",
                ">all 4 snapshots saved<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            let words = pledges_phone_words(&s);
            assert!(words.contains("Linux machine (btrfs, 2027-01-02)"));
            assert!(words.contains("kill time, ms after the writer started (window 100 ms)"));
            assert!(!s.contains("2,683") && !s.contains("ext4") && !s.contains(">200<"));
            // Kill at 0 ms with nothing saved is the lower left corner of the plot,
            // at the window end with all saved the upper right one.
            let pts: Vec<(f64, f64)> = s
                .split(r#"<circle class="cp-pt""#)
                .skip(1)
                .map(|p| (pledges_phone_num(p, "cx"), pledges_phone_num(p, "cy")))
                .collect();
            assert!(pts[0].0 < pts[2].0 && pts[2].0 < pts[1].0);
            assert!(pts[0].1 > pts[2].1 && pts[2].1 > pts[1].1);
            assert!((pts[2].0 - (pts[0].0 + pts[1].0) / 2.0).abs() < 0.11);
        }
        // Wide numbers make room for themselves.
        let mut big = c;
        big.documents = 1000;
        big.window_ms = 5000;
        big.acknowledged = 1_234_567;
        big.trials = (0..400)
            .map(|i| trial(i * 12, i as usize * 2, i % 3 == 0))
            .collect();
        big.resumed = 400;
        big.mid_publication = 134;
        let s = crash_phone_draw(false, &big);
        pledges_phone_common(&s);
        assert!(s.contains(">1,234,567<") && s.contains(">1000<"));
    }
    /// The quotes a license phone card draws: each quote group's lines joined,
    /// with the quotation marks on its first and last line taken off.
    fn license_phone_drawn(svg: &str) -> Vec<String> {
        svg.split(r#"<g class="lp-q">"#)
            .skip(1)
            .map(|g| {
                let g = g.split_once("</g>").unwrap().0;
                let joined = pledges_phone_words(&format!("</style>{g}"));
                let inner = joined.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
                assert!(!inner.contains('"'), "{inner}");
                inner.to_string()
            })
            .collect()
    }
    #[test]
    fn license_phone_quotes_the_license_word_for_word() {
        check_phone("license");
        let license = fs::read_to_string(LICENSE_FILE).unwrap();
        let folded = license.split_whitespace().collect::<Vec<_>>().join(" ");
        let quotes = license_quotes(&license).unwrap();
        assert_eq!(quotes.len(), 7);
        // The quotes of each subject and the notice, in the wide card's order.
        let shown: Vec<&str> = LICENSE_COLUMNS
            .iter()
            .zip(LICENSE_PHONE_SHOWN)
            .flat_map(|(c, picked)| picked.iter().map(|i| c.2[*i]))
            .chain([LICENSE_NOTICE_QUOTE])
            .collect();
        assert_eq!(shown.len(), LICENSE_COLUMNS.len() + 2);
        // What the public terms give, who may evaluate and within what limit,
        // what money needs. The right to evaluate is never drawn without its
        // limit: this sentence is typed here, not taken from the list.
        let limit = "in a non-production environment solely to assess, learn about, validate, or decide whether to obtain commercial rights.";
        assert!(folded.contains(limit));
        assert!(shown[0].ends_with("share the Software for Noncommercial Purposes."));
        assert!(shown[1].starts_with("Any person or organization may evaluate"));
        assert!(shown[3].starts_with("Before beginning a Monetized Use"));
        for dark in [false, true] {
            let s = license_phone(dark).unwrap();
            pledges_phone_common(&s);
            let drawn = license_phone_drawn(&s);
            assert_eq!(drawn, shown);
            let at = |q: &str| drawn.iter().position(|d| d == q);
            assert_eq!(
                at(limit).unwrap(),
                at(shown[1]).unwrap() + 1,
                "the limit follows the right it limits"
            );
            // The description says what the card draws, in the wide card's words.
            let wide = xml(&license_text());
            let desc = s.split_once("<desc id=\"d\">").unwrap().1;
            let desc = desc.split_once("</desc>").unwrap().0;
            for q in &quotes {
                assert!(wide.contains(&xml(q)), "{q}");
                assert_eq!(desc.contains(&xml(q)), shown.contains(q), "{q}");
            }
            assert!(desc.contains("A summary adds no rights; LICENSE controls (section 17)."));
            for q in &drawn {
                assert!(folded.contains(q.as_str()), "{q}");
            }
            for (section, heading, _) in LICENSE_COLUMNS {
                assert!(s.contains(&format!(">{heading}<")), "{heading}");
                assert!(s.contains(&format!(">LICENSE {section}<")), "{section}");
            }
            for want in [
                ">LICENSE controls (section 17).<",
                ">A summary adds no rights;<",
                ">gelram.licensing@gmail.com<",
                ">EVERY COPY · SECTION 8<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            let words = pledges_phone_words(&s);
            for want in [
                "GEL RAM Noncommercial Reciprocal License 1.0",
                "Source-available, not an OSI-approved open-source license.",
            ] {
                assert!(words.contains(want), "{want}");
            }
        }
        // A LICENSE that no longer says a quoted sentence draws no card.
        for (from, to) in [
            ("non-production environment", "production environment"),
            ("without a time limit", "for one year"),
            ("`Required Notice:`", "`Notice:`"),
        ] {
            assert!(license.contains(from), "{from}");
            assert!(license_phone_of(false, &license.replacen(from, to, 1)).is_err());
        }
    }
    #[test]
    fn license_phone_draws_the_quotes_it_is_given() {
        let long = "One hand-made sentence, long enough & <odd> to need three lines or more on a narrow phone card, kept whole.";
        let columns = vec![
            ("section 9", "Hand-made heading", vec![long, "Short."]),
            ("section 10", "Second", vec!["Only one quote here."]),
        ];
        let notice = "a hand-made notice with `ticks` in it.";
        for dark in [false, true] {
            let s = license_phone_draw(dark, &columns, notice);
            pledges_phone_common(&s);
            assert_eq!(
                license_phone_drawn(&s),
                [long, "Short.", "Only one quote here.", notice]
            );
            assert!(s.contains("&amp; &lt;odd&gt;"));
            assert!(s.contains(">LICENSE section 10<") && s.contains(">Hand-made heading<"));
            assert!(!s.contains("Noncommercial use") && !s.contains("Monetized"));
            assert!(s.contains("Hand-made heading (section 9): 'One hand-made sentence, long enough &amp; &lt;odd&gt;"));
        }
    }
    // ===== PHONE TEST a4 END =====

    // ===== PHONE TEST a5 BEGIN =====
    /// Every text of a picture as (x, y, font size, letter spacing, anchored
    /// at its end, text), the text unescaped. The phone kicker has no size
    /// attribute: 20 px with 2 px letter spacing.
    fn core_phone_texts(svg: &str) -> Vec<(f64, f64, f64, f64, bool, String)> {
        let mut out = Vec::new();
        for part in svg.split("<text ").skip(1) {
            let (attrs, rest) = part.split_once('>').unwrap();
            let attrs = format!(" {attrs}");
            let attr = |k: &str| {
                attrs
                    .split_once(&format!(" {k}=\""))
                    .map(|(_, v)| v.split('"').next().unwrap().to_string())
            };
            let num = |k: &str| attr(k).map(|v| v.parse::<f64>().unwrap());
            let kick = attr("class").is_some_and(|c| c.split(' ').any(|c| c == "kick"));
            let (size, spacing) = match num("font-size") {
                Some(s) => (s, 0.0),
                None if kick => (20.0, 2.0),
                None => panic!("a text without a size: {attrs}"),
            };
            let text = rest.split_once("</text>").unwrap().0;
            let text = text
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&amp;", "&");
            out.push((
                num("x").unwrap(),
                num("y").unwrap(),
                size,
                spacing,
                attr("text-anchor").as_deref() == Some("end"),
                text,
            ));
        }
        out
    }
    /// The left and right edge of a text at 0.62 × size per character, widened
    /// by 8 % to spare.
    fn core_phone_extent(t: &(f64, f64, f64, f64, bool, String)) -> (f64, f64) {
        let w = t.5.chars().count() as f64 * (0.62 * t.2 + t.3) / 0.92;
        if t.4 {
            (t.0 - w, t.0)
        } else {
            (t.0, t.0 + w)
        }
    }
    /// A hand-made map of three parts, one dependency, one part alone.
    fn core_phone_mutant() -> CoreMap {
        let part = |name: &str, krate: &str, slot, hue| CorePart {
            name: name.to_string(),
            krate: krate.to_string(),
            tag: [format!("first line of {name}"), format!("second of {name}")],
            slot,
            hue,
        };
        CoreMap {
            parts: vec![
                part("Lantern", "zz-lantern", (0.0, 0.0), CoreHue::Green),
                part("Kettle", "zz-kettle", (1.0, 0.0), CoreHue::Amber),
                part("Pebble", "zz-pebble", (2.0, 2.0), CoreHue::Purple),
            ],
            links: vec![(1, 0)],
        }
    }
    #[test]
    fn core_phone_is_readable_on_a_phone() {
        check_phone("core");
        for dark in [false, true] {
            let s = core_phone(dark).unwrap();
            let (missing, hidden) = status_motion(&s);
            assert!(missing.is_empty(), "not stilled: {missing:?}");
            assert!(hidden.is_empty(), "hidden: {hidden:?}");
            for class in ["pb", "pn", "pa", "ph", "px", "pp"] {
                assert!(s.contains(&format!("class=\"{class}\"")), "{class}");
            }
            assert!(!s.contains("textLength") && !s.contains("scale("));
            assert!(s.chars().all(|c| c.is_ascii() || "·—→×✓".contains(c)));
        }
    }
    #[test]
    fn core_phone_draws_the_parts_and_the_dependencies_of_the_crates() {
        // A second reading: the crates of the table rows of the section, and
        // b builds on a when a line of b's Cargo.toml begins with the name of a.
        let section = NATIVE.split("## The memory core").nth(1).unwrap();
        let section = section.split("\n## ").next().unwrap();
        let crates: Vec<&str> = section
            .lines()
            .filter(|l| l.starts_with("| ") && l.contains(" (`"))
            .map(|l| l.split("(`").nth(1).unwrap().split('`').next().unwrap())
            .collect();
        let mut links = 0;
        let mut joined = std::collections::BTreeSet::new();
        for b in &crates {
            let toml = fs::read_to_string(format!("crates/{b}/Cargo.toml")).unwrap();
            for a in &crates {
                if toml
                    .lines()
                    .any(|l| l.starts_with(&format!("{a} ")) || l.starts_with(&format!("{a}.")))
                {
                    links += 1;
                    joined.extend([*a, *b]);
                }
            }
        }
        assert!(links > 0 && crates.len() > joined.len());
        let m = CoreMap::read().unwrap();
        for dark in [false, true] {
            let s = core_phone(dark).unwrap();
            assert_eq!(s.matches("<g class=\"pb\"").count(), crates.len());
            for class in [
                "<path class=\"pa\"",
                "<polygon class=\"ph\"",
                "<path class=\"pp\"",
            ] {
                assert_eq!(s.matches(class).count(), links, "{class}");
            }
            assert!(s.contains(&format!(r#"class="big">{}<"#, crates.len())));
            assert!(s.contains(&format!(r#"class="big">{links}<"#)));
            assert!(s.contains(">dependencies<") && s.contains(">parts<"));
            for k in &crates {
                assert!(s.contains(&format!(">{k}<")), "{k}");
            }
            for p in &m.parts {
                for line in [&p.name, &p.tag[0], &p.tag[1]] {
                    assert!(s.contains(&format!(">{}<", xml(line))), "{line}");
                }
            }
            assert!(s.contains(">stand alone<"));
            assert!(s.contains(&format!("<desc id=\"d\">{}</desc>", xml(&core_text(&m)))));
            assert!(s.contains(">Arrow A → B: B builds on A,<"));
            assert!(s.contains(">as its Cargo.toml says<"));
        }
        // Every arrow runs down the card: a part stands above what builds on it.
        let lay = core_phone_layout(&m);
        for &(a, b) in &m.links {
            assert!(
                lay.top[a] < lay.top[b],
                "{} {}",
                m.parts[a].krate,
                m.parts[b].krate
            );
        }
        // The parts that stand alone are below every joined part, in the frame.
        let (top, bottom) = lay.frame.unwrap();
        for (i, p) in m.parts.iter().enumerate() {
            let inside = lay.top[i] > top && lay.top[i] + CORE_PHONE_H < bottom;
            assert_eq!(inside, !joined.contains(p.krate.as_str()), "{}", p.krate);
        }
    }
    #[test]
    fn core_phone_follows_other_data() {
        let m = core_phone_mutant();
        for dark in [false, true] {
            let s = core_phone_draw(dark, &m).unwrap();
            assert_eq!(s.matches("<g class=\"pb\"").count(), 3);
            assert_eq!(s.matches("<path class=\"pa\"").count(), 1);
            assert!(s.contains(r#"class="big">3<"#) && s.contains(r#"class="big">1<"#));
            assert!(s.contains(">dependency<") && !s.contains(">dependencies<"));
            for word in ["Lantern", "zz-kettle", "second of Pebble", ">stand alone<"] {
                assert!(s.contains(word), "{word}");
            }
            for real in ["gel-orb", "Bit record", "16 judgments"] {
                assert!(!s.contains(real), "{real}");
            }
        }
        // Kettle builds on nothing and stands first; Lantern builds on it.
        let lay = core_phone_layout(&m);
        assert!(lay.top[1] < lay.top[0] && lay.top[0] < lay.top[2]);
        // With nothing alone there is no frame, and the card is shorter.
        let mut joined = core_phone_mutant();
        joined.links.push((0, 2));
        let s = core_phone_draw(false, &joined).unwrap();
        assert!(!s.contains("stand alone") && s.contains(">dependencies<"));
        assert!(core_phone_layout(&joined).height < lay.height);
    }
    #[test]
    fn core_phone_text_fits_its_boxes() {
        let right = f64::from(PHONE_WIDTH) - 16.0;
        let block = (CORE_PHONE_X + 8.0, CORE_PHONE_X + CORE_PHONE_W - 8.0);
        for dark in [false, true] {
            let s = core_phone(dark).unwrap();
            let texts = core_phone_texts(&s);
            assert!(texts.len() > 20);
            let mut counts = 0;
            for t in &texts {
                let (l, r) = core_phone_extent(t);
                assert!(l >= 16.0 && r <= right, "{} at {l:.0}..{r:.0}", t.5);
                // Block texts start 22 in from a block's edge or end 16 before
                // its other edge; the caption of the frame starts 6 in.
                if t.0 >= CORE_PHONE_X + 16.0 {
                    assert!(l >= block.0 && r <= block.1, "{} leaves its block", t.5);
                }
                // The first count and its word end before the second begin.
                if t.1 == 104.0 && t.0 < 240.0 {
                    counts += 1;
                    assert!(r <= 240.0, "{}", t.5);
                }
            }
            assert_eq!(counts, 2);
            // The name and the crate of a block share a line without meeting.
            let mut shared = 0;
            for a in texts.iter().filter(|t| !t.4 && t.0 > CORE_PHONE_X) {
                for b in texts.iter().filter(|t| t.4 && t.1 == a.1) {
                    assert!(
                        core_phone_extent(a).1 + 8.0 <= core_phone_extent(b).0,
                        "{} {}",
                        a.5,
                        b.5
                    );
                    shared += 1;
                }
            }
            assert_eq!(shared, CoreMap::read().unwrap().parts.len());
        }
        // A name too long for one line beside its crate is refused, not squeezed.
        let mut long = core_phone_mutant();
        long.parts[0].name = "Very long part name here".to_string();
        let e = core_phone_draw(false, &long).unwrap_err().to_string();
        assert!(e.contains("do not fit one line"), "{e}");
        let mut tag = core_phone_mutant();
        tag.parts[2].tag[1] = "a tag line that runs on and on and on".to_string();
        assert!(core_phone_draw(true, &tag).is_err());
        // Counts too wide for their line are refused too.
        let mut many = core_phone_mutant();
        while many.parts.len() < 100 {
            many.parts.push(core_phone_mutant().parts.remove(0));
        }
        let e = core_phone_draw(true, &many).unwrap_err().to_string();
        assert!(e.contains("the counts do not fit"), "{e}");
    }
    /// The tallest a phone card may be, except the q8 card, and the most the
    /// fourteen may add up to: about ten phone screens together. Both hold at
    /// the commitments and properties counted below; more of those grow the
    /// cards that list them, and each limit by exactly that.
    const PHONE_MOST_TALL: f64 = 1180.0;
    const PHONE_MOST_SUM: f64 = 10_800.0;
    const PHONE_BASE_COMMITMENTS: usize = 9;
    const PHONE_BASE_PROPERTIES: usize = 40;
    /// One phone card: its name, its height, and how much of that the data
    /// beyond the base counts adds.
    type PhoneHeight = (&'static str, f64, f64);
    /// The height a drawn phone card states. The view box must say the same.
    fn phone_height(name: &str, s: &str) -> f64 {
        let root = s.split_once('>').unwrap().0;
        let word = |key: &str| root.split_once(key).unwrap().1.split('"').next().unwrap();
        let (tall, view) = (word(" height=\""), word(" viewBox=\"0 0 600 "));
        assert_eq!(tall, view, "{name}");
        tall.parse().unwrap()
    }
    /// Every phone card but q8 in one theme, the two that grow with data drawn
    /// from `p` and `e` and again from their first rows up to the base counts.
    fn phone_heights_of(dark: bool, p: &Pledges, e: &Evidence) -> Vec<PhoneHeight> {
        let base_p = Pledges {
            rows: p.rows[..p.rows.len().min(PHONE_BASE_COMMITMENTS)].to_vec(),
            ..p.clone()
        };
        let base_e = Evidence {
            rows: e.rows[..e.rows.len().min(PHONE_BASE_PROPERTIES)].to_vec(),
            unix_only: e.unix_only.clone(),
            formats: Vec::new(),
        };
        let out: Vec<PhoneHeight> = PHONES
            .iter()
            .filter(|c| c.0 != "q8")
            .map(|(name, draw)| {
                let (now, base) = match *name {
                    "pledges" => (
                        pledges_phone_draw(dark, p).unwrap(),
                        pledges_phone_draw(dark, &base_p).unwrap(),
                    ),
                    "wall" => (wall_phone_draw(dark, e), wall_phone_draw(dark, &base_e)),
                    _ => {
                        let s = draw(dark).unwrap();
                        (s.clone(), s)
                    }
                };
                let tall = phone_height(name, &now);
                (*name, tall, tall - phone_height(name, &base))
            })
            .collect();
        assert_eq!(out.len(), 14);
        out
    }
    /// The tallest cards first, for the message of a limit that is passed.
    fn phone_largest(cards: &[PhoneHeight]) -> String {
        let mut by_height = cards.to_vec();
        by_height.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top: Vec<String> = by_height
            .iter()
            .take(5)
            .map(|c| format!("{} {}", c.0, c.1))
            .collect();
        top.join(", ")
    }
    fn phone_tall_check(cards: &[PhoneHeight]) -> std::result::Result<(), String> {
        match cards.iter().find(|c| c.1 > PHONE_MOST_TALL + c.2) {
            Some(c) => Err(format!(
                "{} is {} tall, more than {PHONE_MOST_TALL} + {} for its data",
                c.0, c.1, c.2
            )),
            None => Ok(()),
        }
    }
    fn phone_sum_check(cards: &[PhoneHeight]) -> std::result::Result<(), String> {
        let sum: f64 = cards.iter().map(|c| c.1).sum();
        let grown: f64 = cards.iter().map(|c| c.2).sum();
        if sum > PHONE_MOST_SUM + grown {
            return Err(format!(
                "the cards add up to {sum}, more than {PHONE_MOST_SUM} + {grown} for their data; largest: {}",
                phone_largest(cards)
            ));
        }
        Ok(())
    }
    fn phone_heights(dark: bool) -> Vec<PhoneHeight> {
        phone_heights_of(dark, &Pledges::read().unwrap(), &Evidence::read().unwrap())
    }
    #[test]
    fn no_phone_card_but_q8_is_taller_than_its_limit() {
        for dark in [false, true] {
            phone_tall_check(&phone_heights(dark)).unwrap();
        }
    }
    #[test]
    fn the_fourteen_phone_cards_stay_within_their_sum() {
        for dark in [false, true] {
            phone_sum_check(&phone_heights(dark)).unwrap();
        }
    }
    #[test]
    fn the_height_limits_follow_the_data_and_not_a_taller_card() {
        let (p, e) = (Pledges::read().unwrap(), Evidence::read().unwrap());
        let mut more_p = p.clone();
        for _ in 0..12 {
            more_p.rows.push(p.rows[0].clone());
        }
        let mut more_e = Evidence {
            rows: e.rows.clone(),
            unix_only: e.unix_only.clone(),
            formats: Vec::new(),
        };
        for _ in 0..60 {
            more_e.rows.push(e.rows[0].clone());
        }
        for dark in [false, true] {
            let cards = phone_heights_of(dark, &p, &e);
            // New commitments and properties grow their cards past the plain
            // limits, and no code changes: the limits grow with them.
            let grown = phone_heights_of(dark, &more_p, &more_e);
            let sum = |c: &[PhoneHeight]| c.iter().map(|h| h.1).sum::<f64>();
            assert!(sum(&grown) > PHONE_MOST_SUM && sum(&grown) > sum(&cards));
            assert!(grown
                .iter()
                .any(|c| c.1 > PHONE_MOST_TALL && c.0 == "pledges"));
            phone_tall_check(&grown).unwrap();
            phone_sum_check(&grown).unwrap();
            // Only what the data adds: all other cards keep their heights.
            for (now, then) in grown.iter().zip(&cards) {
                let listed = matches!(now.0, "pledges" | "wall");
                assert_eq!(now.1 > then.1, listed, "{}", now.0);
                assert_eq!(now.2 > then.2, listed, "{}", now.0);
            }
            // A card that merely got taller has no data to show for it.
            let over = PHONE_MOST_SUM + cards.iter().map(|c| c.2).sum::<f64>() - sum(&cards);
            let mut taller = cards.clone();
            taller[0].1 += over + 1.0;
            let message = phone_sum_check(&taller).unwrap_err();
            assert!(message.contains(&phone_largest(&taller)), "{message}");
            assert!(message.contains("largest: "), "{message}");
            let mut taller = cards.clone();
            taller[0].1 = PHONE_MOST_TALL + taller[0].2 + 1.0;
            assert!(phone_tall_check(&taller).is_err());
        }
    }
    /// The numbers of the badges counted a second way from the files: the
    /// drawn values in the order of the row, and the views, the passing runs
    /// and the kills that the texts around the values state.
    struct BadgeCounted {
        values: [String; 8],
        views: usize,
        runs: usize,
        kills: usize,
    }
    fn badge_counts() -> BadgeCounted {
        // Views: the four packed copies over one record in a recorded run;
        // runs: those whose verdict line passes for all views, out of all.
        let (mut runs, mut all, mut views) = (0, 0, 0);
        for e in fs::read_dir(RECORD_DIR).unwrap() {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            if name.starts_with("v2-") && name.ends_with(".txt") {
                let text = fs::read_to_string(format!("{RECORD_DIR}/{name}")).unwrap();
                runs += usize::from(text.lines().any(|l| l == Q8_PASS));
                let get = |key: &str| -> usize {
                    let at = text.split(&format!("{key}=")).nth(1).unwrap();
                    at.split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                };
                views = get("packed_four_record_bytes") / get("canonical_record_bytes");
                all += 1;
            }
        }
        assert!(runs > 0);
        // Mutants: each mutant has exactly one row read with the original pin.
        let matrix = fs::read_to_string("docs/evidence-mutation/matrix-r1.txt").unwrap();
        let mutants = matrix
            .lines()
            .filter(|l| l.split('\t').nth(4) == Some("original"))
            .count();
        // Crash series: the trial rows, their acknowledged and lost columns.
        let crash = fs::read_to_string(CRASH_FILE).unwrap();
        let rows: Vec<Vec<&str>> = crash
            .lines()
            .filter(|l| l.starts_with(|c: char| c.is_ascii_digit()))
            .map(|l| l.split('\t').collect())
            .collect();
        let sum = |k: usize| {
            rows.iter()
                .map(|r| r[k].parse::<usize>().unwrap())
                .sum::<usize>()
        };
        // Speed: a case is faster when its gain is above both A2/A and A/A2.
        let faster = |dir: &str| -> Vec<(String, bool)> {
            let mut out = Vec::new();
            for e in fs::read_dir(dir).unwrap() {
                let e = e.unwrap();
                let summary = e.path().join("summary.txt");
                let Ok(text) = fs::read_to_string(&summary) else {
                    continue;
                };
                for line in text.lines().filter(|l| l.starts_with("SPEED_AB ")) {
                    let field = |k: &str| {
                        line.split(' ')
                            .find_map(|w| w.strip_prefix(k))
                            .unwrap()
                            .to_string()
                    };
                    let (aa, gain): (f64, f64) = (
                        field("aa=").parse().unwrap(),
                        field("gain=").parse().unwrap(),
                    );
                    let key = format!("{}/{}", e.file_name().to_string_lossy(), field("case="));
                    out.push((key, gain > aa && gain > 1.0 / aa));
                }
            }
            out.sort();
            out
        };
        let (old, new) = (faster(SPEED_DIR), faster(RERUN_DIR));
        assert_eq!(old.len(), new.len());
        let twice = old
            .iter()
            .zip(&new)
            .filter(|(a, b)| a.0 == b.0 && a.1 && b.1)
            .count();
        // Commitments: the dated rows of the published table.
        let pledges = fs::read_to_string("docs/COMMITMENTS.md").unwrap();
        let table = pledges.split("## Published commitments").nth(1).unwrap();
        let dated = table
            .lines()
            .take_while(|l| !l.starts_with("## "))
            .filter(|l| l.starts_with("| 2"))
            .count();
        // Packages: the names in Cargo.lock.
        let lock = fs::read_to_string("Cargo.lock").unwrap();
        let names = lock.lines().filter(|l| l.starts_with("name = ")).count();
        let toml = fs::read_to_string("Cargo.toml").unwrap();
        let version = toml
            .split("[workspace.package]")
            .nth(1)
            .unwrap()
            .lines()
            .find_map(|l| l.strip_prefix("version = \""))
            .unwrap()
            .trim_end_matches('"')
            .to_string();
        BadgeCounted {
            values: [
                format!("{runs}/{all}"),
                grouped(mutants),
                format!("{}/{}", grouped(sum(3)), grouped(sum(2))),
                names.to_string(),
                format!("{twice}/{}", new.len()),
                dated.to_string(),
                version,
                "noncommercial".to_string(),
            ],
            views,
            runs,
            kills: rows.len(),
        }
    }
    /// The number a badge picture draws.
    fn badge_drawn(svg: &str) -> String {
        let at = svg.split_once(r#"<text class="bv""#).unwrap().1;
        let at = at.split_once('>').unwrap().1;
        at.split_once('<').unwrap().0.replace("&amp;", "&")
    }
    /// The label a badge picture draws.
    fn badge_label(svg: &str) -> String {
        let at = svg.split_once(r#"<text class="bl""#).unwrap().1;
        let at = at.split_once('>').unwrap().1;
        at.split_once('<').unwrap().0.replace("&amp;", "&")
    }
    /// The cells of the README row, one per badge: its link and its pictures.
    fn badge_cells(row: &str) -> Vec<&str> {
        row.split("<a href=\"").skip(1).collect()
    }
    /// The alternative text of one cell of the row.
    fn badge_alt(cell: &str) -> &str {
        cell.split_once("alt=\"")
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
    }
    #[test]
    fn badges_draw_the_numbers_of_their_files() {
        let c = badge_counts();
        let want = &c.values;
        assert_eq!(c.views, 4);
        assert_eq!(want[0], format!("{0}/{0}", c.runs));
        let assets = badge_assets().unwrap();
        assert_eq!(assets.len(), 2 * want.len());
        let row = badges_row().unwrap();
        let cells = badge_cells(&row);
        assert_eq!(cells.len(), want.len());
        // Words each badge's own alternative text must carry, with its number.
        let of = |v: &str| v.replace('/', " of ");
        let kills = grouped(c.kills);
        let says: [Vec<String>; 8] = [
            vec![
                format!("{0} of {0} views score bit for bit", c.views),
                format!("in {} recorded runs", c.runs),
            ],
            vec![format!("{} format mutants", want[1])],
            vec![
                format!("{} confirmed snapshots lost", of(&want[2])),
                format!("in {kills} process kills, not a power cut"),
            ],
            vec![format!("{} packages in Cargo.lock", want[3])],
            vec![format!("{} cases faster", of(&want[4])), "both runs".into()],
            vec![format!("{} published commitments", want[5])],
            vec![format!("Version {}", want[6])],
            vec![format!("License: {}", want[7])],
        ];
        for (i, ((pair, value), cell)) in assets.chunks(2).zip(want).zip(&cells).enumerate() {
            // The cell shows this badge's pictures and none of another's.
            for (path, svg) in pair {
                assert!(
                    cell.contains(&format!("src=\"{path}\""))
                        || cell.contains(&format!("srcset=\"{path}\"")),
                    "{path}"
                );
                assert_eq!(&badge_drawn(svg), value, "{path}");
                assert!(svg.contains(&format!(": {}</title>", xml(value))), "{path}");
            }
            let alt = badge_alt(cell);
            for words in &says[i] {
                assert!(alt.contains(&xml(words)), "{alt}: {words}");
            }
        }
        // The crash badge names its kills in its label too; the speed badge
        // says both runs, not twice, which reads as twice as fast.
        let lost = &assets[4].1;
        assert_eq!(badge_label(lost), format!("lost in {kills} kills"));
        assert_eq!(badge_label(&assets[8].1), "faster, both runs");
    }
    #[test]
    fn badges_follow_other_data() {
        // The version is the one whose release notes the root holds, since the
        // badge links them; every other number is made up.
        let version = &RELEASE[1..];
        let version_alt = format!("Version {version}");
        let other = BadgeFacts {
            views: 3,
            q8_runs: 7,
            q8_all: 8,
            mutants: 1234,
            kills: 31,
            acknowledged: 40,
            lost: 2,
            faster_twice: 5,
            cases: 9,
            commitments: 77,
            packages: 15,
            version: version.to_string(),
            license: "noncommercial".to_string(),
        };
        let badges = badges_checked(&other).unwrap();
        let drawn: Vec<String> = badges
            .iter()
            .enumerate()
            .map(|(i, b)| badge_drawn(&badge_svg(false, i, b)))
            .collect();
        assert_eq!(
            drawn,
            [
                "7/8",
                "1,234",
                "2/40",
                "15",
                "5/9",
                "77",
                version,
                "noncommercial"
            ]
        );
        assert_eq!(badges[2].label, "lost in 31 kills");
        let row = badges_html(&badges);
        assert_eq!(badge_cells(&row).len(), 8);
        for (cell, alt) in badge_cells(&row).into_iter().zip([
            "3 of 3 views score bit for bit as their written-out copies in 7 of 8 recorded runs",
            "1,234 format mutants",
            "2 of 40 confirmed snapshots lost in 31 process kills",
            "15 packages",
            "5 of 9 cases",
            "77 published",
            version_alt.as_str(),
            "License: noncommercial",
        ]) {
            assert!(badge_alt(cell).contains(alt), "{alt}");
        }
        // A version without release notes is refused: every badge links to a file.
        let missing = BadgeFacts {
            version: "9.9.9".to_string(),
            ..other
        };
        let e = badges_checked(&missing).err().unwrap().to_string();
        assert!(e.contains("RELEASE-NOTES-v9.9.9.md"), "{e}");
        // The readers refuse what the badges cannot state.
        assert_eq!(
            badge_version("[workspace.package]\nversion = \"1.2.3\"\n").unwrap(),
            "1.2.3"
        );
        assert!(badge_version("[package]\nversion = \"1.2.3\"\n").is_err());
        assert!(badge_license("MIT License\n").is_err());
    }
    #[test]
    fn badges_are_pictures_that_fit_move_and_stand_still() {
        let row = badges_row().unwrap();
        // One line, centred, no visible words: only pictures and links.
        assert!(row.starts_with("<p align=\"center\"><a href=\"") && row.ends_with("</a></p>"));
        assert!(!row.contains('\n'));
        assert_eq!(visible_words(&row), 0);
        assert!(native_intro().contains(&format!("\n\n{row}\n\n")));
        let alts: Vec<&str> = row
            .split("alt=\"")
            .skip(1)
            .map(|a| a.split('"').next().unwrap())
            .collect();
        assert_eq!(alts.len(), row.matches("<a href=\"").count());
        for alt in &alts {
            assert!(alt.len() <= 90 && !alt.is_empty(), "{alt}");
        }
        let assets = badge_assets().unwrap();
        for pair in assets.chunks(2) {
            let ((light_path, light), (dark_path, dark)) = (&pair[0], &pair[1]);
            assert!(light_path.ends_with("-light.svg") && dark_path.ends_with("-dark.svg"));
            assert_ne!(light, dark, "{light_path}");
            for (path, s) in pair {
                assert!(s.contains("<title"), "{path}");
                assert!(!s.contains("<script") && !s.contains("href"), "{path}");
                assert!(s.contains("@keyframes"), "{path}");
                let (missing, hidden) = status_motion(s);
                assert!(
                    missing.is_empty() && hidden.is_empty(),
                    "{path}: {missing:?} {hidden:?}"
                );
                // Labels at least 14 px, numbers at least 16 px.
                for size in font_sizes(s) {
                    assert!(size >= 14.0, "{path}: {size} px text");
                }
                assert!(s.contains(".bl{font-size:14px;") && s.contains(".bv{font-size:16px;"));
                // The pill is 30 to 32 high, the picture at most 34, shown at its
                // own width.
                let width = s
                    .split_once("width=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap();
                let tall: f64 = s
                    .split_once(" height=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(tall <= 34.0, "{path}: {tall}");
                assert!(
                    row.contains(&format!("src=\"{light_path}\" width=\"{width}\"")),
                    "{path}"
                );
                let pill = s.split_once("<g clip-path=\"url(#pill)\">").unwrap().1;
                let rects: Vec<f64> = pill
                    .split("width=\"")
                    .skip(1)
                    .take(2)
                    .map(|w| w.split('"').next().unwrap().parse().unwrap())
                    .collect();
                let height: f64 = pill
                    .split_once("height=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!((30.0..=32.0).contains(&height), "{path}");
                // Label and number each fit their part in Arial, 8 % to spare.
                for (text, size, bold, part) in [
                    (badge_label(s), BADGE_LABEL_PX, false, rects[0]),
                    (badge_drawn(s), BADGE_VALUE_PX, true, rects[1]),
                ] {
                    let w = badge_text_width(&text, size, bold);
                    assert!(w <= (part - 2.0 * BADGE_PAD) * 0.92, "{path}: {text}");
                }
            }
        }
        // Two by two, the badges stand side by side in a 358-px phone column,
        // with the space between them and 8 px to spare.
        let widths: Vec<f64> = assets
            .chunks(2)
            .map(|p| {
                p[0].1
                    .split_once("width=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect();
        assert_eq!(widths.len() % 2, 0);
        for two in widths.chunks(2) {
            assert!(two[0] + 5.0 + two[1] <= 358.0 - 8.0, "{two:?}");
        }
    }
    #[test]
    fn badge_text_width_is_the_width_of_arial() {
        // Arial's advance widths, in thousandths of the size: "noncommercial"
        // bold is 7,391, "lost in 200 kills" regular 6,502.
        assert!((badge_text_width("noncommercial", 16.0, true) - 118.256).abs() < 1e-9);
        assert!((badge_text_width("lost in 200 kills", 14.0, false) - 91.028).abs() < 1e-9);
        assert!((badge_text_width("0/2,683", 16.0, true) - 53.376).abs() < 1e-9);
        // Anything else counts as wide as the widest capital.
        assert!((badge_text_width("W", 10.0, false) - 9.44).abs() < 1e-9);
        // Bold is never narrower than regular.
        for c in 'a'..='z' {
            let c = c.to_string();
            assert!(badge_text_width(&c, 10.0, true) >= badge_text_width(&c, 10.0, false));
        }
    }
    #[test]
    fn badges_link_to_the_files_that_state_their_numbers() {
        let row = badges_row().unwrap();
        let c = badge_counts();
        let cells = badge_cells(&row);
        assert_eq!(cells.len(), c.values.len());
        let read = |cell: &str| {
            let link = cell.split('"').next().unwrap();
            (link.to_string(), fs::read_to_string(link).unwrap())
        };
        let count =
            |text: &str, keep: &dyn Fn(&str) -> bool| text.lines().filter(|l| keep(l)).count();
        for (i, cell) in cells.iter().enumerate() {
            let (link, text) = read(cell);
            let v = &c.values[i];
            let ok = match i {
                // The record's evidence directory names all its runs.
                0 => text.contains(&format!("all {} runs", c.runs)),
                // The matrix lists one row per mutant read with the original pin.
                1 => {
                    let rows = count(&text, &|l| l.split('\t').nth(4) == Some("original"));
                    grouped(rows) == *v
                }
                // The crash series states kills, confirmed snapshots and lost.
                2 => {
                    let (lost, ack) = v.split_once('/').unwrap();
                    text.contains(&format!("| {} | {ack} | **{lost}** |", grouped(c.kills)))
                }
                // Cargo.lock lists the packages.
                3 => count(&text, &|l| l.starts_with("name = ")).to_string() == *v,
                // The re-run states the cases and those faster in both runs.
                4 => {
                    let (faster, cases) = v.split_once('/').unwrap();
                    text.contains(&format!("{faster} of them beyond their band"))
                        && text.contains(&format!("| {cases} |"))
                }
                // The commitments list one dated row each.
                5 => count(&text, &|l| l.starts_with("| 2")).to_string() == *v,
                6 => text.contains(&format!("v{v}")),
                7 => text.lines().next().unwrap().contains("Noncommercial"),
                _ => false,
            };
            assert!(ok, "{link} does not state {v}");
        }
    }
    // ===== PHONE TEST a5 END =====

    // ===== TEST status BEGIN =====
    /// Classes animated in a card's style that the reduced-motion rule does not
    /// still, and base rules (outside keyframes) that hide content.
    fn status_motion(svg: &str) -> (Vec<String>, Vec<String>) {
        let style = svg.split_once("<style>").unwrap().1;
        let style = style.split_once("</style>").unwrap().0;
        let (main, reduced) = style
            .split_once("@media (prefers-reduced-motion:reduce){")
            .expect("a reduced-motion rule");
        let stilled: Vec<&str> = reduced
            .split_once("{animation:none}")
            .expect("animation:none")
            .0
            .split(',')
            .map(|s| s.trim().trim_start_matches('.'))
            .collect();
        let (mut missing, mut hidden, mut rest) = (Vec::new(), Vec::new(), main);
        while let Some((selector, after)) = rest.split_once('{') {
            if selector.trim_start().starts_with("@keyframes") {
                rest = after.split_once("}}").unwrap().1;
                continue;
            }
            let (body, after) = after.split_once('}').unwrap();
            for name in selector.split(',') {
                let name = name.trim().trim_start_matches('.');
                if body.contains("animation") && !stilled.contains(&name) {
                    missing.push(name.to_string());
                }
                if body.contains("opacity:0") || body.contains("display:none") {
                    hidden.push(name.to_string());
                }
            }
            rest = after;
        }
        (missing, hidden)
    }
    /// The drawn count of published commitments.
    fn status_count(svg: &str) -> String {
        let at = svg.split_once(r#"<text class="sn b""#).unwrap().1;
        at.split_once('>')
            .unwrap()
            .1
            .split_once('<')
            .unwrap()
            .0
            .to_string()
    }
    #[test]
    fn status_card_counts_the_published_commitments() {
        let text = fs::read_to_string("docs/COMMITMENTS.md").unwrap();
        let table = text.split_once("## Published commitments\n").unwrap().1;
        let dated = table
            .lines()
            .take_while(|l| l.is_empty() || l.starts_with('|'))
            .filter(|l| {
                l.split('|').nth(1).is_some_and(|d| {
                    let d = d.trim().as_bytes();
                    d.len() == 10
                        && d.iter().enumerate().all(|(i, c)| match i {
                            4 | 7 => *c == b'-',
                            _ => c.is_ascii_digit(),
                        })
                })
            })
            .count();
        assert!(dated > 0);
        let f = StatusFacts::read().unwrap();
        assert_eq!(f.commitments, dated);
        for dark in [false, true] {
            let s = status_card(dark).unwrap();
            assert_eq!(status_count(&s), dated.to_string());
            let mut wants: Vec<&str> = STATUS_COLUMNS.iter().map(|c| c.0).collect();
            wants.extend(STATUS_FEET);
            wants.extend(STATUS_PLEDGE);
            for want in wants {
                assert!(s.contains(&format!(">{}<", xml(want))), "{want}");
            }
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—→×✓".contains(ch)));
        }
        let alt = status_alt().unwrap();
        assert!(
            alt.contains(&format!("{dated} published commitments")),
            "{alt}"
        );
        // Some labels say what their commitment holds, so the card does not
        // claim that none does.
        assert!(alt.contains("; only their labels say what each holds."));
        assert!(!alt.contains("which holds what"));
        assert!(alt.chars().count() <= 300 && !alt.contains('"'), "{alt}");
        assert!(status_card(false).unwrap().contains(&xml(&alt)));
        assert!(status_phone(false).unwrap().contains(&xml(&alt)));
    }
    /// The kind of a commitment its label names: "Private data set 3" is a
    /// data set, "Private source tree 1: Rust sources only" a source tree.
    fn status_kind(label: &str) -> String {
        let head = label.split(':').next().unwrap();
        let head = head.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ');
        let low = head.to_lowercase();
        low.strip_prefix("private ").unwrap_or(&low).to_string()
    }
    /// The kinds the README's Private row names, checked against the labels of
    /// a commitments table: each kind named, one with "a" or "the", more in the plural.
    fn status_kinds_named(row: &str, table: &str) -> std::result::Result<usize, String> {
        let low = row.to_lowercase();
        let named = low
            .split_once("fix the exact contents of ")
            .and_then(|(_, r)| r.split_once(" as of their dates"))
            .ok_or("the Private row has no list of what is fixed")?
            .0
            .to_string();
        let mut kinds: Vec<(String, usize)> = Vec::new();
        for line in table.lines().filter(|l| l.starts_with("| 2")) {
            let kind = status_kind(line.split('|').nth(2).unwrap().trim());
            match kinds.iter_mut().find(|k| k.0 == kind) {
                Some(k) => k.1 += 1,
                None => kinds.push((kind, 1)),
            }
        }
        for (kind, n) in &kinds {
            let plural = named.contains(&format!("{kind}s"));
            let one =
                named.contains(&format!("a {kind}")) || named.contains(&format!("the {kind}"));
            if (*n == 1 && (plural || !one)) || (*n > 1 && !plural) {
                return Err(format!(
                    "{n} commitments of kind {kind:?}; the row says {named:?}"
                ));
            }
        }
        Ok(kinds.len())
    }
    #[test]
    fn status_readme_row_names_every_kind_of_commitment() {
        let row = NATIVE
            .lines()
            .find(|l| l.starts_with("| **Private** |"))
            .unwrap();
        let text = fs::read_to_string("docs/COMMITMENTS.md").unwrap();
        let table = text.split_once("## Published commitments\n").unwrap().1;
        let table = table.split("\n\n").next().unwrap();
        assert!(status_kinds_named(row, table).unwrap() >= 4);
        assert_eq!(
            status_kind("Private source tree 1: Rust sources only"),
            "source tree"
        );
        assert_eq!(
            status_kind("Target Q8 Quad quantizer"),
            "target q8 quad quantizer"
        );
        // A new kind of commitment, or a second one of a kind named as one, is
        // caught until the row names it.
        let more = |line: &str| format!("{table}\n{line}");
        let hash = format!("`{}`", "0".repeat(64));
        for line in [
            format!("| 2027-01-01 | Private model 1 | 1 | 1 | {hash} |"),
            format!("| 2027-01-01 | Sealed sample 2 | 1 | 1 | {hash} |"),
        ] {
            assert!(status_kinds_named(row, &more(&line)).is_err(), "{line}");
        }
        assert!(row.contains(" and the target Q8 Quad quantizer as of their dates;"));
        assert!(row.contains("; only their labels say what each holds. |"));
    }
    #[test]
    fn status_card_keeps_the_words_of_the_readme() {
        // Every item is quoted from its own row, so the picture cannot claim more.
        status_rows(NATIVE).unwrap();
        for (from, to) in [
            // A heading elsewhere still says it; the row is what counts.
            (
                "a killed process losing nothing it confirmed",
                "a killed process may lose what it confirmed",
            ),
            ("the private build that answers from them", "the build"),
            (
                "power-cut durability of snapshots and of the record history",
                "power-cut durability of snapshots",
            ),
            ("One command each", "Several commands"),
            ("are `NOT_ESTABLISHED`", "are open"),
            // No line under the card, or no row of a column.
            ("The text AI is a goal, not a result (", "The text AI ("),
            ("| **Private** |", "| **Closed** |"),
        ] {
            assert!(NATIVE.contains(from), "{from}");
            let changed = NATIVE.replace(from, to);
            assert!(status_rows(&changed).is_err(), "{from}");
        }
        // The rewritten row alone fails, though the section heading keeps the phrase.
        let row = NATIVE.replace(
            "a killed process losing nothing it confirmed",
            "a killed process may lose what it confirmed",
        );
        assert!(row.contains("A killed process loses nothing it confirmed"));
    }
    #[test]
    fn status_card_draws_another_count_and_refuses_none() {
        let s = status_draw(true, &StatusFacts { commitments: 23 }).unwrap();
        assert_eq!(status_count(&s), "23");
        assert!(s.contains("23 published commitments fix the contents"));
        assert_ne!(s, status_card(true).unwrap());
        assert!(status_draw(false, &StatusFacts { commitments: 0 }).is_err());
        assert!(StatusFacts { commitments: 0 }.check().is_err());
        // A count too long for its box is refused, not drawn over the edge.
        let e = status_draw(false, &StatusFacts { commitments: 123 }).unwrap_err();
        assert!(e.to_string().contains("does not fit"), "{e}");
    }
    #[test]
    fn status_card_moves_and_can_stand_still() {
        let (light, dark) = (status_card(false).unwrap(), status_card(true).unwrap());
        assert_ne!(light, dark);
        assert!(light.contains(LIGHT.good) && dark.contains(DARK.good));
        let items: usize = STATUS_COLUMNS.iter().map(|c| c.2.len()).sum();
        for s in [&light, &dark] {
            assert_eq!(status_motion(s), (Vec::new(), Vec::new()));
            assert_eq!(s.matches(r#"<g class="sc""#).count(), STATUS_COLUMNS.len());
            assert_eq!(s.matches(r#"class="sd""#).count(), STATUS_COLUMNS.len());
            assert_eq!(s.matches(r#"<g class="si""#).count(), items);
            // Each coloured bar is clipped to its rounded column.
            assert_eq!(
                s.matches("clip-path=\"url(#stcol").count(),
                STATUS_COLUMNS.len()
            );
            let h: f64 = s
                .split_once("height=\"")
                .unwrap()
                .1
                .split_once('"')
                .unwrap()
                .0
                .parse()
                .unwrap();
            assert!((400.0..=460.0).contains(&h), "{h}");
        }
        // The checker catches an animated class left out, and a base rule that hides.
        let broken = light.replace("{.sc,.si,", "{.si,");
        assert_eq!(status_motion(&broken).0, ["sc"]);
        let hiding = light.replace(".kick{font-size:16px}", ".kick{opacity:0}");
        assert_eq!(status_motion(&hiding).1, ["kick"]);
    }
    // ===== TEST status END =====

    // ===== TEST terminal BEGIN =====
    /// The README's check block and table, cut out of NATIVE, so a mutant
    /// changes one line of the real text.
    fn terminal_part() -> String {
        let from = NATIVE.find("```sh").unwrap();
        let to = NATIVE[from..].find("</details>").unwrap() + from;
        NATIVE[from..to].to_string()
    }
    /// Counted another way: the code block lines after the fetch.
    fn terminal_runs() -> Vec<String> {
        terminal_part()
            .lines()
            .skip_while(|l| *l != "cargo fetch --locked")
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .map(str::to_string)
            .collect()
    }
    /// Counted another way: the table rows between the header and the end of the details.
    fn terminal_rows() -> Vec<String> {
        terminal_part()
            .lines()
            .skip_while(|l| !l.starts_with("| Command |"))
            .skip(2)
            .take_while(|l| l.starts_with('|'))
            .map(str::to_string)
            .collect()
    }
    #[test]
    fn terminal_card_shows_every_command_and_line_of_the_readme() {
        let checks = terminal_checks(NATIVE).unwrap();
        let (runs, rows) = (terminal_runs(), terminal_rows());
        assert!(!runs.is_empty() && runs.iter().all(|r| r.starts_with("cargo run ")));
        assert_eq!(checks.len(), runs.len());
        assert_eq!(rows.len(), runs.len());
        // Each drawn name is the first cell of a table row, each row drawn once.
        let mut firsts: Vec<String> = rows
            .iter()
            .map(|r| {
                r.split('|')
                    .nth(1)
                    .unwrap()
                    .replace('`', "")
                    .trim()
                    .to_string()
            })
            .collect();
        let mut keys: Vec<String> = checks.iter().map(|c| c.key.clone()).collect();
        firsts.sort();
        keys.sort();
        assert_eq!(keys, firsts);
        assert_eq!(
            checks[0].look,
            [
                (String::new(), "ROUNDTRIP=4/4 DIFFERENT_BITS=0".to_string()),
                (",".to_string(), "TAMPER=REJECTED".to_string())
            ]
        );
        assert_eq!(checks[3].look[1].0, "with");
        let alt = terminal_alt().unwrap();
        assert!(!alt.contains('"') && alt.starts_with("Drawn") && alt.len() <= 300);
        for dark in [false, true] {
            let s = terminal_card(dark).unwrap();
            for (c, run) in checks.iter().zip(&runs) {
                // The full command is checked against the code block, not drawn.
                assert_eq!(c.command, *run);
                assert!(run.contains(c.key.as_str()));
                assert!(s.contains(&format!(">{}</text>", xml(&c.key))), "{run}");
                assert!(!s.contains(run.as_str()) && alt.contains(c.key.as_str()));
                for (_, line) in &c.look {
                    assert!(s.contains(&format!(">{line}</text>")), "{line}");
                    assert!(alt.contains(line.as_str()), "{line}");
                    assert!(rows.iter().any(|r| r.contains(&format!("`{line}`"))));
                }
            }
            for line in [
                "ROUNDTRIP=4/4 DIFFERENT_BITS=0",
                "TAMPER=REJECTED",
                "Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED",
                "MUTATION_MATRIX=PASS mutants=179",
                "CRASH_SERIES=PASS",
                "acknowledged_lost=0",
                "GEL_VERIFY_ALL=PASS",
            ] {
                assert!(s.contains(&format!(">{line}<")), "{line}");
            }
            // One check mark per command, and the joining word of the table.
            assert!(s.contains(">with</text>") && s.matches("l7 7 13-15").count() == runs.len());
            assert!(s.contains("Run it yourself — drawn, not a recording"));
            assert!(s.contains("not a recording"));
            assert!(s.contains(&format!(">{} COMMANDS · WHAT TO LOOK FOR<", runs.len())));
            assert!(!s.contains("LINES TO LOOK FOR"));
            assert!(s.contains("this README's code block and table"));
            // A checklist, not a session: no prompt and no output beyond the lines.
            assert!(!s.contains(">$") && !s.contains("$ cargo"));
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—→".contains(ch)));
        }
    }
    #[test]
    fn terminal_card_follows_a_changed_template() {
        let part = terminal_part();
        let changed = part
            .replace(
                "-- crash-series 20",
                "-- crash-series 7\ncargo run --locked --offline -p xtask -- audit",
            )
            .replace("| `crash-series 20` |", "| `crash-series 7` |")
            .replace(
                "`CRASH_SERIES=PASS` with `acknowledged_lost=0`",
                "`CRASH_SERIES=PASS` with `acknowledged_lost=3`",
            )
            .replace(
                "| `verify` |",
                "| `audit` | `AUDIT=PASS` | a | b |\n| `verify` |",
            );
        let checks = terminal_checks(&changed).unwrap();
        assert_eq!(checks.len(), 6);
        assert_eq!(
            checks[4].command,
            "cargo run --locked --offline -p xtask -- audit"
        );
        assert_eq!(checks[4].look, [(String::new(), "AUDIT=PASS".to_string())]);
        // A hand-made check list draws its own lines.
        let made = [TerminalCheck {
            key: "probe <a&b>".to_string(),
            command: "cargo run --locked --offline -p xtask -- probe <a&b>".to_string(),
            look: vec![(String::new(), "PROBE=PASS rows=12".to_string())],
        }];
        for dark in [false, true] {
            let s = terminal_draw(dark, &checks).unwrap();
            assert!(s.contains(">acknowledged_lost=3<") && !s.contains(">acknowledged_lost=0<"));
            assert!(s.contains(">crash-series 7</text>") && !s.contains("crash-series 20"));
            assert!(s.contains(">AUDIT=PASS<") && s.contains(">audit</text>"));
            assert!(s.contains(">6 COMMANDS ·"));
            // With no free cell the note goes on one line under the grid.
            assert!(s.contains("read from this README's code block and table<"));
            let m = terminal_draw(dark, &made).unwrap();
            assert!(
                m.contains(">PROBE=PASS rows=12<") && m.contains(">probe &lt;a&amp;b&gt;</text>")
            );
            assert!(m.contains("probe &lt;a&amp;b&gt;: PROBE=PASS rows=12"));
            assert!(!m.contains("GEL_VERIFY_ALL") && m.contains(">1 COMMANDS ·"));
            assert!(!m.contains("<a&b>"));
        }
        assert!(terminal_text(&made).contains("probe <a&b>: PROBE=PASS rows=12"));
    }
    #[test]
    fn terminal_card_refuses_a_template_that_does_not_add_up() {
        let part = terminal_part();
        let verify_row = terminal_rows()
            .into_iter()
            .find(|r| r.starts_with("| `verify` |"))
            .unwrap();
        let verify_run = "cargo run --locked --offline -p xtask -- verify\n";
        assert!(part.contains(verify_run));
        // Each mutant with the words of the refusal it must reach.
        let broken = [
            (
                part.replace(
                    verify_run,
                    &format!("{verify_run}cargo run -p xtask -- audit\n"),
                ),
                "cargo run lines but",
            ),
            (part.replace(verify_run, ""), "cargo run lines but"),
            (
                part.replace("| `verify` |", "| `audit` |"),
                "0 table rows name",
            ),
            (
                part.replace("| `verify` |", "| `xtask` |"),
                "2 table rows name",
            ),
            (
                part.replace(verify_run, &format!("{verify_run}{verify_run}"))
                    .replace(
                        &verify_row,
                        &format!("{verify_row}\n| `probe` | `P=1` | a | b |"),
                    ),
                "names two commands",
            ),
            (
                part.replace("| `GEL_VERIFY_ALL=PASS` |", "| GEL_VERIFY_ALL=PASS |"),
                "no line to look for",
            ),
            (
                part.replace("| `GEL_VERIFY_ALL=PASS` |", "| `GEL_VERIFY_ALL=PASS` ` |"),
                "no line to look for",
            ),
            (
                part.replace("| `GEL_VERIFY_ALL=PASS` |", "| `` |"),
                "an empty code span",
            ),
            (
                part.replace(
                    "`MUTATION_MATRIX=PASS mutants=179`",
                    "`MUTATION_MATRIX=PASS` or so",
                ),
                "text outside the code",
            ),
            (
                part.replace("| `GEL_VERIFY_ALL=PASS` |", "| see `GEL_VERIFY_ALL=PASS` |"),
                "text outside the code",
            ),
            (
                part.replace(&verify_row, "| `verify` |"),
                "without a line to look for",
            ),
            (part.replace("| `verify` |", "|  |"), "without a command"),
            (
                part.replace(verify_run, &format!("{verify_run}echo done\n")),
                "not a cargo run line",
            ),
            (part.replace("```sh", "```text"), "not in the sh block"),
            (part.replacen("```sh\n", "", 1), "outside a code block"),
            (
                part.replace("cargo fetch --locked", "cargo fetch"),
                "no cargo fetch",
            ),
            (String::new(), "no cargo fetch"),
            (
                "```sh\ncargo fetch --locked\ncargo run x\n".to_string(),
                "not closed",
            ),
            (
                "```sh\ncargo fetch --locked\n```\n".to_string(),
                "not closed",
            ),
            (
                part.replace("What each line shows, and its limit", "Notes"),
                "no table of the lines",
            ),
            (
                part.replace("| Command | Line to look for |", "| Command | Output |"),
                "no Command / Line",
            ),
            (part.replace("|:---|:---|:---|:---|", ""), "no separator"),
        ];
        for (i, (text, words)) in broken.iter().enumerate() {
            let e = terminal_checks(text).unwrap_err().to_string();
            assert!(e.contains(words), "mutant {i}: {e}");
        }
        let checks = terminal_checks(NATIVE).unwrap();
        let drawn =
            |c: &TerminalCheck| terminal_draw(false, &[c.clone()]).map_err(|e| e.to_string());
        assert!(terminal_draw(false, &[])
            .unwrap_err()
            .to_string()
            .contains("no checks"));
        // The longest name and the widest lines of the README fit with room to spare.
        for c in &checks {
            assert!(drawn(c).is_ok(), "{}", c.key);
        }
        let mut long = checks[0].clone();
        long.key = "x".repeat(29);
        assert!(drawn(&long).unwrap_err().contains("name does not fit"));
        long.key = "x".repeat(28);
        assert!(drawn(&long).is_ok());
        long.look.push(("with".to_string(), "X".repeat(4)));
        assert!(drawn(&long).unwrap_err().contains("do not fit"));
    }
    #[test]
    fn terminal_card_moves_and_can_stand_still() {
        let (light, dark) = (terminal_card(false).unwrap(), terminal_card(true).unwrap());
        assert_ne!(light, dark);
        for s in [light, dark] {
            let style = &s[s.find("<style>").unwrap()..s.find("</style>").unwrap()];
            let (moving, quiet) = style
                .split_once("@media (prefers-reduced-motion:reduce){")
                .unwrap();
            assert!(quiet.contains("{animation:none}"));
            let names: Vec<&str> = quiet[..quiet.find("{animation:none}").unwrap()]
                .split(',')
                .collect();
            // Every class that animates is quieted under reduced motion, and
            // the whole sequence is over within six seconds.
            let mut animated = 0;
            let mut end: f64 = 0.0;
            for rule in moving.split('}') {
                if let Some((sel, body)) = rule.split_once('{') {
                    if let Some(value) = body.strip_prefix("animation:") {
                        animated += 1;
                        assert!(names.contains(&sel.trim()), "{sel}");
                        for one in value.split(',') {
                            let times: Vec<f64> = one
                                .split_whitespace()
                                .filter_map(|w| w.strip_suffix('s')?.parse().ok())
                                .collect();
                            assert_eq!(times.len(), 2, "{one}");
                            end = end.max(times[0] + times[1]);
                        }
                    }
                }
            }
            // Four animated parts per command, counted from the code block.
            assert_eq!(animated, 4 * terminal_runs().len());
            assert_eq!(names.len(), animated);
            assert!(end > 3.0 && end <= 6.0, "{end}");
            // Typing ends on the last letter: one step per character of the name.
            for c in terminal_checks(NATIVE).unwrap() {
                assert!(style.contains(&format!("steps({})", c.key.chars().count())));
            }
            assert!(s.contains("@keyframes ttw{from{transform:scaleX(1)}"));
            // Without any animation the final state shows: the typing covers
            // are already shrunk in their base rule, and nothing else but the
            // caret starts hidden there.
            assert!(moving.contains(
                ".tv{transform-box:fill-box;transform-origin:right;transform:scaleX(0)}"
            ));
            assert_eq!(
                moving.matches("opacity:0}").count(),
                2,
                "only .tk and the tcs frames"
            );
            assert!(moving.contains(".tk{opacity:0}"));
        }
    }
    // ===== TEST terminal END =====

    // ===== TEST rerun BEGIN =====
    fn rerun_case(name: &str, aa: f64, gain: f64) -> SpeedCase {
        SpeedCase {
            name: name.to_string(),
            aa,
            gain,
            runs: 27,
        }
    }
    fn rerun_mutant() -> RerunData {
        RerunData {
            base: "v9.9".to_string(),
            runs: [
                RerunRun {
                    date: "2031-01-02".to_string(),
                    sides: [7, 1, 3],
                    noise: 0.25,
                },
                RerunRun {
                    date: "2031-02-03".to_string(),
                    sides: [2, 6, 3],
                    noise: 0.0125,
                },
            ],
            cases: 11,
            again: 2,
            above_one: 5,
        }
    }
    /// A made-up origin and a CAMPAIGN.txt that it accepts, with examples p and q.
    fn rerun_fake_origin() -> RerunOrigin {
        RerunOrigin {
            set_dir: "docs/t/2031-01-02-against-v9.9".to_string(),
            base: "v9.9".to_string(),
            base_rev: "ab".repeat(20),
            dates: ("2031-01-02".to_string(), "2031-02-03".to_string()),
        }
    }
    fn rerun_fake_campaign() -> String {
        let rev = "ab".repeat(20);
        [
            "FORMAT=GEL_SPEED_CAMPAIGN_1".to_string(),
            "reruns=docs/t/2031-01-02-against-v9.9".to_string(),
            "timing_validity=AS_MEASURED".to_string(),
            format!("tree=a revision={rev} working_tree=clean"),
            format!("tree=a2 revision={rev} working_tree=clean"),
            format!("tree=b revision={SPEED_B}0123 working_tree=clean"),
            "example=p expect=same_output verdict=SAME_OUTPUT speed_ab=ok check=PASS met=yes"
                .to_string(),
            "example=q expect=same_output verdict=SAME_OUTPUT speed_ab=ok check=PASS met=yes"
                .to_string(),
            "SPEED_CAMPAIGN=COMPLETE".to_string(),
        ]
        .join("\n")
    }
    #[test]
    fn rerun_card_follows_the_two_timing_records() {
        let d = rerun_read().unwrap();
        assert_eq!(d.cases, 133);
        assert_eq!(d.runs[0].sides, [103, 12, 18]);
        assert_eq!(d.runs[1].sides, [105, 4, 24]);
        assert_eq!((d.again, d.above_one), (100, 102));
        let dates = (d.runs[0].date.as_str(), d.runs[1].date.as_str());
        assert_eq!(dates, ("2026-10-20", "2026-11-04"));
        let noise = (rerun_number(d.runs[0].noise), rerun_number(d.runs[1].noise));
        assert_eq!(noise, ("0.04".to_string(), "0.004".to_string()));
        // The base, as the set names it, and the A revision the campaign gives.
        assert_eq!(d.base, "v0.6.1");
        let campaign = fs::read_to_string(format!("{RERUN_DIR}/CAMPAIGN.txt")).unwrap();
        let readme = fs::read_to_string(format!("{SPEED_DIR}/README.md")).unwrap();
        let o = rerun_origin(SPEED_DIR, RERUN_DIR, &readme).unwrap();
        assert!(campaign.contains(&format!("tree=a2 revision={} ", o.base_rev)));
        // The record agrees with the speed card's own counts: faster, slower, inside.
        assert_eq!(SpeedSet::read().unwrap().sides(), [103, 18, 12]);
        // Counted another way: the case lines of the re-run, beside its warm-up rounds.
        let (mut lines, mut warmups) = (0, 0);
        for entry in fs::read_dir(RERUN_DIR).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let text = fs::read_to_string(path.join("summary.txt")).unwrap();
                lines += text
                    .lines()
                    .filter(|l| l.starts_with("SPEED_AB case="))
                    .count();
                warmups += usize::from(path.join("w-b-r1.txt").is_file());
            }
        }
        assert_eq!((lines, warmups), (133, 23));
        for dark in [false, true] {
            let s = rerun_card(dark).unwrap();
            for want in [
                "<title id=\"t\">The same 133 cases, timed twice</title>",
                ">The same 133 cases, timed twice<",
                ">v0.6.1 time / new time, each case against its own A/A band<",
                ">103<",
                ">12<",
                ">18<",
                ">105<",
                ">4<",
                ">24<",
                ">133 cases<",
                ">slower: 18 → 24<",
                ">2026-10-20<",
                ">2026-11-04<",
                ">100 of 103 faster again<",
                ">102 with a ratio above 1<",
                ">0.04<",
                ">0.004<",
                ">0.04 → 0.004<",
                "One machine, not pinned to cores · A, A2: two checkouts of v0.6.1 · B: main at 876247f<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            // The headline is the card's own rule, not the looser ratio above 1.
            assert!(!s.contains(">102 of 103"));
            assert!(s.contains(&format!("<desc id=\"d\">{}</desc>", rerun_alt().unwrap())));
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—→×✓".contains(ch)));
        }
        let alt = rerun_alt().unwrap();
        for want in [
            "The same 133 cases, timed twice against v0.6.1",
            "103 faster, 12 inside, 18 slower on 2026-10-20",
            "105 faster, 4 inside, 24 slower on 2026-11-04",
            "100 of the 103 faster again (102 with a ratio above 1)",
            "A/A noise 0.04 → 0.004",
            "One machine, not pinned to cores.",
        ] {
            assert!(alt.contains(want), "{want}");
        }
        assert!(alt.chars().count() <= 300, "{}", alt.chars().count());
        assert!(!alt.contains('"') && !alt.contains('<') && !alt.contains('&'));
        assert!(visual_picture("rerun").contains(&alt));
    }
    #[test]
    fn rerun_card_names_no_case_and_no_program() {
        let mut names = Vec::new();
        for dir in [SPEED_DIR, RERUN_DIR] {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if !path.is_dir() {
                    continue;
                }
                names.push(path.file_name().unwrap().to_string_lossy().into_owned());
                if let Ok(text) = fs::read_to_string(path.join("summary.txt")) {
                    for line in text.lines() {
                        if let Some(rest) = line.strip_prefix("SPEED_AB case=") {
                            names.push(rest.split(' ').next().unwrap().to_string());
                        }
                    }
                }
            }
        }
        assert_eq!(names.len(), 24 + 133 + 23 + 133);
        let alt = rerun_alt().unwrap();
        for dark in [false, true] {
            let c = rerun_card(dark).unwrap();
            for name in &names {
                assert!(!c.contains(&format!(">{name}<")), "{name}");
                if name.contains('_') || name.bytes().any(|b| b.is_ascii_digit()) {
                    assert!(
                        !c.contains(name.as_str()) && !alt.contains(name.as_str()),
                        "{name}"
                    );
                }
            }
        }
    }
    #[test]
    fn rerun_card_draws_the_numbers_it_is_given() {
        let d = rerun_mutant();
        let s = rerun_draw(false, &d).unwrap();
        for want in [
            ">7<",
            ">1<",
            ">3<",
            ">2<",
            ">6<",
            ">11 cases<",
            ">slower: 3 → 3<",
            ">2031-01-02<",
            ">2031-02-03<",
            ">2 of 7 faster again<",
            ">5 with a ratio above 1<",
            ">0.25 → 0.0125<",
            ">The same 11 cases, timed twice<",
            ">v9.9 time / new time, each case against its own A/A band<",
            "two checkouts of v9.9 ·",
        ] {
            assert!(s.contains(want), "{want}");
        }
        for real in [
            ">103<",
            ">105<",
            ">100 of 103",
            "0.004",
            "2026-11-04",
            ">133 cases<",
            "v0.6.1",
        ] {
            assert!(!s.contains(real), "{real}");
        }
        let alt = rerun_text(&d).unwrap();
        assert!(alt.starts_with("The same 11 cases, timed twice against v9.9,"));
        assert!(alt.contains("7 faster, 1 inside, 3 slower on 2031-01-02"));
        assert!(alt.contains("2 faster, 6 inside, 3 slower on 2031-02-03"));
        assert!(alt.contains("2 of the 7 faster again (5 with a ratio above 1)"));
        // The segments are as wide as their counts: 700 px over 11 cases.
        assert!(s.contains(&format!(r#"width="{:.1}" height="56""#, 7.0 * 700.0 / 11.0)));
        // Text from the data is escaped.
        let mut odd = rerun_mutant();
        odd.base = "<v&\">".to_string();
        let s = rerun_draw(true, &odd).unwrap();
        assert!(s.contains(">&lt;v&amp;&quot;&gt; time / new time") && !s.contains("<v&"));
        // Paired by name: the order of the programs does not matter.
        let (a, b) = (
            vec![rerun_case("x", 1.0, 2.0), rerun_case("y", 0.9, 1.0)],
            vec![rerun_case("z", 1.0, 0.5)],
        );
        let (c, e) = (
            vec![rerun_case("z", 1.01, 1.001)],
            vec![rerun_case("y", 0.99, 0.5), rerun_case("x", 1.002, 1.5)],
        );
        let p = rerun_compare(
            "v9.9",
            ("2031-01-02", "2031-02-03"),
            &[("p", &a, true), ("q", &b, false)],
            &[("q", &c, false), ("p", &e, true)],
        )
        .unwrap();
        assert_eq!(p.runs[0].sides, [1, 1, 1]);
        assert_eq!(p.runs[1].sides, [1, 1, 1]);
        assert_eq!((p.cases, p.again, p.above_one), (3, 1, 1));
        // Faster before, a ratio above 1 now but inside its band: counted apart.
        let (f, g) = (
            vec![rerun_case("x", 1.0, 2.0)],
            vec![rerun_case("x", 0.99, 1.005)],
        );
        let q = rerun_compare("v9.9", ("a", "b"), &[("p", &f, true)], &[("p", &g, true)]).unwrap();
        assert_eq!((q.again, q.above_one), (0, 1));
        assert_eq!(rerun_number(p.runs[0].noise), "0");
        assert_eq!(rerun_number(p.runs[1].noise), "0.01");
    }
    #[test]
    fn rerun_card_refuses_data_that_does_not_add_up() {
        let dates = ("2031-01-02", "2031-02-03");
        let one = vec![rerun_case("x", 1.0, 2.0), rerun_case("y", 1.0, 2.0)];
        let renamed = vec![rerun_case("x", 1.0, 2.0), rerun_case("w", 1.0, 2.0)];
        let fewer = vec![rerun_case("x", 1.0, 2.0)];
        let twice = vec![rerun_case("x", 1.0, 2.0), rerun_case("x", 1.0, 2.0)];
        let ok = |a: &[RerunProgram], b: &[RerunProgram]| rerun_compare("v9.9", dates, a, b);
        assert!(ok(&[("p", &one, true)], &[("p", &one, true)]).is_ok());
        for (old, new) in [
            (vec![("p", &one[..], true)], vec![("p", &renamed[..], true)]),
            (vec![("p", &one[..], true)], vec![("p", &fewer[..], true)]),
            (vec![("p", &one[..], true)], vec![("q", &one[..], true)]),
            (vec![("p", &twice[..], true)], vec![("p", &twice[..], true)]),
            (vec![("p", &[][..], true)], vec![("p", &[][..], true)]),
            (vec![], vec![]),
        ] {
            assert!(ok(&old, &new).is_err());
        }
        // The same cases, but a program's output verdict changed between the runs.
        let e = ok(&[("p", &one, true)], &[("p", &one, false)])
            .err()
            .unwrap();
        assert!(
            e.to_string().starts_with("p: the output verdict differs"),
            "{e}"
        );
        let broken: [fn(&mut RerunData); 9] = [
            |d| d.runs[0].sides[1] += 1,
            |d| d.runs[1].sides[2] -= 1,
            |d| d.above_one = d.runs[0].sides[0] + 1,
            |d| d.again = d.above_one + 1,
            |d| d.again = d.runs[1].sides[0] + 1,
            |d| d.runs[1].noise = f64::NAN,
            |d| d.runs[0].noise = -0.5,
            |d| d.base = String::new(),
            |d| {
                d.cases = 0;
                d.runs[0].sides = [0; 3];
                d.runs[1].sides = [0; 3];
                d.again = 0;
                d.above_one = 0;
            },
        ];
        for (k, f) in broken.iter().enumerate() {
            let mut d = rerun_mutant();
            f(&mut d);
            assert!(rerun_draw(false, &d).is_err(), "{k}");
            assert!(rerun_text(&d).is_err(), "{k}");
        }
    }
    #[test]
    fn rerun_card_refuses_records_of_another_origin() {
        let err = |r: Result<RerunOrigin>| r.err().map(|e| e.to_string()).unwrap_or_default();
        let set = "docs/t/2031-01-02-against-v9.9";
        let rev = "ab".repeat(20);
        let readme = format!("# Timing against v9.9, 2031-01-02\n\n| A and A2 | two clean checkouts of the v9.9 tag, `{rev}` |\n");
        let o = rerun_origin(
            set,
            "docs/t/2031-02-03-reruns/rerun-2031-01-02-against-v9.9",
            &readme,
        )
        .unwrap();
        assert_eq!(
            (o.base.as_str(), o.base_rev.as_str()),
            ("v9.9", rev.as_str())
        );
        assert_eq!(
            o.dates,
            ("2031-01-02".to_string(), "2031-02-03".to_string())
        );
        for (set_dir, rerun_dir, readme, why) in [
            (
                set,
                "docs/t/2031-02-03-reruns/rerun-2031-01-02-against-v9.8",
                readme.clone(),
                "is not the re-run of",
            ),
            (
                set,
                "docs/t/2031-02-03-other/rerun-2031-01-02-against-v9.9",
                readme.clone(),
                "is not the re-run of",
            ),
            (
                set,
                "docs/t/2031-2-03-reruns/rerun-2031-01-02-against-v9.9",
                readme.clone(),
                "does not start with a date",
            ),
            (
                "docs/t/2031-1-02-against-v9.9",
                "x/2031-02-03-reruns/rerun-2031-1-02-against-v9.9",
                readme.clone(),
                "does not start with a date",
            ),
            (
                "docs/t/2031-01-02-versus-v9.9",
                "x/2031-02-03-reruns/rerun-2031-01-02-versus-v9.9",
                readme.clone(),
                "does not name its base",
            ),
            (
                "docs/t/2031-01-02-against-",
                "x/2031-02-03-reruns/rerun-2031-01-02-against-",
                readme.clone(),
                "does not name its base",
            ),
            (
                set,
                "docs/t/2031-02-03-reruns/rerun-2031-01-02-against-v9.9",
                readme.replace("v9.9, 2031", "v9.8, 2031"),
                "README.md: its title",
            ),
            (
                set,
                "docs/t/2031-02-03-reruns/rerun-2031-01-02-against-v9.9",
                readme.replace(&rev, &rev[1..]),
                "README.md: no full revision",
            ),
            (
                set,
                "docs/t/2031-02-03-reruns/rerun-2031-01-02-against-v9.9",
                readme.replace("v9.9 tag", "v9.8 tag"),
                "README.md: no full revision",
            ),
        ] {
            let e = err(rerun_origin(set_dir, rerun_dir, &readme));
            assert!(e.contains(why), "{why}: {e}");
        }
        // CAMPAIGN.txt: one refusal per line that is missing or changed.
        let o = rerun_fake_origin();
        let good = rerun_fake_campaign();
        assert_eq!(rerun_campaign(&good, &o).unwrap(), ["p", "q"]);
        let b_line = format!("tree=b revision={SPEED_B}0123 working_tree=clean");
        for (from, to, why) in [
            (
                "FORMAT=GEL_SPEED_CAMPAIGN_1",
                "FORMAT=GEL_SPEED_CAMPAIGN_2",
                "not a GEL_SPEED_CAMPAIGN_1 file",
            ),
            (
                "SPEED_CAMPAIGN=COMPLETE",
                "SPEED_CAMPAIGN=PARTIAL",
                "not complete",
            ),
            (
                "reruns=docs/t/2031-01-02-against-v9.9",
                "reruns=docs/t/other",
                "does not re-run docs/t/2031",
            ),
            (
                "timing_validity=AS_MEASURED",
                "timing_validity=NOT_VALID",
                "not as measured",
            ),
            (
                "tree=a revision=abab",
                "tree=a revision=cdab",
                "A and A2 are not clean",
            ),
            (
                "tree=a2 revision=abab",
                "tree=a2 revision=cdab",
                "A and A2 are not clean",
            ),
            (
                "abab working_tree=clean\ntree=b",
                "abab working_tree=dirty\ntree=b",
                "A and A2 are not clean",
            ),
            (
                &b_line,
                &b_line.replace(SPEED_B, "0000000"),
                "B is not a clean checkout",
            ),
            (
                &b_line,
                &b_line.replace("clean", "dirty"),
                "B is not a clean checkout",
            ),
            (
                "SAME_OUTPUT speed_ab=ok check=PASS met=yes\nexample=q",
                "SAME_OUTPUT speed_ab=ok check=FAIL met=yes\nexample=q",
                "p is not checked",
            ),
            (
                "check=PASS met=yes\nSPEED",
                "check=PASS met=no\nSPEED",
                "q is not checked or not as expected",
            ),
            ("example=q", "example=p", "p is repeated"),
        ] {
            assert!(good.contains(from), "{from}");
            let e = rerun_campaign(&good.replacen(from, to, 1), &o)
                .err()
                .unwrap()
                .to_string();
            assert!(
                e.starts_with("CAMPAIGN.txt: ") && e.contains(why),
                "{why}: {e}"
            );
        }
        let none: Vec<&str> = good
            .lines()
            .filter(|l| !l.starts_with("example="))
            .collect();
        let e = rerun_campaign(&none.join("\n"), &o)
            .err()
            .unwrap()
            .to_string();
        assert_eq!(e, "CAMPAIGN.txt: no example");
        // Manifests: the format and measured timings.
        let manifest = "FORMAT=GEL_SPEED_AB_2\nprotocol=P\ntiming_validity=AS_MEASURED\n";
        assert!(rerun_manifest(manifest).is_ok());
        let e = |m: &str| rerun_manifest(m).err().unwrap().to_string();
        assert!(e(&manifest.replace("AB_2", "AB_1")).contains("not a GEL_SPEED_AB_2 manifest"));
        assert!(e(&manifest.replace("AS_MEASURED", "UNKNOWN")).contains("not as measured"));
        // The program directories against the campaign, and errors that name the file.
        let summary = fs::read_to_string(format!("{RERUN_DIR}/speed_crc/summary.txt")).unwrap();
        let program =
            |name: &str, m: &str, s: &str| (name.to_string(), m.to_string(), s.to_string());
        let both = [
            program("p", manifest, &summary),
            program("q", manifest, &summary),
        ];
        let read = rerun_programs_from(&good, &both, &o).unwrap();
        assert_eq!(read.len(), 2);
        assert!(read
            .iter()
            .all(|(_, cases, same)| cases.len() == 3 && *same));
        let from = |programs: &[(String, String, String)]| {
            rerun_programs_from(&good, programs, &o)
                .err()
                .unwrap()
                .to_string()
        };
        assert!(from(&both[..1]).contains("its examples are not the program directories"));
        let extra = [
            both[0].clone(),
            both[1].clone(),
            program("r", manifest, &summary),
        ];
        assert!(from(&extra).contains("its examples are not the program directories"));
        let bad_manifest = [both[0].clone(), program("q", "FORMAT=X", &summary)];
        assert!(from(&bad_manifest).starts_with("q/manifest.txt: "));
        let bad_summary = [
            program("p", manifest, "SPEED_AB_VERDICT=SAME_OUTPUT"),
            both[1].clone(),
        ];
        assert!(from(&bad_summary).starts_with("p/summary.txt: "));
        // On disk: a directory that is not a program name is refused before any
        // file is read, and a missing file is named.
        let e = rerun_programs("docs/speed-results", &o)
            .err()
            .unwrap()
            .to_string();
        assert!(
            e.starts_with("docs/speed-results/2026-")
                && e.ends_with(": unexpected program directory"),
            "{e}"
        );
        let e = rerun_programs(SPEED_DIR, &o).err().unwrap().to_string();
        assert!(e.starts_with(&format!("{SPEED_DIR}/CAMPAIGN.txt: ")), "{e}");
        let e = rerun_programs(RERUN_DIR, &o).err().unwrap().to_string();
        assert!(e.starts_with(&format!("{RERUN_DIR}/CAMPAIGN.txt: ")), "{e}");
    }
    #[test]
    fn rerun_card_moves_and_can_stand_still() {
        let (light, dark) = (rerun_card(false).unwrap(), rerun_card(true).unwrap());
        assert_ne!(light, dark);
        assert!(light.contains(LIGHT.blue[1]) && dark.contains(DARK.blue[1]));
        for s in [&light, &dark] {
            let style = s
                .split("<style>")
                .nth(1)
                .unwrap()
                .split("</style>")
                .next()
                .unwrap();
            let (moving, still) = style
                .split_once("@media (prefers-reduced-motion:reduce){")
                .unwrap();
            assert!(moving.contains("@keyframes"));
            let still = still.split("{animation:none}").next().unwrap();
            let classes: Vec<&str> = moving
                .split('}')
                .filter(|rule| rule.contains("{animation:"))
                .map(|rule| rule.split('{').next().unwrap().trim_start_matches('.'))
                .collect();
            assert_eq!(classes, ["rn-g", "rn-f", "rn-p", "rn-d"]);
            for class in classes {
                assert!(
                    still.split(',').any(|c| c == format!(".{class}")),
                    "{class}"
                );
                assert!(
                    s.contains(&format!("class=\"{class}")) || s.contains(&format!(" {class}\"")),
                    "{class}"
                );
            }
            // Nothing starts hidden outside its animation.
            assert!(!moving
                .split("@keyframes")
                .next()
                .unwrap()
                .contains("opacity:0"));
            assert!(!s.contains("opacity=\"0\""));
        }
    }
    /// Every text line of a drawing: its left and right ends by a conservative
    /// width estimate with 8 % slack, and its font size.
    fn rerun_lines(svg: &str) -> Vec<(String, f64, f64, f64)> {
        let attr = |head: &str, key: &str| -> Option<String> {
            let rest = head.split(&format!(" {key}=\"")).nth(1)?;
            Some(rest.split('"').next()?.to_string())
        };
        let mut out = Vec::new();
        for part in svg.split("<text").skip(1) {
            let (head, rest) = part.split_once('>').unwrap();
            let raw = rest.split("</text>").next().unwrap();
            // Inner tags (tspans) carry no width; an entity is one character.
            let mut text = String::new();
            let mut inside = false;
            for ch in raw.chars() {
                match ch {
                    '<' => inside = true,
                    '>' => inside = false,
                    _ if !inside => text.push(ch),
                    _ => {}
                }
            }
            for (entity, ch) in [
                ("&lt;", "<"),
                ("&gt;", ">"),
                ("&quot;", "\""),
                ("&amp;", "&"),
            ] {
                text = text.replace(entity, ch);
            }
            let size: f64 = attr(head, "font-size")
                .or_else(|| {
                    Some(
                        head.split("font-size:")
                            .nth(1)?
                            .split("px")
                            .next()?
                            .to_string(),
                    )
                })
                .unwrap()
                .parse()
                .unwrap();
            let spacing = if head.contains("kick") { 2.0 } else { 0.0 };
            let chars = text.chars().count() as f64;
            let w = (rerun_width(&text, size) + spacing * chars) * 1.08;
            let x: f64 = attr(head, "x").unwrap().parse().unwrap();
            let (l, r) = match attr(head, "text-anchor").as_deref() {
                Some("middle") => (x - w / 2.0, x + w / 2.0),
                Some("end") => (x - w, x),
                _ => (x, x + w),
            };
            out.push((text, l, r, size));
        }
        out
    }
    #[test]
    fn rerun_card_is_legible_on_a_phone() {
        let mut odd = rerun_mutant();
        odd.runs[0].sides = [1, 1, 9];
        odd.again = 1;
        odd.above_one = 1;
        odd.runs[1].sides = [9, 1, 1];
        for s in [
            rerun_card(false).unwrap(),
            rerun_card(true).unwrap(),
            rerun_draw(false, &rerun_mutant()).unwrap(),
            rerun_draw(false, &odd).unwrap(),
        ] {
            let lines = rerun_lines(&s);
            assert!(lines.len() > 20);
            for (text, l, r, size) in lines {
                assert!(size >= 16.0, "{text}: {size}");
                assert!(l >= 20.0 && r <= 1180.0, "{text}: {l:.0} to {r:.0}");
            }
        }
        // With no animation at all the final state shows: no base rule hides a
        // thing; only keyframes start from nothing.
        let s = rerun_card(false).unwrap();
        let style = s.split(".b{font-weight:700}").nth(1).unwrap();
        let base = style.split("@keyframes").next().unwrap();
        assert!(
            !base.contains("opacity") && !base.contains("scale"),
            "{base}"
        );
    }
    // ===== TEST rerun END =====

    // ===== TEST glance BEGIN =====
    /// The numbers that roll into place, in drawing order, each checked against
    /// its final characters (class `rf`).
    fn glance_rolled(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        for part in svg.split(r#"<g class="roll" aria-label=""#).skip(1) {
            let (label, rest) = part.split_once('"').unwrap();
            // The roll ends at its own closing tag; digits nest two groups deep.
            let mut finals = String::new();
            let mut depth = 1;
            let mut s = rest;
            while depth > 0 {
                let open = s.find("<g").unwrap_or(usize::MAX);
                let close = s.find("</g>").unwrap();
                let fin = s.find(r#"class="rf""#).unwrap_or(usize::MAX);
                if fin < open && fin < close {
                    let after = &s[fin..];
                    let text = after.split_once('>').unwrap().1;
                    finals.push_str(text.split_once('<').unwrap().0);
                    s = &after[1..];
                } else if open < close {
                    depth += 1;
                    s = &s[open + 2..];
                } else {
                    depth -= 1;
                    s = &s[close + 4..];
                }
            }
            assert_eq!(finals, label, "the final characters spell the label");
            out.push(label.to_string());
        }
        out
    }
    /// Two fault test files of a release 0.9 after 0.8: three tests and an
    /// ignored child process.
    const GLANCE_FAULT_TEXTS: [&str; 2] = [
        "use x;\n#[test]\nfn one() {}\n/// The child.\n#[test]\n#[ignore]\nfn child() {}\n",
        "#[cfg(unix)]\n#[test]\nfn two() {\n    #[cfg(unix)]\n    let x = 1;\n}\nfn helper() {}\n#[test]\nfn three() {}\n",
    ];
    /// Texts of a release 0.9 after 0.8, read the way the files are read.
    fn glance_texts() -> GlanceTexts<'static> {
        GlanceTexts {
            release: "v0.9.2",
            notes: "# GEL RAM v0.9.2 — x\n\n- `xtask witness NEW_DIR` runs the two short\n  public checks; 3 tests in two new files; two processes of each of two\n  operations: `records=2 samples=4`\n",
            roadmap: "# Roadmap\n\n## v0.9.0\n\nnew\n\n## v0.8.0\n\n| Every program prints one identity line for\n`--version`, and for `--license` the license name |\n\n## v0.7.0\n\nold\n",
            binaries: "Every program of the workspace (`a`, `b`\nand `c`) given\n",
            memory: "GEL_MEMORY_1\nsample op=a rep=1\nsample op=a rep=2\nsample op=b rep=1\nsample op=b rep=2\nrecord op=a reps=2 x=1\nrecord op=b reps=2 x=1\nend records=2 samples=4 redactions=0\n",
            faults: &GLANCE_FAULT_TEXTS,
            reproduce: "# R\n\n## A short witness\n\n```text\ncargo run --locked --offline -p xtask -- witness NEW_DIR\n```\n\nIt runs two steps, each as its own process:\n\n| Step | What it runs | Passing line |\n|---|---|---|\n| literal | x | `A=PASS` |\n| same-bits | y | `B=MATCH` |\n\nAfter.\n\n## Report it\n\n| Step | z | w |\n|---|---|---|\n| more | x | y |\n",
            readme: "## New in 0.9, and in 0.8\n\nthe three tool programs answer `--version` and `--license`; `xtask witness NEW_DIR` runs the two short public checks; 3 tests in two new files; two processes of each of two operations, 4 samples ([record](docs/memory-results/2026-11-08/README.md))\n",
        }
    }
    /// The facts of `glance_texts`, made by hand.
    fn glance_other_facts() -> GlanceFacts {
        GlanceFacts {
            release: "0.9".to_string(),
            since: "0.8".to_string(),
            faults: 3,
            operations: 2,
            processes: 2,
            samples: 4,
            steps: 2,
            tools: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        }
    }
    #[test]
    fn glance_card_shows_the_numbers_of_the_files() {
        let f = GlanceFacts::read().unwrap();
        // Counted a second way: each `#[test]` of the fault files, less the
        // ignored child processes.
        let tests: usize = GLANCE_FAULTS
            .iter()
            .map(|file| {
                let text = fs::read_to_string(file).unwrap();
                text.matches("#[test]").count() - text.matches("\n#[ignore]\n").count()
            })
            .sum();
        assert_eq!(f.faults, tests);
        // The memory record as its own README says `memory-check` prints it.
        let record = fs::read_to_string(format!("{GLANCE_MEMORY}/README.md")).unwrap();
        assert!(record.contains(&format!(
            "`GEL_MEMORY_CHECK=PASS records={} samples={}`",
            f.operations, f.samples
        )));
        // The steps as the witness names them, each a row of the guide's table.
        let witness = fs::read_to_string("xtask/src/witness.rs").unwrap();
        let names: Vec<&str> = witness
            .split("            name: \"")
            .skip(1)
            .map(|s| s.split_once('"').unwrap().0)
            .collect();
        assert_eq!(f.steps, names.len());
        let guide = fs::read_to_string(GLANCE_REPRODUCE).unwrap();
        for name in &names {
            assert!(guide.contains(&format!("\n| {name} | ")), "{name}");
        }
        // The programs as the claim registry lists them.
        let claims = fs::read_to_string("xtask/src/claims.rs").unwrap();
        let scope = claims
            .split_once("tool programs gel-")
            .unwrap()
            .1
            .split_once("; not the xtask runner")
            .unwrap()
            .0;
        let mut listed: Vec<String> = format!("gel-{scope}")
            .replace(" and ", ", ")
            .split(", ")
            .map(str::to_string)
            .collect();
        let mut tools = f.tools.clone();
        listed.sort();
        tools.sort();
        assert_eq!(tools, listed);
        // The release, and the earlier version's own roadmap section.
        assert!(RELEASE.starts_with(&format!("v{}.", f.release)));
        let roadmap = fs::read_to_string(GLANCE_ROADMAP).unwrap();
        assert!(roadmap.contains(&format!("\n## v{}.0\n", f.since)));
        assert!(roadmap.contains(&format!("\n## v{}.0\n", f.release)));
        for dark in [false, true] {
            let s = glance_card(dark).unwrap();
            assert_eq!(
                glance_rolled(&s),
                [
                    f.faults.to_string(),
                    f.samples.to_string(),
                    f.steps.to_string(),
                    f.tools.len().to_string(),
                ]
            );
            let kicker = format!("{} AND {} AT A GLANCE", f.since, f.release);
            let new = format!(">new in {}<", f.release);
            let since = format!(">since {}<", f.since);
            assert_eq!(s.matches(&new).count(), 3);
            assert_eq!(s.matches(&since).count(), 1);
            for want in [
                kicker.as_str(),
                ">fault tests of saved files<",
                ">no power-loss test among them<",
                &format!(">memory samples of {} operations<", f.operations),
                ">one machine, process-wide, no time<",
                ">steps of xtask witness, any host<",
                ">one block, no host or user names<",
                ">--version and --license<",
                GLANCE_HEADLINE[0],
                GLANCE_HEADLINE[1],
            ] {
                assert!(s.contains(want), "{want}");
            }
            // The facts of 0.7 are stated under the speed cards, not here.
            for gone in ["Cargo.lock", "Unicode", "SHA-256"] {
                assert!(!s.contains(gone), "{gone}");
            }
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—→×✓".contains(ch)));
        }
        let alt = glance_alt().unwrap();
        for want in [
            format!(
                "{} fault tests of saved files, no power-loss test",
                f.faults
            ),
            format!(
                "{} memory samples of {} operations, one machine, process-wide, no time",
                f.samples, f.operations
            ),
            format!("{} steps of xtask witness", f.steps),
            format!("{} tool programs", f.tools.len()),
            format!("New in {}:", f.release),
            format!("Since {}:", f.since),
        ] {
            assert!(alt.contains(&want), "{want}");
        }
        assert!(alt.chars().count() <= 300 && !alt.contains('"'), "{alt}");
    }
    #[test]
    fn glance_card_draws_other_facts() {
        let f = glance_other_facts();
        let s = glance_draw(false, &f).unwrap();
        assert_eq!(glance_rolled(&s), ["3", "4", "2", "3"]);
        assert!(s.contains("0.8 AND 0.9 AT A GLANCE") && s.contains(">new in 0.9<"));
        assert!(s.contains(">memory samples of 2 operations<"));
        assert!(glance_text(&f).contains("3 tool programs"));
        assert!(glance_text(&f).contains("2 steps of xtask witness"));
        // Read from texts of their own, the facts follow those texts.
        let g = GlanceFacts::from_texts(&glance_texts()).unwrap();
        assert_eq!(
            (
                g.release.as_str(),
                g.since.as_str(),
                (g.faults, g.operations, g.processes, g.samples, g.steps),
                g.tools.len()
            ),
            ("0.9", "0.8", (3, 2, 2, 4, 2), 3)
        );
        assert_eq!(
            glance_draw(true, &g).unwrap(),
            glance_draw(true, &f).unwrap()
        );
    }
    #[test]
    fn glance_card_refuses_empty_or_disagreeing_files() {
        fn refused(t: GlanceTexts) -> bool {
            GlanceFacts::from_texts(&t).is_err()
        }
        let base = glance_texts();
        assert!(!refused(glance_texts()));
        // A release that is not vX.Y.Z.
        for release in ["0.9.2", "v0.9", "v1.0.0"] {
            assert!(refused(GlanceTexts { release, ..base }), "{release}");
        }
        // Notes that are not those of the release, or do not state a number the
        // card says is new in it: a version bump cannot relabel later work.
        for (from, to) in [
            ("# GEL RAM v0.9.2 ", "# GEL RAM v0.9.1 "),
            ("runs the two short", "runs the three short"),
            ("3 tests in two", "4 tests in two"),
            ("in two new files", "in three new files"),
            ("two processes of each", "three processes of each"),
            ("of two\n  operations", "of three\n  operations"),
            ("records=2 samples=4", "records=2 samples=5"),
        ] {
            let notes = base.notes.replace(from, to);
            assert_ne!(notes, base.notes, "{from}");
            assert!(
                refused(GlanceTexts {
                    notes: &notes,
                    ..base
                }),
                "{to}"
            );
        }
        // The README names no earlier version, or the same one, states another
        // number, or does not link the memory record the card counts.
        for (from, to) in [
            ("## New in 0.9, and in 0.8", "## New in 0.9"),
            ("## New in 0.9, and in 0.8", "## New in 0.9, and in 0.9"),
            ("the three tool programs", "the two tool programs"),
            ("runs the two short", "runs the six short"),
            ("3 tests in two", "20 tests in two"),
            ("two processes of each", "nine processes of each"),
            ("operations, 4 samples", "operations, 72 samples"),
            ("2026-11-08/README.md", "2026-11-09/README.md"),
        ] {
            let readme = base.readme.replace(from, to);
            assert_ne!(readme, base.readme, "{from}");
            assert!(
                refused(GlanceTexts {
                    readme: &readme,
                    ..base
                }),
                "{to}"
            );
        }
        // The earlier version's section is missing or does not name the
        // programs; the sentence only in another version's section does not count.
        for (from, to) in [
            ("## v0.8.0", "## v0.8.1"),
            ("for\n`--version`, and", "for\n`--help`, and"),
        ] {
            let roadmap = base.roadmap.replace(from, to);
            assert_ne!(roadmap, base.roadmap, "{from}");
            assert!(
                refused(GlanceTexts {
                    roadmap: &roadmap,
                    ..base
                }),
                "{to}"
            );
        }
        let moved = base
            .roadmap
            .replace("| Every program prints one identity line for\n`--version`, and for `--license` the license name |", "")
            .replace("old", GLANCE_SINCE_FACT);
        assert!(refused(GlanceTexts {
            roadmap: &moved,
            ..base
        }));
        // The memory record: another format, no end line, an end line that does
        // not count its lines, an operation with another number of processes,
        // or no operation at all.
        for memory in [
            base.memory.replace("GEL_MEMORY_1", "GEL_MEMORY_2"),
            base.memory
                .replace("end records=2 samples=4 redactions=0\n", ""),
            base.memory.replace("records=2", "records=3"),
            base.memory.replace("samples=4", "samples=5"),
            base.memory.replace("sample op=b rep=2\n", ""),
            base.memory
                .replace("record op=b reps=2", "record op=b reps=3"),
            base.memory.replace("record op=b reps=2", "record op=b"),
            "GEL_MEMORY_1\nend records=0 samples=0\n".to_string(),
        ] {
            assert!(
                refused(GlanceTexts {
                    memory: &memory,
                    ..base
                }),
                "{memory}"
            );
        }
        // The fault test files: one without a test, with its only test ignored or
        // cut off from its function, one test more, or a file fewer.
        let more = format!("{}#[test]\nfn four() {{}}\n", GLANCE_FAULT_TEXTS[1]);
        for second in [
            "fn helper() {}\n",
            "#[test]\n#[ignore]\nfn child() {}\n",
            "#[test]\n// a comment\nfn two() {}\n",
            more.as_str(),
        ] {
            let faults = [GLANCE_FAULT_TEXTS[0], second];
            assert!(
                refused(GlanceTexts {
                    faults: &faults,
                    ..base
                }),
                "{second}"
            );
        }
        assert!(refused(GlanceTexts {
            faults: &GLANCE_FAULT_TEXTS[..1],
            ..base
        }));
        // The guide of the short witness: no section, another number in words,
        // no command, a repeated step, a step that is not a plain name, or a
        // row more than it says.
        let more = base.reproduce.replace(
            "| same-bits | y | `B=MATCH` |\n",
            "| same-bits | y | `B=MATCH` |\n| quad | z | `C=PASS` |\n",
        );
        for (from, to) in [
            ("## A short witness", "## A long witness"),
            ("It runs two steps", "It runs three steps"),
            ("-p xtask -- witness ", "-p xtask -- witnesses "),
            ("| same-bits |", "| literal |"),
            ("| same-bits |", "| Same bits |"),
        ] {
            let reproduce = base.reproduce.replace(from, to);
            assert_ne!(reproduce, base.reproduce, "{from}");
            assert!(
                refused(GlanceTexts {
                    reproduce: &reproduce,
                    ..base
                }),
                "{to}"
            );
        }
        assert!(refused(GlanceTexts {
            reproduce: &more,
            ..base
        }));
        // The programs: no list, an empty one, a repeated name, the runner, more
        // than words go to, or another number than the README states.
        let many = format!(
            "Every program of the workspace ({}) given",
            (0..13)
                .map(|i| format!("`p{i}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for binaries in [
            "",
            "Every program of the workspace () given",
            "Every program of the workspace (`a`, `a` and `c`) given",
            "Every program of the workspace (`a`, `xtask` and `c`) given",
            "Every program of the workspace (`a` and `b`) given",
            many.as_str(),
        ] {
            assert!(refused(GlanceTexts { binaries, ..base }), "{binaries}");
        }
        // Facts made by hand are checked before they are drawn.
        let good = glance_other_facts;
        assert!(glance_draw(false, &good()).is_ok());
        for bad in [
            GlanceFacts {
                tools: Vec::new(),
                ..good()
            },
            GlanceFacts {
                faults: 0,
                ..good()
            },
            GlanceFacts { steps: 0, ..good() },
            GlanceFacts {
                operations: 0,
                samples: 0,
                ..good()
            },
            GlanceFacts {
                samples: 5,
                ..good()
            },
            GlanceFacts {
                processes: usize::MAX,
                ..good()
            },
            GlanceFacts {
                since: "0.9".to_string(),
                ..good()
            },
            GlanceFacts {
                since: "0".to_string(),
                ..good()
            },
            GlanceFacts {
                release: "0.9.0".to_string(),
                ..good()
            },
        ] {
            assert!(glance_draw(false, &bad).is_err());
            assert!(glance_phone_draw(false, &bad).is_err());
        }
        // A number or a line too long for its tile is refused, not drawn over the edge.
        let e = glance_draw(
            false,
            &GlanceFacts {
                faults: usize::MAX,
                ..good()
            },
        )
        .unwrap_err();
        assert!(e.to_string().contains("does not fit"), "{e}");
        let many = GlanceFacts {
            operations: 10_000_000_000_000_000,
            processes: 1,
            samples: 10_000_000_000_000_000,
            ..good()
        };
        assert!(glance_draw(false, &many).is_err());
        let wide = GlanceFacts {
            since: "1234567890123456789012345678901234567890.0".to_string(),
            ..good()
        };
        assert!(glance_draw(false, &wide).is_err());
        assert!(glance_fit("fault tests of saved files", 20.0, 0.0, 494.0).is_ok());
        assert!(glance_fit(&"x".repeat(40), 20.0, 0.0, 494.0).is_err());
    }
    #[test]
    fn the_facts_of_0_7_are_read_from_their_files() {
        // The Unicode version as the name of the normalization test file the fast
        // identity check includes (an older file is kept for a comparison).
        let unicode = unicode_version().unwrap();
        let fast = fs::read_to_string("crates/gel-source/src/nfc_fast_identity.rs").unwrap();
        let named: Vec<&str> = fast
            .split("tests/data/NormalizationTest-")
            .skip(1)
            .filter_map(|s| s.split_once(".txt").map(|v| v.0))
            .collect();
        assert_eq!(named, [unicode.as_str()]);
        let readme = native_intro();
        assert!(readme.contains(&format!(
            "normalization follows Unicode {unicode}, the version"
        )));
        assert!(!readme.contains("__UNICODE__"));
        // The packages: each name line of Cargo.lock, a crate directory here.
        let lock = fs::read_to_string("Cargo.lock").unwrap();
        let names: Vec<&str> = lock
            .lines()
            .filter_map(|l| l.strip_prefix("name = \"")?.strip_suffix('"'))
            .collect();
        assert_eq!(packages().unwrap(), names.len());
        for n in &names {
            let dir = if *n == "xtask" {
                n.to_string()
            } else {
                format!("crates/{n}")
            };
            assert!(Path::new(&dir).is_dir(), "{dir}");
        }
        // The tables: no header, no constant, or two versions.
        let tables = "// Character Database 17.0.0; do not edit. Inputs (SHA-256):\npub(crate) const UNICODE_VERSION: (u8, u8, u8) = (17, 0, 0);\n";
        assert_eq!(unicode_of(tables).unwrap(), "17.0.0");
        for bad in [
            "pub(crate) const UNICODE_VERSION: (u8, u8, u8) = (17, 0, 0);\n",
            "// Character Database 17.0.0; do not edit. Inputs (SHA-256):\n",
            "// Character Database 17.0.0; do not edit. Inputs (SHA-256):\npub(crate) const UNICODE_VERSION: (u8, u8, u8) = (16, 0, 0);\n",
            "// Character Database 17.0; do not edit. Inputs (SHA-256):\npub(crate) const UNICODE_VERSION: (u8, u8, u8) = (17, 0);\n",
        ] {
            assert!(unicode_of(bad).is_err(), "{bad}");
        }
        // The roadmap section of the release the README names states each fact;
        // a fact stated only in another version's section does not count.
        let roadmap = "# Roadmap\n\n## v0.8.0\n\nnew\n\n## v0.7.0\n\nCargo.lock lists no third-party package; NFC follows Unicode 17.0.0; SHA-256 here.\n\n## v0.6.0\n\nold\n";
        assert!(unicode_release_check(roadmap, "17.0.0").is_ok());
        assert!(unicode_release_check(roadmap, "16.0.0").is_err());
        for (from, to) in [
            ("## v0.7.0", "## v0.7.1"),
            ("lists no third-party package", "lists one package"),
            ("follows Unicode 17.0.0", "follows Unicode 16.0.0"),
            ("SHA-256 here", "hashes here"),
        ] {
            let changed = roadmap.replace(from, to);
            assert!(unicode_release_check(&changed, "17.0.0").is_err(), "{to}");
        }
        let moved = roadmap
            .replace("SHA-256 here.", "")
            .replace("old", "SHA-256");
        assert!(unicode_release_check(&moved, "17.0.0").is_err());
        // Cargo.lock: a package from outside, without a plain name, not a crate
        // directory of this checkout, or no package at all.
        let has = |dir: &str| dir == "crates/a" || dir == "xtask";
        let lock = "version = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"xtask\"\nversion = \"0.1.0\"\n";
        assert_eq!(lock_packages(lock, &has).unwrap(), 2);
        for bad in [
            "[[package]]\nname = \"a\"\nsource = \"registry+https://example.invalid\"\n",
            "[[package]]\nversion = \"0.1.0\"\n",
            "[[package]]\nname = \"../a\"\n",
            "[[package]]\nname = \"b\"\n",
            "version = 4\n",
        ] {
            assert!(lock_packages(bad, &has).is_err(), "{bad}");
        }
        assert!(lock_packages(lock, &|_: &str| false).is_err());
    }
    #[test]
    fn glance_card_moves_and_can_stand_still() {
        let (light, dark) = (glance_card(false).unwrap(), glance_card(true).unwrap());
        assert_ne!(light, dark);
        for s in [&light, &dark] {
            assert_eq!(status_motion(s), (Vec::new(), Vec::new()));
            assert_eq!(s.matches(r#"<g class="gt""#).count(), 4);
            assert_eq!(s.matches("clip-path=\"url(#gtile").count(), 4);
            assert!(s.contains("@keyframes ro60s"));
        }
        // The checker catches an animated class left out, and a base rule that hides.
        let broken = light.replace("{.gt,.gp,", "{.gp,");
        assert_eq!(status_motion(&broken).0, ["gt"]);
        let hiding = light.replace(".kick{font-size:16px}", ".kick{opacity:0}");
        assert_eq!(status_motion(&hiding).1, ["kick"]);
    }
    // ===== TEST glance END =====

    // ===== TEST pledges BEGIN =====
    /// A hand-made table in the shape of docs/COMMITMENTS.md, with its count columns swapped.
    const PLEDGES_MUTANT: &str = "# x\n\n## Published commitments\n\n| Date (UTC) | Label | Files | Directories | SHA-256 of the manifest |\n|---|---|---|---|---|\n| 2027-01-01 | Alpha `set` & more | 1 | 12,345 | `0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef` |\n| 2027-01-02 | Beta: kept in the table | 7 | 1 | `fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210` |\n| 2027-01-11 | Gamma | 0 | 3 | `aaaabbbbccccddddeeeeffff00001111222233334444555566667777888899990` |\n\nAfter the table.\n";
    fn pledges_mutant() -> String {
        // The third hash above has 65 characters; the valid table drops the last.
        PLEDGES_MUTANT.replace("888899990`", "88889999`")
    }
    /// The rows of the repository's table read a second way: the lines of its
    /// section that hold a backticked 64-hex hash, as (date, hash).
    fn pledges_rows_by_hand() -> Vec<(String, String)> {
        let text = fs::read_to_string(PLEDGES_FILE).unwrap();
        let section = text.split("## Published commitments").nth(1).unwrap();
        let section = section.split("\n## ").next().unwrap();
        section
            .lines()
            .filter_map(|line| {
                let hash = line
                    .split('`')
                    .find(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))?;
                let date = line.trim_start_matches('|').trim().get(..10)?;
                Some((date.to_string(), hash.to_string()))
            })
            .collect()
    }
    /// The layout keeps every line inside its box by the cautious width, keeps
    /// chips and same-row date labels apart, and stays inside the card.
    fn pledges_fits(p: &Pledges, l: &PledgesLayout) {
        let n = p.rows.len();
        let full = (1200.0 - 2.0 * PLEDGES_SIDE) * PLEDGES_SLACK;
        let title = format!(
            "{} · {} → {}",
            pledges_many(n),
            p.rows[0].date,
            p.rows[n - 1].date
        );
        assert!(pledges_width(&title, PLEDGES_TITLE) <= full, "{title}");
        for line in PLEDGES_FOOT {
            assert!(pledges_width(line, PLEDGES_SMALL) <= full, "{line}");
        }
        assert!(l.foot + 24.0 + 12.0 <= f64::from(l.height));
        // The top label row stays clear of the title.
        let rows = l.dots.iter().map(|d| d.label_row).max().unwrap();
        assert!(l.ay - 22.0 - 20.0 * rows as f64 - PLEDGES_SMALL > 56.0 + 6.0);
        let half = |d: &PledgesDot| pledges_width(&d.tag, PLEDGES_SMALL) / 2.0;
        for (i, d) in l.dots.iter().enumerate() {
            assert!(d.x - 13.0 >= PLEDGES_SIDE && d.x + 13.0 <= 1200.0 - PLEDGES_SIDE);
            assert!(
                d.label_x - half(d) + 1e-9 >= PLEDGES_SIDE
                    && d.label_x + half(d) <= 1200.0 - PLEDGES_SIDE + 1e-9,
                "{}",
                d.tag
            );
            // Dots do not overlap, and labels in one label row do not touch.
            if let Some(e) = l.dots.get(i + 1) {
                assert!(e.x - d.x >= 2.0 * 13.0 + 2.0, "{} {}", d.tag, e.tag);
            }
            for e in &l.dots[i + 1..] {
                if e.label_row == d.label_row {
                    assert!(
                        e.label_x - half(e) >= d.label_x + half(d) + 10.0 - 1e-9,
                        "{} {}",
                        d.tag,
                        e.tag
                    );
                }
            }
        }
        assert_eq!(l.dots.iter().map(|d| d.count).sum::<usize>(), n);
        let mut drawn: Vec<usize> = l.chips.iter().map(|c| c.row).collect();
        drawn.sort_unstable();
        assert_eq!(drawn, (0..n).collect::<Vec<_>>());
        for (i, c) in l.chips.iter().enumerate() {
            assert!(c.x >= PLEDGES_SIDE && c.x + c.w <= 1200.0 - PLEDGES_SIDE + 1e-9);
            assert!(c.y > l.ay + 13.0 && c.y + c.h + 16.0 < l.foot);
            assert!(!c.name.is_empty());
            for line in &c.name {
                assert!(
                    pledges_width(line, PLEDGES_NAME) <= pledges_room(c.w, true),
                    "{line}"
                );
            }
            for line in &c.counts {
                assert!(
                    pledges_width(line, PLEDGES_SMALL) <= pledges_room(c.w, false),
                    "{line}"
                );
            }
            let tag = pledges_width(&c.tag, PLEDGES_SMALL)
                + 12.0
                + pledges_width("01234567", PLEDGES_SMALL);
            assert!(tag <= pledges_room(c.w, true));
            // The text of a chip ends above its bottom edge.
            let text = 28.0 + 24.0 * (c.name.len() - 1) as f64 + 22.0 * (c.counts.len() + 1) as f64;
            assert!(text + 8.0 <= c.h);
            // No two chips overlap: side by side in a row, one row below the other.
            for e in &l.chips[i + 1..] {
                if e.y == c.y {
                    assert!(c.x + c.w + PLEDGES_GAP <= e.x + 1e-9);
                    assert_eq!(c.h, e.h);
                } else {
                    assert!(c.y + c.h < e.y);
                }
            }
        }
    }
    #[test]
    fn pledges_card_shows_every_published_commitment() {
        let p = Pledges::read().unwrap();
        let by_hand = pledges_rows_by_hand();
        assert!(!by_hand.is_empty());
        assert_eq!(p.rows.len(), by_hand.len());
        assert_eq!(commitments().unwrap(), by_hand.len());
        for (r, (date, hash)) in p.rows.iter().zip(&by_hand) {
            assert_eq!((&r.date, &r.hash), (date, hash));
        }
        assert_eq!(p.counted, [("directory", "directories"), ("file", "files")]);
        let mut dates: Vec<&str> = by_hand.iter().map(|r| r.0.as_str()).collect();
        dates.dedup();
        let (first, last) = (&by_hand[0].0, &by_hand[by_hand.len() - 1].0);
        let l = pledges_layout(&p);
        pledges_fits(&p, &l);
        for dark in [false, true] {
            let s = pledges_card(dark).unwrap();
            assert!(s.contains(&format!(r#"height="{}""#, l.height)));
            assert_eq!(s.matches(r#"<g class="cd""#).count(), by_hand.len());
            assert_eq!(s.matches(r#"<path class="sh""#).count(), by_hand.len());
            assert_eq!(s.matches(r#"<g class="dt""#).count(), dates.len());
            assert!(s.contains(&format!(
                ">{} commitments · {first} → {last}<",
                by_hand.len()
            )));
            for (_, hash) in &by_hand {
                // Each hash is drawn by its first 8 characters only.
                assert!(s.contains(&format!(">{}<", &hash[..8])), "{hash}");
                assert!(!s.contains(&hash[..9]), "{hash}");
            }
            for want in [
                ">69c10bd5<",
                ">10-20<",
                ">Private source tree 1<",
                ">1,436 directories · 4,054 files<",
                ">0 directories · 8 files<",
                ">Sealed sample 1<",
                ">Hashed now · checkable with xtask commitment-check once the files are handed over<",
                "not that they are correct",
                "first 8 hex of its SHA-256",
            ] {
                assert!(s.contains(want), "{want}");
            }
            // The words after a label's colon stay in the table; no per-chip prefix.
            assert!(!s.contains("Rust sources only") && !s.contains("SHA-256 <tspan"));
        }
        let alt = pledges_alt().unwrap();
        let chars = alt.chars().count();
        assert!(!alt.contains('"') && chars <= 300, "{chars}");
        assert!(alt.starts_with(&format!(
            "{} commitments, {first} to {last}: each the SHA-256",
            by_hand.len()
        )));
        for want in [
            "directory and file counts",
            "once the files are handed over",
        ] {
            assert!(alt.contains(want), "{want}");
        }
    }
    #[test]
    fn pledges_card_draws_the_table_it_is_given() {
        let p = pledges_parse(&pledges_mutant()).unwrap();
        assert_eq!(p.counted, [("file", "files"), ("directory", "directories")]);
        assert_eq!(p.rows[0].label, "Alpha set & more");
        pledges_fits(&p, &pledges_layout(&p));
        let real = Pledges::read().unwrap();
        for dark in [false, true] {
            let s = pledges_draw(dark, &p).unwrap();
            assert_eq!(s.matches(r#"<g class="cd""#).count(), 3);
            for want in [
                ">01234567<",
                ">fedcba98<",
                ">aaaabbbb<",
                ">01-01<",
                ">01-11<",
                ">Alpha set &amp; more<",
                ">Beta<",
                ">1 file · 12,345 directories<",
                ">7 files · 1 directory<",
                ">0 files · 3 directories<",
                ">3 commitments · 2027-01-01 → 2027-01-11<",
                // Spaced by real date: 10 days over 1070.4 px, one day is 107.04 px.
                r#"cx="64.8""#,
                r#"cx="171.8""#,
                r#"cx="1135.2""#,
            ] {
                assert!(s.contains(want), "{want}");
            }
            for r in &real.rows {
                assert!(!s.contains(&r.hash[..8]), "{}", r.date);
                assert!(!s.contains(&r.date[5..]), "{}", r.date);
            }
            assert!(!s.contains("kept in the table") && !s.contains("Private"));
        }
        assert!(pledges_text(&p).contains("its file and directory counts"));
        // Dates of two years are drawn in full.
        let years = pledges_parse(&pledges_mutant().replace("2027-01-01", "2026-12-31")).unwrap();
        let s = pledges_draw(false, &years).unwrap();
        for want in [">2026-12-31<", ">2027-01-02<", ">2027-01-11<"] {
            assert!(s.contains(want), "{want}");
        }
        pledges_fits(&years, &pledges_layout(&years));
        // One date only: one dot in the middle of the line, holding all three.
        let one = pledges_mutant()
            .replace("2027-01-02", "2027-01-01")
            .replace("2027-01-11", "2027-01-01");
        let one = pledges_parse(&one).unwrap();
        let s = pledges_draw(false, &one).unwrap();
        assert_eq!(s.matches(r#"<g class="dt""#).count(), 1);
        assert!(s.contains(r#"cx="600.0""#) && s.contains(">3</text>"));
        pledges_fits(&one, &pledges_layout(&one));
    }
    #[test]
    fn pledges_card_refuses_a_malformed_table() {
        let good = pledges_mutant();
        pledges_parse(&good).unwrap();
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let rows_start = good.find("| 2027-01-01").unwrap();
        let rows_end = good.find("\nAfter").unwrap();
        let header = &good[good.find("| Date").unwrap()..good.find("|---").unwrap()];
        for (why, bad) in [
            ("65 hex characters", PLEDGES_MUTANT.to_string()),
            ("63 hex characters", good.replace(hash, &hash[1..])),
            ("not hex", good.replace(hash, &hash.replace('a', "g"))),
            ("no backticks", good.replace(&format!("`{hash}`"), hash)),
            ("short date", good.replace("2027-01-01", "2027-1-01")),
            ("slashes", good.replace("2027-01-01", "2027/01/01")),
            ("month 13", good.replace("2027-01-01", "2027-13-01")),
            ("month 0", good.replace("2027-01-01", "2027-00-01")),
            ("day 0", good.replace("2027-01-01", "2027-01-00")),
            ("no 30 February", good.replace("2027-01-01", "2027-02-30")),
            (
                "2100 is no leap year",
                good.replace("2027-01-11", "2100-02-29"),
            ),
            ("out of order", good.replace("2027-01-02", "2026-12-31")),
            ("count", good.replace("| 12,345 |", "| 12345 |")),
            ("count text", good.replace("| 7 |", "| seven |")),
            ("empty label", good.replace("| Gamma |", "| |")),
            ("cell missing", good.replace("| 7 | 1 |", "| 7 |")),
            ("header cells", good.replace("| Label | ", "| ")),
            ("date column", good.replace("| Date (UTC) |", "| When |")),
            ("label column", good.replace("| Label |", "| Name |")),
            (
                "hash column",
                good.replace("| SHA-256 of the manifest |", "| Hash |"),
            ),
            (
                "unknown count column",
                good.replace("| Files |", "| Items |"),
            ),
            (
                "one column twice",
                good.replace("| Directories |", "| Files |"),
            ),
            (
                "rule shape",
                good.replace("|---|---|---|---|---|", "|---|---|---|---|"),
            ),
            ("rule text", good.replace("|---|---|---|", "|---|-x-|---|")),
            ("no rule", format!("## Published commitments\n\n{header}")),
            (
                "no header",
                "## Published commitments\n\nNone yet.\n".to_string(),
            ),
            ("no section", good.replace("## Published", "## Withdrawn")),
            (
                "empty table",
                format!("{}{}", &good[..rows_start], &good[rows_end..]),
            ),
        ] {
            assert!(pledges_parse(&bad).is_err(), "{why}");
        }
        // Calendar days: 2000 and 2028 have a 29 February, 2027 and 2100 do not.
        for leap in ["2000-02-29", "2028-02-29"] {
            pledges_day(leap).unwrap();
        }
        for no in ["2027-02-29", "2100-02-29"] {
            assert!(pledges_day(no).is_err(), "{no}");
        }
        let day = |d: &str| pledges_day(d).unwrap();
        assert_eq!(day("1970-01-01"), 0);
        assert_eq!(day("2000-03-01") - day("2000-02-28"), 2);
        assert_eq!(day("2100-03-01") - day("2100-02-28"), 1);
        assert_eq!(day("2026-11-01") - day("2026-10-20"), 12);
        // The table must hold the rows the README counts.
        let text = fs::read_to_string(PLEDGES_FILE).unwrap();
        let n = commitments().unwrap();
        pledges_checked(&text, n).unwrap();
        for wrong in [n - 1, n + 1] {
            let e = pledges_checked(&text, wrong).unwrap_err().to_string();
            assert!(e.starts_with(PLEDGES_FILE), "{e}");
        }
        // A table without rows is not drawn either.
        let empty = Pledges {
            counted: [("directory", "directories"), ("file", "files")],
            rows: Vec::new(),
        };
        assert!(pledges_draw(false, &empty).is_err());
    }
    #[test]
    fn pledges_card_scales_to_many_dates() {
        // 9 dates, some one day apart, 14 rows.
        let days = [
            ("2027-01-01", 2),
            ("2027-01-02", 1),
            ("2027-01-03", 3),
            ("2027-01-20", 1),
            ("2027-02-01", 1),
            ("2027-02-02", 2),
            ("2027-03-15", 1),
            ("2027-03-16", 1),
            ("2027-06-30", 2),
        ];
        let mut text = String::from("## Published commitments\n\n| Date (UTC) | Label | Directories | Files | SHA-256 of the manifest |\n|---|---|---|---|---|\n");
        let mut hashes = Vec::new();
        for (date, count) in days {
            for _ in 0..count {
                let i = hashes.len();
                let hash = format!("{:08x}", 0xa1b2_c300 + i).repeat(8);
                let label = match i {
                    3 => "Private source tree 9: Rust sources only (`*.rs`, `Cargo.toml`)"
                        .to_string(),
                    5 => "A commitment whose name is far too long for one line".to_string(),
                    _ => format!("Set {i}"),
                };
                text.push_str(&format!(
                    "| {date} | {label} | {} | {} | `{hash}` |\n",
                    grouped(if i == 7 { 1 } else { i * 1000 }),
                    grouped(i * 37 + 1)
                ));
                hashes.push(hash);
            }
        }
        let p = pledges_parse(&text).unwrap();
        assert_eq!(p.rows.len(), 14);
        let l = pledges_layout(&p);
        pledges_fits(&p, &l);
        // Over 180 days one day is 6 px: dates one day apart share a dot.
        let dots: Vec<(&str, usize)> = l.dots.iter().map(|d| (d.tag.as_str(), d.count)).collect();
        assert_eq!(
            dots,
            [
                ("01-01 → 01-03", 6),
                ("01-20", 1),
                ("02-01 → 02-02", 3),
                ("03-15 → 03-16", 2),
                ("06-30", 2)
            ]
        );
        // The long name wraps, and its row of chips grows with it.
        assert!(l.chips[5].name.len() >= 2);
        assert!(l.chips[3].h > l.chips[0].h);
        assert!(l.chips[7].counts.concat().contains("1 directory"));
        for dark in [false, true] {
            let s = pledges_draw(dark, &p).unwrap();
            assert!(s.contains(&format!(r#"height="{}""#, l.height)));
            assert_eq!(s.matches(r#"<g class="cd""#).count(), 14);
            assert_eq!(s.matches(r#"<g class="dt""#).count(), 5);
            assert!(s.contains(">01-01 → 01-03<") && s.contains(">6</text>"));
            for hash in &hashes {
                assert!(s.contains(&format!(">{}<", &hash[..8])), "{hash}");
            }
            for (date, _) in days {
                assert!(s.contains(&format!(">{}<", &date[5..])), "{date}");
            }
            assert!(s.contains(">14 commitments · 2027-01-01 → 2027-06-30<"));
            assert!(s.contains(">Private source tree 9<") && s.contains(">1 directory"));
        }
    }
    #[test]
    fn pledges_card_moves_and_can_stand_still() {
        let (light, dark) = (pledges_card(false).unwrap(), pledges_card(true).unwrap());
        assert_ne!(light, dark);
        assert!(light.contains(LIGHT.accent) && dark.contains(DARK.accent));
        for s in [&light, &dark] {
            let style = s
                .split("<style>")
                .nth(1)
                .unwrap()
                .split("</style>")
                .next()
                .unwrap();
            let (moving, still) = style
                .split_once("@media (prefers-reduced-motion:reduce){")
                .unwrap();
            assert!(moving.contains("@keyframes"));
            // Every class with an animation stops under reduced motion, where its
            // base style is the final state: nothing starts hidden outside a keyframe.
            let mut animated = 0;
            for rule in moving.split('}') {
                if let Some((sel, body)) = rule.split_once('{') {
                    if body.contains("animation:") {
                        let class = sel.trim();
                        assert!(class.starts_with('.'), "{class}");
                        assert!(still.contains(class), "{class}");
                        assert!(!body.contains("opacity"), "{class}");
                        animated += 1;
                    }
                }
            }
            assert_eq!(animated, 5);
            assert!(still.contains("animation:none"));
            for word in ["<script", "href", "url(http", "<image", "STUB"] {
                assert!(!s.contains(word), "{word}");
            }
            // Readable on a phone: no drawn text under 16 px.
            let sizes: Vec<f64> = s
                .split(r#"font-size=""#)
                .skip(1)
                .map(|rest| rest.split('"').next().unwrap().parse().unwrap())
                .collect();
            assert!(
                !sizes.is_empty() && sizes.iter().all(|&px| px >= 16.0),
                "{sizes:?}"
            );
            assert!(!s.contains("class=\"kick\""));
        }
    }
    // ===== TEST pledges END =====

    // ===== TEST core BEGIN =====
    fn core_cargo(k: &str) -> Result<String> {
        let p = format!("crates/{k}/Cargo.toml");
        Ok(fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"))?)
    }
    fn core_of(template: &str) -> Result<CoreMap> {
        core_map(&core_rows(template)?, &CORE_TAGS, &core_cargo)
    }
    /// The arrows of a map as (crate built on, crate that builds on it), sorted.
    fn core_links(m: &CoreMap) -> Vec<(String, String)> {
        let mut l: Vec<(String, String)> = m
            .links
            .iter()
            .map(|&(a, b)| (m.parts[a].krate.clone(), m.parts[b].krate.clone()))
            .collect();
        l.sort();
        l
    }
    fn core_refused(m: Result<CoreMap>, why: &str) {
        let e = m.err().expect(why).to_string();
        assert!(e.contains(why), "{e}");
    }
    #[test]
    fn core_card_draws_the_parts_and_the_dependencies_of_the_crates() {
        // A second reading of the table: the crate between "(`" and "`)" of
        // every table row of the section.
        let section = NATIVE.split("## The memory core").nth(1).unwrap();
        let section = section.split("\n## ").next().unwrap();
        let crates: Vec<&str> = section
            .lines()
            .filter(|l| l.starts_with("| ") && l.contains(" (`"))
            .map(|l| l.split("(`").nth(1).unwrap().split('`').next().unwrap())
            .collect();
        let m = CoreMap::read().unwrap();
        let drawn: Vec<&str> = m.parts.iter().map(|p| p.krate.as_str()).collect();
        assert_eq!(drawn, crates);
        // A second reading of the dependencies: b builds on a when a line of
        // b's Cargo.toml begins with the name of a.
        let mut expected = Vec::new();
        for b in &crates {
            let toml = fs::read_to_string(format!("crates/{b}/Cargo.toml")).unwrap();
            for a in &crates {
                let named = |l: &str| {
                    l.strip_prefix(a)
                        .is_some_and(|r| r.trim_start().starts_with('=') || r.starts_with('.'))
                };
                if toml.lines().any(named) {
                    expected.push((a.to_string(), b.to_string()));
                }
            }
        }
        expected.sort();
        assert!(!expected.is_empty());
        assert_eq!(core_links(&m), expected);
        let alone: Vec<&CorePart> = m
            .parts
            .iter()
            .filter(|p| !expected.iter().any(|(a, b)| *a == p.krate || *b == p.krate))
            .collect();
        let (n, e) = (crates.len(), expected.len());
        let alt = core_alt().unwrap();
        for dark in [false, true] {
            let s = core_card(dark).unwrap();
            assert_eq!(s.matches("<g class=\"cb\"").count(), n);
            for class in [
                "<path class=\"cf\"",
                "<polygon class=\"ch\"",
                "<path class=\"cp\"",
            ] {
                assert_eq!(s.matches(class).count(), e, "{class}");
            }
            assert_eq!(s.matches("<rect class=\"cg\"").count(), e);
            for p in &m.parts {
                for line in [&p.name, &p.krate, &p.tag[0], &p.tag[1]] {
                    assert!(s.contains(&format!(">{}<", xml(line))), "{line}");
                }
            }
            let title = format!("{n} parts, {e} dependencies between them");
            assert!(s.contains(&format!(">{title}<")));
            assert!(s.contains(">Arrow A → B: B builds on A, as its Cargo.toml says<"));
            assert_eq!(s.contains(">stand alone<"), !alone.is_empty());
            assert!(s.contains(&format!("<desc id=\"d\">{}</desc>", xml(&alt))));
            assert!(!s.contains("__") && !s.contains("<script") && !s.contains("href"));
            assert!(s.chars().all(|ch| ch.is_ascii() || "·—→".contains(ch)));
            // The heading lines end left of the frame of the parts that stand alone.
            let right = m.frame().unwrap().map_or(1160.0, |f| f.0);
            assert!(40.0 + title.chars().count() as f64 * 0.66 * 30.0 <= right * 0.98);
            assert!(40.0 + 51.0 * 0.62 * 20.0 <= right * 0.98);
        }
        assert_eq!(alt, core_text(&m));
        assert!(alt.len() <= 300 && !alt.contains('"'), "{alt}");
        assert!(alt.starts_with(&format!(
            "The {n} parts of the memory core and the {e} dependencies"
        )));
        for p in &alone {
            assert!(alt.contains(&p.name), "{}", p.name);
        }
        assert!(native_intro().contains(&visual_picture("core")));
        assert!(visual_picture("core").contains(&format!("alt=\"{alt}\"")));
    }
    #[test]
    fn core_card_follows_changed_dependencies_and_a_changed_table() {
        let real = CoreMap::read().unwrap();
        // History no longer builds on the bit record: no arrow runs below.
        let fewer = |k: &str| -> Result<String> {
            let s = core_cargo(k)?;
            Ok(if k == "gel-history" {
                s.lines()
                    .filter(|l| !l.starts_with("gel-orb"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                s
            })
        };
        let m = core_map(&core_rows(NATIVE).unwrap(), &CORE_TAGS, &fewer).unwrap();
        let mut want = core_links(&real);
        want.retain(|l| *l != ("gel-orb".to_string(), "gel-history".to_string()));
        assert_eq!(core_links(&m), want);
        assert_eq!(m.lanes(), 0);
        let s = core_draw(false, &m).unwrap();
        assert_eq!(s.matches("<path class=\"cf\"").count(), want.len());
        assert!(core_text(&m).contains("; History builds on Rebuild."));
        // Memory cost builds on Reader: one more arrow, and only Q8 record stands alone.
        let more = |k: &str| -> Result<String> {
            let s = core_cargo(k)?;
            Ok(if k == "gel-physics" {
                s.replace(
                    "[dependencies]",
                    "[dependencies]\ngel-reader = { path = \"../gel-reader\" }",
                )
            } else {
                s
            })
        };
        let m = core_map(&core_rows(NATIVE).unwrap(), &CORE_TAGS, &more).unwrap();
        assert_eq!(m.links.len(), real.links.len() + 1);
        let s = core_draw(true, &m).unwrap();
        assert_eq!(
            s.matches("<path class=\"cf\"").count(),
            real.links.len() + 1
        );
        let text = core_text(&m);
        assert!(
            text.contains("Memory cost builds on Reader")
                && text.ends_with(" Q8 record stands alone."),
            "{text}"
        );
        let (x, y) = core_at((2.0, 0.0));
        let frame = format!("<rect x=\"{:.1}\" y=\"{:.1}\"", x - 14.0, y - CORE_D - 40.0);
        assert!(s.contains(&frame), "{frame}");
        // Renamed parts are drawn as the table names them.
        let renamed = NATIVE
            .replace("| Bit record (`gel-orb`)", "| One record (`gel-orb`)")
            .replace("| Reader (`gel-reader`)", "| Search (`gel-reader`)");
        let m = core_of(&renamed).unwrap();
        let s = core_draw(false, &m).unwrap();
        assert!(s.contains(">One record<") && s.contains(">Search<"));
        assert!(!s.contains(">Bit record<") && !s.contains(">Reader<"));
        assert!(core_text(&m).contains("on One record"));
        // Reordered rows: the block drawn first is the first row of the table.
        let row = |k: &str| {
            NATIVE
                .lines()
                .find(|l| l.contains(&format!("(`{k}`)")))
                .unwrap()
                .to_string()
        };
        let swapped = NATIVE
            .replace(&row("gel-orb"), "@@ORB@@")
            .replace(&row("gel-history"), &row("gel-orb"))
            .replace("@@ORB@@", &row("gel-history"));
        let m = core_of(&swapped).unwrap();
        assert_eq!(m.parts[0].krate, "gel-history");
        assert_eq!(core_links(&m), core_links(&real));
        let s = core_draw(true, &m).unwrap();
        let first = s
            .find("<g class=\"cb\" style=\"animation-delay:0.20s\"")
            .unwrap();
        assert!(s[first..].find(">History<").unwrap() < s[first..].find(">Bit record<").unwrap());
    }
    #[test]
    fn core_card_refuses_what_it_cannot_draw_faithfully() {
        let rows = core_rows(NATIVE).unwrap();
        let with = |krate: &'static str, from: &'static str, to: &'static str| {
            move |k: &str| -> Result<String> {
                let s = core_cargo(k)?;
                Ok(if k == krate { s.replace(from, to) } else { s })
            }
        };
        // The table: none, empty, a row without a crate, a row of two cells, a
        // row without its closing bar, an odd part name, a crate named twice.
        core_refused(core_of("# GEL RAM\n"), "no table of the memory core");
        let head = "## The memory core\n\n| Part | What it does | Check it |\n|:---|:---|:---|\n";
        core_refused(core_of(head), "the core table has no rows");
        for (row, why) in [
            ("| Bit record | x | y |", "no crate in the core row"),
            ("| Bit record (`gel-orb`) | x |", "unexpected core row"),
            ("| Bit record (`gel-orb`) | x | y", "unexpected core row"),
            (
                "| Bit-record (`gel-orb`) | x | y |",
                "unexpected part or crate name",
            ),
        ] {
            core_refused(core_of(&format!("{head}{row}\n")), why);
        }
        let orb = NATIVE.lines().find(|l| l.contains("(`gel-orb`)")).unwrap();
        core_refused(
            core_of(&format!("{head}{orb}\n{orb}\n")),
            "gel-orb is named twice",
        );
        // A crate that does not exist, even with a tag of its own.
        let missing = NATIVE.replace("(`gel-orb`)", "(`gel-imaginary`)");
        core_refused(core_of(&missing), "crates/gel-imaginary/Cargo.toml: ");
        let mut tags = CORE_TAGS;
        tags[0].krate = "gel-imaginary";
        core_refused(
            core_map(&core_rows(&missing).unwrap(), &tags, &core_cargo),
            "crates/gel-imaginary/Cargo.toml: ",
        );
        // Dependency tables this reader does not understand.
        let target = with(
            "gel-orb",
            "[lib]",
            "[target.'cfg(unix)'.dependencies]\nlibc = \"1\"\n[lib]",
        );
        core_refused(
            core_map(&rows, &CORE_TAGS, &target),
            "crates/gel-orb/Cargo.toml: unsupported dependency table",
        );
        let split = with(
            "gel-store",
            "[dependencies]",
            "[dependencies]\nx = { version = \"1\", features = [\n\"y\",\n] }",
        );
        core_refused(
            core_map(&rows, &CORE_TAGS, &split),
            "crates/gel-store/Cargo.toml: unexpected dependency line",
        );
        // A part with no tag, a tag with no part.
        core_refused(
            core_of(&NATIVE.replace(
                "| Store (`gel-store`)",
                "| Kernel (`gel-kernel`) | a part | `t` |\n| Store (`gel-store`)",
            )),
            "no tag for gel-kernel",
        );
        let without = NATIVE
            .lines()
            .filter(|l| !l.contains("(`gel-physics`)"))
            .collect::<Vec<_>>()
            .join("\n");
        core_refused(core_of(&without), "the tag of gel-physics has no row");
        // A tag saying what its row no longer says, or a limit the row dropped.
        core_refused(
            core_of(&NATIVE.replace("One 1,152-byte record", "One 1,280-byte record")),
            "the tag of gel-phase-quad says 1,152",
        );
        core_refused(
            core_of(&NATIVE.replace(", a wrong parent is not detected", "")),
            "the tag of gel-structural says wrong",
        );
        // A name too long for its block.
        core_refused(
            core_of(&NATIVE.replace(
                "| Store (`gel-store`)",
                "| Store of every record (`gel-store`)",
            )),
            "Store of every record does not fit its block",
        );
        // Slots: off the grid, between rows, shared.
        for (i, slot, why) in [
            (5, (3.0, 0.0), "the slot of gel-phase-quad is off the grid"),
            (0, (0.0, 0.5), "the slot of gel-orb is off the grid"),
            (6, (2.0, 0.0), "gel-physics and gel-phase-quad share a slot"),
        ] {
            let mut tags = CORE_TAGS;
            tags[i].slot = slot;
            core_refused(core_map(&rows, &tags, &core_cargo), why);
        }
        // Arrows that would cross a block: within one column, and below a
        // column whose end block has another under it.
        let same = with(
            "gel-reader",
            "[dependencies]",
            "[dependencies]\ngel-store = { path = \"../gel-store\" }",
        );
        core_refused(
            core_map(&rows, &CORE_TAGS, &same),
            "gel-reader builds on gel-store, but an arrow",
        );
        let blocked = with(
            "gel-phase-quad",
            "[dependencies]",
            "[dependencies]\ngel-orb = { path = \"../gel-orb\" }",
        );
        core_refused(
            core_map(&rows, &CORE_TAGS, &blocked),
            "gel-phase-quad builds on gel-orb, but an arrow",
        );
        // Rebuild and History without dependencies would stand alone among Store and Reader.
        let loose = |k: &str| -> Result<String> {
            let s = core_cargo(k)?;
            Ok(if k == "gel-structural" || k == "gel-history" {
                s.lines()
                    .filter(|l| !l.starts_with("gel-"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                s
            })
        };
        core_refused(
            core_map(&rows, &CORE_TAGS, &loose),
            "cannot be framed apart: gel-store stands among them",
        );
        assert!(core_of(NATIVE).is_ok());
    }
    #[test]
    fn core_card_moves_and_can_stand_still() {
        let (light, dark) = (core_card(false).unwrap(), core_card(true).unwrap());
        assert_ne!(light, dark);
        assert!(light.contains(CORE_PURPLE[0][1]) && dark.contains(CORE_PURPLE[1][1]));
        for s in [light, dark] {
            let style = &s[s.find("<style>").unwrap()..s.find("</style>").unwrap()];
            let (moving, still) = style
                .split_once("@media (prefers-reduced-motion:reduce){")
                .unwrap();
            let (overridden, _) = still.split_once("{animation:none}").unwrap();
            let mut animated = Vec::new();
            for rule in moving.split('}') {
                let Some((selectors, body)) = rule.split_once('{') else {
                    continue;
                };
                if body.starts_with("animation:") || body.contains(";animation:") {
                    for class in selectors.split(',') {
                        animated.push(class.trim_start_matches(['\n', '}']).to_string());
                    }
                }
            }
            let mut listed: Vec<String> = overridden.split(',').map(str::to_string).collect();
            animated.sort();
            listed.sort();
            assert_eq!(animated, listed);
            // Nothing outside the keyframes hides or strokes what a still
            // picture must show; pulse and glow carry no stroke of their own.
            let base = &style[..style.find("@keyframes").unwrap()];
            for hide in [
                "opacity:0",
                "dasharray",
                "dashoffset",
                "stroke:",
                "display",
                "visibility",
            ] {
                assert!(!base.contains(hide), "{hide}");
            }
            for class in ["<path class=\"cp\"", "<rect class=\"cg\""] {
                for el in s.split(class).skip(1) {
                    let el = &el[..el.find("/>").unwrap()];
                    assert!(!el.contains(" stroke=\""), "{el}");
                }
            }
            assert!(style.contains("stroke-dasharray:12 200;stroke-dashoffset:-110}"));
            assert!(style.contains("stroke-dasharray:100 100;stroke-dashoffset:0}"));
        }
    }
    // ===== TEST core END =====

    // ===== TEST q8 BEGIN =====
    /// A record of other sizes that still add up, drawn by hand.
    fn q8_mutant() -> Q8Record {
        Q8Record {
            values: 2048,
            mask: 256,
            record: 2304,
            copies: 9216,
        }
    }
    /// A commitments table with the given rows, as docs/COMMITMENTS.md writes one.
    fn q8_table(rows: &[String]) -> String {
        format!(
            "# Commitments\n\n## Published commitments\n\n| Date (UTC) | Label | Directories | Files | SHA-256 of the manifest |\n|---|---|---|---|---|\n{}\n\nAfter the table.\n",
            rows.join("\n")
        )
    }
    /// A table row: date, label, two counts and a hash.
    fn q8_row(date: &str, label: &str, counts: (&str, &str), hash: &str) -> String {
        format!(
            "| {date} | {label} | {} | {} | `{hash}` |",
            counts.0, counts.1
        )
    }
    /// Another hash of the target quantizer, read through the same readers.
    fn q8_pledge_mutant() -> Q8Pledge {
        let hash = format!("fedcba98{}", "7".repeat(56));
        let table = q8_table(&[
            q8_row(
                "2026-10-20",
                "Private data set 1",
                ("6", "44"),
                &"ab".repeat(32),
            ),
            q8_row("2026-12-24", Q8_TARGET, ("1,436", "40,054"), &hash),
        ]);
        Q8Pledge::from_table(&pledges_parse(&table).unwrap()).unwrap()
    }
    #[test]
    fn q8_card_shows_the_commitment_of_the_target_quantizer() {
        // Read a second way: the table line itself, split by hand.
        let text = fs::read_to_string(PLEDGES_FILE).unwrap();
        let lines: Vec<&str> = text
            .lines()
            .filter(|l| l.contains(&format!("| {Q8_TARGET} |")))
            .collect();
        assert_eq!(lines.len(), 1, "{lines:?}");
        let cells: Vec<&str> = lines[0].split('|').map(str::trim).collect();
        let (date, dirs, files) = (cells[1], cells[3], cells[4]);
        let hash = cells[5].trim_matches('`');
        assert_eq!(hash.len(), 64);
        let c = Q8Pledge::read().unwrap();
        assert_eq!((c.date.as_str(), c.hash.as_str()), (date, hash));
        let others: Vec<String> = Pledges::read()
            .unwrap()
            .rows
            .into_iter()
            .filter(|r| r.label != Q8_TARGET)
            .map(|r| r.hash)
            .collect();
        assert!(!others.is_empty());
        let unit =
            |n: &str, one: &str, many: &str| format!("{n} {}", if n == "1" { one } else { many });
        let counts = format!(
            "{}, {}",
            unit(dirs, "directory", "directories"),
            unit(files, "file", "files")
        );
        for dark in [false, true] {
            let s = q8_card(dark).unwrap();
            let p = q8_phone(dark).unwrap();
            assert!(s.contains(&format!(">{Q8_TARGET}: private<")));
            assert!(s.contains(&format!(">fixed {date} · {counts} · SHA-256 <")));
            assert!(p.contains(&format!(">private · fixed {date}<")));
            assert!(p.contains(&format!(">{counts}<")));
            for card in [&s, &p] {
                assert_eq!(card.matches(&format!(">{}</tspan>", &hash[..8])).count(), 1);
                // Only the first 8 hex characters, and no other hash.
                assert!(!card.contains(&hash[..9]));
                for other in &others {
                    assert!(!card.contains(&other[..8]), "{other}");
                }
                assert_eq!(q8_hex_runs(card), vec![hash[..8].to_string()]);
            }
            // The commitment is set apart from the record: its own kicker comes
            // first, once, and only then its label.
            for card in [&s, &p] {
                let kick = format!(">{Q8_SEPARATE}<");
                assert_eq!(card.matches(&kick).count(), 1);
                assert!(card.find(&kick).unwrap() < card.find(Q8_TARGET).unwrap());
            }
            // The limit of every commitment, whole on the wide card and line by
            // line on the phone.
            assert!(s.contains(&format!(">{Q8_PLEDGE_LIMIT}<")));
            let lines = wrap(Q8_PLEDGE_LIMIT, 28);
            assert!(lines.len() > 1);
            for line in lines {
                assert!(p.contains(&format!(">{line}<")), "{line}");
            }
        }
    }
    /// Every run of 8 or more hex characters in the text a card shows.
    fn q8_hex_runs(svg: &str) -> Vec<String> {
        let mut shown = String::new();
        for part in svg.split('>').skip(1) {
            shown.push_str(part.split('<').next().unwrap());
            shown.push(' ');
        }
        shown
            .split(|ch: char| !ch.is_ascii_hexdigit())
            .filter(|w| w.len() >= 8)
            .map(str::to_string)
            .collect()
    }
    #[test]
    fn q8_card_draws_another_commitment_and_refuses_none() {
        let c = q8_pledge_mutant();
        assert_eq!(c.short(), "fedcba98");
        assert_eq!(c.counts, ["1,436 directories", "40,054 files"]);
        let m = q8_mutant();
        for dark in [false, true] {
            for card in [q8_draw(dark, &m, &c), q8_phone_draw(dark, &m, &c)] {
                assert_eq!(q8_hex_runs(&card), vec!["fedcba98".to_string()]);
                assert!(card.contains("2026-12-24") && !card.contains("2026-11-05"));
                assert!(card.contains("1,436 directories, 40,054 files"));
            }
        }
        // No row of the target quantizer, or two of them: refused.
        let other = q8_row(
            "2026-10-20",
            "Private data set 1",
            ("6", "44"),
            &"ab".repeat(32),
        );
        let target = q8_row("2026-11-05", Q8_TARGET, ("1", "1"), &"0".repeat(64));
        for rows in [vec![other.clone()], vec![other, target.clone(), target]] {
            let p = pledges_parse(&q8_table(&rows)).unwrap();
            assert!(Q8Pledge::from_table(&p).is_err(), "{rows:?}");
        }
        // A label that only resembles it is not the row.
        let near = q8_row(
            "2026-11-05",
            "Target Q8 Quad quantizer 2",
            ("1", "1"),
            &"0".repeat(64),
        );
        assert!(Q8Pledge::from_table(&pledges_parse(&q8_table(&[near])).unwrap()).is_err());
    }
    /// Every text line of a card: its words, its left and right edge at 0.62 em
    /// per character with 8 % slack, and its top and bottom.
    fn q8_texts(svg: &str) -> Vec<(String, f64, f64, f64, f64)> {
        let attr = |head: &str, key: &str| -> Option<f64> {
            head.split(&format!(" {key}=\""))
                .nth(1)?
                .split('"')
                .next()?
                .parse()
                .ok()
        };
        let kick = if svg.contains(".kick{font-size:20px;") {
            20.0
        } else {
            13.0
        };
        let mut out = Vec::new();
        for part in svg.split("<text").skip(1) {
            let (head, rest) = part.split_once('>').unwrap();
            let raw = rest.split("</text>").next().unwrap();
            // Inner tags (tspans) carry no width; the largest size they set
            // measures the whole line.
            let mut text = String::new();
            let mut inner = 0.0_f64;
            for (i, piece) in raw.split('<').enumerate() {
                let words = if i == 0 {
                    piece
                } else {
                    let (tag, words) = piece.split_once('>').unwrap();
                    inner = inner.max(attr(&format!(" {tag}"), "font-size").unwrap_or(0.0));
                    words
                };
                text.push_str(words);
            }
            let text = text
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">");
            let n = text.chars().count();
            let (size, width) = if head.contains("class=\"kick\"") {
                (kick, q8_width(n, kick) + 2.0 * n as f64)
            } else {
                let size = attr(head, "font-size").unwrap().max(inner);
                (size, q8_width(n, size))
            };
            let x = attr(head, "x").unwrap();
            let y = attr(head, "y").unwrap();
            let l = if head.contains("text-anchor=\"middle\"") {
                x - width / 2.0
            } else if head.contains("text-anchor=\"end\"") {
                x - width
            } else {
                x
            };
            out.push((text, l, l + width, y - 0.8 * size, y + 0.2 * size));
        }
        out
    }
    /// The bytes drawn as cells in the block: values, then mask.
    fn q8_cells(svg: &str) -> [usize; 2] {
        let pitch: f64 = svg
            .split(r#"<pattern id="q8v""#)
            .nth(1)
            .unwrap()
            .split(" width=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let mut cells = [0.0, 0.0];
        for part in svg.split("<rect ").skip(1) {
            let head = part.split('>').next().unwrap();
            for (k, id) in ["url(#q8v)", "url(#q8m)"].iter().enumerate() {
                if head.contains(id) {
                    let w: f64 = head
                        .split(" width=\"")
                        .nth(1)
                        .unwrap()
                        .split('"')
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();
                    cells[k] += w / pitch;
                }
            }
        }
        cells.map(|c| c.round() as usize)
    }
    /// The widths of the drawn sizes: the stored record, then the four copies.
    fn q8_sizes(svg: &str) -> Vec<f64> {
        svg.split(r#"<rect class="q8size" "#)
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
            .collect()
    }
    #[test]
    fn q8_card_follows_its_contract() {
        let r = Q8Record::read().unwrap();
        // Counted a second way: from the constants, and from the contract's own
        // sentence about four copies.
        let record = RECORD_VALUES + RECORD_MASK;
        assert_eq!(
            r,
            Q8Record {
                values: RECORD_VALUES,
                mask: RECORD_MASK,
                record,
                copies: 4 * record
            }
        );
        let doc = fs::read_to_string(Q8_DOC).unwrap();
        assert!(doc.contains(&format!("{} bytes versus {record}", 4 * record)));
        assert_eq!(r.saved(), 75);
        for dark in [false, true] {
            let s = q8_card(dark).unwrap();
            for want in [
                ">1,024 values, one byte each<",
                ">1,152 B<",
                ">1,024 B values<",
                ">128 B activity mask<",
                ">1,024 B + 128 B = 1,152 B<",
                ">in memory, held once<",
                ">held once<",
                ">four written-out copies would take<",
                ">WHAT ONE Q8 RECORD HOLDS<",
                ">FOUR VIEWS, ONE RECORD IN MEMORY<",
                "<title id=\"t\">What one Q8 record holds</title>",
                ">the same 1,152 B<",
                ">4,608 B<",
                ">75% less record material, not 75% less process RAM<",
                ">not four independent memories<",
                ">not four votes<",
                ">not a speedup<",
                ">not a knowledge encoder<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            assert_eq!(s.matches(">✓ bit for bit<").count(), 4);
            let p = q8_phone(dark).unwrap();
            for want in [
                ">1,024 values<",
                ">1,152 B<",
                ">1,024 B<",
                ">128 B<",
                ">1,024 B + 128 B = 1,152 B<",
                ">in memory,<",
                ">held once<",
                ">4 copies would take<",
                ">WHAT ONE Q8 RECORD HOLDS<",
                ">FOUR VIEWS, ONE RECORD IN MEMORY<",
                "<title id=\"t\">What one Q8 record holds</title>",
                ">the same 1,152 B<",
                ">4,608 B<",
                ">75% less record material,<",
                ">not 75% less process RAM<",
                ">not 4 independent memories<",
                ">not 4 votes · not a speedup<",
                ">not a knowledge encoder<",
            ] {
                assert!(p.contains(want), "{want}");
            }
            assert_eq!(p.matches(">✓<").count(), 4);
            // The contract states a layout in memory and keeps the printing
            // step private: neither card says the record is stored or made.
            for c in [&s, &p] {
                let low = c.to_lowercase();
                for word in ["stored", "is made", "record is made"] {
                    assert!(!low.contains(word), "{word}");
                }
            }
            // One cell per byte, and the sizes to one scale: the record a
            // quarter of the four copies.
            for c in [&s, &p] {
                assert_eq!(q8_cells(c), [RECORD_VALUES, RECORD_MASK]);
                let sizes = q8_sizes(c);
                assert_eq!(sizes.len(), 5);
                assert!((sizes[0] - 130.0).abs() < 0.01, "{sizes:?}");
                for w in &sizes[1..] {
                    assert!((w - 126.0).abs() < 0.01, "{sizes:?}");
                }
            }
        }
        // It stands in its section, just before the record card.
        let readme = native_intro();
        let at = |needle: &str| readme.find(needle).unwrap();
        assert!(at("## One record, four exact views") < at("media/presentation/q8-light.svg"));
        assert!(at("media/presentation/q8-light.svg") < at("media/presentation/record-light.svg"));
    }
    #[test]
    fn q8_card_draws_the_numbers_it_is_given() {
        let m = q8_mutant();
        let c = q8_pledge_mutant();
        for dark in [false, true] {
            let s = q8_draw(dark, &m, &c);
            let p = q8_phone_draw(dark, &m, &c);
            for want in [
                ">2,048 values, one byte each<",
                ">2,304 B<",
                ">256 B activity mask<",
                ">9,216 B<",
            ] {
                assert!(s.contains(want), "{want}");
            }
            for want in [">2,048 values<", ">2,304 B<", ">256 B<", ">9,216 B<"] {
                assert!(p.contains(want), "{want}");
            }
            for c in [&s, &p] {
                assert!(!c.contains("1,152") && !c.contains("4,608") && !c.contains("1,024"));
                assert_eq!(q8_cells(c), [2048, 256]);
                let sizes = q8_sizes(c);
                assert!((sizes[0] - 520.0 * 2304.0 / 9216.0).abs() < 0.01);
            }
        }
        let alt = q8_text(&m);
        assert!(alt.contains("2,048") && alt.contains("2,304") && alt.contains("9,216"));
    }
    #[test]
    fn q8_card_refuses_a_contract_that_disagrees() {
        let doc = fs::read_to_string(Q8_DOC).unwrap();
        assert!(Q8Record::from_doc(&doc, RECORD_VALUES, RECORD_MASK).is_ok());
        // Constants the contract does not state.
        assert!(Q8Record::from_doc(&doc, 2048, 256).is_err());
        assert!(Q8Record::from_doc(&doc, RECORD_VALUES, 64).is_err());
        // A contract whose own numbers no longer add up, or that drops a limit
        // the card draws.
        for (from, to) in [
            ("1024 + 128 = 1152 bytes", "1024 + 128 = 1160 bytes"),
            ("1024 + 128 = 1152 bytes", "1024 + 120 = 1144 bytes"),
            ("holds 1024 phase", "holds 1000 phase"),
            ("and 1024 explicit", "and 1000 explicit"),
            ("occupy 4608 bytes", "occupy 4600 bytes"),
            ("versus 1152", "versus 1150"),
            ("not a knowledge encoder", "a readout"),
            ("not 75% less process RAM", "and less process RAM"),
            ("Four packed canonical records", "Packed records"),
            // The card says "in memory": the contract must keep saying so.
            (
                "This is an in-memory representation",
                "This is a representation",
            ),
            ("1152 bytes on the tested ABI", "1152 bytes on every ABI"),
        ] {
            assert!(doc.contains(from), "{from}");
            let changed = doc.replacen(from, to, 1);
            assert!(
                Q8Record::from_doc(&changed, RECORD_VALUES, RECORD_MASK).is_err(),
                "{to}"
            );
        }
        // The same contract laid out on other lines reads the same.
        let rewrapped = doc.replace(" bytes on the tested ABI", "\nbytes on the tested ABI");
        assert!(Q8Record::from_doc(&rewrapped, RECORD_VALUES, RECORD_MASK).is_ok());
    }
    #[test]
    fn q8_phone_card_passes_the_phone_checks() {
        check_phone("q8");
    }
    #[test]
    fn q8_text_fits_its_boxes_and_never_overlaps() {
        let m = q8_mutant();
        for (w, cards) in [
            (
                1200.0,
                [
                    q8_card(false).unwrap(),
                    q8_card(true).unwrap(),
                    q8_draw(false, &m, &q8_pledge_mutant()),
                ],
            ),
            (
                600.0,
                [
                    q8_phone(false).unwrap(),
                    q8_phone(true).unwrap(),
                    q8_phone_draw(false, &m, &q8_pledge_mutant()),
                ],
            ),
        ] {
            for s in cards {
                let lines = q8_texts(&s);
                assert!(lines.len() > 20);
                for (text, l, r, top, _) in &lines {
                    assert!(*l >= 30.0 && *r <= w - 30.0, "{text}: {l:.0} to {r:.0}");
                    // On the wide card nothing above the full-width rows (the
                    // chips and the commitment) crosses from one column into
                    // the other.
                    if w > 1000.0 && *l < 600.0 && *top < 560.0 {
                        assert!(*r <= 600.0, "{text}: {r:.0}");
                    }
                    // The commitment's lines stay inside its strip.
                    let (strip, right) = if w > 1000.0 {
                        (Q8_STRIP_WIDE, 1140.0)
                    } else {
                        (Q8_STRIP_PHONE, 540.0)
                    };
                    if *top > strip {
                        assert!(*l >= 90.0 && *r <= right, "{text}: {l:.0} to {r:.0}");
                    }
                }
                for (i, a) in lines.iter().enumerate() {
                    for b in &lines[i + 1..] {
                        let apart = a.2 <= b.1 || b.2 <= a.1 || a.4 <= b.3 || b.4 <= a.3;
                        assert!(apart, "{} overlaps {}", a.0, b.0);
                    }
                }
            }
        }
        // The fit test has teeth: a line too long for the phone fails it.
        let long = q8_line(40.0, 100.0, 20, "", &"x".repeat(42));
        let (_, _, r, _, _) = q8_texts(&long)[0];
        assert!(r > 570.0, "{r}");
        // With no animation at all the final state shows: no base rule hides a
        // thing; only keyframes start from nothing. Reduced motion stills every
        // animated class.
        let style = q8_style(theme(false));
        let base = style.split("@keyframes").next().unwrap();
        assert!(
            !base.contains("opacity") && !base.contains("scale"),
            "{base}"
        );
        let reduced = style
            .split_once("@media (prefers-reduced-motion:reduce){")
            .unwrap()
            .1;
        for class in base.split('.').skip(1) {
            let name = class.split(['{', ',']).next().unwrap();
            if class.contains("animation:") {
                assert!(reduced.contains(&format!(".{name}")), "{name}");
            }
        }
        // Every animated element of both cards holds its final state.
        for s in [q8_card(false).unwrap(), q8_phone(false).unwrap()] {
            assert_eq!(
                s.matches(" both}").count(),
                4,
                "four animations, all filling both ways"
            );
            assert!(!s.contains("infinite"));
        }
    }
    #[test]
    fn q8_does_not_say_how_a_view_arranges_bytes() {
        // The card says that four views read one record, never how a view
        // arranges its bytes; the words of the first-page check, and "arrang",
        // stay out of it.
        let lower_words = [
            "reversal",
            "additive offset",
            "modulo",
            "seed",
            "arrang",
            "/home/",
        ];
        let mut texts = vec![q8_alt().unwrap(), q8_text(&q8_mutant())];
        for dark in [false, true] {
            texts.push(q8_card(dark).unwrap());
            texts.push(q8_phone(dark).unwrap());
        }
        for text in &texts {
            let lower = text.to_lowercase();
            for word in lower_words {
                assert!(!lower.contains(word), "{word}");
            }
            assert!(
                text.chars().all(|c| c.is_ascii() || "·—→×✓".contains(c)),
                "{text}"
            );
            assert!(
                !text.contains("<image")
                    && !text
                        .replace("xmlns=\"http://www.w3.org/2000/svg\"", "")
                        .contains("http")
            );
        }
    }
    #[test]
    fn q8_guide_and_roadmap_say_what_the_record_holds() {
        let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let guide = flat(GUIDE);
        let roadmap = fs::read_to_string("docs/ROADMAP.md").unwrap();
        let row = roadmap
            .lines()
            .find(|l| l.starts_with("| Every wide card of the README also comes 600 wide"))
            .unwrap();
        for (text, wants) in [
            (
                guide.as_str(),
                [
                    "and what one Q8 record holds, as docs/Q8-QUAD.md states it:",
                    "a row of badges shows numbers read from files when the README is built, each linking to the file its number comes from;",
                ],
            ),
            (
                row,
                [
                    "a card of what one Q8 record holds, as docs/Q8-QUAD.md states it, with the separate commitment of the target quantizer;",
                    "each a number read from a file when the README is built and linking to the file it comes from;",
                ],
            ),
        ] {
            for want in wants {
                assert!(text.contains(want), "{want}");
            }
            for stale in ["record is made", "file it describes", "linking to it;"] {
                assert!(!text.contains(stale), "{stale}");
            }
        }
    }
    #[test]
    fn q8_alt_is_short_and_says_its_own_words() {
        let alt = q8_alt().unwrap();
        assert!(alt.chars().count() <= 300, "{}", alt.chars().count());
        assert!(!alt.contains('"'));
        for want in ["1,024", "128-byte", "1,152 bytes", "4,608 bytes", "votes"] {
            assert!(alt.contains(want), "{want}");
        }
        // What the record holds, in memory, as its contract states the layout.
        assert!(alt.starts_with("What a Q8 record holds: "), "{alt}");
        assert!(alt.contains(" bytes in memory, held once."), "{alt}");
        assert!(!alt.contains("stored") && !alt.contains("made"), "{alt}");
        // A sentence the claim check cites must stand in the README once: the
        // alternative text repeats none of them.
        let claims = fs::read_to_string("xtask/src/claims.rs").unwrap();
        let cited: Vec<String> = claims
            .split("words: \"")
            .skip(1)
            .map(|s| s.split("\",\n").next().unwrap().replace("\\\"", "\""))
            .collect();
        assert!(cited.len() > 10);
        assert!(cited
            .iter()
            .any(|c| c == "One 1,152-byte record read through four equivalent views"));
        for c in &cited {
            assert!(!alt.contains(c.as_str()), "{c}");
        }
        // Nor does it repeat the record card or any other words of its section:
        // no run of six words in common.
        let readme = native_intro();
        let section = readme
            .split_once("## One record, four exact views")
            .unwrap()
            .1
            .split_once("</details>")
            .unwrap()
            .0
            .replace(&alt, "");
        assert!(section.contains(&record_text(&Q8Runs::read().unwrap())));
        assert!(section.contains("One record, 1,152 bytes, read four ways, each bit for bit."));
        let words = |s: &str| -> Vec<String> {
            s.split_whitespace()
                .map(|w| {
                    w.trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase()
                })
                .collect()
        };
        let (mine, theirs) = (words(&alt), words(&section));
        for run in mine.windows(6) {
            assert!(!theirs.windows(6).any(|t| t == run), "{run:?}");
        }
    }
    // ===== TEST q8 END =====
}
