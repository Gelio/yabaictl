use std::borrow::Cow;

use anyhow::Context;
use clap::ValueEnum;

use crate::{
    label::space::create_space_with_label,
    yabai::{
        self,
        cli::execute_yabai_cmd,
        command::{FocusSpaceByIndex, QuerySpaces},
    },
};

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
        log::info!("Only one space in the current display. It is already focused.");
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

    log::info!("Focusing space {}", *space_to_focus.index);
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
    .context("Could not queyr spaces")?
    .context("Could not parse spaces")?;

    let (space_to_focus, space_label_to_focus) = {
        let space_with_prefix_result = find_space_with_prefix(&spaces, label_prefix);

        if matches!(
            space_with_prefix_result,
            Err(FindSpaceWithLabelPrefixError::NoSpacesFoundWithLabelPrefix { .. })
        ) && create_space_if_not_found
        {
            log::debug!("Space with label prefix {label_prefix} not found. Creating a new one");

            let created_space = create_space_with_label(label_prefix.to_owned())
                .with_context(|| format!("Could not create space with label {label_prefix}"))?;

            Ok((Cow::Owned(created_space), label_prefix))
        } else {
            space_with_prefix_result.map(|(space, label)| (Cow::Borrowed(space), label))
        }
    }
    .context("Cannot find the space to focus")?;

    log::info!("Focusing space {space_label_to_focus}");
    execute_yabai_cmd(&FocusSpaceByIndex::new(space_to_focus.index))
        .with_context(|| format!("Cannot focus space with index {}", *space_to_focus.index))
}

#[derive(Debug, thiserror::Error)]
enum FindSpaceWithLabelPrefixError {
    #[error("No spaces found with label prefix \"{label_prefix}\"")]
    NoSpacesFoundWithLabelPrefix { label_prefix: String },

    #[error(
        "More than one space found with prefix \"{label_prefix}\": {matching_spaces_labels:?}"
    )]
    MoreThanOneSpaceFoundWithLabelPrefix {
        label_prefix: String,
        matching_spaces_labels: Vec<String>,
    },
}

fn find_space_with_prefix<'s>(
    spaces: &'s [yabai::transport::Space],
    label_prefix: &str,
) -> Result<(&'s yabai::transport::Space, &'s str), FindSpaceWithLabelPrefixError> {
    let spaces_with_prefix: Vec<_> = spaces
        .iter()
        .filter_map(|space| match space.label.as_deref() {
            Some(label) if label.starts_with(label_prefix) => Some((space, label)),
            _ => None,
        })
        .collect();

    match spaces_with_prefix.len() {
        0 => Err(
            FindSpaceWithLabelPrefixError::NoSpacesFoundWithLabelPrefix {
                label_prefix: label_prefix.to_owned(),
            },
        ),
        1 => Ok(spaces_with_prefix
            .into_iter()
            .next()
            .expect("The vector contains exactly one element")),
        _ => Err(
            FindSpaceWithLabelPrefixError::MoreThanOneSpaceFoundWithLabelPrefix {
                label_prefix: label_prefix.to_owned(),
                matching_spaces_labels: spaces_with_prefix
                    .into_iter()
                    .map(|(_, label)| label.to_owned())
                    .collect::<Vec<_>>(),
            },
        ),
    }
}
