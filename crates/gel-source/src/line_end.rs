//! The end of a line in a byte slice, eight bytes at a time.

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
/// offset in `bytes`, or `end` when there is none. The same value as
/// `bytes[start..end].iter().position(|&b| b == b'\r' || b == b'\n')
/// .map_or(end, |p| start + p)`.
pub(crate) fn find(bytes: &[u8], start: usize, end: usize) -> usize {
    let hay = &bytes[start..end];
    let mut words = hay.chunks_exact(8);
    let mut at = start;
    for word in &mut words {
        let mut le = [0u8; 8];
        le.copy_from_slice(word);
        let v = u64::from_le_bytes(le);
        let found =
            zero_bytes(v ^ (ONES * u64::from(b'\n'))) | zero_bytes(v ^ (ONES * u64::from(b'\r')));
        if found != 0 {
            // Little endian: the first byte in memory is the lowest one.
            return at + (found.trailing_zeros() / 8) as usize;
        }
        at += 8;
    }
    words
        .remainder()
        .iter()
        .position(|&b| b == b'\r' || b == b'\n')
        .map_or(end, |p| at + p)
}
