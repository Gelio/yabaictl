pub trait Labelable {
    type ParseIndexError;

    fn label(&self) -> Option<&str>;
    fn parse_index(label: &str) -> Result<u32, Self::ParseIndexError>;
}
