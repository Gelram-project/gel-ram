# GEL RAM document-readout publication scope

2026-09-19. The project owner authorized publication of the reviewed public
document-readout addition based on commit
`d0c9b67110e43bb9e72c463a55f1c084175bc3df`, conditional on successful checks.
This authorization excludes the private application and its newer experiments.
Public pseudonym: **RR — GEL RAM Project**.

```text
REVIEW_PUBLICATION_APPROVED=YES
PUBLICATION_APPROVED=YES
SNAPSHOT_KIND=PUBLIC_CORE_UPDATE
LOCAL_VALIDATION_WINDOWS_TESTED=NO
LOCAL_VALIDATION_MACOS_TESTED=NO
EXACT_REVISION_PLATFORM_RESULTS=SEE_GITHUB_ACTIONS
CURRENT_LINUX_WORKSPACE_TESTS=222_PASS
CURRENT_OCEAN_ARCHIVE_TESTS=111_PASS
ACTIVE_PUBLIC_LICENSE=GEL-RAM-NCRL-1.0
LICENSE_ACTIVATED=YES
VERSION_LINE=0.3.0
V0_3_0_VERSION_METADATA=APPROVED
V0_3_0_TAGGED_RELEASE=PUBLISHED
CI_REQUIRED_FOR_VERSION_CUTOVER=YES
LEGAL_APPROVED=NO
PUBLIC_PSEUDONYM_CONFIRMED=YES
PAID_SERVICES_AUTHORIZED=NO
```

Technical checks accept a pending candidate and report its approval status
without changing it. Here the owner supplied publication permission separately;
a successful verification alone never supplies that permission.
The active root license is unchanged; no new licensing policy is activated.
Linux, Windows and macOS CI results belong to their exact tested SHA and are
available in [GitHub Actions](https://github.com/Gelram-project/gel-ram/actions).
The local validation fields above are not claims about subsequent CI runs.

New scope: bounded phrase extraction in gel-source, Unicode tests, a pinned
real-document demo and reproduction instructions. No private application code,
encrypted vault, speaker, encoder, private corpus or user data was exported.
Current results: [local package validation](docs/PACKAGE-VALIDATION.md).

`V0_3_0_TAGGED_RELEASE=PUBLISHED` records that the annotated `v0.3.0` tag and GitHub Release have been created from the exact green public-main commit selected for the release. The public `main` branch may advance after that immutable release point through separately verified changes.

`LEGAL_APPROVED=NO` means no claim is made that independent legal counsel has certified the custom license. It does not withdraw the project owner's publication authorization.

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
