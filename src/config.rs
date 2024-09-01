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
}

impl<'a, T: TimeSource> Config<'a, T> {
    pub fn new(time_source: &'a T) -> Self {
        // This is where we set the VALUE of timesource
        Config { time_source }
    }
}
