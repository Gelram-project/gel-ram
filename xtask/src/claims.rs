//! One registry for scoped claims, executable counterexamples and explicit gaps.
use gel_source::{collection::Collection, context, digest};
use std::{collections::BTreeSet, fs, path::Path};

struct Claim {
    id: &'static str,
    dimension: &'static str,
    scope: &'static str,
    input: &'static str,
    expected: &'static str,
    counterexample: &'static str,
    source: &'static str,
    evidence: Evidence,
}
enum Evidence {
    Probe(fn() -> Result<bool, String>),
    Deferred(&'static str),
}

/// The only open states a row may carry; none of them is a PASS.
const DEFERRED_MODES: &[&str] = &[
    "SEPARATE_GATE",
    "MEASURED_LOCAL",
    "NOT_VERIFIED",
    "NOT_ESTABLISHED",
];

const CLAIMS: &[Claim] = &[
    Claim {
        id: "gel-resident-addressed-read",
        dimension: "timing",
        scope: "separate private implementation, known address, 40 observations",
        input: "author-retained addressed read timing series after RAM load",
        expected: "p50 53.872 us, p95 79.640 us; no global search or semantic decision claim",
        counterexample: "addressed lookup latency presented as full Ocean search latency",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-source-integrity-controls",
        dimension: "integrity",
        scope: "separate private implementation, 1000 PL/EN fragments, five controls each",
        input: "valid, corrupt, missing, wrong-source and stale-generation cases",
        expected: "1000 valid admitted and 4000 invalid rejected; no semantic correctness inferred",
        counterexample: "matching source bytes treated as proof of a claim or general understanding",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-single-quad-slot-ranking",
        dimension: "ranking",
        scope: "separate private implementation, known 250k slot of logical 1M bank, 400 probes",
        input: "same probes for Single and Quad, including one empty probe in quality denominator",
        expected: "Single top1 368/400 and top10 393/400; Quad 310/400 and 361/400",
        counterexample: "slot ranking described as global million-record semantic recall or Quad superiority",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-slot-ranking-update",
        dimension: "ranking",
        scope: "separate private implementation, same 400 probes and known 250k slot, updated build",
        input: "same probes for Single and Quad; two private pipeline changes between runs",
        expected: "Single top1 379/400 and top10 397/400; Quad 371/400 and 396/400",
        counterexample: "gain attributed to one change, or slot ranking presented as global 1M recall",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-source-field-dialogue-timing",
        dimension: "timing",
        scope: "separate private implementation, dialogue pilot on five articles, 13 questions, three runs of 1000 warm repeats",
        input: "resident source fields; loading, display, report writes and fallback excluded",
        expected: "field p50 2.054-4.819 us, comparison p50 19.417-21.220 us, slowest 711.017 us",
        counterexample: "medians presented as worst case, or UNHANDLED counted as an answer",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-new-article-transfer",
        dimension: "retrieval",
        scope: "separate private implementation, dialogue on four new PL articles, 13 questions, no timing",
        input: "69 source fields; six field outputs checked against the source HTML cells",
        expected: "6 field answers, 2 quotations, 1 limited comparison, 3 UNHANDLED, 1 UNKNOWN",
        counterexample: "result counted as 9/13 semantic accuracy or a million-record deployment",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-answer-verdict-self-read",
        dimension: "retrieval",
        scope: "separate private implementation after three measured changes; answer bank of 671,416 passages (167,854 per slot); 50,000 stored-passage probes ranked within their slot",
        input: "verdict lead threshold fixed in advance from another corpus; a variant of the private build chosen on a separate sample",
        expected: "46,376 of 50,000 answered (92.8%): 46,353 correct (99.95% of answers) and 23 wrong (0.046% of probes)",
        counterexample: "self-read precision presented as natural-question accuracy, global 1M search or an unchanged bank",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-natural-question-answers",
        dimension: "retrieval",
        scope: "80 PL/EN questions (question set v1) written by the project's AI coding assistant, frozen before the run, all four slots of the 671,416-passage answer bank searched",
        input: "source verification of the best candidates; threshold from a separate 80-question calibration set",
        expected: "top-1 right article 40/80; 11 answers, all correct; 69 UNKNOWN",
        counterexample: "11 of 11 reported as a precision rate or the set treated as an independent benchmark",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-answer-set-v2",
        dimension: "retrieval",
        scope: "394 frozen PL/EN questions drawn at random (answer_or_abstain_v2), one run of the private build from its bank",
        input: "questions and accepted spellings fixed by SHA-256 before the run; one scoring rule; the manual review listed",
        expected: "GEL 63 answers, 59 correct, 4 wrong, 331 UNKNOWN",
        counterexample: "read as GEL never answering wrongly, or as independent of the question writer",
        source: "docs/answer-or-abstain-v2/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "store-kill-during-learning",
        dimension: "persistence",
        scope: "separate private implementation under continuous writes: 100 + 100 random kills in two series and 7 fixed stop points",
        input: "SIGKILL at a seeded random time or at a fixed point; reopen and check, then resume",
        expected: "0 of 235,712 confirmed records lost; every reopen check passed; every resume equal to the run without a kill",
        counterexample: "read as power-loss durability, as a large-store result, or as reproducible from this checkout",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-bm25-v3",
        dimension: "comparison",
        scope: "979 frozen PL/EN questions drawn at random (answer_or_abstain_v3): GEL beside Tantivy BM25 and SQLite FTS5 on the same bank of 671,416 passages; all four GEL runs on the set published",
        input: "questions, accepted spellings, plan and one selection rule fixed by SHA-256 before any run; settings of every system chosen on v1 + v2; one scoring rule for all; the manual review listed",
        expected: "GEL 191 answers, 180 correct, 11 wrong; BM25 engines with thresholds 238-241 correct, 28 wrong each; paired: engines more correct, GEL fewer wrong",
        counterexample: "read as GEL finding more answers than BM25, as a held-out result for the final build, or as a tuned BM25 baseline",
        source: "docs/answer-or-abstain-v3/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-bm25-v4",
        dimension: "comparison",
        scope: "985 new PL/EN questions (answer_or_abstain_v4), frozen before any run and written after the tested change and its settings were fixed on v1 + v2; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank",
        input: "one run per system; settings of every system chosen on v1 + v2 by one rule (precision >= 0.95 or >= 0.99); one scoring rule for all; the manual review listed",
        expected: "GEL 423 answers, 405 correct, 18 wrong (95.7%); the v3 build 172 correct, 17 wrong; engines at the same rule 531-558 correct, 40-41 wrong; at precision >= 0.99 the engines ahead",
        counterexample: "read as GEL finding more answers than BM25, as 99% precision, as a released build, or as a tuned BM25 baseline",
        source: "docs/answer-or-abstain-v4/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-bm25-v5",
        dimension: "comparison",
        scope: "987 new PL/EN questions (answer_or_abstain_v5), frozen before any run; the v4 build unchanged and one candidate change, both fixed on v1 + v2; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank",
        input: "one run per system; settings of every system chosen on v1 + v2 by one rule (precision >= 0.95 or >= 0.99); one scoring rule for all; the manual review listed",
        expected: "GEL 432 answers, 416 correct, 16 wrong (96.3%); precise setting 297 correct, 1 wrong; engines at the same rule 567-597 correct, 43-47 wrong; the candidate change 92.3%, below the 0.95 fixed in advance, rolled back",
        counterexample: "read as GEL finding more answers than BM25, as 99% precision across sets, as a released build, or as a tuned BM25 baseline",
        source: "docs/answer-or-abstain-v5/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-bm25-v6",
        dimension: "comparison",
        scope: "990 new PL/EN questions (answer_or_abstain_v6), frozen before any run and used internally before (an experiment and an analysis of its errors) without changing any build or setting; the v4 build unchanged, fixed on v1 + v2; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank",
        input: "one recorded run per system, GEL's answers equal to those of the earlier internal experiment; settings of every system chosen on v1 + v2 by one rule (precision >= 0.95 or >= 0.99); one scoring rule for all; the manual review listed",
        expected: "GEL 465 answers, 447 correct, 18 wrong (96.1%); precise setting 313 correct, 2 wrong; engines at the same rule 588-616 correct, 33-36 wrong",
        counterexample: "read as GEL finding more answers than BM25, as 99% precision across sets, as a released build, or as a tuned BM25 baseline",
        source: "docs/answer-or-abstain-v6/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-bm25-v7",
        dimension: "comparison",
        scope: "982 new PL/EN questions with an answer and 599 without one in the bank (399 about real topics outside it, 200 about invented subjects) (answer_or_abstain_v7), frozen before any run and not used before; the v4 build unchanged, fixed on v1 + v2, and one candidate change fixed before the draw; GEL beside Tantivy BM25 and SQLite FTS5 on the same bank",
        input: "one run per system; settings of the build and the engines chosen on v1 + v2 by one rule (precision >= 0.95 or >= 0.99), the candidate's two settings fixed before the draw; one scoring rule for all; the blind manual review listed",
        expected: "GEL 425 answers, 413 correct, 12 wrong (97.2%) and 26 answers to the 599 questions without one; precise setting 291 correct, 3 wrong, 3 answers without one; engines at the strict threshold 380-384 correct with 34-38 wrong in all; the candidate change 245 correct, 11 wrong in all, rolled back",
        counterexample: "read as GEL finding more answers than BM25 at the same rule, as 99% precision across sets, as proof that a topic is absent from the bank, as a released build, or as a tuned BM25 baseline",
        source: "docs/answer-or-abstain-v7/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "source-roundtrip",
        dimension: "bytes",
        scope: "GELSET01 source bytes",
        input: "decomposed Polish Unicode, CRLF and emoji",
        expected: "identical serialized bytes and root after reload",
        counterexample: "one flipped byte with original pin must be refused",
        source: "crates/gel-source/tests/collection.rs",
        evidence: Evidence::Probe(roundtrip),
    },
    Claim {
        id: "stale-citation",
        dimension: "provenance",
        scope: "one collection revision",
        input: "source hit followed by replace",
        expected: "old hit rejected, new hit accepted",
        counterexample: "accepting a citation from the old generation",
        source: "crates/gel-source/tests/collection.rs",
        evidence: Evidence::Probe(stale),
    },
    Claim {
        id: "hash-not-structure",
        dimension: "structure",
        scope: "GELSET01 header",
        input: "zero next-ID with freshly recomputed hash",
        expected: "structural refusal",
        counterexample: "acceptance based on matching SHA alone",
        source: "crates/gel-source/tests/collection.rs",
        evidence: Evidence::Probe(structure),
    },
    Claim {
        id: "bounded-context",
        dimension: "context",
        scope: "source context, not semantic understanding",
        input: "negation on preceding line and bounded Unicode padding",
        expected: "negation retained when in range; truncation flagged",
        counterexample: "hidden negation or an unflagged clipped span",
        source: "crates/gel-source/src/context.rs",
        evidence: Evidence::Probe(bounded_context),
    },
    Claim {
        id: "no-cross-document",
        dimension: "retrieval",
        scope: "phrase lookup",
        input: "not and approved in different documents",
        expected: "UNKNOWN for combined phrase",
        counterexample: "joining independent documents into one quote",
        source: "crates/gel-source/tests/collection.rs",
        evidence: Evidence::Probe(cross_document),
    },
    Claim {
        id: "numeric-loss",
        dimension: "numeric",
        scope: "public reference quantizers only",
        input: "quantization_matrix, precision_matrix Q1-Q16/F16 and small_signal_outliers",
        expected: "separate numeric error, metadata and loss report",
        counterexample: "byte roundtrip described as lossless F32 quantization",
        source: "docs/PRECISION-MATRIX.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "ranking-oracle",
        dimension: "ranking",
        scope: "public numeric reader, not semantic recall",
        input: "data_integrity reference ranking",
        expected: "rank agreement reported separately from byte checks",
        counterexample: "synthetic exact matches called general semantic accuracy",
        source: "docs/CODEC-SCOPE.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "q8-one-record-four-views",
        dimension: "bytes",
        scope: "public gel-phase-quad Record and Reader on synthetic records",
        input: "quad_compare scores every record through four materialized reference views per query; xtask verify runs it",
        expected: "a 1152-byte record; each view's score bit-identical to the shared read on the same build and platform; Q8_QUAD_EXACT=PASS",
        counterexample: "read as four independent memories, four votes, a 4x speedup, process RAM or semantic accuracy",
        source: "docs/Q8-QUAD.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "q8-literal-record",
        dimension: "bytes",
        scope: "the first 1024 bytes of one caller-owned file placed byte for byte into one public Q8 record (gel-live-lab --literal); literal bytes, not GEL knowledge printing",
        input: "Polish text with combining marks and emoji, one longer and one shorter than a record; the four public views and one Q8DEMO01 file with a retained pin",
        expected: "the gel-live-lab module agrees with an expectation built from the public gel-phase-quad API alone: view 0 holds the bytes, view 2 holds them reversed at the end, views 1 and 3 add one offset; active only where a byte was placed; every view is restored with 0 different bits; a 1164-byte file; a 1-byte body change is rejected under the pin although the format still decodes",
        counterexample: "read as GEL printing knowledge, as compression, as four copies of the file, or as a search over its text",
        source: "crates/gel-live-lab/src/literal.rs",
        evidence: Evidence::Probe(literal_record),
    },
    Claim {
        id: "record-history-exact",
        dimension: "bytes",
        scope: "public gel-history on synthetic ORB128 states; the history of one record",
        input: "a walk of 11 synthetic states changing 0 to 120 bits per step, its GELHIS01 bytes, the bytes with one bit changed, a file whose depth byte does not continue its chain and a file holding a dense residual",
        expected: "every entry equals an expectation built from the public gel-structural residual alone: a residual entry is its serialized length plus 10 bytes, kept only when smaller than the 129-byte literal and at most two residuals from a literal; the file length is the 48-byte header plus those entries; every state rebuilt bit for bit after reopening; the changed, chain-breaking and dense files refused",
        counterexample: "read as a compression ratio for real data, as tamper-proof storage (CRC64 is not a pin), as power-loss durability, or as storage that does not grow with every state",
        source: "crates/gel-history/src/lib.rs",
        evidence: Evidence::Probe(record_history),
    },
    Claim {
        id: "mutation-memory",
        dimension: "memory",
        scope: "streamed versus historical mutation",
        input: "mutation_compare identical paired states",
        expected: "allocations, user-space copies and peak additional heap measured locally (DHAT, heaptrack); fresh-process RSS peak of one mutation after a peak reset measured locally (campaign r1)",
        counterexample: "removed temporary Vec called measured RAM saving",
        source: "docs/MUTATION-COMPARISON.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "publication-os-faults",
        dimension: "persistence",
        scope: "no-replace snapshot publication",
        input: "0500 directory (Unix test) and a full 1 MiB tmpfs (Linux CI)",
        expected: "PermissionDenied / ENOSPC, no new file, previous snapshot reloads",
        counterexample: "partial or temporary file treated as a committed snapshot",
        source: "docs/PUBLICATION-FAULT-TESTS.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "platform-exclusions",
        dimension: "test-scope",
        scope: "seven Unix-only whole tests",
        input: "per-platform ci-evidence report and platform-diff of two platforms",
        expected: "ran once on Unix, absent on Windows, never counted as success there",
        counterexample: "a compile-time exclusion reported as a Windows pass",
        source: "docs/CI-EVIDENCE.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "recorder-fail-closed",
        dimension: "tooling",
        scope: "scripted film recorder",
        input: "recorder-lint: 14 forbidden lints and 10 rejected probes; process failure tests",
        expected: "controlled RECORDING_FAILED/REFUSED, no panic, no COMPLETE",
        counterexample: "a panic or partial COMPLETE after an I/O failure",
        source: "docs/RECORDER-SAFETY.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "crash-series-no-acknowledged-loss",
        dimension: "persistence",
        scope: "public gel-evidence: 24 documents added one at a time, a new snapshot after each, killed at a random moment",
        input: "xtask crash-series: SIGKILL after a seeded random time; xtask verify runs 5 trials on Unix",
        expected: "every acknowledged snapshot reloads with its pin and exact bytes; no partial snapshot; a fresh process resumes to the uninterrupted result",
        counterexample: "an acknowledged snapshot missing or different, a partial snapshot, or the series read as power-loss durability",
        source: "docs/CRASH-SERIES.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "batch-contract",
        dimension: "interface",
        scope: "gel-evidence --batch, schema gel-evidence/1",
        input: "HIT, UNKNOWN, an over-long line, a first ERROR and later commands",
        expected: "data on stdout only, exit 0/3/2, the first ERROR stops and later commands count as not run",
        counterexample: "an INCOMPLETE search or an ERROR reported as a clean exit",
        source: "docs/EVIDENCE-BATCH.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "answer-bench-rescoring",
        dimension: "scoring",
        scope: "answer-or-abstain sets: v1 with 80 questions with an answer and 80 without, v2 with 394 with an answer, v3 with 979, v4 with 985, v5 with 987, v6 with 990 with an answer, and v7 with 982 with an answer and 599 without; recorded answers of four systems (v1, v2), eight (v3, v6), eleven (v4) and ten (v5, v7)",
        input: "xtask answer-bench check re-scores every recorded answer with the published and strict rules",
        expected: "each set README, including the v3 and v4 precision and paired tables, equal the re-scored results; passage hashes and set identities match",
        counterexample: "re-scoring read as re-running GEL or the models",
        source: "docs/answer-or-abstain/README.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "backup-restore",
        dimension: "persistence",
        scope: "gel-backup directory with MANIFEST",
        input: "interrupted, tampered, foreign, withdrawn and complete backups",
        expected: "only a complete, not withdrawn backup restores, and only to a path that does not exist",
        counterexample: "a backup without a committed manifest restored, or a restore replacing a file",
        source: "docs/BACKUP.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "network-isolation",
        dimension: "environment",
        scope: "xtask isolation-check on Linux",
        input: "networked CI runner and an unprivileged network namespace",
        expected: "fails on the runner, passes in the namespace; only network-unreachable counts as blocked",
        counterexample: "Cargo offline flags presented as proof that nothing used the network",
        source: "docs/REPRODUCE-ISOLATED.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "format-mutations",
        dimension: "structure",
        scope: "GELSET01, GELSRC01 and Q8DEMO01 fixtures",
        input: "a finite matrix of 179 mutants and a lenient-reader control",
        expected: "every mutant rejected or its acceptance explained; report equal to the recorded matrix",
        counterexample: "an unexplained accepted mutant, or a lenient reader passing the control",
        source: "docs/MUTATION-MATRIX.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "binary-packages",
        dimension: "distribution",
        scope: "CI-built packages for three targets",
        input: "license files against the inventory, build-path scan, smoke run per platform",
        expected: "package refused on any failure; attestation only on a hand-started run",
        counterexample: "a package presented as code-signed, reproducible or tested beyond its build runner",
        source: "docs/BINARIES.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "property-map",
        dimension: "test-scope",
        scope: "35 documented properties mapped to 71 tests",
        input: "per-platform ci-evidence TESTS.txt",
        expected: "each mapped test ran and passed once, or is a declared Unix-only exclusion",
        counterexample: "a passing row read as full coverage of its property",
        source: "docs/PROPERTY-TESTS.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "grep-comparison",
        dimension: "comparison",
        scope: "36 public queries against grep and sha256sum, one host",
        input: "the same public corpus, answers compared before any timing",
        expected: "time compared only for same-answer queries; GEL slower on this corpus",
        counterexample: "timing queries whose answers differ, or a speed-up drawn from them",
        source: "docs/BENCHMARK-GREP.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "mutation-timing",
        dimension: "timing",
        scope: "one collection mutation, stream versus historical",
        input: "30 pairs per size and operation with identical results",
        expected: "paired ratio recorded with its run-to-run variation",
        counterexample: "absolute times from a mixed-core host compared across runs",
        source: "docs/MUTATION-COMPARISON.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-sketch-search",
        dimension: "timing",
        scope: "separate private implementation, compact sketches, three replays",
        input: "201 measured batches per size with raw CSV",
        expected: "author-reported per-scan p50; not reproducible from this checkout",
        counterexample: "divided by a full-scan row to claim a speed-up",
        source: "docs/GEL-EXPERIMENTAL-MEASUREMENTS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "ocean-full-scan",
        dimension: "timing",
        scope: "historical 1M/10M full scans",
        input: "two seeds, 100 queries each, 24 workers",
        expected: "author-reported baseline; not re-runnable from this checkout",
        counterexample: "presented as addressed-read latency or as GEL's own search path",
        source: "docs/OCEAN-SCALE.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "media-full-review",
        dimension: "presentation",
        scope: "three historical MP4 files",
        input: "complete timeline and source-log comparison",
        expected: "full human visual and privacy review; an AI review of every distinct frame is recorded separately",
        counterexample: "decode PASS substituted for visual review",
        source: "docs/MEDIA-DECODE-REVIEW.md",
        evidence: Evidence::Deferred("NOT_VERIFIED"),
    },
    Claim {
        id: "record-history-durability",
        dimension: "persistence",
        scope: "write_atomic of a history file under process kill and power cut",
        input: "a kill series or a power-cut test of gel-history saving; none has been run",
        expected: "not established: the history file format is outside the crash series",
        counterexample: "an atomic rename in the code read as a measured survival of a process kill or a power cut",
        source: "docs/RECORD-HISTORY.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
    Claim {
        id: "hardware-memory-compute",
        dimension: "mechanism",
        scope: "hardware-level memory computation",
        input: "hardware experiment and mechanism trace required",
        expected: "not established by CPU-only experiments",
        counterexample: "renamed CPU baseline presented as hardware memory compute",
        source: "docs/GEL-EXPERIMENTAL-MEASUREMENTS.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
    Claim {
        id: "commercial-advantage",
        dimension: "comparison",
        scope: "matched end-to-end tasks",
        input: "same corpus, oracle and operation boundaries required",
        expected: "no general superiority claim",
        counterexample: "different-sized sketches compared with full Q8 scans",
        source: "docs/GEL-EXPERIMENTAL-MEASUREMENTS.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
];

fn roundtrip() -> Result<bool, String> {
    let mut c = Collection::new();
    c.add("Polish", "Zażo\u{301}łć\r\nŁódź 🦀")?;
    let bytes = c.to_bytes();
    let loaded = Collection::from_bytes(&bytes, c.root())?;
    let mut changed = bytes.clone();
    *changed.last_mut().ok_or("empty snapshot")? ^= 1;
    Ok(loaded.to_bytes() == bytes
        && loaded.root() == c.root()
        && Collection::from_bytes(&changed, c.root()).is_err())
}
/// An expectation built from the public gel-phase-quad API alone must equal the
/// gel-live-lab module the row names, value for value.
fn literal_record() -> Result<bool, String> {
    use gel_live_lab::literal;
    use gel_phase_quad::{fixture, grid::DIM, Reader, Record};
    let long = "Zażo\u{301}łć gęślą jaźń 🦀\n".repeat(60).into_bytes();
    let short = "Zażółć 🦀".as_bytes().to_vec();
    let mut ok = literal::SEED == 510051 && long.len() > DIM && short.len() < DIM;
    for bytes in [long, short] {
        let n = bytes.len().min(DIM);
        let record = Record::new(
            std::array::from_fn(|j| if j < n { bytes[j] } else { 0 }),
            &std::array::from_fn(|j| j < n),
        );
        let reader = Reader::new(510051);
        let mut want = [[0u8; DIM]; 4];
        for pole in 0..4u8 {
            want[usize::from(pole)] = reader.bound_view(&record, pole)?.parts().1.phase;
        }
        // P1 adds an offset to P0, P2 mirrors P0, P3 adds the same offset to P2.
        // For seed 510051 the first offset is 168 and 4 of 1024 offsets are 0,
        // computed outside Rust from the published seed expansion.
        ok &= want[1][0].wrapping_sub(want[0][0]) == 168
            && (0..DIM).filter(|&j| want[1][j] == want[0][j]).count() == 4;
        ok &= (0..DIM).all(|j| {
            let offset = want[1][j].wrapping_sub(want[0][j]);
            want[0][j] == record.phase()[j]
                && want[2][DIM - 1 - j] == record.phase()[j]
                && want[3][j] == want[2][j].wrapping_add(offset)
        });
        let lit = literal::from_bytes(&bytes)?;
        let views = literal::views(lit.record())?;
        ok &= lit.placed() == n
            && lit.record().active_mask() == record.active_mask()
            && views.values == want
            && views.restored == [true; 4]
            && views.different_bits == 0;
        let raw = literal::encode(lit.record())?;
        let pin = digest(&raw);
        let mut changed = raw.clone();
        changed[12 + n / 2] ^= 1;
        ok &= raw == fixture::encode(std::slice::from_ref(&record))?
            && raw.len() == 1164
            && literal::check(&raw, &pin).is_ok()
            && literal::check(&changed, &pin).is_err()
            && fixture::decode(&changed).is_ok();
    }
    Ok(ok)
}
/// An expectation built from the public gel-structural residual alone must
/// equal what gel-history stores, entry for entry and byte for byte.
fn record_history() -> Result<bool, String> {
    use gel_core::{crc64_ecma, splitmix64, GelError, ORB_WORDS};
    use gel_history::{HistoryEntry, RecordHistory};
    use gel_orb::Orb1024;
    use gel_structural::Residual;
    let text = |e: GelError| e.to_string();
    let mut words = [0u64; ORB_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        *word = splitmix64(2026 + i as u64);
    }
    let mut states = vec![Orb1024::from_words(words)];
    for (step, k) in [0usize, 3, 92, 93, 1, 64, 120, 7, 7, 7]
        .into_iter()
        .enumerate()
    {
        let mut next = states[step];
        for j in 0..k {
            let bit = (step * 101 + 37 * j) % 1024;
            next.words_mut()[bit / 64] ^= 1u64 << (bit % 64);
        }
        states.push(next);
    }
    let mut history = RecordHistory::new();
    let (mut ok, mut depth, mut want_len, mut literals) = (true, 0u8, 48usize, 0);
    let mut offsets = Vec::new();
    for (i, state) in states.iter().enumerate() {
        ok &= history.append(*state).map_err(text)? == i;
        let kept = i
            .checked_sub(1)
            .map(|p| (p, Residual::from_exact_xor(state, &states[p])))
            .filter(|(_, r)| depth < 2 && r.serialized_len() + 10 < 129);
        offsets.push(want_len);
        ok &= match (&history.entries()[i], kept) {
            (HistoryEntry::Literal(stored), None) => {
                (depth, literals) = (0, literals + 1);
                want_len += 129;
                stored == state
            }
            (
                HistoryEntry::Residual {
                    parent,
                    depth: stored_depth,
                    residual,
                },
                Some((p, r)),
            ) => {
                depth += 1;
                want_len += r.serialized_len() + 10;
                *parent as usize == p && *stored_depth == depth && *residual == r
            }
            _ => false,
        };
    }
    let bytes = history.to_bytes().map_err(text)?;
    let reopened = RecordHistory::from_bytes(&bytes, 11, (bytes.len() - 48) as u64);
    ok &= literals == 5
        && bytes.len() == want_len
        && history.encoded_len() == want_len
        && reopened.and_then(|h| h.exact_history()).map_err(text)? == states;
    // Header fields as the format states them, CRC64s recomputed.
    let reseal = |mut file: Vec<u8>, count: u64| {
        let payload_len = (file.len() - 48) as u64;
        let payload_crc = crc64_ecma(&file[48..]);
        file[16..24].copy_from_slice(&count.to_le_bytes());
        file[24..32].copy_from_slice(&payload_len.to_le_bytes());
        file[32..40].copy_from_slice(&payload_crc.to_le_bytes());
        let header_crc = crc64_ecma(&file[..40]);
        file[40..48].copy_from_slice(&header_crc.to_le_bytes());
        file
    };
    let open = |file: &[u8]| RecordHistory::from_bytes(file, u64::MAX, u64::MAX);
    let mut changed = bytes.clone();
    *changed.last_mut().ok_or("empty history file")? ^= 1;
    // Entry 2 is two residuals from its literal; its depth byte (after the tag
    // and the 4-byte parent) now says 1.
    let mut lying = bytes.clone();
    lying[offsets[2] + 5] = 1;
    let mut dense = bytes[..offsets[1]].to_vec();
    dense.extend_from_slice(&[1, 0, 0, 0, 0, 1, 1]);
    dense.extend_from_slice(&[0u8; 128]);
    ok &= reseal(bytes.clone(), 11) == bytes
        && open(&changed) == Err(GelError::CorruptStore)
        && open(&reseal(lying, 11))
            == Err(GelError::InvalidResidual(
                "residual depth does not continue its parent chain",
            ))
        && open(&reseal(dense, 2))
            == Err(GelError::InvalidResidual(
                "dense residual is not written by this format",
            ));
    Ok(ok)
}
fn stale() -> Result<bool, String> {
    let mut c = Collection::new();
    c.add("A", "old source")?;
    let old = c
        .search("old source")?
        .hits
        .pop()
        .ok_or("missing old hit")?;
    c.replace(1, "new source")?;
    let new = c
        .search("new source")?
        .hits
        .pop()
        .ok_or("missing new hit")?;
    Ok(c.validate(&old).is_err() && c.validate(&new).is_ok())
}
fn structure() -> Result<bool, String> {
    let mut bytes = Collection::new().to_bytes();
    bytes[16..24].fill(0);
    Ok(Collection::from_bytes(&bytes, digest(&bytes)).is_err())
}
fn bounded_context() -> Result<bool, String> {
    let text = "Do not\r\nopen the valve.";
    let c = context::surrounding(text, 8..text.len(), 512)?;
    let long = format!("{}HIT{}", "ż".repeat(50), "🦀".repeat(50));
    let clipped = context::surrounding(&long, 100..103, 17)?;
    Ok(&text[c.context_span] == text
        && !c.omitted_before
        && !c.omitted_after
        && clipped.omitted_before
        && clipped.omitted_after
        && long.get(clipped.context_span).is_some())
}
fn cross_document() -> Result<bool, String> {
    let mut c = Collection::new();
    c.add("A", "not")?;
    c.add("B", "approved")?;
    Ok(c.search("not approved")?.status() == "UNKNOWN" && c.search("approved")?.hits.len() == 1)
}

fn registry_table() -> String {
    let mut s = String::from("| ID | Dimension | Evidence mode |\n|---|---|---|\n");
    for c in CLAIMS {
        let mode = match c.evidence {
            Evidence::Probe(_) => "EXECUTABLE_CHECK",
            Evidence::Deferred(s) => s,
        };
        s.push_str(&format!("| {} | {} | {} |\n", c.id, c.dimension, mode));
    }
    s
}

fn validate_table(doc: &str) -> Result<(), String> {
    if doc.matches("<!-- REGISTRY-BEGIN -->").count() != 1
        || doc.matches("<!-- REGISTRY-END -->").count() != 1
    {
        return Err("registry markers must be unique".into());
    }
    let table = doc
        .split_once("<!-- REGISTRY-BEGIN -->\n")
        .and_then(|(_, s)| s.split_once("<!-- REGISTRY-END -->"))
        .ok_or("missing registry markers")?
        .0;
    if table != registry_table() {
        return Err("claim documentation diverges from registry".into());
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let doc = fs::read_to_string(root.join("docs/CLAIMS.md")).map_err(|e| e.to_string())?;
    validate_table(&doc)?;
    let mut ids = BTreeSet::new();
    println!("id\tdimension\tstatus\tscope\tinput\texpected\tcounterexample\tsource");
    for c in CLAIMS {
        if !ids.insert(c.id) || !root.join(c.source).is_file() {
            return Err(format!("invalid registry entry {}", c.id));
        }
        let status = match c.evidence {
            Evidence::Probe(f) => {
                if f()? {
                    "PASS"
                } else {
                    return Err(format!("claim counterexample failed: {}", c.id));
                }
            }
            Evidence::Deferred(s) if DEFERRED_MODES.contains(&s) => s,
            Evidence::Deferred(s) => return Err(format!("unknown evidence mode {s}: {}", c.id)),
        };
        println!(
            "{}\t{}\t{status}\t{}\t{}\t{}\t{}\t{}",
            c.id, c.dimension, c.scope, c.input, c.expected, c.counterexample, c.source
        );
    }
    println!("CLAIMS_REGISTRY=PASS; only executable rows were tested, deferred rows remain open");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invented_pass_omitted_claim_and_duplicate_table() {
        let doc = format!(
            "<!-- REGISTRY-BEGIN -->\n{}<!-- REGISTRY-END -->",
            registry_table()
        );
        validate_table(&doc).unwrap();
        assert!(validate_table(&doc.replacen("NOT_VERIFIED", "PASS", 1)).is_err());
        assert!(validate_table(&doc.replacen("MEASURED_LOCAL", "PASS", 1)).is_err());
        assert!(validate_table(
            &doc.replace("| no-cross-document | retrieval | EXECUTABLE_CHECK |\n", "")
        )
        .is_err());
        assert!(validate_table(&format!("{doc}\n{doc}")).is_err());
    }
    #[test]
    fn executable_claims_require_positive_and_negative_cases() {
        for c in CLAIMS {
            if let Evidence::Probe(f) = c.evidence {
                assert!(f().unwrap(), "{}", c.id);
            }
        }
    }
    #[test]
    fn every_open_row_uses_an_allowed_mode() {
        for c in CLAIMS {
            if let Evidence::Deferred(s) = c.evidence {
                assert!(DEFERRED_MODES.contains(&s), "{}", c.id);
            }
        }
        assert!(!DEFERRED_MODES.contains(&"PASS"));
    }
    #[test]
    fn documentation_and_registry_agree() {
        check(crate::workspace_root().unwrap()).unwrap();
    }
}
