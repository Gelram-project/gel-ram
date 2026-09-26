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
