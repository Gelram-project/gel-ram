# Third-party notices

## 1. Boundary

GEL RAM Noncommercial Reciprocal License 1.0 applies only to material for which the Project Licensor has authority to grant those rights.

Third-party software, documentation, data, artwork, standards text, generated material, or other content remains subject to the rights and license terms supplied by its respective owner. Nothing in the GEL RAM public license replaces, narrows, expands, or relicenses those third-party rights.

## 2. Current non-vendored Rust dependency inventory

The current source tree records the following external crates in `Cargo.lock`:

| Crate | Locked version |
| --- | --- |
| block-buffer | 0.10.4 |
| cfg-if | 1.0.4 |
| cpufeatures | 0.2.17 |
| crypto-common | 0.1.7 |
| digest | 0.10.7 |
| generic-array | 0.14.7 |
| libc | 0.2.189 |
| sha2 | 0.10.9 |
| tinyvec | 1.13.3 |
| typenum | 1.20.1 |
| unicode-normalization | 0.1.25 |
| version_check | 0.9.5 |

These packages are dependency references fetched from their upstream distribution sources; they are not relicensed under GEL RAM NCRL 1.0 by this repository.

The [exact inventory](docs/DEPENDENCY-INVENTORY.md) records archive checksums,
declared license expressions, upstream repositories and license-file hashes
checked against the exact cached crate archives. It does not replace the
upstream license text or approve a future binary redistribution.

The authoritative license for each package is the license information and license text shipped by that exact upstream package version. This file intentionally does not paraphrase or replace those upstream terms.

## 3. Release-package rule

Before activation for a new release, the dependency inventory must be regenerated from the exact locked release tree.

If a release archive, installer, binary bundle, vendored source tree, generated artifact, image, fixture, dataset, or other distributed material contains third-party content, the release must include every copyright notice, attribution, license text, source offer, or other notice required by the applicable third-party license.

A dependency being compatible with GEL RAM does not make that dependency part of the GEL RAM license grant.

## 4. No assumptions from Cargo metadata

A package name or `Cargo.lock` entry alone is not a legal conclusion about its license. Release preparation must verify the license metadata and license files of the exact dependency version actually distributed.

## 5. Contributions containing third-party material

A contributor must identify third-party material before submission, including its origin and applicable license or permission. Material that cannot lawfully coexist with the intended GEL RAM public and commercial licensing model must not be incorporated.

## 6. Redistributed Rust Book excerpt

The real-source demo includes an unmodified excerpt of *The Rust Programming
Language*, Copyright (c) 2010 The Rust Project Developers, under its MIT license.
The full license accompanies the fixture. The pinned revision, original byte
ranges and hashes are documented in [real-source provenance](docs/REAL-SOURCE-DEMO.md).
This documentation excerpt remains MIT-licensed, independently of the active GEL
terms. The generated GEL catalog and example code are separate project material.

## 7. Wikipedia text (CC BY-SA 4.0)

Measurement files in this repository quote passages of Polish and English
Wikipedia articles, word for word, and the questions paraphrase them:

- the answer-or-abstain sets in [docs/answer-or-abstain](docs/answer-or-abstain/README.md),
  [v2](docs/answer-or-abstain-v2/README.md), [v3](docs/answer-or-abstain-v3/README.md),
  [v4](docs/answer-or-abstain-v4/README.md) and [v5](docs/answer-or-abstain-v5/README.md): question
  files and recorded answers;
- the side-by-side pages [GEL beside Groq](docs/GEL-BESIDE-GROQ.md) and
  [no-answer control](docs/GEL-BESIDE-GROQ-NO-ANSWER.md), the records in
  `docs/evidence-side-by-side/` and the recordings and images in `media/beside-groq/`.

Wikipedia text is licensed under the Creative Commons Attribution-ShareAlike 4.0
International License (CC BY-SA 4.0, <https://creativecommons.org/licenses/by-sa/4.0/>).
It is not GEL RAM-owned material and is not licensed under GEL RAM NCRL 1.0: each
quoted passage, and any adaptation of it in these files, remains under CC BY-SA 4.0,
including the uses that license permits.

- **Source and authors.** For every question passage, the question files name the
  article title and URL, the dump (`wikipedia_pl_all_maxi_2026-05` or
  `wikipedia_en_all_maxi_2026-02`) and the entry it was taken from. The authors are
  the contributors listed in the history of each article at its URL.
- **Changes.** Articles were split into sections and stored as plain text; questions
  were written as paraphrases for measurement.
- **Recorded answers.** A passage returned as an answer comes from the same two
  dumps; its article title is not yet listed next to each answer.

## 8. Project contact

Questions about third-party notices or licensing boundaries: `gelram.licensing@gmail.com`.
