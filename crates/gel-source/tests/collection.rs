use gel_source::{collection::*, digest, document};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        loop {
            let p = std::env::temp_dir().join(format!(
                "gel-collection-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("{e}"),
            }
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn bank() -> Collection {
    let mut c = Collection::new();
    c.add("Polish", "Zażo\u{301}łć gęślą.\r\nŁÓDŹ").unwrap();
    c.add("English", "RAM is volatile.\nCheck exact source bytes.")
        .unwrap();
    c
}
#[test]
fn streamed_root_matches_serialized_oracle_through_mutation_and_reload() {
    let mut c = Collection::new();
    for step in 0..240 {
        let ids: Vec<_> = c.documents().map(|d| d.id()).collect();
        match step % 4 {
            0 | 1 => {
                c.add(
                    &format!("doc {step}"),
                    &format!("Zażółć 🦀\r\n{step}\n{}", "abc".repeat(step)),
                )
                .unwrap();
            }
            2 if !ids.is_empty() => {
                c.replace(ids[step % ids.len()], &format!("replacement {step}"))
                    .unwrap();
            }
            3 if !ids.is_empty() => {
                c.remove(ids[step % ids.len()]).unwrap();
            }
            _ => {}
        }
        let bytes = c.to_bytes();
        assert_eq!(c.root(), digest(&bytes));
        let loaded = Collection::from_bytes(&bytes, c.root()).unwrap();
        assert_eq!(loaded.to_bytes(), bytes);
        assert_eq!(loaded.root(), c.root());
        c = loaded;
    }
}
#[test]
fn no_cross_document_phrase() {
    let mut c = Collection::new();
    c.add("A", "not ").unwrap();
    c.add("B", "approved").unwrap();
    assert_eq!(c.search("not approved").unwrap().status(), "UNKNOWN");
    let hit = c.search("approved").unwrap().hits.remove(0);
    assert_eq!(hit.document_id(), 2);
    c.validate(&hit).unwrap();
}
#[test]
fn unicode_offsets_and_duplicate_titles_keep_identity() {
    let mut c = bank();
    c.add("Polish", "ŁÓDŹ").unwrap();
    let s = c.search("łódź").unwrap();
    assert_eq!(s.hits.len(), 2);
    for h in s.hits {
        c.validate(&h).unwrap();
        assert_eq!(&c.get(h.document_id()).unwrap().text()[h.span()], h.quote());
    }
    assert_eq!(c.search("zażółć gęślą").unwrap().hits.len(), 1);
}
#[test]
fn replace_delete_and_unrelated_add_invalidate_hits() {
    let mut c = bank();
    let h = c.search("RAM").unwrap().hits.remove(0);
    c.replace(2, "RAM is not a disk.").unwrap();
    assert!(c.validate(&h).is_err());
    let h = c.search("RAM").unwrap().hits.remove(0);
    c.remove(2).unwrap();
    assert!(c.validate(&h).is_err());
    assert_eq!(c.search("RAM").unwrap().status(), "UNKNOWN");
    let h = c.search("łódź").unwrap().hits.remove(0);
    c.add("third", "new").unwrap();
    assert!(c.validate(&h).is_err());
}
#[test]
fn rejected_mutations_are_transactional() {
    let mut c = bank();
    let before = c.to_bytes();
    assert!(c.add("bad\nlabel", "text").is_err());
    assert!(c.add("valid", "").is_err());
    assert!(c.replace(999, "x").is_err());
    assert!(c.remove(999).is_err());
    assert!(c.replace(1, "").is_err());
    assert_eq!(before, c.to_bytes());
}
#[test]
fn ids_and_revision_survive_empty_bank_roundtrip() {
    let mut c = bank();
    c.remove(1).unwrap();
    c.remove(2).unwrap();
    let b = c.to_bytes();
    let mut restored = Collection::from_bytes(&b, c.root()).unwrap();
    assert_eq!(restored.revision(), 4);
    assert_eq!(restored.add("new", "text").unwrap(), 3);
    assert_eq!(restored.revision(), 5);
}
#[test]
fn exact_canonical_roundtrip_and_citation() {
    let c = bank();
    let h = c.search("RAM").unwrap().hits.remove(0);
    let d = Collection::from_bytes(&c.to_bytes(), c.root()).unwrap();
    assert_eq!(c.to_bytes(), d.to_bytes());
    d.validate(&h).unwrap();
    assert_eq!(
        c.get(1).unwrap().hash(),
        digest(c.get(1).unwrap().text().as_bytes())
    );
}
#[test]
fn every_byte_mutation_and_truncation_rejected_with_original_pin() {
    let c = bank();
    let b = c.to_bytes();
    for i in 0..b.len() {
        let mut changed = b.clone();
        changed[i] ^= 1;
        assert!(Collection::from_bytes(&changed, c.root()).is_err());
    }
    for n in 0..b.len() {
        assert!(Collection::from_bytes(&b[..n], c.root()).is_err());
    }
}
#[test]
fn malformed_structure_rejected_even_with_matching_pin() {
    let b = bank().to_bytes();
    let mut fixtures = Vec::new();
    for (at, n) in [(8, 0u64), (16, 0), (24, 1025), (32, 0), (44, u64::MAX)] {
        let mut x = b.clone();
        x[at..at + 8].copy_from_slice(&n.to_le_bytes());
        fixtures.push(x);
    }
    let mut x = b.clone();
    x.push(0);
    fixtures.push(x);
    let mut x = b.clone();
    x[0] = 0;
    fixtures.push(x);
    let mut x = b.clone();
    x[52] = 255;
    fixtures.push(x);
    for x in fixtures {
        assert!(Collection::from_bytes(&x, digest(&x)).is_err());
    }
}

#[test]
fn well_hashed_structure_boundary_matrix() {
    let mut c = Collection::new();
    c.add("A", "abc").unwrap();
    c.add("B", "def").unwrap();
    let bytes = c.to_bytes();
    // Header: magic, revision, next ID, count. Each record: ID, u32 title
    // length, u64 text length, title, text. The first record is 24 bytes.
    assert_eq!(bytes.len(), 80);
    let mut cases: Vec<(&str, Vec<u8>)> = Vec::new();
    for (label, offset, value) in [
        ("next before first ID", 16, 1_u64),
        ("next above revision", 16, 4),
        ("count omits record", 24, 1),
        ("count invents record", 24, 3),
        ("count overflow", 24, u64::MAX),
        ("ID not below next", 32, 3),
        ("ID overflow", 32, u64::MAX),
        ("duplicate ID", 56, 1),
        ("descending ID", 56, 0),
        ("empty text", 44, 0),
        ("text over limit", 44, document::MAX_TEXT as u64 + 1),
        ("text length overflow", 44, u64::MAX),
    ] {
        let mut b = bytes.clone();
        b[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        cases.push((label, b));
    }
    for value in [0_u32, MAX_TITLE as u32 + 1, u32::MAX] {
        let mut b = bytes.clone();
        b[40..44].copy_from_slice(&value.to_le_bytes());
        cases.push(("invalid title length", b));
    }
    for (label, offset, value) in [
        ("blank title", 52, b' '),
        ("control title", 52, 0),
        ("invalid title UTF8", 52, 255),
        ("invalid text UTF8", 53, 255),
    ] {
        let mut b = bytes.clone();
        b[offset] = value;
        cases.push((label, b));
    }
    for (label, b) in cases {
        // Deliberately recompute the pin: rejection must come from structure,
        // not from the easier original-hash mismatch check.
        assert!(Collection::from_bytes(&b, digest(&b)).is_err(), "{label}");
    }
    assert_eq!(c.to_bytes(), bytes);
}

#[test]
fn exhausted_revision_and_id_counters_reject_without_mutation() {
    let original = bank().to_bytes();
    for (revision, next_id, expected_add) in [
        (u64::MAX, 3_u64, "COLLECTION_REVISION_OVERFLOW"),
        (u64::MAX - 1, u64::MAX, "COLLECTION_ID_OVERFLOW"),
    ] {
        let mut b = original.clone();
        b[8..16].copy_from_slice(&revision.to_le_bytes());
        b[16..24].copy_from_slice(&next_id.to_le_bytes());
        let mut c = Collection::from_bytes(&b, digest(&b)).unwrap();
        let hit = c.search("RAM").unwrap().hits.remove(0);
        assert_eq!(c.add("extra", "new text").unwrap_err(), expected_add);
        assert_eq!(c.to_bytes(), b);
        assert_eq!(c.root(), digest(&b));
        c.validate(&hit).unwrap();
        if revision == u64::MAX {
            assert_eq!(
                c.replace(2, "replacement").unwrap_err(),
                "COLLECTION_REVISION_OVERFLOW"
            );
            assert_eq!(c.remove(2).unwrap_err(), "COLLECTION_REVISION_OVERFLOW");
            assert_eq!(c.to_bytes(), b);
            c.validate(&hit).unwrap();
        }
        let restored = Collection::from_bytes(&c.to_bytes(), c.root()).unwrap();
        assert_eq!(restored.to_bytes(), b);
    }
}
#[test]
fn search_caps_do_not_hide_total_counts() {
    let mut c = Collection::new();
    for n in 0..20 {
        c.add(&format!("doc{n}"), "hit\nhit\nhit\nhit\nhit")
            .unwrap();
    }
    let s = c.search("hit").unwrap();
    assert_eq!(s.hits.len(), MAX_RESULTS);
    assert_eq!(s.matching_lines, 100);
    assert_eq!(s.documents_examined, 20);
}
#[test]
fn skipped_lines_are_incomplete_and_empty_bank_validates_query() {
    let mut c = Collection::new();
    assert_eq!(c.search("x").unwrap().status(), "UNKNOWN");
    assert!(c.search("").is_err());
    c.add("long", &"x".repeat(document::MAX_LINE + 1)).unwrap();
    c.add("short", "x").unwrap();
    let s = c.search("x").unwrap();
    assert_eq!(s.status(), "INCOMPLETE");
    assert_eq!(s.skipped_long_lines, 1);
    assert_eq!(s.hits.len(), 1);
}
#[test]
fn document_and_collection_limits_fail_before_mutating() {
    let mut c = Collection::new();
    assert!(c.add("large", &"x".repeat(document::MAX_TEXT + 1)).is_err());
    for n in 0..MAX_DOCUMENTS {
        c.add(&format!("{n}"), "x").unwrap();
    }
    let root = c.root();
    assert!(c.add("extra", "x").is_err());
    assert_eq!(root, c.root());
}
#[test]
fn no_replace_and_successful_load() {
    let s = Scratch::new();
    let p = s.0.join("bank");
    let c = bank();
    let pin = c.save_new(&p).unwrap();
    assert!(c.save_new(&p).is_err());
    let d = Collection::load(&p, pin).unwrap();
    assert_eq!(c.to_bytes(), d.to_bytes());
    assert_eq!(fs::read_dir(&s.0).unwrap().count(), 1);
}
#[test]
fn concurrent_publish_has_one_winner() {
    let s = Scratch::new();
    let p = s.0.join("bank");
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let p = p.clone();
            std::thread::spawn(move || {
                let mut c = bank();
                c.add("thread", &i.to_string()).unwrap();
                c.save_new(&p)
            })
        })
        .collect();
    let pins: Vec<_> = threads
        .into_iter()
        .filter_map(|t| t.join().unwrap().ok())
        .collect();
    assert_eq!(pins.len(), 1);
    Collection::load(&p, pins[0]).unwrap();
    assert_eq!(fs::read_dir(&s.0).unwrap().count(), 1);
}
#[test]
fn arbitrary_short_inputs_never_panic() {
    let mut seed = 91u64;
    for len in 0..512 {
        let b: Vec<_> = (0..len)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as u8
            })
            .collect();
        assert!(Collection::from_bytes(&b, digest(&b)).is_err());
    }
}
