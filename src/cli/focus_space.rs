use anyhow::Context;
use clap::ValueEnum;
use log::{debug, info};

use crate::yabai::{
    self,
    cli::execute_yabai_cmd,
    command::{FocusSpaceByIndex, QuerySpaceByIndex, QuerySpaces},
    transport::{Space, SpaceIndex},
};

pub fn focus_space_by_index(index: SpaceIndex) -> anyhow::Result<()> {
    let space = execute_yabai_cmd(&QuerySpaceByIndex::new(index))
        .context("Could not query space by index in yabai")?
        .context("Could not parse query space output")?;

    if space.has_focus {
        info!("Space is alredy focused. Skipping");
        return Ok(());
    }

    execute_yabai_cmd(&FocusSpaceByIndex::new(index)).context("Could not focus space")
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum NextOrPrevious {
    Next,
    Previous,
}

pub fn focus_next_or_previous_space(next_or_previous: NextOrPrevious) -> anyhow::Result<()> {
    let spaces_in_display = execute_yabai_cmd(&QuerySpaces {
        only_current_display: true,
    })
    .context("Could not get spaes in the current display")?
    .context("Could not parse spaces")?;

    let active_space_index = spaces_in_display
        .iter()
        .position(|space| space.has_focus)
        .context("No space in the current display has focus")?;

    if spaces_in_display.len() == 1 {
        info!("Only one space in the current display. It is already focused.");
        return Ok(());
    }

    let space_to_focus_index = match next_or_previous {
        NextOrPrevious::Next => (active_space_index + 1) % spaces_in_display.len(),
        NextOrPrevious::Previous => {
            if active_space_index == 0 {
                spaces_in_display.len() - 1
            } else {
                active_space_index - 1
            }
        }
    };

    let space_to_focus = &spaces_in_display[space_to_focus_index];

    info!("Focusing space {}", *space_to_focus.index);
    execute_yabai_cmd(&FocusSpaceByIndex::new(space_to_focus.index))
        .with_context(|| format!("Could not focus space {}", *space_to_focus.index))
}

pub fn focus_space_by_label(
    label_prefix: &str,
    create_space_if_not_found: bool,
) -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Could not get spaes in the current display")?
    .context("Could not parse spaces")?;

    let spaces_with_prefix: Vec<_> = spaces
        .into_iter()
        .filter_map(|space| match space.label.as_ref() {
            Some(label) if label.starts_with(label_prefix) => {
                let label = label.clone();
                Some((space, label))
            }
            _ => None,
        })
        .collect();

    let (space_to_focus, space_label_to_focus) = match spaces_with_prefix.len() {
        0 => {
            if create_space_if_not_found {
                debug!("Space with label prefix {label_prefix} not found. Creating a new one");

                let created_space = create_space_with_label(label_prefix.to_owned())
                    .with_context(|| format!("Could not create space with label {label_prefix}"))?;

                (created_space, label_prefix.to_owned())
            } else {
                anyhow::bail!("No spaces found with prefix {label_prefix}")
            }
        }
        1 => spaces_with_prefix
            .into_iter()
            .next()
            .expect("The vector contains exactly one element"),
        _ => anyhow::bail!(
            "More than one space found with prefix {label_prefix}: {matching_spaces_labels:?}",
            matching_spaces_labels = spaces_with_prefix
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>()
        ),
    };

    info!("Focusing space {space_label_to_focus}");
    execute_yabai_cmd(&FocusSpaceByIndex::new(space_to_focus.index))
        .with_context(|| format!("Cannot focus space with index {}", *space_to_focus.index))
}

fn create_space_with_label(label: String) -> anyhow::Result<Space> {
    execute_yabai_cmd(&yabai::command::CreateSpace).context("Cannot create a new space")?;

    let spaces = execute_yabai_cmd(&yabai::command::QuerySpaces {
        only_current_display: true,
    })
    .context("Cannot query spaces")?
    .context("Cannot parse spaces")?;

    let created_space = spaces.into_iter().last().expect("The created space is added as the last one in the current display. It must have at least 1 space");

    debug!(
        "Created new space with index {:?} on display {:?}",
        created_space.index, created_space.display_index
    );

    execute_yabai_cmd(&yabai::command::LabelSpace::new(
        created_space.index,
        label.to_owned(),
    ))
    .with_context(|| {
        format!(
            "Cannot set label {label} to a space with index {:?}",
            created_space.index
        )
    })?;

    Ok(created_space)
}
