use serde::{Deserialize, Serialize};

use crate::saves::{
    AreaCount,
    StrawberryCount,
    def::{Areas, Poem},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelSetStats {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "Areas")]
    pub areas: Areas,
    #[serde(rename = "Poem")]
    pub poem: Poem,
    #[serde(rename = "UnlockedAreas")]
    pub unlocked_areas: AreaCount,
    #[serde(rename = "TotalStrawberries")]
    pub total_strawberries: StrawberryCount,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LevelSets {
    #[serde(default)]
    #[serde(rename = "LevelSetStats")]
    pub(crate) level_set_stats: Vec<LevelSetStats>,
}
