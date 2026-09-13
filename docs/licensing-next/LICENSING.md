# GEL RAM licensing for the next release

> STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.

The current repository remains governed by the root licensing files until an explicit release cutover. This document explains the intended next-release model and does not retroactively alter rights already granted for earlier releases.

## 1. Public license

The next release is intended to use **GEL RAM Noncommercial Reciprocal License 1.0 (GEL RAM NCRL 1.0)** in this directory.

This is a source-available, noncommercial, reciprocal license. It is not represented as an OSI-approved open-source license.

The controlling text is [LICENSE](LICENSE). This document is explanatory. If wording here conflicts with [LICENSE](LICENSE), [LICENSE](LICENSE) controls.

## 2. Core rule

The intended rule is simple:

1. Noncommercial use is permitted subject to the license.
2. Solely private noncommercial modifications may remain private without a time limit. Distribution, access or functionality supplied to another person requires publication of complete Corresponding Source for the Modification.
3. Commercial use requires a separate written commercial agreement with the Project Licensor specifying fees or royalties and payment conditions. A separate express permission is needed to waive source-publication obligations for shared or hosted Modifications.
4. Publishing source code never converts commercial use into permitted noncommercial use.

## 3. Noncommercial examples

Examples that are normally noncommercial when they do not serve a commercial beneficiary:

- personal study and experimentation;
- hobby projects;
- independent research with no anticipated commercial deployment;
- classroom teaching and coursework;
- charitable or public-interest research;
- public reproducibility work and benchmarking;
- publication of a modified GEL RAM implementation together with its complete Corresponding Source under GEL RAM NCRL 1.0.

These examples remain subject to the exact definitions and conditions in [LICENSE](LICENSE).

## 4. Commercial examples

A separate commercial agreement is required for, among other things:

- use inside a for-profit company for business operations;
- incorporation into a paid product;
- incorporation into a SaaS or hosted service;
- advertising-supported services;
- paid consulting or contract delivery;
- production infrastructure;
- customer-facing systems;
- use that reduces business costs or provides operational or competitive advantage;
- commercial R&D;
- development, training, tuning, evaluation, or operation of systems intended for later commercial deployment;
- use by a contractor on behalf of a commercial customer.

The fact that GEL RAM is not directly sold does not by itself make a business use noncommercial.

Employment at a company alone does not make an individual's separate personal hobby commercial. Work for that company's business benefit is different. A nonprofit label does not exempt activity serving a commercial beneficiary. If the intended use is uncertain, obtain written clarification before using the Software for it.

## 5. Company evaluation

A for-profit entity and its controlled or commonly controlled affiliates collectively receive only one 30-day Evaluation period under the public license.

That period begins when a member first runs the Software on behalf of the group for Evaluation under NCRL 1.0. Reading source or documentation or downloading without execution does not start the clock. A new version, release, branch, fork, mirror, copy, or build does not restart the period.

Evaluation covers only internal examination of an unmodified copy to decide whether to seek a commercial license. It does not permit production deployment, integration into business operations, customer-facing use, commercial R&D output, or use that produces operational, competitive, or economic advantage.

Internal test measurements solely for the licensing decision are allowed. They are not permission to deploy the Software or reuse its outputs in the excluded business activities.

The Project Licensor may extend Evaluation only in writing.

## 6. Reciprocity

Solely private noncommercial use of a Modification has no publication deadline. This is not a commercial-use exception.

Distribution of a Modification, access to it, or provision of its functionality to another person triggers the source obligation immediately. A network service, P2P node, invitation-only group or service for friends is not exempt. A wrapper does not hide the underlying Modification from this requirement. Use of an unmodified copy does not create a new modification-publication obligation.

Complete Corresponding Source must be public under GEL RAM NCRL 1.0 while the Modification is distributed or made available to others, and for at least three years after the last such act. Later solely private use does not extend that period; subsequently shared updates need their own corresponding source. Source must be accessible without payment or invitation, not just on request from the Project Licensor.

The requirement covers GEL RAM-derived source and material necessary to build and run the Modification. It does not claim all independent software communicating with GEL RAM. Section 6 of the license governs separate components. Publishing changes does not require submitting a PR, signing a CLA for a fork, or assigning ownership to the Project Licensor.

User Content is not Corresponding Source. Conversations, photographs, personal knowledge, ordinary ORB payloads and independently created input/output data do not become GEL RAM modifications merely by being processed or stored. Publishing source is not permission to publish other people's data. Necessary configuration must use documented placeholders or synthetic fixtures instead of secrets and private content. Conversely, packing modified GEL RAM code into an ORB or calling it data does not avoid reciprocity. These are licensing boundaries, not encryption or access-control guarantees.

## 7. No AGPL alternative

AGPL is deliberately not offered as an alternative public license. AGPL allows commercial use when its conditions are met. That does not implement the GEL RAM rule that commercial use requires a separate commercial agreement.

## 8. Commercial licensing

[COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md) describes the contact path only. It is not itself a commercial license grant.

Commercial terms exist only in a separate written agreement signed by the Project Licensor and the customer. The agreement specifies payment; commercial permission does not by itself waive section 5. Permission for closed shared or hosted Modifications must be express. No royalty percentage, automatic invoice or commercial right is created by this explanatory document or an unsolicited payment.

Contact: `gelram.licensing@gmail.com`.

## 9. Contributions and CLA privacy

After activation, external code contributions are intended to require **GEL RAM CLA 2.0** from [CLA.md](CLA.md) before a pull request is opened.

The CLA becomes effective only after both Contributor acceptance and Project Licensor written acceptance are recorded privately. A CI checkbox is not proof of a completed CLA.

The CLA is non-exclusive. Contributors retain ownership of their contributions while granting the Project Licensor rights sufficient to maintain the public license and offer separate commercial licenses.

Publishing a fork under NCRL is not the same as signing that CLA. Modification
Authors grant recipients rights in their covered portions directly under section 6;
they do not automatically grant the Project Licensor commercial relicensing rights.
A commercial offering containing those portions must have the necessary rights
from every relevant rights holder. A permission from RR cannot replace a missing
permission from another author.

An independently authored API client does not become a Modification merely by
communicating with GEL. Copied or adapted GEL code does not become independent
by moving it into a plugin or separate process. Exact boundaries depend on the
material and applicable law. Unmodified external dependencies may be identified
with exact versions, licenses and lawful acquisition/build instructions where
their licenses allow; this cannot conceal required GEL-derived source.

Personal information collected for CLA administration is covered by [CLA-PRIVACY.md](CLA-PRIVACY.md) after activation and must not be published in the repository.

## 10. Third-party material

GEL RAM NCRL 1.0 applies only to material for which the Project Licensor has authority to grant those rights. Third-party dependencies and other identified third-party material remain under their own terms. See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## 11. Names and marks

The license does not grant ownership or general branding rights in `GEL RAM`, `GEL RAM Project`, project logos, or other marks. Truthful identification of the software is allowed only to the extent described by [LICENSE](LICENSE).

## 12. Governing law

The staged public license deliberately does not guess the Project Licensor's legal jurisdiction. It does not select a governing law or exclusive forum. The activation review must confirm whether that approach remains appropriate for the actual legal controller and release context.

A separate commercial agreement may specify governing law and dispute resolution independently.

## 13. Historical releases

Historical tags retain the license under which they were originally distributed. The next-release license is not intended to revoke or rewrite rights already granted for those historical releases.

## 14. Activation

The staged package becomes active only through the controlled procedure in [ACTIVATION.md](ACTIVATION.md), including replacement of the root license files and update of the machine licensing gate on the same source tree.
