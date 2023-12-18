use crate::{
    label::partition::partition_labelables,
    yabai::{self, cli::execute_yabai_cmd, command::QuerySpaces, transport::Space},
};
use anyhow::Context;
use log::{debug, info, warn};

pub fn label_spaces() -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Could not query spaces")?
    .context("Could not parse spaces")?;

    let partitioned_spaces = partition_labelables(spaces.into_iter());

    let incorrectly_labeled_spaces = partitioned_spaces.incorrectly_labeled();
    if !incorrectly_labeled_spaces.is_empty() {
        warn!(
            "Detected {len} spaces with incorrect labels: {labels:#?}",
            len = incorrectly_labeled_spaces.len(),
            labels = incorrectly_labeled_spaces
                .iter()
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
