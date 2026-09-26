//! Bounded source import and no-replace publication in an owner-controlled directory.
//! This is plaintext persistence, not an encrypted private memory vault.
use crate::{
    digest, Corpus, CorpusBuilder, EncodedCorpus, Error, Hash, MAX_CATALOG, MAX_PASSAGE, MAX_TEXT,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

const MAGIC: &[u8; 8] = b"GELSRC01";
const HEADER: usize = 24;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum BundleError {
    Io(std::io::Error),
    Data(Error),
    NotRegular,
}
impl From<std::io::Error> for BundleError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<Error> for BundleError {
    fn from(e: Error) -> Self {
        Self::Data(e)
    }
}

/// Does not recurse or follow an existing symlink. Caller must control the parent
/// directory: portable std does not protect against a hostile concurrent path swap.
pub fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>, BundleError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() {
        return Err(BundleError::NotRegular);
    }
    if metadata.len() > limit as u64 {
        return Err(Error::Limit.into());
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(BundleError::NotRegular);
    }
    let mut bytes = Vec::new();
    file.take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::Limit.into());
    }
    Ok(bytes)
}

/// One caller-owned UTF-8 document, segmented only on UTF-8 boundaries.
/// These are byte-bounded parts, NOT semantic paragraphs or learned ORBs.
pub fn import_text(bytes: &[u8], title: &str) -> Result<EncodedCorpus, Error> {
    if bytes.is_empty() || bytes.len() > MAX_TEXT {
        return Err(Error::Limit);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Format)?;
    let mut builder = CorpusBuilder::new();
    let mut start = 0;
    let mut part = 1;
    while start < text.len() {
        let mut end = (start + MAX_PASSAGE).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        builder.push(
            crate::Address {
                node: 1,
                role: part,
            },
            1,
            title,
            &format!("Lead ({part})"),
            &text[start..end],
        )?;
        start = end;
        part += 1;
    }
    builder.finish()
}

fn encode(value: &EncodedCorpus) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(HEADER + value.catalog_bytes().len() + value.text_bytes().len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(value.catalog_bytes().len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(value.text_bytes().len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.catalog_bytes());
    bytes.extend_from_slice(value.text_bytes());
    bytes
}

fn decode(bytes: &[u8], trusted_pin: Hash) -> Result<Corpus, Error> {
    if bytes.len() > HEADER + MAX_CATALOG + MAX_TEXT {
        return Err(Error::Limit);
    }
    if digest(bytes) != trusted_pin {
        return Err(Error::Integrity);
    }
    if bytes.len() < HEADER || &bytes[..8] != MAGIC {
        return Err(Error::Format);
    }
    let catalog_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let text_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    if catalog_len > MAX_CATALOG as u64 || text_len > MAX_TEXT as u64 {
        return Err(Error::Limit);
    }
    let split = HEADER + catalog_len as usize;
    if split + text_len as usize != bytes.len() {
        return Err(Error::Format);
    }
    let catalog = &bytes[HEADER..split];
    let text = &bytes[split..];
    // The independently supplied full-file pin already authenticates these bytes.
    Corpus::load(catalog, text, digest(catalog), digest(text))
}

/// Requires an independently trusted full-file pin, not one read from this file.
pub fn load_bundle(path: &Path, trusted_pin: Hash) -> Result<Corpus, BundleError> {
    Ok(decode(
        &read_regular(path, HEADER + MAX_CATALOG + MAX_TEXT)?,
        trusted_pin,
    )?)
}

/// Publish a complete file without replacing an existing path, using a same-directory
/// hard link. Unsupported filesystems fail closed; there is no overwrite fallback.
/// On Unix syncs the parent directory too. Non-Unix power-loss directory durability
/// is not guaranteed. An I/O error after linking may leave a complete destination:
/// callers must inspect it, never blindly delete/overwrite it. Parent must be trusted.
pub fn write_bundle_new(path: &Path, value: &EncodedCorpus) -> Result<Hash, BundleError> {
    write_bytes_new(path, &encode(value))
}

/// Internal shared no-replace publisher. Callers enforce their format limits.
pub(crate) fn write_bytes_new(path: &Path, bytes: &[u8]) -> Result<Hash, BundleError> {
    publish_with(path, bytes, &mut SystemPublication)
}

trait PublicationIo {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, file: &File) -> std::io::Result<()> {
        file.sync_all()
    }
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        fs::hard_link(tmp, path)
    }
    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = parent;
        Ok(())
    }
}
struct SystemPublication;
impl PublicationIo for SystemPublication {}

