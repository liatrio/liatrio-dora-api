use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Default)]
pub struct ResponseRecord {
  pub repository: String,
  pub team: String, 
  pub title: Option<String>,
  pub user: Option<String>,
  pub sha: String,
  pub status: bool,
  pub failed_at: Option<DateTime<Utc>>,
  pub merged_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub fixed_at: Option<DateTime<Utc>>,
  pub fixed_url: Option<String>,
  pub deploy_url: String,
  pub issue_url: Option<String>,
  pub change_url: String,
  pub total_cycle_time: Option<f32>
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct TeamsResponse {
  pub teams: Vec<String>
}