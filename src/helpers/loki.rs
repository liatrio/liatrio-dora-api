use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env,
};

use super::{
    event_vendor::EventVendorFunctions,
    gatherer::{DeployEntry, GatheredData, IssueEntry, MergeEntry},
    github::GitHub,
    request::DataRequest,
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct QueryParams {
    pub query: String,
    pub start: String,
    pub end: String,
    pub limit: u16,
}

#[derive(Deserialize, Debug, Default)]
pub struct QueryResponse {
    pub data: Data,
}

#[derive(Deserialize, Debug, Default)]
pub struct Data {
    pub result: Vec<ResultItem>,
}

#[derive(Deserialize, Debug)]
pub struct ResultItem {
    pub stream: Stream,
    pub values: Vec<ValueItem>,
}

#[derive(Deserialize, Debug)]
pub struct Stream {
    pub deployment_environment_name: Option<String>,
    pub environment_name: String,
    pub vcs_repository_name: String,
    pub team_name: String,
    pub merged_at: Option<DateTime<Utc>>,
    pub deployment_status: Option<String>,
}

#[derive(Debug)]
pub struct ValueItem {
    pub json_data: JsonData,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
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
    pub merge_commit_sha: String,
}

#[derive(Deserialize, Debug)]
pub struct Deployment {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub sha: String,
    pub url: String,
    pub environment: String,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub head_sha: String,
    pub workflow_id: Option<u32>,
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
        Ok(ValueItem { json_data })
    }
}

async fn get_response(
    url: String,
    user: String,
    password: String,
    data: QueryParams,
) -> Result<Response, Error> {
    let client = reqwest::Client::new();

    match user.as_str() {
        "" => client.get(url).query(&data).send().await,
        _ => {
            client
                .get(url)
                .query(&data)
                .basic_auth(user, Some(password))
                .send()
                .await
        }
    }
}

async fn query(data: QueryParams) -> Result<QueryResponse> {
    let url_var = env::var("LOKI_URL");

    let url = match url_var {
        Ok(value) => value,
        Err(e) => return Err(anyhow!(format!("{}: LOKI_URL", e.to_string()))),
    };

    let user = env::var("LOKI_USER").unwrap_or_default();
    let password = env::var("LOKI_TOKEN").unwrap_or_default();

    let response_result = get_response(url, user, password, data.clone()).await;

    match response_result {
        Ok(response) => {
            let status = response.status();

            if !status.is_success() {
                return Err(anyhow!(format!("Loki Responded with status: {:?}", status)));
            }

            let parse_result: Result<QueryResponse, Error> = response.json().await;

            match parse_result {
                Ok(value) => Ok(value),
                Err(e) => {
                    tracing::error!("Loki Response Parsing Failed: {:?}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            tracing::error!("Loki Request Failed: {:?}", e);
            Err(e.into())
        }
    }
}

fn fill_query_params<T: AsRef<str>>(
    request: &DataRequest,
    query: T,
    filter: Option<T>,
) -> QueryParams {
    let service_name_var = env::var("SERVICE_NAME").unwrap_or("github".to_string());

    let team_query = match &request.team {
        Some(t) => format!(r#"team_name="{}", "#, t),
        None => "".to_string(),
    };

    let repo_query = match &request.repositories {
        Some(r) => format!(r#"vcs_repository_name="{}", "#, r.join("|")),
        None => "".to_string(),
    };

    let query = match filter {
        Some(f) => format!(
            r#"{{service_namespace=`{:?}`}} | {:?}{:?}{:?} {:?}"#,
            service_name_var,
            team_query,
            repo_query,
            query.as_ref().to_string(),
            f.as_ref().to_string()
        ),
        None => format!(
            r#"{{service_namespace=`{:?}`}} | {:?}{:?}{:?}"#,
            service_name_var,
            team_query,
            repo_query,
            query.as_ref().to_string()
        ),
    };

    QueryParams {
        start: request.start.timestamp_nanos_opt().unwrap().to_string(),
        end: request.end.timestamp_nanos_opt().unwrap().to_string(),
        query,
        limit: 5000,
    }
}

async fn query_merge_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"event_name=`change_closed`, merged_at!="""#,
        None::<&str>,
    );

    query(query_params).await
}

async fn query_deploy_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"deployment_state=~"success|failure""#,
        None::<&str>,
    );

    query(query_params).await
}

async fn query_issue_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"event_name=`issue_closed`"#,
        Some("|= `incident`"),
    );

    query(query_params).await
}

