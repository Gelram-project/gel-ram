use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};
pub const LIMIT: usize = 8 * 1024 * 1024;
#[derive(Default)]
pub struct Capture {
    pub bytes: Vec<u8>,
    pub done: bool,
    pub error: Option<String>,
}
impl Capture {
    pub fn marker(&self, from: usize, needle: &[u8]) -> io::Result<bool> {
        if let Some(e) = &self.error {
            return Err(io::Error::other(e.clone()));
        }
        if needle.is_empty() {
            return Err(io::Error::other("empty marker"));
        }
        Ok(self
            .bytes
            .get(from..)
            .unwrap_or_default()
            .windows(needle.len())
            .any(|w| w == needle))
    }
}
pub fn new_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}
pub fn pump(
    mut input: impl Read,
    mut log: File,
    mut display: impl Write,
    shared: Arc<Mutex<Capture>>,
) {
    let result = (|| -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let n = input.read(&mut buf)?;
            if n == 0 {
                break;
            }
            {
                let mut state = shared
                    .lock()
                    .map_err(|_| io::Error::other("capture poisoned"))?;
                if state.bytes.len().saturating_add(n) > LIMIT {
                    return Err(io::Error::other("capture limit"));
                }
                log.write_all(&buf[..n])?;
                state.bytes.extend_from_slice(&buf[..n]);
            }
            display.write_all(&buf[..n])?;
            display.flush()?;
        }
        log.sync_all()
    })();
    if let Ok(mut state) = shared.lock() {
        state.done = true;
        state.error = result.err().map(|e| e.to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_utf8_byte_split_is_safe() {
        let raw = "Zażółć 🦀 gel> ".as_bytes();
        for split in 0..=raw.len() {
            let mut c = Capture {
                bytes: raw[..split].to_vec(),
                ..Default::default()
            };
            let _ = c.marker(0, b"gel> ").unwrap();
            c.bytes.extend_from_slice(&raw[split..]);
            assert!(c.marker(0, b"gel> ").unwrap());
        }
    }
    #[test]
    fn refuses_existing_output_without_overwrite() {
        let dir = std::env::temp_dir().join(format!(
            "gel-recorder-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let p = dir.join("log.txt");
        new_file(&p).unwrap().write_all(b"old").unwrap();
        assert!(new_file(&p).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"old");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn capture_error_is_not_a_successful_marker() {
        let c = Capture {
            bytes: b"gel> ".to_vec(),
            error: Some("read failed".into()),
            done: true,
        };
        assert!(c.marker(0, b"gel> ").is_err());
    }
    #[test]
    fn pump_bounds_memory_and_preserves_partial_raw_log() {
        let p = std::env::temp_dir().join(format!(
            "gel-cap-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let shared = Arc::new(Mutex::new(Capture::default()));
        pump(
            io::repeat(65).take((LIMIT + 1) as u64),
            new_file(&p).unwrap(),
            io::sink(),
            shared.clone(),
        );
        let s = shared.lock().unwrap();
        assert!(s.done && s.error.is_some());
        assert_eq!(s.bytes.len(), LIMIT);
        assert_eq!(std::fs::metadata(&p).unwrap().len(), LIMIT as u64);
        std::fs::remove_file(p).unwrap();
    }
}
