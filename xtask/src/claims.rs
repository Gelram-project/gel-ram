// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! One registry for scoped claims, executable counterexamples and explicit gaps.
use gel_source::{collection::Collection, context, digest};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

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
        counterexample: "read as four independent memories, facts or files, four votes, 4x storage capacity, a 4x speedup, process RAM or semantic accuracy",
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
        scope: "public gel-phase-quad Record in memory and as the public Q8DEMO01 encoder writes it",
        input: "a record of 1,024 phase bytes and a 1,024-position activity mask, encoded alone and as four copies",
        expected: "the type takes 1,152 bytes; the encoder writes it after the 12-byte header as its 1,024 phase bytes and a 128-byte mask (position j as bit j%8 of byte j/8), four copies as 4,608 bytes of four equal records; decoding gives the record back",
        counterexample: "read as process memory, as the 1,164-byte file, or as four independent memories",
        source: "docs/Q8-QUAD.md",
        evidence: Evidence::Probe(q8_record_size),
    },
    Claim {
        id: "store-file-refusals",
        dimension: "integrity",
        scope: "one public gel-store .gel v2 file of three seeded records, 448 bytes",
        input: "the saved file; each of its 512 header bits flipped; each payload byte with one bit flipped and inverted; each shorter length; a save with an older and an equal counter; a payload change with both CRC64 fields recomputed",
        expected: "both CRC64 fields equal an expectation built from the file bytes alone; the file reopens to the same records; every flipped and every cut file is refused; an older or equal counter is refused and leaves the file as it was, a newer one is saved; the change with recomputed CRC64 fields opens without complaint",
        counterexample: "read as tamper-proof storage (CRC64 is not a pin), as a result for every possible file, or as power-loss durability",
        source: "crates/gel-store/src/lib.rs",
        evidence: Evidence::Probe(store_refusals),
    },
    Claim {
        id: "reader16-one-comparison",
        dimension: "structure",
        scope: "public gel-reader reader16 on five pairs: two seeded records both ways, a record beside a near copy and beside its opposite, and two empty records",
        input: "the 16 values against the same 16 formulas computed from counts built bit by bit; a pair with two bit positions swapped in both records",
        expected: "every value equal bit for bit; contradiction is 1 minus global agreement and global agreement is the mean of the eight local views; the swapped pair is another pair of records with the same 16 values",
        counterexample: "16 judgments read as 16 independent measurements, or any value read as a measure of meaning",
        source: "docs/READER16.md",
        evidence: Evidence::Probe(reader16_comparison),
    },
    Claim {
        id: "progressive-top-k-exact",
        dimension: "ranking",
        scope: "public gel-reader top_k_progressive on a bank of 256 seeded records, 64 of them near the query, each closer than the one before, and two pairs tied",
        input: "three queries, k of 0, 1, 2, 8, 32, 256 and 300, against a reference scored bit by bit and sorted in full",
        expected: "the search that stops early and the full scan both equal the reference, entry for entry, with ties to the lower index; every record is counted once and some are left after 32 or 64 bytes",
        counterexample: "read as a speedup on every bank, as a result for every bank, or as relevance rather than bit agreement",
        source: "docs/READER16.md",
        evidence: Evidence::Probe(progressive_top_k),
    },
    Claim {
        id: "source-builder-limits",
        dimension: "interface",
        scope: "public gel-source CorpusBuilder, the limits of a push; the 64 MiB total-text limit is checked here by its constant only, its boundary in the gel-source tests",
        input: "passages of 0, 32,768 and 32,769 bytes, labels of 512 and 513 bytes, blank and control labels, a repeated address, a node with another entry or title, 50,001 records and rows up to the 16 MiB catalog limit",
        expected: "the passage, label and record limits hold at their boundary with their errors and rows stop within about 1% of the catalog limit; a refused push leaves the same bytes as a builder that never saw it; an empty builder cannot finish; the kept passage reads back byte for byte",
        counterexample: "read as limits on process memory, or the writer read as a semantic retriever or an answer generator",
        source: "docs/SOURCE-BUILDER.md",
        evidence: Evidence::Probe(source_builder_limits),
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
        scope: "source text of every .rs file of the checkout, searched for six names of the standard library: the three TCP and UDP socket types and three names of host-name resolution (its trait, the trait's method and a lookup function); a name the scan does not list, or a program a tool starts, is outside it",
        input: "the six names; file sets where a library file names a socket type or calls name resolution as counterexamples (unit test)",
        expected: "only xtask/src/isolation.rs, the isolation check, names any of them",
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
        scope: "twenty-three platform-only whole tests (19 Unix, 4 Linux)",
        input: "per-platform ci-evidence report, platform-diff of two platforms and a unit test that reads the workspace's Rust sources for platform cfg attributes",
        expected: "ran once where compiled, absent elsewhere, never counted as success there",
        counterexample: "a compile-time exclusion reported as a pass where it was not compiled",
        source: "docs/CI-EVIDENCE.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "ci-evidence-records",
        dimension: "test-scope",
        scope: "sanitized reports of the recorded revisions, hosted CI runs of one revision each",
        input: "docs/evidence-ci and the list of recorded revisions in xtask",
        expected: "nine files per revision; checksums, report format, run, commit and strict comparison all hold, and the table is recomputed from the reports",
        counterexample: "an altered report, a missing platform file, a table row other than the recomputed one, or hosted CI read as an independent second host",
        source: "docs/evidence-ci/README.md",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "saved-file-faults",
        dimension: "persistence",
        scope: "GELHIS01, GEL store v1 and v2, GELSET01 and GELSRC01 files through their public save and open paths; one machine, one file system per run",
        input: "cuts at every byte (12 stated field cuts over 1 MiB), damage that keeps the length, two child processes on one path, a save stopped at every byte, a writer killed along a save, leftover temporary names at the limit, readers beside writers",
        expected: "every damaged file refused, collections and bundles under the pin kept apart from the file; one whole file after two writers and one winner on the no-replace publisher; restart resumes to the uninterrupted bytes; reading changes no byte, time of change, permission or name",
        counterexample: "a mix of two saves accepted, part of a file visible at the path, an acknowledged save lost to a kill of the one writer, a leftover opened, or any of this read as power-loss durability",
        source: "docs/PUBLICATION-FAULT-TESTS.md",
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
        scope: "40 documented properties mapped to 91 tests",
        input: "per-platform ci-evidence TESTS.txt",
        expected: "each mapped test ran and passed once, or is a declared platform-only exclusion",
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
        id: "memory-core-selftest",
        dimension: "test-scope",
        scope: "gel-cli selftest: one run over the bit record, the store, the reader and the rebuild",
        input: "verify runs the release program; views of one record, a Reader16 count, Top-1, Top-K, a structural delta and one saved store",
        expected: "GEL_SELFTEST_V2=PASS, or the program stops with an error",
        counterexample: "one small round trip read as the exhaustive tests of those parts",
        source: "crates/gel-cli/src/main.rs",
        evidence: Evidence::Deferred("SEPARATE_GATE"),
    },
    Claim {
        id: "real-source-readout",
        dimension: "provenance",
        scope: "the gel-source source_real example on a 1,157-byte excerpt of three paragraphs, its pins fixed in the example",
        input: "verify runs it; a changed text, a changed catalog and a wrong part count",
        expected: "paragraphs 1, 2 and 3 in order from shuffled rows; each changed input rejected; SOURCE_REAL_E2E=PASS",
        counterexample: "read as factual truth checking, semantic accuracy, compression, or fourfold independent capacity",
        source: "docs/REAL-SOURCE-DEMO.md",
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
        id: "history-crash-series-recorded",
        dimension: "persistence",
        scope: "public gel-history, one recorded series of 200 trials and 20 control trials on the author's Linux machine, ext4, debug build of xtask, 2026-10-10",
        input: "240 states of one record, the whole file saved after each, SIGKILL after a seeded random 1 to 3,444 ms; a control writer rewriting the file in place",
        expected: "23,379 acknowledged states, 0 lost and 0 partial; 200 of 200 trials resumed to the uninterrupted bytes; 181 kills in the middle of a save; 14 of 20 control kills left a partial file",
        counterexample: "read as power-loss durability, as a result on another file system or with two writers, or the reopen times read as those of a release build",
        source: "docs/evidence-crash/history-crash-series-linux.txt",
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
        id: "q8-r2-recorded-runs",
        dimension: "bytes",
        scope: "the complete local rerun of the Q8 campaign after the input fix of revision 2, 2026-09-11, on synthetic fixtures and one input that is not shipped",
        input: "48 V2 runs and 24 runs beside a packed canonical-single baseline, kept with their summary",
        expected: "all 8,355,840 V2 and 2,509,056 canonical view-score comparisons passed",
        counterexample: "the transforms read as creating independent information, or the comparisons read as semantic accuracy",
        source: "docs/evidence-q8-r2/summary.txt",
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
        id: "speed-sets-rerun",
        dimension: "timing",
        scope: "xtask speed-campaign: the four re-run plans from public revisions, one machine, not pinned to cores, 2026-11-04",
        input: "195 timed cases in 33 programs, 3 rounds with the order rotated and 1 warm-up round, two clean checkouts of each baseline (A, A2)",
        expected: "each plan SPEED_CAMPAIGN=COMPLETE and 33 of 33 records SPEED_AB_CHECK=PASS; against v0.6.1 105 cases faster beyond their A/A band, 24 slower beyond it and 4 inside it; output digests equal to the recorded sets except B of speed_tree_gates",
        counterexample: "the re-run read as hardware-independent, a ratio read without its A/A band, or across programs",
        source: "docs/speed-results/2026-11-04-reruns/README.md",
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
        scope: "gel-evidence snapshots and gel-source bundles, published through the same no-replace publisher, under a power cut",
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
    Claim {
        id: "queries-per-second",
        dimension: "timing",
        scope: "a rate of queries per second taken from internal update-rate measurements",
        input: "a measured query rate on a stated task; none is published",
        expected: "not established: the public pages give no such rate and no such conversion",
        counterexample: "an internal update rate read as queries per second",
        source: "docs/VERIFIED-RESULTS.md",
        evidence: Evidence::Deferred("NOT_ESTABLISHED"),
    },
];

