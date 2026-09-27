# Binary packages of the Evidence Lab tools

Ready-to-run packages of `gel-evidence`, `gel-backup` and `gel-live-lab` let
someone try the Evidence Lab without installing Rust. They are built only by
the [binaries workflow](../.github/workflows/binaries.yml) on GitHub-hosted
runners, from a public revision, never on a developer machine.

| Package | Built on | Runs on |
|---|---|---|
| `gel-VERSION-x86_64-unknown-linux-gnu.tar.gz` | Ubuntu 22.04 runner | 64-bit x86 Linux with glibc; checked only on the build runner |
| `gel-VERSION-aarch64-apple-darwin.tar.gz` | macOS runner (Apple silicon) | Apple silicon Macs; checked only on the build runner |
| `gel-VERSION-x86_64-pc-windows-msvc.zip` | Windows runner | 64-bit x86 Windows; checked only on the build runner |

Pushes and pull requests that touch the tools, the dependency lock or this
workflow build and check all three packages with a read-only token and publish
nothing. A package is archived, given a signed build-provenance attestation and
uploaded as a workflow artifact only when the owner starts the workflow by hand.
Attaching the archives to a release is a separate owner decision.

## What a package contains

| Path | Content |
|---|---|
| `gel-evidence`, `gel-backup`, `gel-live-lab` (`.exe` on Windows) | the three programs, release profile, symbols stripped |
| BUILD-INFO.txt | revision and working-tree state, target, rustc and cargo versions, runner image, each third-party crate, each binary's size and SHA-256, the results of the checks below |
| SHA256SUMS.txt | SHA-256 of every other file in the package |
| `LICENSE`, `NOTICE`, `LICENSE-MODE.txt`, `LICENSING.md`, `COMMERCIAL-LICENSE.md` | the project's terms, unchanged; the packages are under the same license as the source |
| THIRD-PARTY-NOTICES.md, DEPENDENCY-INVENTORY.md | the third-party inventory with the reviewed license-file hashes |
| `licenses/NAME-VERSION/` | the license files of every third-party crate in the target's dependency graph, copied from the exact locked crate |
| `licenses/rust-VERSION/` | `COPYRIGHT`, `LICENSE-APACHE` and `LICENSE-MIT` of the Rust toolchain whose standard library is linked in |

## What the package check refuses

`cargo run --locked --offline -p xtask -- package-binaries NEW_DIR` builds and
assembles the package. BUILD-INFO.txt and SHA256SUMS.txt are written last, so a
refused package has neither. It is refused when:

- a third-party crate in the target's normal dependency graph is missing from
  the [reviewed inventory](DEPENDENCY-INVENTORY.md), or one of its license files
  is missing or differs from the recorded SHA-256;
- the toolchain's license files are missing;
- a binary contains the build workspace path, the Cargo home, the home
  directory or a user-directory prefix (build paths are remapped at compile time);
- a smoke run of the packaged binaries on that platform fails: help and both
  demos, a batch session (add, three finds with HIT, HIT and UNKNOWN, save,
  exit), a backup create, inspect and restore with a byte-identical result, and
  a restore with a wrong pin that is refused without writing a file.

`release_eligible=yes` in BUILD-INFO.txt requires a build in GitHub Actions from
a clean working tree; the workflow fails otherwise.

## Checking a downloaded package

```sh
gh attestation verify gel-VERSION-TARGET.tar.gz --repo Gelram-project/gel-ram
tar -xzf gel-VERSION-TARGET.tar.gz
cd gel-VERSION-TARGET && sha256sum -c SHA256SUMS.txt
```

On macOS use `shasum -a 256 -c SHA256SUMS.txt`; on Windows compare
`Get-FileHash -Algorithm SHA256` with the listed values. The attestation ties
the archive to the workflow run and revision that built it; `revision=` in
BUILD-INFO.txt names the source to compare with.

## Limits

- The binaries are not code-signed or notarized. macOS quarantines downloaded
  unsigned programs and Windows SmartScreen may warn; the attestation and
  checksums are the evidence of origin, not an operating-system signature.
- Each package is checked only on its build runner. Older Linux distributions
  (glibc older than Ubuntu 22.04's), Intel Macs and Windows on ARM are not
  built or tested.
- The system C runtime (glibc, libSystem, the Microsoft C runtime) is linked
  dynamically and is not shipped; Windows needs the Microsoft Visual C++ runtime.
- Crates inside the Rust standard library are covered by the toolchain's
  COPYRIGHT file and are not inventoried one by one.
- Byte-for-byte reproducible builds are not claimed.
- A passing package check is a technical gate, not legal approval of the
  redistribution; the upstream license texts in the package govern.
- The programs themselves are the same as in the source: offline, no network
  code, no language model; see [security](../SECURITY.md).
