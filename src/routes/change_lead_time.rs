use chrono::{DateTime, Utc};
use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};
use anyhow::Result;
use std::str::FromStr;
use std::collections::HashMap;


use crate::helpers::{
  common::DataRequest, queries::{gather_deploy_data, gather_merge_data}
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct ChangeLeadTimeRecord {
  merged_at: DateTime<Utc>,
  deployed_at: DateTime<Utc>,
  repository: String,
  team: String,
  title: String,
  user: String,
  sha: String,
}

#[derive(Serialize, Debug)]
pub struct ChangeLeadTimeResponse {
  records: Vec<ChangeLeadTimeRecord>
}

async fn organize_data(request: DataRequest) -> Result<Vec<ChangeLeadTimeRecord>> {
  let deploy_data_task = gather_deploy_data(&request);
  let merge_data_task = gather_merge_data(&request);

  let (deploy_data_result, merge_data_result) = tokio::join!(deploy_data_task, merge_data_task);

  let mut records_by_sha: HashMap<String, ChangeLeadTimeRecord> = HashMap::new();

  match merge_data_result {
    Ok(entry) => {
      for result in entry.data.result {
        for value in result.values {
          
          let sha = result.stream.merge_sha.as_ref().unwrap().to_string();
          let pr = value.json_data.body.pull_request.unwrap();

          let record = ChangeLeadTimeRecord { 
            user: pr.user.login.clone(),
            title: pr.title.clone(),
            merged_at: result.stream.merged_at.unwrap().clone(),
            sha: sha.clone(),
            ..Default::default()
          };

          records_by_sha.entry(sha)
            .or_insert(record);
        }
      }
    }
    Err(e) => {
      println!("I: {e}");
      return Err(e);
    }
  }

  let min_utc_datetime = DateTime::<Utc>::from_str("1970-01-01T00:00:00Z").unwrap();

  for query_result in &[deploy_data_result] {
    for entry in query_result {
      for result in &entry.data.result {
        for value in &result.values {
          records_by_sha.entry(value.json_data.body.deployment.as_ref().unwrap().sha.clone()).and_modify(|e| {
            match result.stream.environment_name.as_deref() {
              //this should really look at prod...
              Some("dev") => e.deployed_at = value.json_data.body.deployment.as_ref().unwrap().created_at,
              _ => {}
            }
          });
        }
      }
    }
  }

  let records = records_by_sha.into_values()
    .filter(|f| f.deployed_at != min_utc_datetime)
    .collect();

  Ok(records)
}

pub async fn handle_request(Json(request): Json<DataRequest>) -> Result<Json<ChangeLeadTimeResponse>, StatusCode> {
  let mut response : ChangeLeadTimeResponse = ChangeLeadTimeResponse{ records: [].to_vec() };

  let data = organize_data(request).await;

  match data {
      Ok(d) => {
        response.records = d;

        Ok(Json(response))
      },
      Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}