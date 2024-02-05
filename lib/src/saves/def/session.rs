use serde::{Deserialize, Serialize};

use crate::saves::def::{util::RespawnPoint, AreaDef, EntityID, LastAreaRef, Modes};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentSession {
    #[serde(rename = "Area")]
    pub area: LastAreaRef,
    #[serde(rename = "RespawnPoint")]
    pub respawn_point: RespawnPoint,
    #[serde(rename = "Audio")]
    pub audio: Audio,
    #[serde(rename = "Inventory")]
    pub inventory: Inventory,
    #[serde(rename = "Flags")]
    pub flags: LevelFlags,
    #[serde(rename = "LevelFlags")]
    pub level_flags: LevelFlags,
    #[serde(rename = "Strawberries")]
    pub strawberries: Strawberries,
    #[serde(rename = "DoNotLoad")]
    pub do_not_load: DoNotLoad,
    #[serde(rename = "Keys")]
    pub keys: Keys,
    #[serde(rename = "Counters")]
    pub counters: Counters,
    #[serde(rename = "SummitGems")]
    pub summit_gems: SummitGems,
    #[serde(rename = "OldStats")]
    pub old_stats: OldStats,
    #[serde(rename = "UnlockedCSide")]
    pub unlocked_c_side: bool,
    #[serde(rename = "FurthestSeenLevel")]
    pub furthest_seen_level: String,
    #[serde(rename = "BeatBestTime")]
    pub beat_best_time: bool,
    #[serde(rename = "RestartedFromGolden")]
    pub restarted_from_golden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelFlags {
    #[serde(default)]
    #[serde(rename = "string")]
    pub(crate) level_flags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Strawberries {
    #[serde(default)]
    #[serde(rename = "EntityID")]
    pub(crate) strawberries: Vec<EntityID>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoNotLoad {
    #[serde(default)]
    #[serde(rename = "EntityID")]
    pub(crate) do_not_load: Vec<EntityID>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keys {
    #[serde(default)]
    #[serde(rename = "EntityID")]
    pub(crate) keys: Vec<EntityID>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Counters {
    #[serde(default)]
    #[serde(rename = "Counter")]
    pub(crate) counters: Vec<Counter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SummitGems {
    #[serde(default)]
    #[serde(rename = "boolean")]
    pub(crate) summit_gems: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Audio {
    #[serde(rename = "Music")]
    pub music: Music,
    #[serde(rename = "Ambience")]
    pub ambience: Music,
    #[serde(rename = "AmbienceVolume")]
    pub ambience_volume: (),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Music {
    #[serde(rename = "@Event")]
    pub event: String,
    #[serde(rename = "Parameters")]
    pub parameters: Parameters,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameters {
    #[serde(default)]
    #[serde(rename = "MEP")]
    pub(crate) parameters: Vec<MusicParam>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MusicParam {
    #[serde(rename = "@Key")]
    pub key: String,
    #[serde(rename = "@Value")]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inventory {
    #[serde(rename = "Dashes")]
    pub dashes: u8,
    #[serde(rename = "DreamDash")]
    pub dream_dash: bool,
    #[serde(rename = "Backpack")]
    pub backpack: bool,
    #[serde(rename = "NoRefills")]
    pub no_refills: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Counter {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OldStats {
    #[serde(flatten)]
    pub area: AreaDef,
    #[serde(rename = "Modes")]
    pub modes: Modes,
}
