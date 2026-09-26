//! Physically full filesystem: snapshot publication must fail closed on ENOSPC.
//! Run ONLY on a small dedicated, empty filesystem (for example a 1 MiB tmpfs).
//! The filler is capped at 64 MiB; a larger filesystem is refused, not filled.
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
    use gel_source::{import_text, load_bundle, write_bundle_new, BundleError, EncodedCorpus};
    use std::{fs, io::Write, path::Path};

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
        if !names(dir)?.is_empty() {
            return Err("refusing a non-empty directory".into());
        }
        let previous = dir.join("previous.gelsrc");
        let previous_pin =
            write_bundle_new(&previous, &corpus("previous")?).map_err(|e| format!("{e:?}"))?;

        // Fill the device until the kernel reports ENOSPC.
        let filler_path = dir.join("filler.bin");
        let mut filler = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filler_path)
            .map_err(|e| e.to_string())?;
        let chunk = vec![0xA5u8; 64 * 1024];
        let mut filled = 0u64;
        let full = loop {
            if filled >= CAP {
                drop(filler);
                let _ = fs::remove_file(&filler_path);
                return Err("filesystem larger than the 64 MiB cap; refusing to fill it".into());
            }
            match filler.write_all(&chunk) {
                Ok(()) => filled += chunk.len() as u64,
                Err(e) if e.raw_os_error() == Some(ENOSPC) => break e,
                Err(e) => return Err(format!("filler: {e}")),
            }
        };
        drop(filler);
        println!(
            "FULL_DISK_SETUP filler_bytes_at_least={filled} kernel_error={full} previous_pin_prefix={:02x}{:02x}{:02x}{:02x}",
            previous_pin[0], previous_pin[1], previous_pin[2], previous_pin[3]
        );

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
        println!(
            "FULL_DISK_PUBLISH result=REFUSED error={refused} destination=ABSENT temporaries=0 previous=RELOADED"
        );

        // After space is freed the same publication succeeds and reloads.
        fs::remove_file(&filler_path).map_err(|e| e.to_string())?;
        let new_pin = write_bundle_new(&target, &corpus("new")?).map_err(|e| format!("{e:?}"))?;
        load_bundle(&target, new_pin).map_err(|e| format!("{e:?}"))?;
        load_bundle(&previous, previous_pin).map_err(|e| format!("{e:?}"))?;
        let left = names(dir)?;
        if left != ["new.gelsrc", "previous.gelsrc"] {
            return Err(format!("unexpected files after recovery: {left:?}"));
        }
        println!("FULL_DISK_RECOVERY filler_removed=true publish=PASS reload_new=PASS reload_previous=PASS");
        println!("FULL_DISK_PUBLICATION=PASS");
        Ok(())
    }
}
