//! Implements operations for all of the types found in the [saves::def](crate::saves::def) module
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    io::BufRead,
    ops::{Deref, DerefMut},
};

use chrono::NaiveDateTime;
pub use quick_xml::DeError;

use crate::saves::{
    AreaCount,
    Assists,
    DashCount,
    DashMode,
    DeathCount,
    Flags,
    ModSaveData,
    Poem,
    SaveData,
    StrawberryCount,
    VanillaFlags,
    VanillaFlagsWrapper,
    def::{everest::LevelSetStats, vanilla::AreaStats},
    util::FileTime,
};

pub mod everest;
pub mod mods;
pub mod session;
pub mod util;
pub mod vanilla;

#[doc(hidden)]
pub const XML_VERSION_HEADER: &str = r#"<?xml version="1.0" encoding="utf-8"?>"#;
#[doc(hidden)]
pub const XSI_URL: &str = r#"http://www.w3.org/2001/XMLSchema-instance"#;
#[doc(hidden)]
pub const XSD_URL: &str = r#"http://www.w3.org/2001/XMLSchema"#;

fn area_sid_matches(area: &AreaStats, sid: &str) -> bool {
    area.def.sid.as_ref().is_some_and(|a_sid| a_sid == sid)
}

impl SaveData {
    pub fn from_reader(reader: impl BufRead) -> Result<Self, DeError> {
        quick_xml::de::from_reader(reader)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<Self, DeError> {
        quick_xml::de::from_str(str)
    }

    pub fn to_string(&self) -> Result<String, DeError> {
        let mut xml = XML_VERSION_HEADER.to_owned();
        xml.push_str(&quick_xml::se::to_string(&self)?);
        Ok(xml)
    }

    pub fn to_writer(&self, mut writer: impl Write) -> Result<(), DeError> {
        writer.write_str(XML_VERSION_HEADER)?;
        quick_xml::se::to_writer(writer, &self)
    }

    /// Combines all the [LevelSetStats] into one vec,
    /// with a boolean indicating whether they are in the recycle bin or not
    pub fn all_level_sets(&self) -> Vec<(&LevelSetStats, bool)> {
        self.level_sets
            .iter()
            .map(|a| (a, false))
            .chain(self.level_set_recycle_bin.iter().map(|a| (a, true)))
            .collect()
    }

    /// Combines all the [LevelSetStats] into one vec,
    /// with a boolean indicating whether they are in the recycle bin or not
    pub fn all_level_sets_mut(&mut self) -> Vec<(&mut LevelSetStats, bool)> {
        self.level_sets
            .iter_mut()
            .map(|a| (a, false))
            .chain(self.level_set_recycle_bin.iter_mut().map(|a| (a, true)))
            .collect()
    }

    /// Combines all the areas into one list, adding a tag that denotes what group the area is in
    pub fn all_areas(&self) -> Vec<(&AreaStats, AreaSource)> {
        self.areas
            .iter()
            .map(|a| (a, AreaSource::Vanilla))
            .chain(
                self.level_sets
                    .iter()
                    .flat_map(|a| a.areas.iter().map(|a| (a, AreaSource::LevelSets))),
            )
            .chain(
                self.level_set_recycle_bin
                    .iter()
                    .flat_map(|a| a.areas.iter().map(|a| (a, AreaSource::LevelSetRecycleBin))),
            )
            .collect()
    }

    /// Combines all the areas into one list, adding a tag that denotes what group the area is in
    pub fn all_areas_mut(&mut self) -> Vec<(&mut AreaStats, AreaSource)> {
        self.areas
            .iter_mut()
            .map(|a| (a, AreaSource::Vanilla))
            .chain(
                self.level_sets
                    .iter_mut()
                    .flat_map(|a| a.areas.iter_mut().map(|a| (a, AreaSource::LevelSets))),
            )
            .chain(self.level_set_recycle_bin.iter_mut().flat_map(|a| {
                a.areas
                    .iter_mut()
                    .map(|a| (a, AreaSource::LevelSetRecycleBin))
            }))
            .collect()
    }

    /// Returns a reference to an [AreaStats] given the area's sid
    ///
    /// Returns `None` if no area with the given sid is found.
    pub fn find_area_by_sid<'a>(&'a self, sid: &str) -> Option<&'a AreaStats> {
        if sid.starts_with("Celeste") {
            self.areas.iter().find(|a| area_sid_matches(a, sid))
        } else {
            self.level_sets
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid))
                .or(self
                    .level_set_recycle_bin
                    .iter()
                    .flat_map(|l| l.areas.iter())
                    .find(|a| area_sid_matches(a, sid)))
        }
    }

    /// Returns a mutable reference to an [AreaStats] given the area's sid
    ///
    /// Returns `None` if no area with the given sid is found.
    pub fn find_area_by_sid_mut<'a>(&'a mut self, sid: &str) -> Option<&'a mut AreaStats> {
        if sid.starts_with("Celeste") {
            self.areas.iter_mut().find(|a| area_sid_matches(a, sid))
        } else {
            self.level_sets
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid))
                .or(self
                    .level_set_recycle_bin
                    .iter_mut()
                    .flat_map(|l| l.areas.iter_mut())
                    .find(|a| area_sid_matches(a, sid)))
        }
    }

    /// Returns a reference to an [AreaStats] given the area's sid and [AreaSource]
    ///
    /// Assuming the [AreaSource] is accurate, this is more optimized than [find_area_by_sid](Self::find_area_by_sid) when searching for modded maps.<br>
    /// There should be very little or no performance gain when searching for vanilla maps.
    ///
    /// Returns `None` if no area with the given sid is found in the list indicated by the source.
    pub fn find_area_by_sid_and_source<'a>(
        &'a self,
        sid: &str,
        source: AreaSource,
    ) -> Option<&'a AreaStats> {
        match source {
            AreaSource::Vanilla => self.areas.areas.iter().find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSets => self
                .level_sets
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSetRecycleBin => self
                .level_set_recycle_bin
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid)),
        }
    }

    /// Returns a mutable reference to an [AreaStats] given the area's sid and [AreaSource]
    ///
    /// Assuming the [AreaSource] is accurate, this is more optimized than [find_area_by_sid_mut](Self::find_area_by_sid_mut) when searching for modded maps.<br>
    /// There should be very little or no performance gain when searching for vanilla maps.
    ///
    /// Returns `None` if no area with the given sid is found in the list indicated by the source.
    pub fn find_area_by_sid_and_source_mut<'a>(
        &'a mut self,
        sid: &str,
        source: AreaSource,
    ) -> Option<&'a mut AreaStats> {
        match source {
            AreaSource::Vanilla => self.areas.iter_mut().find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSets => self
                .level_sets
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSetRecycleBin => self
                .level_set_recycle_bin
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid)),
        }
    }

    /// Load ins a [ModSaveData], adding any entries that are not in our
    /// [level_sets](SaveData::level_sets) or [level_set_recycle_bin](SaveData::level_set_recycle_bin) and ignoring any that are already there
    pub fn load_in_mod_save_data(&mut self, mod_save_data: &ModSaveData) {
        // Create HashSets of all the level sets we currently have
        // Should make checking if we have a set faster than iterating over the Vecs
        let set_names = self
            .all_level_sets()
            .iter()
            .map(|(s, _)| &s.name)
            .cloned()
            .collect::<HashSet<_>>();

        for set in mod_save_data.level_sets.iter() {
            if !set_names.contains(&set.name) {
                self.level_sets.push(set.clone());
            }
        }

        for set in mod_save_data.level_set_recycle_bin.iter() {
            if !set_names.contains(&set.name) {
                self.level_set_recycle_bin.push(set.clone());
            }
        }

        if self.last_area_safe.is_none() {
            self.last_area_safe
                .clone_from(&mod_save_data.last_area_safe);
        }
    }

    /// Merges the applicable data from another [SaveData] into this one
    ///
    /// #### Unmerged Data
    /// These are the fields that are currently not merged that might be in a future version
    /// - [SaveData::total_golden_strawberries]
    /// - [SaveData::flags]
    /// - [SaveData::summit_gems]
    pub fn merge_data(&mut self, other: &SaveData) {
        // Merge the basic stats
        self.time += other.time;
        self.total_deaths += other.total_deaths;
        self.total_dashes += other.total_dashes;
        self.total_jumps += other.total_jumps;
        self.total_wall_jumps += other.total_wall_jumps;
        self.revealed_farewell |= other.revealed_farewell;

        self.cheat_mode |= other.cheat_mode;
        self.assist_mode |= other.assist_mode;
        self.variant_mode |= other.variant_mode;

        // This even being a thing could be a negative for some people.
        // If more than one person asks us about it we cut it.
        self.assists.badeline |= other.assists.badeline;
        self.assists.dash_assist |= other.assists.dash_assist;
        self.assists.full_dashing |= other.assists.full_dashing;
        self.assists.hiccups |= other.assists.hiccups;
        self.assists.infinite_stamina |= other.assists.infinite_stamina;
        self.assists.invincible |= other.assists.invincible;
        self.assists.invisible_motion |= other.assists.invisible_motion;
        self.assists.low_friction |= other.assists.low_friction;
        self.assists.mirror_mode |= other.assists.mirror_mode;
        self.assists.no_grabbing |= other.assists.no_grabbing;
        self.assists.super_dash |= other.assists.super_dash;

        self.unlocked_areas = AreaCount::max(self.unlocked_areas, other.unlocked_areas);

        // Get the areas
        let areas_to_copy = other.all_areas();
        let curr_areas = self.all_areas_mut();

        for (self_area, _source) in curr_areas {
            if let Some((other_area, _source)) = areas_to_copy
                .iter()
                .find(|(a, _)| a.def.sid == self_area.def.sid)
            {
                for (self_mode, other_mode) in
                    self_area.modes.iter_mut().zip(other_area.modes.iter())
                {
                    for (idx, cp) in other_mode.checkpoints.iter().enumerate() {
                        if !self_mode.checkpoints.contains(cp) {
                            // We assume if the other save has a cp in the same position
                            // that it comes earlier than the cps we have.
                            // there is no real good way to handle conflicts in the middle of the list
                            // but this allows merging [a, b, c] into [b, c] to work properly
                            // while keeping [a, b, c] into [a, b] working properly
                            //
                            // The only map I know of that has cps that you might miss is solaris
                            // but even then the cps come after the original cps.
                            //
                            // Ultimately I have no idea if this is a correct implementation
                            // but also it shouldn't matter cause the amount of times
                            // an issue will occur is likely very small
                            self_mode.checkpoints.insert(idx, cp.clone());
                        }
                    }

                    let mut total_strawberries = HashSet::with_capacity(
                        self_mode
                            .strawberries
                            .len()
                            .max(other_mode.strawberries.len()),
                    );

                    for strawberry in self_mode.strawberries.iter() {
                        total_strawberries.insert(strawberry.clone());
                    }

                    for strawberry in other_mode.strawberries.iter() {
                        total_strawberries.insert(strawberry.clone());
                    }

                    self_mode.strawberries.strawberries = total_strawberries.into_iter().collect();

                    let self_stats = &mut self_mode.stats;
                    let other_stats = &other_mode.stats;

                    self_stats.completed |= other_stats.completed;
                    self_stats.full_clear |= other_stats.full_clear;
                    self_stats.heart_gem |= other_stats.heart_gem;
                    self_stats.single_run_completed |= other_stats.single_run_completed;

                    self_stats.best_dashes =
                        DashCount::min(self_stats.best_dashes, other_stats.best_dashes);
                    self_stats.best_deaths =
                        DeathCount::min(self_stats.best_deaths, other_stats.best_deaths);

                    self_stats.best_full_clear_time =
                        if self_stats.best_full_clear_time < other_stats.best_full_clear_time {
                            self_stats.best_full_clear_time
                        } else {
                            other_stats.best_full_clear_time
                        };

                    self_stats.best_time = if self_stats.best_time < other_stats.best_time {
                        self_stats.best_time
                    } else {
                        other_stats.best_time
                    };

                    self_stats.deaths += other_stats.deaths;
                    self_stats.time_played += other_stats.time_played;

                    self_stats.total_strawberries = self_mode.strawberries.len() as StrawberryCount;
                }
            }
        }

        for (set, _) in self.all_level_sets_mut() {
            set.total_strawberries = set
                .areas
                .iter()
                .flat_map(|a| a.modes.iter())
                .map(|m| m.strawberries.len())
                .sum::<usize>() as StrawberryCount;

            if let Some((other_set, _)) = other
                .all_level_sets()
                .iter()
                .find(|(s, _)| s.name == set.name)
            {
                set.unlocked_areas = AreaCount::max(set.unlocked_areas, other_set.unlocked_areas);
            }
        }

        self.total_strawberries = self
            .areas
            .iter()
            .flat_map(|a| a.modes.iter())
            .map(|m| m.strawberries.len())
            .sum::<usize>() as StrawberryCount;

        // This does not actually merge the counts properly BUT it does
        // merge at least some of the data when `other` has more goldens than us
        //
        // Doing this properly would require some indication of which strawberries are goldens in the save
        if self.total_golden_strawberries < other.total_golden_strawberries {
            self.total_golden_strawberries +=
                other.total_golden_strawberries - self.total_golden_strawberries;
        }
    }
}

