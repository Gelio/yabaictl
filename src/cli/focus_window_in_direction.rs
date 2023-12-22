use anyhow::{anyhow, Context};
use log::{info, warn};

use crate::{
    position::{get_element_to_focus, Direction},
    yabai::{
        cli::execute_yabai_cmd,
        command::{FocusSpaceByIndex, FocusWindowById, QueryDisplays, QuerySpaces, QueryWindows},
        transport::{Display, Frame, Space, Window},
    },
};

pub fn focus_window_in_direction(direction: Direction) -> anyhow::Result<()> {
    let windows = execute_yabai_cmd(&QueryWindows)
        .context("Could not query windows")?
        .context("Could not parse windows")?;

    let active_ui_element =
        find_active_ui_element(&windows).context("Could not find the active UI element")?;

    let other_windows: Vec<_> = windows
        .iter()
        .filter(|window| window.is_visible)
        .filter(|window| match active_ui_element {
            ActiveUIElement::Window(active_window) => !std::ptr::eq(*window, active_window),
            ActiveUIElement::Space(..) => true,
        })
        .collect();

    let focused_frame: &Frame = active_ui_element.as_ref();

    if let Some(window_to_focus) = get_element_to_focus(focused_frame, &other_windows, direction) {
        info!("Focusing window with ID {}", window_to_focus.id.0);

        let _ = execute_yabai_cmd(&FocusWindowById::new(window_to_focus.id))
            .with_context(|| format!("Could not focus window with ID {}", window_to_focus.id.0));
    } else {
        warn!("No window in direction {:?}", direction);

        let visible_spaces: Vec<_> = get_spaces()?
            .into_iter()
            .filter(|space| space.is_visible && !space.has_focus)
            .collect();

        let displays: Vec<_> = get_displays()?;
        let spaces_with_frames: Vec<_> = visible_spaces
            .into_iter()
            .map(|space| {
                let display = displays
                    .iter()
                    .find(|display| display.index == space.display_index)
                    .expect("Each space belongs to some display");

                SpaceWithFrame {
                    space,
                    frame: display.frame.clone(),
                }
            })
            .collect();

        match get_element_to_focus(focused_frame, &spaces_with_frames, direction) {
            Some(space_to_focus) => {
                info!("Focusing space with index {:?}", space_to_focus.space.index);

                let _ = execute_yabai_cmd(&FocusSpaceByIndex::new(space_to_focus.space.index))
                    .with_context(|| {
                        format!(
                            "Could not focus space with index {:?}",
                            space_to_focus.space.index
                        )
                    });
            }

            None => {
                warn!("No space in direction {:?}", direction);
            }
        }
    }

    Ok(())
}

enum ActiveUIElement<'w> {
    Space(SpaceWithFrame),
    Window(&'w Window),
}

impl<'w> AsRef<Frame> for ActiveUIElement<'w> {
    fn as_ref(&self) -> &Frame {
        match self {
            ActiveUIElement::Space(SpaceWithFrame { space: _, frame }) => frame,
            ActiveUIElement::Window(window) => &window.frame,
        }
    }
}

fn find_active_ui_element(windows: &[Window]) -> anyhow::Result<ActiveUIElement> {
    let active_window = windows.iter().find(|window| window.has_focus);

    if let Some(window) = active_window {
        return Ok(ActiveUIElement::Window(window));
    }

    let focused_space = get_spaces()?
        .into_iter()
        .find(|space| space.has_focus)
        .ok_or_else(|| anyhow!("No space has focus"))?;

    let display_with_focused_space = get_displays()?
        .into_iter()
        .find(|display| display.index == focused_space.display_index)
        .ok_or_else(|| {
            anyhow!(
                "Could not find display for the focused space {}",
                focused_space.index.0
            )
        })?;

    Ok(ActiveUIElement::Space(SpaceWithFrame {
        space: focused_space,
        frame: display_with_focused_space.frame,
    }))
}

fn get_spaces() -> anyhow::Result<Vec<Space>> {
    execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Could not query spaces")?
    .context("Could not parse spaces")
}

fn get_displays() -> anyhow::Result<Vec<Display>> {
    execute_yabai_cmd(&QueryDisplays)
        .context("Could not query displays")?
        .context("Could not parse displays")
}

#[derive(Debug)]
struct SpaceWithFrame {
    space: Space,
    frame: Frame,
}

impl AsRef<Frame> for SpaceWithFrame {
    fn as_ref(&self) -> &Frame {
        &self.frame
    }
}
