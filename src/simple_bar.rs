use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum SimpleBarUpdateError {
    #[error("Could not execute osascript to update simple-bar: {0}")]
    IOError(#[from] std::io::Error),

    #[error("osascript returned a non-zero status code\nStdout: {stdout}")]
    NonZeroStatusCode { stdout: String },
}

pub fn update() -> Result<(), SimpleBarUpdateError> {
    let output = Command::new("osascript").args([
        "-e",
        "tell application id \"tracesOf.Uebersicht\" to refresh widget id \"simple-bar-index-jsx\"",
    ]).output()?;

    if !output.status.success() {
        return Err(SimpleBarUpdateError::NonZeroStatusCode {
            stdout: String::from_utf8(output.stdout).expect("stdout was not valid utf-8"),
        });
    }

    Ok(())
}
