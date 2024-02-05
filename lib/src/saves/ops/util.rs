use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

use crate::saves::def::util::*;


impl FileTime {
    pub fn as_duration(&self) -> Duration {
        // We won't get any more precise than milliseconds because
        // ...
        // why
        Duration::from_millis(self.as_millis())
    }

    pub fn as_millis(&self) -> u64 {
        self.0 / 10000
    }

    pub fn from_duration(dur: Duration) -> Self {
        // Why does this not return u64??
        // The from_millis takes u64, but then returns u128???
        Self::from_millis(dur.as_millis() as u64)
    }

    pub fn from_millis(millis: u64) -> Self {
        FileTime(millis * 10000)
    }

    pub fn from_parts(hours: u64, minutes: u64, seconds: u64, millis: u64) -> Self {
        Self::from_millis(Self::parts_to_millis(hours, minutes, seconds, millis))
    }

    pub fn add_duration(&mut self, dur: Duration) {
        self.add_millis(dur.as_millis() as u64)
    }

    pub fn add_millis(&mut self, millis: u64) {
        self.0 += millis * 10000
    }

    pub fn add_parts(&mut self, hours: u64, minutes: u64, seconds: u64, millis: u64) {
        self.add_millis(Self::parts_to_millis(hours, minutes, seconds, millis))
    }

    fn parts_to_millis(hours: u64, minutes: u64, seconds: u64, millis: u64) -> u64 {
        ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis
    }
}

impl Add for FileTime {
    type Output = FileTime;

    fn add(self, rhs: Self) -> Self::Output {
        FileTime(self.0 + rhs.0)
    }
}

impl Sub for FileTime {
    type Output = FileTime;

    fn sub(self, rhs: Self) -> Self::Output {
        FileTime(self.0 - rhs.0)
    }
}

impl AddAssign for FileTime {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for FileTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl Add for &FileTime {
    type Output = FileTime;

    fn add(self, rhs: Self) -> Self::Output {
        FileTime(self.0 + rhs.0)
    }
}

impl Sub for &FileTime {
    type Output = FileTime;

    fn sub(self, rhs: Self) -> Self::Output {
        FileTime(self.0 - rhs.0)
    }
}

impl Display for FileTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.as_millis();
        let hours = duration / 3600000;
        let mins = (duration / 60000) % 60;
        let secs = (duration / 1000) % 60;
        let millis = duration % 1000;
        write!(f, "{hours:02}:{mins:02}:{secs:02}:{millis:03}")
    }
}
