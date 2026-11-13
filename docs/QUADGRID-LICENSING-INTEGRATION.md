# QuadGrid licensing and joint-release integration plan

Status: **PLANNING / NO GRANT OF RIGHTS**. This document coordinates the additive QuadGrid technical work with a license and disclosure review. It does not amend [LICENSE](../LICENSE), itself grant any proprietary or commercial license, establish ownership of private code, or certify a legal outcome.

## 1. Current public baseline

- GEL RAM-owned material expressly under [GEL RAM NCRL 1.0](../LICENSE) remains governed by those terms. In particular, noncommercial rights, free Evaluation, commercial-agreement requirements, notices, historical rights and patent provisions remain unchanged (LICENSE sections 3 to 9 and 13).
- Existing `gel-phase-quad` phase-Q8 `Record` remains a 1152 B in-memory representation, not a 256 B codec. No existing Q8, ORB128, public binary format, data store, or historical release is rewritten here.
- Current Quad four-view scoring does not establish four independently decodable datasets. The 4x density target remains an unproven, separately benchmarked research hypothesis.

## 2. Scope and rights inventory before any implementation release

| Asset | Proposed placement | Intended terms / review | Initial status |
| --- | --- | --- | --- |
| Existing GEL RAM code, docs, tests and media | Public `gel-ram` | Existing NCRL 1.0 and identified third-party licenses | Unchanged |
| Non-sensitive adapter specification and optional format identifiers | Public only after disclosure review | NCRL 1.0 if published as GEL RAM-owned material | Plan only |
| Q8-256 and Q8-QG4-256 experimental encoder/decoder, learned tables, tuning and unpublished fixtures | Access-controlled private repository/workspace | Separate proprietary terms **only where rights permit**; review code provenance and derivation | Not provided in public tree |
| Independent candidate module communicating through a documented public interface | Private initially | Separate terms possible if independently developed without incorporating/adapting GEL RAM code (LICENSE section 7) | Independence not established |
| GEL RAM-derived private or closed commercial improvement | Private only as permitted | Existing owner rights, contributor grants and any applicable signed Commercial Agreement; no automatic closed commercial permission | Contract review required |
| Reproducible non-sensitive results | Public after approval | Attribute sources, specify data rights, limitations and measurement context | Pending |

Rights audit must record: authorship/owner, contributor CLA 2.0 coverage and acceptance, third-party dependencies and their licenses, prior release origins, any copied/adapted GEL RAM code, patent disclosure questions, and rights to test datasets. `CLA.md` grants the Project Licensor broad rights to accepted contributions; it does not prove all contributors signed or that third-party restrictions are absent.

**Important:** Calling an implementation “QuadGrid Proprietary” does not make already-published code proprietary, revoke prior licenses, assign someone else's copyright, or make a derivative component independent. Any actual proprietary distribution needs its own approved license text and verified rights chain. This document is not that license.

## 3. Boundary for joint technical and license delivery

1. Preserve existing `main` and its public `Record` and persistent formats; keep the candidate opt-in and separately versioned.
2. Implement private prototypes and private test fixtures outside this public branch. Do not commit algorithms, secrets, proprietary dictionaries, unpublished experimental results, or private file paths.
3. Maintain a public adapter specification **only** if a technical and legal disclosure review finds it safe. Runtime selection must use explicit format identifiers; reject unknown format IDs rather than silently reinterpret bytes.
4. Where a plugin is genuinely independent, document the API/protocol boundary and provenance. Where private code adapts GEL RAM, label it as derived and assess permissions accordingly; a process boundary alone is insufficient.
5. Before any distribution, prepare a release manifest with file-by-file license/attribution mapping, required notices, permitted uses, owner approvals, compatibility evidence and third-party dataset licenses. The public NCRL text remains the source of public rights.
6. If offering QG commercially, prepare and sign applicable agreements defining modules/versions, scope, closed-modification rights, consideration, source-sharing obligations, patent terms, confidentiality, sublicensing, support, termination and post-termination use. No commercial rights are created by this proposal.

## 4. Joint acceptance gates (all required)

- **L0: Rights inventory.** No unresolved ownership, CLA, third-party-license, or derivative/independence questions for proposed distributed code.
- **L1: Public/private segregation.** Review the full diff, metadata, source maps, logs, fixtures and documentation for disclosure, not merely automated secret scanners; public branch contains no undisclosed implementation or restricted data.
- **L2: License mapping.** Every distributed component has documented terms, copyright/attribution and notices; license text and commercial-agreement execution are approved separately. No retroactive changes to historical grants.
- **L3: Technical gates.** Pass [Q8/QG technical G0–G4](QUADGRID-ADDITIVE-PLAN.md) with pinned, independently checkable metrics. For 4x claims include total physical bytes (side data included), numerical error, semantic utility and a high-entropy negative control.
- **L4: Release approval.** Owner explicitly reviews both technical and legal evidence and approves the precise public payload. No automatic publication, PR merge, release or deployment.

## 5. Handoff and release checklist

- [x] Public baseline and current NCRL 1.0 preserved in this planning branch.
- [x] Technical G0–G4 and legal L0–L4 made part of a single decision record.
- [ ] Obtain complete private source and third-party provenance map from authorized rights holders.
- [ ] Review independent-versus-derived status of any private QG implementation.
- [ ] Complete a legally reviewed proprietary-license/Commercial Agreement **if and only if** needed for a distribution; do not attach a speculative license to code.
- [ ] Implement and benchmark Q8-256 and Q8-QG4-256 privately.
- [ ] Run existing and candidate regression checks and verify resource accounting.
- [ ] Review exact intended publication or customer distribution with an appropriately qualified lawyer.
- [ ] Owner approves a narrow, versioned release or records NO-GO.

References: [LICENSE](../LICENSE), [LICENSING.md](../LICENSING.md), [COMMERCIAL-LICENSE.md](../COMMERCIAL-LICENSE.md), [CLA.md](../CLA.md), [CONTRIBUTING.md](../CONTRIBUTING.md), [public disclosure review](PUBLIC-DISCLOSURE-GATE.md), [technical plan](QUADGRID-ADDITIVE-PLAN.md).
