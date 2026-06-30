use super::StationError;
pub enum EndEvent<T: Clone + Copy + PartialEq> {
    Backtrack,
    Continue(T),
}
pub trait SubState: Sized + Copy + Clone + PartialEq {
    fn xml_start_event(self, tag_lowercase: &str) -> Result<Self, StationError>;
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
