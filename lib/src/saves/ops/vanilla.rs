use std::ops::{Deref, DerefMut};

use crate::saves::def::vanilla::*;


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
