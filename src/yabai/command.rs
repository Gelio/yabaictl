use super::transport::{Space, SpaceIndex};

pub trait YabaiCommand {
    type Output;

    fn to_args(&self) -> Vec<String>;

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

    fn parse_output(&self, _output: &str) -> Self::Output {
        ()
    }
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
