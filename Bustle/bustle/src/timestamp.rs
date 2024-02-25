// Import necessary modules from the standard library
use std::{
    ops,
    time::{Duration, SystemTime}, // Import Duration and SystemTime structs from the time module
};

// Import the glib module from the gtk crate
use gtk::glib;

// Define a custom boxed type BustleTimestamp
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, glib::Boxed)]
#[boxed_type(name = "BustleTimestamp")]
pub struct Timestamp(Duration); // Define a struct Timestamp that holds a Duration

impl Timestamp {
    // Define a method now() that returns the current timestamp
    pub fn now() -> Self {
        // Get the duration since UNIX EPOCH
        let dur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!"); // Handle error if SystemTime is before UNIX EPOCH
        Self(dur) // Return Timestamp with the obtained duration
    }

    // Define a method as_millis_f64() that returns the timestamp in milliseconds as f64
    pub fn as_millis_f64(self) -> f64 {
        // self.0 ::> Accesses the first field (Duration) of the Timestamp struct. In Rust, fields within a struct are accessed using dot notation with the variable (self in this case) followed by the field index or name (0 in this case because Duration is the first and only field in the struct).
        self.0.as_micros() as f64 / 1_000.0 // Convert duration to microseconds and then to milliseconds as f64
    }
}

// Implement the Subtrait for Timestamp to support subtraction
impl ops::Sub for Timestamp {
    type Output = Self;

    // Define the sub() method to subtract two Timestamps
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.sub(rhs.0)) // Subtract the durations and return the result as a new Timestamp
    }
}

// Implement the Div trait for Timestamp to support division by u32
impl ops::Div<u32> for Timestamp {
    type Output = Self;

    // Define the div() method to divide a Timestamp by a u32
    fn div(self, rhs: u32) -> Self {
        Self(self.0.div(rhs)) // Divide the duration by the given value and return the result as a new Timestamp
    }
}

// Implement the AddAssign trait for Timestamp to support addition assignment
impl ops::AddAssign for Timestamp {
    // Define the add_assign() method to add another Timestamp to the current Timestamp
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0); // Add the duration of the rhs Timestamp to the duration of the current Timestamp
    }
}

// Implement conversion traits between Duration and Timestamp
impl From<Duration> for Timestamp {
    // Define the from() method to convert a Duration to a Timestamp
    fn from(dur: Duration) -> Self {
        Self(dur) // Return a new Timestamp with the given Duration
    }
}

impl From<Timestamp> for Duration {
    // Define the from() method to convert a Timestamp to a Duration
    fn from(ts: Timestamp) -> Self {
        ts.0 // Return the Duration contained within the Timestamp
    }
}
