use super::structs::Nstime;
/// Length of source ID string
pub const LM_SIDLEN: usize = 64;
pub const NULL: usize = 0;
///  Special nstime_t value meaning "unset". The time value corresponds to '1902-01-01T00:00:00.000000001Z'.
pub const NSTUNSET: Nstime = -2145916799999999999;
pub const MSTRACEID_SKIPLIST_HEIGHT: usize = 8;
pub const MAX_LOG_MSG_LENGTH: usize = 200;