/// A sentence of another public file that names registry rows: the file, its
/// words (a line break or a run of spaces counts as one space), which must occur
/// exactly once in the file, and each row it stands for with the state the row
/// must keep. Rewording the sentence or
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
        words: "The text AI is a goal, not a result ([the goal](docs/GOAL.md)). Hardware-level memory computation and power-cut durability of snapshots and of the record history are `NOT_ESTABLISHED` ([claim registry](docs/CLAIMS.md))",
        rows: &[
            ("text-ai-goal", "NOT_ESTABLISHED"),
            ("hardware-memory-compute", "NOT_ESTABLISHED"),
            ("snapshot-power-loss", "NOT_ESTABLISHED"),
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
    // The limit of each README table cell and each number a reader will quote.
    Citation {
        file: "README.md",
        words: "`.gel` v2 files carry a CRC64 over the header and over the payload; in the tests every single header bit flip, every payload byte flip and every truncation is rejected, and so is an older or equal save counter",
        rows: &[("store-file-refusals", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "README.md",
        words: "One comparison of two records returns 16 judgments at once, not 16 independent measurements; a search that stops early returns exactly the same top-K matches as a full scan",
        rows: &[
            ("reader16-one-comparison", "EXECUTABLE_CHECK"),
            ("progressive-top-k-exact", "EXECUTABLE_CHECK"),
        ],
    },
    Citation {
        file: "README.md",
        words: "`cargo run --locked --offline --release -p gel-cli -- selftest` checks the first four parts in one run (`GEL_SELFTEST_V2=PASS`)",
        rows: &[("memory-core-selftest", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "Four views are not four independent memories, four votes or a speedup.",
        rows: &[
            ("q8-one-record-four-views", "SEPARATE_GATE"),
            ("q8-record-size", "EXECUTABLE_CHECK"),
        ],
    },
    Citation {
        file: "README.md",
        words: "Meaning is not measured; one plain copy can still be read faster than the shared record",
        rows: &[
            ("q8-semantic-accuracy", "NOT_ESTABLISHED"),
            ("q8-recorded-runs", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "README.md",
        words: "Your bytes as they are, not knowledge printing: no meaning and no search",
        rows: &[("q8-literal-record", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "README.md",
        words: "A finite set of changes, not every possible one",
        rows: &[("format-mutations", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "Recorded on Linux; `verify` runs it on Unix hosts only; a process kill, not a power cut",
        rows: &[
            ("crash-series-no-acknowledged-loss", "SEPARATE_GATE"),
            ("crash-series-recorded", "MEASURED_LOCAL"),
            ("snapshot-power-loss", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "README.md",
        words: "200 random process kills in one recorded run: 0 of 2,683 confirmed snapshots lost.",
        rows: &[("crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "README.md",
        words: "A matching hash shows these are your bytes, not that the source is true or private",
        rows: &[
            ("sha256-fips-vectors", "EXECUTABLE_CHECK"),
            ("source-roundtrip", "EXECUTABLE_CHECK"),
        ],
    },
    Citation {
        file: "README.md",
        words: "It is phrase lookup, not question answering.",
        rows: &[
            ("no-cross-document", "EXECUTABLE_CHECK"),
            ("text-ai-goal", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "README.md",
        words: "The map shows the rule CI enforces, not the outcome of one run",
        rows: &[("property-map", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "103 cases are faster beyond their A/A band, 18 slower beyond it (SHA-256 alone at 0.28 and 0.39) and 12 inside it.",
        rows: &[("speed-set-against-v0.6.1", "MEASURED_LOCAL")],
    },
    Citation {
        file: "README.md",
        words: "105 faster, 4 inside, 24 slower on 2026-11-04",
        rows: &[("speed-sets-rerun", "MEASURED_LOCAL")],
    },
    Citation {
        file: "README.md",
        words: "every run printed the same output digest, apart from the whole-tree gates, which count the files of their own tree",
        rows: &[
            ("speed-set-against-v0.6.1", "MEASURED_LOCAL"),
            ("speed-set-quiet-machine", "MEASURED_LOCAL"),
            ("speed-sets-rerun", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "README.md",
        words: "the target Q8 Quad quantizer as of their dates; only their labels say what each holds.",
        rows: &[("commitments-fixed-in-time", "SEPARATE_GATE")],
    },
    Citation {
        file: "README.md",
        words: "The author's own measurements are in [measured progress](docs/MEASURED-PROGRESS.md); nobody else has re-run them.",
        rows: &[("independent-reproduction", "NOT_VERIFIED")],
    },
    Citation {
        file: "docs/GOAL.md",
        words: "Status: goal: knowledge updates without training.",
        rows: &[("text-ai-goal", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/GOAL.md",
        words: "Status: goal: answers in its own words, not yet measured; today the document tools return exact quotations.",
        rows: &[("text-ai-goal", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/GOAL.md",
        words: "Status: goal: one conversation across both languages, not yet measured.",
        rows: &[("text-ai-goal", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/GOAL.md",
        words: "Status: process killed mid-write: 0 confirmed entries lost in the recorded crash series; not a power-cut test.",
        rows: &[
            ("crash-series-recorded", "MEASURED_LOCAL"),
            ("snapshot-power-loss", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "Four views are not four independent facts or 4× storage capacity",
        rows: &[("q8-one-record-four-views", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- fourfold independent capacity from four reversible views.",
        rows: &[("q8-one-record-four-views", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "**1152 bytes per record**, excluding runtime tables and buffers.",
        rows: &[("q8-record-size", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- 8,355,840 V2 view-score comparisons passing; - 2,509,056 canonical-baseline comparisons passing;",
        rows: &[("q8-r2-recorded-runs", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- no claim that the coordinate transforms create independent information.",
        rows: &[("q8-r2-recorded-runs", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "| 1M | 41119 | 100 | 72.427 ms | 81.018 ms | 82.859 ms | 90.949 ms | | 1M | 61141 | 100 | 71.962 ms | 79.860 ms | 80.576 ms | 88.227 ms | | 10M | 41119 | 100 | 615.428 ms | 684.888 ms | 710.313 ms | 734.085 ms | | 10M | 61141 | 100 | 704.995 ms | 757.900 ms | 783.038 ms | 810.236 ms |",
        rows: &[("ocean-full-scan", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "With only 100 observations per long run, the project does **not** claim stable p99.9, p99.99 or p99.999 tails from this dataset.",
        rows: &[("extreme-tail-latency", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- stable extreme-tail latency without sufficient observations;",
        rows: &[("extreme-tail-latency", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- unrestricted semantic AI accuracy;",
        rows: &[
            ("text-ai-goal", "NOT_ESTABLISHED"),
            ("q8-semantic-accuracy", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "- 30M queries/s or any conversion from internal update-rate measurements to queries/s;",
        rows: &[("queries-per-second", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "they do not establish an end-to-end advantage over this baseline.",
        rows: &[
            ("commercial-advantage", "NOT_ESTABLISHED"),
            ("gel-sketch-search", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "Re-scores recorded answers; it does not re-run GEL, the models or the engines",
        rows: &[("answer-bench-rescoring", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "| Not independent second-host acceptance |",
        rows: &[("independent-reproduction", "NOT_VERIFIED")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "| Not a power-cut test |",
        rows: &[
            ("publication-os-faults", "SEPARATE_GATE"),
            ("snapshot-power-loss", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "| Platform-only tests are declared per platform |",
        rows: &[("platform-exclusions", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "207 trials: 0 of 235,712 confirmed records lost; every reopen check passed; every resume equal to a run without a kill | A process kill, not a power cut; a small store",
        rows: &[("store-kill-during-learning", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/VERIFIED-RESULTS.md",
        words: "99% precision is not established across v4 to v7",
        rows: &[("gel-beside-bm25-v7", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/ILLUSTRATED-GUIDE.md",
        words: "Four equivalent views do not store four arbitrary, independent files in one record.",
        rows: &[("q8-one-record-four-views", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/ILLUSTRATED-GUIDE.md",
        words: "Numeric exactness after quantization does not make quantization itself lossless.",
        rows: &[("numeric-loss", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/ILLUSTRATED-GUIDE.md",
        words: "These are not tests of full semantic understanding, universal acceleration, fourfold independent capacity, hardware-level memory computation or hardware identity.",
        rows: &[
            ("q8-semantic-accuracy", "NOT_ESTABLISHED"),
            ("q8-one-record-four-views", "SEPARATE_GATE"),
            ("hardware-memory-compute", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/ILLUSTRATED-GUIDE.md",
        words: "the reported timing campaign is CPU-only.",
        rows: HARDWARE,
    },
    Citation {
        file: "docs/ILLUSTRATED-GUIDE.md",
        words: "Reference implementations and CI are not an independent third-party scientific validation.",
        rows: &[("independent-reproduction", "NOT_VERIFIED")],
    },
    Citation {
        file: "docs/SOURCE-BUILDER.md",
        words: "- Labels: nonblank, no control characters, at most 512 UTF-8 bytes each",
        rows: &[("source-builder-limits", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/SOURCE-BUILDER.md",
        words: "- Passages: 1..32768 bytes; total text <=64 MiB, catalog <=16 MiB and <=50000 records. Those are serialized-content limits, **not** total process RSS limits.",
        rows: &[("source-builder-limits", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/SOURCE-BUILDER.md",
        words: "This is not a hardware power-cut test.",
        rows: &[("snapshot-power-loss", "NOT_ESTABLISHED")],
    },
    Citation {
        file: "docs/SOURCE-BUILDER.md",
        words: "This is a **source-catalog writer**, not a private encoder, Q8 numeric quantizer, multimedia analyzer, semantic retriever or answer generator.",
        rows: &[("source-builder-limits", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/SOURCE-BUILDER.md",
        words: "It does not prove truth or add fourfold independent capacity.",
        rows: &[("q8-one-record-four-views", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/REAL-SOURCE-DEMO.md",
        words: "The success marker is `SOURCE_REAL_E2E=PASS`.",
        rows: &[("real-source-readout", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/REAL-SOURCE-DEMO.md",
        words: "This demonstrates exact approved-source readout, **not** factual truth checking, semantic accuracy, autonomous reasoning, compression, quantization quality or fourfold independent information capacity.",
        rows: &[
            ("real-source-readout", "SEPARATE_GATE"),
            ("q8-one-record-four-views", "SEPARATE_GATE"),
        ],
    },
    Citation {
        file: "docs/REAL-SOURCE-DEMO.md",
        words: "There are no speedup or tokens-per-second claims from this tiny fixture.",
        rows: &[("real-source-readout", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "| 200 | 2,683 | **0** | **0** | 200 | 153 | 2.3 / 4.5 / 4.5 ms |",
        rows: &[("crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "Its own SHA-256 ran at 0.28 and 0.39 of the speed of v0.6.1, which used that crate, on 2026-10-20, 0.26 and 0.35 in the re-run of 2026-11-04",
        rows: &[
            ("speed-set-against-v0.6.1", "MEASURED_LOCAL"),
            ("speed-sets-rerun", "MEASURED_LOCAL"),
        ],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "`xtask verify` runs 5 trials on Unix hosts.",
        rows: &[("crash-series-no-acknowledged-loss", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "It tests the public collection tool, not the private implementation.",
        rows: &[("crash-series-no-acknowledged-loss", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "| 200 | 23,379 | **0** | **0** | 200 | 181 | 1.6 / 3.9 / 5.2 ms |",
        rows: &[("history-crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "In the control, 14 of 20 kills left a partial file",
        rows: &[("history-crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "`xtask verify` runs 5 trials on Unix hosts, without the control.",
        rows: &[("record-history-kill-series", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/CRASH-SERIES.md",
        words: "One writer per file: this series does not race two processes on one path.",
        rows: &[("record-history-kill-series", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "Storage grows with every appended state.",
        rows: &[("record-history-exact", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "Synthetic walks show the format, not how real records change.",
        rows: &[("record-history-exact", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "CRC64 detects accidental change; whoever rewrites the file can recompute it.",
        rows: &[("record-history-exact", "EXECUTABLE_CHECK")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "of GELHIS01 changes one 338-byte file of four states, not every possible file.",
        rows: &[("history-format-mutations", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/RECORD-HISTORY.md",
        words: "A process kill during saving is tested by the [kill series](CRASH-SERIES.md#the-record-history-one-file-replaced-while-it-grows); a power cut is not.",
        rows: &[
            ("record-history-kill-series", "SEPARATE_GATE"),
            ("record-history-durability", "NOT_ESTABLISHED"),
        ],
    },
    Citation {
        file: "docs/COMMITMENTS.md",
        words: "It says nothing about what the contents are or whether they are correct.",
        rows: &[("commitments-fixed-in-time", "SEPARATE_GATE")],
    },
    Citation {
        file: "docs/COMMITMENTS.md",
        words: "`COMMITMENT_CHECK=FAIL first=PATH` names the first path in byte order that differs, and the command exits with status 2.",
        rows: &[("commitments-fixed-in-time", "SEPARATE_GATE")],
    },
    // The file behind a recorded number, so that the page and the record cannot part.
    Citation {
        file: "docs/evidence-crash/crash-series-linux.txt",
        words: "CRASH_SERIES=PASS trials=200 acknowledged=2683 acknowledged_lost=0 partial=0 resumed=200 killed_mid_publication=153",
        rows: &[("crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/evidence-crash/history-crash-series-linux.txt",
        words: "HISTORY_CRASH_SERIES=PASS trials=200 acknowledged=23379 acknowledged_lost=0 partial=0 resumed=200",
        rows: &[("history-crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/evidence-crash/history-crash-series-linux.txt",
        words: "control_trials=20 control_partial=14",
        rows: &[("history-crash-series-recorded", "MEASURED_LOCAL")],
    },
    Citation {
        file: "docs/evidence-q8-r2/summary.txt",
        words: "V2_view_comparisons=8355840 EVIDENCE_view_comparisons=2509056",
        rows: &[("q8-r2-recorded-runs", "MEASURED_LOCAL")],
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
/// The size of the type and the bytes the public encoder writes, against the
/// numbers the README gives.
fn q8_record_size() -> Result<bool, String> {
    use gel_phase_quad::{fixture, grid::DIM, Record};
    let phase: [u8; DIM] = std::array::from_fn(|j| (j * 7 % 256) as u8);
    let mask: [bool; DIM] = std::array::from_fn(|j| j % 3 == 0);
    let record = Record::new(phase, &mask);
    let one = fixture::encode(std::slice::from_ref(&record))?;
    let four = fixture::encode(&vec![record.clone(); 4])?;
    let Some((body, masks)) = one.get(12..).map(|b| b.split_at(b.len().min(1024))) else {
        return Ok(false);
    };
    let back = fixture::decode(&one)?;
    Ok(std::mem::size_of::<Record>() == 1152
        && one.len() == 12 + 1152
        && body == phase
        && masks.len() == 128
        && (0..DIM).all(|j| (masks[j / 8] >> (j % 8)) & 1 == u8::from(mask[j]))
        && four.len() == 12 + 4608
        && four[12..].chunks(1152).all(|r| r == &one[12..])
        && back.len() == 1
        && *back[0].phase() == phase
        && back[0].active_mask() == mask)
}
/// A directory of its own for one probe, removed when the probe ends. `check`
/// and the unit tests run a probe at the same time in one process, so the
/// process id alone does not keep them apart.
struct ProbeDir(PathBuf);
impl ProbeDir {
    fn new() -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("gel-claims-{}-{n}", std::process::id()));
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        Ok(Self(dir))
    }
}
impl Drop for ProbeDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// The README row of gel-store on a saved file, with the two CRC64 fields
/// rebuilt from the file bytes alone; the same rebuilding shows what they do
/// not give: a change with both fields recomputed opens.
fn store_refusals() -> Result<bool, String> {
    use gel_core::{crc64_ecma, GelError, Generation};
    use gel_store::{RamStore, HEADER_BYTES};
    let text = |e: GelError| e.to_string();
    let dir = ProbeDir::new()?;
    let path = dir.0.join("store.gel");
    let orbs = vec![seeded_orb(300), seeded_orb(400), seeded_orb(500)];
    let save = |generation: u64| RamStore::from_orbs(orbs.clone(), Generation(generation));
    save(7).write_atomic(&path).map_err(text)?;
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    if bytes.len() != HEADER_BYTES + 3 * 128 {
        return Ok(false);
    }
    let seal = |file: &mut Vec<u8>| {
        let payload = crc64_ecma(&file[HEADER_BYTES..]);
        file[40..48].copy_from_slice(&payload.to_le_bytes());
        file[48..56].fill(0);
        let header = crc64_ecma(&file[..HEADER_BYTES]);
        file[48..56].copy_from_slice(&header.to_le_bytes());
    };
    let mut sealed = bytes.clone();
    seal(&mut sealed);
    let opens = |file: &[u8]| -> Result<Option<RamStore>, String> {
        fs::write(&path, file).map_err(|e| e.to_string())?;
        Ok(RamStore::open_verified(&path).ok())
    };
    let mut ok = sealed == bytes
        && opens(&bytes)?.is_some_and(|s| s.orbs() == orbs && s.header().generation.0 == 7);
    let flipped = |index: usize, mask: u8| -> Result<bool, String> {
        let mut changed = bytes.clone();
        changed[index] ^= mask;
        Ok(opens(&changed)?.is_none())
    };
    for bit in 0..HEADER_BYTES * 8 {
        ok &= flipped(bit / 8, 1 << (bit % 8))?;
    }
    for index in HEADER_BYTES..bytes.len() {
        ok &= flipped(index, 1 << (index % 8))? && flipped(index, 0xFF)?;
    }
    for len in 0..bytes.len() {
        ok &= opens(&bytes[..len])?.is_none();
    }
    let mut forged = bytes.clone();
    forged[HEADER_BYTES + 13] ^= 1;
    seal(&mut forged);
    ok &= opens(&forged)?.is_some_and(|s| s.orbs() != orbs);
    // An older and an equal counter leave the file, a newer one replaces it.
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    for attempted in [6, 7] {
        ok &= matches!(
            save(attempted).write_if_newer(&path),
            Err(GelError::GenerationRollback { current: 7, attempted: a }) if a == attempted
        );
    }
    ok &= fs::read(&path).map_err(|e| e.to_string())? == bytes;
    save(8).write_if_newer(&path).map_err(text)?;
    Ok(ok
        && RamStore::open_verified(&path)
            .map_err(text)?
            .header()
            .generation
            == Generation(8))
}
fn record_bit(orb: &gel_orb::Orb1024, bit: usize) -> u64 {
    (orb.words()[bit / 64] >> (bit % 64)) & 1
}
/// The 16 values of Reader16 from counts built bit by bit, without the kernel.
fn reader16_formulas(a: &gel_orb::Orb1024, b: &gel_orb::Orb1024) -> [f32; 16] {
    let (mut n, mut local) = ([0u32; 4], [0u32; 8]);
    for bit in 0..1024 {
        let (x, y) = (record_bit(a, bit), record_bit(b, bit));
        n[(x * 2 + y) as usize] += 1;
        local[bit / 128] += u32::from(x == y);
    }
    let [n00, n01, n10, n11] = n;
    let part = |top: u32, bottom: u32| {
        if bottom == 0 {
            1.0
        } else {
            top as f32 / bottom as f32
        }
    };
    let product =
        f64::from(n11 + n10) * f64::from(n01 + n00) * f64::from(n11 + n01) * f64::from(n10 + n00);
    let phi = if product == 0.0 {
        0.0
    } else {
        ((f64::from(n11) * f64::from(n00) - f64::from(n10) * f64::from(n01)) / product.sqrt())
            as f32
    };
    let mut want = [0f32; 16];
    want[..8].copy_from_slice(&[
        (n00 + n11) as f32 / 1024.0,
        part(n11, n11 + n10 + n01),
        part(2 * n11, 2 * n11 + n10 + n01),
        part(n11, n11 + n10),
        part(n11, n11 + n01),
        phi,
        (n10 + n01) as f32 / 1024.0,
        (n10 as f32 - n01 as f32) / 1024.0,
    ]);
    for (slot, count) in want[8..].iter_mut().zip(local) {
        *slot = count as f32 / 128.0;
    }
    want
}
/// 16 values from one pair of records: each equals its formula, and they are
/// not independent of each other, which is the limit the README states.
fn reader16_comparison() -> Result<bool, String> {
    use gel_orb::Orb1024;
    use gel_reader::{reader16, READER16_NAMES};
    let (a, b) = (seeded_orb(600), seeded_orb(700));
    let mut near = a;
    near.words_mut()[3] ^= 0xFF00;
    let opposite = Orb1024::from_words(a.words().map(|w| !w));
    let pairs = [
        (a, b),
        (b, a),
        (a, near),
        (a, opposite),
        (Orb1024::ZERO, Orb1024::ZERO),
    ];
    let mut ok = READER16_NAMES.len() == 16;
    for (x, y) in &pairs {
        let got = reader16(x, y).values;
        let want = reader16_formulas(x, y);
        ok &= got
            .iter()
            .zip(&want)
            .all(|(g, w)| g.to_bits() == w.to_bits())
            && got[0] + got[6] == 1.0
            && got[8..].iter().sum::<f32>() / 8.0 == got[0];
    }
    // The same counts from other records: two positions of one local view,
    // holding different bits of `a`, swapped in both records.
    let (p, q) = (
        0,
        1 + (1..128)
            .position(|i| record_bit(&a, i) != record_bit(&a, 0))
            .ok_or("a flat local view")?,
    );
    let swap = |orb: &Orb1024| {
        let mut out = *orb;
        let (bp, bq) = (record_bit(orb, p), record_bit(orb, q));
        out.words_mut()[p / 64] ^= (bp ^ bq) << (p % 64);
        out.words_mut()[q / 64] ^= (bp ^ bq) << (q % 64);
        out
    };
    let (moved_a, moved_b) = (swap(&a), swap(&b));
    let same = reader16(&moved_a, &moved_b).values == reader16(&a, &b).values;
    Ok(ok && same && moved_a != a)
}
/// A search that stops early against a reference that scores every bit and
/// sorts everything. Each near record is one bit closer to the query than the
/// one before it, and the closest differs from it only in the first 32 bytes,
/// so its bound there equals its score: a bound that leaves out a score one
/// above the k-th drops the true best match.
fn progressive_top_k() -> Result<bool, String> {
    use gel_reader::{top_k, top_k_progressive};
    let query = seeded_orb(4000);
    let mut bank: Vec<_> = (0..256usize)
        .map(|i| {
            if i % 4 != 0 {
                return seeded_orb(5000 + i as u64 * 32);
            }
            let mut near = query;
            for j in 0..64 - i / 4 {
                // 37 is odd, so these positions of one 256-bit stretch differ.
                let bit = ((i / 4 + 1) % 3) * 256 + (i * 91 + j * 37) % 256;
                near.words_mut()[bit / 64] ^= 1 << (bit % 64);
            }
            near
        })
        .collect();
    bank[200] = bank[8];
    bank[252] = bank[248];
    let mut ok = true;
    let mut left_early = 0;
    for q in [query, bank[1], bank[100]] {
        let mut sorted: Vec<(usize, u16)> = bank
            .iter()
            .enumerate()
            .map(|(index, orb)| {
                let same = (0..1024).filter(|&b| record_bit(orb, b) == record_bit(&q, b));
                (index, same.count() as u16)
            })
            .collect();
        sorted.sort_unstable_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
        for k in [0, 1, 2, 8, 32, 256, 300] {
            let want = &sorted[..k.min(sorted.len())];
            let (got, stats) = top_k_progressive(&bank, &q, k);
            let early = stats.rejected_after_32b + stats.rejected_after_64b;
            left_early += early;
            ok &= got == want
                && top_k(&bank, &q, k) == want
                && stats.candidates == bank.len()
                && (k == 0 || stats.full_128b_scores + early == bank.len());
        }
    }
    Ok(ok && left_early > 0)
}
/// The limits of the contract in docs/SOURCE-BUILDER.md, each at its boundary.
fn source_builder_limits() -> Result<bool, String> {
    use gel_source::{
        Address, CorpusBuilder, Error, MAX_CATALOG, MAX_PASSAGE, MAX_RECORDS, MAX_TEXT,
    };
    let at = |node, role| Address { node, role };
    let text = |e: Error| format!("{e:?}");
    let longest = "ł".repeat(256);
    let quote = "Zażo\u{301}łć\r\n🦀";
    let passage = "x".repeat(32_768);
    let mut kept = CorpusBuilder::new();
    let mut seen = CorpusBuilder::new();
    let good: [(Address, u64, &str, &str, &str); 2] = [
        (at(1, 1), 1, &longest, &longest, quote),
        (at(1, 2), 1, &longest, "Lead", &passage),
    ];
    let mut ok = (MAX_PASSAGE, MAX_RECORDS, MAX_CATALOG, MAX_TEXT)
        == (32_768, 50_000, 16 << 20, 64 << 20)
        && CorpusBuilder::new().finish().err() == Some(Error::Format);
    for (address, entry, title, section, body) in good {
        ok &= kept.push(address, entry, title, section, body) == Ok(());
        ok &= seen.push(address, entry, title, section, body) == Ok(());
    }
    let long_label = "a".repeat(513);
    let long_passage = "x".repeat(32_769);
    let refused: [(Address, u64, &str, &str, &str, Error); 8] = [
        (at(2, 1), 2, "T", "S", "", Error::Limit),
        (at(2, 1), 2, "T", "S", &long_passage, Error::Limit),
        (at(2, 1), 2, &long_label, "S", "x", Error::Limit),
        (at(2, 1), 2, "T", &long_label, "x", Error::Limit),
        (at(2, 1), 2, " \t", "S", "x", Error::Format),
        (at(2, 1), 2, "T", "a\nb", "x", Error::Format),
        (at(1, 1), 1, &longest, "S", "x", Error::Duplicate),
        (at(1, 3), 9, &longest, "S", "x", Error::Format),
    ];
    for (address, entry, title, section, body, error) in refused {
        ok &= seen.push(address, entry, title, section, body) == Err(error);
    }
    ok &= seen.push(at(1, 3), 1, "Another title", "S", "x") == Err(Error::Format);
    let (kept, seen) = (kept.finish().map_err(text)?, seen.finish().map_err(text)?);
    ok &= seen.text_bytes() == kept.text_bytes() && seen.catalog_bytes() == kept.catalog_bytes();
    let loaded = seen.load().map_err(text)?;
    ok &= loaded.quote(at(1, 1)).map_err(text)?.quote() == quote;
    let mut records = CorpusBuilder::new();
    for role in 0..50_000 {
        ok &= records.push(at(1, role), 1, "T", "S", "x") == Ok(());
    }
    ok &= records.push(at(1, 50_000), 1, "T", "S", "x") == Err(Error::Limit);
    // A row of two longest labels holds 2,048 hexadecimal digits, 64 of the
    // digest and 8 separators: at least 2,125 bytes and fewer than 2,150 here.
    let mut rows = CorpusBuilder::new();
    let mut row = |role| rows.push(at(1, role), 1, &longest, &longest, "x");
    let mut accepted = 0;
    while accepted < 8_000 && row(accepted).is_ok() {
        accepted += 1;
    }
    ok &= (MAX_CATALOG as u64 / 2_150..=MAX_CATALOG as u64 / 2_125).contains(&accepted)
        && row(accepted) == Err(Error::Limit);
    Ok(ok)
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
/// The standard socket types and the names of name resolution, split so that
/// this file does not name them.
const NETWORK_NAMES: [[&str; 2]; 6] = [
    ["Tcp", "Stream"],
    ["Tcp", "Listener"],
    ["Udp", "Socket"],
    ["ToSocket", "Addrs"],
    ["to_socket", "_addrs"],
    ["lookup", "_host"],
];
/// The files, of (path relative to the root with `/`, text), that name one of
/// the network names.
fn socket_users(files: &[(String, String)]) -> Vec<String> {
    let needles: Vec<String> = NETWORK_NAMES.iter().map(|p| p.concat()).collect();
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
        match words(&read(c.file)?).matches(&words(c.words)).count() {
            1 => {}
            0 => {
                return Err(format!(
                    "{} no longer says \"{}\"; update CITED in xtask/src/claims.rs",
                    c.file, c.words
                ))
            }
            n => {
                return Err(format!(
                    "{} says \"{}\" {n} times; a citation must name one place",
                    c.file, c.words
                ))
            }
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

/// In the lines outside a table of a paragraph that links the registry, a code
/// span shaped like a row id must be a registry id and one shaped like a state
/// must be a state. Returns the number of paragraphs with such lines.
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
            if !paragraph.contains("CLAIMS.md") {
                continue;
            }
            // Table rows name commands and programs in code; the other lines
            // of the paragraph are read.
            let prose: Vec<&str> = paragraph
                .lines()
                .filter(|l| !l.trim_start().starts_with('|'))
                .collect();
            if prose.is_empty() {
                continue;
            }
            paragraphs += 1;
            for span in prose.join("\n").split('`').skip(1).step_by(2) {
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
    fn every_citation_fails_when_its_sentence_goes_or_its_row_moves() {
        // Each sentence reworded by one letter in a copy of its file.
        for c in CITED {
            let changed = words(c.words).replacen(' ', "  x", 1);
            let reworded = |file: &str| {
                let text = words(&read_tree(file)?);
                Ok(if file == c.file {
                    text.replace(&words(c.words), &changed)
                } else {
                    text
                })
            };
            let one = std::slice::from_ref(c);
            assert!(
                check_citations(CLAIMS, one, reworded).is_err(),
                "{}",
                c.words
            );
            // Each row it stands for moved to another state.
            for (id, mode) in c.rows {
                let other = if *mode == "NOT_VERIFIED" {
                    "NOT_ESTABLISHED"
                } else {
                    "NOT_VERIFIED"
                };
                let mut moved: Vec<Claim> = CLAIMS.to_vec();
                moved.iter_mut().find(|r| r.id == *id).unwrap().evidence =
                    Evidence::Deferred(other);
                let err = check_citations(&moved, one, read_tree).unwrap_err();
                assert!(err.contains(id), "{err}");
            }
        }
    }
    #[test]
    fn probe_directories_do_not_collide() {
        let (a, b) = (ProbeDir::new().unwrap(), ProbeDir::new().unwrap());
        assert_ne!(a.0, b.0);
        let path = a.0.clone();
        drop(a);
        assert!(!path.exists());
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
        // Table rows and paragraphs without the link are not read.
        let table = "| a | [claims](CLAIMS.md) `no-such-row` |";
        let unlinked = "`no-such-row` is `NOT_A_STATE`.";
        assert_eq!(check_paragraphs(CLAIMS, &file(table)), Ok(0));
        assert_eq!(check_paragraphs(CLAIMS, &file(unlinked)), Ok(0));
        // The text lines next to a table row are read, wherever the link is.
        let beside = |id: &str| {
            format!("| a | `gel-backup` |\nRegistry row: `{id}`.\n| [claims](CLAIMS.md) |")
        };
        assert_eq!(
            check_paragraphs(CLAIMS, &file(&beside("q8-literal-record"))),
            Ok(1)
        );
        assert!(check_paragraphs(CLAIMS, &file(&beside("q8-literal-recrod"))).is_err());
    }
    #[test]
    fn a_citation_must_name_one_place() {
        let first = &CITED[0];
        let twice = |file: &str| {
            let text = read_tree(file)?;
            Ok(if file == first.file {
                format!("{text}\n{}\n", first.words)
            } else {
                text
            })
        };
        let err = check_citations(CLAIMS, CITED, twice).unwrap_err();
        assert!(err.contains("2 times"), "{err}");
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
        let socket = NETWORK_NAMES[0].concat();
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
        // Name resolution names no socket type and is found too.
        let resolve = NETWORK_NAMES[4].concat();
        assert_eq!(
            socket_users(&files(&format!(
                "pub fn f() {{ let _ = (\"example.org\", 1).{resolve}(); }}"
            ))),
            ["crates/gel-core/src/lib.rs", "xtask/src/isolation.rs"]
        );
    }
}
