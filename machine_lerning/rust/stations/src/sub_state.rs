use super::{
    local_prelude::parse_to_date_time, Channel, Network, Station, StationError, StationXML, Unit,
};
use prelude::chrono::DateTime;

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
fn get_last_network(xml: &mut StationXML) -> Option<&mut Network> {
    xml.networks.last_mut()
}
fn get_last_station(xml: &mut StationXML) -> Option<&mut Station> {
    get_last_network(xml)
        .map(|v| v.stations.last_mut())
        .flatten()
}
fn get_last_channel(xml: &mut StationXML) -> Option<&mut Channel> {
    get_last_station(xml)
        .map(|v| v.channels.last_mut())
        .flatten()
}
pub enum EndEvent<T: Clone + Copy + PartialEq> {
    Backtrack,
    Continue(T),
}
pub trait SubState: Sized + Copy + Clone + PartialEq {
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError>;
    fn xml_text_event(&self, _text: &str, xml: StationXML) -> Result<StationXML, StationError> {
        Ok(xml)
    }
    fn xml_end_event(self) -> Result<EndEvent<Self>, StationError>;
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UnitsSubState {
    Initial,
    Name,
    Description,
}
impl UnitsSubState {
    pub fn from_start_text(text: &str) -> Result<Unit, StationError> {
        Ok(Unit {
            name: None,
            description: None,
        })
    }
}
impl SubState for UnitsSubState {
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        _full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            UnitsSubState::Initial => match tag_lowercase {
                "name" => Ok((Self::Name, xml)),
                "description" => Ok((Self::Description, xml)),
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
    fn xml_text_event(&self, _text: &str, xml: StationXML) -> Result<StationXML, StationError> {
        match self {
            Self::Initial => Ok(xml),
            Self::Description => Ok(xml),
            Self::Name => Ok(xml),
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
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match tag_lowercase {
            "description" => Ok((Self::Description, xml)),
            _ => panic!("invalid sensor substate tag: {}", tag_lowercase),
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
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "value" => Ok((Self::Value, xml)),
                "frequency" => Ok((Self::Frequency, xml)),
                "inputunits" => Ok((Self::InputUnits(UnitsSubState::Initial), xml)),
                "outputunits" => Ok((Self::InputUnits(UnitsSubState::Initial), xml)),
                _ => panic!("invalid response sub tag: {}", tag_lowercase),
            },
            Self::Value => return Err(StationError::XMLStructureError),
            Self::Frequency => return Err(StationError::XMLStructureError),
            Self::InputUnits(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::InputUnits(state), xml))
            }
            Self::OutputUnits(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::OutputUnits(state), xml))
            }
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
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "instrumentsensitivity" => Ok((
                    Self::InstrumentSensitivity(InstrumentSensitivityState::Initial),
                    xml,
                )),
                _ => panic!("invalid response sub tag: {}", tag_lowercase),
            },
            Self::InstrumentSensitivity(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((ResponseSubState::InstrumentSensitivity(state), xml))
            }
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
impl ChannelSubState {
    pub fn from_start_text(text: &str) -> Result<Channel, StationError> {
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
                .map(|code| if code.is_empty() { None } else { Some(code) })
                .flatten(),
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
}
impl SubState for ChannelSubState {
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        mut xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        let channel = get_last_channel(&mut xml).expect("need channel");
        match self {
            Self::Initial => match tag_lowercase {
                "latitude" => Ok((Self::Latitude, xml)),
                "longitude" => Ok((Self::Longitude, xml)),
                "elevation" => Ok((Self::Elevation, xml)),
                "depth" => Ok((Self::Depth, xml)),
                "azimuth" => Ok((Self::Azimuth, xml)),
                "dip" => Ok((Self::Dip, xml)),
                "type" => Ok((Self::Type, xml)),
                "samplerate" => Ok((Self::SampleRate, xml)),
                "clockdrift" => Ok((Self::ClockDrift, xml)),
                "calibrationunits" => {
                    channel.calibration_unit =
                        Some(UnitsSubState::from_start_text(full_start_text)?);
                    Ok((Self::CalibrationUnits(UnitsSubState::Initial), xml))
                }
                "sensor" => Ok((Self::Sensor(SensorSubState::Initial), xml)),
                "response" => Ok((Self::Response(ResponseSubState::Initial), xml)),
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
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;

                Ok((Self::CalibrationUnits(state), xml))
            }
            Self::Sensor(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::Sensor(state), xml))
            }
            Self::Response(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::Response(state), xml))
            }
        }
    }
    fn xml_text_event(&self, text: &str, mut xml: StationXML) -> Result<StationXML, StationError> {
        let channel = get_last_channel(&mut xml).expect("should have channel");
        match self {
            Self::Initial => Ok(xml),
            Self::Latitude => {
                channel.latitude = Some(text.parse()?);
                Ok(xml)
            }
            Self::Longitude => {
                channel.longitude = Some(text.parse()?);
                Ok(xml)
            }
            Self::Elevation => {
                channel.elevation = Some(text.parse()?);
                Ok(xml)
            }
            Self::Depth => {
                channel.depth = Some(text.parse()?);
                Ok(xml)
            }
            Self::Azimuth => {
                channel.azimuth = Some(text.parse()?);
                Ok(xml)
            }
            Self::Dip => {
                channel.dip = Some(text.parse()?);
                Ok(xml)
            }
            Self::Type => Ok(xml),
            Self::SampleRate => {
                channel.sample_rate = Some(text.parse()?);
                Ok(xml)
            }
            Self::ClockDrift => {
                channel.clock_drift = Some(text.parse()?);
                Ok(xml)
            }
            Self::CalibrationUnits(state) => state.xml_text_event(text, xml),
            Self::Sensor(state) => state.xml_text_event(text, xml),
            Self::Response(state) => state.xml_text_event(text, xml),
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
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        _full_start_text: &str,
        xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "name" => Ok((Self::Name, xml)),
                _ => todo!("site tag: {}", tag_lowercase),
            },
            SiteSubState::Name => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(&self, text: &str, mut xml: StationXML) -> Result<StationXML, StationError> {
        match self {
            Self::Initial => Ok(xml),
            Self::Name => {
                let site = get_last_station(&mut xml).expect("should have station");

                site.site_name = Some(text.to_string());
                Ok(xml)
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
impl StationSubState {
    pub fn from_start_text(start_text: &str) -> Result<Station, StationError> {
        let header = parse_header(start_text)?;

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
}
impl SubState for StationSubState {
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        mut xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "latitude" => Ok((Self::Latitude, xml)),
                "longitude" => Ok((Self::Longitude, xml)),
                "elevation" => Ok((Self::Elevation, xml)),
                "site" => Ok((Self::Site(SiteSubState::Initial), xml)),
                "creationdate" => Ok((Self::CreationDate, xml)),
                "totalnumberchannels" => Ok((Self::TotalNumberChannels, xml)),
                "selectednumberchannels" => Ok((Self::SelectedNumberChannels, xml)),
                "channel" => {
                    let station = get_last_station(&mut xml).expect("should have last station");
                    station
                        .channels
                        .push(ChannelSubState::from_start_text(full_start_text)?);

                    Ok((Self::Channel(ChannelSubState::Initial), xml))
                }
                _ => todo!("station tag: {}", tag_lowercase),
            },
            Self::Latitude => Err(StationError::XMLStructureError),
            Self::Longitude => Err(StationError::XMLStructureError),
            Self::Elevation => Err(StationError::XMLStructureError),
            Self::Site(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::Site(state), xml))
            }
            Self::CreationDate => Err(StationError::XMLStructureError),
            Self::TotalNumberChannels => Err(StationError::XMLStructureError),
            Self::SelectedNumberChannels => Err(StationError::XMLStructureError),
            Self::Channel(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::Channel(state), xml))
            }
        }
    }
    fn xml_text_event(&self, text: &str, mut xml: StationXML) -> Result<StationXML, StationError> {
        let station = get_last_station(&mut xml).expect("should have station");
        match self {
            Self::Initial => Ok(xml),
            Self::Latitude => {
                station.latitude = Some(text.parse()?);
                Ok(xml)
            }
            Self::Longitude => {
                station.longitude = Some(text.parse()?);
                Ok(xml)
            }
            Self::Elevation => {
                station.elevation = Some(text.parse()?);
                Ok(xml)
            }
            Self::Site(site) => site.xml_text_event(text, xml),
            Self::CreationDate => {
                station.creation_date = Some(parse_to_date_time(text)?);

                Ok(xml)
            }
            Self::TotalNumberChannels => Ok(xml),
            Self::SelectedNumberChannels => Ok(xml),
            Self::Channel(channel) => channel.xml_text_event(text, xml),
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
    return Ok(header_map);
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
impl NetworkSubState {
    fn network_from_start_text(start_text: &str) -> Result<Network, StationError> {
        let header_map = parse_header(start_text)?;
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
}
impl SubState for NetworkSubState {
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        mut xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "description" => Ok((Self::Description, xml)),
                "identifier" => Ok((Self::Identifier, xml)),
                "totalnumberstations" => Ok((Self::TotalNumberStations, xml)),
                "selectednumberstations" => Ok((Self::SelectedNumberStations, xml)),
                "station" => {
                    get_last_network(&mut xml)
                        .expect("should have network")
                        .stations
                        .push(StationSubState::from_start_text(full_start_text)?);
                    Ok((Self::Station(StationSubState::Initial), xml))
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
                let (state, mut xml) =
                    state.xml_start_event(tag_lowercase, full_start_text, xml)?;

                Ok((Self::Station(state), xml))
            }
        }
    }
    fn xml_text_event(&self, text: &str, mut xml: StationXML) -> Result<StationXML, StationError> {
        match self {
            Self::Initial => {}
            Self::Description => {}
            Self::Identifier => {}
            Self::TotalNumberStations => {}
            Self::SelectedNumberStations => {}
            Self::Station(state) => {
                xml = state.xml_text_event(text, xml)?;
            }
        };
        Ok(xml)
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
    fn xml_start_event(
        self,
        tag_lowercase: &str,
        full_start_text: &str,
        mut xml: StationXML,
    ) -> Result<(Self, StationXML), StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "source" => Ok((Self::Source, xml)),
                "sender" => Ok((Self::Sender, xml)),
                "module" => Ok((Self::Module, xml)),
                "moduleuri" => Ok((Self::ModuleUri, xml)),
                "created" => Ok((Self::Created, xml)),
                "network" => {
                    xml.networks
                        .push(NetworkSubState::network_from_start_text(full_start_text)?);
                    Ok((Self::Network(NetworkSubState::Initial), xml))
                }
                _ => {
                    todo!("tag not implemented: \"{}\"", tag_lowercase)
                }
            },
            Self::Network(state) => {
                let (state, xml) = state.xml_start_event(tag_lowercase, full_start_text, xml)?;
                Ok((Self::Network(state), xml))
            }
            Self::Source => Err(StationError::XMLStructureError),
            Self::Sender => Err(StationError::XMLStructureError),
            Self::Module => Err(StationError::XMLStructureError),
            Self::ModuleUri => Err(StationError::XMLStructureError),
            Self::Created => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(&self, text: &str, mut xml: StationXML) -> Result<StationXML, StationError> {
        match self {
            Self::Initial => {}
            Self::Network(state) => {
                xml = state.xml_text_event(text, xml)?;
            }
            Self::Source => {
                xml.source = text.to_string();
            }
            Self::Sender => {
                xml.sender = text.to_string();
            }
            Self::Module => {
                xml.module = text.to_string();
            }
            Self::ModuleUri => {
                xml.module_uri = text.to_string();
            }
            Self::Created => {
                xml.creation_date = parse_to_date_time(text)?;
            }
        }
        Ok(xml)
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
