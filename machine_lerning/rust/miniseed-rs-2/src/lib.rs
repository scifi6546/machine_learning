use chrono::{DateTime, Duration, Utc};
use std::{io::Read, mem::size_of};
pub mod c_lib;
use c_lib::{
    constants::NULL,
    functions::{mstl3_init, mstl3_readbuffer},
    structs::{ControlFlags, MS3Tolerance, MS3TraceID, MS3TraceList},
};
use prelude::thiserror::Error;
#[derive(Error, Debug)]
pub enum StreamReaderError {
    #[error("Miniseed error")]
    MiniseedError,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No Data in stream")]
    NoData,
}
#[derive(Debug)]
pub struct Stream {
    traces: Vec<Trace>,
}
impl Stream {
    pub fn from_reader(mut reader: impl Read) -> Result<Self, StreamReaderError> {
        let mut reader_data = Vec::new();
        reader.read_to_end(&mut reader_data)?;
        unsafe {
            let mut trace_list = mstl3_init(NULL as *mut MS3TraceList);
            let status = mstl3_readbuffer(
                &mut trace_list,
                reader_data.as_ptr() as *const i8,
                reader_data.len() as u64,
                0,
                ControlFlags::UnpackData as u32 | ControlFlags::RecordList as u32,
                NULL as *const MS3Tolerance,
                0,
            );

            if status < 0 {
                return Err(StreamReaderError::MiniseedError);
            }
            if trace_list.read().traces.next[0] == NULL as *mut MS3TraceID {
                return Ok(Self { traces: Vec::new() });
            }
            let mut current_trace = trace_list.read().traces.next[0];
            let mut traces = Vec::new();
            loop {
                if current_trace.read().numsegments != 1 {
                    todo!("more then one segment")
                }
                if current_trace.read().numsegments >= 1 {
                    let start_time = current_trace.read().first.read().starttime;
                    let sample_rate = current_trace.read().first.read().samprate;
                    let sample_type: SampleType =
                        current_trace.read().first.read().sampletype.into();
                    // for now only int32 are supported
                    assert_eq!(sample_type, SampleType::I32);

                    let data_size = current_trace.read().first.read().numsamples as usize;
                    let mut data: Vec<i32> = Vec::with_capacity(data_size);
                    for i in 0..data_size {
                        let p = current_trace.read().first.read().datasamples as *const i32;
                        data.push(p.offset(i as isize).read())
                    }

                    let sid_cstr = current_trace.read().sid;
                    let sid_data = sid_cstr.iter().map(|v| *v as u8).collect::<Vec<_>>();
                    let sid = String::from_utf8(sid_data).expect("failed to parse string");

                    let stats = TraceStats::from_sid(&sid, sample_rate);

                    traces.push(Trace {
                        start_time: DateTime::from_timestamp_nanos(start_time),

                        stats,
                        data,
                    });
                }

                if current_trace.read().next[0] != NULL as *mut MS3TraceID {
                    current_trace = current_trace.read().next[0];
                } else {
                    break;
                }
            }
            Ok(Self { traces })
        }
    }
    pub fn traces(&self) -> &[Trace] {
        &self.traces
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SampleType {
    TEXT,
    I32,
    F32,
    F64,
}
impl From<i8> for SampleType {
    fn from(value: i8) -> Self {
        const TEXT_VALUE: i8 = 't' as i8;
        const INT32_VAL: i8 = 'i' as i8;
        const FLOAT32_VALUE: i8 = 'f' as i8;
        const FLOAT64_VALUE: i8 = 'd' as i8;
        match value {
            TEXT_VALUE => Self::TEXT,
            INT32_VAL => Self::I32,
            FLOAT32_VALUE => Self::F32,
            FLOAT64_VALUE => Self::F64,
            _ => panic!("unsupported value: {}", value),
        }
    }
}
#[derive(Clone, Debug)]
pub struct TraceStats {
    pub network: String,
    pub station: String,
    pub channel: String,
    pub sample_rate: f64,
}
impl TraceStats {
    pub fn from_sid(s: &str, sample_rate: f64) -> Self {
        let after = s.split(":").skip(1).next().unwrap();

        let mut splits = after.split("_");
        let network = splits.next().unwrap().to_string();
        let station = splits.next().unwrap().to_string();
        let channel = splits
            .skip(1)
            .map(|v| v.trim_matches('\0'))
            .collect::<String>();

        Self {
            network,
            station,
            channel,
            sample_rate,
        }
    }
}
#[derive(Debug)]
pub struct Trace {
    start_time: DateTime<Utc>,
    data: Vec<i32>,
    stats: TraceStats,
}
impl Trace {
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }
    pub fn end_time(&self) -> DateTime<Utc> {
        if self.number_points() > 0 {
            let time = (self.number_points() - 1) as f64 / self.sampling_rate();
            let time = Duration::microseconds((time * 1_000_000.) as i64);
            self.start_time() + time
        } else {
            self.start_time()
        }
    }
    pub fn network(&self) -> &str {
        &self.stats.network
    }
    pub fn station(&self) -> &str {
        &self.stats.station
    }
    pub fn sampling_rate(&self) -> f64 {
        self.stats.sample_rate
    }
    pub fn number_points(&self) -> usize {
        self.data.len()
    }
    pub fn data(&self) -> &[i32] {
        &self.data
    }
    pub fn channel(&self) -> &str {
        &self.stats.channel
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use prelude::expect;
    use serde::Deserialize;
    use std::{io::Cursor, str::FromStr};
    #[derive(Deserialize)]
    struct MetaData {
        traces: Vec<MetadataTrace>,
    }
    #[derive(Deserialize)]
    struct MetadataTrace {
        start_time: String,
        end_time: String,
        network: String,
        station: String,
        sampling_rate: f64,
        num_points: usize,
        channel: String,
    }
    #[test]
    fn test_parse_station() {
        let info = TraceStats::from_sid("FDSN:AK_HDA__B_H_Z", 0.);
        assert_eq!(info.network, "AK");
        assert_eq!(info.station, "HDA");
        assert_eq!(info.channel, "BHZ");
    }
    #[test]
    fn expect_error() {
        // I hope this isnt a valid miniseed file
        let bad_buffer = Cursor::new(include_bytes!("./lib.rs"));
        let e = Stream::from_reader(bad_buffer).expect_err("should error");
        match e {
            StreamReaderError::MiniseedError => {}
            _ => panic!("invalid error: {}", e),
        }
    }
    fn load_from_buffer(metadata_json: &str, miniseed_bytes: &[u8], real_data_bytes: &[u8]) {
        let metadata: MetaData = serde_json::from_str(metadata_json).expect("failed to parse");
        let stream = Stream::from_reader(Cursor::new(miniseed_bytes))
            .expect("failed to read miniseed bytes");
        assert!(stream.traces().len() == 1);
        for (rs_tr, meta_tr) in stream.traces().iter().zip(metadata.traces.iter()) {
            let meta_tr_start =
                DateTime::<Utc>::from_str(&meta_tr.start_time).expect("failed to crete");

            let meta_tr_end =
                DateTime::<Utc>::from_str(&meta_tr.end_time).expect("failed to crete");
            expect(rs_tr.start_time())
                .with_label("start time")
                .to_be_close_to(meta_tr_start);
            println!("{}", rs_tr.end_time() - rs_tr.start_time());
            println!("expected duration: {}", meta_tr_end - meta_tr_start);
            println!(
                "num points: {}, sample rate: {}, elapsed seconds: {}",
                rs_tr.number_points(),
                rs_tr.sampling_rate(),
                rs_tr.number_points() as f64 / rs_tr.sampling_rate(),
            );
            expect(rs_tr.end_time())
                .with_label("end time")
                .to_be_close_to(meta_tr_end);

            let real_data = real_data_bytes
                .chunks(4)
                .map(|bytes| i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .collect::<Vec<_>>();
            assert_eq!(rs_tr.network(), meta_tr.network);
            assert_eq!(rs_tr.station(), meta_tr.station);
            assert_eq!(rs_tr.sampling_rate(), meta_tr.sampling_rate);
            assert_eq!(rs_tr.number_points(), meta_tr.num_points);
            assert_eq!(rs_tr.data(), real_data);
            assert_eq!(rs_tr.channel(), meta_tr.channel);
        }
    }
    #[test]
    fn load_from_short_buffer() {
        load_from_buffer(
            include_str!("../raw_data/metadata.json"),
            include_bytes!("../raw_data/data.mseed"),
            include_bytes!("../raw_data/data_BHZ.bin"),
        );
    }
}
