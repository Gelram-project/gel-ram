# GEL RAM release status

2026-09-13. The project owner has authorized the current public source and media line under the public pseudonym **RR — GEL RAM Project**.

```text
REVIEW_PUBLICATION_APPROVED=YES
PUBLICATION_APPROVED=YES
ACTIVE_PUBLIC_BRANCH=main
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

`REVIEW_PUBLICATION_APPROVED=YES` is retained as a compatibility marker for the repository verification gate. The publication review was completed for the public line; it does not mean the already-active root license has reverted to a staged state.

`V0_3_0_TAGGED_RELEASE=PUBLISHED` records that the annotated `v0.3.0` tag and GitHub Release have been created from the exact green public-main commit selected for the release. The public `main` branch may advance after that immutable release point through separately verified changes.

`LEGAL_APPROVED=NO` means no claim is made that independent legal counsel has certified the custom license. It does not withdraw the project owner's publication authorization.

## Included public scope

1. Rust public core, multipart source readout, integrity and reproduction tools, pinned source fixtures and dependency/third-party inventory.
2. GEL RAM NCRL 1.0 as the operative root public license for GEL RAM-owned material, plus the commercial licensing path and CLA 2.0.
3. Two English terminal demonstration films, previews and six extracted screenshots, with scope and timing limitations in [the film guide](media/FILMS-GUIDE.md).
4. Public media reuse terms in [media/RIGHTS.md](media/RIGHTS.md).

## Excluded private scope

The private application, speaker, private banks of knowledge, private encoder and Ocean/P2P internals; user conversations, signed agreements, keys, personal identity records and local diagnostic logs are not part of the public distribution.

The films are bounded native Rust source-frame demonstrations. They do not prove general AI, semantic accuracy, unrestricted language generation, fourfold independent storage, P2P operation or physical DRAM-refresh synchronization.

## Licensing status

The root [LICENSE](LICENSE) is operative for GEL RAM-owned material on public `main`. [LICENSING.md](LICENSING.md) explains the selected policy. Historical release grants remain historical grants and are not rewritten by a later version.

The directory `docs/licensing-next/` is retained as historical proposal material only. Its staged documents are not an additional active license and do not override the root license.

Third-party material remains subject to its own terms. The redistributed Rust Book excerpt remains MIT-licensed.

## Verification

The v0.3.0 release target passed the configured CI before publication. Linux verification included complete source-manifest validation, `xtask verify` and the independent byte/numeric/ranking audit. Windows and macOS executed workspace tests and configured runtime checks. Subsequent public-main changes are verified independently and do not move the published `v0.3.0` tag.

A green technical gate is evidence for its stated scope only. It is not legal certification, ownership proof or a guarantee of security for every environment.

Licensing and contract enquiries: **gelram.licensing@gmail.com**. Completed agreements and personal records must remain private.
