use super::{Network, Station, station_xml::StationXML};
use crate::local_prelude::StationError;
use mockall::*;

use prelude::chrono::{TimeDelta, Utc};
const BASE_URL: &str = "https://service.iris.edu/fdsnws/station/1/query";
#[derive(Debug, Default, Clone)]
pub struct FetchInfo {
    pub oldest_fetch: TimeDelta,
    pub network: String,
}
impl FetchInfo {
    pub fn oldest_fetch(mut self, time_delta: TimeDelta) -> Self {
        self.oldest_fetch = time_delta;
        self
    }
    pub fn network(mut self, network_code: String) -> Self {
        self.network = network_code;
        self
    }
}
pub type Fetcher = FetcherInternal<WebClient>;
pub struct WebClient {}
impl WebClientTrait for WebClient {
    fn new() -> Result<Self, StationError> {
        Ok(Self {})
    }

    fn fetch(&mut self, url: String) -> Result<String, StationError> {
        reqwest::blocking::get(url)?.text().map_err(|e| e.into())
    }
}
pub struct FetcherInternal<C: WebClientTrait> {
    client: C,
}
impl<C: WebClientTrait> FetcherInternal<C> {
    pub fn new() -> Result<Self, StationError> {
        Ok(Self { client: C::new()? })
    }
    pub fn with_web_client(client: C) -> Result<Self, StationError> {
        Ok(Self { client })
    }
    pub fn fetch_network(&mut self, fetch_info: &FetchInfo) -> Result<Vec<Network>, StationError> {
        let url = format!("{}?network={}", BASE_URL, fetch_info.network.clone());
        let fetch_date = Utc::now();
        let xml_string = self.client.fetch(url)?;
        let station_xml = StationXML::from_xml(xml_string.as_bytes())?;
        station_xml
            .networks
            .iter()
            .map(|network| {
                Ok(Network {
                    code: network
                        .code
                        .clone()
                        .ok_or(StationError::NetworkMissingCode)?,
                    fetch_date,
                    stations: network
                        .stations
                        .iter()
                        .map(Station::try_from)
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect::<_>()
    }
}

#[automock]
pub trait WebClientTrait: Sized {
    fn new() -> Result<Self, StationError>;
    fn fetch(&mut self, url: String) -> Result<String, StationError>;
}
#[cfg(test)]
mod test {
    use super::super::Channel;
    use super::*;
    use prelude::chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
    use pretty_assertions::assert_eq;
    use rand::rngs::Xoshiro256PlusPlus;
    use rstest::rstest;
    #[test]
    fn test_fetch_info() {
        let items = [
            (TimeDelta::hours(1), "bar".to_string()),
            (TimeDelta::hours(2), "best".to_string()),
        ];
        for (delta, network_code) in items {
            let info = FetchInfo::default()
                .oldest_fetch(delta)
                .network(network_code.clone());
            assert_eq!(info.network, network_code);
            assert_eq!(info.oldest_fetch, delta);
        }
    }
    fn build_time_string(time: DateTime<Utc>) -> String {
        let year = time.year();
        let month = time.month();
        let day = time.day();
        let hour = time.hour();
        let minute = time.minute();

        let second = time.second() as f64 + time.timestamp_subsec_micros() as f64 / 1e6;
        format!("{year}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02.4}")
    }
    fn build_channel_xml(channel: &Channel) -> String {
        let channel_code = channel.code.as_str();
        let sample_rate = channel.sample_rate.0.to_string();
        let sensor_name = channel.sensor_name.as_str();
        format!("
         <Channel code=\"{channel_code}\" locationCode=\"\" startDate=\"2020-09-23T00:00:00.0000\" restrictedStatus=\"open\">
     <Latitude>70.2043</Latitude>
     <Longitude>-161.0713</Longitude>
     <Elevation>24</Elevation>
     <Depth>2.6</Depth>
     <Azimuth>90</Azimuth>
     <Dip>0</Dip>
     <Type>GEOPHYSICAL</Type>
     <SampleRate>{sample_rate}</SampleRate>
     <ClockDrift>2E-04</ClockDrift>
     <CalibrationUnits>
      <Name>V</Name>
      <Description>emf in volts</Description>
     </CalibrationUnits>
     <Sensor>
      <Description>{sensor_name}</Description>
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
        ")
    }
    fn build_station_xml(station: &Station) -> String {
        let code = station.code.clone();
        let latitude = station.latitude.0;
        let longitude = station.longitude.0;
        let elevation = station.elevation.0;
        let start_date = build_time_string(station.start_time);
        let site_name = station.name.as_str();
        let total_number_channels = station.channels.len();
        let channel_string = station
            .channels
            .iter()
            .map(|channel| build_channel_xml(channel))
            .fold(String::new(), |acc, x| acc + "\n" + &x);
        format!("<Station code=\"{code}\" startDate=\"{start_date}\" restrictedStatus=\"open\" iris:alternateNetworkCodes=\".EARTHSCOPE,.GREG,_REALTIME,.UNRESTRICTED,_US-ALL,_US-TA,_US-TA-ADOPTED\">
    <Latitude>{latitude}</Latitude>
    <Longitude>{longitude}</Longitude>
    <Elevation>{elevation}</Elevation>
    <Site>
     <Name>{site_name}</Name>
    </Site>
    <CreationDate>2020-09-23T00:00:00.0000</CreationDate>
    <TotalNumberChannels>{total_number_channels}</TotalNumberChannels>
    <SelectedNumberChannels>1</SelectedNumberChannels>
   {channel_string}
   
   </Station>")
    }
    fn build_network_xml(network: &Network) -> String {
        let station_xml = network
            .stations
            .iter()
            .map(|station| build_station_xml(station))
            .fold(String::new(), |acc, x| acc + &x);
        format!(
            "
             <Network code=\"{}\" startDate=\"1987-01-01T00:00:00.0000\" restrictedStatus=\"open\">
   <Description>Alaska Regional Network ()</Description>
   <Identifier type=\"DOI\">10.7914/SN/AK
   </Identifier>
            {}
     </Network>
            ",
            network.code, station_xml
        )
    }
    fn build_xml(networks: Vec<Network>) -> String {
        let network_xml = networks
            .iter()
            .map(|net| build_network_xml(net))
            .fold(String::new(), |acc, x| format!("{}\n{}", acc, x));
        format!("<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>
            <FDSNStationXML xmlns=\"http://www.fdsn.org/xml/station/1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:iris=\"http://www.fdsn.org/xml/station/1/iris\" xsi:schemaLocation=\"http://www.fdsn.org/xml/station/1 http://www.fdsn.org/xml/station/fdsn-station-1.1.xsd\" schemaVersion=\"1.1\">
  <Source>IRIS-DMC</Source>
  <Sender>IRIS-DMC</Sender>
  <Module>IRIS WEB SERVICE: fdsnws-station | version: 1.1.52</Module>
  <ModuleURI>https://service.iris.edu/fdsnws/station/1/query?latitude=64&amp;longitude=-147&amp;maxradius=15&amp;network=AK&amp;nodata=404</ModuleURI>
  <Created>2026-05-29T05:51:16.9508</Created>
        {}
             </FDSNStationXML>
            
            ",network_xml)
    }
    #[rstest]
    #[case("AA", TimeDelta::days(1), 0, 0)]
    #[case("AB", TimeDelta::days(1), 1, 1)]
    #[case("AC", TimeDelta::days(1), 2, 2)]
    fn test_web_pull(
        #[case] network: String,
        #[case] oldest_fetch: TimeDelta,
        #[case] number_stations: u32,
        #[case] number_channels: u32,
    ) {
        use rand::prelude::*;
        let network_number = network.as_bytes().iter().map(|v| *v as u64).sum::<u64>();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(number_stations as u64 + network_number);
        let stations = (0..number_stations)
            .map(|_| Station {
                channels: (0..number_channels)
                    .map(|_| Channel {
                        code: rng.random::<u64>().to_string(),
                        sensor_name: rng.random::<u64>().to_string(),
                        sample_rate: rng.random_range(3.0..90.).into(),
                    })
                    .collect(),
                elevation: rng.random_range(0.0..90.).into(),
                latitude: rng.random_range(0.0..90.).into(),
                longitude: rng.random_range(0.0..90.).into(),
                name: rng.random::<u64>().to_string(),
                code: rng.random::<u64>().to_string(),
                start_time: Utc
                    .timestamp_opt(rng.random_range(1_600_000_000i64..1_800_000_000i64), 0)
                    .single()
                    .unwrap(),
            })
            .collect::<Vec<_>>();

        let mut client = MockWebClientTrait::default();

        let now = Utc::now();
        {
            let code = network.clone();
            let stations = stations.clone();
            client
                .expect_fetch()
                .with(predicate::eq(format!(
                    "{}?network={}",
                    super::BASE_URL,
                    network.clone()
                )))
                .returning(move |_| {
                    Ok(build_xml(vec![Network {
                        code: code.clone(),
                        fetch_date: Utc::now(),
                        stations: stations.clone(),
                    }]))
                });
        }

        let info = FetchInfo::default()
            .network(network.clone())
            .oldest_fetch(oldest_fetch);
        let mut fetcher = FetcherInternal::with_web_client(client).expect("failed to create");
        let networks = fetcher.fetch_network(&info).expect("failed to get");
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].code, network);
        assert!(networks[0].fetch_date - now < TimeDelta::seconds(1));
        assert_eq!(networks[0].stations, stations);
    }
}
