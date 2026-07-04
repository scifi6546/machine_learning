use super::{
    Channel, InstrumentSensitivity, Network, Response, Sensor, Station, StationError, StationXML,
    Unit, local_prelude::parse_to_date_time,
};

use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq)]
pub enum NameSyntaxError {
    ExpectedEquals { got: char },
    NoIdentifierFound,
    NoValueFound,
}
impl std::fmt::Display for NameSyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ExpectedEquals { got } => {
                write!(f, "expected '=' after identifier got: '{}'", got)?;
            }
            Self::NoIdentifierFound => write!(f, "no identifier present")?,
            Self::NoValueFound => write!(f, "no value present")?,
        }
        Ok(())
    }
}

pub enum EndEvent<T: Clone + Copy + PartialEq> {
    Backtrack,
    Continue(T),
}
pub trait SubState: Sized + Copy + Clone + PartialEq {
    type Output;
    fn from_start_text(_text: &str) -> Result<Self::Output, StationError>;
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        element: &mut Self::Output,
    ) -> Result<Self, StationError>;
    fn xml_text_event(&self, _text: &str, _element: &mut Self::Output) -> Result<(), StationError> {
        Ok(())
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError>;
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UnitsSubState {
    Initial,
    Name,
    Description,
}

impl SubState for UnitsSubState {
    type Output = Unit;
    fn from_start_text(_text: &str) -> Result<Self::Output, StationError> {
        Ok(Unit {
            name: None,
            description: None,
        })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        _full_start_text: &str,
        _unit: &mut Unit,
    ) -> Result<Self, StationError> {
        match self {
            UnitsSubState::Initial => match tag_lowercase {
                "name" => Ok(Self::Name),
                "description" => Ok(Self::Description),
                _ => todo!("unit state tag: {}", tag_lowercase),
            },
            UnitsSubState::Name => Err(StationError::XMLStructureError),
            UnitsSubState::Description => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(&self, text: &str, unit: &mut Unit) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Name => {
                unit.name = Some(text.to_string());
                Ok(())
            }
            Self::Description => {
                unit.description = Some(text.to_string());
                Ok(())
            }
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            UnitsSubState::Initial => Ok(EndEvent::Backtrack),
            UnitsSubState::Name => Ok(EndEvent::Continue(Self::Initial)),
            UnitsSubState::Description => Ok(EndEvent::Continue(Self::Initial)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SensorSubState {
    Initial,
    Description,
}
impl SubState for SensorSubState {
    type Output = Sensor;
    fn from_start_text(_text: &str) -> Result<Sensor, StationError> {
        Ok(Sensor { description: None })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        _full_start_text: &str,
        _sensor: &mut Sensor,
    ) -> Result<Self, StationError> {
        match tag_lowercase {
            "description" => Ok(Self::Description),
            _ => panic!("invalid sensor substate tag: {}", tag_lowercase),
        }
    }
    fn xml_text_event(&self, text: &str, sensor: &mut Sensor) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Description => {
                sensor.description = Some(text.to_string());
                Ok(())
            }
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Description => Ok(EndEvent::Continue(Self::Initial)),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InstrumentSensitivityState {
    Initial,
    Value,
    Frequency,
    InputUnits(UnitsSubState),
    OutputUnits(UnitsSubState),
}
impl SubState for InstrumentSensitivityState {
    type Output = InstrumentSensitivity;
    fn from_start_text(_text: &str) -> Result<InstrumentSensitivity, StationError> {
        Ok(InstrumentSensitivity {
            value: None,
            frequency: None,
            input_unit: None,
            output_unit: None,
        })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        sensitivity: &mut InstrumentSensitivity,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "value" => Ok(Self::Value),
                "frequency" => Ok(Self::Frequency),
                "inputunits" => {
                    sensitivity.input_unit = Some(UnitsSubState::from_start_text(full_start_text)?);
                    Ok(Self::InputUnits(UnitsSubState::Initial))
                }
                "outputunits" => {
                    sensitivity.output_unit =
                        Some(UnitsSubState::from_start_text(full_start_text)?);
                    Ok(Self::OutputUnits(UnitsSubState::Initial))
                }
                _ => panic!("invalid response sub tag: {}", tag_lowercase),
            },
            Self::Value => Err(StationError::XMLStructureError),
            Self::Frequency => Err(StationError::XMLStructureError),
            Self::InputUnits(state) => {
                let unit = sensitivity
                    .input_unit
                    .as_mut()
                    .expect("should have input unit");
                let state = state.xml_start_event(tag_lowercase, full_start_text, unit)?;

                Ok(Self::InputUnits(state))
            }
            Self::OutputUnits(state) => {
                let unit = sensitivity
                    .output_unit
                    .as_mut()
                    .expect("should have input unit");
                let state = state.xml_start_event(tag_lowercase, full_start_text, unit)?;

                Ok(Self::OutputUnits(state))
            }
        }
    }
    fn xml_text_event(
        &self,
        text: &str,
        sensitivity: &mut InstrumentSensitivity,
    ) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Value => {
                sensitivity.value = Some(text.parse()?);
                Ok(())
            }
            Self::Frequency => {
                sensitivity.frequency = Some(text.parse()?);
                Ok(())
            }
            Self::InputUnits(state) => state.xml_text_event(
                text,
                sensitivity
                    .input_unit
                    .as_mut()
                    .expect("should have input unit"),
            ),
            Self::OutputUnits(state) => state.xml_text_event(
                text,
                sensitivity
                    .output_unit
                    .as_mut()
                    .expect("should have input unit"),
            ),
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Value => Ok(EndEvent::Continue(Self::Initial)),
            Self::Frequency => Ok(EndEvent::Continue(Self::Initial)),
            Self::InputUnits(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::InputUnits(state))),
            },
            Self::OutputUnits(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::OutputUnits(state))),
            },
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResponseSubState {
    Initial,
    InstrumentSensitivity(InstrumentSensitivityState),
}

impl SubState for ResponseSubState {
    type Output = Response;
    fn from_start_text(_text: &str) -> Result<Response, StationError> {
        Ok(Response {
            instrument_sensitivity: None,
        })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        response: &mut Response,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "instrumentsensitivity" => {
                    response.instrument_sensitivity = Some(
                        InstrumentSensitivityState::from_start_text(full_start_text)?,
                    );
                    Ok(Self::InstrumentSensitivity(
                        InstrumentSensitivityState::Initial,
                    ))
                }
                _ => panic!("invalid response sub tag: {}", tag_lowercase),
            },
            Self::InstrumentSensitivity(state) => {
                let sensitivity = response
                    .instrument_sensitivity
                    .as_mut()
                    .expect("should have sensitivity");
                let state = state.xml_start_event(tag_lowercase, full_start_text, sensitivity)?;

                Ok(ResponseSubState::InstrumentSensitivity(state))
            }
        }
    }
    fn xml_text_event(&self, text: &str, response: &mut Response) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::InstrumentSensitivity(state) => state.xml_text_event(
                text,
                response
                    .instrument_sensitivity
                    .as_mut()
                    .expect("should have instrument sensitivity"),
            ),
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChannelSubState {
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

impl SubState for ChannelSubState {
    type Output = Channel;
    fn from_start_text(text: &str) -> Result<Channel, StationError> {
        let mut header = parse_header(text)?;
        let start_date = if let Some(text) = header.get("startDate") {
            Some(parse_to_date_time(text)?)
        } else {
            None
        };
        Ok(Channel {
            code: header.remove("code"),
            location_code: header
                .remove("locationCode")
                .and_then(|code| if code.is_empty() { None } else { Some(code) }),
            start_date,
            latitude: None,
            longitude: None,
            elevation: None,
            depth: None,
            azimuth: None,
            dip: None,
            sample_rate: None,
            clock_drift: None,
            calibration_unit: None,
            sensor: None,
            response: None,
        })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        channel: &mut Channel,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "latitude" => Ok(Self::Latitude),
                "longitude" => Ok(Self::Longitude),
                "elevation" => Ok(Self::Elevation),
                "depth" => Ok(Self::Depth),
                "azimuth" => Ok(Self::Azimuth),
                "dip" => Ok(Self::Dip),
                "type" => Ok(Self::Type),
                "samplerate" => Ok(Self::SampleRate),
                "clockdrift" => Ok(Self::ClockDrift),
                "calibrationunits" => {
                    channel.calibration_unit =
                        Some(UnitsSubState::from_start_text(full_start_text)?);
                    Ok(Self::CalibrationUnits(UnitsSubState::Initial))
                }
                "sensor" => {
                    channel.sensor = Some(SensorSubState::from_start_text(full_start_text)?);
                    Ok(Self::Sensor(SensorSubState::Initial))
                }
                "response" => {
                    channel.response = Some(ResponseSubState::from_start_text(full_start_text)?);
                    Ok(Self::Response(ResponseSubState::Initial))
                }
                _ => todo!("channel substate initial tag: {}", tag_lowercase),
            },
            Self::Latitude => Err(StationError::XMLStructureError),
            Self::Longitude => Err(StationError::XMLStructureError),
            Self::Elevation => Err(StationError::XMLStructureError),
            Self::Depth => Err(StationError::XMLStructureError),
            Self::Azimuth => Err(StationError::XMLStructureError),
            Self::Dip => Err(StationError::XMLStructureError),
            Self::Type => Err(StationError::XMLStructureError),
            Self::SampleRate => Err(StationError::XMLStructureError),
            Self::ClockDrift => Err(StationError::XMLStructureError),
            Self::CalibrationUnits(state) => {
                let unit = channel.calibration_unit.as_mut().expect("should have unit");
                let state = state.xml_start_event(tag_lowercase, full_start_text, unit)?;

                Ok(Self::CalibrationUnits(state))
            }
            Self::Sensor(state) => {
                let sensor = channel.sensor.as_mut().expect("should have sensor");
                let state = state.xml_start_event(tag_lowercase, full_start_text, sensor)?;

                Ok(Self::Sensor(state))
            }
            Self::Response(state) => {
                let response = channel.response.as_mut().expect("should have response");
                let state = state.xml_start_event(tag_lowercase, full_start_text, response)?;

                Ok(Self::Response(state))
            }
        }
    }
    fn xml_text_event(&self, text: &str, channel: &mut Channel) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Latitude => {
                channel.latitude = Some(text.parse()?);
                Ok(())
            }
            Self::Longitude => {
                channel.longitude = Some(text.parse()?);
                Ok(())
            }
            Self::Elevation => {
                channel.elevation = Some(text.parse()?);
                Ok(())
            }
            Self::Depth => {
                channel.depth = Some(text.parse()?);
                Ok(())
            }
            Self::Azimuth => {
                channel.azimuth = Some(text.parse()?);
                Ok(())
            }
            Self::Dip => {
                channel.dip = Some(text.parse()?);
                Ok(())
            }
            Self::Type => Ok(()),
            Self::SampleRate => {
                channel.sample_rate = Some(text.parse()?);
                Ok(())
            }
            Self::ClockDrift => {
                channel.clock_drift = Some(text.parse()?);
                Ok(())
            }
            Self::CalibrationUnits(state) => state.xml_text_event(
                text,
                channel
                    .calibration_unit
                    .as_mut()
                    .expect("should have calibration unit"),
            ),
            Self::Sensor(state) => {
                state.xml_text_event(text, channel.sensor.as_mut().expect("should have sensor"))
            }
            Self::Response(state) => state.xml_text_event(
                text,
                channel.response.as_mut().expect("should have response"),
            ),
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            ChannelSubState::Initial => Ok(EndEvent::Backtrack),
            ChannelSubState::Latitude => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Longitude => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Elevation => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Depth => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Azimuth => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Dip => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::Type => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::SampleRate => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::ClockDrift => Ok(EndEvent::Continue(Self::Initial)),
            ChannelSubState::CalibrationUnits(unit_state) => match unit_state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::CalibrationUnits(state))),
            },
            ChannelSubState::Sensor(sensor_state) => match sensor_state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Sensor(state))),
            },
            ChannelSubState::Response(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Response(state))),
            },
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SiteSubState {
    Initial,
    Name,
}
impl SubState for SiteSubState {
    type Output = Option<String>;
    fn from_start_text(_text: &str) -> Result<Self::Output, StationError> {
        Ok(None)
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        _full_start_text: &str,
        _site_name: &mut Option<String>,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "name" => Ok(Self::Name),
                _ => todo!("site tag: {}", tag_lowercase),
            },
            SiteSubState::Name => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(
        &self,
        text: &str,
        site_name: &mut Option<String>,
    ) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Name => {
                *site_name = Some(text.to_string());
                Ok(())
            }
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Name => Ok(EndEvent::Continue(Self::Initial)),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StationSubState {
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

impl SubState for StationSubState {
    type Output = Station;
    fn from_start_text(text: &str) -> Result<Self::Output, StationError> {
        let header = parse_header(text)?;

        let start_date = if let Some(text) = header.get("startDate") {
            Some(parse_to_date_time(text)?)
        } else {
            None
        };
        Ok(Station {
            code: header.get("code").cloned(),
            start_date,
            end_date: None,
            elevation: None,
            latitude: None,
            longitude: None,
            site_name: None,
            creation_date: None,
            channels: Vec::new(),
        })
    }

    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        station: &mut Station,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "latitude" => Ok(Self::Latitude),
                "longitude" => Ok(Self::Longitude),
                "elevation" => Ok(Self::Elevation),
                "site" => Ok(Self::Site(SiteSubState::Initial)),
                "creationdate" => Ok(Self::CreationDate),
                "totalnumberchannels" => Ok(Self::TotalNumberChannels),
                "selectednumberchannels" => Ok(Self::SelectedNumberChannels),
                "channel" => {
                    station
                        .channels
                        .push(ChannelSubState::from_start_text(full_start_text)?);

                    Ok(Self::Channel(ChannelSubState::Initial))
                }
                _ => todo!("station tag: {}", tag_lowercase),
            },
            Self::Latitude => Err(StationError::XMLStructureError),
            Self::Longitude => Err(StationError::XMLStructureError),
            Self::Elevation => Err(StationError::XMLStructureError),
            Self::Site(state) => {
                let state = state.xml_start_event(
                    tag_lowercase,
                    full_start_text,
                    &mut station.site_name,
                )?;

                Ok(Self::Site(state))
            }
            Self::CreationDate => Err(StationError::XMLStructureError),
            Self::TotalNumberChannels => Err(StationError::XMLStructureError),
            Self::SelectedNumberChannels => Err(StationError::XMLStructureError),
            Self::Channel(state) => {
                let channel = station.channels.last_mut().expect("should have channel");
                let state = state.xml_start_event(tag_lowercase, full_start_text, channel)?;

                Ok(Self::Channel(state))
            }
        }
    }
    fn xml_text_event(&self, text: &str, station: &mut Station) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Latitude => {
                station.latitude = Some(text.parse()?);
                Ok(())
            }
            Self::Longitude => {
                station.longitude = Some(text.parse()?);
                Ok(())
            }
            Self::Elevation => {
                station.elevation = Some(text.parse()?);
                Ok(())
            }
            Self::Site(site) => site.xml_text_event(text, &mut station.site_name),
            Self::CreationDate => {
                station.creation_date = Some(parse_to_date_time(text)?);

                Ok(())
            }
            Self::TotalNumberChannels => Ok(()),
            Self::SelectedNumberChannels => Ok(()),
            Self::Channel(channel) => channel.xml_text_event(
                text,
                station
                    .channels
                    .last_mut()
                    .expect("should have last channel"),
            ),
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Latitude => Ok(EndEvent::Continue(Self::Initial)),
            Self::Longitude => Ok(EndEvent::Continue(Self::Initial)),
            Self::Elevation => Ok(EndEvent::Continue(Self::Initial)),
            Self::Site(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Site(state))),
            },
            Self::CreationDate => Ok(EndEvent::Continue(Self::Initial)),
            Self::TotalNumberChannels => Ok(EndEvent::Continue(Self::Initial)),
            Self::SelectedNumberChannels => Ok(EndEvent::Continue(Self::Initial)),
            Self::Channel(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Channel(state))),
            },
        }
    }
}
fn parse_header(header: &str) -> Result<HashMap<String, String>, StationError> {
    #[derive(Debug, Clone, Copy)]
    enum ParseState {
        InName,
        WaitingIdentifier,
        InIdentifier,
        WaitingEquals,
        WaitingValue,

        InValueInQuotes,
    }
    let mut parse_state = ParseState::InName;

    let mut current_identifier = Some(String::new());
    let mut current_value = Some(String::new());
    let mut header_map: HashMap<String, String> = HashMap::new();
    for c in header.chars() {
        match parse_state {
            ParseState::InName => {
                if c.is_whitespace() {
                    parse_state = ParseState::WaitingIdentifier;
                }
            }
            ParseState::WaitingIdentifier => {
                if !c.is_whitespace() {
                    current_identifier = Some(c.into());
                    parse_state = ParseState::InIdentifier;
                }
            }
            ParseState::InIdentifier => {
                if !c.is_whitespace() {
                    if c == '=' {
                        parse_state = ParseState::WaitingValue;
                    } else {
                        current_identifier
                            .as_mut()
                            .expect("should already be present")
                            .push(c);
                    }
                } else {
                    parse_state = ParseState::WaitingEquals;
                }
            }
            ParseState::WaitingEquals => {
                if !c.is_whitespace() {
                    if c == '=' {
                        parse_state = ParseState::WaitingValue;
                    } else {
                        return Err(StationError::InvalidNameSyntax(
                            NameSyntaxError::ExpectedEquals { got: c },
                        ));
                    }
                }
            }
            ParseState::WaitingValue => {
                if !c.is_whitespace() {
                    if c == '\"' {
                        parse_state = ParseState::InValueInQuotes;
                        current_value = Some(String::new());
                    } else {
                        return Err(StationError::NeedNameQuotes);
                    }
                }
            }
            ParseState::InValueInQuotes => {
                if c == '\"' {
                    parse_state = ParseState::WaitingIdentifier;
                    let identifier = if let Some(id) = current_identifier.take() {
                        id
                    } else {
                        return Err(StationError::InvalidNameSyntax(
                            NameSyntaxError::NoIdentifierFound,
                        ));
                    };
                    let value = if let Some(id) = current_value.take() {
                        id
                    } else {
                        return Err(StationError::InvalidNameSyntax(
                            NameSyntaxError::NoValueFound,
                        ));
                    };
                    header_map.insert(identifier, value);
                } else {
                    current_value
                        .as_mut()
                        .expect("should alrady have value")
                        .push(c);
                }
            }
        }
    }
    Ok(header_map)
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NetworkSubState {
    Initial,
    Description,
    Identifier,
    TotalNumberStations,
    SelectedNumberStations,
    Station(StationSubState),
}

impl SubState for NetworkSubState {
    type Output = Network;
    fn from_start_text(text: &str) -> Result<Self::Output, StationError> {
        let header_map = parse_header(text)?;
        let start_date = if let Some(text) = header_map.get("startDate") {
            Some(parse_to_date_time(text)?)
        } else {
            None
        };
        Ok(Network {
            code: header_map.get("code").cloned(),
            start_date,
            restricted_status: header_map.get("restrictedStatus").cloned(),
            stations: Vec::new(),
        })
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        network: &mut Network,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "description" => Ok(Self::Description),
                "identifier" => Ok(Self::Identifier),
                "totalnumberstations" => Ok(Self::TotalNumberStations),
                "selectednumberstations" => Ok(Self::SelectedNumberStations),
                "station" => {
                    network
                        .stations
                        .push(StationSubState::from_start_text(full_start_text)?);
                    Ok(Self::Station(StationSubState::Initial))
                }
                _ => {
                    todo!("invalid tag: {}", tag_lowercase)
                }
            },
            Self::Description => Err(StationError::XMLStructureError),
            Self::Identifier => Err(StationError::XMLStructureError),
            Self::TotalNumberStations => Err(StationError::XMLStructureError),
            Self::SelectedNumberStations => Err(StationError::XMLStructureError),
            Self::Station(state) => {
                let station = network.stations.last_mut().expect("should have station");
                let state = state.xml_start_event(tag_lowercase, full_start_text, station)?;

                Ok(Self::Station(state))
            }
        }
    }
    fn xml_text_event(&self, text: &str, network: &mut Network) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Description => Ok(()),
            Self::Identifier => Ok(()),
            Self::TotalNumberStations => Ok(()),
            Self::SelectedNumberStations => Ok(()),
            Self::Station(state) => state.xml_text_event(
                text,
                network.stations.last_mut().expect("should have station"),
            ),
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Description => Ok(EndEvent::Continue(Self::Initial)),
            Self::Identifier => Ok(EndEvent::Continue(Self::Initial)),
            Self::TotalNumberStations => Ok(EndEvent::Continue(Self::Initial)),
            Self::SelectedNumberStations => Ok(EndEvent::Continue(Self::Initial)),
            Self::Station(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Station(state))),
            },
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FDSNStationXMLState {
    Initial,
    Source,
    Sender,
    Module,
    ModuleUri,
    Created,
    Network(NetworkSubState),
}
impl SubState for FDSNStationXMLState {
    type Output = StationXML;
    fn from_start_text(_text: &str) -> Result<Self::Output, StationError> {
        todo!()
    }
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        xml: &mut StationXML,
    ) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "source" => Ok(Self::Source),
                "sender" => Ok(Self::Sender),
                "module" => Ok(Self::Module),
                "moduleuri" => Ok(Self::ModuleUri),
                "created" => Ok(Self::Created),
                "network" => {
                    xml.networks
                        .push(NetworkSubState::from_start_text(full_start_text)?);
                    Ok(Self::Network(NetworkSubState::Initial))
                }
                _ => {
                    todo!("tag not implemented: \"{}\"", tag_lowercase)
                }
            },
            Self::Network(state) => {
                let last_network = xml.networks.last_mut().expect("should have last network");
                let state = state.xml_start_event(tag_lowercase, full_start_text, last_network)?;

                Ok(Self::Network(state))
            }
            Self::Source => Err(StationError::XMLStructureError),
            Self::Sender => Err(StationError::XMLStructureError),
            Self::Module => Err(StationError::XMLStructureError),
            Self::ModuleUri => Err(StationError::XMLStructureError),
            Self::Created => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(&self, text: &str, xml: &mut StationXML) -> Result<(), StationError> {
        match self {
            Self::Initial => Ok(()),
            Self::Network(state) => {
                state.xml_text_event(text, xml.networks.last_mut().expect("should have network"))
            }
            Self::Source => {
                xml.source = text.to_string();
                Ok(())
            }
            Self::Sender => {
                xml.sender = text.to_string();
                Ok(())
            }
            Self::Module => {
                xml.module = text.to_string();
                Ok(())
            }
            Self::ModuleUri => {
                xml.module_uri = text.to_string();
                Ok(())
            }
            Self::Created => {
                xml.creation_date = parse_to_date_time(text)?;
                Ok(())
            }
        }
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError> {
        match self {
            Self::Initial => Ok(EndEvent::Backtrack),
            Self::Network(state) => match state.xml_end_event()? {
                EndEvent::Backtrack => Ok(EndEvent::Continue(Self::Initial)),
                EndEvent::Continue(state) => Ok(EndEvent::Continue(Self::Network(state))),
            },
            Self::Source => Ok(EndEvent::Continue(Self::Initial)),
            Self::Sender => Ok(EndEvent::Continue(Self::Initial)),
            Self::Module => Ok(EndEvent::Continue(Self::Initial)),
            Self::ModuleUri => Ok(EndEvent::Continue(Self::Initial)),
            Self::Created => Ok(EndEvent::Continue(Self::Initial)),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_parse_header() {
        let header_text = "Foo bar=\"1\" test=\"2\"";
        let expected_output = [
            ("bar".to_string(), "1".to_string()),
            ("test".to_string(), "2".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(parse_header(header_text).unwrap(), expected_output);
    }
}
