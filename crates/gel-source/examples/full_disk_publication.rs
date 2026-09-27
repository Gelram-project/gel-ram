//! Physically full filesystem: snapshot publication must fail closed on ENOSPC.
//! Run ONLY on a small dedicated, empty tmpfs (for example 1 MiB). The directory
//! must itself be a tmpfs mount point with an explicit size of at most 64 MiB
//! (read from /proc/self/mountinfo); anything else is refused before writing.
//! The filler is also capped at 64 MiB and removed on every exit path.
//! Linux only. This is a full-device test, not a power-loss test.
#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), String> {
    Err("Linux only: requires a small dedicated tmpfs".into())
}

#[cfg(target_os = "linux")]
fn main() -> Result<(), String> {
    linux::run()
}

#[cfg(target_os = "linux")]
mod linux {
    use gel_source::{
        backup::{self, CreateError, RestoreError, State},
        collection::Collection,
        import_text, load_bundle, write_bundle_new, BundleError, EncodedCorpus,
    };
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
    };

    const CAP: u64 = 64 * 1024 * 1024;
    const ENOSPC: i32 = 28;

    fn names(dir: &Path) -> Result<Vec<String>, String> {
        let mut names = fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        names.sort();
        Ok(names)
    }

    /// Removes the filler file on every exit path, including early errors.
    struct Filler(Option<PathBuf>);
    impl Filler {
        fn remove(mut self) -> Result<(), String> {
            match self.0.take() {
                Some(path) => fs::remove_file(path).map_err(|e| e.to_string()),
                None => Ok(()),
            }
        }
    }
    impl Drop for Filler {
        fn drop(&mut self) {
            if let Some(path) = self.0.take() {
                let _ = fs::remove_file(path);
            }
        }
    }

    fn unescape(field: &str) -> String {
        field
            .replace("\\040", " ")
            .replace("\\011", "\t")
            .replace("\\012", "\n")
            .replace("\\134", "\\")
    }

    fn tmpfs_bytes(size: &str) -> Option<u64> {
        let (digits, scale) = match size.as_bytes().last()? {
            b'k' | b'K' => (&size[..size.len() - 1], 1024),
            b'm' | b'M' => (&size[..size.len() - 1], 1024 * 1024),
            b'g' | b'G' => (&size[..size.len() - 1], 1024 * 1024 * 1024),
            _ => (size, 1),
        };
        digits.parse::<u64>().ok()?.checked_mul(scale)
    }

    /// Linux dev_t split into (major, minor), as printed in mountinfo.
    fn major_minor(dev: u64) -> (u64, u64) {
        let major = ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff);
        let minor = (dev & 0xff) | ((dev >> 12) & !0xff);
        (major, minor)
    }

    /// The directory must be the root of its own tmpfs mount (not a bind of a
    /// subdirectory), the filesystem it resolves to must be that mount, and the
    /// mount must have an explicit, non-zero size <= CAP (size=0 is unlimited).
    fn small_tmpfs(dir: &Path) -> Result<u64, String> {
        use std::os::unix::fs::MetadataExt;
        let canonical = fs::canonicalize(dir).map_err(|e| e.to_string())?;
        let target = canonical.to_str().ok_or("non-UTF-8 directory path")?;
        let info = fs::read_to_string("/proc/self/mountinfo").map_err(|e| e.to_string())?;
        // The last matching entry is the visible mount; earlier ones are shadowed.
        let line = info
            .lines()
            .filter(|l| l.split(' ').nth(4).map(unescape).as_deref() == Some(target))
            .last()
            .ok_or("refusing: the directory is not a mount point")?;
        let fields: Vec<&str> = line.split(' ').collect();
        let (Some(device), Some(root)) = (fields.get(2), fields.get(3)) else {
            return Err("malformed mountinfo line".into());
        };
        if *root != "/" {
            return Err("refusing: the mount is a bind of a subdirectory".into());
        }
        let (major, minor) =
            major_minor(fs::metadata(&canonical).map_err(|e| e.to_string())?.dev());
        if *device != format!("{major}:{minor}") {
            return Err("refusing: the directory does not resolve to that mount".into());
        }
        let (_, tail) = line.split_once(" - ").ok_or("malformed mountinfo line")?;
        let mut rest = tail.split(' ');
        if rest.next() != Some("tmpfs") {
            return Err("refusing: the mount point is not tmpfs".into());
        }
        let options = rest.nth(1).unwrap_or("");
        let bytes = options
            .split(',')
            .find_map(|o| o.strip_prefix("size="))
            .and_then(tmpfs_bytes)
            .ok_or("refusing: tmpfs without an explicit size")?;
        if bytes == 0 {
            return Err("refusing: tmpfs size 0 means no limit".into());
        }
        if bytes > CAP {
            return Err(format!(
                "refusing: tmpfs size {bytes} B exceeds the 64 MiB cap"
            ));
        }
        Ok(bytes)
    }

    /// Output without a panic path: with panic=abort a panic would skip the
    /// filler guard, so a failed write is returned as an error instead.
    fn say(line: &str) -> Result<(), String> {
        writeln!(std::io::stdout(), "{line}").map_err(|e| format!("stdout: {e}"))
    }

    /// Fills the device until the kernel reports ENOSPC. The returned guard
    /// removes the filler on every exit path.
    fn fill(dir: &Path, name: &str) -> Result<(Filler, u64, std::io::Error), String> {
        let filler_path = dir.join(name);
        let mut filler = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filler_path)
            .map_err(|e| e.to_string())?;
        let guard = Filler(Some(filler_path));
        let chunk = vec![0xA5u8; 64 * 1024];
        let mut filled = 0u64;
        loop {
            if filled >= CAP {
                return Err("filesystem larger than the 64 MiB cap; refusing to fill it".into());
            }
            match filler.write_all(&chunk) {
                Ok(()) => filled += chunk.len() as u64,
                Err(e) if e.raw_os_error() == Some(ENOSPC) => return Ok((guard, filled, e)),
                Err(e) => return Err(format!("filler: {e}")),
            }
        }
    }

    fn corpus(label: &str) -> Result<EncodedCorpus, String> {
        let text = format!("{label}: full-disk publication fixture line.\n").repeat(256);
        import_text(text.as_bytes(), label).map_err(|e| format!("{e:?}"))
    }

    pub fn run() -> Result<(), String> {
        let args: Vec<String> = std::env::args().collect();
        let [_, dir] = args.as_slice() else {
            return Err("usage: full_disk_publication EMPTY_SMALL_FILESYSTEM_DIR".into());
        };
        let dir = Path::new(dir);
        let device_bytes = small_tmpfs(dir)?;
        if !names(dir)?.is_empty() {
            return Err("refusing a non-empty directory".into());
        }
        let previous = dir.join("previous.gelsrc");
        let previous_pin =
            write_bundle_new(&previous, &corpus("previous")?).map_err(|e| format!("{e:?}"))?;

        let (guard, filled, full) = fill(dir, "filler.bin")?;
        say(&format!(
            "FULL_DISK_SETUP tmpfs_bytes={device_bytes} filler_bytes_at_least={filled} kernel_error={full} previous_pin_prefix={:02x}{:02x}{:02x}{:02x}",
            previous_pin[0], previous_pin[1], previous_pin[2], previous_pin[3]
        ))?;

        // Publication on the full device must fail closed.
        let target = dir.join("new.gelsrc");
        let before = names(dir)?;
        let refused = match write_bundle_new(&target, &corpus("new")?) {
            Ok(_) => return Err("publication succeeded on a full device".into()),
            Err(BundleError::Io(e)) if e.raw_os_error() == Some(ENOSPC) => e,
            Err(other) => return Err(format!("unexpected error: {other:?}")),
        };
        let after = names(dir)?;
        if after != before || target.exists() {
            return Err(format!("directory changed: {before:?} -> {after:?}"));
        }
        let reopened = load_bundle(&previous, previous_pin).map_err(|e| format!("{e:?}"))?;
        drop(reopened);
        say(&format!(
            "FULL_DISK_PUBLISH result=REFUSED error={refused} destination=ABSENT temporaries=0 previous=RELOADED"
        ))?;

        // After space is freed the same publication succeeds and reloads.
        guard.remove()?;
        let new_pin = write_bundle_new(&target, &corpus("new")?).map_err(|e| format!("{e:?}"))?;
        load_bundle(&target, new_pin).map_err(|e| format!("{e:?}"))?;
        load_bundle(&previous, previous_pin).map_err(|e| format!("{e:?}"))?;
        let left = names(dir)?;
        if left != ["new.gelsrc", "previous.gelsrc"] {
            return Err(format!("unexpected files after recovery: {left:?}"));
        }
        say("FULL_DISK_RECOVERY filler_removed=true publish=PASS reload_new=PASS reload_previous=PASS")?;

        // Backups on the same device: with no space left, creation leaves no
        // committed manifest and restore leaves no target.
        let mut bank = Collection::new();
        bank.add(
            "notes.txt",
            "RAM is volatile.\nA backup is committed by its manifest.\n",
        )?;
        let snapshot = dir.join("bank.gelset");
        let pin = bank.save_new(&snapshot)?;
        let complete = dir.join("backup ok");
        backup::create(&snapshot, pin, &complete).map_err(|e| format!("{e:?}"))?;
        let (guard, _, _) = fill(dir, "filler2.bin")?;
        let unfinished = dir.join("backup full");
        let create_error = match backup::create(&snapshot, pin, &unfinished) {
            Ok(_) => return Err("backup created on a full device".into()),
            Err(CreateError::CreatedUnconfirmed(why)) => {
                return Err(format!(
                    "backup reported as created on a full device: {why}"
                ))
            }
            Err(CreateError::NotCreated(why)) => why,
        };
        if unfinished.exists() && backup::inspect(&unfinished, pin)?.state != State::Incomplete {
            return Err("interrupted backup is not INCOMPLETE".into());
        }
        let target = dir.join("restored.gelset");
        match backup::restore(&complete, pin, &target) {
            Err(RestoreError::NotPublished(why)) if why.contains("No space") => {}
            other => return Err(format!("restore on a full device: {other:?}")),
        }
        if target.exists() {
            return Err("restore left a target on a full device".into());
        }
        say(&format!(
            "FULL_DISK_BACKUP create=REFUSED({}) manifest=ABSENT restore=NOT_PUBLISHED target=ABSENT",
            create_error.split_whitespace().next().unwrap_or("")
        ))?;
        guard.remove()?;
        backup::restore(&complete, pin, &target).map_err(|e| format!("{e:?}"))?;
        Collection::load(&target, pin)?;
        if unfinished.exists() {
            fs::remove_dir_all(&unfinished).map_err(|e| e.to_string())?;
        }
        say("FULL_DISK_BACKUP_RECOVERY restore=PASS reload=PASS unfinished_backup_removed=true")?;
        say("FULL_DISK_PUBLICATION=PASS")?;
        Ok(())
    }
}
