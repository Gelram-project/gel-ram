//! Generate a script-free README presentation. Existing recordings are never rewritten.
#![forbid(unsafe_code)]
use std::{fs,io::Write,path::Path,process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "ea44234918dc64b8e1fadcd7eb250dc56fdfdf9d";
const BRANCH: &str = "docs/multimedia-readme-20260927";
const SCENES: &[(&str,&str,&str)] = &[
    ("01-evidence", "Exact quotes. A verifiable restart.", "Add a source, inspect its citation, save and reopen it in a separate process."),
    ("02-stale", "Changed source. Old citation refused.", "Replace the source and inspect the refusal of the previous result."),
    ("03-backup", "Backup. Inspect. Restore.", "Restore into a new path and check equality with the saved snapshot."),
    ("04-reproduce", "One command. Inspect every result.", "Read the recorded strict reproduction result and its declared scope."),
    ("05-integrity", "Changed bytes. Retained pin. Refusal.", "A changed-byte copy is rejected against the original trusted hash."),
    ("06-compare", "GEL and grep. Answers before speed.", "See matching and differing answers. No universal speedup is claimed."),
];
const SVG: &str = r####"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="228" viewBox="0 0 1200 228" role="img" aria-labelledby="title desc">
<title id="title">GEL RAM | Evidence Lab</title><desc id="desc">Your documents. Exact quotes. A restart you can check. Public Rust tools with source-linked evidence.</desc>
<defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="__BG1__"/><stop offset="1" stop-color="__BG2__"/></linearGradient></defs>
<rect x="1" y="1" width="1198" height="226" rx="24" fill="url(#bg)" stroke="__LINE__"/>
<path d="M680 0Q755 180 1200 92V0Z" fill="__ACCENT__" opacity=".06"/><path d="M560 228Q850 38 1200 156V228Z" fill="__ACCENT__" opacity=".05"/>
<g transform="translate(36 48)" fill="none" stroke="__INK__" stroke-width="5" stroke-linejoin="round">
<path d="M51 0 94 25V76L51 101 8 76V25Z"/><path d="M8 25 51 50 94 25M51 50V101"/>
<path d="M51 27 72 39V63L51 75 30 63V39Z" fill="__ACCENT__"/><path d="M30 39 51 51 72 39M51 51V75"/>
</g>
<g font-family="Arial,Helvetica,sans-serif" fill="__INK__">
<text x="162" y="86" font-size="46" font-weight="700" letter-spacing="1">GEL RAM</text>
<text x="163" y="128" font-size="30" font-weight="600">Evidence Lab</text>
<text x="42" y="192" font-size="23">Your documents. Exact quotes. A restart you can check.</text>
<text x="814" y="76" font-size="16" font-weight="700" letter-spacing="2" fill="__ACCENT__">PUBLIC RUST TOOLS</text>
<text x="814" y="108" font-size="21">Source-linked evidence.</text>
<text x="814" y="137" font-size="21">Inspect it. Reproduce it.</text>
</g></svg>
"####;
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
a{color:var(--blue);text-underline-offset:3px}a:hover{text-decoration-thickness:2px}a:focus-visible,label:focus-visible{outline:3px solid var(--blue);outline-offset:5px}img{max-width:100%;height:auto;display:block}h1,h2,h3,p{margin-top:0}h1{font-size:clamp(34px,4.3vw,57px);letter-spacing:-.05em;line-height:1.07;margin-bottom:22px}h2{font-size:29px;letter-spacing:-.025em;line-height:1.2}h3{font-size:18px;line-height:1.35}code,pre{font:13px/1.6 ui-monospace,SFMono-Regular,Consolas,monospace}pre{white-space:pre-wrap;overflow-wrap:anywhere;border:1px solid var(--line);background:var(--soft);border-radius:12px;padding:20px}code{overflow-wrap:anywhere}.page{max-width:1220px;padding:32px 32px 0;margin:auto}.brand-row{display:flex;justify-content:space-between;gap:28px;align-items:center}.brand{display:flex;align-items:center;gap:18px}.brand svg{width:66px;flex:none;color:var(--ink)}.brand-name{font-size:33px;letter-spacing:-.03em;line-height:1.12;font-weight:780}.brand-name small{display:block;font-size:23px;letter-spacing:0}.strap{margin:10px 0 0;font-size:14px;color:var(--muted)}.eyebrow{text-transform:uppercase;letter-spacing:.14em;font-size:12px;font-weight:750;color:var(--blue);margin-bottom:14px}.header-right{text-align:right;max-width:355px}.header-right p{font-size:17px;margin:0 0 10px}.chips{display:flex;gap:7px;flex-wrap:wrap;justify-content:flex-end}.chip{border:1px solid var(--line);padding:3px 10px;border-radius:6px;font-size:12px;background:var(--paper);color:var(--muted)}nav{display:flex;gap:26px;flex-wrap:wrap;padding:18px 0;margin:23px 0;border-top:1px solid var(--line);border-bottom:1px solid var(--line);font-weight:600;font-size:14px}nav a{text-decoration:none}.hero{display:grid;grid-template-columns:1.18fr 1fr;gap:36px;align-items:center;padding:30px;background:linear-gradient(135deg,var(--soft),var(--paper));border:1px solid var(--line);border-radius:20px;box-shadow:var(--shadow)}.hero>*{min-width:0}.hero figure{margin:0}.media-link{display:block;border:1px solid var(--line);border-radius:10px;overflow:hidden;background:var(--paper);text-decoration:none;box-shadow:var(--shadow)}.hero figcaption{font-size:12px;color:var(--muted);margin-top:13px}.hero-lead{font-size:17px}.checks{list-style:none;padding:0;margin:20px 0}.checks li{margin:8px 0;font-size:14px}.checks li::before{content:'✓';font-weight:800;color:var(--blue);margin-right:10px}.buttons{display:flex;flex-wrap:wrap;gap:10px}.button{display:inline-block;background:var(--blue);color:#fff;border:1px solid var(--blue);border-radius:10px;padding:10px 17px;text-decoration:none;font-weight:650;font-size:14px}.button.secondary{background:var(--paper);color:var(--blue);border-color:var(--line)}.boundary{margin:19px 0 0;color:var(--muted);font-size:12px}.section{margin:44px 0}.section-heading{text-align:center;max-width:760px;margin:0 auto 24px}.section-heading p{color:var(--muted);font-size:14px;margin-bottom:0}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:19px}.card{padding:17px;background:var(--paper);border:1px solid var(--line);border-radius:14px;box-shadow:var(--shadow)}.card .num{font-size:11px;letter-spacing:.14em;color:var(--blue);font-weight:750;margin-bottom:8px}.card h3{min-height:49px;margin-bottom:12px}.card p{color:var(--muted);font-size:13px;margin:14px 0 12px}.card .links{font-size:12px;display:flex;gap:12px;flex-wrap:wrap}.trust{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:20px;border-block:1px solid var(--line);padding:24px 0;margin-top:34px}.trust strong{display:block;margin-bottom:5px;font-size:16px}.trust p{font-size:13px;color:var(--muted);margin:0}.columns{display:grid;grid-template-columns:1.2fr 1fr 1fr;gap:30px}.columns>section{min-width:0}.columns h2{font-size:23px}.columns p,.columns li{font-size:14px;color:var(--muted)}.doclinks{padding:0;list-style:none}.doclinks li{margin:9px 0}.scope{background:var(--soft);border:1px solid var(--line);border-radius:14px;padding:22px;font-size:13px}.scope h2{font-size:20px}.scope p:last-child{margin-bottom:0}footer{display:flex;justify-content:space-between;gap:24px;flex-wrap:wrap;padding:25px 0;border-top:1px solid var(--line);font-size:12px;color:var(--muted)}footer p{margin:0}.poster{display:none}.motion-control{position:absolute;left:-9999px}.view-tools{display:flex;gap:20px;justify-content:flex-end;align-items:center;font-size:12px;margin:0 0 18px;color:var(--muted)}.view-tools label{cursor:pointer;border:1px solid var(--line);border-radius:7px;background:var(--paper);padding:6px 10px}.motion-control:focus-visible~.page label[for=still-mode]{outline:3px solid var(--blue)}.motion-control:checked~.page .motion{display:none}.motion-control:checked~.page .poster{display:block}.motion-control:checked~.page label[for=still-mode]{background:var(--soft);outline:2px solid var(--blue)}.skip{position:absolute;left:12px;top:-80px;padding:8px 12px;background:var(--paper);z-index:5}.skip:focus{top:10px}
@media(prefers-color-scheme:dark){:root{--bg:#0d1117;--paper:#151b23;--ink:#e6edf3;--muted:#a9b9ce;--line:#303d50;--blue:#79c0ff;--soft:#16253a;--shadow:0 18px 45px #0003}body{background:radial-gradient(ellipse at 95% 0%,#1a3555 0,transparent 55%),var(--bg)}.button{color:#081322}.button.secondary{color:var(--blue)}}
@media(max-width:920px){.hero{grid-template-columns:1fr;gap:24px}.hero figure{order:2}.hero h1{max-width:680px}.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.columns{grid-template-columns:1fr 1fr}.columns>section:first-child{grid-column:1/-1}.header-right{max-width:290px}.brand-name{font-size:28px}.brand-name small{font-size:20px}.page{padding:24px 22px 0}}
@media(max-width:580px){.page{padding:20px 16px 0}.brand-row{display:block}.header-right{text-align:left;max-width:none;margin-top:18px}.header-right p{font-size:15px}.chips{justify-content:flex-start}.hero{padding:19px}.hero h1{font-size:38px}.grid,.columns{grid-template-columns:1fr}.card h3{min-height:0}.trust{grid-template-columns:1fr 1fr}.view-tools{justify-content:flex-start;flex-wrap:wrap;gap:10px}nav{gap:16px;font-size:13px}.section{margin:32px 0}}
@media print{.motion{display:none!important}.poster{display:block!important}body{background:white;color:black}.view-tools{display:none}.hero,.card{box-shadow:none}}
</style>
</head>
<body>
<a class="skip" href="#content">Skip to content</a>
<input id="still-mode" class="motion-control" type="checkbox" aria-label="Show still images instead of moving previews">
<div class="page">
<header><div class="brand-row"><div><div class="brand">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 104 110" fill="none" stroke="currentColor" stroke-width="5" stroke-linejoin="round" aria-hidden="true"><path d="M52 4 96 29V80L52 106 8 80V29Z"/><path d="M8 29 52 54 96 29M52 54V106"/><path d="M52 30 74 43V68L52 81 30 68V43Z" fill="#218bff"/><path d="M30 43 52 56 74 43M52 56V81"/></svg>
<div class="brand-name">GEL RAM<small>Evidence Lab</small></div></div><p class="strap">Your documents. Exact quotes. A restart you can check.</p></div>
<div class="header-right"><p>Public tools for local work<br>and independent inspection.</p><div class="chips"><span class="chip">Rust 1.85</span><span class="chip">Local CLI</span><span class="chip">Source-linked media</span></div></div></div>
<nav aria-label="Page navigation"><a href="#content">Overview</a><a href="#quick-start">Quick start</a><a href="#workflows">Six workflows</a><a href="#reproduce">Reproduce</a><a href="#documentation">Documentation</a><a href="#scope">Scope &amp; license</a></nav></header>
<div class="view-tools"><label for="still-mode">Show still images</label><a href="__DOC__/media/gifs/MANIFEST.txt">Media provenance</a><span>12-second edited replays, not wall time</span></div>
<main id="content">
<section class="hero" aria-labelledby="hero-heading">
<figure><a class="media-link" href="media/gifs/01-evidence-light.gif" aria-label="Open the exact quotes and restart animation at full size">__HERO__</a><figcaption>Real public CLI output, edited into three readable cards.<br><a href="media/gifs/01-evidence.txt">Full transcript and source revision</a> · <a href="media/gifs/01-evidence-dark.gif">Dark full-size version</a></figcaption></figure>
<div><p class="eyebrow">Inspect the source. Keep the evidence.</p><h1 id="hero-heading">Find the passage.<br>Check the source.</h1><p class="hero-lead">Load your text, retrieve exact quotations and check that a saved collection reopens with the expected bytes.</p><ul class="checks"><li>Source text, identifiers and byte ranges</li><li>SHA-256 pins retained independently</li><li>Explicit refusal after source replacement</li><li>Public Rust tools, without an LLM</li></ul><div class="buttons"><a class="button" href="#quick-start">Run it yourself →</a><a class="button secondary" href="#workflows">Explore six workflows</a></div><p class="boundary">Source correspondence is not source truth. This is phrase retrieval, not unrestricted question answering.</p></div>
</section>
<section class="section" id="workflows" aria-labelledby="workflow-heading"><div class="section-heading"><p class="eyebrow">Six workflows. Their original evidence.</p><h2 id="workflow-heading">See it in action</h2><p>These are edited replays of recorded public command output, not product screenshots or new benchmark runs. Open each image at full size to read the terminal text.</p></div><div class="grid">__CARDS__</div>
<div class="trust"><div><strong>Inspectable</strong><p>Transcripts and retained hashes accompany the previews.</p></div><div><strong>Explicit failures</strong><p>Inspect refusal and incomplete states, not only success.</p></div><div><strong>Local operation</strong><p>The CLI uses your local documents. No automatic upload.</p></div><div><strong>Reproducible scope</strong><p>Commands, sources and limitations remain linked.</p></div></div></section>
<div class="columns section"><section id="quick-start"><p class="eyebrow">Try the public tool</p><h2>Start with your own source.</h2><p>In a new checkout, run each line separately. Setup needs a network; the demo uses locked offline dependencies.</p><pre><code>git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
git checkout __BRANCH__
git rev-parse HEAD
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo</code></pre><p>Expected marker: <code>GEL_EVIDENCE_DEMO=PASS</code>. This is a result to check, not a pre-approved outcome.</p></section>
<section id="reproduce"><p class="eyebrow">Check the declared scope</p><h2>Reproduce the checks.</h2><pre><code>cargo run --locked --offline -p xtask -- verify
cargo run --locked --offline -p xtask -- reproduce ../gel-repro-new</code></pre><p>Use a new output directory outside the checkout. Read failed, skipped and unmeasured steps. Cargo offline mode is not system-wide network isolation.</p><ul class="doclinks"><li><a href="__DOC__/docs/REPRODUCE.md">Reproduction instructions</a></li><li><a href="__DOC__/docs/CI-EVIDENCE.md">Per-revision CI evidence</a></li><li><a href="__DOC__/docs/CLAIMS.md">Executable claim registry</a></li></ul></section>
<section id="documentation"><p class="eyebrow">Go beyond the preview</p><h2>Documentation.</h2><ul class="doclinks"><li><a href="__DOC__/README.md">Technical README</a></li><li><a href="__DOC__/docs/EVIDENCE-LAB.md">Evidence Lab guide</a></li><li><a href="__DOC__/docs/EVIDENCE-BATCH.md">Batch mode and exit codes</a></li><li><a href="__DOC__/docs/BACKUP.md">Backup and restore</a></li><li><a href="__DOC__/docs/MUTATION-MATRIX.md">Format mutation matrix</a></li><li><a href="__DOC__/docs/BENCHMARK-GREP.md">Comparison with grep</a></li><li><a href="__DOC__/docs/ROADMAP.md">Public roadmap</a></li><li><a href="__DOC__/media/INDEX.md">Films and their scope</a></li></ul><p>Documentation links open the pinned source on GitHub. Viewing this local page loads only adjacent assets, with no scripts, trackers or remote fonts.</p></section></div>
<section class="scope section" id="scope"><h2>What this page does and does not establish</h2><p>This is a documentation presentation, not the application interface. The public base is <code>__BASE__</code>. The older GIFs retain the source revision declared in their <a href="media/gifs/MANIFEST.txt">original manifest</a>. Presentation changes do not retime them, certify a release or close outstanding human review.</p><p>Snapshots and backups are plaintext. A hash is not encryption, a signature or proof that a source is true. Different search tasks and matching semantics are not interchangeable benchmarks.</p><p>GEL-owned material remains under <a href="__DOC__/LICENSE">GEL RAM Noncommercial Reciprocal License 1.0</a>. Read the <a href="__DOC__/LICENSING.md">licensing guide</a>, <a href="__DOC__/THIRD-PARTY-NOTICES.md">third-party notices</a> and <a href="__DOC__/media/RIGHTS.md">media rights</a>. This page grants no additional rights.</p></section>
</main>
<footer><p><strong>GEL RAM · Evidence Lab</strong><br>RR, GEL RAM Project · Public documentation</p><p><a href="__DOC__/LICENSE">License</a> · <a href="__DOC__/SECURITY.md">Security</a> · <a href="__DOC__/CLA-PRIVACY.md">CLA privacy</a> · <a href="__DOC__/CONTRIBUTING.md">Contributing</a><br>Existing evidence preserved. New presentation only.</p></footer>
</div></body></html>
"####;
const NATIVE: &str = r####"# GEL RAM

<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/presentation/header-dark.svg">
  <img alt="GEL RAM Evidence Lab. Your documents. Exact quotes. A restart you can check." src="media/presentation/header-light.svg" width="1200" height="228">
</picture>

**Find the passage. Check the source.** Local Rust tools for exact source-bound
quotations, stale-citation refusal and independently pinned snapshots.

[Quick start](#quick-start) · [Six workflows](#see-it-in-action) · [Reproduce](#reproduce-the-checks) · [Documentation](#documentation) · [License](#about-and-licensing)

> **Multimedia presentation candidate.** This branch changes presentation, not the
> public runtime or license. The workspace reports 0.5.0; this page does not publish
> a release. Use the branch in the commands below and record its exact commit.
> [Publication status](CANDIDATE-STATUS.md) · [Tagged releases](https://github.com/Gelram-project/gel-ram/releases)

**Full-page edition:** [README-MULTIMEDIA.html](README-MULTIMEDIA.html).
Open that file from this checkout in a browser for the responsive blue-panel layout,
light/dark backgrounds and a still-image control. GitHub displays HTML files as
source, not as a hosted page; this PR does not enable Pages.
[Presentation guide](docs/README-PRESENTATION.md).

## See it in action

**Actual public-tool runs, presented as six edited log replays.**
Each animation lasts 12 seconds; pacing is editorial, not execution time.
[Static view](media/gifs/STATIC.md) · [Full gallery](media/gifs/README.md) · [Original source and hashes](media/gifs/MANIFEST.txt)

__HERO__

[Full source/restart transcript](media/gifs/01-evidence.txt) · [Full-size dark replay](media/gifs/01-evidence-dark.gif)

### Six workflows, one evidence trail

Open a preview at full size to read the terminal text. All six existing scenarios
are visible here rather than hidden in collapsed sections. Their original
transcripts and source revisions remain unchanged.

__GRID__

The earlier [70-second Evidence Lab film](media/GEL-EVIDENCE-LAB-EN.mp4),
its [original process logs](media/EVIDENCE-LAB-GUIDE.md) and
[open human review](docs/MEDIA-DECODE-REVIEW.md) remain separate historical material.
New presentation is not a new execution, benchmark or human acceptance.

"####;
const GUIDE: &str = r####"# Multimedia README presentation

This presentation has two views. The root README is normal GitHub Markdown with
an SVG banner and six real GIF previews. The adjacent README-MULTIMEDIA.html is a
script-free local document with the full responsive panel layout, theme-aware
backgrounds, a still-image control and links to the public documentation.
Open it from the checkout in a browser. GitHub's file viewer does not host it as
a web page; no Pages deployment or new public service is configured.

## Preservation and source

The parent public source is __BASE__. The exact prior README is retained as
[README-HISTORY-PRE-MULTIMEDIA.md](../README-HISTORY-PRE-MULTIMEDIA.md).
The already existing pre-visual history also remains untouched. The new README
keeps its technical content below the presentation, changing only the quick-start
checkout from an unverified tag to this named presentation branch.

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
adds only three exact presentation-asset pins. It does not allow arbitrary SVG or
HTML files. The source manifest is then refreshed and the complete candidate must
pass xtask verify. Normal three-platform CI is read separately for the final head.

The one-off CI builder is restricted to __BRANCH__ and the owner account.
It refuses concurrent branch advancement and never force-pushes, merges or
publishes a release. A rerun validates already-produced files instead of replacing
historic recordings. Its public source archive includes licenses and notices, not
machine-local logs or fonts. The final PR records actual visual and CI results;
this guide does not predeclare PASS.

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
    format!(r#"<picture><source media="(prefers-reduced-motion: reduce) and (prefers-color-scheme: dark)" srcset="media/gifs/{id}-dark.png"><source media="(prefers-reduced-motion: reduce)" srcset="media/gifs/{id}-light.png"><source media="(prefers-color-scheme: dark)" srcset="media/gifs/{id}-dark.{ext}"><img alt="{title}" src="media/gifs/{id}-light.{ext}" width="1000" height="460" loading="lazy"></picture>"#)
}
fn pair(id: &str, title: &str) -> String {
    format!("<div class=\"motion\">{}</div><div class=\"poster\">{}</div>", picture(id,title,false),picture(id,title,true))
}
fn header(dark: bool) -> String {
    let values = if dark { ["#101d31","#0d1117","#303d50","#79c0ff","#e6edf3"] } else { ["#f7fbff","#deedff","#cbdcf0","#0969da","#102949"] };
    let mut s = SVG.to_string();
    for (key,val) in ["BG1","BG2","LINE","ACCENT","INK"].into_iter().zip(values) { s=s.replace(&format!("__{key}__"),val); }
    s
}
fn page() -> String {
    let mut cards=String::new();
    for (i,(id,title,desc)) in SCENES.iter().enumerate() {
        if i>0 { cards.push('\n'); }
        cards.push_str(&format!(r#"<article class="card"><div class="num">WORKFLOW 0{}</div><h3>{title}</h3><a class="media-link" href="media/gifs/{id}-light.gif" aria-label="Open {title} at full size">{}</a><p>{desc}</p><div class="links"><a href="media/gifs/{id}.txt">Full transcript</a><a href="media/gifs/{id}-dark.gif">Dark full size</a></div></article>"#,i+1,pair(id,title)));
    }
    HTML.replace("__HERO__",&pair(SCENES[0].0,SCENES[0].1)).replace("__CARDS__",&cards).replace("__DOC__",&format!("https://github.com/Gelram-project/gel-ram/blob/{BASE}")).replace("__BASE__",BASE).replace("__BRANCH__",BRANCH)
}
fn native_intro() -> String {
    let mut grid=String::from("<table>\n");
    for row in SCENES.chunks(2) {
        grid.push_str("<tr>\n");
        for (id,title,desc) in row {
            grid.push_str(&format!("<td width=\"50%\" valign=\"top\"><h4>{title}</h4><a href=\"media/gifs/{id}-light.gif\">{}</a><p>{desc}</p><p><a href=\"media/gifs/{id}.txt\">Transcript</a> · <a href=\"media/gifs/{id}-dark.gif\">Dark full size</a></p></td>\n",picture(id,title,false)));
        }
        grid.push_str("</tr>\n");
    }
    grid.push_str("</table>\n");
    NATIVE.replace("__HERO__",&picture(SCENES[0].0,SCENES[0].1,false)).replace("__GRID__",&grid)
}
fn rewrite_readme(before: &str) -> Result<String> {
    let marker="### What you can inspect\n";
    if !before.starts_with("# GEL RAM\n\n### Evidence Lab\n") || before.matches(marker).count()!=1 {
        return Err("unexpected README structure; no guessed replacement".into());
    }
    let tail=before.split_once(marker).ok_or("missing content boundary")?.1;
    let tail=tail.replace("git checkout v0.5.0",&format!("git checkout {BRANCH}"));
    Ok(format!("{}{marker}{tail}",native_intro()))
}
fn write_new(name: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent)=Path::new(name).parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
    let mut f=fs::OpenOptions::new().write(true).create_new(true).open(name)?;
    f.write_all(bytes)?; f.sync_all()?; Ok(())
}
fn hash(name: &str) -> Result<String> {
    let out=Command::new("sha256sum").args(["--",name]).output()?;
    if !out.status.success() { return Err(format!("cannot hash {name}").into()); }
    let s=String::from_utf8(out.stdout)?;
    let h=s.split_whitespace().next().ok_or("missing hash")?;
    if h.len()!=64 || !h.bytes().all(|b|b.is_ascii_hexdigit()) { return Err("invalid hash".into()); }
    Ok(h.to_string())
}
fn check_media() -> Result<Vec<(String,String)>> {
    let text=fs::read_to_string("media/gifs/MANIFEST.txt")?;
    let mut checked=Vec::new();
    for line in text.lines().filter(|l|l.starts_with("sha256 ")) {
        let words: Vec<_>=line.split_whitespace().collect();
        if words.len()!=3 || !words[2].starts_with("media/gifs/") || words[2].contains("..") { return Err("unexpected original media pin".into()); }
        if hash(words[2])?!=words[1] { return Err(format!("changed original media: {}",words[2]).into()); }
        checked.push((words[2].to_string(),words[1].to_string()));
    }
    if checked.len()!=24 { return Err("expected 24 original media assets".into()); }
    Ok(checked)
}
fn check_page(text: &str) -> Result<()> {
    let low=text.to_ascii_lowercase();
    for bad in ["<script","<iframe","<object","<embed","<form","javascript:","onerror=","onload=","@import","url(","src=\"http","srcset=\"http","__hero__","__cards__","__doc__","__base__","__branch__"] {
        if low.contains(bad) { return Err(format!("unexpected active/remote/unresolved markup: {bad}").into()); }
    }
    for (id,_,_) in SCENES {
        for ending in ["-dark.gif","-light.gif","-dark.png","-light.png",".txt"] {
            if !text.contains(&format!("media/gifs/{id}{ending}")) { return Err(format!("missing {id}{ending}").into()); }
        }
    }
    Ok(())
}
fn run() -> Result<()> {
    if std::env::args().len()!=1 { return Err("run the builder without arguments from the repository root".into()); }
    let media=check_media()?;
    let immutable=["LICENSE","NOTICE","CLA.md","CLA-PRIVACY.md","LICENSING.md","media/RIGHTS.md","media/gifs/MANIFEST.txt","media/gifs/01-evidence.txt","media/gifs/02-stale.txt","media/gifs/03-backup.txt","media/gifs/04-reproduce.txt","media/gifs/05-integrity.txt","media/gifs/06-compare.txt"];
    let saved: Vec<_>=immutable.iter().map(|s| Ok((*s,hash(s)?))).collect::<Result<_>>()?;
    let before=fs::read_to_string("README.md")?;
    if before.contains("<!-- GEL_MULTIMEDIA_PRESENTATION_V1 -->") {
        if fs::read_to_string("README-MULTIMEDIA.html")?!=page() || fs::read_to_string("media/presentation/header-light.svg")?!=header(false) || fs::read_to_string("media/presentation/header-dark.svg")?!=header(true) {
            return Err("existing presentation differs; refuse to overwrite it".into());
        }
        check_page(&page())?;
        println!("README_PRESENTATION=UNCHANGED original media pins verified");
        return Ok(());
    }
    let new= rewrite_readme(&before)?;
    let html=page(); check_page(&html)?;
    let paths=["README-MULTIMEDIA.html","media/presentation/header-light.svg","media/presentation/header-dark.svg"];
    let bodies=[html,header(false),header(true)];
    for name in paths.iter().copied().chain(["README-HISTORY-PRE-MULTIMEDIA.md","docs/README-PRESENTATION.md"]) {
        if Path::new(name).try_exists()? { return Err(format!("new path already exists: {name}").into()); }
    }
    let main=fs::read_to_string("xtask/src/main.rs")?;
    let mark="const REVIEWED_ASSETS: &[(&str, &str)] = &[\n";
    if main.matches(mark).count()!=1 { return Err("asset pin insertion point is not unique".into()); }
    write_new("README-HISTORY-PRE-MULTIMEDIA.md",before.as_bytes())?;
    for (name,body) in paths.iter().zip(bodies.iter()) { write_new(name,body.as_bytes())?; }
    let mut pins=mark.to_string();
    pins.push_str("    // Exact script-free documentation presentation assets; no HTML/SVG wildcard.\n");
    for name in paths { pins.push_str(&format!("    (\n        \"{name}\",\n        \"{}\",\n    ),\n",hash(name)?)); }
    fs::write("xtask/src/main.rs",main.replacen(mark,&pins,1))?;
    fs::write("README.md",new)?;
    write_new("docs/README-PRESENTATION.md",GUIDE.replace("__BASE__",BASE).replace("__BRANCH__",BRANCH).as_bytes())?;
    let index=fs::read_to_string("media/INDEX.md")?;
    fs::write("media/INDEX.md",format!("[Multimedia README and local full-page edition](../docs/README-PRESENTATION.md). Existing recordings below retain their original scope.\n\n{index}"))?;
    if fs::read("README-HISTORY-PRE-MULTIMEDIA.md")?!=before.as_bytes() { return Err("README archive differs".into()); }
    if check_media()?!=media { return Err("media inventory changed".into()); }
    for (name,pin) in saved { if hash(name)?!=pin { return Err(format!("protected file changed: {name}").into()); } }
    println!("README_PRESENTATION=BUILT six existing previews, exact archive, unchanged licenses and recordings");
    Ok(())
}
fn main() { if let Err(e)=run() { eprintln!("README_PRESENTATION=FAIL {e}"); std::process::exit(1); } }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn six_real_scenarios(){assert_eq!(SCENES.len(),6);let p=page();for(id,_,_)in SCENES{assert!(p.contains(&format!("media/gifs/{id}.txt")));}}
    #[test] fn no_active_or_remote_assets(){check_page(&page()).unwrap();for s in ["<script>x</script>","<img src=\"https://remote.test/a.png\">","__HERO__"]{assert!(check_page(&(page()+s)).is_err());}}
    #[test] fn themes_have_no_placeholders(){for b in [false,true]{let s=header(b);assert!(!s.contains("__"));assert!(!s.contains("<script"));assert!(s.contains("<title"));}}
    #[test] fn keeps_technical_tail(){let s="# GEL RAM\n\n### Evidence Lab\nold\n### What you can inspect\nkeep these bytes\n";assert!(rewrite_readme(s).unwrap().ends_with("### What you can inspect\nkeep these bytes\n"));}
    #[test] fn unknown_readme_refused(){assert!(rewrite_readme("unrelated").is_err());}
    #[test] fn no_false_status_badges(){for bad in ["CI passing","No hallucinations","platforms 3/3"]{assert!(!page().contains(bad));assert!(!native_intro().contains(bad));}}
    #[test] fn movement_has_alternatives(){assert!(page().contains("prefers-reduced-motion"));assert!(page().contains("id=\"still-mode\""));assert_eq!(page().matches("<article class=\"card\"").count(),6);}
    #[test] fn native_is_not_one_big_image(){let s=native_intro();assert!(s.contains("<table>"));assert!(s.contains("README-MULTIMEDIA.html"));assert_eq!(s.matches("<td width=\"50%\"").count(),6);}
}
