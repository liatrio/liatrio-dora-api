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
  common::DataRequest, loki::ResultItem, queries::{gather_deploy_data, gather_merge_data, gather_opened_data}
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct ChangeLeadTimeRecord {
  opened_at: DateTime<Utc>,
  merged_at: DateTime<Utc>,
  dev_deployed_at: DateTime<Utc>,
  test_deployed_at: Option<DateTime<Utc>>,
  prod_deployed_at: Option<DateTime<Utc>>,
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

fn fill_record_base(record: &mut ChangeLeadTimeRecord, entry: &ResultItem) {
  match &entry.stream.merged_at {
    Some(ma) => {
      record.merged_at = ma.clone();
      record.sha = entry.stream.merge_sha.as_ref().unwrap().to_string();
    }
    None => {}
  }

  match &entry.stream.repository_name {
    Some(rn) => record.repository = rn.clone(),
    None => {}
  }

  match &entry.stream.team_name {
    Some(tn) => record.team = tn.clone(),
    None => {}
  }

  match entry.stream.created_at {
    Some(ca) => record.opened_at = ca,
    None => {}
  }

  match &entry.values[0].json_data.body.pull_request {
    Some(pr) => {
      record.user = pr.user.login.clone();
      record.title = pr.title.clone();
    },
    None => {}
  }
}

async fn organize_data(request: DataRequest) -> Result<Vec<ChangeLeadTimeRecord>> {
  let deploy_data_task = gather_deploy_data(&request);
  let merge_data_task = gather_merge_data(&request);
  let open_data_task = gather_opened_data(&request);

  let (deploy_data, merge_data, open_data) = tokio::join!(deploy_data_task, merge_data_task, open_data_task);

  let mut grouped_prs_by_nbr: HashMap<u32, ChangeLeadTimeRecord> = HashMap::new();
  let mut grouped_prs_by_sha: HashMap<String, ChangeLeadTimeRecord> = HashMap::new();

  for a in &[merge_data, open_data] {
    for s in a {
      for t in &s.data.result {
        for v in &t.values {
          grouped_prs_by_nbr.entry(v.json_data.body.number.unwrap()).and_modify(|e| {
            fill_record_base(e, t);
          }).or_insert_with(|| {
            let mut record = ChangeLeadTimeRecord { ..Default::default() };

            fill_record_base(&mut record, t);

            record
          });
        }
      }
    }
  }

  let min_utc_datetime = DateTime::<Utc>::from_str("1970-01-01T00:00:00Z").unwrap();

  grouped_prs_by_nbr.into_values()
    .filter(|f| f.sha != "".to_string())
    .for_each(|m| {
      grouped_prs_by_sha.entry(m.sha.clone()).or_insert(m);
    });

  for a in &[deploy_data] {
    for s in a {
      for t in &s.data.result {
        for v in &t.values {
          grouped_prs_by_sha.entry(v.json_data.body.deployment.as_ref().unwrap().sha.clone()).and_modify(|e| {
            match t.stream.environment_name.as_deref() {
              Some("dev") => e.dev_deployed_at = v.json_data.body.deployment.as_ref().unwrap().created_at,
              Some("test") => e.test_deployed_at = Some(v.json_data.body.deployment.as_ref().unwrap().created_at),
              Some("prod") => e.prod_deployed_at = Some(v.json_data.body.deployment.as_ref().unwrap().created_at),
              _ => {}
            }
          });
        }
      }
    }
  }
  
  let records = grouped_prs_by_sha.into_values()
    .filter(|f| f.dev_deployed_at != min_utc_datetime)
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