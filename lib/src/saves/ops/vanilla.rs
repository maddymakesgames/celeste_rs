use std::ops::{Deref, DerefMut};

use crate::saves::def::vanilla::*;

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
    "Celeste/10-Farewell",
];

impl AreaDef {
    /// Gets a reference to an sid for the level
    pub fn sid(&self) -> &str {
        // We just use vanilla level sids since sid is only None in vanilla saves
        // Or manually edited saves
        self.sid
            .as_deref()
            .unwrap_or(VANILLA_SIDS[self.id as usize])
    }
}

impl LastAreaRef {
    /// Gets a reference to an sid for the level
    pub fn sid(&self) -> &str {
        // see the comment for AreaDef::sid
        self.s_id
            .as_deref()
            .unwrap_or(VANILLA_SIDS[self.id as usize])
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
