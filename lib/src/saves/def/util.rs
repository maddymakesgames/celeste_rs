use serde::{Deserialize, Serialize};
use std::{num::ParseIntError, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityID {
    #[serde(rename = "@Key")]
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RespawnPoint {
    #[serde(rename = "X")]
    pub x: i64,
    #[serde(rename = "Y")]
    pub y: i64,
}

/// Used to represent a duration of playtime.
///
/// The representation for this is the same as Win32's FileTime.
/// It measures 100-nanosecond intervals since January 1st, 1601.
///
/// *opinion*: This is stupid
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileTime(pub u64);

// We impl this here instead of in the impl module because this is relevant to parsing
impl FromStr for FileTime {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(FileTime)
    }
}
