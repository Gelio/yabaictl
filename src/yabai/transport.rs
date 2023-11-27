use std::ops::Deref;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    #[serde(rename = "w")]
    pub width: f32,
    #[serde(rename = "h")]
    pub height: f32,
}

impl Frame {
    pub fn is_north_of(&self, other: &Frame) -> bool {
        self.y + self.height <= other.y
    }

    pub fn is_south_of(&self, other: &Frame) -> bool {
        other.is_north_of(self)
    }

    pub fn is_east_of(&self, other: &Frame) -> bool {
        self.x >= other.x + other.width
    }

    pub fn is_west_of(&self, other: &Frame) -> bool {
        other.is_east_of(self)
    }

    pub fn overlaps_vertically(&self, other: &Frame) -> bool {
        let other_starts_north = other.y <= self.y;
        if other_starts_north {
            let other_y_end = other.y + other.height;
            other_y_end >= self.y
        } else {
            let y_end = self.y + self.height;
            other.y <= y_end
        }
    }

    pub fn overlaps_horizontally(&self, other: &Frame) -> bool {
        let other_starts_east = other.x <= self.x;
        if other_starts_east {
            let other_x_end = other.x + other.width;
            other_x_end >= self.x
        } else {
            let x_end = self.x + self.width;
            other.x <= x_end
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct SpaceId(pub u32);

/// Index used by MacOS Mission Control.
/// Changes when the space is moved between displays
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct SpaceIndex(pub u32);

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct DisplayId(pub u32);

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct DisplayIndex(pub u32);

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct WindowId(pub u32);

impl Deref for WindowId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct Display {
    pub id: DisplayId,
    pub uuid: String,
    pub index: DisplayIndex,
    pub frame: Frame,
    pub spaces: Vec<SpaceIndex>,
}

impl<'a> From<&'a Display> for &'a Frame {
    fn from(value: &'a Display) -> Self {
        &value.frame
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceType {
    BSP,
    Stack,
}

fn deserialize_window_id_maybe_zero<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<WindowId>, D::Error> {
    let window_id = WindowId::deserialize(deserializer)?;

    Ok(match window_id.0 {
        0 => None,
        _ => Some(window_id),
    })
}

fn deserialize_space_label<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let label = String::deserialize(deserializer)?;

    Ok(if label.is_empty() { None } else { Some(label) })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Space {
    pub id: SpaceId,
    pub uuid: String,

    pub index: SpaceIndex,

    #[serde(deserialize_with = "deserialize_space_label")]
    pub label: Option<String>,

    pub r#type: SpaceType,
    #[serde(rename = "display")]
    pub display_index: DisplayIndex,
    pub windows: Vec<WindowId>,

    #[serde(deserialize_with = "deserialize_window_id_maybe_zero")]
    pub first_window: Option<WindowId>,
    #[serde(deserialize_with = "deserialize_window_id_maybe_zero")]
    pub last_window: Option<WindowId>,

    pub has_focus: bool,
    pub is_visible: bool,
    pub is_native_fullscreen: bool,
}

/// Only relevant window properties were included.
/// See `man yabai` for all properties.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Window {
    pub id: WindowId,
    pub pid: u32,
    pub app: String,
    pub title: String,
    pub frame: Frame,

    #[serde(rename = "display")]
    pub display_index: DisplayIndex,
    #[serde(rename = "space")]
    pub space_index: SpaceIndex,

    pub has_focus: bool,
    pub is_visible: bool,
    pub is_hidden: bool,
    pub is_floating: bool,
    pub is_sticky: bool,
}

impl<'a> From<&'a Window> for &'a Frame {
    fn from(value: &'a Window) -> Self {
        &value.frame
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn frame_west_east() {
        assert!(Frame {
            x: -100.0,
            y: 0.0,
            width: 100.0,
            height: 100.0
        }
        .is_west_of(&Frame {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0
        }));

        assert!(Frame {
            x: 100.0,
            y: 0.0,
            width: 100.0,
            height: 100.0
        }
        .is_east_of(&Frame {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0
        }));
    }

    #[test]
    fn frame_north_south() {
        assert!(Frame {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0
        }
        .is_north_of(&Frame {
            x: 0.0,
            y: 150.0,
            width: 10.0,
            height: 10.0
        }));

        assert!(Frame {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0
        }
        .is_south_of(&Frame {
            x: 0.0,
            y: -10.0,
            width: 10.0,
            height: 10.0
        }));
    }
}
