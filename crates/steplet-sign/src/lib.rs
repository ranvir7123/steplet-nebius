//! The sender's tool: check a .steplet file, and sign it with the sender's own ssh-keygen.

pub mod check;
pub mod sign;

use std::time::{SystemTime, UNIX_EPOCH};

use steplet_core::date::Date;

/// Today's date in UTC, from this computer's clock.
pub fn today() -> Date {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    Date::from_unix(secs).expect("this computer's clock is between the years 2000 and 9999")
}
