//! One caller-owned file placed byte for byte into one public Q8 record.
//!
//! Literal bytes, not GEL knowledge printing: the step that prints text into GEL
//! records is private and is not part of this module. A byte is already one of
//! the 256 levels a record value can take, so nothing is rounded; the first
//! 1,024 bytes fit. One file makes one record. No score is computed here, and
//! every view is shown only where it holds the bytes of the identity preview.
use crate::safe;
use gel_phase_quad::{fixture, grid::DIM, Reader, Record};
use gel_source::{digest, hex, read_regular, write_file_new, Hash, MAX_TEXT};
use std::path::Path;

/// The reader seed of the synthetic panel, so both panels share one view profile.
pub const SEED: u64 = 510051;
/// A saved literal record: the 12-byte Q8DEMO01 header and one 1,152-byte record.
pub const SAVED_BYTES: usize = 12 + fixture::RECORD_BYTES;
const PREVIEW: usize = 56;
const HEX: usize = 16;

/// One record holding the first bytes of one file: active exactly on positions
/// below `placed`, zero beyond. Built only from bytes or from an accepted file.
pub struct Literal {
    record: Record,
    placed: usize,
    total: Option<usize>,
}
impl Literal {
    pub fn record(&self) -> &Record {
        &self.record
    }
    /// Bytes placed in the record: the first `min(file length, 1024)`.
    pub fn placed(&self) -> usize {
        self.placed
    }
    /// The file length; unknown for a reopened record, which does not store it.
    pub fn total(&self) -> Option<usize> {
        self.total
    }
}

/// Value j is byte j and is active for j below the placed length; the rest is
/// inactive and zero. An empty input is refused.
pub fn from_bytes(bytes: &[u8]) -> Result<Literal, String> {
    if bytes.is_empty() {
        return Err("empty file: no bytes to place".into());
    }
    let placed = bytes.len().min(DIM);
    let record = Record::new(
        std::array::from_fn(|j| if j < placed { bytes[j] } else { 0 }),
        &std::array::from_fn(|j| j < placed),
    );
    Ok(Literal {
        record,
        placed,
        total: Some(bytes.len()),
    })
}

/// Read one regular file (no symlink, at most `MAX_TEXT` bytes) and place it.
pub fn read(path: &Path) -> Result<Literal, String> {
    let bytes = read_regular(path, MAX_TEXT).map_err(|e| format!("Read refused: {e:?}"))?;
    from_bytes(&bytes)
}

/// The four public views of one record and whether each restores it exactly.
pub struct Views {
    pub values: [[u8; DIM]; 4],
    pub restored: [bool; 4],
    /// Value and activity bits that differ after restoring, over all four views.
    pub different_bits: u32,
}

/// Value bits plus activity bits in which two records differ.
pub fn differing_bits(a: &Record, b: &Record) -> u32 {
    let value_bits: u32 = a
        .phase()
        .iter()
        .zip(b.phase())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum();
    let mask_bits = a
        .active_mask()
        .iter()
        .zip(b.active_mask())
        .filter(|(x, y)| *x != y)
        .count() as u32;
    value_bits + mask_bits
}

pub fn views(record: &Record) -> Result<Views, String> {
    let reader = Reader::new(SEED);
    let mut values = [[0; DIM]; 4];
    let mut bits = [0; 4];
    for pole in 0..4u8 {
        let view = reader.bound_view(record, pole)?;
        values[usize::from(pole)] = view.parts().1.phase;
        bits[usize::from(pole)] = differing_bits(&reader.restore_bound_view(&view)?, record);
    }
    Ok(Views {
        values,
        restored: bits.map(|b| b == 0),
        different_bits: bits.iter().sum(),
    })
}

/// The Q8DEMO01 bytes of one record.
pub fn encode(record: &Record) -> Result<Vec<u8>, String> {
    fixture::encode(std::slice::from_ref(record))
}

