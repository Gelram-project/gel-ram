//! Generate a script-free README presentation. Existing recordings are never rewritten.
#![forbid(unsafe_code)]
use std::{fs, io::Write, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// The release whose sources the page links to and checks out.
const RELEASE: &str = "v0.5.0";
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
/// Animated diagram of the public citation check. It illustrates the flow of
/// the recorded workflows; it is not a recording or a timing measurement.
const FLOW: &str = r####"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="330" viewBox="0 0 1200 330" role="img" aria-labelledby="t d">
<title id="t">How a GEL citation is checked</title>
<desc id="d">Animated diagram: your document, an exact quote with its byte range, a SHA-256 pin you keep, a new process that reopens the snapshot with the pin and passes, and a copy with one changed byte that is refused.</desc>
<style>
text{font-family:Arial,Helvetica,sans-serif;fill:__INK__}
.mono{font-family:ui-monospace,Consolas,"DejaVu Sans Mono",monospace;font-size:13px;fill:__MUTED__}
.title{font-size:18px;font-weight:700}
.box{fill:__PAPER__;stroke:__LINE__;stroke-width:2}
.ico{fill:none;stroke:__INK__;stroke-width:3;stroke-linejoin:round;stroke-linecap:round}
.acc{fill:__ACCENT__}
.wire{fill:none;stroke:__ACCENT__;stroke-width:4;stroke-linecap:round;stroke-dasharray:130;stroke-dashoffset:130}
.n1,.n2,.n3,.n4,.n5,.ok,.bad{opacity:0}
.n1{animation:n1 12s infinite}.n2{animation:n2 12s infinite}.n3{animation:n3 12s infinite}.n4{animation:n4 12s infinite}.n5{animation:n5 12s infinite}
.w1{animation:w1 12s infinite}.w2{animation:w2 12s infinite}.w3{animation:w3 12s infinite}.w4{animation:w4 12s infinite}
.ok{animation:ok 12s infinite}.bad{animation:bad 12s infinite}
@keyframes n1{0%{opacity:0;transform:translateY(10px)}6%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes n2{0%,14%{opacity:0;transform:translateY(10px)}20%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes n3{0%,28%{opacity:0;transform:translateY(10px)}34%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes n4{0%,42%{opacity:0;transform:translateY(10px)}48%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes n5{0%,56%{opacity:0;transform:translateY(10px)}62%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes w1{0%,8%{stroke-dashoffset:130}16%,94%{stroke-dashoffset:0}100%{stroke-dashoffset:130}}
@keyframes w2{0%,22%{stroke-dashoffset:130}30%,94%{stroke-dashoffset:0}100%{stroke-dashoffset:130}}
@keyframes w3{0%,36%{stroke-dashoffset:130}44%,94%{stroke-dashoffset:0}100%{stroke-dashoffset:130}}
@keyframes w4{0%,50%{stroke-dashoffset:130}58%,94%{stroke-dashoffset:0}100%{stroke-dashoffset:130}}
@keyframes ok{0%,63%{opacity:0;transform:scale(.8)}67%,94%{opacity:1;transform:none}100%{opacity:0}}
@keyframes bad{0%,72%{opacity:0}75%{opacity:1;transform:translateX(0)}76.5%{transform:translateX(-5px)}78%{transform:translateX(5px)}79.5%{transform:translateX(-3px)}81%,94%{opacity:1;transform:none}100%{opacity:0}}
.n1,.n2,.n3,.n4,.n5,.ok,.bad{transform-box:fill-box;transform-origin:center}
@media (prefers-reduced-motion:reduce){*{animation:none!important}.n1,.n2,.n3,.n4,.n5,.ok,.bad{opacity:1}.wire{stroke-dashoffset:0}}
</style>
<defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="__BG1__"/><stop offset="1" stop-color="__BG2__"/></linearGradient></defs>
<rect x="1" y="1" width="1198" height="328" rx="24" fill="url(#bg)" stroke="__LINE__"/>
<text x="40" y="44" font-size="13" font-weight="700" letter-spacing="2" style="fill:__ACCENT__">HOW A CITATION IS CHECKED</text>
<path class="wire w1" d="M215 150H285"/><path class="wire w2" d="M455 150H525"/><path class="wire w3" d="M695 150H765"/><path class="wire w4" d="M935 150H995"/>
<g class="n1"><rect class="box" x="40" y="80" width="175" height="140" rx="16"/>
<path class="ico" d="M104 96h34l14 14v44h-48z M138 96v14h14 M114 124h26 M114 134h26 M114 144h18"/>
<text class="title" x="127" y="186" text-anchor="middle">Your document</text><text class="mono" x="127" y="207" text-anchor="middle">UTF-8, stays local</text></g>
<g class="n2"><rect class="box" x="285" y="80" width="170" height="140" rx="16"/>
<path class="ico" d="M346 96h34l14 14v44h-48z M380 96v14h14"/><rect class="acc" x="352" y="124" width="36" height="10" rx="3"/><path class="ico" d="M356 142h26"/>
<text class="title" x="370" y="186" text-anchor="middle">Exact quote</text><text class="mono" x="370" y="207" text-anchor="middle">bytes 67..105</text></g>
<g class="n3"><rect class="box" x="525" y="80" width="170" height="140" rx="16"/>
<rect class="ico" x="590" y="118" width="40" height="34" rx="6"/><path class="ico" d="M598 118v-8a12 12 0 0 1 24 0v8"/><circle class="acc" cx="610" cy="135" r="5"/>
<text class="title" x="610" y="186" text-anchor="middle">SHA-256 pin</text><text class="mono" x="610" y="207" text-anchor="middle">you keep it</text></g>
<g class="n4"><rect class="box" x="765" y="80" width="170" height="140" rx="16"/>
<path class="ico" d="M870 125a20 20 0 1 1-6-14 M864 100v11h11"/>
<text class="title" x="850" y="186" text-anchor="middle">New process</text><text class="mono" x="850" y="207" text-anchor="middle">load with the pin</text></g>
<g class="n5"><rect class="box" x="995" y="80" width="170" height="140" rx="16"/>
<text class="title" x="1080" y="106" text-anchor="middle">Reopen</text>
<g class="ok"><rect x="1015" y="116" width="130" height="32" rx="16" fill="__GREEN__"/><text x="1080" y="137" text-anchor="middle" font-size="14" font-weight="700" style="fill:#ffffff">✓ PASS</text></g>
<g class="bad"><rect x="1015" y="156" width="130" height="32" rx="16" fill="__RED__"/><text x="1080" y="177" text-anchor="middle" font-size="14" font-weight="700" style="fill:#ffffff">✗ REFUSED</text>
<text class="mono" x="1080" y="207" text-anchor="middle" style="font-size:11.5px">copy, 1 changed byte</text></g></g>
<text x="40" y="262" font-size="17">A citation matches the bytes you pinned, or it is refused.</text>
<text class="mono" x="40" y="292">Illustration of the public Evidence Lab flow · not a timing measurement · see the six recorded runs below</text>
</svg>
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
a{color:var(--blue);text-underline-offset:3px}a:hover{text-decoration-thickness:2px}a:focus-visible,label:focus-visible{outline:3px solid var(--blue);outline-offset:5px}img{max-width:100%;height:auto;display:block}h1,h2,h3,p{margin-top:0}h1{font-size:clamp(34px,4.3vw,57px);letter-spacing:-.05em;line-height:1.07;margin-bottom:22px}h2{font-size:29px;letter-spacing:-.025em;line-height:1.2}h3{font-size:18px;line-height:1.35}code,pre{font:13px/1.6 ui-monospace,SFMono-Regular,Consolas,monospace}pre{white-space:pre-wrap;overflow-wrap:anywhere;border:1px solid var(--line);background:var(--soft);border-radius:12px;padding:20px}code{overflow-wrap:anywhere}.page{max-width:1220px;padding:32px 32px 0;margin:auto}.brand-row{display:flex;justify-content:space-between;gap:28px;align-items:center}.brand{display:flex;align-items:center;gap:18px}.brand svg{width:66px;flex:none;color:var(--ink)}.brand-name{font-size:33px;letter-spacing:-.03em;line-height:1.12;font-weight:780}.brand-name small{display:block;font-size:23px;letter-spacing:0}.strap{margin:10px 0 0;font-size:14px;color:var(--muted)}.eyebrow{text-transform:uppercase;letter-spacing:.14em;font-size:12px;font-weight:750;color:var(--blue);margin-bottom:14px}.header-right{text-align:right;max-width:355px}.header-right p{font-size:17px;margin:0 0 10px}.chips{display:flex;gap:7px;flex-wrap:wrap;justify-content:flex-end}.chip{border:1px solid var(--line);padding:3px 10px;border-radius:6px;font-size:12px;background:var(--paper);color:var(--muted)}nav{display:flex;gap:26px;flex-wrap:wrap;padding:18px 0;margin:23px 0;border-top:1px solid var(--line);border-bottom:1px solid var(--line);font-weight:600;font-size:14px}nav a{text-decoration:none}.hero{display:grid;grid-template-columns:1.18fr 1fr;gap:36px;align-items:center;padding:30px;background:linear-gradient(135deg,var(--soft),var(--paper));border:1px solid var(--line);border-radius:20px;box-shadow:var(--shadow)}.hero>*{min-width:0}.hero figure{margin:0}.media-link{display:block;border:1px solid var(--line);border-radius:10px;overflow:hidden;background:var(--paper);text-decoration:none;box-shadow:var(--shadow)}.hero figcaption{font-size:12px;color:var(--muted);margin-top:13px}.hero-lead{font-size:17px}.checks{list-style:none;padding:0;margin:20px 0}.checks li{margin:8px 0;font-size:14px}.checks li::before{content:'✓';font-weight:800;color:var(--blue);margin-right:10px}.buttons{display:flex;flex-wrap:wrap;gap:10px}.button{display:inline-block;background:var(--blue);color:#fff;border:1px solid var(--blue);border-radius:10px;padding:10px 17px;text-decoration:none;font-weight:650;font-size:14px}.button.secondary{background:var(--paper);color:var(--blue);border-color:var(--line)}.boundary{margin:19px 0 0;color:var(--muted);font-size:12px}.section{margin:44px 0}.section-heading{text-align:center;max-width:760px;margin:0 auto 24px}.section-heading p{color:var(--muted);font-size:14px;margin-bottom:0}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:19px}.card{padding:17px;background:var(--paper);border:1px solid var(--line);border-radius:14px;box-shadow:var(--shadow)}.card .num{font-size:11px;letter-spacing:.14em;color:var(--blue);font-weight:750;margin-bottom:8px}.card h3{min-height:49px;margin-bottom:12px}.card p{color:var(--muted);font-size:13px;margin:14px 0 12px}.card .links{font-size:12px;display:flex;gap:12px;flex-wrap:wrap}.trust{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:20px;border-block:1px solid var(--line);padding:24px 0;margin-top:34px}.trust strong{display:block;margin-bottom:5px;font-size:16px}.trust p{font-size:13px;color:var(--muted);margin:0}.columns{display:grid;grid-template-columns:1.2fr 1fr 1fr;gap:30px}.columns>section{min-width:0}.columns h2{font-size:23px}.columns p,.columns li{font-size:14px;color:var(--muted)}.doclinks{padding:0;list-style:none}.doclinks li{margin:9px 0}.scope{background:var(--soft);border:1px solid var(--line);border-radius:14px;padding:22px;font-size:13px}.scope h2{font-size:20px}.scope p:last-child{margin-bottom:0}footer{display:flex;justify-content:space-between;gap:24px;flex-wrap:wrap;padding:25px 0;border-top:1px solid var(--line);font-size:12px;color:var(--muted)}footer p{margin:0}.poster{display:none}.motion-control{position:absolute;left:-9999px}.view-tools{display:flex;gap:20px;justify-content:flex-end;align-items:center;font-size:12px;margin:0 0 18px;color:var(--muted)}.view-tools label{cursor:pointer;border:1px solid var(--line);border-radius:7px;background:var(--paper);padding:6px 10px}.motion-control:focus-visible~.page label[for=still-mode]{outline:3px solid var(--blue)}.motion-control:checked~.page .motion{display:none}.motion-control:checked~.page .poster{display:block}.motion-control:checked~.page label[for=still-mode]{background:var(--soft);outline:2px solid var(--blue)}.skip{position:absolute;left:12px;top:-80px;padding:8px 12px;background:var(--paper);z-index:5}.skip:focus{top:10px}
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
<section class="section flow" aria-label="How a citation is checked">__FLOW__<div class="facts">__FACTS__</div></section>
<section class="section" id="workflows" aria-labelledby="workflow-heading"><div class="section-heading"><p class="eyebrow">Six workflows. Their original evidence.</p><h2 id="workflow-heading">See it in action</h2><p>These are edited replays of recorded public command output, not product screenshots or new benchmark runs. Open each image at full size to read the terminal text.</p></div><div class="grid">__CARDS__</div>
<div class="trust"><div><strong>Inspectable</strong><p>Transcripts and retained hashes accompany the previews.</p></div><div><strong>Explicit failures</strong><p>Inspect refusal and incomplete states, not only success.</p></div><div><strong>Local operation</strong><p>The CLI uses your local documents. No automatic upload.</p></div><div><strong>Reproducible scope</strong><p>Commands, sources and limitations remain linked.</p></div></div></section>
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
  <img alt="GEL RAM Evidence Lab. Your documents. Exact quotes. A restart you can check." src="media/presentation/header-light.svg" width="1200">
</picture>

__FLOW__

**Find the passage. Check the source.** Local Rust tools for exact source-bound
quotations, stale-citation refusal and independently pinned snapshots.

__FACTS__

[Quick start](#quick-start) · [Six workflows](#see-it-in-action) · [Reproduce](#reproduce-the-checks) · [Documentation](#documentation) · [License](#about-and-licensing)

> **Version 0.5.0.** The instructions below use the `v0.5.0` tag. Record the exact
> commit you test.
> [Release notes](RELEASE-NOTES-v0.5.0.md) · [Publication status](CANDIDATE-STATUS.md) · [Tagged releases](https://github.com/Gelram-project/gel-ram/releases)

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
CPU adapter, tested offline in a Linux sandbox, not a deployed API hybrid.

Percentiles use nearest-rank; with N=40, p99 is the maximum. These are
single-host diagnostic runs, not independently replicated measurements.
Matching source bytes does not establish truth or understanding.
Neither physical DRAM-refresh computation nor a speedup over LLMs is
established by these tests.

The underlying logs and private harness remain outside this checkout.
**These rows are not independently reproducible from the public release.**
Public-tool demonstrations and their reproducible evidence above retain
their own, separate scope. No private version identifiers, source code,
knowledge banks or credentials are included here.

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
The adapter is not the full GEL chat or an Ocean retrieval benchmark.

Earlier 20B and 120B PL attempts were incomplete at the 1024-token limit
(1383.914 ms and 2407.528 ms). They are retained in the
[protocol, exact times and failure notes](docs/GEL-GROQ-DIAGNOSTIC.md).
This small, development-exposed comparison is not independently validated
or a general model ranking. Private code, banks and API credentials stay private.

"####;
const GUIDE: &str = r####"# Multimedia README presentation

This presentation has two views. The root README is normal GitHub Markdown with
an SVG banner, an animated SVG diagram of the citation check, a strip of checked
facts, colour-coded workflow badges and six real GIF previews. The adjacent
README-MULTIMEDIA.html is a script-free local document with the full responsive
panel layout, theme-aware backgrounds, a still-image control and links to the
public documentation. Open it from the checkout in a browser. GitHub's file
viewer does not host it as a web page; no Pages deployment or new public service
is configured.

The animated diagram illustrates the flow the six recordings show: an exact
quote with its byte range, a SHA-256 pin kept by the user, a new process that
reopens the snapshot, and a copy with one changed byte that is refused. It is
drawn, not recorded, and its timing is not a measurement. Under reduced motion
it shows its final state. The numbers in the facts strip are read by the
builder from the property map and the recorded mutation matrix, so a change to
either makes the read-only check fail until the strip is rebuilt.

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
adds only three exact presentation-asset pins. It does not allow arbitrary SVG or
HTML files. The source manifest is then refreshed and the complete tree must
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
    s
}
fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = template.to_string();
    for (key, val) in pairs {
        s = s.replace(&format!("__{key}__"), val);
    }
    s
}
fn flow(dark: bool) -> String {
    let v = if dark {
        [
            "#101d31", "#0d1117", "#30465f", "#58a6ff", "#e6edf3", "#9fb3c8", "#151b23", "#238636",
            "#da3633",
        ]
    } else {
        [
            "#f7fbff", "#deedff", "#cbdcf0", "#0969da", "#102949", "#486380", "#ffffff", "#1a7f37",
            "#cf222e",
        ]
    };
    let keys = [
        "BG1", "BG2", "LINE", "ACCENT", "INK", "MUTED", "PAPER", "GREEN", "RED",
    ];
    let pairs: Vec<(&str, &str)> = keys.into_iter().zip(v).collect();
    fill(FLOW, &pairs)
}
/// Counts shown in the facts strip, read from the repository so they cannot
/// drift: property rows and their tests in the property map, and distinct
/// (format, case) mutants in the recorded mutation matrix.
fn fact_counts() -> Result<(usize, usize, usize)> {
    let map = fs::read_to_string("docs/PROPERTY-TESTS.md")?;
    let rows: Vec<&str> = map.lines().filter(|l| l.starts_with("| P")).collect();
    let tests: usize = rows
        .iter()
        .map(|r| r.rsplit('|').nth(1).unwrap_or("").matches('`').count() / 2)
        .sum();
    let matrix = fs::read_to_string("docs/evidence-mutation/matrix-r1.txt")?;
    let mutants: std::collections::BTreeSet<(&str, &str)> = matrix
        .lines()
        .skip(1)
        .filter_map(|l| {
            let mut f = l.split('\t');
            Some((f.next()?, f.next()?))
        })
        .collect();
    if rows.is_empty() || tests == 0 || mutants.is_empty() {
        return Err("missing fact sources".into());
    }
    Ok((rows.len(), tests, mutants.len()))
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
        "How a citation is checked: your document, an exact quote with its byte range, a SHA-256 pin you keep, a new process that reopens the snapshot with the pin; a copy with one changed byte is refused.",
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
/// Every generated SVG asset with its body.
fn assets() -> Result<Vec<(String, String)>> {
    let counts = fact_counts()?;
    let mut out = vec![
        (
            "media/presentation/header-light.svg".to_string(),
            header(false),
        ),
        (
            "media/presentation/header-dark.svg".to_string(),
            header(true),
        ),
        ("media/presentation/flow-light.svg".to_string(), flow(false)),
        ("media/presentation/flow-dark.svg".to_string(), flow(true)),
        (
            "media/presentation/facts-light.svg".to_string(),
            facts(false, counts),
        ),
        (
            "media/presentation/facts-dark.svg".to_string(),
            facts(true, counts),
        ),
    ];
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
        .replace("__CARDS__", &cards)
        .replace("__FLOW__", &flow_picture())
        .replace("__FACTS__", &facts_picture())
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
        .replace("__HERO_CHIP__", &chip_picture(4))
        .replace("__HERO__", &picture(hero_id, hero_title, false))
        .replace("__HERO_ID__", hero_id)
        .replace("__GRID__", &grid)
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
    let mut needed = vec![
        "media/presentation/flow-light.svg".to_string(),
        "media/presentation/flow-dark.svg".to_string(),
        "media/presentation/facts-light.svg".to_string(),
        "media/presentation/facts-dark.svg".to_string(),
    ];
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
        assert_eq!(all.len(), 6 + 2 * CHIPS.len());
        for (name, s) in all {
            assert!(!s.contains("__"), "{name}");
            assert!(!s.contains("<script"), "{name}");
            assert!(s.contains("<title") || s.contains("aria-label="), "{name}");
        }
        for s in [native_intro(), page()] {
            for bad in [
                "__FLOW__",
                "__FACTS__",
                "__HERO_CHIP__",
                "__HERO_ID__",
                "__HERO__",
                "__GRID__",
            ] {
                assert!(!s.contains(bad), "{bad}");
            }
        }
    }
    #[test]
    fn facts_are_read_from_the_repository() {
        let (props, tests, mutants) = fact_counts().unwrap();
        assert!(props > 0 && tests >= props && mutants > 0);
        let s = facts(false, (props, tests, mutants));
        assert!(s.contains(&format!("{props} → {tests}")));
        assert!(s.contains(&format!(">{mutants}<")));
    }
    #[test]
    fn motion_can_be_reduced() {
        for dark in [false, true] {
            assert!(flow(dark).contains("prefers-reduced-motion"));
            assert!(facts(dark, (1, 1, 1)).contains("prefers-reduced-motion"));
        }
        assert!(native_intro().contains("05-integrity"));
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
