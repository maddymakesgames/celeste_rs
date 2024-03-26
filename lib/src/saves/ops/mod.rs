//! Implements operations for all of the types found in the [celeste](crate::celeste) module
use std::{
    collections::HashSet,
    fmt::Write,
    io::BufRead,
    ops::{Deref, DerefMut},
};

pub use quick_xml::DeError;

use crate::saves::{
    def::{everest::LevelSetStats, vanilla::AreaStats},
    DashMode,
    Flags,
    Poem,
    SaveData,
    VanillaFlags,
    VanillaFlagsWrapper,
};

pub mod everest;
pub mod session;
pub mod util;
pub mod vanilla;

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
        quick_xml::se::to_string(&self)
    }

    pub fn to_writer(&self, writer: impl Write) -> Result<(), DeError> {
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

    /// Merges the applicable data from another [SaveData] into this one
    ///
    /// #### Unmerged Data
    /// These are the fields that are currently not merged that will be in a future version
    /// - [SaveData::assists]
    /// - [SaveData::total_strawberries]
    /// - [SaveData::total_golden_strawberries]
    /// - [SaveData::unlocked_areas]
    /// - [SaveData::flags]
    /// - [SaveData::poem]
    /// - [SaveData::summit_gems]
    /// - [LevelSetStats::poem]
    /// - [LevelSetStats::unlocked_areas]
    /// - [LevelSetStats::total_strawberries]
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
                        u64::min(self_stats.best_dashes, other_stats.best_dashes);
                    self_stats.best_deaths =
                        u64::min(self_stats.best_deaths, other_stats.best_deaths);
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

                    self_stats.total_strawberries = self_mode.strawberries.len() as u8;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AreaSource {
    Vanilla,
    LevelSets,
    LevelSetRecycleBin,
}

impl ToString for DashMode {
    fn to_string(&self) -> String {
        match self {
            DashMode::Normal => "Normal".to_owned(),
            DashMode::Two => "Two".to_owned(),
            DashMode::Infinite => "Infinite".to_owned(),
        }
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