/// Accept the bytes only under the retained pin, as exactly one record of the
/// literal shape: active on the first n positions for some n >= 1, zero beyond.
/// The decoder accepts every body byte, so the pin is what detects a changed
/// value; the shape check refuses any record `from_bytes` cannot make.
pub fn check(bytes: &[u8], pin: &Hash) -> Result<Literal, String> {
    if &digest(bytes) != pin {
        return Err("REJECTED: the bytes differ from the retained pin".into());
    }
    let records = fixture::decode(bytes)?;
    let [record] = records.as_slice() else {
        return Err("a literal record file holds exactly one record".into());
    };
    let placed = record.active_mask().iter().take_while(|a| **a).count();
    let rebuilt = from_bytes(&record.phase()[..placed])
        .map_err(|_| "REJECTED: not a literal record".to_string())?;
    if encode(&rebuilt.record)? != bytes {
        return Err("REJECTED: not a literal record".into());
    }
    Ok(Literal {
        total: None,
        ..rebuilt
    })
}

/// Publish without replacing an existing path; returns the pin to retain.
pub fn save(path: &Path, record: &Record) -> Result<Hash, String> {
    write_file_new(path, &encode(record)?)
        .map_err(|e| format!("Save failed; the path may exist, inspect it before retrying: {e:?}"))
}

/// Reopen a saved record against a pin retained independently of the file.
pub fn reopen(path: &Path, pin: &Hash) -> Result<Literal, String> {
    let bytes = read_regular(path, SAVED_BYTES).map_err(|e| format!("Reopen refused: {e:?}"))?;
    check(&bytes, pin)
}

/// A copy with the lowest bit of one byte flipped; the position wraps around.
fn flip(bytes: &[u8], position: usize) -> Vec<u8> {
    let mut copy = bytes.to_vec();
    copy[position % bytes.len()] ^= 1;
    copy
}

/// The unchanged bytes pass under the pin, and a one-byte change of a RAM copy
/// at every given position is rejected.
pub fn tamper_rejected(bytes: &[u8], pin: &Hash, positions: &[usize]) -> bool {
    !positions.is_empty()
        && check(bytes, pin).is_ok()
        && positions
            .iter()
            .all(|&i| check(&flip(bytes, i), pin).is_err())
}

/// Up to `PREVIEW` bytes from the start as escaped text, cut back to a whole
/// character when the limit splits one; ` …` marks bytes left out. Bytes that
/// are not UTF-8 text are shown in hex instead.
fn text(bytes: &[u8]) -> String {
    let shown = &bytes[..bytes.len().min(PREVIEW)];
    let whole = match std::str::from_utf8(shown) {
        Ok(t) => Some(t),
        // Only an incomplete character at the very end is cut away.
        Err(e) if e.error_len().is_none() => std::str::from_utf8(&shown[..e.valid_up_to()]).ok(),
        Err(_) => None,
    }
    .filter(|t| !t.is_empty());
    match whole {
        Some(t) => {
            let more = if t.len() < bytes.len() { " …" } else { "" };
            format!("\"{}\"{more}", safe(t))
        }
        None => hex16(bytes),
    }
}

