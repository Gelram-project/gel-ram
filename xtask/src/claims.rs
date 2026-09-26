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

const CLAIMS: &[Claim] = &[
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
        expected: "allocator and peak-RSS measurement still required",
        counterexample: "removed temporary Vec called measured RAM saving",
        source: "docs/MUTATION-COMPARISON.md",
        evidence: Evidence::Deferred("NOT_MEASURED"),
    },
    Claim {
        id: "media-full-review",
        dimension: "presentation",
        scope: "three historical MP4 files",
        input: "complete timeline and source-log comparison",
        expected: "full visual and privacy review",
        counterexample: "decode PASS substituted for visual review",
        source: "docs/MEDIA-DECODE-REVIEW.md",
        evidence: Evidence::Deferred("NOT_VERIFIED"),
    },
    Claim {
        id: "physical-refresh-compute",
        dimension: "mechanism",
        scope: "physical DRAM-side computation",
        input: "hardware experiment and mechanism trace required",
        expected: "not established by CPU phase simulations",
        counterexample: "renamed CPU baseline presented as hardware refresh compute",
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
            Evidence::Deferred(s) => s,
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
        assert!(validate_table(&doc.replacen("NOT_MEASURED", "PASS", 1)).is_err());
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
    fn documentation_and_registry_agree() {
        check(crate::workspace_root().unwrap()).unwrap();
    }
}
