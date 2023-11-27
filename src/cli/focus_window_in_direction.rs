use std::{cmp, str::FromStr};

use anyhow::{anyhow, Context};
use clap::ValueEnum;
use log::{info, warn};

use crate::yabai::{
    cli::execute_yabai_cmd,
    command::{FocusWindowById, QueryDisplays, QuerySpaces, QueryWindows},
    transport::{Display, Frame, Space, Window},
};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    West,
    South,
    East,
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "west" => Ok(Self::West),
            "east" => Ok(Self::East),
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            _ => Err(anyhow!(format!("{s} is not a valid direction"))),
        }
    }
}

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

    let window_candidates =
        get_candidates_in_direction(focused_frame, other_windows.iter().copied(), direction);

    let window_to_focus = find_closest_in_direction(
        Vector2D::from_frame_center(focused_frame),
        window_candidates.iter().copied(),
        |window| Vector2D::from_frame_center(&window.frame),
        direction.into(),
    );

    if let Some(window_to_focus) = window_to_focus {
        info!("Focusing window with ID {}", window_to_focus.id.0);

        let _ = execute_yabai_cmd(&FocusWindowById::new(window_to_focus.id))
            .with_context(|| format!("Could not focus window with ID {}", window_to_focus.id.0));
    } else {
        warn!("No window in direction {:?}", direction);
        // TODO: try focusing a space
    }

    Ok(())
}

enum ActiveUIElement<'w> {
    Space(Space, Display),
    Window(&'w Window),
}

impl<'w> AsRef<Frame> for ActiveUIElement<'w> {
    fn as_ref(&self) -> &Frame {
        match self {
            ActiveUIElement::Space(_, display) => &display.frame,
            ActiveUIElement::Window(window) => &window.frame,
        }
    }
}

fn find_active_ui_element(windows: &[Window]) -> anyhow::Result<ActiveUIElement> {
    let active_window = windows.iter().find(|window| window.has_focus);

    if let Some(window) = active_window {
        return Ok(ActiveUIElement::Window(window));
    }

    let spaces = execute_yabai_cmd(&QuerySpaces)
        .context("Could not query spaces")?
        .context("Could not parse spaces")?;

    let focused_space = spaces
        .into_iter()
        .find(|space| space.has_focus)
        .ok_or_else(|| anyhow!("No space has focus"))?;

    let displays = execute_yabai_cmd(&QueryDisplays)
        .context("Could not query displays")?
        .context("Could not parse displays")?;

    let display_with_focused_space = displays
        .into_iter()
        .find(|display| display.index == focused_space.display_index)
        .ok_or_else(|| {
            anyhow!(
                "Could not find display for the focused space {}",
                focused_space.index.0
            )
        })?;

    Ok(ActiveUIElement::Space(
        focused_space,
        display_with_focused_space,
    ))
}

fn get_candidates_in_direction<'a, 'b, T, U, Iter>(
    main: &'a T,
    candidates: Iter,
    direction: Direction,
) -> Vec<&'b U>
where
    &'a T: Into<&'a Frame>,
    &'b U: Into<&'b Frame>,
    Iter: Iterator<Item = &'b U>,
{
    let check_frame_direction = match direction {
        Direction::West => Frame::is_west_of,
        Direction::East => Frame::is_east_of,
        Direction::North => Frame::is_north_of,
        Direction::South => Frame::is_south_of,
    };
    let check_overlap = match direction {
        Direction::West | Direction::East => Frame::overlaps_vertically,
        Direction::North | Direction::South => Frame::overlaps_horizontally,
    };

    let main_frame: &Frame = main.into();

    candidates
        .filter(|&candidate| {
            let other_frame: &Frame = candidate.into();

            check_frame_direction(other_frame, main_frame) && check_overlap(other_frame, main_frame)
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn from_frame_center(frame: &Frame) -> Self {
        Self {
            x: frame.x + frame.width / 2.0,
            y: frame.y + frame.height / 2.0,
        }
    }

    pub fn dot(v1: Self, v2: Self) -> f32 {
        v1.x * v2.x + v1.y + v2.y
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl From<Direction> for Vector2D {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Self { x: 0.0, y: -1.0 },
            Direction::East => Self { x: 1.0, y: 0.0 },
            Direction::South => Self { x: 0.0, y: 1.0 },
            Direction::West => Self { x: -1.0, y: 0.0 },
        }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Vector2D {
    type Output = Vector2D;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

fn find_closest_in_direction<T, Iter, F>(
    center: Vector2D,
    candidates: Iter,
    candidate_to_vector: F,
    direction: Vector2D,
) -> Option<T>
where
    F: Fn(&T) -> Vector2D,
    Iter: Iterator<Item = T>,
{
    let mut candidates_with_points: Vec<_> = candidates
        .filter_map(|candidate| {
            let candidate_direction: Vector2D = candidate_to_vector(&candidate) - center;

            let cos = Vector2D::dot(direction, candidate_direction)
                / (direction.length() * candidate_direction.length());

            if cos.abs() < f32::EPSILON {
                // This candidate is almost perpendicular to the direction_vector
                None
            } else {
                let score = candidate_direction.length() / cos;
                Some((candidate, score))
            }
        })
        .collect();

    candidates_with_points.sort_by(|(_, score_a), (_, score_b)| {
        PartialOrd::partial_cmp(score_a, score_b).unwrap_or(cmp::Ordering::Less)
    });

    candidates_with_points
        .into_iter()
        .map(|(candidate, _)| candidate)
        .next()
}