// Manual default because we need the urls, version, and names to be specific things
impl Default for SaveData {
    fn default() -> Self {
        Self {
            xsi_url: XSI_URL.to_owned(),
            xsd_url: XSD_URL.to_owned(),
            version: "1.4.0.0".to_owned(),
            name: "Madeline".to_owned(),
            time: FileTime(0),
            last_save: NaiveDateTime::default(),
            cheat_mode: false,
            assist_mode: false,
            variant_mode: false,
            assists: Default::default(),
            theo_sister_name: "Alex".to_owned(),
            unlocked_areas: 0,
            total_deaths: 0,
            total_strawberries: 0,
            total_golden_strawberries: 0,
            total_jumps: 0,
            total_wall_jumps: 0,
            total_dashes: 0,
            flags: Default::default(),
            poem: Default::default(),
            summit_gems: None,
            revealed_farewell: false,
            last_area: Default::default(),
            current_session: None,
            areas: Default::default(),
            level_sets: Default::default(),
            level_set_recycle_bin: Default::default(),
            has_modded_save_data: false,
            last_area_safe: None,
            current_session_safe: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AreaSource {
    Vanilla,
    LevelSets,
    LevelSetRecycleBin,
}

impl ModSaveData {
    pub fn from_reader(reader: impl BufRead) -> Result<Self, DeError> {
        quick_xml::de::from_reader(reader)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<Self, DeError> {
        quick_xml::de::from_str(str)
    }

    pub fn to_string(&self) -> Result<String, DeError> {
        let mut xml = XML_VERSION_HEADER.to_owned();
        xml.push_str(&quick_xml::se::to_string(&self)?);
        Ok(xml)
    }

    pub fn to_writer(&self, mut writer: impl Write) -> Result<(), DeError> {
        writer.write_str(XML_VERSION_HEADER)?;
        quick_xml::se::to_writer(writer, &self)
    }

    /// Combines all the [LevelSetStats] into one vec,
    /// with a boolean indicating whether they are in the recycle bin or not
    pub fn all_level_sets(&self) -> Vec<(&LevelSetStats, bool)> {
        self.level_sets
            .iter()
            .map(|a| (a, false))
            .chain(self.level_set_recycle_bin.iter().map(|a| (a, true)))
            .collect()
    }

    /// Combines all the [LevelSetStats] into one vec,
    /// with a boolean indicating whether they are in the recycle bin or not
    pub fn all_level_sets_mut(&mut self) -> Vec<(&mut LevelSetStats, bool)> {
        self.level_sets
            .iter_mut()
            .map(|a| (a, false))
            .chain(self.level_set_recycle_bin.iter_mut().map(|a| (a, true)))
            .collect()
    }

    /// Combines all the areas into one list, adding a tag that denotes what group the area is in
    pub fn all_areas(&self) -> Vec<(&AreaStats, AreaSource)> {
        self.level_sets
            .iter()
            .flat_map(|a| a.areas.iter().map(|a| (a, AreaSource::LevelSets)))
            .chain(
                self.level_set_recycle_bin
                    .iter()
                    .flat_map(|a| a.areas.iter().map(|a| (a, AreaSource::LevelSetRecycleBin))),
            )
            .collect()
    }

    /// Combines all the areas into one list, adding a tag that denotes what group the area is in
    pub fn all_areas_mut(&mut self) -> Vec<(&mut AreaStats, AreaSource)> {
        self.level_sets
            .iter_mut()
            .flat_map(|a| a.areas.iter_mut().map(|a| (a, AreaSource::LevelSets)))
            .chain(self.level_set_recycle_bin.iter_mut().flat_map(|a| {
                a.areas
                    .iter_mut()
                    .map(|a| (a, AreaSource::LevelSetRecycleBin))
            }))
            .collect()
    }

    /// Returns a reference to an [AreaStats] given the area's sid
    ///
    /// Returns `None` if no area with the given sid is found.
    pub fn find_area_by_sid<'a>(&'a self, sid: &str) -> Option<&'a AreaStats> {
        if sid.starts_with("Celeste") {
            None
        } else {
            self.level_sets
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid))
                .or(self
                    .level_set_recycle_bin
                    .iter()
                    .flat_map(|l| l.areas.iter())
                    .find(|a| area_sid_matches(a, sid)))
        }
    }

