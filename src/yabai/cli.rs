use std::{io, process::Command};

use thiserror::Error;

use super::command::YabaiCommand;

#[derive(Error, Debug)]
pub enum YabaiCommandExecError {
    #[error("could not invoke command")]
    Exec(#[from] io::Error),
}

pub fn execute_yabai_cmd(yabai_cmd: &impl YabaiCommand) -> Result<(), YabaiCommandExecError> {
    let output = Command::new("yabai").args(yabai_cmd.to_args()).output()?;

    Ok(())
}

