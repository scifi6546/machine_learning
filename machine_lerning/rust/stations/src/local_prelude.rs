use super::sub_state::NameSyntaxError;
use prelude::chrono::{DateTime, ParseError as TimeParseError, TimeDelta, TimeZone, Utc};
use std::{io::BufRead, num::ParseIntError};
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
    #[error("The values of name fields need to be surrounded by quotes")]
    NeedNameQuotes,
    #[error("name syntax error: {0}")]
    InvalidNameSyntax(NameSyntaxError),
}
/// Maximum number of digits that can be behind the seconds part
const MAXIMUM_SECONDS_DECIMAL: u32 = 6;
pub fn parse_to_date_time(text: &str) -> Result<DateTime<Utc>, StationError> {
    // format: YYYY-MM-DDTHH:mm:SS
    // Where Y: Year
    // M: Month
    // DD: DAY
    let mut semi_split = text.split("T").take(2);
    let year_month_day_part = semi_split.next().unwrap();
    let hour_minute_second_part = semi_split.next().unwrap();
    let year: i32 = year_month_day_part[0..=3].parse()?;
    let month: u32 = year_month_day_part[5..=6].parse()?;
    let day: u32 = year_month_day_part[8..=9].parse()?;
    let hour: u32 = hour_minute_second_part[0..=1].parse()?;
    let minute: u32 = hour_minute_second_part[3..=4].parse()?;
    let seconds_whole: u32 = hour_minute_second_part[6..=7].parse()?;
    let seconds_fraction_str = &hour_minute_second_part[9..];
    let number_digits = seconds_fraction_str.len();
    if number_digits > MAXIMUM_SECONDS_DECIMAL as usize {
        return Err(StationError::ToManySeconds {
            number_digits: number_digits as u32,
        });
    }
    let microseconds = seconds_fraction_str.parse::<u64>()?
        * 10_u64.pow(MAXIMUM_SECONDS_DECIMAL - number_digits as u32);
    Ok(Utc
        .with_ymd_and_hms(year, month, day, hour, minute, seconds_whole)
        .unwrap()
        + TimeDelta::microseconds(microseconds as i64))
}
#[cfg(test)]
mod test {
    use prelude::chrono::{TimeZone, Utc};

    use super::*;
    #[test]
    fn parse_date() {
        let date_str = "2020-09-23T00:00:00.0000";
        assert_eq!(
            parse_to_date_time(date_str).unwrap(),
            Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap()
        );
    }
}
