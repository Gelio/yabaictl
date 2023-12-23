use anyhow::Context;

use crate::{
    position::{get_element_to_focus, Direction},
    yabai::{
        self,
        cli::execute_yabai_cmd,
        transport::{Space, Window},
    },
};

pub fn move_window_in_direction(direction: Direction) -> anyhow::Result<()> {
    // TODO:
    // 1. Find a window in <direction> in the same display
    // 2. If found, do a yabai -m window --warp <direction>
    // 3. If not found, find a window in <direction> in another display
    // 4. If found, yabai -m window --warp <target window ID> && yabai -m window --warp <opposite direction>
    // 5. If not found, yabai -m window --space <space ID in a given direction>
    let IntrospectedWindows {
        active_window,
        other_visible_windows,
    } = introspect_windows()?;

    match get_element_to_focus(&active_window.frame, &other_visible_windows, direction) {
        Some(target_window) => {
            log::trace!("The closest window in direction {direction:?} is {target_window:?}");

            if target_window.space_index == active_window.space_index {
                log::info!(
                    "Detected a window in direction {direction:?} in the same space ({:?}). Doing a regular yabai warp",
                    active_window.space_index
                );

                execute_yabai_cmd(&yabai::command::WarpWindow::new(
                    yabai::command::WarpWindowArg::Direction(direction),
                ))
                .with_context(|| format!("Cannot warp window in direction {direction:?}"))
            } else {
                log::info!(
                    "The closest window in direction {direction:?} is in another space ({:?}). Warping the active window to {:?}",
                    target_window.space_index,
                    target_window.id
                );

                execute_yabai_cmd(&yabai::command::WarpWindow::new(
                    yabai::command::WarpWindowArg::WindowId(target_window.id),
                ))
                .with_context(|| format!(
                    "Cannot warp window to target window with ID {target_window_id:?} ({target_window_title})",
                    target_window_id = target_window.id,
                    target_window_title = target_window.title
                ))?;

                log::info!("Focusing the warped window {:?}", active_window.id);
                execute_yabai_cmd(&yabai::command::FocusWindowById::new(active_window.id))
                    .with_context(|| format!("Cannot focus window {:?}", active_window.id))?;

                let opposite_direction = direction.into_opposite();
                log::info!("Now warping that window in the opposite direction ({opposite_direction:?}), so it is closer to the original space.");

                execute_yabai_cmd(&yabai::command::WarpWindow::new(
                    yabai::command::WarpWindowArg::Direction(opposite_direction),
                ))
                .with_context(|| {
                    format!("Cannot warp window to in opposite direction {opposite_direction:?}",)
                })
            }
        }
        None => {
            log::debug!("There are no windows in direction {direction:?}. Looking for spaces in that direction to move the window there");
            let target_space = find_space_in_direction(direction)
                .with_context(|| format!("Cannot find a space in direction {direction:?}"))?;
            log::debug!("Found target space {:?}", target_space.index);

            let target_space_specifier = target_space.index.to_string();
            log::info!("Moving the window to space {target_space_specifier}");

            execute_yabai_cmd(&yabai::command::MoveWindowToSpace {
                // TODO: convert to an enum for handling SpaceIndex and String
                target_space_label: target_space_specifier.clone(),
            })
            .with_context(|| format!("Cannot move window to space {target_space_specifier}"))?;

            log::info!("Focusing the moved window {:?}", active_window.id);
            execute_yabai_cmd(&yabai::command::FocusWindowById::new(active_window.id))
                .with_context(|| format!("Cannot focus window {:?}", active_window.id))
        }
    }
}

#[derive(Debug)]
struct IntrospectedWindows {
    active_window: Window,

    other_visible_windows: Vec<Window>,
}

fn introspect_windows() -> anyhow::Result<IntrospectedWindows> {
    let all_windows = execute_yabai_cmd(&yabai::command::QueryWindows)
        .context("Cannot query windows")?
        .context("Cannot parse windows")?;

    let (active_window, other_windows) = {
        let (active_windows, other_windows): (Vec<_>, Vec<_>) =
            all_windows.into_iter().partition(|window| window.has_focus);

        let active_window = match active_windows.len() {
            0 => anyhow::bail!("No windows have focus. Not sure which window to move"),
            1 => active_windows
                .into_iter()
                .next()
                .expect("There is exactly one active window"),
            _ => anyhow::bail!("Two windows have focus. Unreachable"),
        };

        (active_window, other_windows)
    };

    let other_visible_windows: Vec<_> = other_windows
        .into_iter()
        .filter(|window| window.is_visible)
        .collect();

    Ok(IntrospectedWindows {
        active_window,
        other_visible_windows,
    })
}

fn find_space_in_direction(direction: Direction) -> anyhow::Result<Space> {
    let spaces = execute_yabai_cmd(&yabai::command::QuerySpaces {
        only_current_display: false,
    })
    .context("Could not query spaces")?
    .context("Could not parse spaces")?;

    let displays = execute_yabai_cmd(&yabai::command::QueryDisplays)
        .context("Could not query displays")?
        .context("Could not parse displays")?;

    let active_space = spaces
        .iter()
        .find(|space| space.has_focus)
        .context("Cannot find the active space")?;

    let active_display = displays
        .iter()
        .find(|display| display.index == active_space.display_index)
        .with_context(|| {
            format!("Cannot find the display with the active space {active_space:?}")
        })?;

    log::trace!("Determined active display to be {active_space:?}");

    let target_display = get_element_to_focus(&active_display.frame, &displays, direction)
        .with_context(|| format!("Could not find a display in direction {direction:?}"))?;

    log::trace!("Determined target display in direction {direction:?} to be {target_display:?}");

    let visible_space_on_target_display = spaces
        .into_iter()
        .find(|space| space.display_index == target_display.index && space.is_visible)
        .with_context(|| {
            format!(
                "Cannot find the visible space on display with index {:?}",
                target_display.index
            )
        })?;

    log::trace!("Found space {visible_space_on_target_display:?} in direction {direction:?}");

    Ok(visible_space_on_target_display)
}