fn hex16(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(HEX)
        .map(|v| format!("{v:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The mirror view at the mirrored positions of the identity preview: as text
/// only when those bytes are plain ASCII, so it reads as that preview reversed,
/// in hex otherwise.
fn mirror(values: &[u8; DIM], placed: usize) -> String {
    let k = placed.min(PREVIEW);
    let reversed = &values[DIM - k..];
    if reversed.is_ascii() {
        let s = std::str::from_utf8(reversed).unwrap_or_default();
        format!("{}\"{}\"", if placed > k { "… " } else { "" }, safe(s))
    } else {
        hex16(&values[DIM - placed.min(HEX)..])
    }
}

/// The terminal report. `line` names what happened to the record file.
pub fn report(lit: &Literal, v: &Views, pin: &Hash, rejected: bool, line: &str) -> String {
    let n = lit.placed;
    let rule = "  ------------------------------------------------------------------------\n";
    let mut out = String::new();
    out += "  GEL LIVE LAB  /  LITERAL RECORD  /  OFFLINE  /  NO LLM\n";
    out += "  ========================================================================\n";
    out += "  Literal bytes, not GEL knowledge printing: the step that prints text\n";
    out += "  into GEL records is private and is not part of this command.\n";
    out += &match lit.total {
        Some(total) => format!(
            "  SOURCE | caller-selected file, path not displayed | {total} B, first {n} B placed\n"
        ),
        None => format!(
            "  SOURCE | saved Q8DEMO01 record, path not displayed | file length not stored | {n} B placed\n"
        ),
    };
    out += &format!(
        "  RECORD | 1024 values of 256 levels + 128-byte activity mask = {} B | active {n}/1024\n",
        fixture::RECORD_BYTES
    );
    out += rule;
    out += &format!("  P0 identity        | {}\n", text(&v.values[0][..n]));
    out += &format!(
        "  P1 offset          | {}  (offset view, not text)\n",
        hex16(&v.values[1][..n])
    );
    out += &format!(
        "  P2 mirror          | {}  (the bytes above, reversed, at the end)\n",
        mirror(&v.values[2], n)
    );
    out += &format!(
        "  P3 mirror + offset | {}  (offset view, not text)\n",
        hex16(&v.values[3][DIM - n.min(HEX)..])
    );
    out += &format!(
        "  ROUNDTRIP={}/4 DIFFERENT_BITS={} | each view restored to the original record\n",
        v.restored.iter().filter(|r| **r).count(),
        v.different_bits
    );
    out += rule;
    out += &format!("  Q8DEMO01 | {SAVED_BYTES} B | pin {}\n", hex(pin));
    out += &format!("  {line}\n");
    out += &format!(
        "  TAMPER={} | a 1-byte change in a RAM copy is checked against the pin\n",
        if rejected { "REJECTED" } else { "FAIL" }
    );
    out += "  4 views of 1 record, not 4 copies\n";
    out
}

/// True when every view restored the record and the changed copy was rejected.
pub fn passed(v: &Views, rejected: bool) -> bool {
    v.restored.iter().all(|r| *r) && v.different_bits == 0 && rejected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_value() -> Vec<u8> {
        (0..=255u8).cycle().take(DIM).collect()
    }

    fn saved(record: Record) -> Vec<u8> {
        encode(&record).unwrap()
    }

    #[test]
    fn values_are_the_bytes_and_the_mirror_holds_them_reversed() {
        let bytes = "Zażółć gęślą jaźń\n".repeat(80).into_bytes();
        let lit = from_bytes(&bytes).unwrap();
        assert_eq!((lit.placed(), lit.total()), (DIM, Some(bytes.len())));
        let v = views(lit.record()).unwrap();
        assert_eq!(&v.values[0][..], &bytes[..DIM]);
        for (j, byte) in bytes.iter().enumerate().take(DIM) {
            assert_eq!(v.values[2][DIM - 1 - j], *byte, "{j}");
        }
        assert_ne!(v.values[1], v.values[0]);
        assert_ne!(v.values[3], v.values[2]);
    }

    #[test]
    fn short_input_is_active_only_where_placed() {
        let lit = from_bytes(b"GEL").unwrap();
        assert_eq!((lit.placed(), lit.total()), (3, Some(3)));
        assert_eq!(&lit.record().phase()[..4], &[b'G', b'E', b'L', 0]);
        let mask = lit.record().active_mask();
        assert_eq!(mask.iter().filter(|a| **a).count(), 3);
        assert!(mask[..3].iter().all(|a| *a));
        let v = views(lit.record()).unwrap();
        assert_eq!(&v.values[2][DIM - 3..], b"LEG");
    }

    #[test]
    fn every_byte_value_restores_through_every_view() {
        for bytes in [every_value(), vec![0xff], vec![0; 2000], b"x".repeat(1023)] {
            let v = views(from_bytes(&bytes).unwrap().record()).unwrap();
            assert_eq!(v.restored, [true; 4]);
            assert_eq!(v.different_bits, 0);
        }
    }

    #[test]
    fn differing_bits_counts_value_and_activity_bits() {
        let a = from_bytes(b"GEL").unwrap();
        assert_eq!(differing_bits(a.record(), a.record()), 0);
        // 'G' 0x47 -> 'D' 0x44 differs in 2 bits; one more placed byte adds 1 mask bit
        // and the 7 set bits of '\x7f'.
        let b = from_bytes(b"DEL\x7f").unwrap();
        assert_eq!(differing_bits(a.record(), b.record()), 2 + 1 + 7);
        assert_eq!(differing_bits(b.record(), a.record()), 10);
    }

    #[test]
    fn passed_needs_every_view_zero_bits_and_a_rejected_change() {
        let mut v = views(from_bytes(b"GEL").unwrap().record()).unwrap();
        assert!(passed(&v, true));
        assert!(!passed(&v, false));
        v.restored[3] = false;
        assert!(!passed(&v, true));
        v.restored[3] = true;
        v.different_bits = 1;
        assert!(!passed(&v, true));
    }

    #[test]
    fn previews_are_bounded_marked_and_cut_at_a_whole_character() {
        assert_eq!(text(b"GEL"), "\"GEL\"");
        let a = "a".repeat(PREVIEW);
        assert_eq!(text(a.as_bytes()), format!("\"{a}\""));
        assert_eq!(text(format!("{a}b").as_bytes()), format!("\"{a}\" …"));
        // 'ż' is 2 bytes; its first byte would be the 56th.
        let split = format!("{}ż", "a".repeat(PREVIEW - 1));
        assert_eq!(
            text(split.as_bytes()),
            format!("\"{}\" …", "a".repeat(PREVIEW - 1))
        );
        // A 4-byte character with 3 bytes inside the preview is cut back whole.
        let emoji = format!("{}🦀", "a".repeat(PREVIEW - 3));
        assert_eq!(
            text(emoji.as_bytes()),
            format!("\"{}\" …", "a".repeat(PREVIEW - 3))
        );
        assert_eq!(text(&[0x61, 0xff, 0x62]), "61 ff 62");
        assert_eq!(hex16(&[0x00, 0xab, 0xff]), "00 ab ff");
        assert_eq!(hex16(&[7; 40]).split(' ').count(), HEX);
    }

    #[test]
    fn only_bytes_from_the_start_of_the_file_appear() {
        let long: Vec<u8> = (0..DIM).map(|j| b'a' + (j % 26) as u8).collect();
        let v = views(from_bytes(&long).unwrap().record()).unwrap();
        let mut first: Vec<u8> = long[..PREVIEW].to_vec();
        first.reverse();
        assert_eq!(
            mirror(&v.values[2], DIM),
            format!("… \"{}\"", String::from_utf8(first).unwrap())
        );
        let mut file = vec![b'a'; 512];
        file.extend([b'~'; 512]);
        let lit = from_bytes(&file).unwrap();
        let shown = report(&lit, &views(lit.record()).unwrap(), &[0; 32], true, "-");
        assert!(!shown.contains('~'), "{shown}");
    }

    #[test]
    fn a_preview_that_is_not_text_forward_is_not_text_reversed() {
        let mut file = b"plain ".to_vec();
        file.extend("żółw".bytes().rev());
        let lit = from_bytes(&file).unwrap();
        let v = views(lit.record()).unwrap();
        let mut first16 = file[..file.len().min(HEX)].to_vec();
        first16.reverse();
        assert_eq!(text(&v.values[0][..file.len()]), hex16(&file));
        assert_eq!(mirror(&v.values[2], file.len()), hex16(&first16));
        let pl = "żółw ".repeat(30).into_bytes();
        let v = views(from_bytes(&pl).unwrap().record()).unwrap();
        let mut first16 = pl[..HEX].to_vec();
        first16.reverse();
        assert_eq!(mirror(&v.values[2], pl.len()), hex16(&first16));
        let shown = report(&lit, &views(lit.record()).unwrap(), &[0; 32], true, "-");
        assert!(!shown.contains('\u{fffd}'), "{shown}");
    }

    #[test]
    fn a_flip_changes_exactly_the_named_byte() {
        let raw = saved(from_bytes(b"GEL").unwrap().record);
        let changed = flip(&raw, 612);
        assert_eq!(changed[612], raw[612] ^ 1);
        assert_eq!(changed.iter().zip(&raw).filter(|(a, b)| a != b).count(), 1);
        assert_eq!(flip(&raw, raw.len() + 612), changed);
    }

    #[test]
    fn tamper_check_needs_the_unchanged_bytes_to_pass() {
        let raw = saved(from_bytes(b"GEL").unwrap().record);
        let pin = digest(&raw);
        assert!(tamper_rejected(&raw, &pin, &[0, 12, 612, raw.len() - 1]));
        assert!(!tamper_rejected(&raw, &pin, &[]));
        assert!(!tamper_rejected(&raw, &digest(&flip(&raw, 612)), &[612]));
    }

    #[test]
    fn empty_input_is_refused() {
        assert!(from_bytes(b"").is_err());
    }

    #[test]
    fn saved_bytes_have_the_fixture_layout_and_a_fixed_pin() {
        let raw = saved(from_bytes(b"GEL").unwrap().record);
        assert_eq!(raw.len(), SAVED_BYTES);
        assert_eq!(&raw[..8], b"Q8DEMO01");
        assert_eq!(&raw[8..12], &1u32.to_le_bytes());
        assert_eq!(&raw[12..15], b"GEL");
        assert_eq!(raw[12 + DIM], 0b111);
        // Computed outside Rust from the documented layout: header, "GEL", 1,021
        // zero values, mask byte 0x07 and 127 zero mask bytes.
        assert_eq!(
            hex(&digest(&raw)),
            "e389c7f09511b6521e3f2bd36dfd4ea490d128c026b98b01e32e821f54714e5f"
        );
    }

    #[test]
    fn a_changed_byte_anywhere_is_rejected() {
        let lit = from_bytes(&every_value()).unwrap();
        let raw = encode(lit.record()).unwrap();
        let pin = digest(&raw);
        let back = check(&raw, &pin).unwrap();
        assert_eq!(back.record().phase(), lit.record().phase());
        assert_eq!(back.record().active_mask(), lit.record().active_mask());
        assert_eq!((back.placed(), back.total()), (DIM, None));
        let all: Vec<usize> = (0..raw.len()).collect();
        assert!(tamper_rejected(&raw, &pin, &all));
    }

    #[test]
    fn only_the_literal_shape_is_accepted_even_under_its_own_pin() {
        let one = from_bytes(b"one").unwrap().record;
        let mut gap = [true; DIM];
        gap[1] = false;
        let mut nonzero_beyond = [0u8; DIM];
        nonzero_beyond[..3].copy_from_slice(b"one");
        nonzero_beyond[900] = 1;
        let mut first3 = [false; DIM];
        first3[..3].fill(true);
        let mut split_mask = first3;
        split_mask[1000..].fill(true);
        for raw in [
            fixture::encode(&[one.clone(), one]).unwrap(),
            saved(Record::new([b'x'; DIM], &gap)),
            saved(Record::new([b'x'; DIM], &[false; DIM])),
            saved(Record::new(nonzero_beyond, &first3)),
            saved(Record::new(nonzero_beyond, &split_mask)),
        ] {
            assert!(check(&raw, &digest(&raw)).is_err());
        }
    }

    #[test]
    fn report_shows_markers_and_no_score_or_time() {
        let lit = from_bytes(b"GEL RAM").unwrap();
        let v = views(lit.record()).unwrap();
        let raw = encode(lit.record()).unwrap();
        let pin = digest(&raw);
        let rejected = tamper_rejected(&raw, &pin, &[0, 12, raw.len() - 1]);
        let shown = report(&lit, &v, &pin, rejected, "not written");
        for want in [
            "ROUNDTRIP=4/4 DIFFERENT_BITS=0",
            "TAMPER=REJECTED",
            "\"GEL RAM\"",
            "\"MAR LEG\"",
            "7 B, first 7 B placed",
        ] {
            assert!(shown.contains(want), "{want}\n{shown}");
        }
        assert!(
            !shown.contains("score") && !shown.contains(" ns"),
            "{shown}"
        );
        assert!(passed(&v, rejected));
        let back = check(&raw, &pin).unwrap();
        assert!(report(&back, &v, &pin, rejected, "-").contains("file length not stored"));
    }
}
