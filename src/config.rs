use crate::store;
use std::path::Path;

pub trait TimeSource {
    fn now(&self) -> std::time::SystemTime;
}

// Need to wrap std::time::SystemTime because the std implementation has private
// fields that prevent intantiation here.
pub struct SystemTime;
impl TimeSource for SystemTime {
    fn now(&self) -> std::time::SystemTime {
        return std::time::SystemTime::now();
    }
}

pub struct Config<'a, T: TimeSource> {
    // This is where we set the TYPE of timesource
    pub time_source: &'a T,
    pub store: store::Store,
}

impl<'a, T: TimeSource> Config<'a, T> {
    pub fn new(time_source: &'a T, store_path: &'a Path) -> Result<Self, anyhow::Error> {
        // This is where we set the VALUE of timesource
        let s = store::Store::new(store_path)?;

        Ok(Config {
            time_source,
            store: s,
        })
    }
}
