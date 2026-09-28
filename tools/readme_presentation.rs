//! Generate a script-free README presentation. Existing recordings are never rewritten.
#![forbid(unsafe_code)]
use std::{fs, io::Write, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// The release whose sources the page links to and checks out.
const RELEASE: &str = "v0.5.1";
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
        "06-compare",
        "GEL and grep. Answers before speed.",
        "See matching and differing answers. No universal speedup is claimed.",
    ),
];
const SVG: &str = r####"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="228" viewBox="0 0 1200 228" role="img" aria-labelledby="title desc">
<title id="title">GEL RAM | Evidence Lab</title><desc id="desc">Your documents. Exact quotes. A restart you can check. Public Rust tools with source-linked evidence. Animated logo: document sheets settle into a translucent GEL cube as source cells, a query lights one exact cell and a pin seal appears. Drawn, not recorded.</desc>
<style>
__LOGO_STYLE__
</style>
<defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="__BG1__"/><stop offset="1" stop-color="__BG2__"/></linearGradient></defs>
<rect x="1" y="1" width="1198" height="226" rx="24" fill="url(#bg)" stroke="__LINE__"/>
<path d="M680 0Q755 180 1200 92V0Z" fill="__ACCENT__" opacity=".06"/><path d="M560 228Q850 38 1200 156V228Z" fill="__ACCENT__" opacity=".05"/>
__LOGO__
<g font-family="Arial,Helvetica,sans-serif" fill="__INK__">
<text x="252" y="86" font-size="46" font-weight="700" letter-spacing="1">GEL RAM</text>
<text x="253" y="128" font-size="30" font-weight="600">Evidence Lab</text>
<text x="253" y="192" font-size="23">Your documents. Exact quotes. A restart you can check.</text>
<text x="814" y="76" font-size="16" font-weight="700" letter-spacing="2" fill="__ACCENT__">PUBLIC RUST TOOLS</text>
<text x="814" y="108" font-size="21">Source-linked evidence.</text>
<text x="814" y="137" font-size="21">Inspect it. Reproduce it.</text>
</g></svg>
"####;
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
        "06 · ANSWERS BEFORE SPEED",
        "M4 8h15l-3-3M20 16H5l3 3",
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
<title>GEL RAM | Evidence Lab | Multimedia guide</title>
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
<div class="brand-name">GEL RAM<small>Evidence Lab</small></div></div><p class="strap">Your documents. Exact quotes. A restart you can check.</p></div>
<div class="header-right"><p>Public tools for local work<br>and independent inspection.</p><div class="chips"><span class="chip">Rust 1.85</span><span class="chip">Local CLI</span><span class="chip">Source-linked media</span></div></div></div>
<nav aria-label="Page navigation"><a href="#content">Overview</a><a href="#quick-start">Quick start</a><a href="#workflows">Six workflows</a><a href="#checks">Checks</a><a href="#reproduce">Reproduce</a><a href="#documentation">Documentation</a><a href="#scope">Scope &amp; license</a></nav></header>
<div class="view-tools"><label for="still-mode">Show still images</label><a href="__DOC__/media/gifs/MANIFEST.txt">Media provenance</a><span>12-second edited replays, not wall time</span></div>
<main id="content">
<section class="hero" aria-labelledby="hero-heading">
<figure><a class="media-link" href="media/gifs/01-evidence-light.gif" aria-label="Open the exact quotes and restart animation at full size">__HERO__</a><figcaption>Real public CLI output, edited into three readable cards.<br><a href="media/gifs/01-evidence.txt">Full transcript and source revision</a> · <a href="media/gifs/01-evidence-dark.gif">Dark full-size version</a></figcaption></figure>
<div><p class="eyebrow">Inspect the source. Keep the evidence.</p><h1 id="hero-heading">Find the passage.<br>Check the source.</h1><p class="hero-lead">Load your text, retrieve exact quotations and check that a saved collection reopens with the expected bytes.</p><ul class="checks"><li>Source text, identifiers and byte ranges</li><li>SHA-256 pins retained independently</li><li>Explicit refusal after source replacement</li><li>Public Rust tools, without an LLM</li></ul><div class="buttons"><a class="button" href="#quick-start">Run it yourself →</a><a class="button secondary" href="#workflows">Explore six workflows</a></div><p class="boundary">Source correspondence is not source truth. This is phrase retrieval, not unrestricted question answering.</p></div>
</section>
<section class="section flow" aria-label="How a citation is checked">__FLOW__<div class="facts">__FACTS__</div></section>
<section class="section" id="workflows" aria-labelledby="workflow-heading"><div class="section-heading"><p class="eyebrow">Six workflows. Their original evidence.</p><h2 id="workflow-heading">See it in action</h2><p>These are edited replays of recorded public command output, not product screenshots or new benchmark runs. Open each image at full size to read the terminal text.</p></div><div class="grid">__CARDS__</div>
<div class="trust"><div><strong>Inspectable</strong><p>Transcripts and retained hashes accompany the previews.</p></div><div><strong>Explicit failures</strong><p>Inspect refusal and incomplete states, not only success.</p></div><div><strong>Local operation</strong><p>The CLI uses your local documents. No automatic upload.</p></div><div><strong>Reproducible scope</strong><p>Commands, sources and limitations remain linked.</p></div></div></section>
<section class="section" id="checks" aria-labelledby="checks-heading"><div class="section-heading"><p class="eyebrow">Drawn from this checkout</p><h2 id="checks-heading">What the public checks cover</h2><p>The property map, the CI exclusion list and the committed mutation report are read when the images are built; the read-only presentation check fails if they drift.</p></div>__WALL__<div class="facts">__BARS__</div></section>
<div class="columns section"><section id="quick-start"><p class="eyebrow">Try the public tool</p><h2>Start with your own source.</h2><p>In a new checkout, run each line separately. Setup needs a network; the demo uses locked offline dependencies.</p><pre><code>git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout __RELEASE__
git rev-parse HEAD
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo</code></pre><p>Expected marker: <code>GEL_EVIDENCE_DEMO=PASS</code>. This is a result to check, not a pre-approved outcome.</p></section>
<section id="reproduce"><p class="eyebrow">Check the declared scope</p><h2>Reproduce the checks.</h2><pre><code>cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p xtask -- reproduce ../gel-repro-new</code></pre><p>Use a new output directory outside the checkout. Read failed, skipped and unmeasured steps. Cargo offline mode is not system-wide network isolation.</p><ul class="doclinks"><li><a href="__DOC__/docs/REPRODUCE.md">Reproduction instructions</a></li><li><a href="__DOC__/docs/CI-EVIDENCE.md">Per-revision CI evidence</a></li><li><a href="__DOC__/docs/CLAIMS.md">Executable claim registry</a></li></ul></section>
<section id="documentation"><p class="eyebrow">Go beyond the preview</p><h2>Documentation.</h2><ul class="doclinks"><li><a href="__DOC__/README.md">Technical README</a></li><li><a href="__DOC__/docs/EVIDENCE-LAB.md">Evidence Lab guide</a></li><li><a href="__DOC__/docs/EVIDENCE-BATCH.md">Batch mode and exit codes</a></li><li><a href="__DOC__/docs/BACKUP.md">Backup and restore</a></li><li><a href="__DOC__/docs/MUTATION-MATRIX.md">Format mutation matrix</a></li><li><a href="__DOC__/docs/BENCHMARK-GREP.md">Comparison with grep</a></li><li><a href="__DOC__/docs/ROADMAP.md">Public roadmap</a></li><li><a href="__DOC__/media/INDEX.md">Films and their scope</a></li></ul><p>Documentation links open the pinned source on GitHub. Viewing this local page loads only adjacent assets, with no scripts, trackers or remote fonts.</p></section></div>
<section class="scope section" id="scope"><h2>What this page does and does not establish</h2><p>This is a documentation presentation, not the application interface. It belongs to the <code>__RELEASE__</code> source. The older GIFs retain the source revision declared in their <a href="media/gifs/MANIFEST.txt">original manifest</a>. Presentation changes do not retime them, certify a release or close outstanding human review.</p><p>Snapshots and backups are plaintext. A hash is not encryption, a signature or proof that a source is true. Different search tasks and matching semantics are not interchangeable benchmarks.</p><p>GEL-owned material remains under <a href="__DOC__/LICENSE">GEL RAM Noncommercial Reciprocal License 1.0</a>. Read the <a href="__DOC__/LICENSING.md">licensing guide</a>, <a href="__DOC__/THIRD-PARTY-NOTICES.md">third-party notices</a> and <a href="__DOC__/media/RIGHTS.md">media rights</a>. This page grants no additional rights.</p></section>
</main>
<footer><p><strong>GEL RAM · Evidence Lab</strong><br>RR, GEL RAM Project · Public documentation</p><p><a href="__DOC__/LICENSE">License</a> · <a href="__DOC__/SECURITY.md">Security</a> · <a href="__DOC__/CLA-PRIVACY.md">CLA privacy</a> · <a href="__DOC__/CONTRIBUTING.md">Contributing</a><br>Existing evidence preserved. New presentation only.</p></footer>
</div></body></html>
"####;
const NATIVE: &str = r####"# GEL RAM

