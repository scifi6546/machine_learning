use crate::local_prelude::StationError;
use prelude::chrono::{DateTime, Utc};
use quick_xml::{Reader as XMLReader, events::Event as XMLEvent};
use std::io::{BufRead, Read};
#[derive(Clone, PartialEq, Debug)]
pub struct Sensor {
    pub description: Option<String>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Unit {
    pub name: Option<String>,
    pub description: Option<String>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct InstrumentSensitivity {
    pub value: Option<f64>,
    pub frequency: Option<f64>,
    pub input_unit: Option<Unit>,
    pub output_unit: Option<Unit>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Response {
    pub instrument_sensitivity: Option<InstrumentSensitivity>,
}
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Channel {
    pub code: Option<String>,
    pub location_code: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: Option<f64>,
    pub azimuth: Option<f64>,
    pub dip: Option<f64>,
    pub sample_rate: Option<f64>,
    pub clock_drift: Option<f64>,
    pub calibration_unit: Option<Unit>,
    pub sensor: Option<Sensor>,
    pub response: Option<Response>,
}
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Station {
    pub code: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub site_name: Option<String>,
    pub channels: Vec<Channel>,
    pub creation_date: Option<DateTime<Utc>>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Network {
    pub code: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub restricted_status: Option<String>,
    pub stations: Vec<Station>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StationXML {
    pub source: String,
    pub sender: String,
    pub module: String,
    pub module_uri: String,
    pub creation_date: DateTime<Utc>,
    pub networks: Vec<Network>,
}
impl StationXML {
    pub fn from_xml<R: Read + BufRead>(r: R) -> Result<Self, StationError> {
        use super::sub_state::{EndEvent, FDSNStationXMLState, SubState};

        #[derive(Clone, PartialEq, Debug)]
        enum State {
            Initial,
            InXML,
            FDSNStationXML(FDSNStationXMLState),
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
                    XMLEvent::Decl(_decl) => {
                        state = match state {
                            State::Initial => State::InXML,
                            _ => return Err(StationError::XMLStructureError),
                        };
                    }
                    XMLEvent::Text(v) => {
                        let text_string = String::from_utf8(v.to_vec())?;
                        match &state {
                            State::InXML => {}
                            State::FDSNStationXML(state) => {
                                state.xml_text_event(&text_string, &mut output)?;
                            }
                            State::Initial => {}
                        }
                    }
                    XMLEvent::Start(v) => {
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
                                State::FDSNStationXML(FDSNStationXMLState::Initial)
                            }
                            State::FDSNStationXML(state) => {
                                State::FDSNStationXML(state.xml_start_event(
                                    &tag_lowercase,
                                    &start_string,
                                    &mut output,
                                )?)
                            }
                        };
                    }
                    XMLEvent::End(v) => {
                        state = match state {
                            State::FDSNStationXML(state) => match state.xml_end_event()? {
                                EndEvent::Backtrack => State::InXML,
                                EndEvent::Continue(state) => State::FDSNStationXML(state),
                            },
                            _ => {
                                todo!("end: {:#?}, state: {:#?}", v, state)
                            }
                        };
                    }
                    _ => {}
                },
            }
        }

        Ok(output)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use prelude::chrono::{TimeDelta, TimeZone, Utc};
    use pretty_assertions::assert_eq;
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
    <Channel code="BHE" locationCode="Foo" startDate="2020-09-23T00:00:00.0000" restrictedStatus="open">
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
        let expected_xml = StationXML {
            source: "IRIS-DMC".to_string(),
            sender: "IRIS-DMC".to_string(),
            module: "IRIS WEB SERVICE: fdsnws-station | version: 1.1.52".to_string(),
            module_uri: "test".to_string(),
            creation_date: Utc.with_ymd_and_hms(2026, 05, 29, 5, 51, 16).unwrap()
                + TimeDelta::milliseconds(950),
            networks: vec![Network {
                code: Some("AK".to_string()),
                start_date: Some(Utc.with_ymd_and_hms(1987, 01, 01, 0, 0, 0).unwrap()),
                restricted_status: Some("open".to_string()),
                stations: vec![Station {
                    code: Some("A19K".to_string()),
                    start_date: Some(Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap()),
                    end_date: None,
                    latitude: Some(70.2043),
                    longitude: Some(-161.0713),
                    elevation: Some(24.0),
                    site_name: Some("Wainwright, AK, USA".to_string()),
                    creation_date: Some(Utc.with_ymd_and_hms(2020, 09, 23, 0, 0, 0).unwrap()),
                    channels: vec![Channel {
                        code: Some("BHE".to_string()),
                        location_code: Some("Foo".to_string()),
                        start_date: Some(Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap()),
                        latitude: Some(70.2043),
                        longitude: Some(-161.0713),
                        elevation: Some(24.),
                        depth: Some(2.6),
                        azimuth: Some(90.),
                        dip: Some(0.),
                        sample_rate: Some(50.),
                        clock_drift: Some(2.0e-4),
                        calibration_unit: Some(Unit {
                            name: Some("V".to_string()),
                            description: Some("emf in volts".to_string()),
                        }),
                        sensor: Some(Sensor {
                            description: Some(
                                "Streckeisen STS-5A/Quanterra 330 Linear Phase Belo".to_string(),
                            ),
                        }),
                        response: Some(Response {
                            instrument_sensitivity: Some(InstrumentSensitivity {
                                value: Some(6.28316E8),
                                frequency: Some(0.2),
                                input_unit: Some(Unit {
                                    name: Some("m/s".to_string()),
                                    description: Some("velocity in meters per second".to_string()),
                                }),
                                output_unit: Some(Unit {
                                    name: Some("counts".to_string()),
                                    description: Some("digital counts".to_string()),
                                }),
                            }),
                        }),
                    }],
                }],
            }],
        };
        let xml = StationXML::from_xml(Cursor::new(xml_str)).unwrap();
        assert_eq!(xml, expected_xml);
    }
    #[test]
    fn station_no_location_code() {
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
        let expected_xml = StationXML {
            source: "IRIS-DMC".to_string(),
            sender: "IRIS-DMC".to_string(),
            module: "IRIS WEB SERVICE: fdsnws-station | version: 1.1.52".to_string(),
            module_uri: "test".to_string(),
            creation_date: Utc.with_ymd_and_hms(2026, 05, 29, 5, 51, 16).unwrap()
                + TimeDelta::milliseconds(950),
            networks: vec![Network {
                code: Some("AK".to_string()),
                start_date: Some(Utc.with_ymd_and_hms(1987, 01, 01, 0, 0, 0).unwrap()),
                restricted_status: Some("open".to_string()),
                stations: vec![Station {
                    code: Some("A19K".to_string()),
                    start_date: Some(Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap()),
                    end_date: None,
                    latitude: Some(70.2043),
                    longitude: Some(-161.0713),
                    elevation: Some(24.0),
                    site_name: Some("Wainwright, AK, USA".to_string()),
                    creation_date: Some(Utc.with_ymd_and_hms(2020, 09, 23, 0, 0, 0).unwrap()),
                    channels: vec![Channel {
                        code: Some("BHE".to_string()),
                        location_code: None,
                        start_date: Some(Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0).unwrap()),
                        latitude: Some(70.2043),
                        longitude: Some(-161.0713),
                        elevation: Some(24.),
                        depth: Some(2.6),
                        azimuth: Some(90.),
                        dip: Some(0.),
                        sample_rate: Some(50.),
                        clock_drift: Some(2.0e-4),
                        calibration_unit: Some(Unit {
                            name: Some("V".to_string()),
                            description: Some("emf in volts".to_string()),
                        }),
                        sensor: Some(Sensor {
                            description: Some(
                                "Streckeisen STS-5A/Quanterra 330 Linear Phase Belo".to_string(),
                            ),
                        }),
                        response: Some(Response {
                            instrument_sensitivity: Some(InstrumentSensitivity {
                                value: Some(6.28316E8),
                                frequency: Some(0.2),
                                input_unit: Some(Unit {
                                    name: Some("m/s".to_string()),
                                    description: Some("velocity in meters per second".to_string()),
                                }),
                                output_unit: Some(Unit {
                                    name: Some("counts".to_string()),
                                    description: Some("digital counts".to_string()),
                                }),
                            }),
                        }),
                    }],
                }],
            }],
        };
        let xml = StationXML::from_xml(Cursor::new(xml_str)).unwrap();
        assert_eq!(xml, expected_xml);
    }
    #[test]
    fn large_scale() {
        let large_xml = include_str!("../../stations.xml");
        let _stations = StationXML::from_xml(Cursor::new(large_xml)).unwrap();
    }
}
