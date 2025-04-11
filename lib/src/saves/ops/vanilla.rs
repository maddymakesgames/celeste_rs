use std::ops::{Deref, DerefMut};

use crate::saves::{def::vanilla::*, util::FileTime};

const VANILLA_SIDS: [&str; 11] = [
    "Celeste/0-Intro",
    "Celeste/1-ForsakenCity",
    "Celeste/2-OldSite",
    "Celeste/3-CelestialResort",
    "Celeste/4-GoldenRidge",
    "Celeste/5-MirrorTemple",
    "Celeste/6-Reflection",
    "Celeste/7-Summit",
    "Celeste/8-Epilogue",
    "Celeste/9-Core",
    "Celeste/LostLevels",
];

impl AreaDef {
    /// Gets a reference to an sid for the level
    pub fn sid(&self) -> &str {
        // We just use vanilla level sids since sid is only None in vanilla saves
        // Or manually edited saves
        self.sid
            .as_deref()
            .unwrap_or_else(|| VANILLA_SIDS[self.id as usize])
    }

    /// Creates a new AreaDef for a given sid
    ///
    /// `vanilla` determines whether or not the `sid` field is present.
    ///
    /// Note that this only works for vanilla SIDs since there does not seem to be a consistent way to
    /// generate the `id` field for modded areas.
    pub fn for_sid(sid: &str, vanilla: bool) -> Option<AreaDef> {
        if VANILLA_SIDS.contains(&sid) {
            Some(AreaDef {
                id: VANILLA_SIDS.iter().position(|s| *s == sid).unwrap() as u16,
                cassette: false,
                sid: if !vanilla { Some(sid.to_owned()) } else { None },
            })
        } else {
            None
        }
    }
}

impl AreaRef {
    /// Gets a reference to an sid for the level
    pub fn sid(&self) -> &str {
        // We just use vanilla level sids since sid is only None in vanilla saves
        // Or manually edited saves
        self.sid
            .as_deref()
            .unwrap_or_else(|| VANILLA_SIDS[self.id as usize])
    }

    /// Creates an AreaRef for a given sid, mode, and whether to format it for a vanilla session or not
    ///
    /// `vanilla` determines whether or not the `sid` field is present.
    ///
    /// Note that this only works for vanilla SIDs since there does not seem to be a consistent way to
    /// generate the `id` field for modded areas.
    pub fn for_sid(sid: &str, mode: AreaModeType, vanilla: bool) -> Option<AreaRef> {
        if VANILLA_SIDS.contains(&sid) {
            Some(AreaRef {
                id: VANILLA_SIDS.iter().position(|s| *s == sid).unwrap() as u16,
                mode,
                sid: if !vanilla { Some(sid.to_owned()) } else { None },
            })
        } else {
            None
        }
    }
}

// We say the default is just providing a reference to 0-Intro
impl Default for AreaRef {
    fn default() -> Self {
        // Unwrap is safe since we know the sid is in the array
        AreaRef::for_sid(VANILLA_SIDS[0], AreaModeType::Normal, true).unwrap()
    }
}

impl Deref for Areas {
    type Target = Vec<AreaStats>;

    fn deref(&self) -> &Self::Target {
        &self.areas
    }
}

impl DerefMut for Areas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.areas
    }
}

// Default areas is just a list of all vanilla areas
// arguably it should be an empty list since level sets should have it be empty but
// whatever.
impl Default for Areas {
    fn default() -> Self {
        let mut vanilla_areas = Vec::with_capacity(VANILLA_SIDS.len());

        for sid in VANILLA_SIDS {
            // unwrap is safe because we're pulling from VANILLA_SIDS
            vanilla_areas.push(AreaStats::for_sid(sid, true).unwrap())
        }

        Self {
            areas: vanilla_areas,
        }
    }
}

impl AreaStats {
    /// Creates an AreaStats for a given sid
    ///
    /// `vanilla` is used to determine the format of the inner [AreaDef]
    pub fn for_sid(sid: &str, vanilla: bool) -> Option<AreaStats> {
        Some(AreaStats {
            def: AreaDef::for_sid(sid, vanilla)?,
            modes: Default::default(),
        })
    }

    /// Creates an AreaStats for a given `AreaDef`
    ///
    /// Is shorthand for making an `AreaStats` with modes being set to the default.
    pub fn for_def(def: AreaDef) -> AreaStats {
        AreaStats {
            def,
            modes: Default::default(),
        }
    }
}

impl Deref for Modes {
    type Target = Vec<AreaMode>;

    fn deref(&self) -> &Self::Target {
        &self.modes
    }
}

impl DerefMut for Modes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modes
    }
}

// Default for modes is just 3 defaulted AreaModes
// this is the AreaMode for intro, hopefully the game handles this properly
impl Default for Modes {
    fn default() -> Self {
        Self {
            modes: vec![AreaMode::default(); 3],
        }
    }
}

// Pretty sensable default here
// I think I could derive default on this but I don't want to put default on FileTime
impl Default for AreaModeStats {
    fn default() -> Self {
        Self {
            total_strawberries: 0,
            completed: false,
            single_run_completed: false,
            full_clear: false,
            deaths: 0,
            time_played: FileTime(0),
            best_time: FileTime(0),
            best_full_clear_time: FileTime(0),
            best_dashes: 0,
            best_deaths: 0,
            heart_gem: false,
        }
    }
}

impl Deref for Checkpoints {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.checkpoints
    }
}

impl DerefMut for Checkpoints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.checkpoints
    }
}
