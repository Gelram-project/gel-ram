# Multimedia README presentation

This presentation has two views. The root README is normal GitHub Markdown with
an SVG banner with the GEL logo as a turning 3D glass cube (a still version when
reduced motion is requested), result panels drawn from the recorded answers of
frozen question sets v4 and v1 and from the claim registry, each panel naming its set, an
animated 3D scene of the citation check, a strip of checked facts, colour-coded
workflow badges, six real GIF previews and two 3D graphics of the public checks.
The adjacent
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

The presentation belongs to the v0.5.2 source. Earlier README versions are
no longer kept in the tree; they remain in the git history. The README keeps its
technical content below the presentation unchanged, including the quick-start
checkout of the v0.5.2 tag.

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
