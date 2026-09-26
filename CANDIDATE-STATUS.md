# GEL RAM Evidence Lab: integration and review status

Updated 2026-09-26. Evidence Lab was merged into public main through
[PR #9](https://github.com/Gelram-project/gel-ram/pull/9) at
`71142a25e7ad75e4d75acf4e244d8e4a996c4a92`. It was originally prepared from
`8b92deb912a159a78ae1e8a5f7e7234b717cbd6a` and reviewed against the earlier main
`44b6d7af4d1ffc5715585d2d5c5db42ee401e4f8`.
A final tag and release are not authorized by that integration approval.
Public pseudonym: **RR — GEL RAM Project**.

## Scope of the recorded approval

The approval fields and 305/111 test-count fields below describe the PR #9
integration baseline. They do not grant blanket approval to later commits or
authorize merging PR #10. They are not the current test count of every checkout.
The local Windows/macOS fields describe the original local validation, not the
subsequent native CI results linked in the platform record.

```text
REVIEW_PUBLICATION_APPROVED=YES
PUBLICATION_APPROVED=YES
PUBLICATION_SCOPE=PUBLIC_MAIN_INTEGRATION
MERGE_APPROVED=YES
RELEASE_APPROVED=NO
SNAPSHOT_KIND=PUBLIC_INTEGRATION_CANDIDATE
LOCAL_VALIDATION_WINDOWS_TESTED=NO
LOCAL_VALIDATION_MACOS_TESTED=NO
EXACT_REVISION_PLATFORM_RESULTS=SEE_PINNED_CI_RECORD
CURRENT_LINUX_WORKSPACE_TESTS=305_PASS
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

The workspace remains `0.4.0-rc.1`, not a final v0.4.0 release. The v0.3.0
fields describe the historical release. See [candidate release notes](RELEASE-NOTES-v0.4.0-RC.md).
Technical checks report these recorded fields; a successful verification does
not expand their approval scope. No new licensing policy is activated.
PR #9 native CI evidence is pinned by SHA in [the platform record](docs/PLATFORM-REVIEW.md).
Only the configured standard free public-repository runners are in scope.

## Follow-up under review: PR #10

At this dated check, [PR #10](https://github.com/Gelram-project/gel-ram/pull/10)
is open and not merged. The implementation inventory was reviewed at
`6f8d3c1e27707e854217a7eb2e448ee66cd2b2f0` on branch
`docs/gel-measurements-20260926`. The documentation update built on that revision
keeps code and recorded measurements unchanged. Later PR state must be checked
at its canonical link rather than inferred from this snapshot.

The complete PR includes runtime and test changes, not documentation alone:
quote context, collection-root handling and staged construction, publication
failure tests, recorder safety, per-process runtime checks, historical-source
verification and CI evidence reporting. It also adds scoped component evidence,
codec boundaries, a claim registry and film/transcript navigation. The public
F32/F16/affine Q1 to Q16 reference and GPMX container are described in
[the precision matrix](docs/PRECISION-MATRIX.md); this is not the private GEL codec.

Use the [roadmap and A01-A24 acceptance list](docs/ROADMAP.md) to distinguish
implemented review-branch work from open acceptance. Full visual review of all
films, measured collection-optimization gains, independent second-host evidence,
end-to-end application integration of all precision formats and same-task
end-to-end performance acceptance are
not established by this document. Historical data and videos remain historical.

Each follow-up commit needs checks for its exact revision. The [CI evidence
contract](docs/CI-EVIDENCE.md) distinguishes test executions, profiles, platform
exclusions and actual command outcomes. Earlier green jobs do not validate a new
commit; a synthetic PR merge does not mean main was updated. PR #10 integration
requires its own review and owner approval. No final tag, release, private-code
export or paid service is authorized by this documentation update.

## PR #9 scope and retained evidence

Bounded multi-document collection, exact citations, no-replace snapshots,
generation invalidation, E2E measurement and Q reference tests were integrated.
No private application code, encrypted vault, speaker, encoder, private corpus
or user data was exported.
Scope and results: [Evidence Lab candidate](RELEASE-NOTES-EVIDENCE-LAB.md).
Historical recording/assessment preparation: [R2 preparation](docs/evidence-collection/FOLLOWUP-R2.md).
Previously published: [Live Lab update](RELEASE-NOTES-LIVE-LAB.md).
Prior results: [historical document validation](docs/PACKAGE-VALIDATION.md).

`V0_3_0_TAGGED_RELEASE=PUBLISHED` records that the annotated `v0.3.0` tag and GitHub Release have been created from the exact green public-main commit selected for the release. The public `main` branch may advance after that immutable release point through separately verified changes.

`LEGAL_APPROVED=NO` means no claim is made that independent legal counsel has certified the custom license. No new license is activated here.

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

The films are bounded native Rust source-frame demonstrations. They do not prove general AI, semantic accuracy, unrestricted language generation, fourfold independent storage, P2P operation or physical DRAM-refresh synchronization.

## Licensing status

The root [LICENSE](LICENSE) is operative for GEL RAM-owned material on public `main`. [LICENSING.md](LICENSING.md) explains the selected policy. Historical release grants remain historical grants and are not rewritten by a later version.

The directory `docs/licensing-next/` is retained as historical proposal material only. Its staged documents are not an additional active license and do not override the root license.

Third-party material remains subject to its own terms. The redistributed Rust Book excerpt remains MIT-licensed.

## Verification

The v0.3.0 release target passed the configured CI before publication. Linux verification included complete source-manifest validation, `xtask verify` and the independent byte/numeric/ranking audit. Windows and macOS executed workspace tests and configured runtime checks. Subsequent public-main changes are verified independently and do not move the published `v0.3.0` tag.

A green technical gate is evidence for its stated scope only. It is not legal certification, ownership proof or a guarantee of security for every environment.

Licensing and contract enquiries: **gelram.licensing@gmail.com**. Completed agreements and personal records must remain private.
