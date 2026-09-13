# Third-party notices for the next GEL RAM release

> STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.

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
| typenum | 1.20.1 |
| version_check | 0.9.5 |

These packages are dependency references fetched from their upstream distribution sources; they are not relicensed under GEL RAM NCRL 1.0 by this repository.

The [exact inventory](../DEPENDENCY-INVENTORY.md) records archive checksums,
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
ranges and hashes are documented in [real-source provenance](../REAL-SOURCE-DEMO.md).
This documentation excerpt remains MIT-licensed, independently of the staged GEL
terms. The generated GEL catalog and example code are separate project material.

## 7. Project contact

Questions about third-party notices or licensing boundaries: `gelram.licensing@gmail.com`.