#[cfg(test)]
#[path = "publication_fault_tests.rs"]
mod publication_fault_tests;

fn publish_with(
    path: &Path,
    bytes: &[u8],
    io: &mut impl PublicationIo,
) -> Result<Hash, BundleError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if path.file_name().is_none() {
        return Err(Error::Format.into());
    }
    let pin = digest(bytes);
    let mut attempt = 0;
    let (tmp, mut file) = loop {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let tmp = parent.join(format!(".gel-source-{}-{id}.tmp", std::process::id()));
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&tmp) {
            Ok(file) => break (tmp, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && attempt < 32 => {
                attempt += 1;
            }
            Err(e) => return Err(e.into()),
        }
    };
    let result = (|| -> Result<(), std::io::Error> {
        io.write(&mut file, bytes)?;
        io.sync(&file)?;
        drop(file);
        io.publish(&tmp, path)?;
        io.sync_parent(parent)?;
        Ok(())
    })();
    // Only our create_new temporary file is removed. A process crash can leave it;
    // it is never considered a committed bundle and no automatic glob cleanup runs.
    let cleanup = fs::remove_file(&tmp);
    result?;
    cleanup?;
    Ok(pin)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> EncodedCorpus {
        import_text("Zażółć 🦀\r\nCafe\u{301}".as_bytes(), "Notes").unwrap()
    }
    #[test]
    fn envelope_roundtrip() {
        let original = sample();
        let bytes = encode(&original);
        let c = decode(&bytes, digest(&bytes)).unwrap();
        assert_eq!(
            c.quote(crate::Address { node: 1, role: 1 })
                .unwrap()
                .quote()
                .as_bytes(),
            original.text_bytes()
        );
    }
    #[test]
    fn every_truncation_fails_with_original_pin() {
        let bytes = encode(&sample());
        let pin = digest(&bytes);
        for n in 0..bytes.len() {
            assert_eq!(decode(&bytes[..n], pin).unwrap_err(), Error::Integrity);
        }
    }
    #[test]
    fn every_byte_mutation_fails_with_original_pin() {
        let mut bytes = encode(&sample());
        let pin = digest(&bytes);
        for n in 0..bytes.len() {
            bytes[n] ^= 1;
            assert_eq!(decode(&bytes, pin).unwrap_err(), Error::Integrity);
            bytes[n] ^= 1;
        }
    }
    #[test]
    fn malformed_even_with_matching_pin_fails() {
        for n in 0..HEADER {
            let b = vec![0; n];
            assert_eq!(decode(&b, digest(&b)).unwrap_err(), Error::Format);
        }
        let mut b = encode(&sample());
        b[0] ^= 1;
        assert_eq!(decode(&b, digest(&b)).unwrap_err(), Error::Format);
        let mut b = encode(&sample());
        b[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(decode(&b, digest(&b)).unwrap_err(), Error::Limit);
        let mut b = encode(&sample());
        b.push(0);
        assert_eq!(decode(&b, digest(&b)).unwrap_err(), Error::Format);
    }
    #[test]
    fn utf8_boundary_split_preserves_every_byte() {
        let input = format!("{}🦀\r\nzażółć", "x".repeat(MAX_PASSAGE - 1));
        let b = import_text(input.as_bytes(), "Notes").unwrap();
        let c = b.load().unwrap();
        let parts = c.lead_parts("Notes", Some(2)).unwrap();
        assert_eq!(parts.iter().map(|p| p.quote()).collect::<String>(), input);
        assert!(parts.iter().all(|p| p.quote().len() <= MAX_PASSAGE));
    }
    #[test]
    fn empty_and_invalid_utf8_rejected() {
        assert!(matches!(import_text(b"", "Notes"), Err(Error::Limit)));
        assert!(matches!(import_text(&[255], "Notes"), Err(Error::Format)));
    }
}