    /// Returns a mutable reference to an [AreaStats] given the area's sid
    ///
    /// Returns `None` if no area with the given sid is found.
    pub fn find_area_by_sid_mut<'a>(&'a mut self, sid: &str) -> Option<&'a mut AreaStats> {
        if sid.starts_with("Celeste") {
            None
        } else {
            self.level_sets
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid))
                .or(self
                    .level_set_recycle_bin
                    .iter_mut()
                    .flat_map(|l| l.areas.iter_mut())
                    .find(|a| area_sid_matches(a, sid)))
        }
    }

    /// Returns a reference to an [AreaStats] given the area's sid and [AreaSource]
    ///
    /// Assuming the [AreaSource] is accurate, this is more optimized than [find_area_by_sid](Self::find_area_by_sid) when searching for modded maps.<br>
    /// There should be very little or no performance gain when searching for vanilla maps.
    ///
    /// Returns `None` if no area with the given sid is found in the list indicated by the source.
    pub fn find_area_by_sid_and_source<'a>(
        &'a self,
        sid: &str,
        source: AreaSource,
    ) -> Option<&'a AreaStats> {
        match source {
            AreaSource::Vanilla => None,
            AreaSource::LevelSets => self
                .level_sets
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSetRecycleBin => self
                .level_set_recycle_bin
                .iter()
                .flat_map(|l| l.areas.iter())
                .find(|a| area_sid_matches(a, sid)),
        }
    }

    /// Returns a mutable reference to an [AreaStats] given the area's sid and [AreaSource]
    ///
    /// Assuming the [AreaSource] is accurate, this is more optimized than [find_area_by_sid_mut](Self::find_area_by_sid_mut) when searching for modded maps.<br>
    /// There should be very little or no performance gain when searching for vanilla maps.
    ///
    /// Returns `None` if no area with the given sid is found in the list indicated by the source.
    pub fn find_area_by_sid_and_source_mut<'a>(
        &'a mut self,
        sid: &str,
        source: AreaSource,
    ) -> Option<&'a mut AreaStats> {
        match source {
            AreaSource::Vanilla => None,
            AreaSource::LevelSets => self
                .level_sets
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid)),
            AreaSource::LevelSetRecycleBin => self
                .level_set_recycle_bin
                .iter_mut()
                .flat_map(|l| l.areas.iter_mut())
                .find(|a| area_sid_matches(a, sid)),
        }
    }
}

