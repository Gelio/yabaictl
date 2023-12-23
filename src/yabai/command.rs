use crate::position::Direction;

use super::transport::{Display, DisplayIndex, Space, SpaceIndex, Window, WindowId};

pub trait YabaiCommand {
    type Output;

    // TODO: rename to `into_args` and consume self
    fn to_args(&self) -> Vec<String>;

    // TODO: remove &self
    fn parse_output(&self, output: &str) -> Self::Output;
}

pub struct FocusSpaceByIndex {
    index: SpaceIndex,
}

impl FocusSpaceByIndex {
    pub fn new(index: SpaceIndex) -> Self {
        Self { index }
    }
}

impl YabaiCommand for FocusSpaceByIndex {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            "--focus".to_string(),
            self.index.0.to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct QuerySpaceByIndex {
    index: SpaceIndex,
}

impl QuerySpaceByIndex {
    pub fn new(index: SpaceIndex) -> Self {
        Self { index }
    }
}

impl YabaiCommand for QuerySpaceByIndex {
    type Output = Result<Space, serde_json::Error>;

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "query".to_string(),
            "--spaces".to_string(),
            "--space".to_string(),
            self.index.0.to_string(),
        ]
    }

    fn parse_output(&self, output: &str) -> Self::Output {
        serde_json::from_str(output)
    }
}

pub struct QueryWindows;

impl YabaiCommand for QueryWindows {
    type Output = Result<Vec<Window>, serde_json::Error>;

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "query".to_string(),
            "--windows".to_string(),
        ]
    }

    fn parse_output(&self, output: &str) -> Self::Output {
        serde_json::from_str(output)
    }
}

pub struct FocusWindowById {
    id: WindowId,
}

impl FocusWindowById {
    pub fn new(id: WindowId) -> Self {
        Self { id }
    }
}

impl YabaiCommand for FocusWindowById {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "window".to_string(),
            "--focus".to_string(),
            self.id.to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct QuerySpaces {
    pub only_current_display: bool,
}

impl YabaiCommand for QuerySpaces {
    type Output = Result<Vec<Space>, serde_json::Error>;

    fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-m".to_string(),
            "query".to_string(),
            "--spaces".to_string(),
        ];

        if self.only_current_display {
            args.push("--display".to_string());
        }

        args
    }

    fn parse_output(&self, output: &str) -> Self::Output {
        serde_json::from_str(output)
    }
}

pub struct QueryDisplays;

impl YabaiCommand for QueryDisplays {
    type Output = Result<Vec<Display>, serde_json::Error>;

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "query".to_string(),
            "--displays".to_string(),
        ]
    }

    fn parse_output(&self, output: &str) -> Self::Output {
        serde_json::from_str(output)
    }
}

pub struct SendSpaceToDisplay {
    space_index: SpaceIndex,
    display_index: DisplayIndex,
}

impl SendSpaceToDisplay {
    pub fn new(space_index: SpaceIndex, display_index: DisplayIndex) -> Self {
        Self {
            space_index,
            display_index,
        }
    }
}

impl YabaiCommand for SendSpaceToDisplay {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            self.space_index.to_string(),
            "--display".to_string(),
            self.display_index.to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct LabelSpace {
    index: SpaceIndex,
    label: String,
}

impl LabelSpace {
    pub fn new(index: SpaceIndex, label: String) -> Self {
        Self { index, label }
    }
}

impl YabaiCommand for LabelSpace {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            self.index.to_string(),
            "--label".to_string(),
            self.label.clone(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct MoveSpace {
    pub source_label: String,
    pub target_label: String,
}

impl YabaiCommand for MoveSpace {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            self.source_label.clone(),
            "--move".to_string(),
            self.target_label.clone(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct MoveWindowToSpace {
    pub target_space_label: String,
}

impl YabaiCommand for MoveWindowToSpace {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "window".to_string(),
            "--space".to_string(),
            self.target_space_label.clone(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct CreateSpace;

impl YabaiCommand for CreateSpace {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            "--create".to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub struct DestoySpace {
    pub index: SpaceIndex,
}

impl YabaiCommand for DestoySpace {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            "--destroy".to_string(),
            self.index.to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}

pub enum WarpWindowArg {
    Direction(Direction),
    WindowId(WindowId),
}

impl std::fmt::Display for WarpWindowArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpWindowArg::Direction(direction) => direction.fmt(f),
            WarpWindowArg::WindowId(window_id) => window_id.fmt(f),
        }
    }
}

pub struct WarpWindow {
    arg: WarpWindowArg,
}

impl WarpWindow {
    pub fn new(arg: WarpWindowArg) -> Self {
        Self { arg }
    }
}

impl YabaiCommand for WarpWindow {
    type Output = ();

    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "window".to_string(),
            "--warp".to_string(),
            self.arg.to_string(),
        ]
    }

    fn parse_output(&self, _output: &str) -> Self::Output {}
}
