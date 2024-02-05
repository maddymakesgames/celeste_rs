use serde::{Deserialize, Serialize};

use crate::saves::def::{Areas, Poem};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelSetStats {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "Areas")]
    pub areas: Areas,
    #[serde(rename = "Poem")]
    pub poem: Poem,
    #[serde(rename = "UnlockedAreas")]
    pub unlocked_areas: u8,
    #[serde(rename = "TotalStrawberries")]
    pub total_strawberries: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelSets {
    #[serde(default)]
    #[serde(rename = "LevelSetStats")]
    pub(crate) level_set_stats: Vec<LevelSetStats>,
}
