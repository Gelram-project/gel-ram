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
    // The independently supplied full-file pin already authenticates these bytes,
    // and these pins are their own digests, so comparing them again cannot fail.
    Corpus::load_verified(catalog, text, digest(catalog), digest(text))
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

/// Publish a caller-encoded file of at most `MAX_TEXT` bytes, such as one Q8DEMO01
/// record, with the same no-replace publisher as bundles. Returns the SHA-256 pin
/// of exactly the bytes written. The caller validates the format.
pub fn write_file_new(path: &Path, bytes: &[u8]) -> Result<Hash, BundleError> {
    write_file_new_with(path, bytes, &mut SystemPublication)
}

/// [`write_file_new`] with the four I/O steps of the publication taken from
/// `io`, so a test can fail or stop the process at a known step. Every other
/// step (the exclusive temporary file, the order of the steps and the removal of
/// the temporary file) is the publisher's own.
pub fn write_file_new_with(
    path: &Path,
    bytes: &[u8],
    io: &mut impl PublicationIo,
) -> Result<Hash, BundleError> {
    if bytes.len() > MAX_TEXT {
        return Err(Error::Limit.into());
    }
    publish_with(path, bytes, io)
}

/// Internal shared no-replace publisher. Callers enforce their format limits.
pub(crate) fn write_bytes_new(path: &Path, bytes: &[u8]) -> Result<Hash, BundleError> {
    publish_with(path, bytes, &mut SystemPublication)
}

/// [`write_bytes_new`] for bytes whose SHA-256 the caller already holds:
/// `pin` must equal `digest(bytes)`, and it is returned unchanged.
pub(crate) fn write_bytes_new_pinned(
    path: &Path,
    bytes: &[u8],
    pin: Hash,
) -> Result<Hash, BundleError> {
    publish_pinned_with(path, bytes, Some(pin), &mut SystemPublication)
}

/// The I/O steps of a no-replace publication, in the order the publisher calls
/// them: write the bytes to the exclusive temporary file, flush it, link it to
/// the new path (which fails if the path exists) and flush the parent directory.
/// The default methods do the real I/O; [`write_file_new_with`] takes an
/// implementation that overrides some of them, for fault and crash tests.
pub trait PublicationIo {
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
    publish_pinned_with(path, bytes, None, io)
}

