// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! One registry for scoped claims, executable counterexamples and explicit gaps.
use gel_source::{collection::Collection, context, digest};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
enum Evidence {
    Probe(fn() -> Result<bool, String>),
    Deferred(&'static str),
}
impl Claim {
    fn mode(&self) -> &'static str {
        match self.evidence {
            Evidence::Probe(_) => "EXECUTABLE_CHECK",
            Evidence::Deferred(s) => s,
        }
    }
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
        expected: "GEL 425 answers, 413 correct, 12 wrong (97.2%) and 25 answers to the 599 questions without one, plus one from a passage that holds the answer; precise setting 291 correct, 3 wrong, 3 answers without one; engines at the strict threshold 380-384 correct with 32-36 wrong in all; the candidate change 245 correct, 10 wrong in all, rolled back",
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
        id: "q8-record-save-stopped",
        dimension: "persistence",
        scope: "one Q8DEMO01 literal record of 1,164 bytes saved without replacing a file (gel-live-lab --save) through gel-source write_file_new_with",
        input: "a child process ended at 23 known points of the save (after 0 to 19 pieces of 64 bytes, after the flush, after the link, after the directory flush), each confirmed by a marker; a naive writer stopped at the 21 points of the first stage as control",
        expected: "the new path holds nothing or the whole record under its pin at all 23 points; a fresh save succeeds where it was free and is refused where the record is; the naive writer leaves part of a record at 19 points; cut files are refused and a zeroed 4 KiB block is caught by the pin only",
        counterexample: "part of a record at the new path, a point never reached counted as passed, a control without a partial file, or the test read as power-loss durability",
        source: "docs/PUBLICATION-FAULT-TESTS.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "q8-score-bits-across-platforms",
        dimension: "numeric",
        scope: "public Z256 Q8 scores: the 256 cosine values of the public Reader and 1,048,576 scores of a synthetic bank",
        input: "gel-live-lab --witness and a test of the 256 table bits on each CI platform (Linux, macOS arm64, Windows x86_64), against SHA-256 pins recorded on the author's Linux host",
        expected: "the same bits on every platform: the reader uses the frozen table grid::COS256_BITS, not the platform's maths library, which gave a different last bit at 13 levels on macOS and 7 on Windows before the table",
        counterexample: "a platform whose witness reports MISMATCH, or the table read as a claim about the accuracy of any maths library",
        source: "docs/LIVE-LAB.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
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
        id: "sha256-fips-vectors",
        dimension: "bytes",
        scope: "the SHA-256 of gel-source (gel_source::digest), written in this checkout",
        input: "the FIPS 180-2 example messages abc and the 448-bit two-block message, the empty message, and abc with one bit changed",
        expected: "the three published digests; another digest for the changed message",
        counterexample: "a matching digest read as proof of who wrote a file or that its content is true, or three vectors read as a proof for every input",
        source: "crates/gel-source/src/sha256.rs",
        evidence: Evidence::Probe(sha256_vectors),
    },
    Claim {
        id: "orb-record-layout",
        dimension: "bytes",
        scope: "public gel-orb Orb1024, one bit record in memory",
        input: "a seeded record, its 128 little-endian bytes, the record with its last bit flipped, and 127- and 129-byte inputs",
        expected: "128 bytes of 1,024 bits, aligned to 64 bytes; the bytes give back the same record; the flipped record differs in one bit and in its bytes; the wrong lengths are refused",
        counterexample: "read as a file format or as a store with integrity checks (see the gel-store tests)",
        source: "docs/FORMAT.md",
        evidence: Evidence::Probe(orb_layout),
    },
    Claim {
        id: "structural-exact-xor",
        dimension: "structure",
        scope: "public gel-structural EncodedOrb in memory, a prototype parent",
        input: "a seeded parent and targets 94 and 95 bits away from it; decoding with the parent, without it, with stored depths 0 and 3, and with a wrong parent; a child of a depth-2 parent",
        expected: "94 changed bits kept as a delta of depth 1 and 95 stored as a literal; decoding with the parent gives the target bit for bit; no parent, depth 0 or 3 and a child of depth 2 refused; a wrong parent decodes without an error to another record",
        counterexample: "a wrong parent read as detected, or the break-even read as a compression ratio for real data",
        source: "docs/STRUCTURAL-CODEC.md",
        evidence: Evidence::Probe(structural_xor),
    },
    Claim {
        id: "q8-record-size",
        dimension: "bytes",
        scope: "public gel-phase-quad Record in memory",
        input: "a record of 1,024 phase bytes and a 1,024-position activity mask",
        expected: "the record takes 1,152 bytes (1,024 values and a 128-byte mask), four such copies 4,608; every phase byte and mask position reads back",
        counterexample: "read as process memory, as the 1,164-byte file, or as four independent memories",
        source: "docs/Q8-QUAD.md",
        evidence: Evidence::Probe(q8_record_size),
    },
    Claim {
        id: "no-third-party-packages",
        dimension: "distribution",
        scope: "Cargo.lock of this checkout and the workspace members of Cargo.toml",
        input: "every package entry of Cargo.lock; a copy with a registry source line or an extra package as counterexamples (unit test)",
        expected: "the package names equal the workspace member names; no package has a source or checksum line",
        counterexample: "read as a review of the Rust toolchain or its standard library, which are linked in and covered by their own notices",
        source: "docs/DEPENDENCY-INVENTORY.md",
        evidence: Evidence::Probe(local_packages),
    },
    Claim {
        id: "no-network-code",
        dimension: "environment",
        scope: "source text of every .rs file of the checkout; with no third-party package and unsafe code forbidden, the standard socket types are the only network interface in reach",
        input: "the names of the standard TCP and UDP socket types; a file set where a library file names one as counterexample (unit test)",
        expected: "they are named only in xtask/src/isolation.rs, the isolation check",
        counterexample: "read as proof that a run used no network (see network-isolation), or as covering programs a tool starts as child processes",
        source: "SECURITY.md",
        evidence: Evidence::Probe(no_network_code),
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
        id: "record-history-kill-series",
        dimension: "persistence",
        scope: "public gel-history: 240 states appended to one record, the whole GELHIS01 file saved with write_atomic after each, the writer killed at a random moment",
        input: "xtask history-crash-series: SIGKILL after a seeded random time, then a fresh process continues; a control writer rewrites the file in place in four flushed pieces; xtask verify runs 5 trials on Unix without the control",
        expected: "the file opens with open_verified and equals the last acknowledged history or the next one byte for byte; no partial file; every resume reaches the bytes of a run without a kill; the in-place control leaves at least one partial file",
        counterexample: "an acknowledged state missing, a partial file accepted, a control without a partial file, or the series read as power-loss durability",
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
        id: "history-format-mutations",
        dimension: "structure",
        scope: "a GELHIS01 record-history fixture of four states, 338 bytes",
        input: "a finite matrix of 151 mutants with their CRC64 fields recomputed, a reference reader with its own CRC-64/ECMA-182, and a reader without the canonical-form rule as control",
        expected: "every mutant rejected under the original pin; with a recomputed pin the library and the reference agree on every row and the 14 accepted changes rebuild the same states; report equal to the recorded matrix",
        counterexample: "a row where the library and the reference disagree, or a reader without the canonical-form rule passing the control",
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
        id: "tool-version-license",
        dimension: "interface",
        scope: "the eight tool programs gel-cli, gel-bench, gel-physics, gel-live-lab, gel-evidence, gel-backup, gel-source and gel-license; not the xtask runner",
        input: "--version or --license as the only argument; the tool_flags and license_cli tests run the built programs",
        expected: "one line GEL_TOOL=NAME VERSION=VERSION LICENSE=GEL-RAM-NCRL-1.0 and exit 0; --license adds the license name, the Required Notice line, the NOTICE summary and the third-party line; with other arguments a flag keeps its earlier meaning",
        counterexample: "the printed lines read as the license terms, which only LICENSE holds",
        source: "docs/BINARIES.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "source-license-headers",
        dimension: "provenance",
        scope: "every GEL RAM-owned .rs file; the generated Unicode tables are third-party data and exempt",
        input: "xtask source-headers, run by verify",
        expected: "each file begins with the exact two-line header, the license name and the Required Notice, with no byte order mark and LF line endings; SOURCE_HEADERS_GATE=PASS",
        counterexample: "a header read as a grant beyond LICENSE or as proof of who wrote the file",
        source: "docs/GATES.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "license-self-check",
        dimension: "licensing",
        scope: "gel-license check DIR and gel-license use ANSWERS_FILE",
        input: "verify runs the release gel-license check of this workspace; the package smoke run checks each package and one answers file",
        expected: "for this workspace LICENSE_FILE=EXACT, REQUIRED_NOTICE=KEPT, THIRD_PARTY_NOTICES=KEPT, every source header intact and DISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT; use names the LICENSE sentences that apply to the user's own answers",
        counterexample: "a report read as a finding of compliance, as proof that every notice was kept, or as legal advice",
        source: "docs/LICENSE-DECISION.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "unicode-16-normalization",
        dimension: "text",
        scope: "Unicode Normalization Form C of gel-source, tables from the Unicode Character Database 16.0.0",
        input: "workspace tests: every line of NormalizationTest 16.0.0 and of 15.1.0, every scalar value and random and adversarial strings against the earlier and an independent implementation",
        expected: "every line gives its listed forms; the table version is 16.0.0",
        counterexample: "normalization read as language understanding, or as a result for a later Unicode version",
        source: "crates/gel-source/src/nfc.rs",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "readme-generated",
        dimension: "presentation",
        scope: "README.md and README-MULTIMEDIA.html, generated as a whole by tools/readme_presentation.rs",
        input: "the read-only readme-presentation workflow in CI builds both again and compares; not part of verify",
        expected: "README_PRESENTATION=UNCHANGED; its counts come from docs/COMMITMENTS.md, Cargo.lock, the crash series, the recorded Q8 runs and the timing records; a series that disagrees with its own rows, a run without its PASS line or a license quote missing from LICENSE stops the build",
        counterexample: "a generated number read as checked beyond the file it is counted from",
        source: "docs/README-PRESENTATION.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "version-line",
        dimension: "interface",
        scope: "the files that name the current version: the citation file, the status file, the guides, the batch example, the README builder and the README link to the release notes",
        input: "xtask/tests/version_line.rs against the workspace version",
        expected: "each listed text carries the workspace version, and the root holds the release notes of that version only",
        counterexample: "a passing test read as a review of every version number in the tree",
        source: "xtask/tests/version_line.rs",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "goal-statement",
        dimension: "presentation",
        scope: "docs/GOAL.md, the README and docs/START-HERE.md",
        input: "xtask/tests/goal_statement.rs",
        expected: "the README and start-here repeat the goal sentence word for word; the goal file keeps its shape: the sentence and four commitments, each with one status line",
        counterexample: "the goal sentence read as a result (see text-ai-goal)",
        source: "xtask/tests/goal_statement.rs",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "commitments-fixed-in-time",
        dimension: "provenance",
        scope: "the published commitments in docs/COMMITMENTS.md: date, label, counts and the SHA-256 of a manifest kept private",
        input: "xtask commitment-check once a manifest and its files are handed over; its unit tests change every kind of entry",
        expected: "the handed-over files match the published SHA-256, or the check names the first difference; no change passes",
        counterexample: "a commitment read as evidence that the files are correct, or of which commitment holds what",
        source: "docs/COMMITMENTS.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "rebuild-identical",
        dimension: "distribution",
        scope: "the four packaged binaries, built twice from git archive HEAD in two directories whose paths differ",
        input: "xtask rebuild-check; the check job of the binaries workflow runs it on Linux, macOS and Windows",
        expected: "REBUILD=IDENTICAL on the same system; a difference fails the job",
        counterexample: "a release binary read as rebuildable byte for byte on another system: the v0.6.1 tag rebuilt on the author's machine gave other bytes",
        source: "docs/BINARIES.md",
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
        id: "crash-series-recorded",
        dimension: "persistence",
        scope: "public gel-evidence, one recorded series of 200 trials on the author's Linux machine, ext4, 2026-09-29",
        input: "24 documents, a new snapshot after each, SIGKILL after a seeded random 1 to 344 ms",
        expected: "2,683 acknowledged snapshots, 0 lost and 0 partial; 200 of 200 trials resumed to the uninterrupted result; 153 kills in the middle of a publication",
        counterexample: "read as power-loss durability, as a result on another file system, or as the private store",
        source: "docs/evidence-crash/crash-series-linux.txt",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "q8-recorded-runs",
        dimension: "bytes",
        scope: "the final local Q8 campaign of 2026-09-11 on synthetic fixtures",
        input: "all 48 runs of the V2 benchmark and all 24 runs beside a packed canonical-single baseline, kept with their summary",
        expected: "48 of 48 V2 runs end with Q8_QUAD_EXACT=PASS",
        counterexample: "their timings read as a speedup of four views, or the runs read as semantic accuracy",
        source: "docs/evidence-q8-current/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "speed-set-against-v0.6.1",
        dimension: "timing",
        scope: "xtask speed-ab: two checkouts of v0.6.1 (A, A2) against main at 876247f (B), one loaded machine, not pinned to cores, 2026-10-20",
        input: "133 timed cases in 23 of 24 timing programs, 3 rounds with the order rotated",
        expected: "103 cases faster beyond their A/A band, 18 slower beyond it (SHA-256 alone at 0.28 and 0.39) and 12 inside it; 22 of 23 timed programs printed the same output digest in all three trees",
        counterexample: "ratios of different programs compared with each other, a ratio read without its A/A band, or the timings read as the claim rather than the output digests",
        source: "docs/speed-results/2026-10-20-against-v0.6.1/README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "speed-set-quiet-machine",
        dimension: "timing",
        scope: "xtask speed-ab for the second and third speed packages, a quieter machine, not pinned to cores, 2026-10-25",
        input: "each change (B) against two checkouts of its baseline (A, A2), 3 rounds with the order rotated",
        expected: "every example printed the same output digest in A, A2 and B (SPEED_AB_VERDICT=SAME_OUTPUT); the ratios are in each summary",
        counterexample: "a ratio read without its A/A band, or across programs",
        source: "docs/speed-results/2026-10-25-quiet-machine/README.md",
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
        id: "independent-reproduction",
        dimension: "reproduction",
        scope: "the public checks run on a second machine by someone other than the author",
        input: "a reproduction report through the reproduction form or issue #20",
        expected: "not verified: no independent run on a second machine has been reported yet",
        counterexample: "hosted CI or a run on the author's machine read as an independent reproduction",
        source: "docs/REPRODUCE.md",
        evidence: Evidence::Deferred("NOT_VERIFIED"),
    },
    Claim {
        id: "record-history-durability",
        dimension: "persistence",
        scope: "write_atomic of a history file under a power cut",
        input: "a power-cut test of gel-history saving; none has been run",
        expected: "not established: the kill series covers a process kill only",
        counterexample: "the kill series or the directory flush in the code read as a measured survival of a power cut",
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
    Claim {
        id: "text-ai-goal",
        dimension: "goal",
        scope: "a text AI that holds a free conversation in Polish and English in sentences of its own, shows the source it relied on and says it does not know",
        input: "a measured conversation; none has been run",
        expected: "not established: the goal page gives each commitment as a goal; today the document tools return exact quotations",
        counterexample: "phrase lookup, exact quotations or answer-or-abstain counts read as the text AI",
        source: "docs/GOAL.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
    Claim {
        id: "q8-semantic-accuracy",
        dimension: "retrieval",
        scope: "public Q8 records, which hold synthetic values or literal bytes",
        input: "a measurement of meaning on public Q8 scores; none is part of the checks",
        expected: "not established: quad_compare prints SEMANTIC_ACCURACY=NOT_MEASURED",
        counterexample: "bit-identical scores read as accuracy of meaning",
        source: "docs/Q8-QUAD.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
    Claim {
        id: "snapshot-power-loss",
        dimension: "persistence",
        scope: "gel-evidence snapshot publication under a power cut",
        input: "a power-cut test; none has been run",
        expected: "not established: the crash series kills a process, and data in the operating system's or the disk's cache may not survive a power loss",
        counterexample: "the crash series, the fault tests or a directory flush in the code read as a measured survival of a power cut",
        source: "docs/CRASH-SERIES.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
    Claim {
        id: "extreme-tail-latency",
        dimension: "timing",
        scope: "the historical Ocean full scans, 100 observations per long run",
        input: "enough observations for p99.9 to p99.999; not collected",
        expected: "not established: no stable p99.9, p99.99 or p99.999 tail",
        counterexample: "the slowest of 100 runs read as a p99.9 or higher tail",
        source: "docs/OCEAN-SCALE.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
];

/// A sentence of another public file that names registry rows: the file, its
/// words (a line break or a run of spaces counts as one space) and each row it
/// stands for with the state the row must keep. Rewording the sentence or
/// changing the row's state fails `xtask claims` until both agree again.
struct Citation {
    file: &'static str,
    words: &'static str,
    rows: &'static [(&'static str, &'static str)],
}

const HARDWARE: &[(&str, &str)] = &[("hardware-memory-compute", "NOT_ESTABLISHED")];
const HARDWARE_NOT_ESTABLISHED: &str = "Hardware-level memory computation, the direction of the project, is not established by any run on these pages or in this repository";

const CITED: &[Citation] = &[
    Citation {
        file: "README.md",
        words: "The text AI is a goal, not a result ([the goal](docs/GOAL.md)). Hardware-level memory computation and power-cut durability of the record history are `NOT_ESTABLISHED` ([claim registry](docs/CLAIMS.md))",
        rows: &[
            ("text-ai-goal", "NOT_ESTABLISHED"),
            ("hardware-memory-compute", "NOT_ESTABLISHED"),
            ("record-history-durability", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "README.md",
        words: "No independent run on a second machine has been reported yet",
        rows: &[("independent-reproduction", "NOT_VERIFIED")],
    },
    Citation {
        file: "README.md",
        words: "| `--literal` | `ROUNDTRIP=4/4 DIFFERENT_BITS=0`, `TAMPER=REJECTED` |",
        rows: &[("q8-literal-record", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "README.md",
        words: "| `quad_compare` | `Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED` |",
        rows: &[
            ("q8-one-record-four-views", "SEPARATE_GATE"),
            ("q8-semantic-accuracy", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "README.md",
        words: "| `mutation-matrix` | `MUTATION_MATRIX=PASS mutants=179` |",
        rows: &[("format-mutations", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "| `crash-series 20` | `CRASH_SERIES=PASS` with `acknowledged_lost=0` |",
        rows: &[("crash-series-no-acknowledged-loss", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "48 of 48 recorded runs end with Q8_QUAD_EXACT=PASS",
        rows: &[("q8-recorded-runs", "MEASURED_LOCAL")],
    },
    Citation {
        file: "README.md",
        words: "a replaced document's old citation is refused, and a saved snapshot with one changed byte fails to load under the SHA-256 you kept",
        rows: &[
            ("stale-citation", "EXECUTABLE_CHECK"),
            ("source-roundtrip", "EXECUTABLE_CHECK"),
        ],
    },
    Citation {
        file: "README.md",
        words: "0 of 2,683 acknowledged snapshots were lost, none was partial, and every trial resumed to the uninterrupted result",
        rows: &[("crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "README.md",
        words: "A process kill, not a power cut: power-loss durability is not established.",
        rows: &[("snapshot-power-loss", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "README.md",
        words: "Write failures, denied permissions and a full disk leave the previous snapshot intact",
        rows: &[("publication-os-faults", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "the presentation check fails if either file changes without a rebuild",
        rows: &[("readme-generated", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "the eight tool programs answer `--version` and `--license` (the `xtask` runner does not); every GEL RAM-owned Rust source file begins with a two-line license header",
        rows: &[
            ("tool-version-license", "SEPARATE_GATE"),
            ("source-license-headers", "SEPARATE_GATE"),
        ],
    },
    Citation {
        file: "README.md",
        words: "packages in `Cargo.lock` is a crate of this checkout. SHA-256 and Unicode normalization are written here; SHA-256 alone runs at 0.28 and 0.39 of the speed of v0.6.1",
        rows: &[
            ("no-third-party-packages", "EXECUTABLE_CHECK"),
            ("sha256-fips-vectors", "EXECUTABLE_CHECK"),
            ("speed-set-against-v0.6.1", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "README.md",
        words: "normalization follows Unicode 16.0.0",
        rows: &[("unicode-16-normalization", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "the files can be handed over later and checked with `xtask commitment-check`. It shows the files existed and did not change, not that they are correct.",
        rows: &[("commitments-fixed-in-time", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "One fixed 128-byte record of 1,024 bits, 64-byte aligned",
        rows: &[("orb-record-layout", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "README.md",
        words: "a missing parent or an invalid depth is refused, a wrong parent is not detected",
        rows: &[("structural-exact-xor", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "README.md",
        words: "One 1,152-byte record read through four equivalent views",
        rows: &[("q8-record-size", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/GOAL.md",
        words: "The text AI is a goal, not a result.",
        rows: &[("text-ai-goal", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/START-HERE.md",
        words: "This repository does not establish hardware-level memory computation ([claim registry](CLAIMS.md): `NOT_ESTABLISHED`)",
        rows: HARDWARE,
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "may not survive a power loss. Power-loss durability is not established.",
        rows: &[("snapshot-power-loss", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "Power-loss durability of the history file stays `NOT_ESTABLISHED` in the [claim registry](CLAIMS.md)",
        rows: &[("record-history-durability", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "The [claim registry](CLAIMS.md) lists `record-history-exact` as an executable check, the kill series `record-history-kill-series` and the format mutation matrix `history-format-mutations` as `SEPARATE_GATE`, and `record-history-durability`, now a power cut only, as `NOT_ESTABLISHED`.",
        rows: &[
            ("record-history-exact", "EXECUTABLE_CHECK"),
            ("record-history-kill-series", "SEPARATE_GATE"),
            ("history-format-mutations", "SEPARATE_GATE"),
            ("record-history-durability", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/MEASURED-PROGRESS.md",
        words: "The claim registry lists this result as `MEASURED_LOCAL`.",
        rows: &[("store-kill-during-learning", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "the [claim registry](CLAIMS.md) lists them as `MEASURED_LOCAL`",
        rows: &[
            ("gel-answer-verdict-self-read", "MEASURED_LOCAL"),
            ("gel-natural-question-answers", "MEASURED_LOCAL"),
            ("gel-answer-set-v2", "MEASURED_LOCAL"),
            ("gel-beside-bm25-v3", "MEASURED_LOCAL"),
            ("gel-beside-bm25-v4", "MEASURED_LOCAL"),
            ("gel-beside-bm25-v5", "MEASURED_LOCAL"),
            ("gel-beside-bm25-v6", "MEASURED_LOCAL"),
            ("gel-beside-bm25-v7", "MEASURED_LOCAL"),
            ("store-kill-during-learning", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- physical memory-side compute or other hardware-level memory computation; - production power-loss durability;",
        rows: &[
            ("hardware-memory-compute", "NOT_ESTABLISHED"),
            ("snapshot-power-loss", "NOT_ESTABLISHED"),
            ("record-history-durability", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/OCEAN-SCALE.md",
        words: "100 observations do not establish stable p99.9–p99.999 tails.",
        rows: &[("extreme-tail-latency", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/Q8-QUAD.md",
        words: "`Q8_QUAD_EXACT=PASS SEMANTIC_ACCURACY=NOT_MEASURED`",
        rows: &[("q8-semantic-accuracy", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/EXTERNAL-CHECKS.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
    },
    Citation {
        file: "docs/answer-or-abstain-v3/README.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
    },
    Citation {
        file: "docs/answer-or-abstain-v4/README.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
    },
    Citation {
        file: "docs/answer-or-abstain-v5/README.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
    },
    Citation {
        file: "docs/answer-or-abstain-v6/README.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
    },
    Citation {
        file: "docs/answer-or-abstain-v7/README.md",
        words: HARDWARE_NOT_ESTABLISHED,
        rows: HARDWARE,
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
/// The published FIPS 180-2 digests, written out here, not computed.
fn sha256_vectors() -> Result<bool, String> {
    let known: [(&[u8], &str); 3] = [
        (
            b"abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            b"",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
    ];
    let changed = [b'a', b'b', b'c' ^ 1];
    Ok(known
        .iter()
        .all(|(message, want)| gel_source::hex(&digest(message)) == *want)
        && digest(&changed) != digest(b"abc"))
}
fn seeded_orb(seed: u64) -> gel_orb::Orb1024 {
    gel_orb::Orb1024::from_words(std::array::from_fn(|i| {
        gel_core::splitmix64(seed + i as u64)
    }))
}
fn orb_layout() -> Result<bool, String> {
    use gel_orb::Orb1024;
    use std::mem::{align_of, size_of};
    let text = |e: gel_core::GelError| e.to_string();
    let orb = seeded_orb(7);
    let bytes = orb.to_le_bytes();
    let mut flipped = orb;
    flipped
        .set_bit(1023, !orb.bit(1023).map_err(text)?)
        .map_err(text)?;
    Ok(size_of::<Orb1024>() == 128
        && align_of::<Orb1024>() == 64
        && bytes.len() == 128
        && Orb1024::from_le_bytes(&bytes).map_err(text)? == orb
        && orb.hamming_distance(&flipped) == 1
        && flipped.to_le_bytes() != bytes
        && Orb1024::from_le_bytes(&bytes[..127]).is_err()
        && Orb1024::from_le_bytes(&[0u8; 129]).is_err())
}
/// The break-even of docs/STRUCTURAL-CODEC.md for a prototype parent, the
/// refusals of the decoder, and the limit the README states: a wrong parent
/// is not detected.
fn structural_xor() -> Result<bool, String> {
    use gel_orb::Orb1024;
    use gel_structural::{EncodedOrb, ParentRef};
    let text = |e: gel_core::GelError| e.to_string();
    let parent = seeded_orb(99);
    let away = |k: usize| {
        let mut target = parent;
        for j in 0..k {
            // 37 is odd, so the first 1,024 positions are all different.
            let bit = j * 37 % 1024;
            target.words_mut()[bit / 64] ^= 1u64 << (bit % 64);
        }
        target
    };
    let (t94, t95) = (away(94), away(95));
    let encode = |t: &Orb1024, depth: u8| {
        EncodedOrb::encode_against(t, &parent, ParentRef::Prototype(1), depth)
    };
    let e94 = encode(&t94, 0).map_err(text)?;
    let EncodedOrb::Delta { residual, .. } = &e94 else {
        return Ok(false);
    };
    let with_depth = |depth: u8| EncodedOrb::Delta {
        parent: ParentRef::Prototype(1),
        parent_depth: depth,
        residual: residual.clone(),
    };
    Ok(matches!(
        e94,
        EncodedOrb::Delta {
            parent_depth: 1,
            ..
        }
    ) && matches!(encode(&t95, 0).map_err(text)?, EncodedOrb::Literal(_))
        && e94.decode(Some(&parent)).map_err(text)? == t94
        && e94.decode(None).is_err()
        && with_depth(0).decode(Some(&parent)).is_err()
        && with_depth(3).decode(Some(&parent)).is_err()
        && encode(&t94, 2).is_err()
        && e94.decode(Some(&Orb1024::ZERO)).map_err(text)? != t94)
}
fn q8_record_size() -> Result<bool, String> {
    use gel_phase_quad::{grid::DIM, Record};
    let phase: [u8; DIM] = std::array::from_fn(|j| (j * 7 % 256) as u8);
    let mask: [bool; DIM] = std::array::from_fn(|j| j % 3 == 0);
    let record = Record::new(phase, &mask);
    Ok(std::mem::size_of::<Record>() == 1152
        && DIM + DIM / 8 == 1152
        && 4 * std::mem::size_of::<Record>() == 4608
        && *record.phase() == phase
        && record.active_mask() == mask)
}
/// The package names of `[[package]]` entries, or None when one has a
/// source or checksum line, which only a package from outside the checkout has.
fn lock_packages(lock: &str) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for entry in lock.split("[[package]]\n").skip(1) {
        let mut name = None;
        for line in entry.lines() {
            if line.starts_with("source = ") || line.starts_with("checksum = ") {
                return None;
            }
            if let Some(n) = line.strip_prefix("name = \"") {
                name = n.strip_suffix('"');
            }
        }
        names.insert(name?.to_string());
    }
    Some(names)
}
/// The package names of the workspace members listed in the root Cargo.toml.
fn member_packages(root: &Path) -> Result<BTreeSet<String>, String> {
    let manifest = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let list = manifest
        .split_once("\nmembers = [\n")
        .and_then(|(_, rest)| rest.split_once("\n]"))
        .ok_or("Cargo.toml: no members list")?
        .0;
    let mut names = BTreeSet::new();
    for member in list.lines() {
        let member = member
            .trim()
            .strip_prefix('"')
            .and_then(|m| m.strip_suffix("\","))
            .ok_or("Cargo.toml: unexpected members line")?;
        let text = fs::read_to_string(root.join(member).join("Cargo.toml"))
            .map_err(|e| format!("{member}: {e}"))?;
        let name = text
            .lines()
            .find_map(|l| l.strip_prefix("name = \"")?.strip_suffix('"'))
            .ok_or(format!("{member}: no package name"))?;
        names.insert(name.to_string());
    }
    Ok(names)
}
fn local_packages() -> Result<bool, String> {
    let root = crate::workspace_root()?;
    let lock = fs::read_to_string(root.join("Cargo.lock")).map_err(|e| e.to_string())?;
    let members = member_packages(root)?;
    Ok(!members.is_empty() && lock_packages(&lock) == Some(members))
}
/// The standard socket types, split so that this file does not name them.
const SOCKET_TYPES: [[&str; 2]; 3] = [["Tcp", "Stream"], ["Tcp", "Listener"], ["Udp", "Socket"]];
/// The files, of (path relative to the root with `/`, text), that name a socket type.
fn socket_users(files: &[(String, String)]) -> Vec<String> {
    let needles: Vec<String> = SOCKET_TYPES.iter().map(|p| p.concat()).collect();
    files
        .iter()
        .filter(|(_, text)| needles.iter().any(|n| text.contains(n.as_str())))
        .map(|(path, _)| path.clone())
        .collect()
}
fn relative(root: &Path, path: &Path) -> Option<String> {
    let parts: Option<Vec<&str>> = path
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|c| c.as_os_str().to_str())
        .collect();
    Some(parts?.join("/"))
}
/// Every file of the checkout with the given extension, as (relative path, text).
fn files_with(root: &Path, extension: &str) -> Result<Vec<(String, String)>, String> {
    let tree = crate::tree_scan::walk(root).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for (path, meta) in tree.paths.iter().zip(&tree.metadata) {
        if !meta.is_file() || path.extension().and_then(|e| e.to_str()) != Some(extension) {
            continue;
        }
        let rel = relative(root, path).ok_or("a path that is not UTF-8")?;
        let text = fs::read_to_string(path).map_err(|e| format!("{rel}: {e}"))?;
        files.push((rel, text.replace("\r\n", "\n")));
    }
    files.sort();
    Ok(files)
}
fn no_network_code() -> Result<bool, String> {
    let files = files_with(crate::workspace_root()?, "rs")?;
    Ok(files.len() > 1 && socket_users(&files) == ["xtask/src/isolation.rs"])
}

/// Words with every line break or run of whitespace as one space.
fn words(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every citation is in its file and every row it names carries the state
/// given; returns the number of (citation, row) pairs checked.
fn check_citations(
    claims: &[Claim],
    cited: &[Citation],
    read: impl Fn(&str) -> Result<String, String>,
) -> Result<usize, String> {
    let mut pairs = 0;
    for c in cited {
        if !words(&read(c.file)?).contains(&words(c.words)) {
            return Err(format!(
                "{} no longer says \"{}\"; update CITED in xtask/src/claims.rs",
                c.file, c.words
            ));
        }
        for (id, mode) in c.rows {
            let row = claims.iter().find(|r| r.id == *id).ok_or(format!(
                "{} cites {id}, which is not in the registry",
                c.file
            ))?;
            if row.mode() != *mode {
                return Err(format!(
                    "{} gives {id} as {mode}; the registry has {}",
                    c.file,
                    row.mode()
                ));
            }
            pairs += 1;
        }
    }
    Ok(pairs)
}

/// In a paragraph (not a table) that links the registry, a code span shaped
/// like a row id must be a registry id and one shaped like a state must be a
/// state. Returns the number of such paragraphs.
fn check_paragraphs(claims: &[Claim], files: &[(String, String)]) -> Result<usize, String> {
    let is_id = |s: &str| {
        s.contains('-')
            && s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
    };
    let is_state =
        |s: &str| s.contains('_') && s.chars().all(|c| c.is_ascii_uppercase() || c == '_');
    let mut paragraphs = 0;
    for (file, text) in files {
        if file == "docs/CLAIMS.md" {
            continue;
        }
        for paragraph in text.split("\n\n") {
            if !paragraph.contains("CLAIMS.md")
                || paragraph.lines().any(|l| l.trim_start().starts_with('|'))
            {
                continue;
            }
            paragraphs += 1;
            for span in paragraph.split('`').skip(1).step_by(2) {
                if is_id(span) && !claims.iter().any(|c| c.id == span) {
                    return Err(format!("{file} names `{span}`, which is not a registry id"));
                }
                if is_state(span) && !claims.iter().any(|c| c.mode() == span) {
                    return Err(format!(
                        "{file} names `{span}`, which is not a registry state"
                    ));
                }
            }
        }
    }
    Ok(paragraphs)
}

fn registry_table() -> String {
    let mut s = String::from("| ID | Dimension | Evidence mode |\n|---|---|---|\n");
    for c in CLAIMS {
        s.push_str(&format!("| {} | {} | {} |\n", c.id, c.dimension, c.mode()));
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
    let pairs = check_citations(CLAIMS, CITED, |file| {
        fs::read_to_string(root.join(file))
            .map(|t| t.replace("\r\n", "\n"))
            .map_err(|e| format!("{file}: {e}"))
    })?;
    let paragraphs = check_paragraphs(CLAIMS, &files_with(root, "md")?)?;
    println!(
        "CLAIMS_CITATIONS=PASS citations={} rows={pairs} paragraphs={paragraphs}",
        CITED.len()
    );
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
    fn read_tree(file: &str) -> Result<String, String> {
        fs::read_to_string(crate::workspace_root()?.join(file)).map_err(|e| e.to_string())
    }
    #[test]
    fn a_reworded_citation_or_a_changed_state_is_refused() {
        assert!(check_citations(CLAIMS, CITED, read_tree).unwrap() > CITED.len());
        // The README first-screen sentence removed from a copy.
        let first = &CITED[0];
        let reworded = |file: &str| {
            let text = read_tree(file)?;
            Ok(if file == first.file {
                words(&text).replace(&words(first.words), "")
            } else {
                text
            })
        };
        assert!(check_citations(CLAIMS, CITED, reworded).is_err());
        // The registry with a cited row moved to another state.
        let mut moved: Vec<Claim> = CLAIMS.to_vec();
        let row = moved
            .iter_mut()
            .find(|c| c.id == "hardware-memory-compute")
            .unwrap();
        row.evidence = Evidence::Deferred("MEASURED_LOCAL");
        assert!(check_citations(&moved, CITED, read_tree).is_err());
        // A citation of a row that is not in the registry.
        let unknown = [Citation {
            file: "docs/GOAL.md",
            words: "The text AI is a goal",
            rows: &[("text-ai-gaol", "NOT_ESTABLISHED")],
        }];
        assert!(check_citations(CLAIMS, &unknown, read_tree).is_err());
    }
    #[test]
    fn a_paragraph_linking_the_registry_names_only_its_ids_and_states() {
        let good = "See the [claims](CLAIMS.md): `q8-literal-record` is an executable check and\n`record-history-durability` is `NOT_ESTABLISHED`; `TAMPER=REJECTED`, `--save`.";
        let file = |text: &str| vec![("docs/X.md".to_string(), text.to_string())];
        assert_eq!(check_paragraphs(CLAIMS, &file(good)), Ok(1));
        for bad in [
            good.replace("q8-literal-record", "q8-literal-recrod"),
            good.replace("`NOT_ESTABLISHED`", "`NOT_ESTABLISH`"),
        ] {
            assert!(check_paragraphs(CLAIMS, &file(&bad)).is_err(), "{bad}");
        }
        // Tables and paragraphs without the link are not read.
        let table = "| a | [claims](CLAIMS.md) `no-such-row` |";
        let unlinked = "`no-such-row` is `NOT_A_STATE`.";
        assert_eq!(check_paragraphs(CLAIMS, &file(table)), Ok(0));
        assert_eq!(check_paragraphs(CLAIMS, &file(unlinked)), Ok(0));
    }
    #[test]
    fn a_package_from_outside_the_checkout_is_refused() {
        let root = crate::workspace_root().unwrap();
        let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
        let members = member_packages(root).unwrap();
        assert_eq!(lock_packages(&lock).as_ref(), Some(&members));
        let first = "[[package]]\nname = \"gel-bench\"\nversion = \"";
        assert!(lock.contains(first));
        let sourced = lock.replacen(
            first,
            "[[package]]\nname = \"gel-bench\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nversion = \"",
            1,
        );
        assert_eq!(lock_packages(&sourced), None);
        let extra = format!("{lock}\n[[package]]\nname = \"outside\"\nversion = \"1.0.0\"\n");
        assert_ne!(lock_packages(&extra).as_ref(), Some(&members));
    }
    #[test]
    fn a_library_naming_a_socket_type_is_found() {
        let socket = SOCKET_TYPES[0].concat();
        let files = |lib: &str| {
            vec![
                ("crates/gel-core/src/lib.rs".to_string(), lib.to_string()),
                (
                    "xtask/src/isolation.rs".to_string(),
                    format!("use std::net::{socket};"),
                ),
            ]
        };
        assert_eq!(
            socket_users(&files("pub fn f() {}")),
            ["xtask/src/isolation.rs"]
        );
        assert_eq!(
            socket_users(&files(&format!("// {socket}"))),
            ["crates/gel-core/src/lib.rs", "xtask/src/isolation.rs"]
        );
    }
}
