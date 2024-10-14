use crate::store;

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

pub struct Config<'a, T: TimeSource, S: store::Store> {
    // This is where we set the TYPE of timesource
    pub time_source: &'a T,
    pub store: &'a mut S,
}

impl<'a, T: TimeSource, S: store::Store> Config<'a, T, S> {
    pub fn new(time_source: &'a T, s: &'a mut S) -> Result<Self, anyhow::Error> {
        Ok(Config {
            time_source,
            store: s,
        })
    }
}
