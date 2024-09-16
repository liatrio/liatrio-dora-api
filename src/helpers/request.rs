use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DataRequest {
    pub repositories: Option<Vec<String>>,
    pub team: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