async fn sort_deploy_data(data: QueryResponse) -> HashMap<String, Vec<DeployEntry>> {
    let mut grouped_deploys: HashMap<String, Vec<DeployEntry>> = HashMap::new();
    let prod_env_names =
        env::var("PRODUCTION_ENVIRONMENT_NAMES").unwrap_or("production,prod".to_string());

    for r in data.data.result {
        let env = r.stream.deployment_environment_name.unwrap().to_lowercase();

        if !prod_env_names.contains(&env) {
            continue;
        }

        for b in r.values {
            let rn = r.stream.vcs_repository_name.clone();

            let d = b.json_data.deployment.as_ref().unwrap();
            let status = r.stream.deployment_status.clone().unwrap_or_default();

            let deployment_url = GitHub::extract_deployment_url(&b);
            let change_url = GitHub::extract_change_url(&b);

            let record = DeployEntry {
                status: status == "success",
                repository: rn.clone(),
                team: r.stream.team_name.clone(),
                created_at: d.created_at,
                sha: d.sha.clone(),
                deploy_url: deployment_url,
                change_url,
            };

            grouped_deploys.entry(rn.clone()).or_default().push(record)
        }
    }

    for v in grouped_deploys.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at));

        let mut seen_shas = HashSet::new();

        v.retain(|entry| seen_shas.insert(entry.sha.clone()));
    }

    grouped_deploys
}

async fn sort_issue_data(data: QueryResponse) -> HashMap<String, Vec<IssueEntry>> {
    let mut grouped_issues: HashMap<String, Vec<IssueEntry>> = HashMap::new();

    for result in data.data.result {
        for value in result.values {
            let rn = value.json_data.repository.unwrap().name;
            let issue = value.json_data.issue.unwrap();

            let ie = IssueEntry {
                created_at: issue.created_at,
                closed_at: issue.closed_at,
                number: issue.number,
            };

            grouped_issues.entry(rn.clone()).or_default().push(ie)
        }
    }

    for v in grouped_issues.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at))
    }

    grouped_issues
}

fn sort_merge_data(merge_data: QueryResponse) -> HashMap<String, MergeEntry> {
    let mut records_by_sha: HashMap<String, MergeEntry> = HashMap::new();

    for result in merge_data.data.result {
        for value in result.values {
            let pr = value.json_data.pull_request.unwrap();

            let record = MergeEntry {
                user: pr.user.login.clone(),
                title: pr.title.clone(),
                merged_at: result.stream.merged_at.unwrap(),
            };

            records_by_sha.entry(pr.merge_commit_sha).or_insert(record);
        }
    }

    records_by_sha
}

async fn query_data(request: DataRequest) -> Result<(QueryResponse, QueryResponse, QueryResponse)> {
    let deploy_data_task = query_deploy_data(&request);
    let issue_data_task = query_issue_data(&request);
    let merge_data_task = query_merge_data(&request);

    let (deploy_data_result, issue_data_result, merge_data_result) =
        tokio::join!(deploy_data_task, issue_data_task, merge_data_task);

    let deploy_data = match deploy_data_result {
        Ok(value) => value,
        Err(e) => {
            return {
                println!("Error: {:?}", e);
                Err(e)
            }
        }
    };

    let issue_data = match issue_data_result {
        Ok(value) => value,
        Err(e) => {
            return {
                println!("Error: {:?}", e);
                Err(e)
            }
        }
    };

    let merge_data = match merge_data_result {
        Ok(value) => value,
        Err(e) => {
            return {
                println!("Error: {:?}", e);
                Err(e)
            }
        }
    };

    Ok((deploy_data, issue_data, merge_data))
}

fn get_batch_days_size() -> i64 {
    let var = env::var("LOKI_DAYS_BATCH_SIZE");

    match var {
        Ok(value) => value.parse::<i64>().unwrap_or(5),
        Err(_) => 5,
    }
}

pub async fn gather_data(request: DataRequest) -> Result<GatheredData> {
    let mut time_length = (request.end - request.start).num_days();
    let mut end = request.end;
    let mut all_ok = vec![];

    let batch_days_size = get_batch_days_size();
    let batch_duration = Duration::days(batch_days_size);

    while time_length > 0 {
        let mut sub_request = request.clone();

        sub_request.end = end;

        if time_length > batch_days_size {
            sub_request.start = end - batch_duration;
        } else {
            sub_request.start = end - Duration::days(time_length);
        }

        let gather_result = query_data(sub_request).await;

        match gather_result {
            Ok(result) => all_ok.push(result),
            Err(e) => return Err(e),
        };

        time_length -= batch_days_size;
        end -= batch_duration;
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

    Ok(gathered_data)
}
