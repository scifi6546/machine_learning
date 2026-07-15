pub mod fetcher;
mod local_prelude;
pub mod station_xml;

mod test;
use prelude::chrono::{DateTime, TimeDelta, Utc};
#[derive(Clone, PartialEq, Debug)]
pub struct Network {
    pub code: String,
    pub fetch_date: DateTime<Utc>,
    pub stations: Vec<Station>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Station {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub name: String,
    pub channels: Vec<Channel>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Channel {}
