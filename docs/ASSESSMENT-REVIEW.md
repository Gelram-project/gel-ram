# Post-freeze document assessment

Runtime was frozen at `676e751718881705a1538fb84a6ba2dd7f103677` before these
cases were written. No collection/search implementation was changed to fit them.
The first run passed **24/24 defined contract cases**: 20 over three actual
documents (Rust Book excerpt and two public project guides), plus 4 separately
labelled synthetic Unicode cases. Original cases and first outputs retained.

```sh
cargo run --locked --offline --release -p gel-source --example collection_review
```

Every shown quote is checked against its original byte range and document after
in-memory snapshot decode. Modified data is rejected against the original pin.
Each case emits query, expected/actual IDs, status and search time. The evaluator
does not hide failed cases. Disk/fresh-process coverage is in the separate suite.

[Frozen fixtures and rights](../crates/gel-source/fixtures/collection-review/PROVENANCE.md)
and [first-run output](evidence-collection/assessment-first.txt).

## What the score does not mean

Questions and expectations were authored by the developer, not an independent
evaluator. Some source material appeared in previous demos. This is **not a
blind holdout, semantic recall or 100% AI accuracy**. Paraphrases and cross-line
phrases deliberately return UNKNOWN: that meets the documented phrase-search
contract, not their meaning. Small-suite times do not establish tail latency.

## Ready for external review

An independent reviewer should freeze new documents and questions before seeing
results, record their SHA256, retain first outputs including failures, and grade
source selection, quote fidelity, supported/unsupported answers, ambiguity and
language separately. Do not tune on the same set and call it held-out.
Do not send private documents to public issues/artifacts. A reviewer can run
locally and share only approved synthetic/public evidence. Review is pending.
