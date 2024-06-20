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
  queries::{gather_deploy_data, gather_issue_data, gather_merge_data, gather_opened_data},
  loki::{QueryResponse, ResultItem},
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct Record {
  repository: String,
  team: String,
  title: String,
  user: String,
  sha: String,
  status: bool,
  opened_at: DateTime<Utc>,
  merged_at: DateTime<Utc>,
  created_at: DateTime<Utc>,
  fixed_at: DateTime<Utc>
}

#[derive(Serialize, Debug, Default)]
pub struct DataResponse {
  records: Vec<Record>
}

async fn sort_deploy_data(data: Result<QueryResponse>) -> Result<HashMap<String, Vec<Record>>> {
  match data {
    Ok(dd) => {
      let mut grouped_deploys: HashMap<String, Vec<Record>> = HashMap::new();

      for r in dd.data.result {
        let env = r.stream.environment_name.unwrap().to_lowercase();

        if env != "dev" {
          continue;
        }

        for b in r.values {
          let rn = r.stream.repository_name.clone().unwrap();

          let d = b.json_data.body.deployment.as_ref().unwrap();

          let record = Record {
            status: dd.status == "success",
            repository: rn.clone(),
            team: r.stream.team_name.clone().unwrap(),
            created_at: d.created_at,
            sha: d.sha.clone(),
            ..Default::default()
          };
          grouped_deploys.entry(rn.clone())
            .or_default()
            .push(record)
        }
      }

      for v in grouped_deploys.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at))
      }

      return Ok(grouped_deploys);
    }
    Err(e) => {
      return Err(e);
    }
  }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct IssueEntry {
  created_at: DateTime<Utc>
}

async fn sort_issue_data(data: Result<QueryResponse>) -> Result<HashMap<String, Vec<IssueEntry>>> {
  match data {
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

      return Ok(grouped_issues);
    }
    Err(e) => {
      return Err(e);
    }
  }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct OpenMergeEntry {
  sha: String,
  opened_at: DateTime<Utc>,
  merged_at: DateTime<Utc>,
  user: String,
  title: String,
}

fn fill_open_merge(record: &mut OpenMergeEntry, entry: &ResultItem) {
  match &entry.stream.merged_at {
    Some(ma) => {
      record.merged_at = ma.clone();
      record.sha = entry.stream.merge_sha.as_ref().unwrap().to_string();
    }
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

fn sort_open_merge_data(merge_data: Result<QueryResponse>, open_data: Result<QueryResponse>) -> HashMap<String, OpenMergeEntry> {
  let mut grouped_prs_by_nbr: HashMap<u32, OpenMergeEntry> = HashMap::new();
  let mut grouped_prs_by_sha: HashMap<String, OpenMergeEntry> = HashMap::new();

  for a in &[merge_data, open_data] {
    for s in a {
      for t in &s.data.result {
        for v in &t.values {
          grouped_prs_by_nbr.entry(v.json_data.body.number.unwrap()).and_modify(|e| {
            fill_open_merge(e, t);
          }).or_insert_with(|| {
            let mut record: OpenMergeEntry = Default::default();

            fill_open_merge(&mut record, t);

            record
          });
        }
      }
    }
  }

  grouped_prs_by_nbr.into_values()
    .filter(|f| f.sha != "".to_string())
    .for_each(|m| {
      grouped_prs_by_sha.entry(m.sha.clone()).or_insert(m);
    });

  return grouped_prs_by_sha;
}

fn link_issues_to_deployes(deploy_data: &mut HashMap<String, Vec<Record>>, issue_data: &HashMap<String, Vec<IssueEntry>>) {
  let mut on_failure = false;

  for deploy_set in deploy_data.values_mut() {
    let len = deploy_set.len();

    for idx in 0..len {
      let mut failed = false;
      
      let next_deploy = if idx + 1 < len {
        deploy_set[idx + 1].created_at
      } else {
        DateTime::<Utc>::MAX_UTC
      };

      let deploy = &mut deploy_set[idx];

      if deploy.status {
        let deploy_issue_count = match issue_data.get(&deploy.repository) {
          Some(ies) => {
            ies.iter().filter(|e| {
              e.created_at >= deploy.created_at && e.created_at < next_deploy
            }).count()
          },
          None => 0
        };

        if deploy_issue_count > 0 {
          deploy.status = false;
          failed = true;
        }
      } else {
        failed = true;
      }

      if failed && !on_failure {
        on_failure = true;
      } else if on_failure && !failed {
        on_failure = false;
        deploy.fixed_at = deploy.created_at;
      }
    }
  }
}

fn link_open_and_merge_to_deploys(deploy_by_sha: &mut HashMap<String, Vec<Record>>, open_data_result: Result<QueryResponse>, merge_data_result: Result<QueryResponse>) {
  let open_merge_data = sort_open_merge_data(merge_data_result, open_data_result);

  for omd in open_merge_data.iter() {
    deploy_by_sha.entry(omd.0.to_string()).and_modify(|e| {
      for d in e {
        d.opened_at = omd.1.opened_at;
        d.merged_at = omd.1.merged_at;
        d.title = omd.1.title.clone();
        d.user = omd.1.user.clone();
      }
    }).or_default();
  }
}

async fn organize_data(request: DataRequest) -> Result<Vec<Record>> {  
  let deploy_data_task = gather_deploy_data(&request);
  let issue_data_task = gather_issue_data(&request);
  let merge_data_task = gather_merge_data(&request);
  let open_data_task = gather_opened_data(&request);
  
  let (deploy_data_result, issue_data_result, merge_data_result, open_data_result) = tokio::join!(deploy_data_task, issue_data_task, merge_data_task, open_data_task);

  let mut deploy_data = sort_deploy_data(deploy_data_result).await?;
  let issue_data = sort_issue_data(issue_data_result).await?;

  link_issues_to_deployes(&mut deploy_data, &issue_data);  

  let mut deploy_by_sha: HashMap<String, Vec<Record>> = HashMap::new();

  for deploy_set in deploy_data.values() {
    for deploy in deploy_set {
      deploy_by_sha
        .entry(deploy.sha.clone())
        .or_insert_with(Vec::new)
        .push(deploy.clone());
    }
  }

  link_open_and_merge_to_deploys(&mut deploy_by_sha, open_data_result, merge_data_result);

  let mut all_deploys = Vec::new();

  for deploy_set in deploy_data.values() {
      all_deploys.extend(deploy_set.clone());
  }

  Ok(all_deploys)
}

pub async fn handle_request(Json(request): Json<DataRequest>) -> Result<Json<DataResponse>, StatusCode> {
  let mut response : DataResponse = Default::default();

  let data = organize_data(request).await;

  match data {
    Ok(d) => {
      response.records = d;

      Ok(Json(response))
    },
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}