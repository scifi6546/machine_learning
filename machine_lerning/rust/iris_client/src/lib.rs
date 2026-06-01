use chrono::{DateTime, SecondsFormat, Utc};
use http::{Uri, uri::InvalidUri};
use miniseed_rs_2::{Stream, StreamReaderError};
use prelude::rusqlite;
use prelude::rusqlite::OptionalExtension;
use std::io::Cursor;
use thiserror::Error;
const WAVEFORM_CACHE_PATH: &'static str = "./waveforms.db";
#[derive(Debug, Error)]
pub enum WaveformClientError {
    #[error("Failed to build valid URI: {0}")]
    Uri(InvalidUri),
    #[error("Failed to run http request: {0}")]
    HttpError(reqwest::Error),
    #[error("Failed to read stream: {0}")]
    StreamReadError(StreamReaderError),
    #[error("cache database failed: {0}")]
    SqliteError(rusqlite::Error),
}
impl From<rusqlite::Error> for WaveformClientError {
    fn from(value: rusqlite::Error) -> Self {
        Self::SqliteError(value)
    }
}
impl From<InvalidUri> for WaveformClientError {
    fn from(value: InvalidUri) -> Self {
        Self::Uri(value)
    }
}
impl From<reqwest::Error> for WaveformClientError {
    fn from(value: reqwest::Error) -> Self {
        Self::HttpError(value)
    }
}
impl From<StreamReaderError> for WaveformClientError {
    fn from(value: StreamReaderError) -> Self {
        Self::StreamReadError(value)
    }
}
/// Client that grabs data from IRIS and caches it in sqlite file
pub struct WaveformClient {
    waveform_cache_connection: rusqlite::Connection,
}
impl WaveformClient {
    pub fn new() -> Result<Self, WaveformClientError> {
        let waveform_cache_connection = rusqlite::Connection::open(WAVEFORM_CACHE_PATH)?;
        let schema = "create TABLE waveforms(fetch_url TEXT NOT NULL, waveform_data BLOB)";
        waveform_cache_connection.execute(schema, ());
        Ok(Self {
            waveform_cache_connection,
        })
    }
    ///fetches waveform, first tries cache if data is not in cache loads from IRIS
    pub fn fetch(&mut self, info: &WaveformFetchInfo) -> Result<Stream, WaveformClientError> {
        let url = format!(
            "https://service.earthscope.org/fdsnws/dataselect/1/query?net={}&sta={}&cha={}&start={}&end={}&format=miniseed",
            info.network,
            info.station,
            info.channel_select,
            info.start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            info.end_time.to_rfc3339_opts(SecondsFormat::Millis, true)
        );
        let check_sql = "SELECT waveform_data FROM waveforms WHERE fetch_url = ?1";
        let mut statement = self.waveform_cache_connection.prepare(check_sql)?;

        let raw_data_option = statement
            .query_one((&url,), |a| {
                let count: Vec<u8> = a.get(0).unwrap();
                Ok(count)
            })
            .optional()?;
        let raw_data = if let Some(data) = raw_data_option {
            data
        } else {
            let response = reqwest::blocking::get(&url)?;
            response.status();

            let response_bytes = response.bytes()?;

            let mut insert_statement = self
                .waveform_cache_connection
                .prepare("INSERT INTO waveforms(waveform_data, fetch_url) VALUES (?1, ?2)")?;
            insert_statement.execute((response_bytes.as_ref(), &url))?;
            response_bytes.to_vec()
        };

        Ok(Stream::from_reader(Cursor::new(raw_data))?)
    }
}

#[derive(Clone, Debug)]
pub struct WaveformFetchInfo {
    pub network: String,
    pub station: String,
    pub channel_select: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use prelude::{TestAssertionOptions, expect};

    #[test]
    fn basic_download() {
        let mut client = WaveformClient::new().unwrap();
        let start_time = "2010-02-27T06:30:00.000Z".parse().expect("failed to parse");
        let end_time = "2010-02-27T10:30:00.000Z"
            .parse()
            .expect("failed to parse date");
        let stream = client
            .fetch(&WaveformFetchInfo {
                network: "IU".to_string(),
                station: "ANMO".to_string(),
                channel_select: "BHZ".to_string(),
                start_time,
                end_time,
            })
            .expect("failed to download");
        let options = TestAssertionOptions::default().max_difference(TimeDelta::milliseconds(40));
        expect(stream.traces()[0].start_time())
            .with_options(options)
            .to_be_close_to(start_time);
        expect(stream.traces()[0].end_time())
            .with_options(options)
            .to_be_close_to(end_time);
    }
}
