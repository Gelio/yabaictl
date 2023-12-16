use std::num::ParseIntError;

use anyhow::Context;
use log::{debug, info, warn};
use thiserror::Error;

use crate::{
    label::{partition::partition_labelables, Labelable},
    yabai::{self, cli::execute_yabai_cmd, command::QuerySpaces, transport::Space},
};

const SUPPORTED_LABELED_SPACES: usize = 10;

/// One-based
const MAX_SPACE_INDEX: usize = SUPPORTED_LABELED_SPACES + 1;

pub fn label_spaces() -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Could not query spaces")?
    .context("Could not parse spaces")?;

    let partitioned_spaces = partition_labelables::<_, MAX_SPACE_INDEX>(spaces.into_iter());

    let incorrectly_labeled_spaces = partitioned_spaces.incorrectly_labeled();
    if !incorrectly_labeled_spaces.is_empty() {
        warn!(
            "Detected {len} spaces with incorrect labels: {labels:#?}",
            len = incorrectly_labeled_spaces.len(),
            labels = incorrectly_labeled_spaces
                .into_iter()
                .map(|(space, error)| (
                    space
                        .label
                        .as_deref()
                        .expect("Label must exist since it was attempted to be parsed"),
                    error
                ))
                .collect::<Vec<_>>()
        );
    }

    let assigned_indices = partitioned_spaces.into_assigned_indices();

    if !assigned_indices.leftover_items.is_empty() {
        warn!(
            "Cannot label {len} spaces because there are no empty indices left",
            len = assigned_indices.leftover_items.len()
        );
        debug!(
            "Space yabai indices that cannot be labeled: {:?}",
            assigned_indices
                .leftover_items
                .into_iter()
                .map(|space| space.index)
                .collect::<Vec<_>>()
        );
    }

    let spaces_to_label_len = assigned_indices.assigned_indices.len();
    for (space, index) in assigned_indices.assigned_indices {
        let label = Space::label(index, None);

        execute_yabai_cmd(&yabai::command::LabelSpace::new(space.index, label.clone()))
            .with_context(|| {
                format!(
                    "Cannot set label {label} for space with yabai index {index:?}",
                    index = space.index
                )
            })?;
    }

    info!("Labeled {spaces_to_label_len} spaces");
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidSpaceLabelIndexError {
    #[error("Colon is missing in the label")]
    MissingColon,

    #[error("Cannot parse index from label")]
    ParseIndexError(#[from] ParseIntError),

    #[error("Space index {label_index} too high, exceeds {SUPPORTED_LABELED_SPACES}")]
    IndexTooHigh { label_index: u32 },
}

impl Labelable for Space {
    type ParseIndexError = InvalidSpaceLabelIndexError;

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn parse_index(label: &str) -> Result<u32, Self::ParseIndexError> {
        let stringified_label_index = match label.find(':') {
            Some(colon_index) => &label[0..colon_index],
            // ASSUMPTION: label must start with a number
            None => label,
        };

        let label_index = stringified_label_index.parse()?;

        if label_index > SUPPORTED_LABELED_SPACES as u32 {
            Err(InvalidSpaceLabelIndexError::IndexTooHigh { label_index })
        } else {
            Ok(label_index)
        }
    }
}

impl Space {
    fn label(index: u32, extra_label: Option<String>) -> String {
        let suffix = match extra_label {
            Some(extra_label) => format!(" {extra_label}"),
            None => String::new(),
        };

        format!("{index}:{suffix}",)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_space_index_from_label() {
        assert_eq!(Ok(1), Space::parse_index("1"));
        assert_eq!(Ok(10), Space::parse_index("10: hello"));

        assert!(matches!(
            Space::parse_index("hello"),
            Err(InvalidSpaceLabelIndexError::ParseIndexError(..))
        ));

        assert_eq!(
            Err(InvalidSpaceLabelIndexError::IndexTooHigh {
                label_index: SUPPORTED_LABELED_SPACES as u32 + 1
            }),
            Space::parse_index(&(SUPPORTED_LABELED_SPACES + 1).to_string()),
        );
    }
}
