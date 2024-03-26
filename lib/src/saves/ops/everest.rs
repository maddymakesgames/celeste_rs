use std::ops::{Deref, DerefMut};

use crate::saves::def::everest::*;

impl LevelSets {
    // We only need this for skip_serializing_if
    // so this isn't pub
    pub(crate) fn is_inner_empty(&self) -> bool {
        self.is_empty()
    }
}

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
