use anyhow::{anyhow, Context};
use log::{info, warn};

use crate::{
    position::{get_element_to_focus, Direction},
    yabai::{
        cli::execute_yabai_cmd,
        command::{QueryDisplays, QuerySpaces, SendSpaceToDisplay},
    },
};

pub fn move_space_in_direction(direction: Direction) -> anyhow::Result<()> {
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

    if let Some(display_to_focus) =
        get_element_to_focus(&active_display.frame, &displays, direction)
    {
        info!("Focusing display {}", *display_to_focus.index);

        execute_yabai_cmd(&SendSpaceToDisplay::new(
            active_space.index,
            display_to_focus.index,
        ))
        .with_context(|| {
            format!(
                "Could not send space {} to display {}",
                *active_space.index, *display_to_focus.index
            )
        })?;
    } else {
        warn!("No display found in direction {direction:?}");
    }

    Ok(())
}
