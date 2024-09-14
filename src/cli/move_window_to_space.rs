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

    let focused_space = spaces
        .iter()
        .find(|space| space.has_focus)
        .context("No space has focus")?;
    let focused_space_has_only_one_window = focused_space.windows.len() == 1;
    let focused_space_index = focused_space.index;

    let existing_target_space_label = spaces.into_iter().find_map(|other_space| {
        let other_space_label = other_space.label?;
        let other_space_index = Space::parse_index(&other_space_label).ok()?;

        (stable_space_index == other_space_index).then_some(other_space_label)
    });

    let move_window_to_space = |target_space_label: &str| {
        execute_yabai_cmd(&yabai::command::MoveWindowToSpace {
            target_space_label: target_space_label.to_owned(),
        })
        .with_context(|| format!("Cannot move window to space {}", target_space_label))
    };

    match existing_target_space_label {
        Some(target_space_label) => {
            log::info!(
                "Found target space with index {stable_space_index:?}, moving the window there"
            );
            move_window_to_space(&target_space_label)
        }
        None if create_space_if_not_found => {
            if focused_space_has_only_one_window {
                let label = Space::label(stable_space_index, None);

                log::info!("No target space with index {stable_space_index:?} found, but the current space only has one window. Relabeling the focused space to become the target space");

                execute_yabai_cmd(&yabai::command::LabelSpace::new(
                    focused_space_index,
                    label.clone(),
                ))
                .with_context(|| {
                    format!("Cannot set label \"{label:?}\" for space with index {focused_space_index:?}")
                })
            } else {
                log::info!("No target space with index {stable_space_index:?} found, creating a new space with the target index and moving the window there");
                let label = Space::label(stable_space_index, None);
                create_space_with_label(label.to_owned())
                    .with_context(|| format!("Cannot create new space with label {label}"))?;
                move_window_to_space(&label)
            }
        }
        None => anyhow::bail!(
            "Cannot find space with stable index {:?}",
            stable_space_index
        ),
    }
}
