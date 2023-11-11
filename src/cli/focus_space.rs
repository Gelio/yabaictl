use anyhow::Context;

use crate::yabai::{
    cli::execute_yabai_cmd,
    command::{FocusSpaceByIndex, QuerySpaceByIndex},
    transport::SpaceIndex,
};

pub fn focus_space_by_index(index: SpaceIndex) -> anyhow::Result<()> {
    execute_yabai_cmd(&QuerySpaceByIndex::new(index))
        .context("Could not query space by index in yabai")?
        .context("Could not parse query space output")?;

    execute_yabai_cmd(&FocusSpaceByIndex::new(index)).context("Could not focus space")
}
