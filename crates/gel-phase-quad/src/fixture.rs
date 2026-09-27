//! Q8DEMO01, the public numeric fixture file: an 8-byte magic, a little-endian
//! u32 record count (1..=8192), then exactly that many 1152-byte records. Each
//! record is 1024 phase bytes followed by a 128-byte active mask (bit j%8 of
//! byte j/8). Every phase value and every mask bit is a valid record, so a
//! changed value is a different record, not an error. The file carries no
//! checksum: integrity comes from a pin the caller retains independently.
use crate::{grid::DIM, Record};

pub const MAGIC: &[u8; 8] = b"Q8DEMO01";
pub const RECORD_BYTES: usize = 1152;
pub const MAX_RECORDS: usize = 8192;
pub const MAX_BYTES: usize = 12 + MAX_RECORDS * RECORD_BYTES;

pub fn decode(raw: &[u8]) -> Result<Vec<Record>, String> {
    if raw.len() < 12 || &raw[..8] != MAGIC {
        return Err("invalid Q8DEMO01 header".into());
    }
    let count = u32::from_le_bytes(raw[8..12].try_into().unwrap()) as usize;
    if !(1..=MAX_RECORDS).contains(&count) || raw.len() != 12 + count * RECORD_BYTES {
        return Err("invalid record count or exact file length".into());
    }
    Ok(raw[12..]
        .chunks_exact(RECORD_BYTES)
        .map(|r| {
            let phase = r[..DIM].try_into().unwrap();
            let active = std::array::from_fn(|j| r[DIM + j / 8] & (1 << (j % 8)) != 0);
            Record::new(phase, &active)
        })
        .collect())
}

pub fn encode(records: &[Record]) -> Result<Vec<u8>, String> {
    if !(1..=MAX_RECORDS).contains(&records.len()) {
        return Err("records must be 1..8192".into());
    }
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for record in records {
        out.extend_from_slice(record.phase());
        let mask = record.active_mask();
        out.extend((0..DIM / 8).map(|byte| {
            (0..8).fold(0u8, |bits, bit| {
                bits | (u8::from(mask[byte * 8 + bit]) << bit)
            })
        }));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_is_the_exact_inverse_of_decode() {
        let mut phase = [0u8; DIM];
        phase[5] = 200;
        let mut active = [true; DIM];
        active[9] = false;
        let raw = encode(&[
            Record::new(phase, &active),
            Record::new([7; DIM], &[false; DIM]),
        ])
        .unwrap();
        assert_eq!(raw.len(), 12 + 2 * RECORD_BYTES);
        let back = decode(&raw).unwrap();
        assert_eq!(back[0].phase()[5], 200);
        assert_eq!(back[0].active(9), Some(false));
        assert_eq!(encode(&back).unwrap(), raw);
    }
}
