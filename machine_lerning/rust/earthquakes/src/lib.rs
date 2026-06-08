pub mod connection;
pub mod error;
pub mod query;
use error::EarthquakeDBError;
use prelude::chrono::{DateTime, Utc};

use query::EventQuery;
#[derive(Clone, Debug)]
pub struct Event {
    pub event_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub time: DateTime<Utc>,
    pub magnitude: f32,
    pub magnitude_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
}
