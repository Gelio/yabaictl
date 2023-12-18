use std::{num::ParseIntError, ops::Deref, str::FromStr};

use thiserror::Error;

use crate::yabai::transport::Space;

use super::Labelable;

const SUPPORTED_STABLE_INDEXES: std::ops::RangeInclusive<u32> = 1..=10;

/// Space index that remains stable when the space moves across displays or is reordered.
///
/// Stable space index is enforced in yabai by using it as a prefix of the space label.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StableSpaceIndex(u32);

impl Deref for StableSpaceIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<StableSpaceIndex> for u32 {
    fn from(value: StableSpaceIndex) -> Self {
        value.0
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseStableSpaceIndexError {
    #[error("Cannot parse integer")]
    ParseIntError(#[from] ParseIntError),

    #[error("Number must be within the range [{}, {}]", SUPPORTED_STABLE_INDEXES.start(), SUPPORTED_STABLE_INDEXES.end())]
    OutOfBounds,
}

impl TryFrom<u32> for StableSpaceIndex {
    type Error = ParseStableSpaceIndexError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !SUPPORTED_STABLE_INDEXES.contains(&value) {
            Err(Self::Error::OutOfBounds)
        } else {
            Ok(Self(value))
        }
    }
}

impl FromStr for StableSpaceIndex {
    type Err = ParseStableSpaceIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u32 = s
            .parse()
            .map_err(ParseStableSpaceIndexError::ParseIntError)?;

        number.try_into()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseSpaceLabelError {
    #[error("Colon is missing in the space label")]
    MissingColon,

    #[error("Cannot parse stable index from label prefix \"{prefix}\"")]
    ParseStableIndexError {
        prefix: String,

        #[source]
        cause: ParseStableSpaceIndexError,
    },
}

impl Labelable for Space {
    const INDEX_RANGE: std::ops::RangeInclusive<u32> = SUPPORTED_STABLE_INDEXES;

    type Index = StableSpaceIndex;
    type ParseIndexError = ParseSpaceLabelError;

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn parse_index(label: &str) -> Result<Self::Index, Self::ParseIndexError> {
        let stringified_stable_label = match label.find(':') {
            Some(colon_index) => &label[0..colon_index],
            None => return Err(ParseSpaceLabelError::MissingColon),
        };

        stringified_stable_label.parse().map_err(|error| {
            ParseSpaceLabelError::ParseStableIndexError {
                prefix: stringified_stable_label.to_owned(),
                cause: error,
            }
        })
    }
}

impl Space {
    pub fn label(stable_index: StableSpaceIndex, description: Option<&str>) -> String {
        let mut output = stable_index.to_string();
        output.push(':');

        if let Some(description) = description {
            output.push(' ');
            output.push_str(description);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_space_labels() {
        assert_eq!("1: Hello", Space::label(StableSpaceIndex(1), Some("Hello")));
        assert_eq!("1:", Space::label(StableSpaceIndex(1), None));
    }

    #[test]
    fn gets_space_index_from_label() {
        assert_eq!(Ok(StableSpaceIndex(10)), Space::parse_index("10: hello"));

        assert_eq!(
            Err(ParseSpaceLabelError::MissingColon),
            Space::parse_index("1")
        );
        assert_eq!(
            Err(ParseSpaceLabelError::MissingColon),
            Space::parse_index("hello"),
        );
        assert!(matches!(
            Space::parse_index("hi: hello"),
            Err(ParseSpaceLabelError::ParseStableIndexError {
                prefix: _prefix,
                cause: ParseStableSpaceIndexError::ParseIntError(..)
            }),
        ));

        {
            let mut label = (SUPPORTED_STABLE_INDEXES.end() + 1).to_string();
            let stable_index_exceeding_max = label.clone();

            label.push_str(": hello");

            assert_eq!(
                Err(ParseSpaceLabelError::ParseStableIndexError {
                    prefix: stable_index_exceeding_max,
                    cause: ParseStableSpaceIndexError::OutOfBounds
                }),
                Space::parse_index(&label),
            );
        }
    }
}
