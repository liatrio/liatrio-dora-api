use std::{collections::HashMap, sync::Arc};
use chrono::{DateTime, Duration, TimeDelta, Utc};
use serde::Serialize;
use axum::{
  extract::Extension, http::StatusCode, response::Json
};
use dashmap::DashMap;
use anyhow::Result;

use crate::helpers::{
  common::DataRequest,
  queries::{gather_deploy_data, gather_issue_data, gather_merge_data},
  loki::QueryResponse,
};

pub type DataCache = Arc<DashMap<String, DataResponse>>;

#[derive(Serialize, Debug, Clone, Default)]
pub struct Record {
  repository: String,
  team: String, 
  title: Option<String>,
  user: Option<String>,
  sha: String,
  status: bool,
  failed_at: Option<DateTime<Utc>>,
  merged_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  fixed_at: Option<DateTime<Utc>>,
  deploy_url: String,
  fixed_url: Option<String>,
  change_url: String,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct DataResponse {
  records: Vec<Record>
}

async fn sort_deploy_data(data: QueryResponse) -> HashMap<String, Vec<Record>> {
    let mut grouped_deploys: HashMap<String, Vec<Record>> = HashMap::new();

    for r in data.data.result {
      let env = r.stream.environment_name.unwrap().to_lowercase();

      if env != "dev" {
        continue;
      }

      for b in r.values {
        let rn = r.stream.repository_name.clone().unwrap();

        let d = b.json_data.body.deployment.as_ref().unwrap();
        let wf = b.json_data.body.workflow_run.unwrap();
        let status = r.stream.deployment_state.clone().unwrap_or_default();

        let record = Record {
          status: status == "success",
          repository: rn.clone(),
          team: r.stream.team_name.clone().unwrap(),
          created_at: d.created_at,
          sha: d.sha.clone(),
          deploy_url: wf.url.replace("api.", "").replace("repos/", ""),
          change_url: d.url.replace("api.", "").replace("repos/", "").replace("deployments/", "commit/").replace(d.id.to_string().as_str(), wf.head_sha.as_str()),
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

    return grouped_deploys;
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct IssueEntry {
  created_at: DateTime<Utc>,
  closed_at: Option<DateTime<Utc>>
}

async fn sort_issue_data(data: QueryResponse) -> HashMap<String, Vec<IssueEntry>> {
  let mut grouped_issues: HashMap<String, Vec<IssueEntry>> = HashMap::new();

  for result in data.data.result {
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

  return grouped_issues;
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct MergeEntry {
  sha: String,
  merged_at: DateTime<Utc>,
  user: String,
  title: String,
}

fn sort_merge_data(merge_data: QueryResponse) -> HashMap<String, MergeEntry> {
  let mut records_by_sha: HashMap<String, MergeEntry> = HashMap::new();

  for result in merge_data.data.result {
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

  return records_by_sha;
}

fn find_failures(deploy_data: &mut HashMap<String, Vec<Record>>, issue_data: &HashMap<String, Vec<IssueEntry>>) -> Vec<(String, usize, Option<DateTime<Utc>>, Option<String>)> {
  let mut on_failure: Option<(String, usize)> = None;
  let mut failures: Vec<(String, usize, Option<DateTime<Utc>>, Option<String>)> = [].to_vec();
  
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
      let mut deploy_issues: Vec<&IssueEntry> = [].to_vec();

      if deploy.status {
        match issue_data.get(&deploy.repository) {
          Some(ies) => {
            deploy_issues = ies.iter().filter(|e| {
              e.created_at >= deploy.created_at && e.created_at < next_deploy
            }).collect()
          },
          None => {}
        }
      } else {
        deploy.failed_at = Some(deploy.created_at);
        failed = true;
      }      

      if deploy_issues.len() > 0 {
        let opened_at = deploy_issues.iter().filter_map(|entry| Some(entry.created_at)).min();
        let closed_at = deploy_issues.iter().filter_map(|entry| entry.closed_at).max();

        deploy.failed_at = opened_at;
        deploy.fixed_at = closed_at;
      }

      
      if failed && on_failure.is_none() {
        on_failure = Some((key.to_string(), idx));
      } else if on_failure.is_some() && !failed {
        let failure = on_failure.unwrap();
        let fixed_at = Some(deploy.created_at);
        let fixed_url = Some(deploy.deploy_url.clone());

        failures.push((failure.0, failure.1, fixed_at, fixed_url));
        
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
        failure_record.fixed_url = entry.3;
      }
    }
  }
}

fn link_merge_to_deploys(deploy_by_sha: &mut HashMap<String, Vec<Record>>, merge_data: QueryResponse) {
  let merge_data = sort_merge_data(merge_data);

  for merge_entry in merge_data.iter() {
    deploy_by_sha.entry(merge_entry.0.to_string()).and_modify(|e| {
      for d in e {
        d.merged_at = Some(merge_entry.1.merged_at);
        d.title = Some(merge_entry.1.title.clone());
        d.user = Some(merge_entry.1.user.clone());
      }
    }).or_default();
  }
}

async fn gather_data(request: DataRequest) -> Result<(QueryResponse, QueryResponse, QueryResponse)> {
  let deploy_data_task = gather_deploy_data(&request);
  let issue_data_task = gather_issue_data(&request);
  let merge_data_task = gather_merge_data(&request);

  let (deploy_data_result, issue_data_result, merge_data_result) = tokio::join!(deploy_data_task, issue_data_task, merge_data_task);
  
  let deploy_data = match deploy_data_result {
    Ok(value) => value,
    Err(e) => return {
      println!("Error: {:?}", e);
      Err(e.into())
    }
  };

  let issue_data = match issue_data_result {
    Ok(value) => value,
    Err(e) => return {
      println!("Error: {:?}", e);
      Err(e.into())
    }
  };

  let merge_data = match merge_data_result {
    Ok(value) => value,
    Err(e) => return {
      println!("Error: {:?}", e);
      Err(e.into())
    }
  };

  Ok((deploy_data, issue_data, merge_data))
}

const REQUEST_DAYS: i64 = 5;
const REQUEST_DAYS_DURATION: TimeDelta = Duration::days(5);

async fn organize_data(request: DataRequest) -> Result<Vec<Record>> {
  let mut time_length = (request.end - request.start).num_days();
  let mut end = request.end;
  let mut all_ok = vec![];

  while time_length > 0 {
    let mut sub_request = request.clone();

    sub_request.end = end;

    if time_length > REQUEST_DAYS {
      sub_request.start = end - REQUEST_DAYS_DURATION;
    } else {
      sub_request.start = end - Duration::days(time_length);
    }

    let gather_result = gather_data(sub_request).await;

    match gather_result {
      Ok(result) => all_ok.push(result),
      Err(e) => return Err(e.into())
    };
    
    time_length -= REQUEST_DAYS;
    end = end - REQUEST_DAYS_DURATION;
  }

  let mut deploy_data: QueryResponse = Default::default();
  let mut issue_data: QueryResponse = Default::default();
  let mut merge_data: QueryResponse = Default::default();
  
  for (first, second, third) in all_ok {
    deploy_data.data.result.extend(first.data.result);
    issue_data.data.result.extend(second.data.result);
    merge_data.data.result.extend(third.data.result);
  }

  let mut sorted_deploy_data = sort_deploy_data(deploy_data).await;
  let sorted_issue_data = sort_issue_data(issue_data).await;

  link_issues_to_deployes(&mut sorted_deploy_data, &sorted_issue_data);  

  let mut deploy_by_sha: HashMap<String, Vec<Record>> = HashMap::new();

  for deploy_set in sorted_deploy_data.values() {
    for deploy in deploy_set {
      deploy_by_sha
        .entry(deploy.sha.clone())
        .or_insert_with(Vec::new)
        .push(deploy.clone());
    }
  }

  link_merge_to_deploys(&mut deploy_by_sha, merge_data);

  let mut all_deploys = Vec::new();

  for deploy_set in deploy_by_sha.values() {
    all_deploys.extend(deploy_set.clone());
  }

  Ok(all_deploys)
}

pub async fn handle_request(Extension(cache): Extension<DataCache>, Json(request): Json<DataRequest>) -> Result<Json<DataResponse>, StatusCode> {
  let request_key = format!("{:?}", request);

  if let Some(cached_response) = cache.get(&request_key) {
    return Ok(Json(cached_response.clone()));
  }

  let mut response : DataResponse = Default::default();

  let data = organize_data(request).await;

  match data {
    Ok(d) => {
      response.records = d;
      cache.insert(request_key, response.clone());
      Ok(Json(response))
    },
    Err(e) => {
      tracing::error!("Processing Data Failed: {:?}", e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    },
  }
}