use std::{io, process::Command, string::FromUtf8Error};

use log::trace;
use thiserror::Error;

use super::command::YabaiCommand;

#[derive(Error, Debug)]
pub enum YabaiCommandExecError {
    #[error("could not invoke command")]
    Exec(#[from] io::Error),

    #[error("process terminated by a signal")]
    ProcessTerminatedBySignal,

    #[error("process exited with a non-zero status code: {code}")]
    ExitCode { code: i32, stdout: String },

    #[error("command output is not valid UTF-8")]
    FromUTF8(#[from] FromUtf8Error),
}

pub fn execute_yabai_cmd<C: YabaiCommand>(
    yabai_cmd: &C,
) -> Result<C::Output, YabaiCommandExecError> {
    let args = yabai_cmd.to_args();
    trace!("Invoking yabai with args: {args:?}");
    let output = Command::new("yabai").args(args).output()?;
    let stdout = String::from_utf8(output.stdout)?;
    trace!("yabai command stdout: {stdout}");

    if !output.status.success() {
        return Err(match output.status.code() {
            Some(code) => YabaiCommandExecError::ExitCode { code, stdout },
            None => YabaiCommandExecError::ProcessTerminatedBySignal,
        });
    }

    Ok(yabai_cmd.parse_output(&stdout))
}

