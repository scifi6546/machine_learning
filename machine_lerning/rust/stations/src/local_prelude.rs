use prelude::chrono::ParseError as TimeParseError;
use std::num::ParseIntError;
use thiserror::Error;
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
    #[error(
        "The number of digits must be less then {MAXIMUM_SECONDS_DECIMAL}, Actual count: {number_digits}"
    )]
    ToManySeconds { number_digits: u32 },
}
/// Maximum number of digits that can be behind the seconds part
const MAXIMUM_SECONDS_DECIMAL: u32 = 6;
