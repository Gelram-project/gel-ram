# Next-license activation procedure

> STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.

The next licensing model must be activated atomically on one reviewed release tree. Do not partially activate it.

## 1. Preconditions

Before cutover:

1. Freeze the exact release candidate commit.
2. Confirm that the Project Licensor has authority to license every GEL RAM-owned file in the release.
3. Confirm the actual legal identity and jurisdiction of the Project Licensor sufficiently for licensing, CLA administration, privacy obligations, invoicing, enforcement, and any commercial agreement.
4. Decide explicitly whether the NCRL clause that selects no governing law or exclusive forum remains appropriate for that legal identity and release context; do not insert a jurisdiction by assumption.
5. Review new or changed third-party material and regenerate the dependency inventory.
6. Confirm that every external contribution intended for the release is covered by an applicable completed contributor agreement or another documented rights basis.
7. Confirm that the CLA privacy notice accurately describes the actual controller, storage, service providers, retention process, lawful basis where applicable, and any international-transfer requirements.
8. Review the final license text for the jurisdiction and release context in which it will be used.
9. Confirm the revised policy deliberately permits unlimited solely private noncommercial Modifications, requires source on sharing or supplying their functionality (including P2P), and excludes User Content without permitting Software source to be disguised as data.
10. Confirm payment terms will be set in a signed commercial agreement; closed shared Modifications require an express exception. No fees or royalties are automatically collected by this repository.

## 2. Root-file cutover

In one licensing change set, replace or create the root files from the staged package:

- `docs/licensing-next/LICENSE` -> root `LICENSE`;
- `docs/licensing-next/LICENSING.md` -> root `LICENSING.md`;
- `docs/licensing-next/COMMERCIAL-LICENSE.md` -> root `COMMERCIAL-LICENSE.md`;
- `docs/licensing-next/CLA.md` -> root `CLA.md`;
- `docs/licensing-next/CLA-PRIVACY.md` -> future root CLA-PRIVACY.md;
- `docs/licensing-next/NOTICE` -> root `NOTICE`;
- `docs/licensing-next/LICENSE-MODE.txt` -> root `LICENSE-MODE.txt`;
- [staged third-party notices](THIRD-PARTY-NOTICES.md) -> a new root third-party notices file;
- `docs/licensing-next/CONTRIBUTING.md` -> root `CONTRIBUTING.md`;
- `docs/licensing-next/SECURITY.md` -> root `SECURITY.md`.

Remove the `STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.` status markers from files that become the operative root documents.

Review relative links after relocation. In particular, the staged third-party
notice's relative real-source link must become `docs/REAL-SOURCE-DEMO.md`
when that notice is copied to the root. Preserve the fixture's MIT license and
attribution; do not assert that the entire distribution has no other licenses.

The rights review must explicitly cover direct grants from Modification Authors,
separately licensed dependencies, and authority to offer any included fork under
commercial terms. The draft does not automatically acquire those rights.

Do not alter historical release tags.

## 3. Cargo metadata cutover

Because GEL RAM NCRL 1.0 is a project-specific license rather than an SPDX-listed standard license, the release workspace should use Cargo's `license-file` metadata instead of claiming an SPDX identifier that does not exist.

At cutover:

1. replace the workspace `license = "PolyForm-Noncommercial-1.0.0"` entry with `license-file = "LICENSE"`;
2. update every workspace member that currently inherits `license.workspace = true` to inherit the workspace `license-file` setting;
3. reject manifests that simultaneously claim the obsolete PolyForm identifier or another conflicting public license;
4. keep `publish = false` unless publication policy is intentionally changed.

The exact Cargo syntax must be validated with the release toolchain before merge.

## 4. Licensing gate cutover

Update `xtask licensing` in the same change set.

The new gate must:

1. accept only `GEL-RAM-NCRL-1.0 + Commercial + CLA-2.0` from root `LICENSE-MODE.txt`;
2. verify the complete operative `LICENSE` by SHA-256, not CRC64;
3. verify the exact `Required Notice:` line and Project Licensor identity in root `NOTICE`;
4. verify that `COMMERCIAL-LICENSE.md` states that it is not itself a commercial license grant;
5. verify the CLA version and the copyright relicensing, patent, authority, provenance, AI-assisted-material, bilateral-acceptance, privacy-reference, and private-record clauses;
6. verify root `LICENSING.md` identifies the license as source-available and noncommercial reciprocal and does not claim OSI approval;
7. inspect every workspace package manifest and reject any package that does not inherit the operative license-file metadata;
8. verify the licensing contact consistently uses `gelram.licensing@gmail.com`;
9. fail if an AGPL, GPL, MIT, Apache, PolyForm, or other alternative public license is accidentally presented as an additional license for GEL RAM-owned code unless that change is an explicit later licensing decision;
10. verify that the one-time commercial Evaluation cannot be reset by changing release, version, branch, fork, mirror, copy, or build.
11. verify the private-use, User Content, network-service reciprocity and express commercial-exception clauses against the reviewed license; the commercial Evaluation starts at first execution for Evaluation, not a source-page view or unexecuted download.

The staged Rust regression tests check selected text invariants and simulated omissions. They are not legal validation, a complete contract parser, evidence of ownership, or a substitute for the operative license SHA-256 check at activation.

## 5. Documentation cutover

Current documentation must be updated to describe the new licensing model. Historical evidence and historical release notes must remain truthful about the license that applied at the time; do not rewrite historical statements merely to match the new release.

The current README must state clearly:

- noncommercial reciprocal public use;
- source-publication obligations for Modifications;
- commercial use requires a separate written agreement;
- public source disclosure does not authorize commercial use;
- the license is source-available, not represented as OSI open source;
- where to find commercial licensing, contributor terms, and CLA privacy information.

## 6. Third-party verification

Regenerate the [staged third-party notice inventory](THIRD-PARTY-NOTICES.md) from the exact locked dependency and release contents before copying it to the root release package.

For every third-party component actually redistributed, preserve all license text, notices, attribution, source offers, or other obligations required by that component's license.

Do not infer third-party rights from compatibility alone. Verify the license metadata and shipped license files of the exact versions actually distributed.

## 7. Contributor-rights and privacy verification

Before activation:

1. maintain a private contributor-rights register identifying contributor, CLA version, acceptance date, Project Licensor acceptance, and covered contributions;
2. never publish signed CLA documents or personal-data records in the repository;
3. verify that the operative CLA privacy notice matches actual processing practices;
4. document any alternative rights basis used for a contribution not covered by CLA 2.0.

The PR checkbox remains a declaration only and is not evidence that the private register contains a valid CLA.

## 8. Source-integrity update

After all licensing and documentation files are final:

1. regenerate source SHA-256 manifests for the exact release tree;
2. ensure the licensing files themselves are included in the source manifest;
3. run the independent packaging/source-integrity check;
4. preserve the old manifests with historical evidence rather than rewriting them.

## 9. CI and repository policy

Before release, confirm that protected-branch rules still require the licensing/verify gate and relevant platform checks.

The CLA CI acknowledgement remains metadata only. It must not be described as proof that a signed CLA exists. The private contributor-rights record remains authoritative.

## 10. Release acceptance test

The cutover is complete only when all of the following refer to the same source tree:

- root licensing files;
- Cargo package metadata;
- `xtask licensing`;
- README/current documentation;
- third-party notice inventory;
- CLA privacy notice and contributor-rights process;
- source SHA-256 manifest;
- CI results;
- release tag and release archive.

A green test from a different commit does not satisfy this requirement.

## 11. Rollback rule

If the licensing cutover fails review or verification, do not publish the release. Restore the candidate branch to a coherent pre-cutover state or correct the candidate before release. Never rewrite already-published historical tags to simulate a successful cutover.
