use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeDelta, Utc};
use reqwest::{Response, Error};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

use super::{gatherer::{DeployEntry, GatheredData, IssueEntry, MergeEntry}, request::DataRequest, response::ResponseRecord};

#[derive(Serialize, Debug, Clone, Default)]
pub struct QueryParams {
  pub query: String,
  pub start: String,
  pub end: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct QueryResponse {
  pub status: String,
  pub data: Data,
}

#[derive(Deserialize, Debug, Default)]
pub struct Data {
  #[serde(rename = "resultType")]
  pub result_type: String,
  pub result: Vec<ResultItem>,
}

#[derive(Deserialize, Debug)]
pub struct ResultItem {
  pub stream: Stream,
  pub values: Vec<ValueItem>,
}

#[derive(Deserialize, Debug)]
pub struct Stream {
  pub action: Option<String>,
  pub created_at: Option<DateTime<Utc>>,
  pub environment_name: Option<String>,
  pub repository_name: Option<String>,
  pub service_name: Option<String>,
  pub team_name: Option<String>,
  pub merge_sha: Option<String>,
  pub merged_at: Option<DateTime<Utc>>,
  pub deployment_state: Option<String>,
}

#[derive(Debug)]
pub struct ValueItem {
  pub timestamp: String,
  pub json_data: JsonData,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
  pub body: Body,
}

#[derive(Deserialize, Debug)]
pub struct Body {
  pub number: Option<u32>,
  pub pull_request: Option<PullRequest>,
  pub deployment: Option<Deployment>,
  pub issue: Option<Issue>,
  pub repository: Option<Repository>,
  pub workflow_run: Option<WorkflowRun>,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
  pub created_at: DateTime<Utc>,
  pub closed_at: Option<DateTime<Utc>>,
  pub number: u32,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
  pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
  pub title: String,
  pub user: User,
}

#[derive(Deserialize, Debug)]
pub struct Deployment {
  pub id: u32,
  pub created_at: DateTime<Utc>,
  pub sha: String,
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
  pub url: String,
  pub head_sha: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
  pub login: String,
}

impl<'de> Deserialize<'de> for ValueItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec: Vec<String> = Vec::deserialize(deserializer)?;
        if vec.len() != 2 {
            return Err(serde::de::Error::custom("Expected a tuple of two elements"));
        }
        let json_data: JsonData =
            serde_json::from_str(&vec[1]).map_err(serde::de::Error::custom)?;
        Ok(ValueItem {
            timestamp: vec[0].clone(),
            json_data,
        })
    }
}

async fn get_response(url: String, user: String, password: String, data: QueryParams) -> Result<Response, Error> {
  let client = reqwest::Client::new();
  
  match user.as_str() {
    "" => {
      client.get(url)
        .query(&data)
        .send()
        .await
    }
    _ => {
      client.get(url)
        .query(&data)
        .basic_auth(user, Some(password))
        .send()
        .await
    }
  }
}

async fn query(data: QueryParams) -> Result<QueryResponse> {
    let url_var = env::var("LOKI_URL");
    let user_var = env::var("LOKI_USER");
    let password_var = env::var("LOKI_TOKEN");

    let url = match url_var {
      Ok(value) => value,
      Err(e) => return Err(anyhow!(format!("{}: LOKI_URL", e.to_string())))
    };

    let user = user_var.unwrap_or("".to_string());
    let password = password_var.unwrap_or("".to_string());

    let response_result = get_response(url, user, password, data.clone()).await;
    
    match response_result {
      Ok(response) => {
        let status = response.status();

        if !status.is_success() {
          return Err(anyhow!(format!("Loki Responded with status: {:?}", status)))
        }

        let parse_result: Result<QueryResponse, Error> = response.json().await;

        match parse_result {
          Ok(value) => return Ok(value),
          Err(e) => {
            tracing::error!("Loki Response Parsing Failed: {:?}", e);
            return Err(e.into());
          }
        }
      },
      Err(e) => {
        tracing::error!("Loki Request Failed: {:?}", e);
        return Err(e.into());
      }
    }
}