<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/presentation/header-dark.svg">
  <img alt="GEL RAM Evidence Lab. Your documents. Exact quotes. A restart you can check. Animated logo: document sheets settle into a translucent cube, a query lights one exact cell and a pin seal appears." src="media/presentation/header-light.svg" width="1200">
</picture>

__RESULTS__

__FLOW__

**Find the passage. Check the source.** Local Rust tools for exact source-bound
quotations, stale-citation refusal and independently pinned snapshots.

__FACTS__

[Quick start](#quick-start) · [Six workflows](#see-it-in-action) · [Checks](#what-the-public-checks-cover) · [Reproduce](#reproduce-the-checks) · [Documentation](#documentation) · [License](#about-and-licensing)

> **Version 0.5.1.** The instructions below use the `v0.5.1` tag. Record the exact
> commit you test.
> [Release notes](RELEASE-NOTES-v0.5.1.md) · [Publication status](CANDIDATE-STATUS.md) · [Tagged releases](https://github.com/Gelram-project/gel-ram/releases)

**Full-page edition:** [README-MULTIMEDIA.html](README-MULTIMEDIA.html).
Open that file from this checkout in a browser for the responsive blue-panel layout,
light/dark backgrounds and a still-image control. GitHub displays HTML files as
source, not as a hosted page; this repository does not enable Pages.
[Presentation guide](docs/README-PRESENTATION.md).

## See it in action

**Actual public-tool runs, presented as six edited log replays.**
Each animation lasts 12 seconds; pacing is editorial, not execution time.
[Static view](media/gifs/STATIC.md) · [Full gallery](media/gifs/README.md) · [Original source and hashes](media/gifs/MANIFEST.txt)

__HERO_CHIP__

__HERO__

[Full transcript](media/gifs/__HERO_ID__.txt) · [Full-size dark replay](media/gifs/__HERO_ID__-dark.gif)

### Six workflows, one evidence trail

Open a preview at full size to read the terminal text. All six existing scenarios
are visible here rather than hidden in collapsed sections. Their original
transcripts and source revisions remain unchanged.

__GRID__

The earlier [70-second Evidence Lab film](media/GEL-EVIDENCE-LAB-EN.mp4),
its [original process logs](media/EVIDENCE-LAB-GUIDE.md) and
[open human review](docs/MEDIA-DECODE-REVIEW.md) remain separate historical material.
New presentation is not a new execution, benchmark or human acceptance.

## What the public checks cover

__WALL__

Each cell is one documented property on one CI platform. On every platform the
[CI evidence collector](docs/CI-EVIDENCE.md) withholds COMPLETE unless each test
named in the [property map](docs/PROPERTY-TESTS.md) ran and passed exactly once;
declared Unix-only tests are exempt on Windows. The image shows this rule, not
the outcome of one run: read that in the run's CI report.

__BARS__

Every mutant of the [format mutation matrix](docs/MUTATION-MATRIX.md) is read
twice: with the original pin, which must refuse any changed byte, and with a
recomputed pin, which lets the structural rules decide. The bars count the
committed report; `xtask verify` regenerates it and fails on any difference.
Both images are drawn from these files when the presentation is built, and the
read-only presentation check fails if they drift.

## Measured GEL results — scope matters

These are **author-run measurements of a separate private implementation**,
not benchmarks of this public checkout or an LLM leaderboard.
They are reported here without publishing the private engine.

| Operation | Observations | Measured result | What it establishes |
|:---|---:|:---|:---|
| Resident read at a known address | 40 | p50 **53.872 µs**, p95 **79.640 µs**, p99 **90.009 µs**; 40/40 reference matches | Addressed read after loading into RAM, not semantic search |
| Source-integrity gate | 1,000 source fragments; 5 controls each | **1,000 valid payloads admitted; 4,000 invalid cases rejected** | Changed payload, missing address, wrong source and stale catalog generation are distinguished |
| Single Q8 ranking | 400 probes | top-1 **368/400 (92%)**; top-10 **393/400 (98.25%)** | Ranking within the known 250k-record slot |
| Quad ranking | Same 400 probes | top-1 **310/400 (77.5%)**; top-10 **361/400 (90.25%)** | Same diagnostic task; Quad did not outperform Single in this run |

The logical ranking bank contains **1M fragments across four 250k slots**;
these probes do **not** search all 1M candidates. One empty probe remains in
the quality denominator. Integrity controls use a separate 1,000-fragment
PL/EN catalog, not the million-record bank. The gate is an experimental
CPU adapter, tested offline in a Linux sandbox, not a deployed service.

Percentiles use nearest-rank; with N=40, p99 is the maximum. These are
single-host diagnostic runs, not independently replicated measurements.
Matching source bytes does not establish truth or understanding.
Neither hardware-level memory computation nor a speedup over LLMs is
established by these tests.

The underlying logs and private harness remain outside this checkout.
**These rows are not independently reproducible from the public release.**
Public-tool demonstrations and their reproducible evidence above retain
their own, separate scope. No private version identifiers, source code,
knowledge banks or credentials are included here.

### New measurements: ranking improvement and source-field dialogue

![Animated measured-results table: Quad top-1 improved from 77.5% to 92.75% on 400 known-slot source probes. Source-field dialogue has microsecond medians on a separate five-article pilot. Groq batches are a different task; no speedup ratio.](media/presentation/measured-progress.svg)

Animation highlights rows only; it is **not execution footage or a timing scale**.
All numbers remain visible, with a reduced-motion mode and the text table below.
These author-run private experiments do not change the public implementation.

| Experiment | Earlier result | Updated measurement | Scope |
|:---|:---|:---|:---|
| Quad top-1 | 310/400 (77.5%) | **371/400 (92.75%)** | Same 400 source-text probes; known 250k slot within a 1M bank |
| Quad top-10 | 361/400 (90.25%) | **396/400 (99%)** | Not natural-question accuracy or global 1M search |
| Single Q8 top-1 / top-10 | 368/400 / 393/400 | **379/400 / 397/400** | Single still outperforms Quad on this diagnostic |
| Source-field dialogue | Separate task | **p50 2.054–4.819 µs** across five field questions and three runs | Five articles in RAM; finished source-bound text, not a million-record search |
| Two-source whole-field comparison | Separate task | **p50 19.417–21.220 µs** across three runs | Exact field comparison, not unrestricted reasoning |
| Transfer to four new articles | No latency measurement | **6 source-field answers, 2 quotations, 1 limited comparison, 3 UNHANDLED, 1 UNKNOWN** | 13 PL questions; not 9/13 accuracy |

The timing pilot used 1,000 warm repetitions **per question per run**.
The slowest retained warm observation was **711.017 µs**; medians are not
worst-case guarantees. Newer quotation/list support was not timed in that
campaign. A quotation is not a verified semantic decision.

[All 13 timing rows, provenance and limitations](docs/MEASURED-PROGRESS.md).
The Groq table below remains a separate supplied-source decision diagnostic;
**no GEL/Groq speedup ratio follows from these different tasks**.

### Answer verdict: answer only when the lead is clear

Measured after the v0.5.0 release on the same private 1M bank as the ranking
rows above. GEL answers only when its best passage leads the runner-up by a
threshold fixed in advance (0.06484, set on a different corpus); otherwise it
returns UNKNOWN. Three changes were applied and measured one at a time:

1. identical passages, including ones the encoder cannot tell apart, are stored
   once, with every source and text variant kept;
2. reference, link and bibliography sections stay available as sources but are
   left out of the answer bank (Polish slots −27%, English −3–5%);
3. the private encoder keeps numbers and short words, so passages that differ
   only by a year or a score stay apart.

| Stored passages read back (ranked within their slot) | Original bank | After the three changes |
|:---|---:|---:|
| Probes | 10,000 | 50,000 |
| Answered | 77.0% | 92.8% |
| Correct answers (same article) | 97.7% | **99.95%** (23 wrong of 46,376) |
| Wrong answers among all probes | 1.78% | **0.046%** |
| UNKNOWN | 23.1% | 7.2% |

Two of the four slots reach a 95% Wilson lower bound of at least 0.999 (0.9993
and 0.9994); the other two reach 0.9984 and 0.9986. The probes are stored
passages, not questions; after the changes the bank holds 167,854 passages per
slot.

| 80 natural questions (40 PL, 40 EN), all slots searched | Without verification | With source verification |
|:---|---:|---:|
| Top-1 from the right article | 27 (34%) | **40 (50%)** |
| Answers given | 9 (1 wrong) | **11 (all correct)** |
| UNKNOWN | 71 | 69 |

The questions were written by the project's AI coding assistant for randomly
sampled passages and frozen before any run. Verification compares a question
with the stored sources of 128 candidates; its threshold was set on a separate
calibration set of 80 questions. 11 of 11 has a 95% Wilson lower bound of about 0.74, so this is
not a precision claim. Answering natural questions (14% answered) remains the
open problem.

[Protocol, per-slot results and evidence identities](docs/MEASURED-PROGRESS.md).

### Side by side with three language models

The same 80 frozen questions went to GEL RAM and, closed book, to three
language models on the Groq API, in one recorded run with one scoring rule.
GEL answered 11 and said UNKNOWN to 69; **none of its answers was wrong**. The
models could also say UNKNOWN, yet **11–28 of their answers were wrong**.

| Same 80 questions | Answered | Correct | Wrong | UNKNOWN |
|:---|---:|---:|---:|---:|
| GEL RAM (local bank, answers with the source passage) | 11 | 11 | **0** | 69 |
| GPT-OSS-120B (Groq API, closed book) | 31 | 10 | 21 | 49 |
| GPT-OSS-20B (Groq API, closed book) | 36 | 8 | 28 | 44 |
| Qwen3.8-27B (Groq API, closed book) | 17 | 6 | 11 | 63 |

[![Each of the 80 questions as one cell per system. GEL RAM: 11 correct, 0 wrong, 69 UNKNOWN. GPT-OSS-120B: 10 correct, 21 wrong. GPT-OSS-20B: 8 correct, 28 wrong. Qwen3.8-27B: 6 correct, 11 wrong.](media/beside-groq/all-80-answers.png)](docs/GEL-BESIDE-GROQ.md)

The two sides do different jobs: GEL looks facts up in a bank it holds, the
models answer from training. On 10 of GEL's 11 answers no model was correct; on
12 other questions a model was correct where GEL said UNKNOWN. The visible model
answers were two words at the median; the GPT-OSS models also generated 51 and
154 hidden reasoning tokens per question on average. GEL returns the stored
source passage (33 words at the median). Times are recorded, not compared: GEL
0.25 s for all 80 locally, the models 59–371 ms per question at the median over
the network. [All 80 answers, times, prompts, review and limits](docs/GEL-BESIDE-GROQ.md)
· [Replay of every question (5 min)](media/beside-groq/GEL-BESIDE-GROQ-80-QUESTIONS-EN.mp4)

### Same supplied-source task: GEL adapter and models served by Groq

Twelve development claims (six PL, six EN), with the same supplied Wikipedia
passages and prompts. **One timed batch per language and profile**, not six
latency observations. Label agreement is separate from citation/format validity.

| System / profile | Language | Batch time | Labels matching working gold | Label + required structure | S/R decisions |
|:---|:---:|---:|---:|---:|---:|
| GEL bounded adapter R0 | PL | **85.851 µs** | 2/6 | 2/6 | **0/6** |
| GEL bounded adapter R0 | EN | **78.057 µs** | 2/6 | 2/6 | **0/6** |
| Groq / Qwen R0 | PL | 537.134 ms | 5/6 | 5/6 | 5/6 |
| Groq / Qwen R0 | EN | 552.672 ms | 6/6 | 6/6 | 4/6 |
| Groq / GPT-OSS-20B R1 | PL | 818.245 ms | 5/6 | 1/6 | 5/6 |
| Groq / GPT-OSS-20B R1 | EN | 879.478 ms | protocol rejected | 0/6 | 0/6 admitted |
| Groq / GPT-OSS-120B R1 | PL | 1219.564 ms | 5/6 | 5/6 | 5/6 |
| Groq / GPT-OSS-120B R1 | EN | 1035.560 ms | 6/6 | 1/6 | 4/6 |

**GEL returned UNKNOWN for every claim because the grammar was unsupported.**
Its microsecond times measure parsing and abstention, not successful semantic
decisions; the 2/6 agreement is the always-UNKNOWN baseline.
Groq times include HTTP/network and generation. There is **no justified
GEL/LLM speedup multiplier** here, and N=1 does not support latency percentiles.
The adapter is not a complete GEL application or an Ocean retrieval benchmark.

Earlier 20B and 120B PL attempts were incomplete at the 1024-token limit
(1383.914 ms and 2407.528 ms). They are retained in the
[protocol, exact times and failure notes](docs/GEL-GROQ-DIAGNOSTIC.md).
This small, development-exposed comparison is not independently validated
or a general model ranking. Private code, banks and API credentials stay private.

"####;
const GUIDE: &str = r####"# Multimedia README presentation

This presentation has two views. The root README is normal GitHub Markdown with
an SVG banner with the animated GEL logo, an animated 3D scene of the citation
check, a strip of checked facts, colour-coded workflow badges, six real GIF
previews and two 3D graphics of the public checks. The adjacent
README-MULTIMEDIA.html is a script-free local document with the full responsive
panel layout, theme-aware backgrounds, a still-image control and links to the
public documentation. Open it from the checkout in a browser. GitHub's file
viewer does not host it as a web page; no Pages deployment or new public service
is configured.

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

## Preservation and source

The presentation belongs to the __RELEASE__ source. The exact prior README is
retained as [README-HISTORY-PRE-MULTIMEDIA.md](../README-HISTORY-PRE-MULTIMEDIA.md).
The already existing pre-visual history also remains untouched. The new README
keeps its technical content below the presentation unchanged, including the
quick-start checkout of the __RELEASE__ tag.

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

The original [static gallery](../media/gifs/STATIC.md) remains available. Opening
external documentation links needs a network; opening the local presentation does not.

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
fn header(dark: bool) -> String {
    let values = if dark {
        ["#101d31", "#0d1117", "#303d50", "#79c0ff", "#e6edf3"]
    } else {
        ["#f7fbff", "#deedff", "#cbdcf0", "#0969da", "#102949"]
    };
    let mut s = SVG.to_string();
    for (key, val) in ["BG1", "BG2", "LINE", "ACCENT", "INK"]
        .into_iter()
        .zip(values)
    {
        s = s.replace(&format!("__{key}__"), val);
    }
    let (o, a) = ((120.0, 118.0), 86.0);
    let logo = gel_logo(theme(dark), values[3], o, a) + &gel_sheets(theme(dark), values[3], o, a);
    s.replace("__LOGO_STYLE__", LOGO_STYLE)
        .replace("__LOGO__", &logo)
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
fn bars_picture() -> String {
    themed(
        "media/presentation/bars-light.svg",
        "media/presentation/bars-dark.svg",
        "Format mutation matrix: rejected and accepted rows per public format in the committed report, and how many rows matched their expected verdict.",
        1200,
    )
}
/// The recorded side-by-side run, read from the published evidence file: one
/// verdict letter per question (C correct, W wrong, U UNKNOWN, E error) for each
/// system, and question 41 as the worked example.
struct Recorded {
    systems: Vec<(&'static str, Vec<u8>)>,
    question: String,
    passage: String,
    answers: Vec<(&'static str, String, u8)>,
}
const RECORDED: &str = "docs/evidence-side-by-side/side-by-side.txt";
const RECORDED_SYSTEMS: [(&str, &str); 4] = [
    ("GEL RAM", "GEL"),
    ("GPT-OSS-120B", "gpt-oss-120b"),
    ("GPT-OSS-20B", "gpt-oss-20b"),
    ("Qwen3.8-27B", "qwen3.8-27b"),
];
impl Recorded {
    fn read() -> Result<Recorded> {
        let text = fs::read_to_string(RECORDED)?;
        let header: Vec<&str> = text
            .lines()
            .find_map(|l| l.strip_prefix("# nr\t"))
            .ok_or("missing evidence header")?
            .split('\t')
            .collect();
        let col = |name: &str| {
            header
                .iter()
                .position(|h| *h == name)
                .map(|i| i + 1)
                .ok_or(format!("missing evidence column {name}"))
        };
        let rows: Vec<Vec<&str>> = text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| l.split('\t').collect())
            .collect();
        if rows.len() != 80
            || rows
                .iter()
                .enumerate()
                .any(|(i, r)| r[0] != (i + 1).to_string())
        {
            return Err("the evidence file must hold questions 1 to 80 in order".into());
        }
        let letter = |v: &str| match v {
            "CORRECT" => b'C',
            "WRONG" => b'W',
            "UNKNOWN" => b'U',
            _ => b'E',
        };
        let mut systems = Vec::new();
        for (name, id) in RECORDED_SYSTEMS {
            let c = col(id)?;
            systems.push((name, rows.iter().map(|r| letter(r[c])).collect()));
        }
        let ex = &rows[40];
        let mut answers = Vec::new();
        for (name, id) in RECORDED_SYSTEMS.iter().skip(1) {
            answers.push((
                *name,
                ex[col(&format!("{id}_answer"))?].to_string(),
                letter(ex[col(id)?]),
            ));
        }
        Ok(Recorded {
            systems,
            question: ex[col("question")?].to_string(),
            passage: ex[col("GEL_source_excerpt")?].to_string(),
            answers,
        })
    }
    /// Correct, wrong and UNKNOWN answers of one system.
    fn counts(&self, i: usize) -> (usize, usize, usize) {
        let v = &self.systems[i].1;
        let n = |c: u8| v.iter().filter(|x| **x == c).count();
        (n(b'C'), n(b'W'), n(b'U'))
    }
}
/// Probes, answered and correct shares of the large read-back sample in docs/MEASURED-PROGRESS.md.
fn readback() -> Result<(String, String, String)> {
    let text = fs::read_to_string("docs/MEASURED-PROGRESS.md")?;
    let row = text
        .lines()
        .find(|l| l.starts_with("| Same, large sample |"))
        .ok_or("missing large read-back row")?;
    let f: Vec<&str> = row.split('|').map(str::trim).collect();
    Ok((f[3].to_string(), f[4].to_string(), f[5].to_string()))
}
/// How many claims the registry in docs/CLAIMS.md holds in each evidence mode.
fn statuses() -> Result<Vec<(&'static str, usize)>> {
    let text = fs::read_to_string("docs/CLAIMS.md")?;
    let table = text
        .split_once("<!-- REGISTRY-BEGIN -->")
        .and_then(|(_, rest)| rest.split_once("<!-- REGISTRY-END -->"))
        .ok_or("missing claim registry")?
        .0;
    let modes: Vec<&str> = table
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| ID"))
        .filter_map(|l| l.split('|').nth(3).map(str::trim))
        .collect();
    let order = [
        "EXECUTABLE_CHECK",
        "SEPARATE_GATE",
        "MEASURED_LOCAL",
        "NOT_VERIFIED",
        "NOT_ESTABLISHED",
    ];
    if modes.is_empty() || modes.iter().any(|m| !order.contains(m)) {
        return Err("unexpected claim registry mode".into());
    }
    Ok(order
        .iter()
        .map(|m| (*m, modes.iter().filter(|x| *x == m).count()))
        .collect())
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
/// Greedy word wrap into at most `lines` lines; the last line ends with "…" when cut.
fn wrap(s: &str, width: usize, lines: usize) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    for word in s.split_whitespace() {
        let last = out.last_mut().unwrap_or_else(|| unreachable!());
        if !last.is_empty() && last.chars().count() + 1 + word.chars().count() > width {
            out.push(word.to_string());
        } else {
            if !last.is_empty() {
                last.push(' ');
            }
            last.push_str(word);
        }
    }
    if out.len() > lines {
        out.truncate(lines);
        if let Some(l) = out.last_mut() {
            l.push('…');
        }
    }
    out
}
fn wilson_low(k: usize, n: usize) -> f64 {
    let (z, n, p) = (1.96f64, n as f64, k as f64 / n as f64);
    let c = p + z * z / (2.0 * n);
    let r = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    (c - r) / (1.0 + z * z / n)
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
fn glance(dark: bool, r: &Recorded, rb: &(String, String, String)) -> String {
    let t = theme(dark);
    let (ok, wrong, unknown) = r.counts(0);
    let low = wilson_low(ok, ok + wrong);
    let blocks = [
        (
            format!("{ok} / 80"),
            t.accent,
            "correct answers",
            "each with its source passage".to_string(),
        ),
        (
            wrong.to_string(),
            t.good,
            "wrong answers",
            format!(
                "among the {} answers · 95% Wilson lower bound {low:.2}",
                ok + wrong
            ),
        ),
        (
            unknown.to_string(),
            t.ink,
            "UNKNOWN",
            "said instead of guessing".to_string(),
        ),
    ];
    let mut body = String::from(
        r#"<text class="kick" x="48" y="56">ANSWER OR ABSTAIN · 80 NATURAL QUESTIONS · ONE RECORDED RUN</text><text x="48" y="100" font-size="34" class="b">Evidence you can inspect</text>"#,
    );
    for (i, (big, colour, label, note)) in blocks.iter().enumerate() {
        let x = 48 + i * 384;
        body.push_str(&format!(r#"<text x="{x}" y="180" font-size="60" class="b" style="fill:{colour}">{big}</text><text x="{x}" y="214" font-size="19" class="b">{label}</text><text x="{x}" y="240" font-size="15" class="mut">{note}</text>"#));
    }
    body.push_str(&format!(r#"<path d="M48 266H1152" stroke="{}"/><text x="48" y="298" font-size="15" class="mut">Separate experiment, not questions: {} correct answers when {} stored passages are read back ({} answered).</text>"#, t.line, rb.2, rb.0, rb.1));
    result_card(dark, 330, ("Evidence you can inspect", &format!("On 80 natural questions GEL RAM gave {ok} correct answers, {wrong} wrong and {unknown} UNKNOWN.")), &body)
}
fn path_card(dark: bool) -> String {
    let t = theme(dark);
    let steps = [
        ("QUESTION", "in natural language"),
        ("RETRIEVAL", "search the whole bank"),
        ("SOURCE", "one stored passage"),
        ("ANSWER / UNKNOWN", "answer only when it is clear"),
        ("EVIDENCE", "source, hash, recorded run"),
    ];
    let mut body =
        String::from(r#"<text class="kick" x="40" y="44">WHAT HAPPENS TO A QUESTION</text>"#);
    for (i, (name, note)) in steps.iter().enumerate() {
        let x = 40 + i * 232;
        let stroke = if i == 3 { t.accent } else { t.line };
        body.push_str(&format!(r#"<rect x="{x}" y="64" width="200" height="72" rx="14" fill="{}" stroke="{stroke}" stroke-width="2"/><text x="{}" y="96" font-size="15" class="b" text-anchor="middle" letter-spacing="1">{name}</text><text x="{}" y="120" font-size="13" class="mut" text-anchor="middle">{note}</text>"#, t.bg[0], x + 100, x + 100));
        if i < steps.len() - 1 {
            let a = x + 204;
            body.push_str(&format!(r#"<path d="M{a} 100H{} M{} 94L{} 100L{} 106" stroke="{}" stroke-width="2" fill="none"/>"#, a + 22, a + 16, a + 23, a + 16, t.muted));
        }
    }
    result_card(dark, 160, ("What happens to a question", "Question, retrieval from the whole bank, one stored source passage, an answer only when it is clear or UNKNOWN, and the evidence: source, hash and recorded run."), &body)
}
fn pill(x: usize, y: usize, verdict: u8, t: &Theme) -> String {
    let (label, colour) = match verdict {
        b'C' => ("CORRECT", t.green[2]),
        b'W' => ("WRONG", t.red[2]),
        b'U' => ("UNKNOWN", t.muted),
        _ => ("ERROR", t.amber[2]),
    };
    format!(
        r##"<rect x="{x}" y="{y}" width="96" height="24" rx="12" fill="{colour}"/><text x="{}" y="{}" font-size="12" class="b" text-anchor="middle" style="fill:#ffffff" letter-spacing="1">{label}</text>"##,
        x + 48,
        y + 16
    )
}
fn example(dark: bool, r: &Recorded) -> String {
    let t = theme(dark);
    let mut body = String::from(
        r#"<text class="kick" x="48" y="50">ONE QUESTION, FOUR SYSTEMS · QUESTION 41 OF 80 · RECORDED RUN</text>"#,
    );
    body.push_str(&format!(
        r#"<text x="48" y="92" font-size="24" class="b">{}</text>"#,
        esc(&r.question)
    ));
    body.push_str(&format!(
        r#"<text x="48" y="140" font-size="18" class="b">GEL RAM</text>{}"#,
        pill(48, 152, r.systems[0].1[40], t)
    ));
    body.push_str(
        r#"<text x="240" y="140" font-size="14" class="mut">Source passage it returned:</text>"#,
    );
    for (i, line) in wrap(&r.passage, 104, 3).iter().enumerate() {
        body.push_str(&format!(
            r#"<text x="240" y="{}" font-size="16">{}</text>"#,
            164 + i * 22,
            esc(line)
        ));
    }
    for (i, (name, answer, verdict)) in r.answers.iter().enumerate() {
        let y = 256 + i * 58;
        body.push_str(&format!(r#"<text x="48" y="{y}" font-size="18" class="b">{name}</text>{}<text x="240" y="{}" font-size="16">{}</text>"#, pill(48, y + 10, *verdict, t), y + 14, esc(&wrap(answer, 104, 1)[0])));
    }
    body.push_str(r#"<text x="48" y="438" font-size="13" class="mut">Passage from Wikipedia (CC BY-SA 4.0). Models called through the Groq API, closed book. Full record: docs/GEL-BESIDE-GROQ.md</text>"#);
    result_card(dark, 460, ("One question, four systems", &format!("Question 41: {} GEL RAM returned the source passage: {} The three models answered: {}.", r.question, r.passage, r.answers.iter().map(|a| a.1.as_str()).collect::<Vec<_>>().join(" / "))), &body)
}
fn dots(dark: bool, r: &Recorded) -> String {
    let t = theme(dark);
    let mut body = String::from(
        r#"<text class="kick" x="48" y="50">ANSWER OR ABSTAIN · SAME 80 QUESTIONS, SAME RULE · ONE RECORDED RUN</text>"#,
    );
    for (row, (name, verdicts)) in r.systems.iter().enumerate() {
        let (ok, wrong, unknown) = r.counts(row);
        let y = 96 + row * 54;
        body.push_str(&format!(r#"<text x="48" y="{y}" font-size="17" class="b">{name}</text><text x="48" y="{}" font-size="13" class="mut">{ok} correct · {wrong} wrong · {unknown} UNKNOWN</text>"#, y + 20));
        for (i, v) in verdicts.iter().enumerate() {
            let cx = 336.0 + i as f64 * 10.4 + if i >= 40 { 8.0 } else { 0.0 };
            let cy = y as f64 + 2.0;
            body.push_str(&match v {
                b'C' => format!(r#"<circle cx="{cx:.1}" cy="{cy}" r="4.2" fill="{}"/>"#, t.green[1]),
                b'W' => format!(r#"<circle cx="{cx:.1}" cy="{cy}" r="4.2" fill="{}"/>"#, t.red[1]),
                b'U' => format!(r#"<circle cx="{cx:.1}" cy="{cy}" r="3.6" fill="none" stroke="{}" stroke-width="1.2"/>"#, t.muted),
                _ => format!(r#"<circle cx="{cx:.1}" cy="{cy}" r="4.2" fill="{}"/>"#, t.amber[1]),
            });
        }
    }
    body.push_str(&format!(r#"<circle cx="54" cy="316" r="4.2" fill="{}"/><text x="66" y="321" font-size="13" class="mut">correct</text><circle cx="140" cy="316" r="4.2" fill="{}"/><text x="152" y="321" font-size="13" class="mut">wrong</text><circle cx="214" cy="316" r="3.6" fill="none" stroke="{}" stroke-width="1.2"/><text x="226" y="321" font-size="13" class="mut">UNKNOWN</text><text x="336" y="321" font-size="13" class="mut">one dot per question · 1–40 Polish, 41–80 English · models via the Groq API, closed book</text>"#, t.green[1], t.red[1], t.muted));
    let desc = r
        .systems
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            let (a, b, c) = r.counts(i);
            format!("{name}: {a} correct, {b} wrong, {c} UNKNOWN")
        })
        .collect::<Vec<_>>()
        .join(". ");
    result_card(dark, 344, ("Answer or abstain", &desc), &body)
}
fn surface(dark: bool, s: &[(&str, usize)]) -> String {
    let t = theme(dark);
    let meaning = |m: &str| match m {
        "EXECUTABLE_CHECK" => ("run against the library", "in CI on every change"),
        "SEPARATE_GATE" => ("checked by its own", "gate or report"),
        "MEASURED_LOCAL" => ("measured by the owner;", "not re-runnable here"),
        "NOT_VERIFIED" => ("open:", "not verified yet"),
        _ => ("not shown,", "so not claimed"),
    };
    let mut body = String::from(
        r#"<text class="kick" x="40" y="48">TRUTH SURFACE · EVERY CLAIM IN THE REGISTRY HAS ONE STATUS</text>"#,
    );
    for (i, (mode, n)) in s.iter().enumerate() {
        let x = 40 + i * 228;
        let (a, b) = meaning(mode);
        body.push_str(&format!(r#"<rect x="{x}" y="68" width="212" height="132" rx="14" fill="{}" stroke="{}"/><text x="{}" y="116" font-size="40" class="b">{n}</text><text x="{}" y="142" font-size="12" class="mono">{mode}</text><text x="{}" y="166" font-size="13" class="mut">{a}</text><text x="{}" y="184" font-size="13" class="mut">{b}</text>"#, t.bg[0], t.line, x + 18, x + 18, x + 18, x + 18));
    }
    body.push_str(r#"<text x="40" y="232" font-size="14" class="mut">No claim is marked as independently reproduced: none has been. The registry is docs/CLAIMS.md.</text>"#);
    let desc = s
        .iter()
        .map(|(m, n)| format!("{n} {m}"))
        .collect::<Vec<_>>()
        .join(", ");
    result_card(dark, 256, ("Truth surface", &format!("Claims in the registry by status: {desc}. None is marked as independently reproduced.")), &body)
}
fn goals(dark: bool, r: &Recorded) -> String {
    let t = theme(dark);
    let (ok, wrong, _) = r.counts(0);
    let rows = [
        (
            "Natural questions",
            format!("{} of 80 answered, {wrong} wrong", ok + wrong),
            "most of a new, frozen, held-out set answered, still 0 wrong",
        ),
        (
            "Answer form",
            "the whole source passage".to_string(),
            "a short answer taken from the passage, scored by the models' rule",
        ),
        (
            "No correct answer",
            "no public control yet".to_string(),
            "UNKNOWN on invented subjects and false premises, on a frozen set",
        ),
        (
            "Reproduction",
            "no independent run recorded".to_string(),
            "a first external run of xtask verify, published as it comes",
        ),
    ];
    let mut body = String::from(
        r#"<text class="kick" x="48" y="50">OPEN QUESTIONS · GOALS, NOT RESULTS</text>"#,
    );
    body.push_str(&format!(r#"<text x="330" y="88" font-size="13" class="mut" letter-spacing="1">MEASURED NOW (PUBLIC)</text><text x="680" y="88" font-size="13" letter-spacing="1" style="fill:{}">GOAL — NOT ACHIEVED YET</text>"#, t.accent));
    for (i, (name, now, goal)) in rows.iter().enumerate() {
        let y = 126 + i * 58;
        body.push_str(&format!(r#"<path d="M48 {}H1152" stroke="{}"/><text x="48" y="{y}" font-size="17" class="b">{name}</text><text x="330" y="{y}" font-size="16">{}</text>"#, y - 26, t.line, esc(now)));
        for (j, line) in wrap(goal, 66, 2).iter().enumerate() {
            body.push_str(&format!(
                r#"<text x="680" y="{}" font-size="16" style="fill:{}">{}</text>"#,
                y + j * 20,
                t.accent,
                esc(line)
            ));
        }
    }
    body.push_str(r#"<text x="48" y="376" font-size="14" class="mut">A goal becomes a result only when it is published with its frozen protocol and evidence.</text>"#);
    result_card(dark, 400, ("Open questions and goals", "Goals, not results: more natural questions answered with no wrong answers, a short answer taken from the source, UNKNOWN when there is no correct answer, and an independent reproduction."), &body)
}
fn result_path(name: &str, dark: bool) -> String {
    format!(
        "media/presentation/{name}-{}.svg",
        if dark { "dark" } else { "light" }
    )
}
/// The result graphics in README order: file stem, light body, dark body and an alternative text read from the same data.
fn results() -> Result<Vec<(&'static str, String, String, String)>> {
    let r = Recorded::read()?;
    let (ok, wrong, unknown) = r.counts(0);
    let rb = readback()?;
    let s = statuses()?;
    let both = |f: &dyn Fn(bool) -> String| (f(false), f(true));
    let mut out = Vec::new();
    let (l, d) = both(&|dark| glance(dark, &r, &rb));
    out.push(("glance", l, d, format!("Evidence you can inspect. 80 natural questions, one recorded run: {ok} correct answers with their source passages, {wrong} wrong, {unknown} UNKNOWN. Separate experiment, not questions: {} correct answers when {} stored passages are read back.", rb.2, rb.0)));
    let (l, d) = both(&path_card);
    out.push(("question-path", l, d, "What happens to a question: question, retrieval from the whole bank, one stored source passage, an answer only when it is clear or UNKNOWN, and the evidence.".to_string()));
    let (l, d) = both(&|dark| example(dark, &r));
    out.push(("example-41", l, d, format!("Question 41: {} GEL RAM returned the source passage; the three models answered with a reason the source does not give.", r.question)));
    let (l, d) = both(&|dark| dots(dark, &r));
    out.push((
        "answer-dots",
        l,
        d,
        r.systems
            .iter()
            .enumerate()
            .map(|(i, (n, _))| {
                let (a, b, c) = r.counts(i);
                format!("{n}: {a} correct, {b} wrong, {c} UNKNOWN")
            })
            .collect::<Vec<_>>()
            .join(". "),
    ));
    let (l, d) = both(&|dark| surface(dark, &s));
    out.push((
        "truth-surface",
        l,
        d,
        format!(
            "Claims by status: {}. None is marked as independently reproduced.",
            s.iter()
                .map(|(m, n)| format!("{n} {m}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ));
    let (l, d) = both(&|dark| goals(dark, &r));
    out.push(("goals", l, d, "Open questions and goals, not results: more natural questions answered with no wrong answers, a short answer taken from the source, UNKNOWN when there is no correct answer, and an independent reproduction.".to_string()));
    Ok(out)
}
/// The README block of result pictures, from the same data as the graphics.
fn results_block() -> Result<String> {
    let mut s = String::new();
    for (i, (name, _, _, alt)) in results()?.iter().enumerate() {
        if i > 0 {
            s.push_str("\n\n");
        }
        s.push_str(&themed(
            &result_path(name, false),
            &result_path(name, true),
            &esc(alt),
            1200,
        ));
        if *name == "answer-dots" {
            s.push_str("\n\n");
            s.push_str(TRY_TO_BREAK);
        }
    }
    Ok(s)
}
const TRY_TO_BREAK: &str = "| Try to break GEL | What the public tool does | Watch it |
|:---|:---|:---|
| Change one byte of a saved copy | refuses it against the pin you kept | [Changed byte refused](media/gifs/05-integrity-light.gif) |
| Change the source after citing it | refuses the old citation | [Stale citation refused](media/gifs/02-stale-light.gif) |
| Restart and reopen the snapshot | returns the same citation under the same pin | [Verified restart](media/gifs/01-evidence-light.gif) |
| Back up, then restore to a new path | checks the restored copy equal | [Backup restored](media/gifs/03-backup-light.gif) |";
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
    ];
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
    for (name, light, dark, _) in results()? {
        out.push((result_path(name, false), light));
        out.push((result_path(name, true), dark));
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
        .replace(
            "__DOC__",
            &format!("https://github.com/Gelram-project/gel-ram/blob/{RELEASE}"),
        )
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
    NATIVE
        .replace("__FLOW__", &flow_picture())
        .replace("__FACTS__", &facts_picture())
        .replace("__WALL__", &wall_picture())
        .replace("__BARS__", &bars_picture())
        .replace("__HERO_CHIP__", &chip_picture(4))
        .replace("__HERO__", &picture(hero_id, hero_title, false))
        .replace("__HERO_ID__", hero_id)
        .replace("__GRID__", &grid)
        .replace(
            "__RESULTS__",
            &results_block().unwrap_or_else(|e| format!("__RESULTS_ERROR__ {e}")),
        )
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
        "media/gifs/06-compare.txt",
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
        assert_eq!(all.len(), 26 + 2 * CHIPS.len());
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
            ] {
                assert!(!s.contains(bad), "{bad}");
            }
        }
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
    fn results_follow_the_recorded_answers() {
        let r = Recorded::read().unwrap();
        let (ok, wrong, unknown) = r.counts(0);
        assert_eq!(ok + wrong + unknown, 80);
        let rb = readback().unwrap();
        let g = glance(false, &r, &rb);
        assert!(g.contains(&format!(">{ok} / 80<")));
        assert!(g.contains(&format!(">{wrong}<")) && g.contains(&format!(">{unknown}<")));
        assert!(g.contains("Separate experiment, not questions"));
        assert!(g.contains(&rb.2) && g.contains(&rb.0));
        let d = dots(true, &r);
        assert_eq!(d.matches("<circle").count(), 4 * 80 + 3);
        let e = example(false, &r);
        assert!(r.question.contains("Sudbury") && e.contains("House of Lords"));
        assert!(
            r.answers.iter().all(|a| a.2 == b'W'),
            "question 41 is the published example"
        );
    }
    #[test]
    fn goals_are_labelled_and_claims_are_counted() {
        let r = Recorded::read().unwrap();
        let s = statuses().unwrap();
        for dark in [false, true] {
            let g = goals(dark, &r);
            assert!(g.contains("GOALS, NOT RESULTS") && g.contains("NOT ACHIEVED YET"));
            assert!(surface(dark, &s).contains("none has been"));
        }
        let rows = fs::read_to_string("docs/CLAIMS.md")
            .unwrap()
            .lines()
            .filter(|l| l.starts_with("| ") && !l.starts_with("| ID"))
            .count();
        assert_eq!(s.iter().map(|(_, n)| n).sum::<usize>(), rows);
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
            for s in [
                scene(dark),
                header(dark),
                logo(dark, true),
                facts(dark, (1, 1, 1)),
                wall(dark, &e),
                bars(dark, &e),
            ] {
                assert!(s.contains("prefers-reduced-motion"));
            }
        }
        assert!(native_intro().contains("05-integrity"));
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
            assert!(header(dark).contains(&gel_logo(
                theme(dark),
                if dark { "#79c0ff" } else { "#0969da" },
                (120.0, 118.0),
                86.0
            )));
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
}
