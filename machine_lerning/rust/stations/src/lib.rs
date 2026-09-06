pub mod fetcher;
mod local_prelude;
pub mod station_xml;
pub use local_prelude::StationError;
mod test;
use prelude::{
    chrono::{DateTime, TimeDelta, Utc},
    units::{Hertz, Latitude, Longitude, Meters},
};
#[derive(Clone, PartialEq, Debug)]
pub struct Network {
    pub code: String,
    pub fetch_date: DateTime<Utc>,
    pub stations: Vec<Station>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Station {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub elevation: Meters,
    pub name: String,
    pub code: String,
    /// when the station was created
    pub start_time: DateTime<Utc>,
    pub channels: Vec<Channel>,
}
impl TryFrom<&station_xml::Station> for Station {
    type Error = StationError;

    fn try_from(station: &station_xml::Station) -> Result<Self, Self::Error> {
        let code = station
            .code
            .clone()
            .ok_or(StationError::MissingStationCode)?;

        Ok(Self {
            latitude: station
                .latitude
                .ok_or(StationError::MissingLatitude {
                    station_code: code.clone(),
                })?
                .into(),
            longitude: station
                .longitude
                .ok_or(StationError::MissingLongitude {
                    station_code: code.clone(),
                })?
                .into(),
            elevation: station
                .elevation
                .ok_or(StationError::MissingElevation {
                    station_code: code.clone(),
                })?
                .into(),
            name: station
                .site_name
                .clone()
                .ok_or(StationError::MissingStationName)?,
            code: code.clone(),
            start_time: station
                .start_date
                .ok_or(StationError::MissingStartTime { station_code: code })?,
            channels: Vec::new(),
        })
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Channel {
    pub code: String,
    pub sample_rate: Hertz,
    pub sensor_name: String,
}
