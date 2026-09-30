# Contributing to GEL RAM

GEL RAM remains Rust-only unless the project explicitly changes that engineering rule.

## You do not need to write code to help

Reporting needs no CLA; only code intended for merge follows the steps below.

| Path | What you do | Start here |
|---|---|---|
| **Reproduce** | run `cargo run --locked --offline -p xtask -- reproduce NEW_DIR` on your machine and post REPRODUCTION.txt | [reproduction form](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml), [how to run it](docs/REPRODUCE.md) |
| **Break it** | make a public tool refuse too little, lose data, or pass a check that should fail | [break-it form](https://github.com/Gelram-project/gel-ram/issues/new?template=break-it.yml) |
| **Bring a corpus** | propose redistributable documents or questions for a public evaluation | [corpus form](https://github.com/Gelram-project/gel-ram/issues/new?template=corpus.yml) |
| **Review evidence** | read a result, protocol or claim and say what holds | [review form](https://github.com/Gelram-project/gel-ram/issues/new?template=review.yml), [claim registry](docs/CLAIMS.md) |

An independent reproduction on a second machine is the most needed report of all:
[issue #20](https://github.com/Gelram-project/gel-ram/issues/20). Evidence that does not hold up goes to the
[evidence integrity form](https://github.com/Gelram-project/gel-ram/issues/new?template=evidence-integrity.yml); see [SECURITY.md](SECURITY.md) for what to report privately.

## Before opening a pull request

External contributors must:

1. contact `gelram.licensing@gmail.com`;
2. review [CLA-PRIVACY.md](CLA-PRIVACY.md);
3. complete GEL RAM Contributor License Agreement 2.0 privately and receive written acceptance from the Project Licensor;
4. ensure employer, client, university, sponsor, or other rights-holder permission is in place when applicable;
5. identify all third-party material and its license or permission;
6. disclose material AI-assisted or automated code generation as required by CLA 2.0;
7. remove credentials, secrets, personal data, private datasets, and confidential information (the [public disclosure gate](docs/PUBLIC-DISCLOSURE-GATE.md), `cargo run --locked --offline -p xtask -- disclosure`, catches private paths, network addresses, credential-like strings and unlisted e-mail addresses; `xtask verify` runs it);
8. after changing tracked files, regenerate `SOURCE-SHA256SUMS.txt` the way the [CI manifest check](.github/workflows/ci.yml) builds it, then run `cargo run --locked --offline -p xtask -- verify`.

Only after those steps should a pull request intended for merge be opened.

## CLA acknowledgement

A pull-request checkbox or CI acknowledgement is a declaration by the contributor. It does not prove that an effective CLA exists and does not replace the Project Licensor's private contributor-rights record.

The private record should identify at minimum the Contributor, CLA version, Contributor acceptance, Project Licensor acceptance, effective date, and any specifically agreed coverage of earlier Contributions.

A pull request from an external contributor without the applicable completed and accepted CLA on file may be closed without merge, and its content must not be incorporated merely because CI is green.

Signed CLA documents, signatures, personal email addresses, authority evidence, and related personal information must not be posted in a public issue, pull request, or repository path.

## Contribution provenance

Contributors remain responsible for the provenance and legal compatibility of submitted material.

Third-party code must be clearly identified. Material that cannot lawfully be combined with GEL RAM's public source-available license and separate commercial licensing model must not be submitted for incorporation.

AI-generated or AI-assisted material must be reviewed as code, not treated as automatically safe or unencumbered.

## Correctness and evidence

A contribution that changes correctness, persistence, binary formats, retrieval, quantization, concurrency, integrity, security boundaries, or performance-sensitive code must include appropriate regression evidence.

Performance claims require reproducible before/after measurements and may not weaken exactness or integrity tests.

## Security reports

Do not disclose an unpatched vulnerability in a public issue or pull request. Follow [SECURITY.md](SECURITY.md).

## Licensing and privacy questions

Questions about contribution rights, CLA status, CLA personal-data handling, or commercial licensing must be handled privately through `gelram.licensing@gmail.com`.
