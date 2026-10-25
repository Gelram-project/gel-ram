//! The end of a line in a byte slice, and whether the line is ASCII, eight
//! bytes at a time in one pass.

const ONES: u64 = 0x0101_0101_0101_0101;
const LOW7: u64 = 0x7F7F_7F7F_7F7F_7F7F;
const HIGH: u64 = 0x8080_8080_8080_8080;

/// The high bit of every byte of `v` that is zero, and no other bit. Each
/// byte of `(v & LOW7) + LOW7` is at most 0xFE, so nothing carries into the
/// next byte; its high bit is set when the low seven bits are not all zero.
const fn zero_bytes(v: u64) -> u64 {
    !(((v & LOW7) + LOW7) | v | LOW7) & HIGH
}

/// The position of the first '\r' or '\n' byte of `bytes[start..end]`, as an
/// offset in `bytes`, or `end` when there is none, and whether every byte of
/// the line before it is ASCII. The same value as
/// `let p = bytes[start..end].iter().position(|&b| b == b'\r' || b == b'\n')
/// .map_or(end, |p| start + p); (p, bytes[start..p].is_ascii())`.
pub(crate) fn find_ascii(bytes: &[u8], start: usize, end: usize) -> (usize, bool) {
    let hay = &bytes[start..end];
    let mut words = hay.chunks_exact(8);
    let mut at = start;
    // The OR of every whole word before the line end.
    let mut seen = 0u64;
    for word in &mut words {
        let mut le = [0u8; 8];
        le.copy_from_slice(word);
        let v = u64::from_le_bytes(le);
        let found =
            zero_bytes(v ^ (ONES * u64::from(b'\n'))) | zero_bytes(v ^ (ONES * u64::from(b'\r')));
        if found != 0 {
            // `zero_bytes` marks exactly the matching bytes, so the lowest bit
            // of `found` is the high bit of the first line end. Only the bytes
            // below it (first in memory, little endian) belong to the line.
            let before = (found & found.wrapping_neg()) - 1;
            let ascii = (seen | (v & before)) & HIGH == 0;
            return (at + (found.trailing_zeros() / 8) as usize, ascii);
        }
        seen |= v;
        at += 8;
    }
    let rest = words.remainder();
    let len = rest
        .iter()
        .position(|&b| b == b'\r' || b == b'\n')
        .unwrap_or(rest.len());
    (at + len, seen & HIGH == 0 && rest[..len].is_ascii())
}
