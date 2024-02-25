use std::{
    ops,
    time::{Duration, SystemTime},
};

use gtk::glib;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, glib::Boxed)]
#[boxed_type(name = "BustleTimestamp")]
pub struct Timestamp(Duration);

impl Timestamp {
    pub fn now() -> Self {
        let dur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        Self(dur)
    }

    pub fn as_millis_f64(self) -> f64 {
        self.0.as_micros() as f64 / 1_000.0
    }
}

impl ops::Sub for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0.sub(rhs.0))
    }
}

impl ops::Div<u32> for Timestamp {
    type Output = Self;

    fn div(self, rhs: u32) -> Self {
        Self(self.0.div(rhs))
    }
}

impl ops::AddAssign for Timestamp {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0);
    }
}

impl From<Duration> for Timestamp {
    fn from(dur: Duration) -> Self {
        Self(dur)
    }
}

impl From<Timestamp> for Duration {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}
