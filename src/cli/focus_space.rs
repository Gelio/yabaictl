use anyhow::Context;
use clap::ValueEnum;
use log::info;

use crate::yabai::{
    cli::execute_yabai_cmd,
    command::{FocusSpaceByIndex, QuerySpaceByIndex, QuerySpaces},
    transport::SpaceIndex,
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

pub fn focus_space_by_label(label_prefix: &str) -> anyhow::Result<()> {
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

    match spaces_with_prefix.len() {
        0 => anyhow::bail!("No spaces found with prefix {label_prefix}"),
        2.. => anyhow::bail!(
            "More than one space found with prefix {label_prefix}: {matching_spaces_labels:?}",
            matching_spaces_labels = spaces_with_prefix
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>()
        ),
        _ => {}
    }

    let (space_to_focus, space_label_to_focus) = spaces_with_prefix
        .into_iter()
        .next()
        .expect("There is only one element in the vector");

    info!("Focusing space {}", space_label_to_focus);
    execute_yabai_cmd(&FocusSpaceByIndex::new(space_to_focus.index))
        .with_context(|| format!("Cannot focus space with index {}", *space_to_focus.index))
}
