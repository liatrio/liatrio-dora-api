use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use axum::{
  http::StatusCode,
  response::Json
};

use crate::helpers::loki::{query, QueryParams};

#[derive(Deserialize, Debug)]
pub struct DeploymentFrequencyRequest {
  repositories: Option<Vec<String>>,
  team: Option<String>,
  start: DateTime<Utc>,
  end: DateTime<Utc>
}

#[derive(Serialize, Debug, Clone)]
pub struct DeploymentFrequencyRecord {
  created_at: DateTime<Utc>,
  environment: String,
  state: String,
  repository: String,
  team: String
}

#[derive(Serialize, Debug, Clone)]
pub struct DeploymentFrequencyResponse {
  records: Vec<DeploymentFrequencyRecord>
}

pub async fn handle_request(Json(data): Json<DeploymentFrequencyRequest>) -> Result<Json<DeploymentFrequencyResponse>, StatusCode> {
  let mut response : DeploymentFrequencyResponse = DeploymentFrequencyResponse{ records: [].to_vec() };

  let team_query = match data.team {
    Some(t) => format!(r#",team_name="{}""#, t),
    None => "".to_string()
  };

  let repo_query = match data.repositories {
    Some(r) => format!(r#",repository_name="{}""#, r.join("|")),
    None => "".to_string()
  };

  let params = QueryParams {
    start: data.start.timestamp_nanos_opt().unwrap().to_string(),
    end: data.end.timestamp_nanos_opt().unwrap().to_string(),
    query: format!(r#"{{deployment_state=~"success|failure"{}{}}}"#, team_query, repo_query)
  };

  let query_result = query(params).await;

  match query_result {
      Ok(query_data) => {
        response.records = query_data.data.result.iter().flat_map(|m| {
          m.values.iter().map(|v| {
            DeploymentFrequencyRecord {
              created_at: v.data.body.deployment.created_at.unwrap(),
              environment: m.stream.environment_name.clone().unwrap(),
              repository: m.stream.repository_name.clone().unwrap(),
              state: query_data.status.clone(),
              team: m.stream.team_name.clone().unwrap()
            }  
          }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        Ok(Json(response))
      },
      Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}