impl Default for ModSaveData {
    fn default() -> Self {
        Self {
            xsi_url: XSI_URL.to_owned(),
            xsd_url: XSD_URL.to_owned(),
            level_sets: Default::default(),
            level_set_recycle_bin: Default::default(),
            last_area_safe: None,
        }
    }
}

impl Display for DashMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DashMode::Normal => "Normal".to_owned(),
            DashMode::Two => "Two".to_owned(),
            DashMode::Infinite => "Infinite".to_owned(),
        })
    }
}

impl From<DashMode> for String {
    fn from(value: DashMode) -> Self {
        value.to_string()
    }
}

impl Deref for Flags {
    type Target = Vec<VanillaFlagsWrapper>;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}

impl DerefMut for Flags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

impl Deref for VanillaFlagsWrapper {
    type Target = VanillaFlags;

    fn deref(&self) -> &Self::Target {
        &self.flag
    }
}

impl DerefMut for VanillaFlagsWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flag
    }
}

impl From<VanillaFlags> for VanillaFlagsWrapper {
    fn from(val: VanillaFlags) -> Self {
        VanillaFlagsWrapper { flag: val }
    }
}

impl Deref for Poem {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl DerefMut for Poem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.string
    }
}

impl Default for Assists {
    fn default() -> Self {
        Self {
            game_speed: 10,
            invincible: false,
            dash_mode: Default::default(),
            dash_assist: false,
            infinite_stamina: false,
            mirror_mode: false,
            full_dashing: false,
            invisible_motion: false,
            no_grabbing: false,
            low_friction: false,
            super_dash: false,
            hiccups: false,
            badeline: false,
        }
    }
}
