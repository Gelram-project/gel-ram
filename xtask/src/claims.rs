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
        scope: "private resident reader, known address, 40 observations",
        input: "author-retained addressed read timing series after RAM load",
        expected: "p50 53.872 us, p95 79.640 us; no global search or semantic decision claim",
        counterexample: "addressed lookup latency presented as full Ocean search latency",
        source: "README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-source-integrity-controls",
        dimension: "integrity",
        scope: "private experimental adapter, 1000 PL/EN fragments, five controls each",
        input: "valid, corrupt, missing, wrong-source and stale-generation cases",
        expected: "1000 valid admitted and 4000 invalid rejected; no semantic correctness inferred",
        counterexample: "matching source bytes treated as proof of a claim or general understanding",
        source: "README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-single-quad-slot-ranking",
        dimension: "ranking",
        scope: "private reader, known 250k slot of logical 1M bank, 400 probes",
        input: "same probes for Single and Quad, including one empty probe in quality denominator",
        expected: "Single top1 368/400 and top10 393/400; Quad 310/400 and 361/400",
        counterexample: "slot ranking described as global million-record semantic recall or Quad superiority",
        source: "README.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-groq-supplied-source-diagnostic",
        dimension: "comparison",
        scope: "12 development claims, six per language, one timed batch per profile/language",
        input: "private GEL adapter and Groq Qwen/GPT-OSS responses; incomplete attempts retained",
        expected: "GEL all UNKNOWN; HTTP and adapter times separate; label and structure scores separate",
        counterexample: "microsecond grammar refusal claimed faster successful reasoning or N=1 used for percentiles",
        source: "docs/GEL-GROQ-DIAGNOSTIC.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-slot-ranking-update",
        dimension: "ranking",
        scope: "private reader, same 400 probes and known 250k slot, updated profile",
        input: "same probes for Single and Quad; two private pipeline changes between runs",
        expected: "Single top1 379/400 and top10 397/400; Quad 371/400 and 396/400",
        counterexample: "gain attributed to one change, or slot ranking presented as global 1M recall",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-source-field-dialogue-timing",
        dimension: "timing",
        scope: "private dialogue pilot, five articles, 13 questions, three runs of 1000 warm repeats",
        input: "resident source fields; loading, display, report writes and fallback excluded",
        expected: "field p50 2.054-4.819 us, comparison p50 19.417-21.220 us, slowest 711.017 us",
        counterexample: "medians presented as worst case, or UNHANDLED counted as an answer",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-new-article-transfer",
        dimension: "retrieval",
        scope: "private dialogue on four new PL articles, 13 questions, no timing",
        input: "69 source fields; six field outputs checked against the source HTML cells",
        expected: "6 field answers, 2 quotations, 1 limited comparison, 3 UNHANDLED, 1 UNKNOWN",
        counterexample: "result counted as 9/13 semantic accuracy or a million-record deployment",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-answer-verdict-self-read",
        dimension: "retrieval",
        scope: "private bank after merging duplicates, leaving out reference sections, keeping numbers; 50,000 stored-passage probes ranked within their slot",
        input: "verdict lead threshold 0.06484 fixed in advance from another corpus; encoder variant chosen on a separate sample",
        expected: "answered 92.8%, correct answers 99.95% (23 wrong of 46,376), wrong 0.046% of probes",
        counterexample: "self-read precision presented as natural-question accuracy, global 1M search or an unchanged bank",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-natural-question-answers",
        dimension: "retrieval",
        scope: "80 PL/EN questions written by the project's AI coding assistant, frozen before the run, all slots searched",
        input: "source verification of 128 candidates; threshold from a separate 80-question calibration set",
        expected: "top-1 right article 40/80; 11 answers, all correct; 69 UNKNOWN",
        counterexample: "11 of 11 reported as a precision rate or the set treated as an independent benchmark",
        source: "docs/MEASURED-PROGRESS.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-groq-closed-book",
        dimension: "comparison",
        scope: "the same 80 frozen PL/EN questions, one run: GEL from its bank, three Groq models closed book",
        input: "one scoring rule for all (expected fact from the source passage); UNKNOWN counted separately",
        expected: "GEL 11 correct, 0 wrong, 69 UNKNOWN; models 6-10 correct and 11-28 wrong answers each",
        counterexample: "read as a speed comparison, an engine ranking or superiority over language models",
        source: "docs/GEL-BESIDE-GROQ.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "gel-beside-groq-no-answer",
        dimension: "comparison",
        scope: "80 frozen PL/EN questions with no correct answer: 40 invented subjects, 40 false premises about bank entries",
        input: "right reply is UNKNOWN or rejecting the premise; answering anyway is wrong, including a GEL passage",
        expected: "GEL answered 0 of 40 invented and 6 of 40 false premises; models 3-22 and 2-16",
        counterexample: "read as GEL never answering a question without an answer, or as better than every model",
        source: "docs/GEL-BESIDE-GROQ-NO-ANSWER.md",
        evidence: Evidence::Deferred("MEASURED_LOCAL"),
    },
    Claim {
        id: "store-kill-during-learning",
        dimension: "persistence",
        scope: "private knowledge store growing while it learns: 100 + 100 random kills (consolidated every 25 batches or after every batch) and 7 fixed stop points",
        input: "SIGKILL at a seeded random time between 0.1 and 6 s or at a fixed point; reopen and check, then resume",
        expected: "0 of 235,712 confirmed records lost; every reopen check passed; every resume equal to the run without a kill",
        counterexample: "read as power-loss durability, as a large-store result, or as reproducible from this checkout",
        source: "docs/MEASURED-PROGRESS.md",
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
        scope: "answer-or-abstain set: 80 questions with an answer, 80 without, recorded answers of four systems",
        input: "xtask answer-bench check re-scores every recorded answer with the published and strict rules",
        expected: "the set README and the side-by-side table equal the re-scored results; passage hashes and set identity match",
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
        scope: "private engine, 128-byte sketches, three replays",
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
        id: "hardware-memory-compute",
        dimension: "mechanism",
        scope: "hardware-level memory computation",
        input: "hardware experiment and mechanism trace required",
        expected: "not established by CPU phase simulations",
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
