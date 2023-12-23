use anyhow::{anyhow, Context};

use crate::{
    position::{get_element_to_focus, Direction},
    yabai::{
        self,
        cli::execute_yabai_cmd,
        command::{QueryDisplays, QuerySpaces, SendSpaceToDisplay},
    },
};

pub fn move_space_in_direction(
    direction: Direction,
    create_extra_space_if_last_on_display: bool,
) -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Could not query spaces")?
    .context("Could not parse spaces")?;

    let displays = execute_yabai_cmd(&QueryDisplays)
        .context("Could not query displays")?
        .context("Could not parse displays")?;

    let active_space = spaces
        .iter()
        .find(|space| space.has_focus)
        .ok_or(anyhow!("No space has focus"))?;

    let active_display = displays
        .iter()
        .find(|display| display.index == active_space.display_index)
        .ok_or_else(|| {
            anyhow!("Could not find the display for the active space {active_space:?}",)
        })?;

    let Some(target_display) = get_element_to_focus(&active_display.frame, &displays, direction)
    else {
        anyhow::bail!("No display found in direction {direction:?}")
    };

    if active_display.spaces.len() == 1 {
        if create_extra_space_if_last_on_display {
            log::info!("The active space is the only one in the display {:?}. Creating a new one to allow moving the active space", target_display.index);

            execute_yabai_cmd(&yabai::command::CreateSpace).context("Cannot create a new space")?;
        } else {
            log::warn!("The active space is the only one in the display {:?}. Yabai will most likely fail to send it to another display", target_display.index);
        }
    }

    log::info!(
        "Sending the space {:?} to display {:?}",
        active_space.index,
        target_display.index
    );

    execute_yabai_cmd(&SendSpaceToDisplay::new(
        active_space.index,
        target_display.index,
    ))
    .with_context(|| {
        format!(
            "Could not send space {} to display {}",
            *active_space.index, *target_display.index
        )
    })?;

    Ok(())
}
