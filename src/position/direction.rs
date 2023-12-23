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

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::South => write!(f, "south"),
            Direction::North => write!(f, "north"),
            Direction::East => write!(f, "east"),
            Direction::West => write!(f, "west"),
        }
    }
}

impl Direction {
    pub fn into_opposite(self) -> Self {
        match self {
            Direction::South => Direction::North,
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}
