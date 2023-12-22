use anyhow::Context;

use crate::{
    label::{space::StableSpaceIndex, Labelable},
    yabai::{self, cli::execute_yabai_cmd, command::QuerySpaces, transport::Space},
};

pub fn move_window_to_space(stable_space_index: StableSpaceIndex) -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Cannot query yabai spaces")?
    .context("Cannot parse spaces")?;

    let Some(target_space_label) = spaces.into_iter().find_map(|other_space| {
        let other_space_label = other_space.label?;
        let other_space_index = Space::parse_index(&other_space_label).ok()?;

        (stable_space_index == other_space_index).then_some(other_space_label)
    }) else {
        anyhow::bail!(
            "Cannot find space with stable index {:?}",
            stable_space_index
        );
    };

    execute_yabai_cmd(&yabai::command::MoveWindowToSpace {
        target_space_label: target_space_label.clone(),
    })
    .with_context(|| format!("Cannot move window to space {}", target_space_label))
}
