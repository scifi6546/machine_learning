use thiserror::Error;

#[derive(Debug, Error)]
pub enum EarthquakeDBError {
    #[error("SqliteError: {0}")]
    SqliteError(#[from] prelude::rusqlite::Error),
}
