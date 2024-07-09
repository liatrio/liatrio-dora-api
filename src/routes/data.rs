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
  queries::{gather_deploy_data, gather_issue_data, gather_merge_data},
  loki::QueryResponse,
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct Record {
  repository: String,
  team: String,
  title: String,
  user: String,
  sha: String,
  status: bool,
  post_failure: bool,
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
  created_at: DateTime<Utc>,
  closed_at: Option<DateTime<Utc>>
}

async fn sort_issue_data(data: Result<QueryResponse>) -> Result<HashMap<String, Vec<IssueEntry>>> {
  match data {
    Ok(id) => {
      let mut grouped_issues: HashMap<String, Vec<IssueEntry>> = HashMap::new();

      for result in id.data.result {
        for value in result.values {
          let rn = value.json_data.body.repository.unwrap().name;
          let issue = value.json_data.body.issue.unwrap();

          let ie = IssueEntry {
            created_at: issue.created_at,
            closed_at: issue.closed_at
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
pub struct MergeEntry {
  sha: String,
  merged_at: DateTime<Utc>,
  user: String,
  title: String,
}

fn sort_merge_data(merge_data: Result<QueryResponse>) -> HashMap<String, MergeEntry> {
  let mut records_by_sha: HashMap<String, MergeEntry> = HashMap::new();

  match merge_data {
    Ok(entry) => {
      for result in entry.data.result {
        for value in result.values {
          
          let sha = result.stream.merge_sha.as_ref().unwrap().to_string();
          let pr = value.json_data.body.pull_request.unwrap();

          let record = MergeEntry { 
            user: pr.user.login.clone(),
            title: pr.title.clone(),
            merged_at: result.stream.merged_at.unwrap().clone(),
            sha: sha.clone()
          };

          records_by_sha.entry(sha)
            .or_insert(record);
        }
      }
    }
    Err(_) => {}
  }

  return records_by_sha;
}

fn find_failures(deploy_data: &mut HashMap<String, Vec<Record>>, issue_data: &HashMap<String, Vec<IssueEntry>>) -> Vec<(String, usize, DateTime<Utc>)> {
  let mut on_failure: Option<(String, usize)> = None;
  let mut failures: Vec<(String, usize, DateTime<Utc>)> = [].to_vec();
  
  for (key, values) in deploy_data.iter_mut() {
    let len = values.len();

    for idx in 0..len {
      let mut failed = false;
      
      let next_deploy = if idx + 1 < len {
        values[idx + 1].created_at
      } else {
        DateTime::<Utc>::MAX_UTC
      };

      let deploy = &mut values[idx];

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
          deploy.post_failure = true;
          failed = true;
        }
      } else {
        failed = true;
      }

      if failed && on_failure.is_none() {
        on_failure = Some((key.to_string(), idx));
      } else if on_failure.is_some() && !failed {
        let failure = on_failure.unwrap();
        failures.push((failure.0, failure.1, deploy.created_at));
        on_failure = None;
      }
    }
  }

  return failures;
}

fn link_issues_to_deployes(deploy_data: &mut HashMap<String, Vec<Record>>, issue_data: &HashMap<String, Vec<IssueEntry>>) {
  let failures = find_failures(deploy_data, issue_data);

  for entry in failures {
    if let Some(deploy_set) = deploy_data.get_mut(&entry.0) {
      if let Some(failure_record) = deploy_set.get_mut(entry.1) {
        failure_record.fixed_at = entry.2;
      }
    }
  }
}

fn link_merge_to_deploys(deploy_by_sha: &mut HashMap<String, Vec<Record>>, merge_data_result: Result<QueryResponse>) {
  let merge_data = sort_merge_data(merge_data_result);

  for merge_entry in merge_data.iter() {
    deploy_by_sha.entry(merge_entry.0.to_string()).and_modify(|e| {
      for d in e {
        d.merged_at = merge_entry.1.merged_at;
        d.title = merge_entry.1.title.clone();
        d.user = merge_entry.1.user.clone();
      }
    }).or_default();
  }
}

async fn organize_data(request: DataRequest) -> Result<Vec<Record>> {  
  let deploy_data_task = gather_deploy_data(&request);
  let issue_data_task = gather_issue_data(&request);
  let merge_data_task = gather_merge_data(&request);
  
  let (deploy_data_result, issue_data_result, merge_data_result) = tokio::join!(deploy_data_task, issue_data_task, merge_data_task);

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

  link_merge_to_deploys(&mut deploy_by_sha, merge_data_result);

  let mut all_deploys = Vec::new();

  for deploy_set in deploy_by_sha.values() {
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