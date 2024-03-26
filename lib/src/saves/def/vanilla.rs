use serde::{Deserialize, Serialize};

use crate::saves::{
    def::{FileTime, Strawberries},
    AreaId,
    DashCount,
    DeathCount,
    StrawberryCount,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Areas {
    #[serde(rename = "AreaStats")]
    #[serde(default)]
    pub(crate) areas: Vec<AreaStats>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastAreaRef {
    #[serde(rename = "@ID")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub id: AreaId,
    #[serde(rename = "@Mode")]
    pub mode: String,
    #[serde(rename = "@SID")]
    /// Is `None` in a non-modded save
    pub s_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaStats {
    #[serde(flatten)]
    pub def: AreaDef,
    #[serde(rename = "Modes")]
    pub modes: Modes,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Modes {
    #[serde(rename = "AreaModeStats")]
    pub(crate) modes: Vec<AreaMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaDef {
    #[serde(rename = "@ID")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub id: AreaId,
    #[serde(rename = "@Cassette")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub cassette: bool,
    #[serde(rename = "@SID")]
    /// Is `None` in a non-modded save
    pub sid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaMode {
    #[serde(flatten)]
    pub stats: AreaModeStats,
    #[serde(rename = "Strawberries")]
    pub strawberries: Strawberries,
    #[serde(rename = "Checkpoints")]
    pub checkpoints: Checkpoints,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoints {
    #[serde(default)]
    #[serde(rename = "string")]
    pub(crate) checkpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaModeStats {
    #[serde(rename = "@TotalStrawberries")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub total_strawberries: StrawberryCount,
    #[serde(rename = "@Completed")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_bool_from_anything")]
    pub completed: bool,
    #[serde(rename = "@SingleRunCompleted")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_bool_from_anything")]
    pub single_run_completed: bool,
    #[serde(rename = "@FullClear")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_bool_from_anything")]
    pub full_clear: bool,
    #[serde(rename = "@Deaths")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub deaths: DeathCount,
    #[serde(rename = "@TimePlayed")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub time_played: FileTime,
    #[serde(rename = "@BestTime")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub best_time: FileTime,
    #[serde(rename = "@BestFullClearTime")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub best_full_clear_time: FileTime,
    #[serde(rename = "@BestDashes")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub best_dashes: DashCount,
    #[serde(rename = "@BestDeaths")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub best_deaths: DeathCount,
    #[serde(rename = "@HeartGem")]
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_bool_from_anything")]
    pub heart_gem: bool,
}
