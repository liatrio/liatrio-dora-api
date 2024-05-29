use std::env;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Deserializer};

use crate::helpers::error::AppError;

#[derive(Serialize, Debug)]
pub struct QueryParams {
  pub query: String,
  pub start: String,
  pub end: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntryStream {
  pub action: Option<String>,
  pub created_at: Option<DateTime<Utc>>,
  pub environment_name: Option<String>,
  pub merged_sha: Option<String>,
  pub merged_at: Option<DateTime<Utc>>,
  pub repository_name: Option<String>,
  pub service_name: Option<String>,
  pub team_name: Option<String>,
  pub topics: Option<Vec<String>>,
  pub deployment_environment: Option<String>,
  pub deployment_state: Option<String>,
}

fn deserialize_inner<'de, D>(deserializer: D) -> Result<QueryResultDataEntryValue, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntryValueBodyDeployment {
  pub created_at: Option<DateTime<Utc>>
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntryValueBody {
  pub deployment: QueryResultDataEntryValueBodyDeployment
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntryValue {
  pub body: QueryResultDataEntryValueBody
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntryValues {
  pub timestamp: String,
  #[serde(deserialize_with = "deserialize_inner")]
  pub data: QueryResultDataEntryValue
}


#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultDataEntry {
  pub stream: QueryResultDataEntryStream,
  pub values: Vec<QueryResultDataEntryValues>
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryResultData {
  #[serde(rename = "resultType")]
  pub result_type: String,
  pub result: Vec<QueryResultDataEntry>
}

#[derive(Deserialize, Debug)]
pub struct QueryResult {
  pub status: String,
  pub data: QueryResultData
}

pub async fn query(data: QueryParams) -> Result<QueryResult, AppError> {
  let client = reqwest::Client::new();
  let url = env::var("LOKI_URL")?;
  let user = env::var("LOKI_USER")?;
  let password = env::var("LOKI_TOKEN")?;

  let response = client
    .get(url)
    .query(&data)
    .basic_auth(user, Some(password))
    .send()
    .await?;
  
  let data: QueryResult = response.json().await?;

  return Ok(data);
}