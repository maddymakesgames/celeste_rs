use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::{deserialize_bool_from_anything, deserialize_number_from_string};

use crate::saves::{
    def::{util::RespawnPoint, AreaDef, EntityID, FileTime, Modes},
    vanilla::AreaRef,
    DashCount,
    DeathCount,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionStats {
    /// The name of the screen the session is in
    #[serde(rename = "@Level")]
    pub level: String,
    #[serde(rename = "@Time")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub time: FileTime,
    #[serde(rename = "@StartedFromBeginning")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub started_from_beginning: bool,
    #[serde(rename = "@Deaths")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub deaths: DeathCount,
    #[serde(rename = "@Dashes")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub dashes: DashCount,
    #[serde(rename = "@DashesAtLevelStart")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub dashes_at_start: DashCount,
    #[serde(rename = "@DeathsInCurrentLevel")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub session_deaths: DeathCount,
    #[serde(rename = "@InArea")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub in_area: bool,
    #[serde(rename = "@FirstLevel")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub first_level: bool,
    #[serde(rename = "@Cassette")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub cassette: bool,
    #[serde(rename = "@HeartGem")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub heart_gem: bool,
    #[serde(rename = "@Dreaming")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub dreaming: bool,
    #[serde(rename = "@LightingAlphaAdd")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lighting_alpha_add: f32,
    #[serde(rename = "@BloomBaseAdd")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub bloom_base_add: f32,
    #[serde(rename = "@DarkRoomAlpha")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub dark_room_alpha: f32,
    #[serde(rename = "@CoreMode")]
    pub core_more: String,
    #[serde(rename = "@GrabbedGolden")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub grabbed_golden: bool,
    #[serde(rename = "@HitCheckpoint")]
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub hit_checkpoint: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedSession {
    #[serde(flatten)]
    pub stats: SessionStats,
    #[serde(rename = "Area")]
    pub area: AreaRef,
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
    //TODO: Figure this out
    /// The furthest screen reached in the map
    ///
    /// Don't currently know what causes this to be `None` but I just know it can be
    #[serde(rename = "FurthestSeenLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub furthest_seen_level: Option<String>,
    #[serde(rename = "BeatBestTime")]
    pub beat_best_time: bool,
    #[serde(rename = "RestartedFromGolden")]
    // Seems to be None in vanilla saves
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restarted_from_golden: Option<bool>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambience_volume: Option<AmbienceVolume>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmbienceVolume {
    #[serde(rename(serialize = "@xsi:nil", deserialize = "@nil"))]
    pub val: bool,
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
