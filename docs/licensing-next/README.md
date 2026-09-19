# Historical, inactive licensing proposal

This directory preserves an earlier proposal, not a pending policy change.
Its original wording and proposed Evaluation limits below are historical.
Only the [root LICENSE](../../LICENSE) and [current licensing guide](../../LICENSING.md)
describe the operative public terms. Do not activate this directory merely
because an older heading says "next release". No license terms are changed
by this clarification.

## Original proposal (historical text)

> STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.
>
> The current repository license remains the root `LICENSE` and root `LICENSING.md` until an explicit release cutover replaces them. Nothing in this directory changes the rights granted for the current `main` tree or any historical tag.

This directory contains the complete licensing package prepared for the next GEL RAM release:

- [LICENSE](LICENSE) — GEL RAM Noncommercial Reciprocal License 1.0.
- [LICENSING.md](LICENSING.md) — plain-language scope and examples.
- [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md) — path to a separate commercial agreement.
- [CLA.md](CLA.md) — Contributor License Agreement 2.0 for contributions accepted after activation.
- [CLA-PRIVACY.md](CLA-PRIVACY.md) — personal-data notice for private CLA administration.
- [NOTICE](NOTICE) — required project notice for the next release.
- [LICENSE-MODE.txt](LICENSE-MODE.txt) — machine-readable licensing mode identifier.
- [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) — third-party boundary and current dependency inventory.
- [CONTRIBUTING.md](CONTRIBUTING.md) — contribution and provenance policy after activation.
- [SECURITY.md](SECURITY.md) — security-reporting policy after activation.
- [ACTIVATION.md](ACTIVATION.md) — atomic release-cutover procedure and verification requirements.
- [STAGED-SHA256SUMS.txt](STAGED-SHA256SUMS.txt) — SHA-256 catalog of the staged licensing documents, excluding the catalog itself.

## Intended policy

The public license is deliberately source-available rather than OSI open source. It permits noncommercial use, research, education and hobby work subject to reciprocity. Solely private noncommercial modifications may remain private without a time limit. Sharing a Modification or providing its functionality to others, including via P2P or a service for friends, requires public Corresponding Source. User Content is excluded; hiding modified Software code in a data container is not an exception. Commercial use, including internal business use that provides operational or economic advantage, requires a separate written commercial agreement specifying payment. Closed shared or hosted Modifications require express permission in that agreement.

The commercial Evaluation exception is deliberately narrow. A company group receives only one 30-day Evaluation period under NCRL 1.0, starting at its first run for Evaluation; merely reading or downloading does not start it. Changing release, version, branch, fork, mirror, copy, or build does not reset it. Publishing modifications or sending a payment without a signed agreement never grants commercial-use rights.

External contributions require bilateral CLA acceptance after activation. A CI checkbox is not proof of an effective CLA, and completed CLA records remain private.

AGPL is intentionally not offered as an alternative public license because AGPL permits commercial use when its copyleft conditions are satisfied, which is not the intended GEL RAM policy.

## Activation rule

These files become operative only when a future release explicitly adopts them by replacing the corresponding root licensing files and updating the repository licensing gate. Until then they are preparatory documents only.

The activation review must confirm the Project Licensor's actual legal identity and jurisdiction, third-party rights, CLA privacy practices, exact Cargo metadata, source SHA-256 manifest and CI on the same release tree. The staged public license does not guess a governing-law jurisdiction.

Commercial, contributor and licensing contact: `gelram.licensing@gmail.com`.
