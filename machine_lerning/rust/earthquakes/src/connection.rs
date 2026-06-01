use super::{Event, error::EarthquakeDBError, query::EventQuery};
use prelude::rusqlite;
use prelude::{
    chrono::DateTime,
    rusqlite::{ToSql, params_from_iter},
};
pub struct DatabaseConnection {
    connection: rusqlite::Connection,
}
impl DatabaseConnection {
    pub fn query(&self, query: EventQuery) -> Result<Vec<Event>, EarthquakeDBError> {
        let mut params = Vec::<&dyn ToSql>::new();
        let mut base_query = "
            SELECT 
                aecevent.eventname,
                origin.lat,
                origin.lon,
                origin.time,
                netmag.magnitude, 
                netmag.magtype 
            FROM aecevent 
            JOIN event ON aecevent.evid = event.evid 
            JOIN origin ON event.prefor = origin.orid
            JOIN netmag ON origin.mlid = netmag.magid
            "
        .to_string();
        let mut number_parameters_added = 0;
        if let Some(event_name) = query.event_name.as_ref() {
            let event_query = format!(
                "WHERE aecevent.eventname = ?{}",
                number_parameters_added + 1
            );
            base_query = base_query.to_string() + &event_query;
            params.push(event_name);
            number_parameters_added += 1;
        }
        if let Some(max_events) = query.maximum_events_returned.as_ref() {
            let limit_query = format!(" LIMIT ?{}", number_parameters_added + 1);

            base_query = base_query.to_string() + &limit_query;
            params.push(max_events);
            number_parameters_added += 1;
        }
        println!("{}", base_query);
        let mut statement = self.connection.prepare(&base_query)?;
        let events_result = {
            let params_type_cast: &[&dyn ToSql] = &params;
            statement.query_map(params_type_cast, |row| {
                let time_f64: f64 = row.get(3).unwrap();
                Ok(Event {
                    event_name: row.get(0).unwrap(),
                    latitude: row.get(1).unwrap(),
                    longitude: row.get(2).unwrap(),
                    time: DateTime::from_timestamp_nanos((time_f64 * 1_000_000_000.) as i64),
                    magnitude: row.get(4).unwrap(),
                    magnitude_type: row.get(5).unwrap(),
                })
            })
        }?;
        let mut events = Vec::new();
        for event in events_result {
            match event {
                Ok(e) => events.push(e),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(events)
    }
}
pub struct ConnectionBuilder {
    pub database_path: String,
}
impl ConnectionBuilder {
    pub const DEFAULT_DATABASE_PATH: &'static str = "../all_events.db";
    pub fn with_database_path(mut self, database_path: String) -> Self {
        self.database_path = database_path;
        self
    }
    pub fn connect(&self) -> Result<DatabaseConnection, EarthquakeDBError> {
        let connection = rusqlite::Connection::open(&self.database_path)?;
        Ok(DatabaseConnection { connection })
    }
}
impl Default for ConnectionBuilder {
    fn default() -> Self {
        Self {
            database_path: Self::DEFAULT_DATABASE_PATH.to_string(),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn db_connection_default_path() {
        let builder = ConnectionBuilder::default();
        assert_eq!(
            builder.database_path,
            ConnectionBuilder::DEFAULT_DATABASE_PATH
        );
    }
    #[test]
    fn update_db_path() {
        let new_path = "./foo.db";
        let builder = ConnectionBuilder::default().with_database_path(new_path.to_string());
        assert_eq!(builder.database_path, new_path);
    }
    #[test]
    fn query_number_earthquakes() {
        let num_events = 100;
        let connection = ConnectionBuilder::default().connect().unwrap();
        let events = connection
            .query(EventQuery::default().with_maximum_events_returned(num_events))
            .unwrap();
        assert_eq!(events.len() as i64, num_events);
    }
    #[test]
    fn query_event_name() {
        let connection = ConnectionBuilder::default().connect().unwrap();
        let query = EventQuery::default()
            .with_maximum_events_returned(100)
            .with_event_name("0151odwn3".to_string());
        let events = connection.query(query).unwrap();
        assert_eq!(events.len(), 1);
    }
}
