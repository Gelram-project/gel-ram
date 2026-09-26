//! Deterministic failures at the actual publisher's I/O boundary.
use super::*;
#[derive(Clone, Copy, Debug)]
enum Fault {
    ShortWrite,
    DiskFull,
    Permission,
    Sync,
    Publish,
    ParentSync,
}
struct Inject(Fault);
impl PublicationIo for Inject {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        match self.0 {
            Fault::ShortWrite => {
                struct Short<'a>(&'a mut File);
                impl Write for Short<'_> {
                    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                        self.0.write(&b[..b.len().min(3)])
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        self.0.flush()
                    }
                }
                Short(file).write_all(bytes)
            }
            Fault::DiskFull => {
                file.write_all(&bytes[..bytes.len() / 2])?;
                Err(std::io::Error::other("injected full disk"))
            }
            Fault::Permission => Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            _ => file.write_all(bytes),
        }
    }
    fn sync(&mut self, file: &File) -> std::io::Result<()> {
        if matches!(self.0, Fault::Sync) {
            Err(std::io::Error::other("injected sync"))
        } else {
            file.sync_all()
        }
    }
    fn publish(&mut self, tmp: &Path, path: &Path) -> std::io::Result<()> {
        if matches!(self.0, Fault::Publish) {
            Err(std::io::Error::other("injected publish"))
        } else {
            fs::hard_link(tmp, path)
        }
    }
    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        if matches!(self.0, Fault::ParentSync) {
            Err(std::io::Error::other("injected parent sync"))
        } else {
            SystemPublication.sync_parent(parent)
        }
    }
}
#[test]
fn all_io_failures_preserve_old_snapshot_and_expose_only_complete_new_file() {
    let root = std::env::temp_dir().join(format!(
        "gel-publish-fault-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let old = import_text(b"old snapshot", "old").unwrap();
    let old_path = root.join("old.gel");
    let old_pin = write_bundle_new(&old_path, &old).unwrap();
    let old_bytes = fs::read(&old_path).unwrap();
    let new = import_text(b"new complete snapshot", "new").unwrap();
    let bytes = encode(&new);
    for fault in [
        Fault::ShortWrite,
        Fault::DiskFull,
        Fault::Permission,
        Fault::Sync,
        Fault::Publish,
        Fault::ParentSync,
    ] {
        let path = root.join(format!("{fault:?}.gel"));
        let result = publish_with(&path, &bytes, &mut Inject(fault));
        assert_eq!(result.is_ok(), matches!(fault, Fault::ShortWrite));
        if matches!(fault, Fault::ShortWrite | Fault::ParentSync) {
            assert_eq!(fs::read(&path).unwrap(), bytes);
            load_bundle(&path, digest(&bytes)).unwrap();
        } else {
            assert!(!path.exists());
        }
        assert_eq!(fs::read(&old_path).unwrap(), old_bytes);
        load_bundle(&old_path, old_pin).unwrap();
        assert!(!fs::read_dir(&root).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }
    assert!(publish_with(&old_path, &bytes, &mut SystemPublication).is_err());
    assert_eq!(fs::read(&old_path).unwrap(), old_bytes);
    fs::remove_dir_all(root).unwrap();
}
