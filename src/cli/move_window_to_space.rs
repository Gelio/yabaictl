use anyhow::Context;

use crate::{
    label::{
        space::{create_space_with_label, StableSpaceIndex},
        Labelable,
    },
    yabai::{self, cli::execute_yabai_cmd, command::QuerySpaces, transport::Space},
};

pub fn move_window_to_space(
    stable_space_index: StableSpaceIndex,
    create_space_if_not_found: bool,
) -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Cannot query yabai spaces")?
    .context("Cannot parse spaces")?;

    let target_space_label = match spaces.into_iter().find_map(|other_space| {
        let other_space_label = other_space.label?;
        let other_space_index = Space::parse_index(&other_space_label).ok()?;

        (stable_space_index == other_space_index).then_some(other_space_label)
    }) {
        Some(target_space_label) => target_space_label,
        None if create_space_if_not_found => {
            let label = Space::label(stable_space_index, None);
            create_space_with_label(label.to_owned())
                .with_context(|| format!("Cannot create new space with label {label}"))?;

            label
        }
        _ => {
            anyhow::bail!(
                "Cannot find space with stable index {:?}",
                stable_space_index
            )
        }
    };

    execute_yabai_cmd(&yabai::command::MoveWindowToSpace {
        target_space_label: target_space_label.clone(),
    })
    .with_context(|| format!("Cannot move window to space {}", target_space_label))
}
