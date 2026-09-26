# GEL RAM Evidence Lab — public integration candidate

Updated 2026-09-26. Originally prepared from published commit
`8b92deb912a159a78ae1e8a5f7e7234b717cbd6a`; merged into public `main` through
PR #9 as `71142a25e7ad75e4d75acf4e244d8e4a996c4a92` after successful checks,
with the owner's authorization. PR #10 carries the review repairs; its merge
requires a separate owner decision and is shown on its GitHub page.
A final tag and release are not authorized.
Public pseudonym: **RR — GEL RAM Project**.

```text
REVIEW_PUBLICATION_APPROVED=YES
PUBLICATION_APPROVED=YES
PUBLICATION_SCOPE=PUBLIC_MAIN_INTEGRATION
MERGE_APPROVED=YES
MERGE_APPROVED_FOR=PR_9_MERGED_AS_71142A2
RELEASE_APPROVED=NO
SNAPSHOT_KIND=PUBLIC_INTEGRATION_CANDIDATE
LOCAL_VALIDATION_WINDOWS_TESTED=NO
LOCAL_VALIDATION_MACOS_TESTED=NO
EXACT_REVISION_PLATFORM_RESULTS=SEE_PINNED_CI_RECORD
LINUX_WORKSPACE_TESTS_AT_71142A2=305_PASS
CURRENT_TEST_COUNTS=SEE_PER_PLATFORM_CI_EVIDENCE_REPORT
BASELINE_OCEAN_ARCHIVE_TESTS=111_PASS
ACTIVE_PUBLIC_LICENSE=GEL-RAM-NCRL-1.0
LICENSE_ACTIVATED=YES
VERSION_LINE=0.4.0-rc.1
TARGET_RELEASE=0.4.0
V0_4_0_TAGGED_RELEASE=NOT_CREATED
V0_3_0_VERSION_METADATA=APPROVED
V0_3_0_TAGGED_RELEASE=PUBLISHED
CI_REQUIRED_FOR_VERSION_CUTOVER=YES
LEGAL_APPROVED=NO
PUBLIC_PSEUDONYM_CONFIRMED=YES
PAID_SERVICES_AUTHORIZED=NO
```

The workspace version identifies the `0.4.0-rc.1` candidate on public main.
The v0.3.0 fields above describe the historical release, not approval of this
candidate. No v0.4.0 tag or release is authorized by this approval.
See [candidate release notes](RELEASE-NOTES-v0.4.0-RC.md).

Technical checks accept a pending candidate and report its approval status
without changing it. A successful verification never supplies publication permission.
The active root license is unchanged; no new licensing policy is activated.
Previous Linux, Windows and macOS CI results do not validate these changes.
Native CI evidence for the PR #9 head is pinned by SHA in
[the platform record](docs/PLATFORM-REVIEW.md); PR #10 revisions are evidenced
by the per-platform [CI evidence report](docs/CI-EVIDENCE.md) of their exact run. The LOCAL_VALIDATION fields
above describe local validation, not remote CI results.
Standard public-repository CI is authorized for the branch/PR; execution results
must be checked separately. Only free standard runners are in scope.

New scope: bounded multi-document collection, exact citations, no-replace
snapshots, generation invalidation, E2E measurement and Q reference tests. No private application code,
encrypted vault, speaker, encoder, private corpus or user data was exported.
Current scope and results: [Evidence Lab candidate](RELEASE-NOTES-EVIDENCE-LAB.md).
Historical R2 preparation record (its publication and platform flags predate the approvals above): [R2 preparation](docs/evidence-collection/FOLLOWUP-R2.md). Open external gates: [claim registry](docs/CLAIMS.md).
Previously published: [Live Lab update](RELEASE-NOTES-LIVE-LAB.md).
Prior results: [historical document validation](docs/PACKAGE-VALIDATION.md).

`V0_3_0_TAGGED_RELEASE=PUBLISHED` records that the annotated `v0.3.0` tag and GitHub Release have been created from the exact green public-main commit selected for the release. The public `main` branch may advance after that immutable release point through separately verified changes.

`LEGAL_APPROVED=NO` means no claim is made that independent legal counsel has certified the custom license. No new license is activated here.

## Review repairs in PR #10 (not released)

An external review of `71142a2` listed 24 points. PR #10 addresses them in
small commits, each with local verification and platform CI. Main additions:
fail-closed film recorder with a lint gate that proves it can fail; public
F32/F16/Q1–Q16 [precision matrix](docs/PRECISION-MATRIX.md); per-platform test
counts and declared platform exclusions in the [CI evidence report](docs/CI-EVIDENCE.md);
publication tested against real permission denial and a full device
([fault matrix](docs/PUBLICATION-FAULT-TESTS.md)); raw timing order and
small-sample percentile labels ([measurement protocol](docs/MEASUREMENT-PROTOCOL.md));
corrected film timelines. Human film review, independent reproductions and
the private-path research points remain open; see the [claim registry](docs/CLAIMS.md).
Test counts change with each commit; read them from the per-platform report,
not from a fixed number here.

## Previously published base

The owner additionally authorized the bounded Ocean Scale source/evidence
snapshot described in [the Ocean guide](docs/OCEAN-SCALE.md). Its exact archive
pin is checked by the repository gate and Linux CI before extraction.
This is an experimental research addition, not a v0.4.0 release or replacement
of the v0.3.0 stable workspace. Archive-internal pre-publication status remains
historical; this document records the subsequent owner authorization.

1. Rust public core, multipart source readout, integrity and reproduction tools, pinned source fixtures and dependency/third-party inventory.
2. GEL RAM NCRL 1.0 as the operative root public license for GEL RAM-owned material, plus the commercial licensing path and CLA 2.0.
3. Two English terminal demonstration films, previews and six extracted screenshots, with scope and timing limitations in [the film guide](media/FILMS-GUIDE.md).
4. Public media reuse terms in [media/RIGHTS.md](media/RIGHTS.md).

## Excluded private scope

The private application, speaker, private banks of knowledge, private encoder
and private Ocean/P2P application internals remain excluded. Only the selected
numerical reader, persistence/mapping and test modules described in the Ocean
guide are additionally authorized. User conversations, signed agreements, keys,
personal identity records and private local diagnostic logs are not included.

The two private-preview films are bounded native Rust source-frame demonstrations; the Evidence Lab film shows public phrase retrieval. None of them proves general AI, semantic accuracy, unrestricted language generation, fourfold independent storage, P2P operation or physical DRAM-refresh synchronization.

## Licensing status

The root [LICENSE](LICENSE) is operative for GEL RAM-owned material on public `main`. [LICENSING.md](LICENSING.md) explains the selected policy. Historical release grants remain historical grants and are not rewritten by a later version.

The directory `docs/licensing-next/` is retained as historical proposal material only. Its staged documents are not an additional active license and do not override the root license.

Third-party material remains subject to its own terms. The redistributed Rust Book excerpt remains MIT-licensed.

## Verification

The v0.3.0 release target passed the configured CI before publication. Linux verification included complete source-manifest validation, `xtask verify` and the independent byte/numeric/ranking audit. Windows and macOS executed workspace tests and configured runtime checks. Subsequent public-main changes are verified independently and do not move the published `v0.3.0` tag.

A green technical gate is evidence for its stated scope only. It is not legal certification, ownership proof or a guarantee of security for every environment.

Licensing and contract enquiries: **gelram.licensing@gmail.com**. Completed agreements and personal records must remain private.
