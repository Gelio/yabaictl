pub trait Labelable {
    const INDEX_RANGE: std::ops::RangeInclusive<u32>;

    type Index: Into<u32> + TryFrom<u32>;
    type ParseIndexError;

    fn label(&self) -> Option<&str>;
    fn parse_index(label: &str) -> Result<Self::Index, Self::ParseIndexError>;
}
