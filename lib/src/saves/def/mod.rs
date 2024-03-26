//! Provides the data types used to store the parsed celeste data
//!
//! While every struct and field is public, it is not always ideal to directly edit the structures
//! as some are somewhat uninituitive.
pub mod everest;
pub mod session;
pub mod util;
pub mod vanilla;

use everest::*;
use serde::{Deserialize, Serialize};
use session::*;
use util::*;
use vanilla::*;

use chrono::NaiveDateTime;

pub type AreaCount = i16;
pub type StrawberryCount = u16;
pub type DeathCount = u64;
pub type DashCount = u64;
pub type JumpCount = u64;
pub type AreaId = u16;

/// The root of a celeste save file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    /// The last celeste version that the save file was opened with
    #[serde(rename = "Version")]
    pub version: String,
    /// The name of the save file
    #[serde(rename = "Name")]
    pub name: String,
    /// The total playtime on the save file
    #[serde(rename = "Time")]
    pub time: FileTime,
    /// The last time the save file was written to
    ///
    /// Does not appear to actually be updated?
    #[serde(rename = "LastSave")]
    pub last_save: NaiveDateTime,
    /// Whether cheat mode is enabled
    #[serde(rename = "CheatMode")]
    pub cheat_mode: bool,
    /// Whether assist mode is enabled
    #[serde(rename = "AssistMode")]
    pub assist_mode: bool,
    /// Whether variants are enabled
    #[serde(rename = "VariantMode")]
    pub variant_mode: bool,
    /// What assists are enabled
    #[serde(rename = "Assists")]
    pub assists: Assists,
    /// What Theo's sister is named
    ///
    /// This is only changed in vanilla if the save name is 'alex'
    #[serde(rename = "TheoSisterName")]
    pub theo_sister_name: String,
    /// The number of unlocked areas
    ///
    /// This only takes into account vanilla areas
    #[serde(rename = "UnlockedAreas")]
    pub unlocked_areas: AreaCount,
    /// The total amount of deaths in the save file
    #[serde(rename = "TotalDeaths")]
    pub total_deaths: DeathCount,
    /// The total amount of vanilla strawberries collected
    ///
    /// This does not account for modded strawberries but does count vanilla golden strawberries.
    #[serde(rename = "TotalStrawberries")]
    pub total_strawberries: StrawberryCount,
    /// The total amount of golden strawberries collected
    ///
    /// Unlike [total_strawberries](SaveData::total_strawberries) this DOES include modded berries
    #[serde(rename = "TotalGoldenStrawberries")]
    pub total_golden_strawberries: StrawberryCount,
    /// The total amount of jumps on the save file
    #[serde(rename = "TotalJumps")]
    pub total_jumps: JumpCount,
    /// The total amount of wall jumps on the save file
    #[serde(rename = "TotalWallJumps")]
    pub total_wall_jumps: JumpCount,
    /// The total amount of dashes on the save file
    #[serde(rename = "TotalDashes")]
    pub total_dashes: DashCount,
    /// The flags used in the vanilla storyline
    #[serde(rename = "Flags")]
    pub flags: Flags,
    /// The order of the heart poem in the journal
    #[serde(rename = "Poem")]
    pub poem: Poem,
    /// The gems unlocked in the summit a-side
    ///
    /// Is `None` if the save has never loaded 7a
    // TODO: idk if the option documentation is correct
    #[serde(rename = "SummitGems")]
    pub summit_gems: Option<SummitGems>,
    /// Whether farewell has been revealed
    #[serde(rename = "RevealedChapter9")]
    pub revealed_farewell: bool,
    /// The last vanilla area that was played
    #[serde(rename = "LastArea")]
    pub last_area: LastAreaRef,
    /// The saved session
    ///
    /// This is only used by vanilla celeste, everest uses [current_session_safe](SaveData::current_session_safe)
    #[serde(rename = "CurrentSession")]
    pub current_session: Option<SavedSession>,
    /// Data about each vanilla area
    ///
    /// While the vanilla levels also have an entry in [level_sets](SaveData::level_sets)
    /// the data stored here is what actually is used
    #[serde(rename = "Areas")]
    pub areas: Areas,
    /// Data about all the modded level sets that were loaded last time this save was played on
    #[serde(rename = "LevelSets")]
    #[serde(default)]
    #[serde(skip_serializing_if = "LevelSets::is_inner_empty")]
    pub level_sets: LevelSets,
    /// Data about the all the modded level sets ever loaded on this save file
    #[serde(rename = "LevelSetRecycleBin")]
    #[serde(default)]
    #[serde(skip_serializing_if = "LevelSets::is_inner_empty")]
    pub level_set_recycle_bin: LevelSets,
    /// Whether this save file has been loaded into everest before
    #[serde(rename = "HasModdedSaveData")]
    #[serde(default)]
    pub has_modded_save_data: bool,
    /// A reference to the last area played, including modded levels
    #[serde(rename = "LastArea_Safe")]
    pub last_area_safe: Option<LastAreaRef>,
    /// The current session saved on the file
    ///
    /// This is what is saved when you use "save & exit" while in a level
    #[serde(rename = "CurrentSession_Safe")]
    pub current_session_safe: Option<SavedSession>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Assists {
    #[serde(rename = "GameSpeed")]
    pub game_speed: u8,
    #[serde(rename = "Invincible")]
    pub invincible: bool,
    #[serde(rename = "DashMode")]
    pub dash_mode: DashMode,
    #[serde(rename = "DashAssist")]
    pub dash_assist: bool,
    #[serde(rename = "InfiniteStamina")]
    pub infinite_stamina: bool,
    #[serde(rename = "MirrorMode")]
    pub mirror_mode: bool,
    #[serde(rename = "ThreeSixtyDashing")]
    pub full_dashing: bool,
    #[serde(rename = "InvisibleMotion")]
    pub invisible_motion: bool,
    #[serde(rename = "NoGrabbing")]
    pub no_grabbing: bool,
    #[serde(rename = "LowFriction")]
    pub low_friction: bool,
    #[serde(rename = "SuperDashing")]
    pub super_dash: bool,
    #[serde(rename = "Hiccups")]
    pub hiccups: bool,
    #[serde(rename = "PlayAsBadeline")]
    pub badeline: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashMode {
    Normal,
    Two,
    Infinite,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Flags {
    #[serde(default)]
    #[serde(rename = "string")]
    pub(crate) flags: Vec<VanillaFlagsWrapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VanillaFlagsWrapper {
    #[serde(rename = "$text")]
    pub(crate) flag: VanillaFlags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poem {
    #[serde(default)]
    pub(crate) string: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VanillaFlags {
    MetTheo,
    TheoKnowsName,
}