/// `known_pin`, when given, is `digest(bytes)` already computed by the caller.
fn publish_pinned_with(
    path: &Path,
    bytes: &[u8],
    known_pin: Option<Hash>,
    io: &mut impl PublicationIo,
) -> Result<Hash, BundleError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if path.file_name().is_none() {
        return Err(Error::Format.into());
    }
    let pin = known_pin.unwrap_or_else(|| digest(bytes));
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

    fn envelope(catalog: &[u8], text: &[u8]) -> Vec<u8> {
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(catalog.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(text.len() as u64).to_le_bytes());
        bytes.extend_from_slice(catalog);
        bytes.extend_from_slice(text);
        bytes
    }
    fn same_corpus(a: &Corpus, b: &Corpus) {
        assert_eq!(a.root, b.root);
        assert_eq!(a.text_hash, b.text_hash);
        assert_eq!(a.text, b.text);
        assert_eq!(a.records, b.records);
        assert_eq!(a.titles, b.titles);
    }
    /// The reference: the public reader, which hashes catalog and text itself.
    fn by_reference(catalog: &[u8], text: &[u8]) -> Result<Corpus, Error> {
        Corpus::load(catalog, text, digest(catalog), digest(text))
    }
    fn compare(catalog: &[u8], text: &[u8]) {
        let bytes = envelope(catalog, text);
        match (decode(&bytes, digest(&bytes)), by_reference(catalog, text)) {
            (Ok(a), Ok(b)) => same_corpus(&a, &b),
            (Err(a), Err(b)) => assert_eq!(a, b),
            (a, b) => panic!("decode {:?} but reference {:?}", a.err(), b.err()),
        }
    }
    fn overlapping(parts: usize, len: usize) -> (Vec<u8>, Vec<u8>) {
        let text = vec![b'x'; len];
        let hash = crate::hex(&digest(&text));
        let mut cat = String::from("GELCORPUS082\n");
        for part in 1..=parts {
            cat += &format!(
                "0\t{part}\t0\t{}\t{}\t{hash}\t{}\t{}\n",
                0,
                text.len(),
                crate::hex(b"Demo"),
                crate::hex(format!("Lead ({part})").as_bytes())
            );
        }
        (cat.into_bytes(), text)
    }
    struct Mix(u64);
    impl Mix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }
    fn random_corpus(m: &mut Mix) -> EncodedCorpus {
        const PIECES: [&str; 6] = ["a", "ł", "🦀", "\r\n", " ", "Zażółć"];
        let mut b = CorpusBuilder::new();
        let nodes = 1 + m.below(4);
        for node in 0..nodes {
            for role in 0..1 + m.below(5) {
                let len = 1 + m.below(60);
                let quote: String = (0..len)
                    .map(|_| PIECES[m.below(PIECES.len() as u64) as usize])
                    .collect();
                let title = ["Notes", "notes ", "Łódź", "Other"][m.below(4) as usize];
                let _ = b.push(
                    crate::Address { node, role },
                    node,
                    title,
                    &format!("Part {role}"),
                    &quote,
                );
            }
        }
        b.finish().unwrap()
    }

    /// decode skips the second digest of catalog and text; its corpus and its
    /// errors equal those of the public reader on valid and pin-matched
    /// malformed envelopes.
    #[test]
    fn decode_matches_the_public_reader_on_seeded_and_changed_envelopes() {
        for seed in 0..64u64 {
            let mut m = Mix(seed);
            let e = random_corpus(&mut m);
            let (catalog, text) = (e.catalog_bytes().to_vec(), e.text_bytes().to_vec());
            compare(&catalog, &text);
            let reference = by_reference(&catalog, &text).unwrap();
            same_corpus(&e.load().unwrap(), &reference);
            let bytes = encode(&e);
            assert_eq!(bytes, envelope(&catalog, &text));
            same_corpus(&decode(&bytes, digest(&bytes)).unwrap(), &reference);
            for n in 0..catalog.len() {
                let mut c = catalog.clone();
                c[n] ^= [1u8, 0x20, 0x80][n % 3];
                compare(&c, &text);
            }
            for n in 0..text.len() {
                let mut t = text.clone();
                t[n] ^= [1u8, 0x40, 0xff][n % 3];
                compare(&catalog, &t);
            }
            compare(&catalog[..catalog.len() - 1], &text);
            compare(&catalog, &text[..text.len() - 1]);
            compare(&catalog, &[]);
            compare(&[], &text);
        }
    }

    #[test]
    fn decode_matches_the_public_reader_on_fixtures_and_overlaps() {
        compare(
            include_bytes!("../fixtures/rust-book/catalog.txt"),
            include_bytes!("../fixtures/rust-book/ownership.txt"),
        );
        let (catalog, text) = overlapping(257, MAX_PASSAGE);
        compare(&catalog, &text);
        let (catalog, text) = overlapping(crate::MAX_RECORDS, 1);
        compare(&catalog, &text);
        let (catalog, text) = overlapping(crate::MAX_RECORDS + 1, 1);
        compare(&catalog, &text);
        let e = sample();
        same_corpus(
            &e.load().unwrap(),
            &by_reference(e.catalog_bytes(), e.text_bytes()).unwrap(),
        );
    }

    #[test]
    fn decode_matches_the_public_reader_at_the_size_limits() {
        let (catalog, text) = overlapping(1, 1);
        let mut at_limit = text.clone();
        at_limit.resize(MAX_TEXT, b'y');
        compare(&catalog, &at_limit);
        at_limit.push(b'y');
        compare(&catalog, &at_limit);
        let mut catalog_over = catalog.clone();
        catalog_over.resize(MAX_CATALOG + 1, b'\n');
        compare(&catalog_over, &text);
    }
}
