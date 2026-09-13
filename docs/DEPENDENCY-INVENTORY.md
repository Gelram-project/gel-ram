# Exact external dependency inventory

Review snapshot, 2026-09-13. These are the 10 external packages in Cargo.lock,
including transitive/target-specific/build dependencies, not just the host subset.
Names, versions and declared license expressions come from locked offline Cargo
metadata. Each cached .crate SHA-256 was compared to Cargo.lock. The license-file
hashes below were also checked against the files extracted from those archives.
No dependency source or binary is vendored by this source-only candidate.

This records provenance, not a vulnerability scan or legal approval. The full
upstream license text governs; a binary/vendored release needs its own notices
review. The historical metadata spelling `MIT/Apache-2.0` is preserved verbatim.
Toolchain, OS libraries and CI actions are not Cargo dependencies and are not
included in these ten rows. CI action revisions are pinned in the workflow.

| Package | Version | Declared license | Upstream |
|---|---|---|---|
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | [repository](https://github.com/RustCrypto/utils) |
| cfg-if | 1.0.4 | MIT OR Apache-2.0 | [repository](https://github.com/rust-lang/cfg-if) |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | [repository](https://github.com/RustCrypto/utils) |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | [repository](https://github.com/RustCrypto/traits) |
| digest | 0.10.7 | MIT OR Apache-2.0 | [repository](https://github.com/RustCrypto/traits) |
| generic-array | 0.14.7 | MIT | [repository](https://github.com/fizyk20/generic-array.git) |
| libc | 0.2.189 | MIT OR Apache-2.0 | [repository](https://github.com/rust-lang/libc) |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | [repository](https://github.com/RustCrypto/hashes) |
| typenum | 1.20.1 | MIT OR Apache-2.0 | [repository](https://github.com/paholg/typenum) |
| version_check | 0.9.5 | MIT/Apache-2.0 | [repository](https://github.com/SergioBenitez/version_check) |

## Archive and license-file SHA-256

Names in the blocks are paths within the named upstream archive, not GEL files.

### block-buffer 0.10.4

```text
3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71  block-buffer-0.10.4.crate
a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5  block-buffer-0.10.4/LICENSE-APACHE
d5c22aa3118d240e877ad41c5d9fa232f9c77d757d4aac0c2f943afc0a95e0ef  block-buffer-0.10.4/LICENSE-MIT
```

### cfg-if 1.0.4

```text
9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801  cfg-if-1.0.4.crate
a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2  cfg-if-1.0.4/LICENSE-APACHE
378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397  cfg-if-1.0.4/LICENSE-MIT
```

### cpufeatures 0.2.17

```text
59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280  cpufeatures-0.2.17.crate
a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5  cpufeatures-0.2.17/LICENSE-APACHE
ae9baa7beea910273c2f384c2a6b721fb7bd02bda3436074a1072e4ee689f985  cpufeatures-0.2.17/LICENSE-MIT
```

### crypto-common 0.1.7

```text
78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a  crypto-common-0.1.7.crate
a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5  crypto-common-0.1.7/LICENSE-APACHE
3521672491a3479422d5fe1aca6645dd2984090f85da6e5205abfb18fb7a6897  crypto-common-0.1.7/LICENSE-MIT
```

### digest 0.10.7

```text
9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292  digest-0.10.7.crate
a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5  digest-0.10.7/LICENSE-APACHE
9e0dfd2dd4173a530e238cb6adb37aa78c34c6bc7444e0e10c1ab5d8881f63ba  digest-0.10.7/LICENSE-MIT
```

### generic-array 0.14.7

```text
85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a  generic-array-0.14.7.crate
c09aae9d3c77b531f56351a9947bc7446511d6b025b3255312d3e3442a9a7583  generic-array-0.14.7/LICENSE
```

### libc 0.2.189

```text
3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2  libc-0.2.189.crate
62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a  libc-0.2.189/LICENSE-APACHE
123a331b5dbf04c30097fa43b8f858bc85df671fe776de498d01f3d6b7c1f69e  libc-0.2.189/LICENSE-MIT
```

### sha2 0.10.9

```text
a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283  sha2-0.10.9.crate
a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5  sha2-0.10.9/LICENSE-APACHE
b4eb00df6e2a4d22518fcaa6a2b4646f249b3a3c9814509b22bd2091f1392ff1  sha2-0.10.9/LICENSE-MIT
```

### typenum 1.20.1

```text
b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20  typenum-1.20.1.crate
516b24e051bf5630880ebbd55c40a25ce9552ebaf8970a53e8976eb70e522406  typenum-1.20.1/LICENSE-APACHE
db11fec9946737df39ca3898d9cd8c10ec6f6c3a884a6802b0ad0b81b4e8f23a  typenum-1.20.1/LICENSE
a825bd853ab71619a4923d7b4311221427848070ff44d990da39b0b274c1683f  typenum-1.20.1/LICENSE-MIT
```

### version_check 0.9.5

```text
0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a  version_check-0.9.5.crate
a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2  version_check-0.9.5/LICENSE-APACHE
b7e650f3fce5c53249d1cdc608b54df156a97edd636cf9d23498d0cfe7aec63e  version_check-0.9.5/LICENSE-MIT
```

## Other redistributed third-party material

The MIT Rust Book excerpt is identified separately in [real-source provenance](REAL-SOURCE-DEMO.md), with its full license beside the fixture. It is not a Cargo dependency. GEL-owned material remains under the root license; this inventory neither changes that license nor licenses third-party material on GEL terms. Recheck the inventory whenever Cargo.lock or distributed fixtures change.

