# Security

## Fix policy

Fixes, when made, land only on the current 0.2.x line. No fix, response time or support is promised.

## Attack surface

- The `.gel` parser in `gel-store`. `.gel` files are untrusted input.
- Command-line arguments: `gel-cli` takes a fixed command word and a file path; `gel-bench` and `gel-physics` take unsigned integers.
- `gel-cli verify` streams the payload through a fixed 128-byte buffer and holds no payload in memory.
- The `gel-cli` selftest creates a private directory in the OS temporary directory, writes its store there through a sibling temporary file renamed into place, and removes the directory afterwards (best effort).
- `write_atomic` writes a sibling `<name>.tmp-<pid>-<n>` file created with O_EXCL semantics and renames it into place. On Unix a new store is mode `0600`; replacement preserves the existing mode. `write_if_newer` assumes a single writer and is not a compare-and-swap.

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
- No network code.
- Bounded open API: `open_verified_with_limits` rejects a file above the caller's file-size budget before reading the header, and a record count above the record budget before allocation.
- CRC64-ECMA on the v2 header and on the payload.
- Exact length checks before allocation: the declared record count must match the file length exactly before the record buffer is reserved; the reservation is fallible and returns an error instead of aborting.

## Experimental Q8 resource boundary

The additive phase-Q8 module does not add a `.gel` parser, network endpoint
or text encoder. Its worker budget is bounded per call by the requested
budget, available parallelism, a cap of 64 and the bank length, including
the caller. It is not a global limit across simultaneous readers.

The shared reader and V2 comparison references handle worker-start refusal
by joining started workers and recomputing the full output serially.
This does not certify general out-of-memory or worker-panic recovery.
Applications still need their own aggregate memory/concurrency limits.
The [Q8 contract](docs/Q8-QUAD.md) and [refusal regression tests](docs/Q8-QUAD-VALIDATION.md)
describe the measured boundary; no general denial-of-service protection is claimed.

## Reporting

Use GitHub Private Vulnerability Reporting where it is enabled for this
repository; otherwise contact `gelram.licensing@gmail.com`. Do not disclose an
unpatched vulnerability in a public issue.
## Candidate Q8 compatibility boundary

Checked in-memory views reject unsupported descriptors and another reader's seed.
They do not authenticate payloads or metadata: a relabelled malicious view may
still restore different data. Use trusted manifests/signatures at trust boundaries.
Legacy low-level Grid/View APIs remain available and are not automatically guarded.
The numeric fixture tool reads bounded data, never evaluates it, and has no network
or model integration. See [candidate scope](docs/Q8-EVIDENCE-CANDIDATE.md).
