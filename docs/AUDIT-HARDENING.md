# Public hardening audit

Review baseline: `9e1b63dc847875a44f78241f10f46f1791835be0`.
This repair is separate from release approval. It changes no license grant,
private implementation, historic media bytes or retained measurement data.

## Closed with regression tests

| Finding | Correction | Regression |
|---|---|---|
| Permission scanning missed quoted values, extra spaces and folded scalars; the binary workflow exception was not bound to a job. | A bounded, fail-closed workflow profile checks actual root/job permission maps and the release job's dispatch guard. Unsupported YAML forms are rejected, not guessed. | Quoted/escaped/spaced/folded writes; shorthand and flow maps; duplicate mappings; aliases; root and wrong-job writes; spoofed declarations in scripts/comments. |
| A benchmark could report SAME from equal counts and a contained capped preview while unshown lines differed. | Enumerate every matching line outside timing and compare complete sorted inventories; retain the tools' CR/LF differences. | A contained incomplete preview fails; divergent unseen lines fail; forty matches are all inventoried. |
| A zero repetition count selected answers-only, and an unbounded count could overflow loop bounds. | Explicit options and repetition bounds 1 through 10000; zero and invalid values are refused. | Zero, negative, huge, malformed and valid counts. |
| A hash-producing process could fail but have its output accepted. | Require both successful exit and the expected snapshot digest. | A nonzero process exit with a correct-looking digest is rejected; extra digest suffixes and invalid UTF-8 fail. |
| Completed timing observations were held in memory until the end, so a later process error lost earlier samples. | Write the corpus manifest before comparison and append every completed sample outside timing. A failed sample write stops the run. | The journal contains a completed CSV row before another process starts; failed writes do not enter the accepted sample set. |
| Workflow, corpus and report directory scans dropped enumeration errors. | Shared sorted inventory propagates every error. | First, middle and final iterator errors all fail closed. |
| Report hashing silently ignored symlinks and could emit ambiguous paths. | Reject non-regular entries, paths outside the root, non-UTF8 paths and control characters. | Complete file hashes; wrong root; Unix link rejection separately declared in the platform report. |
| IPv6 proc-file read errors were treated as absent routing state. | Only NotFound denotes an absent optional file. All other errors prevent isolation verification. | PermissionDenied, InvalidData and Interrupted do not become empty state. |
| The roadmap summary disagreed with its 24 rows and called open work delivered. | Correct the wording/counts and check IDs, states and the computed summary in verify. | Missing, repeated, out-of-range and unknown rows; stale/missing/duplicate summaries. |
| Package CI filters omitted workspace metadata, legal notices and helpers. | Packaging checks run on every push and pull request. | Configuration inspection and actual package jobs; write permissions stay confined to the existing manually started release job. |

The benchmark also refuses missing, duplicated or malformed timed search records
instead of silently truncating a zip of queries and returned observations.

The local CLA acknowledgement check now requires the same complete checkbox line
as the workflow, rather than accepting that sentence as an embedded substring.
This verifies an acknowledgement only, not identity, authority or a signed CLA.

## Workflow policy boundary

[Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
is the upstream contract. The local checker intentionally accepts a smaller
profile: ASCII unquoted mapping keys, even space indentation, block jobs and
permissions, and simple scalar permission values. It rejects aliases, anchors,
tags, quoted keys, duplicate mappings and flow-style permission/job definitions.
This avoids claiming a general YAML parser without one. Script block contents
are not permission declarations. Actions must be pinned to full lowercase
40-character commit IDs. Root permissions must be explicit and non-writing;
only the actual binaries release job guarded by manual dispatch may request
id-token and attestations writes. This does not certify the safety of arbitrary
scripts, upstream actions, runner administration or modifications to the checker.

## Evidence and acceptance

The audit source was transferred through a read-only CI artifact and verified
against its published artifact digest and the complete source manifest. Native
Rust 1.85.0 checks run in an unprivileged Linux sandbox using a separately
prepared offline vendor directory. Sandbox Git identifiers denote a local copy,
not the upstream commit; the baseline above and manifest retain that mapping.
Exact-head GitHub CI remains a separate required acceptance step.

Historic benchmark r1 used the old preview-based agreement check. Its raw files
are preserved, not relabelled as having passed the stronger comparison. The
original GIFs remain edited replays at their original source revision.

## Supplemental format campaign

Two fixed-seed tests exercise 20,000 bounded mutants each of GELSET01 and
Q8DEMO01. They cover byte replacement, bit flips, truncation, append, deletion
and bounded zero/0xFF spans. A changed collection must fail its old pin; every
accepted parse must re-encode byte-identically. Both accepted valid changes and
rejections must occur. These 40,000 cases complement, not replace, the existing
179-mutant format matrix; they are not exhaustive coverage or proof of a bug-free parser.

## Still open, not repaired by relabelling

Independent operator/second-host acceptance, full human film review, Windows
power-loss directory durability, identical-task performance comparisons and
research mechanisms outside the approved public scope remain separate work.
An inspection and finite regression suite cannot establish absence of all bugs,
complete parser coverage, legal enforceability or production readiness.
