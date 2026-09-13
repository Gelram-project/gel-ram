# Contributing to GEL RAM

GEL RAM remains Rust-only unless the project explicitly changes that engineering rule.

## Before opening a pull request

External contributors must:

1. contact `gelram.licensing@gmail.com`;
2. review [CLA-PRIVACY.md](CLA-PRIVACY.md);
3. complete GEL RAM Contributor License Agreement 2.0 privately and receive written acceptance from the Project Licensor;
4. ensure employer, client, university, sponsor, or other rights-holder permission is in place when applicable;
5. identify all third-party material and its license or permission;
6. disclose material AI-assisted or automated code generation as required by CLA 2.0;
7. remove credentials, secrets, personal data, private datasets, and confidential information;
8. run `cargo run --locked --offline -p xtask -- verify`.

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
