use super::{FDSNStationXML, StationError, local_prelude::parse_to_date_time};
pub enum EndEvent<T: Clone + Copy + PartialEq> {
    Backtrack,
    Continue(T),
}
pub trait SubState: Sized + Copy + Clone + PartialEq {
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError>;
    fn xml_text_event(
        &self,
        _text: &str,
        xml: FDSNStationXML,
    ) -> Result<FDSNStationXML, StationError> {
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
impl SubState for UnitsSubState {
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match tag_lowercase {
            "description" => Ok(Self::Description),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
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
            Self::InputUnits(state) => Ok(Self::InputUnits(state.xml_start_event(tag_lowercase)?)),
            Self::OutputUnits(state) => {
                Ok(Self::OutputUnits(state.xml_start_event(tag_lowercase)?))
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "instrumentsensitivity" => Ok(Self::InstrumentSensitivity(
                    InstrumentSensitivityState::Initial,
                )),
                _ => panic!("invalid response sub tag: {}", tag_lowercase),
            },
            Self::InstrumentSensitivity(state) => Ok(ResponseSubState::InstrumentSensitivity(
                state.xml_start_event(tag_lowercase)?,
            )),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
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
                "calibrationunits" => Ok(Self::CalibrationUnits(UnitsSubState::Initial)),
                "sensor" => Ok(Self::Sensor(SensorSubState::Initial)),
                "response" => Ok(Self::Response(ResponseSubState::Initial)),
                _ => todo!("channel substate initial tag: {}", tag_lowercase),
            },
            Self::Latitude => {
                return Err(StationError::XMLStructureError);
            }
            Self::Longitude => {
                return Err(StationError::XMLStructureError);
            }
            Self::Elevation => {
                return Err(StationError::XMLStructureError);
            }
            Self::Depth => {
                return Err(StationError::XMLStructureError);
            }
            Self::Azimuth => {
                return Err(StationError::XMLStructureError);
            }
            Self::Dip => {
                return Err(StationError::XMLStructureError);
            }
            Self::Type => {
                return Err(StationError::XMLStructureError);
            }
            Self::SampleRate => {
                return Err(StationError::XMLStructureError);
            }
            Self::ClockDrift => {
                return Err(StationError::XMLStructureError);
            }
            Self::CalibrationUnits(unit_state) => Ok(Self::CalibrationUnits(
                unit_state.xml_start_event(tag_lowercase)?,
            )),
            Self::Sensor(state) => Ok(Self::Sensor(state.xml_start_event(tag_lowercase)?)),
            Self::Response(state) => Ok(Self::Response(state.xml_start_event(tag_lowercase)?)),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "name" => Ok(Self::Name),
                _ => todo!("site tag: {}", tag_lowercase),
            },
            SiteSubState::Name => Err(StationError::XMLStructureError),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "latitude" => Ok(Self::Latitude),
                "longitude" => Ok(Self::Longitude),
                "elevation" => Ok(Self::Elevation),
                "site" => Ok(Self::Site(SiteSubState::Initial)),
                "creationdate" => Ok(Self::CreationDate),
                "totalnumberchannels" => Ok(Self::TotalNumberChannels),
                "selectednumberchannels" => Ok(Self::SelectedNumberChannels),
                "channel" => Ok(Self::Channel(ChannelSubState::Initial)),
                _ => todo!("station tag: {}", tag_lowercase),
            },
            Self::Latitude => Err(StationError::XMLStructureError),
            Self::Longitude => Err(StationError::XMLStructureError),
            Self::Elevation => Err(StationError::XMLStructureError),
            Self::Site(state) => Ok(Self::Site(state.xml_start_event(tag_lowercase)?)),
            Self::CreationDate => Err(StationError::XMLStructureError),
            Self::TotalNumberChannels => Err(StationError::XMLStructureError),
            Self::SelectedNumberChannels => Err(StationError::XMLStructureError),
            Self::Channel(state) => Ok(Self::Channel(state.xml_start_event(tag_lowercase)?)),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "description" => Ok(Self::Description),
                "identifier" => Ok(Self::Identifier),
                "totalnumberstations" => Ok(Self::TotalNumberStations),
                "selectednumberstations" => Ok(Self::SelectedNumberStations),
                "station" => Ok(Self::Station(StationSubState::Initial)),
                _ => {
                    todo!("invalid tag: {}", tag_lowercase)
                }
            },
            Self::Description => Err(StationError::XMLStructureError),
            Self::Identifier => Err(StationError::XMLStructureError),
            Self::TotalNumberStations => Err(StationError::XMLStructureError),
            Self::SelectedNumberStations => Err(StationError::XMLStructureError),
            Self::Station(state) => Ok(Self::Station(state.xml_start_event(tag_lowercase)?)),
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
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError> {
        match self {
            Self::Initial => match tag_lowercase {
                "source" => Ok(Self::Source),
                "sender" => Ok(Self::Sender),
                "module" => Ok(Self::Module),
                "moduleuri" => Ok(Self::ModuleUri),
                "created" => Ok(Self::Created),
                "network" => Ok(Self::Network(NetworkSubState::Initial)),
                _ => {
                    todo!("tag not implemented: \"{}\"", tag_lowercase)
                }
            },
            Self::Network(state) => Ok(Self::Network(state.xml_start_event(tag_lowercase)?)),
            Self::Source => Err(StationError::XMLStructureError),
            Self::Sender => Err(StationError::XMLStructureError),
            Self::Module => Err(StationError::XMLStructureError),
            Self::ModuleUri => Err(StationError::XMLStructureError),
            Self::Created => Err(StationError::XMLStructureError),
        }
    }
    fn xml_text_event(
        &self,
        text: &str,
        mut xml: FDSNStationXML,
    ) -> Result<FDSNStationXML, StationError> {
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
