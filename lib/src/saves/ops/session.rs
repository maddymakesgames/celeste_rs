use std::ops::{Deref, DerefMut};

use crate::saves::def::{session::*, util::EntityID};

impl Deref for LevelFlags {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.level_flags
    }
}

impl DerefMut for LevelFlags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.level_flags
    }
}

impl Deref for Strawberries {
    type Target = Vec<EntityID>;

    fn deref(&self) -> &Self::Target {
        &self.strawberries
    }
}

impl DerefMut for Strawberries {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.strawberries
    }
}

impl Deref for DoNotLoad {
    type Target = Vec<EntityID>;

    fn deref(&self) -> &Self::Target {
        &self.do_not_load
    }
}

impl DerefMut for DoNotLoad {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.do_not_load
    }
}

impl Deref for Keys {
    type Target = Vec<EntityID>;

    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl DerefMut for Keys {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.keys
    }
}

impl Deref for Counters {
    type Target = Vec<Counter>;

    fn deref(&self) -> &Self::Target {
        &self.counters
    }
}

impl DerefMut for Counters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.counters
    }
}

impl Deref for SummitGems {
    type Target = Vec<bool>;

    fn deref(&self) -> &Self::Target {
        &self.summit_gems
    }
}

impl DerefMut for SummitGems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.summit_gems
    }
}

impl Deref for Parameters {
    type Target = Vec<MusicParam>;

    fn deref(&self) -> &Self::Target {
        &self.parameters
    }
}

impl DerefMut for Parameters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parameters
    }
}
