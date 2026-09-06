mod sub_state;
mod xml_structs;
use crate::local_prelude::{MAXIMUM_SECONDS_DECIMAL, StationError};
use prelude::chrono::{DateTime, TimeDelta, TimeZone, Utc};
pub use sub_state::NameSyntaxError;
pub use xml_structs::{
    Channel, InstrumentSensitivity, Network, Response, Sensor, Station, StationXML, Unit,
};
pub fn parse_to_date_time(text: &str) -> Result<DateTime<Utc>, StationError> {
    // format: YYYY-MM-DDTHH:mm:SS
    // Where Y: Year
    // M: Month
    // DD: DAY
    println!("parsing date_time: {text}");
    let mut semi_split = text.split("T").take(2);
    let year_month_day_part = semi_split.next().unwrap();
    let hour_minute_second_part = semi_split.next().unwrap();
    let year: i32 = year_month_day_part[0..=3].parse()?;
    let month: u32 = year_month_day_part[5..=6].parse()?;
    let day: u32 = year_month_day_part[8..=9].parse()?;
    let hour: u32 = hour_minute_second_part[0..=1].parse()?;
    let minute: u32 = hour_minute_second_part[3..=4].parse()?;

    let seconds_str = &hour_minute_second_part[6..];

    let mut seconds_split = seconds_str.split('.');
    let seconds_whole: u32 = seconds_split
        .next()
        .ok_or(StationError::InvalidTimeFormat)?
        .parse()?;
    let seconds_fraction_str = seconds_split
        .next()
        .ok_or(StationError::InvalidTimeFormat)?;
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
    use super::*;

    use prelude::chrono::{TimeZone, Utc};
    use rstest::rstest;
    #[rstest]
    #[case("2020-09-23T00:00:00.0000",Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap() )]
    #[case("2025-03-23T00:00:00.0000", Utc.with_ymd_and_hms(2025,3,23,0,0,0).unwrap())]
    #[case( "2020-09-23T01:02:03.0000",Utc.with_ymd_and_hms(2020,9,23,1,2,3).unwrap() )]
    fn parse_date(#[case] input_string: &'static str, #[case] output_date: DateTime<Utc>) {
        assert_eq!(parse_to_date_time(input_string).unwrap(), output_date);
    }
}
