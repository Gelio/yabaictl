use super::transport::SpaceIndex;

pub trait YabaiCommand {
    fn to_args(&self) -> Vec<String>;
}

pub struct FocusSpaceByIndex {
    pub index: SpaceIndex,
}

impl YabaiCommand for FocusSpaceByIndex {
    fn to_args(&self) -> Vec<String> {
        vec![
            "-m".to_string(),
            "space".to_string(),
            "--focus".to_string(),
            self.index.0.to_string(),
        ]
    }
}
