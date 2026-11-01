// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
//! Fail-fast native process execution, independent of shell exit-code rules.
use std::process::Command;

pub fn sequence(commands: &mut [Command]) -> Result<(), String> {
    for (index, command) in commands.iter_mut().enumerate() {
        let status = command
            .status()
            .map_err(|e| format!("command {index} could not start: {e}"))?;
        if !status.success() {
            return Err(format!("command {index} failed: {status}"));
        }
    }
    Ok(())
}