fn fill_query_params<Q: AsRef<str>, F: AsRef<str>>(request: &DataRequest, query: Option<Q>, filter: Option<F>) -> QueryParams {
  let team_query = match &request.team {
    Some(t) => format!(r#",team_name="{}""#, t),
    None => "".to_string()
  };

  let repo_query = match &request.repositories {
    Some(r) => format!(r#",repository_name="{}""#, r.join("|")),
    None => "".to_string()
  };

  let unwrapped_query = query
    .as_ref()
    .map_or("".to_string(), |q| q.as_ref().to_string());

  let unwrapped_filter = filter
    .as_ref()
    .map_or("".to_string(), |f| f.as_ref().to_string());

  let query = match unwrapped_filter.as_str() {
    "" => format!(r#"{{{}{}{}}}"#, unwrapped_query, team_query, repo_query),
    _ => format!(r#"{{{}{}{}}} {}"#, unwrapped_query, team_query, repo_query, unwrapped_filter)
  };

  let params = QueryParams {
    start: request.start.timestamp_nanos_opt().unwrap().to_string(),
    end: request.end.timestamp_nanos_opt().unwrap().to_string(),
    query: query
  };

  return params;
}

async fn query_merge_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"merged_at=~".+""#), None::<&str>);
  
  let query_result = query(query_params).await;

  return query_result;
}

async fn query_deploy_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"deployment_state=~"success|failure""#), None::<&str>);
  
  let query_result = query(query_params).await;

  return query_result;
}

async fn query_issue_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"action=~"closed|opened""#), Some("|= `incident`"));
  
  let query_result = query(query_params).await;

  return query_result;
}

async fn sort_deploy_data(data: QueryResponse) -> HashMap<String, Vec<DeployEntry>> {
    let mut grouped_deploys: HashMap<String, Vec<DeployEntry>> = HashMap::new();

    for r in data.data.result {
      let env = r.stream.environment_name.unwrap().to_lowercase();

      if env != "dev" {
        continue;
      }

      for b in r.values {
        let rn = r.stream.repository_name.clone().unwrap();

        let d = b.json_data.body.deployment.as_ref().unwrap();
        let status = r.stream.deployment_state.clone().unwrap_or_default();

        let mut wf_url = "".to_string();
        let mut wf_hash = "".to_string();

        match b.json_data.body.workflow_run {
          Some(wf) => {
            wf_url = wf.url.replace("api.", "").replace("repos/", "");
            wf_hash = wf.head_sha;
          },
          None => {}
        }

        let record = DeployEntry {
          status: status == "success",
          repository: rn.clone(),
          team: r.stream.team_name.clone().unwrap(),
          created_at: d.created_at,
          sha: d.sha.clone(),
          deploy_url: wf_url,
          change_url: d.url.replace("api.", "").replace("repos/", "").replace("deployments/", "commit/").replace(d.id.to_string().as_str(), wf_hash.as_str()),
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



async fn sort_issue_data(data: QueryResponse) -> HashMap<String, Vec<IssueEntry>> {
  let mut grouped_issues: HashMap<String, Vec<IssueEntry>> = HashMap::new();

  for result in data.data.result {
    for value in result.values {
      let rn = value.json_data.body.repository.unwrap().name;
      let issue = value.json_data.body.issue.unwrap();

      let ie = IssueEntry {
        created_at: issue.created_at,
        closed_at: issue.closed_at,
        number: issue.number,
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

async fn query_data(request: DataRequest) -> Result<(QueryResponse, QueryResponse, QueryResponse)> {
  let deploy_data_task = query_deploy_data(&request);
  let issue_data_task = query_issue_data(&request);
  let merge_data_task = query_merge_data(&request);

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

pub async fn gather_data(request: DataRequest) -> Result<GatheredData> {
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

    let gather_result = query_data(sub_request).await;

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

  let sorted_deploy_data = sort_deploy_data(deploy_data).await;
  let sorted_issue_data = sort_issue_data(issue_data).await;
  let sorted_merge_data = sort_merge_data(merge_data);

  let gathered_data = GatheredData {
    deployments_by_repo: sorted_deploy_data,
    issues_by_repo: sorted_issue_data,
    merges_by_sha: sorted_merge_data,
  };

  return Ok(gathered_data);
}