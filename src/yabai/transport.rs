use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    #[serde(rename = "w")]
    pub width: u32,
    #[serde(rename = "h")]
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct SpaceId(usize);

/// Index used by MacOS Mission Control.
/// Changes when the space is moved between displays
#[derive(Debug, Deserialize)]
pub struct SpaceIndex(usize);

#[derive(Debug, Deserialize)]
pub struct DisplayId(usize);

#[derive(Debug, Deserialize)]
pub struct DisplayIndex(usize);

#[derive(Debug, Deserialize)]
pub struct WindowId(usize);

#[derive(Debug, Deserialize)]
pub struct Display {
    pub id: DisplayId,
    pub uuid: String,
    pub index: DisplayIndex,
    pub frame: Frame,
    pub spaces: Vec<SpaceIndex>,
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
    pub id: u32,
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
