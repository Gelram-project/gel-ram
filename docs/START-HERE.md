# GEL RAM — start here

If someone sent you one GEL RAM link, use this page to understand the public
project without reading the entire repository.

## In one paragraph

GEL RAM is an independent Rust research project exploring RAM-resident knowledge,
exact source-bound readout and alternative memory execution models. The public
repository contains the reproducible subset: source provenance, integrity gates,
Q8 numeric experiments, persistent source bundles, an offline Live Lab and
1M/10M synthetic Ocean evidence. The complete private research system is larger
and is intentionally not described here.

## Five useful entry points

1. **See what exists:** [README](../README.md)
2. **Run it yourself:** [TRY-IT](TRY-IT.md)
3. **Check the evidence:** [VERIFIED RESULTS](VERIFIED-RESULTS.md)
4. **Inspect 1M/10M measurements:** [Ocean Scale](OCEAN-SCALE.md)
5. **See what comes next publicly:** [Roadmap](ROADMAP.md)

## The most important performance distinction

The published Ocean 1M/10M timings are a **CPU/RAM full-scan baseline**.
They are intentionally retained because they are independently inspectable and
give the project a reproducible comparison point.

They are **not** presented as the performance ceiling or final GEL RAM execution
model. The private project has newer owner-reported experimental results, but
the detailed mechanism and performance claims remain unpublished until a
reproducible public package is ready.

## What you can verify today

- public Rust workspace verification;
- source SHA-256 manifest;
- exact source quotations and byte ranges;
- stale/corrupt input rejection;
- no-replace source-bundle persistence and fresh-process reopen;
- Q8 inverse/exactness checks;
- independent byte/numeric/ranking audit;
- Ocean R3 source/evidence archive and offline verifier;
- Linux/macOS/Windows public-core checks.

## What would help the project most

Independent reproduction is currently more valuable than repeating the same
measurement on the original machine.

Useful contributions include:

- run Ocean R3 on a second Linux host;
- run the public core on a different CPU family;
- report slower or failing cases;
- test Unicode/source-boundary edge cases;
- measure complete memory costs;
- find corruption, restart or concurrency cases the current tests missed.

Use the
[reproduction issue template](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml).
A reproduction report does not require a CLA. Code intended for merge follows
[CONTRIBUTING](../CONTRIBUTING.md).

## Public / private boundary

Public documentation intentionally does not disclose the private next-generation
execution mechanism, unpublished private banks, private encoder, speaker,
private network implementation, keys or user data.

Do not infer those components from the public CPU/RAM baseline.

## Citation

GitHub can expose the repository citation metadata through the root
`CITATION.cff`. When discussing a measurement, also record the **exact commit
SHA or release tag**, because public `main` can advance after a tagged release.
