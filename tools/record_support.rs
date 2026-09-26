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
            let n = match input.read(&mut buf) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                other => other?,
            };
            if n == 0 {
                break;
            }
            let chunk = buf
                .get(..n)
                .ok_or_else(|| io::Error::other("read length exceeds buffer"))?;
            {
                let mut state = shared
                    .lock()
                    .map_err(|_| io::Error::other("capture poisoned"))?;
                if state.bytes.len().saturating_add(n) > LIMIT {
                    return Err(io::Error::other("capture limit"));
                }
                log.write_all(chunk)?;
                state.bytes.extend_from_slice(chunk);
            }
            display.write_all(chunk)?;
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
    fn temp_log(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "gel-capture-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    struct InterruptOnce<R> {
        inner: R,
        interrupted: bool,
    }
    impl<R: Read> Read for InterruptOnce<R> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            self.inner.read(out)
        }
    }

    #[test]
    fn interrupted_read_retries_and_clean_eof_preserves_unicode() {
        let path = temp_log("interrupt");
        let raw = "Zażółć 🦀 gel> ".as_bytes();
        let shared = Arc::new(Mutex::new(Capture::default()));
        pump(
            InterruptOnce {
                inner: raw,
                interrupted: false,
            },
            new_file(&path).unwrap(),
            io::sink(),
            shared.clone(),
        );
        let state = shared.lock().unwrap();
        assert!(state.done && state.error.is_none());
        assert_eq!(state.bytes, raw);
        assert!(state.marker(0, b"gel> ").unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        std::fs::remove_file(path).unwrap();
    }

    struct FailRead;
    impl Read for FailRead {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("injected read failure"))
        }
    }
    struct FailDisplay {
        flush_only: bool,
    }
    impl Write for FailDisplay {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.flush_only {
                Ok(bytes.len())
            } else {
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected flush failure"))
        }
    }

    #[test]
    fn read_failure_after_marker_is_not_success() {
        let path = temp_log("read-error");
        let raw = b"gel> ";
        let shared = Arc::new(Mutex::new(Capture::default()));
        pump(
            raw.as_slice().chain(FailRead),
            new_file(&path).unwrap(),
            io::sink(),
            shared.clone(),
        );
        let state = shared.lock().unwrap();
        assert!(state.done && state.error.is_some());
        assert!(state.marker(0, raw).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn display_write_and_flush_errors_keep_raw_log_but_invalidate_capture() {
        for flush_only in [false, true] {
            let path = temp_log("display");
            let shared = Arc::new(Mutex::new(Capture::default()));
            pump(
                b"gel> ".as_slice(),
                new_file(&path).unwrap(),
                FailDisplay { flush_only },
                shared.clone(),
            );
            let state = shared.lock().unwrap();
            assert!(state.done && state.error.is_some());
            assert!(state.marker(0, b"gel> ").is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"gel> ");
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn unwritable_log_does_not_publish_marker_to_capture() {
        let path = temp_log("readonly");
        new_file(&path).unwrap().write_all(b"original").unwrap();
        let shared = Arc::new(Mutex::new(Capture::default()));
        pump(
            b"gel> ".as_slice(),
            File::open(&path).unwrap(),
            io::sink(),
            shared.clone(),
        );
        let state = shared.lock().unwrap();
        assert!(state.done && state.error.is_some());
        assert!(state.bytes.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        std::fs::remove_file(path).unwrap();
    }
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
