# GEL RAM v0.2 architecture

The public core is Rust-only. The v0.2.1 binary branch below remains unchanged;
main also includes an unreleased, separate Q8 phase-readout module.

```text
QUERY
  |
  +-> binary ORB bank
  |      |
  |      +-> exact POPCOUNT/XNOR
  |      +-> exact progressive 32 -> 64 -> 128 B scoring bounds
  |      +-> Reader16 fused judgments
  |
  +-> optional structural context
         |
         +-> prototype / segment-local parent
         +-> XOR residual
         +-> exact ORB reconstruction

ORB bytes -> v2 .gel store -> header CRC64 + payload CRC64 -> RAM
```

## Additive Q8 phase branch (unreleased)

The [Q8 module](Q8-QUAD.md) holds 1024 phase bytes and 128 activity-mask bytes
per canonical record. A query–record comparison produces one canonical
reduction that preserves the scores of four matching reversible coordinate
views. Archive and BodyActivity have distinct mask semantics; see the contract.

This is separate from the 128-byte binary ORB and v2 store shown above: no Q8
persistence format or automatic text-to-phase encoder is added. Shared
scoring only applies to equivalent views of the same contents, not four
different records. The public module does not implement a physical analog
wave circuit or native knowledge inference. Worker budgets are bounded per
call; serial fallback handles worker-start refusal, not every panic or OOM.

## Two axes of progressive work

The architecture deliberately separates:

- **narrowing candidates**: locator/sketch/Top-K work planned for later scale milestones,
- **deepening one ORB**: 32 B -> 64 B -> 128 B -> structural reconstruction.

v0.2 implements the exact progressive bounds for the second axis. It does not pretend that a `Vec<Orb1024>` layout avoids loading adjacent cache-line data; the physical layout experiment is an F0/F3 measurement problem.

## Geometry versus information

Reversals, rotations, XOR masks and permutations are reversible coordinate changes. Applied symmetrically to query and candidate, they preserve Hamming/XNOR. They remain useful layout/operator primitives but do not count as independent information views.

Reader16 instead exposes global asymmetric relations and disjoint local subspaces from a single comparison.

## Structural exact coding

A structural delta is allowed only when it beats literal storage after metadata is counted. Delta chains are hard-bounded to depth two. Parent identity is represented as either:

- prototype-pool index (`u32`), or
- segment-local index (`u16`).

The public codec does not yet prescribe a global graph or prototype-pool size. Those are workload/topology decisions and must be chosen from F0/F2 measurements.
