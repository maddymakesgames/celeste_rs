use std::ops::{Deref, DerefMut};

use crate::saves::def::everest::*;

impl Deref for LevelSets {
    type Target = Vec<LevelSetStats>;

    fn deref(&self) -> &Self::Target {
        &self.level_set_stats
    }
}

impl DerefMut for LevelSets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.level_set_stats
    }
}
