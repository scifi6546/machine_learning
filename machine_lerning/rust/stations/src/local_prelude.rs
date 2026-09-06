use super::station_xml::NameSyntaxError;
use prelude::chrono::ParseError as TimeParseError;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
/// Maximum number of digits that can be behind the seconds part
pub const MAXIMUM_SECONDS_DECIMAL: u32 = 6;
#[derive(Debug, Error)]
pub enum StationError {
    #[error("Failed to read XML: {0}")]
    XMLError(#[from] quick_xml::Error),
    #[error("failed to parse utf8 text: {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("invalid XML structure")]
    XMLStructureError,
    #[error("Failed to parse time: {0}")]
    TimeParseError(#[from] TimeParseError),
    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Failed to parse float: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error(
        "The number of digits must be less then {MAXIMUM_SECONDS_DECIMAL}, Actual count: {number_digits}"
    )]
    ToManySeconds { number_digits: u32 },
    #[error("The values of name fields need to be surrounded by quotes")]
    NeedNameQuotes,
    #[error("name syntax error: {0}")]
    InvalidNameSyntax(NameSyntaxError),
    #[error("Network missing code")]
    NetworkMissingCode,
    #[error("station missing code")]
    MissingStationCode,
    #[error("station {station_code} is missing a longitude")]
    MissingLongitude { station_code: String },
    #[error("station {station_code} is missing a Latitude")]
    MissingLatitude { station_code: String },
    #[error("station {station_code} is missing its elevation")]
    MissingElevation { station_code: String },
    #[error("time code format invalid")]
    InvalidTimeFormat,
    #[error("Station is missing name")]
    MissingStationName,
    #[error("station {station_code} is missing its start time")]
    MissingStartTime { station_code: String },
}
