use chrono::{DateTime, Utc};
use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};
use anyhow::Result;

use crate::helpers::common::DataRequest;
use crate::helpers::loki::QueryResponse;
use crate::helpers::queries::gather_deploy_data;

#[derive(Serialize, Debug, Clone)]
pub struct DeploymentFrequencyRecord {
  created_at: DateTime<Utc>,
  environment: String,
  state: String,
  repository: String,
  team: String
}

#[derive(Serialize, Debug)]
pub struct DeploymentFrequencyResponse {
  records: Vec<DeploymentFrequencyRecord>
}

fn trim_data(query_response: QueryResponse) -> Vec<DeploymentFrequencyRecord> {
  let records = query_response.data.result.iter().flat_map(|m| {
    m.values.iter().map(|v| {
      DeploymentFrequencyRecord {
        created_at: v.json_data.body.deployment.as_ref().unwrap().created_at,
        environment: m.stream.environment_name.clone().unwrap(),
        repository: m.stream.repository_name.clone().unwrap(),
        state: query_response.status.clone(),
        team: m.stream.team_name.clone().unwrap()
      }  
    }).collect::<Vec<_>>()
  }).collect::<Vec<_>>();

  return records;
}

pub async fn handle_request(Json(request): Json<DataRequest>) -> Result<Json<DeploymentFrequencyResponse>, StatusCode> {
  let mut response : DeploymentFrequencyResponse = DeploymentFrequencyResponse{ records: [].to_vec() };

  let query_result = gather_deploy_data(&request).await;

  match query_result {
      Ok(query_data) => {
        response.records = trim_data(query_data);

        Ok(Json(response))
      },
      Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}