//! Backup, restore, withdraw and delete for GELSET01 collection snapshots.
//! Arguments are separate words, so paths may contain spaces. stdout carries
//! one result line; stderr carries usage and diagnostics.
#![forbid(unsafe_code)]
use gel_live_lab::{parse_pin, safe};
use gel_source::backup::{self, DeleteError, RestoreError, State};
use std::{io::Write, path::Path, process::ExitCode};

const USAGE: &str = "usage:
  gel-backup create  TRUSTED_SHA256 SNAPSHOT NEW_BACKUP_DIR
  gel-backup inspect TRUSTED_SHA256 BACKUP_DIR
  gel-backup restore TRUSTED_SHA256 BACKUP_DIR NEW_SNAPSHOT_PATH
  gel-backup withdraw BACKUP_DIR REASON
  gel-backup delete  TRUSTED_SHA256 SNAPSHOT
exit: 0 done/restorable, 3 not restorable (incomplete or withdrawn),
      4 written but durability unconfirmed, 2 refused, failed or invalid";

/// One result line and the exit code it implies.
struct Outcome(String, u8);

fn state_line(state: &State) -> String {
    match state {
        State::Complete {
            bytes,
            documents,
            revision,
        } => format!("COMPLETE bytes={bytes} documents={documents} revision={revision}"),
        State::Incomplete => "INCOMPLETE".into(),
        State::Untrusted => "UNTRUSTED".into(),
        State::Corrupt(why) => format!("CORRUPT reason={}", safe(why)),
    }
}

fn run(args: &[String]) -> Result<Outcome, String> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    Ok(match words.as_slice() {
        ["create", pin, snapshot, dir] => {
            let state = backup::create(Path::new(snapshot), parse_pin(pin)?, Path::new(dir))?;
            Outcome(format!("BACKUP=CREATED {}", state_line(&state)), 0)
        }
        ["inspect", pin, dir] => {
            let found = backup::inspect(Path::new(dir), parse_pin(pin)?)?;
            let code = match (&found.state, found.withdrawn) {
                (State::Complete { .. }, false) => 0,
                (State::Complete { .. } | State::Incomplete, _) => 3,
                _ => 2,
            };
            Outcome(
                format!(
                    "BACKUP={} withdrawn={} temporaries={}",
                    state_line(&found.state),
                    found.withdrawn,
                    found.temporaries
                ),
                code,
            )
        }
        ["restore", pin, dir, target] => {
            match backup::restore(Path::new(dir), parse_pin(pin)?, Path::new(target)) {
                Ok(bytes) => Outcome(format!("RESTORE=PASS bytes={bytes}"), 0),
                Err(RestoreError::Refused(why)) => {
                    Outcome(format!("RESTORE=REFUSED reason={}", safe(&why)), 2)
                }
                Err(RestoreError::NotPublished(why)) => {
                    Outcome(format!("RESTORE=NOT_PUBLISHED reason={}", safe(&why)), 2)
                }
                Err(RestoreError::PublishedUnconfirmed(why)) => Outcome(
                    format!("RESTORE=PUBLISHED_UNCONFIRMED reason={}", safe(&why)),
                    4,
                ),
            }
        }
        ["withdraw", dir, reason] => {
            backup::withdraw(Path::new(dir), reason)?;
            Outcome(
                "WITHDRAW=PASS bytes kept; restore of this backup is refused".into(),
                0,
            )
        }
        ["delete", pin, path] => match backup::delete_pinned(Path::new(path), parse_pin(pin)?) {
            Ok(bytes) => Outcome(
                format!("DELETE=PASS bytes={bytes}; other copies, backups and snapshots are unchanged; not a secure erase"),
                0,
            ),
            Err(DeleteError::Refused(why)) => {
                Outcome(format!("DELETE=REFUSED reason={}", safe(&why)), 2)
            }
            Err(DeleteError::RemovedUnconfirmed(why)) => Outcome(
                format!("DELETE=REMOVED_UNCONFIRMED reason={}", safe(&why)),
                4,
            ),
        },
        _ => return Err(USAGE.into()),
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = match std::env::args_os()
        .skip(1)
        .map(|a| a.into_string())
        .collect()
    {
        Ok(args) => args,
        Err(_) => {
            eprintln!("arguments must be UTF-8\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(Outcome(line, code)) => {
            if writeln!(std::io::stdout(), "{line}").is_err() {
                eprintln!("stdout write failed");
                return ExitCode::from(2);
            }
            ExitCode::from(code)
        }
        Err(e) if e == USAGE => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("GEL_BACKUP=FAIL {}", safe(&e));
            ExitCode::from(2)
        }
    }
}
