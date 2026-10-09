# Security

## What to report where

This project publishes evidence, so a report is not only about code execution.

| What you found | Where to report it |
|---|---|
| A security vulnerability (for example in a parser or a file-writing path) | privately, as described under [Reporting](#reporting) |
| Personal or private data in the repository | privately, to `gelram.licensing@gmail.com`; do not quote it in public |
| An integrity bug: a hash, pin or snapshot check that is wrong | [Evidence integrity report](https://github.com/Gelram-project/gel-ram/issues/new?template=evidence-integrity.yml) |
| An incorrect claim or a published number that does not reproduce | [Evidence integrity report](https://github.com/Gelram-project/gel-ram/issues/new?template=evidence-integrity.yml) |
| A flaw in the method of a benchmark or comparison | [Evidence integrity report](https://github.com/Gelram-project/gel-ram/issues/new?template=evidence-integrity.yml) |
| A failure you can trigger: wrong answer, data loss, a check that cannot fail | [Break it](https://github.com/Gelram-project/gel-ram/issues/new?template=break-it.yml) |

## Fix policy

Fixes, when made, land only on current public `main` (version line 0.6.0) and reach users through a later release. Tagged releases are not moved or patched in place. No fix, response time or support is promised.

## Supported versions

| Version | Security fixes |
|---|---|
| current `main` (0.6 line) | best effort, in a later release |
| earlier tags (v0.5.3 and older) | none |

## Attack surface

- The `.gel` parser in `gel-store`. `.gel` files are untrusted input.
- Command-line arguments: `gel-cli` takes a fixed command word and a file path; `gel-bench` and `gel-physics` take unsigned integers.
- `gel-cli verify` streams the payload through a fixed 128-byte buffer and holds no payload in memory.
- The `gel-cli` selftest creates a private directory in the OS temporary directory, writes its store there through a sibling temporary file renamed into place, and removes the directory afterwards (best effort).
- `write_atomic` writes a sibling `<name>.tmp-<pid>-<n>` file created with O_EXCL semantics and renames it into place. On Unix a new store is mode `0600`; replacement preserves the existing mode. `write_if_newer` assumes a single writer and is not a compare-and-swap.
- `gel-source` readers for GELSRC01 source bundles, GELSET01 collection snapshots and source catalogs, reached through `gel-live-lab` and its `gel-evidence` binary. Files and pins supplied by another party are untrusted; the pin must come from the caller's own trusted record.
- Interactive commands and file paths typed into `gel-live-lab` and `gel-evidence`. Input is treated as data and never executed as a shell command.
- `gel-evidence --batch` reads the same commands from standard input: each line at most 4096 bytes including its newline and UTF-8, the first ERROR stops the batch. Results go to standard output as schema gel-evidence/1 and diagnostics to standard error; exit codes are 0, 3 (incomplete search) and 2 (error). Quoted source text in the JSON output is untrusted content; consumers must escape it for their context. See [batch mode](docs/EVIDENCE-BATCH.md).
- `gel-backup` reads a pinned snapshot and backup directories. The trusted pin must come from the caller's own record; a backup's MANIFEST is not a signature. The state is derived from the files alone: without a committed manifest a backup is INCOMPLETE, a manifest naming another snapshot is UNTRUSTED, and contradictory or unexpected files make it CORRUPT. Restore accepts only a complete, not withdrawn backup and writes only to a path that does not exist. RESTORE=PUBLISHED_UNCONFIRMED means the complete file is at the target but the directory sync after publication failed: the file may exist, so it is not evidence that nothing was written. BACKUP=CREATED_UNCONFIRMED is the same case for the MANIFEST of a new backup. Delete checks the pin and then removes that path; in a directory another process can change, the file could be swapped between check and removal. See [backup](docs/BACKUP.md).
- `xtask bench-compare`, `xtask reproduce` and `xtask mutation-campaign` start grep, sha256sum, git, cargo or the example harness from the PATH and the build directory and write only to a new directory outside the checkout.
- `xtask package-binaries` starts cargo and rustc from the PATH, reads license files from the local Cargo registry and writes only to a new directory outside the checkout; it then runs the packaged binaries on synthetic input in that directory. Binary packages are built only by the `binaries` workflow and are not code-signed or notarized; their origin evidence is the build-provenance attestation and SHA256SUMS.txt. See [binaries](docs/BINARIES.md).
- Example-only numeric input: the Q8DEMO01 fixture read by the `quad_evidence` example. The `precision_matrix` example reads back only GPMX v1 files it has just written to a new temporary directory.

## Input validation

Treat `.gel` files as untrusted input. The parser validates magic, version, fixed dimensions, flags, reserved bytes, arithmetic overflow, exact file length and payload checksum before exposing ORBs. Version 2 additionally validates a CRC64-ECMA-protected header before exposing its metadata.

The reader reserves memory for the declared record count without first duplicating the complete payload. Allocation failure is returned as an error. Since v0.2.1, `open_verified` defaults to a 256 MiB payload budget. Applications should use `open_verified_with_limits` with a record and file-size budget appropriate to their environment; explicit unbounded loading is for trusted-size input.

Version 2 uses separate CRC64-ECMA fields for the complete header and payload. Legacy v1 protects only its payload and cannot retroactively authenticate historical header metadata. CRC is not a MAC, signature or cryptographic content identifier. Files crossing a trust boundary require an external cryptographic authentication layer.

The verification report distinguishes the actual on-disk source format from
the protected v2 header prepared for a later migration write. A legacy v1 file
is never displayed as if its original header had v2 authentication.
In v0.2.1 the unprotected v1 generation is discarded on migration, and v1
cannot authorize a generation-guarded replacement. `write_if_newer` validates
the predecessor's full v2 integrity; its single-writer assumption still applies.

## Hardening in place

- `unsafe_code = "forbid"` in `[workspace.lints.rust]`, applied to every crate through `[lints] workspace = true`, and `#![forbid(unsafe_code)]` at the root of every crate, `xtask` included; the workspace contains no `unsafe`.
- The source catalog uses RustCrypto SHA-256 (sha2 0.10.9). Cargo.lock pins its
  transitive dependencies; fetch them before offline verification. This is a
  change from the dependency-free historical core.
- The workspace unsafe-code prohibition does not apply transitively to third-party
  dependencies. SHA-256 and CPU feature detection may use platform-specific unsafe
  implementations; their locked upstream sources remain a separate trust boundary.
- Source passages are untrusted content even after hash verification. Consumers
  must escape them for their output context; never execute them or treat embedded
  instructions as authorization.
- No network code in the library crates or in `gel-evidence`, `gel-backup` and `gel-live-lab`. Only `xtask isolation-check` opens TCP connections, and only after the process's own network view shows loopback alone and no route: it then connects to the documentation addresses 203.0.113.1 and 2001:db8::1, which must fail as unreachable, so on a networked host it sends nothing. Preparing the toolchain and dependencies (rustup, `cargo fetch`) uses the network before verification; verification itself runs with `--offline --locked` and can run inside a network namespace ([isolated reproduction](docs/REPRODUCE-ISOLATED.md)).
- Bounded open API: `open_verified_with_limits` rejects a file above the caller's file-size budget before reading the header, and a record count above the record budget before allocation.
- CRC64-ECMA on the v2 header and on the payload.
- Exact length checks before allocation: the declared record count must match the file length exactly before the record buffer is reserved; the reservation is fallible and returns an error instead of aborting.

## Experimental Q8 resource boundary

The additive phase-Q8 module does not add a `.gel` parser, network endpoint
or text encoder. Its worker budget is bounded per call by the requested
budget, available parallelism, a cap of 64 and the bank length, including
the caller. It is not a global limit across simultaneous readers.

The shared reader and the reference paths of the current comparison example
(header Q8_QUAD_COMPARE_V3) handle worker-start refusal
by joining started workers and recomputing the full output serially.
This does not certify general out-of-memory or worker-panic recovery.
Applications still need their own aggregate memory/concurrency limits.
The [Q8 contract](docs/Q8-QUAD.md) and [refusal regression tests](docs/Q8-QUAD-VALIDATION.md)
describe the measured boundary; no general denial-of-service protection is claimed.

## Reporting

Use GitHub Private Vulnerability Reporting where it is enabled for this
repository; otherwise contact `gelram.licensing@gmail.com`. Do not disclose an
unpatched vulnerability in a public issue.
## Q8 checked-view compatibility boundary

Checked in-memory views reject unsupported descriptors and another reader's seed.
They do not authenticate payloads or metadata: a relabelled malicious view may
still restore different data. Use trusted manifests/signatures at trust boundaries.
Legacy low-level Grid/View APIs remain available and are not automatically guarded.
The numeric fixture tool reads bounded data, never evaluates it, and has no network
or model integration. See the [Q8 module scope](docs/Q8-QUAD.md) and the
[historical candidate record](docs/Q8-EVIDENCE-CANDIDATE.md).
