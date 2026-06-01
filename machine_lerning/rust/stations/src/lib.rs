use quick_xml::{Reader as XMLReader, events::Event as XMLEvent};
use std::io::{BufRead, Read};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum StationError {
    #[error("Failed to read XML: {0}")]
    XMLError(#[from] quick_xml::Error),
    #[error("failed to parse utf8 text: {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("invalid XML structure")]
    XMLStructureError,
}
#[derive(Clone, PartialEq, Debug)]
pub struct FDSNStationXML {
    pub source: String,
    pub sender: String,
    pub module: String,
}
impl FDSNStationXML {
    pub fn from_xml<R: Read + BufRead>(r: R) -> Result<Self, StationError> {
        #[derive(Clone, PartialEq, Debug)]
        enum State {
            Initial,
            InXML,
            FDSNStationXML,
            Source,
        }
        let mut output = Self {
            sender: String::new(),
            source: String::new(),
            module: String::new(),
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
                        match state {
                            State::InXML => {}
                            State::FDSNStationXML => {}
                            State::Initial => {}
                            State::Source => output.source = text_string,
                        }
                    }
                    XMLEvent::Start(v) => {
                        let start_string = String::from_utf8(v.to_vec())?;
                        state = match state {
                            State::Initial => return Err(StationError::XMLStructureError),
                            State::InXML => {
                                let first = start_string.split_whitespace().next();
                                if start_string.split_whitespace().next() != Some("FDSNStationXML")
                                {
                                    return Err(StationError::XMLStructureError);
                                }
                                State::FDSNStationXML
                            }
                            State::FDSNStationXML => {
                                let lowercase_start_string = start_string.to_lowercase();
                                match lowercase_start_string.as_str() {
                                    "source" => State::Source,
                                    _ => todo!("other: {}", lowercase_start_string),
                                }
                            }

                            State::Source => return Err(StationError::XMLStructureError),
                        };
                    }
                    XMLEvent::End(v) => {
                        todo!("end: {:#?}", v)
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
    use std::io::Cursor;
    #[test]
    fn basic_station() {
        let xml_str = r#"
<?xml version="1.0" encoding="ISO-8859-1"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:iris="http://www.fdsn.org/xml/station/1/iris" xsi:schemaLocation="http://www.fdsn.org/xml/station/1 http://www.fdsn.org/xml/station/fdsn-station-1.1.xsd" schemaVersion="1.1">
<Source>IRIS-DMC</Source>
<Sender>IRIS-DMC</Sender>
<Module>IRIS WEB SERVICE: fdsnws-station | version: 1.1.52</Module>
<ModuleURI>https://service.iris.edu/fdsnws/station/1/query?latitude=64&amp;longitude=-147&amp;maxradius=15&amp;network=AK&amp;nodata=404</ModuleURI>
<Created>2026-05-29T05:51:16.9508</Created>
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
    <TotalNumberChannels>53</TotalNumberChannels>
    <SelectedNumberChannels>0</SelectedNumberChannels>
   </Station>
     </Network>
 </FDSNStationXML>
    "#;
        let expected_xml = FDSNStationXML {
            source: "IRIS-DMC".to_string(),
            sender: "IRIS-DMC".to_string(),
            module: "IRIS WEB SERVICE: fdsnws-station | version: 1.1.52".to_string(),
        };
        let xml = FDSNStationXML::from_xml(Cursor::new(xml_str)).unwrap();
        assert_eq!(xml, expected_xml);
    }

    use super::*;
}
