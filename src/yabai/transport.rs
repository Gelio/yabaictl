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

#[derive(Debug, Deserialize)]
pub struct DisplayId(usize);

#[derive(Debug, Deserialize)]
pub struct WindowId(usize);

#[derive(Debug, Deserialize)]
pub struct Display {
    pub id: u32,
    pub uuid: String,
    pub index: u32,
    pub frame: Frame,
    pub spaces: Vec<SpaceId>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceType {
    BSP,
    Stack,
}

fn deserialize_window_id<'de, D: Deserializer<'de>>(
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
    pub id: u32,
    pub uuid: String,

    /// Index used by MacOS Mission Control.
    /// Changes when the space is moved between displays
    pub index: u32,

    #[serde(deserialize_with = "deserialize_space_label")]
    pub label: Option<String>,

    pub r#type: SpaceType,
    pub display: DisplayId,
    pub windows: Vec<WindowId>,

    #[serde(deserialize_with = "deserialize_window_id")]
    pub first_window: Option<WindowId>,
    #[serde(deserialize_with = "deserialize_window_id")]
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
    pub display: DisplayId,
    pub space: SpaceId,
    pub has_focus: bool,
    pub is_visible: bool,
    pub is_hidden: bool,
    pub is_floating: bool,
    pub is_sticky: bool,
}
