use std::str::FromStr;

use anyhow::anyhow;
use clap::ValueEnum;

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
