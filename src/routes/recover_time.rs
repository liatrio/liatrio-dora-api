use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};
use anyhow::Result;

use crate::helpers::{
  common::DataRequest,
  queries::{gather_deploy_data, gather_issue_data
  }
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct RecoverTimeRecord {
  repository: String,
  team: String,
  failed: bool,
  created_at: DateTime<Utc>,
  fixed_at: DateTime<Utc>
}

#[derive(Serialize, Debug)]
pub struct RecoverTimeResponse {
  records: Vec<RecoverTimeRecord>
}

struct DeployEntry {
  status: bool,
  repository: String,
  team: String,
  created_at: DateTime<Utc>
}

struct IssueEntry {
  created_at: DateTime<Utc>
}

async fn organize_data(request: DataRequest) -> Result<Vec<RecoverTimeRecord>> {
  let deploy_data_task = gather_deploy_data(&request);
  let issue_data_task = gather_issue_data(&request);

  let (deploy_data_result, issue_data_result) = tokio::join!(deploy_data_task, issue_data_task);

  let deploy_data = match deploy_data_result {
    Ok(dd) => {
      let mut grouped_deploys: HashMap<String, Vec<DeployEntry>> = HashMap::new();

      for r in dd.data.result {
        if r.stream.environment_name.unwrap().to_lowercase() != "prod" {
          continue;
        }

        for b in r.values {
          let rn = r.stream.repository_name.clone().unwrap();

          let de = DeployEntry {
            status: dd.status == "success",
            repository: rn.clone(),
            team: r.stream.team_name.clone().unwrap(),
            created_at: b.json_data.body.deployment.unwrap().created_at
          };

          grouped_deploys.entry(rn.clone())
            .or_default()
            .push(de)
        }
      }

      for v in grouped_deploys.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at))
      }

      grouped_deploys
    }
    Err(e) => {
      println!("D: {e}");
      return Err(e);
    }
  };

  let issue_data = match issue_data_result {
    Ok(id) => {
      let mut grouped_issues: HashMap<String, Vec<IssueEntry>> = HashMap::new();

      for r in id.data.result {
        for b in r.values {
          let rn = b.json_data.body.repository.unwrap().name;

          let ie = IssueEntry {
            created_at: b.json_data.body.issue.unwrap().created_at
          };

          grouped_issues.entry(rn.clone())
            .or_default()
            .push(ie)
        }
      }

      for v in grouped_issues.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at))
      }

      grouped_issues
    }
    Err(e) => {
      println!("I: {e}");
      return Err(e);
    }
  };

  let mut result: Vec<RecoverTimeRecord> = [].to_vec();
  let mut on_failure = false;

  for dd in deploy_data {
    let mut record: RecoverTimeRecord = Default::default();
    
    for (idx, de) in dd.1.iter().enumerate() {
      let mut failed = false;

      if de.status == true {
        let next_deploy = if idx + 1 < dd.1.len() {
          dd.1[idx + 1].created_at
        } else {
          DateTime::<Utc>::MAX_UTC
        };

        let deploy_issue_count = match issue_data.get(&de.repository) {
          Some(ies) => {
            ies.iter().filter(|e| {
              e.created_at >= de.created_at && e.created_at < next_deploy
            }).count()
          },
          None => 0
        };

        if deploy_issue_count > 0 {
          failed = true;
        }
      } else {
        failed = true;
      }

      if failed == true && on_failure == false {
        on_failure = true;

        record = RecoverTimeRecord {
          team: de.team.clone(),
          repository: de.repository.clone(),
          created_at: de.created_at,
          failed: true,
          ..Default::default()
        };
      } else if on_failure == true && failed == false {
        on_failure = false;
        record.fixed_at = de.created_at;

        result.push(record);
        
        record = Default::default();
      } else if failed == false && on_failure == false {
        let success_record = RecoverTimeRecord {
          team: de.team.clone(),
          repository: de.repository.clone(),
          created_at: de.created_at,
          failed: false,
          ..Default::default()
        };

        result.push(success_record);
      }
    }
  }

  Ok(result)
}

pub async fn handle_request(Json(request): Json<DataRequest>) -> Result<Json<RecoverTimeResponse>, StatusCode> {
  let mut response : RecoverTimeResponse = RecoverTimeResponse {
    records: [].to_vec()
  };

  let data = organize_data(request).await;

  match data {
    Ok(d) => {
      response.records = d;

      Ok(Json(response))
    },
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}