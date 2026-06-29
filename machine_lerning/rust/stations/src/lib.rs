use prelude::chrono::{DateTime, ParseError as TimeParseError, TimeZone, Utc};
use quick_xml::{Reader as XMLReader, events::Event as XMLEvent};
use std::{
    io::{BufRead, Read},
    num::ParseIntError,
};
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
pub enum EndEvent<T: Clone + Copy + PartialEq> {
    Backtrack,
    Continue(T),
}
#[derive(Clone, PartialEq, Debug)]
pub struct CalibrationUnit {
    pub name: String,
    pub description: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Sensor {
    pub description: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Unit {
    name: String,
    description: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct InstrumentSensitivity {
    value: f64,
    frequency: f64,
    input_unit: Unit,
    output_unit: Unit,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Response {
    pub instrument_sensitivity: InstrumentSensitivity,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Channel {
    code: String,
    location_code: String,
    start_date: DateTime<Utc>,
    latitude: f64,
    longitude: f64,
    elevation: f64,
    depth: f64,
    azimuth: f64,
    dip: f64,
    sample_rate: f64,
    clock_drift: f64,
    calibration_unit: CalibrationUnit,
    sensor: Sensor,
    response: Response,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Station {
    pub code: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub site_name: String,
    pub channels: Vec<Channel>,
    pub creation_date: DateTime<Utc>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Network {
    pub code: String,
    pub start_date: DateTime<Utc>,
    pub restricted_status: String,
    pub stations: Vec<Station>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FDSNStationXML {
    pub source: String,
    pub sender: String,
    pub module: String,
    pub module_uri: String,
    pub creation_date: DateTime<Utc>,
    pub networks: Vec<Network>,
}
impl FDSNStationXML {
    pub fn from_xml<R: Read + BufRead>(r: R) -> Result<Self, StationError> {
        #[derive(Clone, PartialEq, Debug)]
        enum SiteSubState {
            Initial,
            Name,
        }
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum UnitsSubState {
            Initial,
            Name,
            Description,
        }
        impl UnitsSubState {
            pub fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
                match self {
                    UnitsSubState::Initial => match tag_lowercase {
                        "name" => Ok(Self::Name),
                        "description" => Ok(Self::Description),
                        _ => todo!("unit state tag: {}", tag_lowercase),
                    },
                    UnitsSubState::Name => {
                        return Err(StationError::XMLStructureError);
                    }
                    UnitsSubState::Description => {
                        return Err(StationError::XMLStructureError);
                    }
                }
            }
            pub fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
                match self {
                    UnitsSubState::Initial => Ok(EndEvent::Backtrack),
                    UnitsSubState::Name => Ok(EndEvent::Continue(Self::Initial)),
                    UnitsSubState::Description => Ok(EndEvent::Continue(Self::Initial)),
                }
            }
        }
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum SensorSubState {
            Initial,
            Description,
        }
        impl SensorSubState {
            pub fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
                match tag_lowercase {
                    "description" => Ok(Self::Description),
                    _ => panic!("invalid sensor substate tag: {}", tag_lowercase),
                }
            }
            pub fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
                match self {
                    Self::Initial => Ok(EndEvent::Backtrack),
                    Self::Description => Ok(EndEvent::Continue(Self::Initial)),
                }
            }
        }
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum InstrumentSensitivityState {
            Initial,
            Value,
            Frequency,
            InputUnits(UnitsSubState),
            OutputUnits(UnitsSubState),
        }
        impl InstrumentSensitivityState {
            pub fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
                match self {
                    Self::Initial => match tag_lowercase {
                        "value" => Ok(Self::Value),
                        "frequency" => Ok(Self::Frequency),
                        "inputunits" => Ok(Self::InputUnits(UnitsSubState::Initial)),
                        "outputunits" => Ok(Self::InputUnits(UnitsSubState::Initial)),
                        _ => panic!("invalid response sub tag: {}", tag_lowercase),
                    },
                    Self::Value => return Err(StationError::XMLStructureError),
                    Self::Frequency => return Err(StationError::XMLStructureError),
                    Self::InputUnits(state) => {
                        Ok(Self::InputUnits(state.xml_start_event(tag_lowercase)?))
                    }
                    Self::OutputUnits(state) => {
                        Ok(Self::OutputUnits(state.xml_start_event(tag_lowercase)?))
                    }
                }
            }
            pub fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
                match self {
                    Self::Initial => Ok(EndEvent::Backtrack),
                    Self::Value => Ok(EndEvent::Continue(Self::Initial)),
                    Self::Frequency => Ok(EndEvent::Continue(Self::Initial)),
                    Self::InputUnits(state) => match state.xml_end_event()? {
                        EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                        EndEvent::Continue(state) => {
                            Ok(EndEvent::Continue(Self::InputUnits(state)))
                        }
                    },
                    Self::OutputUnits(state) => match state.xml_end_event()? {
                        EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                        EndEvent::Continue(state) => {
                            Ok(EndEvent::Continue(Self::OutputUnits(state)))
                        }
                    },
                }
            }
        }
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum ResponseSubState {
            Initial,
            InstrumentSensitivity(InstrumentSensitivityState),
        }

        impl ResponseSubState {
            pub fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
                match self {
                    Self::Initial => match tag_lowercase {
                        "instrumentsensitivity" => Ok(Self::InstrumentSensitivity(
                            InstrumentSensitivityState::Initial,
                        )),
                        _ => panic!("invalid response sub tag: {}", tag_lowercase),
                    },
                    Self::InstrumentSensitivity(state) => {
                        Ok(ResponseSubState::InstrumentSensitivity(
                            state.xml_start_event(tag_lowercase)?,
                        ))
                    }
                }
            }
            pub fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
                match self {
                    Self::Initial => Ok(EndEvent::Backtrack),
                    Self::InstrumentSensitivity(state) => match state.xml_end_event()? {
                        EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                        EndEvent::Continue(state) => {
                            Ok(EndEvent::Continue(Self::InstrumentSensitivity(state)))
                        }
                    },
                }
            }
        }
        #[derive(Clone, PartialEq, Debug)]
        enum ChannelSubState {
            Initial,
            Latitude,
            Longitude,
            Elevation,
            Depth,
            Azimuth,
            Dip,
            Type,
            SampleRate,
            ClockDrift,
            CalibrationUnits(UnitsSubState),
            Sensor(SensorSubState),
            Response(ResponseSubState),
        }
        impl ChannelSubState {
            pub fn xml_start_event(self, tag_lowercase: &str) -> Result<State, StationError> {
                match self {
                    ChannelSubState::Initial => match tag_lowercase {
                        "latitude" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Latitude),
                        ))),
                        "longitude" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Longitude),
                        ))),
                        "elevation" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Elevation),
                        ))),
                        "depth" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Depth),
                        ))),
                        "azimuth" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Azimuth),
                        ))),
                        "dip" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Dip),
                        ))),
                        "type" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Type),
                        ))),
                        "samplerate" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::SampleRate),
                        ))),
                        "clockdrift" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::ClockDrift),
                        ))),
                        "calibrationunits" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::CalibrationUnits(
                                UnitsSubState::Initial,
                            )),
                        ))),
                        "sensor" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Sensor(
                                SensorSubState::Initial,
                            )),
                        ))),
                        "response" => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Response(
                                ResponseSubState::Initial,
                            )),
                        ))),
                        _ => todo!("channel substate initial tag: {}", tag_lowercase),
                    },
                    ChannelSubState::Latitude => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Longitude => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Elevation => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Depth => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Azimuth => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Dip => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::Type => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::SampleRate => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::ClockDrift => {
                        return Err(StationError::XMLStructureError);
                    }
                    ChannelSubState::CalibrationUnits(unit_state) => {
                        Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::CalibrationUnits(
                                unit_state.xml_start_event(tag_lowercase)?,
                            )),
                        )))
                    }
                    ChannelSubState::Sensor(state) => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Sensor(
                            state.xml_start_event(tag_lowercase)?,
                        )),
                    ))),
                    ChannelSubState::Response(state) => Ok(State::Network(
                        NetworkSubState::Station(StationSubState::Channel(
                            ChannelSubState::Response(state.xml_start_event(tag_lowercase)?),
                        )),
                    )),
                }
            }
            pub fn end_xml_event(self) -> Result<State, StationError> {
                match self {
                    ChannelSubState::Initial => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Initial,
                    ))),
                    ChannelSubState::Latitude => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Longitude => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Elevation => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Depth => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Azimuth => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Dip => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::Type => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::SampleRate => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::ClockDrift => Ok(State::Network(NetworkSubState::Station(
                        StationSubState::Channel(ChannelSubState::Initial),
                    ))),
                    ChannelSubState::CalibrationUnits(unit_state) => {
                        match unit_state.xml_end_event()? {
                            EndEvent::Backtrack => Ok(State::Network(NetworkSubState::Station(
                                StationSubState::Channel(ChannelSubState::Initial),
                            ))),
                            EndEvent::Continue(state) => Ok(State::Network(
                                NetworkSubState::Station(StationSubState::Channel(
                                    ChannelSubState::CalibrationUnits(state),
                                )),
                            )),
                        }
                    }
                    ChannelSubState::Sensor(sensor_state) => match sensor_state.xml_end_event()? {
                        EndEvent::Backtrack => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Initial),
                        ))),
                        EndEvent::Continue(state) => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Sensor(state)),
                        ))),
                    },
                    ChannelSubState::Response(state) => match state.xml_end_event()? {
                        EndEvent::Backtrack => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Initial),
                        ))),
                        EndEvent::Continue(state) => Ok(State::Network(NetworkSubState::Station(
                            StationSubState::Channel(ChannelSubState::Response(state)),
                        ))),
                    },
                }
            }
        }
        #[derive(Clone, PartialEq, Debug)]
        enum StationSubState {
            Initial,
            Latitude,
            Longitude,
            Elevation,
            Site(SiteSubState),
            CreationDate,
            TotalNumberChannels,
            SelectedNumberChannels,
            Channel(ChannelSubState),
        }
        #[derive(Clone, PartialEq, Debug)]
        enum NetworkSubState {
            Initial,
            Description,
            Identifier,
            TotalNumberStations,
            SelectedNumberStations,
            Station(StationSubState),
        }

        #[derive(Clone, PartialEq, Debug)]
        enum State {
            Initial,
            InXML,
            FDSNStationXML,
            Source,
            Sender,
            Module,
            ModuleUri,
            Created,
            Network(NetworkSubState),
        }

        let mut output = Self {
            sender: String::new(),
            source: String::new(),
            module: String::new(),
            module_uri: String::new(),
            creation_date: DateTime::from_timestamp_nanos(0),
            networks: Vec::new(),
        };
        let mut state = State::Initial;
        let mut reader = XMLReader::from_reader(r);
        let mut buff = Vec::new();
        loop {
            match reader.read_event_into(&mut buff) {
                Err(e) => return Err(e.into()),
                Ok(event) => match event {
                    XMLEvent::Eof => break,
                    XMLEvent::Decl(decl) => {
                        state = match state {
                            State::Initial => State::InXML,
                            _ => return Err(StationError::XMLStructureError),
                        };
                    }
                    XMLEvent::Text(v) => {
                        let text_string = String::from_utf8(v.to_vec())?;
                        match &state {
                            State::InXML => {}
                            State::FDSNStationXML => {}
                            State::Initial => {}
                            State::Source => output.source = text_string,
                            State::Sender => output.sender = text_string,
                            State::Module => output.module = text_string,
                            State::ModuleUri => output.module_uri = text_string,
                            State::Created => {
                                println!("parse string: {}", text_string);
                                // format: YYYY-MM-DDTHH:mm:SS
                                // Where Y: Year
                                // M: Month
                                // DD: DAY
                                let mut semi_split = text_string.split("T").take(2);
                                let year_month_day_part = semi_split.next().unwrap();
                                let hour_minute_second_part = semi_split.next().unwrap();
                                println!(
                                    "{} {} {}",
                                    &year_month_day_part[0..=3],
                                    &year_month_day_part[5..=6],
                                    &year_month_day_part[8..=9]
                                );
                                let year: i32 = year_month_day_part[0..=3].parse()?;
                                let month: u32 = year_month_day_part[5..=6].parse()?;
                                let day: u32 = year_month_day_part[8..=9].parse()?;

                                println!("year: {}, month: {} day: {}", year, month, day);
                                println!("{}", hour_minute_second_part);
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

                                let microseconds = seconds_fraction_str.parse::<u32>()?
                                    * 10_u32.pow(MAXIMUM_SECONDS_DECIMAL - number_digits as u32);

                                println!(
                                    "Hour: {}, Minute: {}, Seconds Whole: {} seconds decimal str: \"{}\", microseconds: {}",
                                    hour, minute, seconds_whole, seconds_fraction_str, microseconds
                                );
                                output.creation_date = Utc
                                    .with_ymd_and_hms(year, month, day, hour, minute, seconds_whole)
                                    .unwrap()
                            }
                            State::Network(sub_state) => {
                                println!("todo: handle network text: \"{}\"", text_string)
                            }
                        }
                    }
                    XMLEvent::Start(v) => {
                        println!("{:#?}", v);
                        let start_string = String::from_utf8(v.to_vec())?;
                        let tag_lowercase = start_string
                            .split_whitespace()
                            .next()
                            .unwrap()
                            .to_lowercase();
                        state = match state {
                            State::Initial => return Err(StationError::XMLStructureError),
                            State::InXML => {
                                if tag_lowercase != "fdsnstationxml" {
                                    return Err(StationError::XMLStructureError);
                                }
                                State::FDSNStationXML
                            }
                            State::FDSNStationXML => match tag_lowercase.as_str() {
                                "source" => State::Source,
                                "sender" => State::Sender,
                                "module" => State::Module,
                                "moduleuri" => State::ModuleUri,
                                "created" => State::Created,
                                "network" => State::Network(NetworkSubState::Initial),
                                _ => {
                                    todo!("tag not implemented: \"{}\"", tag_lowercase)
                                }
                            },
                            State::Network(sub_state) => match sub_state {
                                NetworkSubState::Initial => match tag_lowercase.as_str() {
                                    "description" => State::Network(NetworkSubState::Description),
                                    "identifier" => State::Network(NetworkSubState::Identifier),
                                    "totalnumberstations" => {
                                        State::Network(NetworkSubState::TotalNumberStations)
                                    }
                                    "selectednumberstations" => {
                                        State::Network(NetworkSubState::SelectedNumberStations)
                                    }
                                    "station" => State::Network(NetworkSubState::Station(
                                        StationSubState::Initial,
                                    )),
                                    _ => {
                                        todo!("network sub tag not implemented: {}", tag_lowercase)
                                    }
                                },
                                NetworkSubState::Description => todo!(
                                    "network state, tag: {}, network sub state: {:#?}",
                                    tag_lowercase,
                                    sub_state
                                ),
                                NetworkSubState::Identifier => todo!("identifier"),
                                NetworkSubState::TotalNumberStations => {
                                    todo!("total number of stations")
                                }
                                NetworkSubState::SelectedNumberStations => {
                                    todo!("selected number stations")
                                }
                                NetworkSubState::Station(station_sub_state) => {
                                    match station_sub_state {
                                        StationSubState::Initial => match tag_lowercase.as_str() {
                                            "latitude" => State::Network(NetworkSubState::Station(
                                                StationSubState::Latitude,
                                            )),
                                            "longitude" => {
                                                State::Network(NetworkSubState::Station(
                                                    StationSubState::Longitude,
                                                ))
                                            }
                                            "elevation" => {
                                                State::Network(NetworkSubState::Station(
                                                    StationSubState::Elevation,
                                                ))
                                            }
                                            "site" => State::Network(NetworkSubState::Station(
                                                StationSubState::Site(SiteSubState::Initial),
                                            )),
                                            "creationdate" => {
                                                State::Network(NetworkSubState::Station(
                                                    StationSubState::CreationDate,
                                                ))
                                            }
                                            "totalnumberchannels" => {
                                                State::Network(NetworkSubState::Station(
                                                    StationSubState::TotalNumberChannels,
                                                ))
                                            }
                                            "selectednumberchannels" => {
                                                State::Network(NetworkSubState::Station(
                                                    StationSubState::SelectedNumberChannels,
                                                ))
                                            }
                                            "channel" => State::Network(NetworkSubState::Station(
                                                StationSubState::Channel(ChannelSubState::Initial),
                                            )),
                                            _ => todo!("station tag: {}", tag_lowercase),
                                        },
                                        StationSubState::Latitude => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::Longitude => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::Elevation => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::Site(site_state) => match site_state {
                                            SiteSubState::Initial => match tag_lowercase.as_str() {
                                                "name" => State::Network(NetworkSubState::Station(
                                                    StationSubState::Site(SiteSubState::Name),
                                                )),
                                                _ => todo!("site tag: {}", tag_lowercase),
                                            },
                                            SiteSubState::Name => {
                                                return Err(StationError::XMLStructureError);
                                            }
                                        },
                                        StationSubState::CreationDate => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::TotalNumberChannels => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::SelectedNumberChannels => {
                                            return Err(StationError::XMLStructureError);
                                        }
                                        StationSubState::Channel(channel_sub_state) => {
                                            channel_sub_state.xml_start_event(&tag_lowercase)?
                                        }
                                    }
                                }
                            },

                            State::Source => return Err(StationError::XMLStructureError),
                            State::Sender => return Err(StationError::XMLStructureError),
                            State::Module => return Err(StationError::XMLStructureError),
                            State::ModuleUri => return Err(StationError::XMLStructureError),
                            State::Created => return Err(StationError::XMLStructureError),
                        };
                    }
                    XMLEvent::End(v) => {
                        state = match state {
                            State::Source => State::FDSNStationXML,
                            State::Sender => State::FDSNStationXML,
                            State::Module => State::FDSNStationXML,
                            State::ModuleUri => State::FDSNStationXML,
                            State::Created => State::FDSNStationXML,
                            State::Network(sub_state) => match sub_state {
                                NetworkSubState::Initial => State::FDSNStationXML,
                                NetworkSubState::Description => {
                                    State::Network(NetworkSubState::Initial)
                                }
                                NetworkSubState::Identifier => {
                                    State::Network(NetworkSubState::Initial)
                                }
                                NetworkSubState::TotalNumberStations => {
                                    State::Network(NetworkSubState::Initial)
                                }
                                NetworkSubState::SelectedNumberStations => {
                                    State::Network(NetworkSubState::Initial)
                                }

                                NetworkSubState::Station(station_state) => match station_state {
                                    StationSubState::Initial => {
                                        State::Network(NetworkSubState::Initial)
                                    }
                                    StationSubState::Latitude => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::Longitude => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::Elevation => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::Site(sub_state) => match sub_state {
                                        SiteSubState::Initial => State::Network(
                                            NetworkSubState::Station(StationSubState::Initial),
                                        ),
                                        SiteSubState::Name => {
                                            State::Network(NetworkSubState::Station(
                                                StationSubState::Site(SiteSubState::Initial),
                                            ))
                                        }
                                    },
                                    StationSubState::CreationDate => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::TotalNumberChannels => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::SelectedNumberChannels => State::Network(
                                        NetworkSubState::Station(StationSubState::Initial),
                                    ),
                                    StationSubState::Channel(channel_sub_state) => {
                                        channel_sub_state.end_xml_event()?
                                    }
                                },
                            },
                            State::FDSNStationXML => State::InXML,
                            _ => {
                                todo!("end: {:#?}, state: {:#?}", v, state)
                            }
                        };
                    }
                    _ => println!("{:#?}", event),
                },
            }
        }

        Ok(output)
    }
}
#[cfg(test)]
mod tests {
    use prelude::chrono::{TimeDelta, TimeZone, Utc};
    use std::io::Cursor;
    #[test]
    fn basic_station() {
        let xml_str = r#"
<?xml version="1.0" encoding="ISO-8859-1"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:iris="http://www.fdsn.org/xml/station/1/iris" xsi:schemaLocation="http://www.fdsn.org/xml/station/1 http://www.fdsn.org/xml/station/fdsn-station-1.1.xsd" schemaVersion="1.1">
<Source>IRIS-DMC</Source>
<Sender>IRIS-DMC</Sender>
<Module>IRIS WEB SERVICE: fdsnws-station | version: 1.1.52</Module>
<ModuleURI>test</ModuleURI>
<Created>2026-05-29T05:51:16.950</Created>
<Network code="AK" startDate="1987-01-01T00:00:00.0000" restrictedStatus="open">
           <Description>Alaska Regional Network ()</Description>
   <Identifier type="DOI">10.7914/SN/AK</Identifier>
   <TotalNumberStations>320</TotalNumberStations>
   <SelectedNumberStations>1</SelectedNumberStations>
   <Station code="A19K" startDate="2020-09-23T00:00:00.0000" restrictedStatus="open" iris:alternateNetworkCodes=".EARTHSCOPE,.GREG,_REALTIME,.UNRESTRICTED,_US-ALL,_US-TA,_US-TA-ADOPTED">
    <Latitude>70.2043</Latitude>
    <Longitude>-161.0713</Longitude>
    <Elevation>24.0</Elevation>
    <Site>
     <Name>Wainwright, AK, USA</Name>
    </Site>
    <CreationDate>2020-09-23T00:00:00.0000</CreationDate>
    <TotalNumberChannels>1</TotalNumberChannels>
    <SelectedNumberChannels>1</SelectedNumberChannels>
    <Channel code="BHE" locationCode="" startDate="2020-09-23T00:00:00.0000" restrictedStatus="open">
     <Latitude>70.2043</Latitude>
     <Longitude>-161.0713</Longitude>
     <Elevation>24</Elevation>
     <Depth>2.6</Depth>
     <Azimuth>90</Azimuth>
     <Dip>0</Dip>
     <Type>GEOPHYSICAL</Type>
     <SampleRate>5E01</SampleRate>
     <ClockDrift>2E-04</ClockDrift>
     <CalibrationUnits>
      <Name>V</Name>
      <Description>emf in volts</Description>
     </CalibrationUnits>
     <Sensor>
      <Description>Streckeisen STS-5A/Quanterra 330 Linear Phase Belo</Description>
     </Sensor>
     <Response>
     <InstrumentSensitivity>
       <Value>6.28316E8</Value>
       <Frequency>0.2</Frequency>
       <InputUnits>
         <Name>m/s</Name>
         <Description>velocity in meters per second</Description>
       </InputUnits>
       <OutputUnits>
         <Name>counts</Name>
         <Description>digital counts</Description>
       </OutputUnits>
     </InstrumentSensitivity>
     </Response>
    </Channel>
   
   </Station>
     </Network>
 </FDSNStationXML>
    "#;
        let expected_xml = FDSNStationXML {
            source: "IRIS-DMC".to_string(),
            sender: "IRIS-DMC".to_string(),
            module: "IRIS WEB SERVICE: fdsnws-station | version: 1.1.52".to_string(),
            module_uri: "test".to_string(),
            creation_date: Utc.with_ymd_and_hms(2026, 05, 29, 5, 51, 16).unwrap()
                + TimeDelta::milliseconds(950),
            networks: vec![Network {
                code: "AK".to_string(),
                start_date: Utc.with_ymd_and_hms(1987, 01, 01, 0, 0, 0).unwrap(),
                restricted_status: "open".to_string(),
                stations: vec![Station {
                    code: "A19K".to_string(),
                    start_date: Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap(),
                    end_date: None,
                    latitude: 70.2043,
                    longitude: -161.0713,
                    elevation: 24.0,
                    site_name: "Wainwright, AK, USA".to_string(),
                    creation_date: Utc.with_ymd_and_hms(1987, 01, 01, 0, 0, 0).unwrap(),
                    channels: vec![Channel {
                        code: "BHE".to_string(),
                        location_code: "".to_string(),
                        start_date: Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap(),
                        latitude: 70.2043,
                        longitude: 70.2043,
                        elevation: 24.,
                        depth: 2.6,
                        azimuth: 90.,
                        dip: 0.,
                        sample_rate: 50.,
                        clock_drift: 2.0e-4,
                        calibration_unit: CalibrationUnit {
                            name: "V".to_string(),
                            description: "emf in volts".to_string(),
                        },
                        sensor: Sensor {
                            description: "Streckeisen STS-5A/Quanterra 330 Linear Phase Belo"
                                .to_string(),
                        },
                        response: Response {
                            instrument_sensitivity: InstrumentSensitivity {
                                value: 6.28316E8,
                                frequency: 0.2,
                                input_unit: Unit {
                                    name: "m/s".to_string(),
                                    description: "velocity in meters per second".to_string(),
                                },
                                output_unit: Unit {
                                    name: "counts".to_string(),
                                    description: "digital counts".to_string(),
                                },
                            },
                        },
                    }],
                }],
            }],
        };
        let xml = FDSNStationXML::from_xml(Cursor::new(xml_str)).unwrap();
        assert_eq!(xml, expected_xml);
    }

    use super::*;
}
