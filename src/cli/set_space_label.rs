use anyhow::Context;
use clap::Args;

use crate::{
    label::{space::StableSpaceIndex, Labelable},
    yabai::{self, cli::execute_yabai_cmd, command::QuerySpaces, transport::Space},
};

#[derive(Args)]
pub struct SetSpaceLabelArgs {
    #[arg(long = "stable-index")]
    stable_index: Option<StableSpaceIndex>,
    #[arg(long = "description")]
    description: Option<String>,
}

pub fn set_space_label(args: SetSpaceLabelArgs) -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Cannot query yabai spaces")?
    .context("Cannot parse spaces")?;

    let (active_spaces, inactive_spaces): (Vec<_>, Vec<_>) =
        spaces.iter().partition(|space| space.has_focus);

    let active_space = *active_spaces
        .first()
        .context("Active space cannot be found")?;

    let stable_index = {
        if let Some(stable_index) = args.stable_index {
            let existing_space_with_same_stable_index = inactive_spaces.iter().find(|space| {
                space
                    .label
                    .as_deref()
                    .and_then(|label| Space::parse_index(label).ok())
                    .map(|other_stable_index| other_stable_index == stable_index)
                    .unwrap_or(false)
            });

            anyhow::ensure!(
                existing_space_with_same_stable_index.is_none(),
                "There is a space with the stable index {stable_index:?}",
            );
            stable_index
        } else {
            let active_space_label = active_space.label.as_deref().context("Space stable index not provided and the active space does not have a stable index to reuse")?;
            Space::parse_index(active_space_label).with_context(|| format!("Cannot parse the stable index from the active space label {active_space_label}"))?
        }
    };

    let label = Space::label(stable_index, args.description.as_deref());
    execute_yabai_cmd(&yabai::command::LabelSpace::new(
        active_space.index,
        label.clone(),
    ))
    .with_context(|| {
        format!(
            "Cannot set label {label} for space with index {:?}",
            active_space.index
        )
    })?;

    Ok(())
}
