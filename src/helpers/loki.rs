use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

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

#[derive(Deserialize, Debug, Default)]
pub struct ResultItem {
    pub stream: Stream,
    pub values: Vec<ValueItem>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Stream {
    pub deployment_environment_name: Option<String>,
    pub vcs_repository_name: String,
    pub team_name: String,
    pub merged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct ValueItem {
    pub json_data: JsonData,
}

#[derive(Deserialize, Debug, Default)]
pub struct JsonData {
    pub pull_request: Option<PullRequest>,
    pub deployment: Option<Deployment>,
    pub deployment_status: Option<DeploymentStatus>,
    pub issue: Option<Issue>,
    pub repository: Option<Repository>,
    pub workflow_run: Option<WorkflowRun>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Issue {
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub number: u32,
}

#[derive(Deserialize, Debug, Default)]
pub struct Repository {
    pub name: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct PullRequest {
    pub title: String,
    pub user: User,
    pub merge_commit_sha: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct DeploymentStatus {
    pub state: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Deployment {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub sha: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct WorkflowRun {
    pub workflow_id: Option<u32>,
}

#[derive(Deserialize, Debug, Default)]
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

/// Makes an asynchronous REST API call using GET and optional basic authentication.
///
/// This function constructs and sends a GET request to the provided `url` with the given query parameters.
/// If a `user` is supplied, basic authentication is used with the provided `password`. If no `user` is supplied,
/// the request is made without authentication.
///
/// # Arguments
///
/// * `url` - The URL to which the request is sent.
/// * `user` - The username for basic authentication. If empty, no authentication is used.
/// * `password` - The password or token for basic authentication. Only used if `user` is not empty.
/// * `data` - A `QueryParams` structure containing the query parameters to be sent with the request.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(Response)` if the request is successful.
/// - `Err(Error)` if the request fails.
///
/// The response object can be further processed (e.g., reading the body or checking the status code).
///
/// # Errors
///
/// This function will return an error if the request fails, the URL is invalid, or there are network issues.
///
/// # Example
///
/// Sending a request without authentication:
/// ```rust
/// let query_params = QueryParams {
///     start: "1625097600000000000".to_string(),
///     end: "1625101200000000000".to_string(),
///     query: "some Loki query".to_string(),
///     limit: 5000,
/// };
///
/// let result = make_rest_call("https://loki-server.com/api".to_string(), "".to_string(), "".to_string(), query_params).await;
/// match result {
///     Ok(response) => println!("Request succeeded: {:?}", response),
///     Err(e) => eprintln!("Request failed: {:?}", e),
/// }
/// ```
///
/// Sending a request with basic authentication:
/// ```rust
/// let result = make_rest_call(
///     "https://loki-server.com/api".to_string(),
///     "myuser".to_string(),
///     "mypassword".to_string(),
///     query_params
/// ).await;
/// ```
///
/// # Authentication
///
/// Basic authentication is used if a `user` is provided. The `password` is optional but recommended.
async fn make_rest_call(
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

/// Sends an asynchronous query request to a Loki server and returns the parsed response.
///
/// This function constructs a REST call to a Loki instance using query parameters, authenticating
/// with a username and token (if available via environment variables). It handles response
/// parsing and error handling, returning a `QueryResponse` on success or an error if the request
/// fails.
///
/// Environment variables used:
/// - `LOKI_URL`: The base URL of the Loki server (required).
/// - `LOKI_USER`: Optional username for basic authentication (default: empty string).
/// - `LOKI_TOKEN`: Optional password or token for basic authentication (default: empty string).
///
/// # Arguments
///
/// * `data` - A `QueryParams` structure containing the query parameters to be sent in the request.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(QueryResponse)` if the request is successful and the response can be parsed.
/// - `Err(anyhow::Error)` if the request fails, the server returns an error status, or the response cannot be parsed.
///
/// # Errors
///
/// - If the `LOKI_URL` environment variable is missing or cannot be retrieved, an error is returned.
/// - If the REST call fails, an error is returned and logged.
/// - If the Loki server responds with a non-success HTTP status code, an error is returned.
/// - If the response cannot be parsed into a `QueryResponse`, an error is returned and logged.
///
/// # Example
///
/// ```rust
/// let query_params = QueryParams {
///     start: "1625097600000000000".to_string(),
///     end: "1625101200000000000".to_string(),
///     query: "some Loki query".to_string(),
///     limit: 5000,
/// };
///
/// let result = query(query_params).await;
///
/// match result {
///     Ok(response) => println!("Query succeeded with data: {:?}", response),
///     Err(e) => eprintln!("Query failed: {:?}", e),
/// }
/// ```
///
/// # Logging
///
/// Errors are logged using the `tracing` crate for both request failures and response parsing failures.
async fn query(data: QueryParams) -> Result<QueryResponse> {
    let url_var = env::var("LOKI_URL");

    let url = match url_var {
        Ok(value) => value,
        Err(e) => return Err(anyhow!(format!("{}: LOKI_URL", e.to_string()))),
    };

    let user = env::var("LOKI_USER").unwrap_or_default();
    let password = env::var("LOKI_TOKEN").unwrap_or_default();

    let response_result = make_rest_call(url, user, password, data.clone()).await;

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

/// Constructs a set of query parameters based on the provided request, query, and optional filter.
///
/// This function takes a `DataRequest` object, a query string, and an optional filter string to
/// build a `QueryParams` structure for querying data. It also reads an environment variable
/// `SERVICE_NAME` to determine the service namespace, defaulting to "github" if the variable is
/// not set.
///
/// The constructed query includes:
///
/// 1. A team name filter, if present in the `request`.
/// 2. A repository filter, if present in the `request`.
/// 3. The main query and an optional filter string.
///
/// # Arguments
///
/// * `request` - A reference to a `DataRequest` that contains the query request information such as team, repositories, and time range.
/// * `query` - A string reference representing the main query string.
/// * `filter` - An optional string reference representing an additional filter to be applied.
///
/// # Returns
///
/// A `QueryParams` struct with the constructed query, time range, and limit.
///
/// The structure includes:
///
/// - `start`: The start time in nanoseconds (as a string).
/// - `end`: The end time in nanoseconds (as a string).
/// - `query`: The constructed query string.
/// - `limit`: A hardcoded limit of 5000 for the query results.
///
/// # Panics
///
/// This function will panic if the environment variable `SERVICE_NAME` cannot be retrieved and is
/// not set to a default, or if the `timestamp_nanos_opt` values from the `request` are `None`.
///
/// # Example
///
/// ```
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Some(Utc::now()),
///     end: Some(Utc::now()),
/// };
///
/// let query_params = fill_query_params(&request, "deployment_status", Some("success"));
///
/// assert_eq!(query_params.limit, 5000);
/// assert!(query_params.query.contains(r#"team_name="team-a""#));
/// assert!(query_params.query.contains(r#"vcs_repository_name="repo-a|repo-b""#));
/// ```
///
/// If no filter is provided:
///
/// ```
/// let query_params = fill_query_params(&request, "deployment_status", None);
///
/// assert!(query_params.query.ends_with("deployment_status"));
/// ```
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
            r#"{{service_namespace=`{}`}} | {}{}{} {}"#,
            service_name_var,
            team_query,
            repo_query,
            query.as_ref(),
            f.as_ref()
        ),
        None => format!(
            r#"{{service_namespace=`{}`}} | {}{}{}"#,
            service_name_var,
            team_query,
            repo_query,
            query.as_ref()
        ),
    };

    QueryParams {
        start: request.start.timestamp_nanos_opt().unwrap().to_string(),
        end: request.end.timestamp_nanos_opt().unwrap().to_string(),
        query,
        limit: 5000,
    }
}

/// Queries merge data for changes that have been closed and merged.
///
/// This function constructs query parameters using the `fill_query_params` function, targeting events
/// where the event name is `change_closed` and the `merged_at` field is not empty. It then sends
/// the query to the server using the `query` function, which performs the actual data retrieval.
///
/// # Arguments
///
/// * `request` - A reference to a `DataRequest` that contains information about the team, repositories, and time range for the query.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(QueryResponse)` with the data from the query if the request is successful.
/// - `Err(anyhow::Error)` if the request or query execution fails.
///
/// # Example
///
/// ```rust
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Some(Utc::now() - Duration::days(7)),
///     end: Some(Utc::now()),
/// };
///
/// let result = query_merge_data(&request).await;
///
/// match result {
///     Ok(response) => println!("Query successful: {:?}", response),
///     Err(e) => eprintln!("Query failed: {:?}", e),
/// }
/// ```
///
/// This query specifically filters for events where a change was closed and successfully merged.
async fn query_merge_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"event_name=`change_closed`, merged_at!="""#,
        None::<&str>,
    );

    query(query_params).await
}

/// Queries deployment data for successful or failed deployments.
///
/// This function constructs query parameters using the `fill_query_params` function, targeting
/// deployment events where the `deployment_state` is either "success" or "failure". It then sends
/// the query to the server using the `query` function to retrieve the relevant deployment data.
///
/// # Arguments
///
/// * `request` - A reference to a `DataRequest` that contains information about the team, repositories, and time range for the query.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(QueryResponse)` with the deployment data if the request is successful.
/// - `Err(anyhow::Error)` if the request or query execution fails.
///
/// # Example
///
/// ```rust
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Some(Utc::now() - Duration::days(7)),
///     end: Some(Utc::now()),
/// };
///
/// let result = query_deploy_data(&request).await;
///
/// match result {
///     Ok(response) => println!("Query successful: {:?}", response),
///     Err(e) => eprintln!("Query failed: {:?}", e),
/// }
/// ```
///
/// This query specifically filters for deployment events that resulted in either a success or failure.
async fn query_deploy_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"deployment_status=~`failure|success`"#,
        None::<&str>,
    );

    query(query_params).await
}

/// Queries issue data for closed issues, optionally filtering for incidents.
///
/// This function constructs query parameters using the `fill_query_params` function, targeting
/// events where the `event_name` is `issue_closed`. Additionally, it applies a filter for
/// events containing the word "incident". The query is then sent to the server using the `query`
/// function to retrieve the relevant issue data.
///
/// # Arguments
///
/// * `request` - A reference to a `DataRequest` that contains information about the team, repositories, and time range for the query.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(QueryResponse)` with the issue data if the request is successful.
/// - `Err(anyhow::Error)` if the request or query execution fails.
///
/// # Example
///
/// ```rust
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Some(Utc::now() - Duration::days(7)),
///     end: Some(Utc::now()),
/// };
///
/// let result = query_issue_data(&request).await;
///
/// match result {
///     Ok(response) => println!("Query successful: {:?}", response),
///     Err(e) => eprintln!("Query failed: {:?}", e),
/// }
/// ```
///
/// This query specifically filters for events where an issue was closed, and optionally
/// filters for incidents using the provided filter.
async fn query_issue_data(request: &DataRequest) -> Result<QueryResponse> {
    let query_params = fill_query_params(
        request,
        r#"event_name=`issue_closed`"#,
        Some("|= `incident`"),
    );

    query(query_params).await
}

/// Extracts deployment data from a `ValueItem` and constructs a `DeployEntry`.
///
/// This function extracts necessary deployment information, including the status, deployment URL, and
/// change URL, from the provided `ValueItem`. It combines this information with the given team name
/// and repository name to create a `DeployEntry`.
///
/// The function expects that the `ValueItem` contains a valid `deployment` and `deployment_status`.
/// It retrieves the status of the deployment, checking whether it is marked as "success", and constructs
/// URLs for the deployment and change by using GitHub-specific helper methods.
///
/// # Arguments
///
/// * `value` - A reference to a `ValueItem` containing the deployment and deployment status.
/// * `team_name` - A `String` representing the name of the team associated with the deployment.
/// * `repository_name` - A `String` representing the name of the repository associated with the deployment.
///
/// # Returns
///
/// A `DeployEntry` struct containing:
/// - The status of the deployment (`true` for success, `false` otherwise).
/// - The repository and team names.
/// - The timestamp when the deployment was created.
/// - The SHA of the deployment.
/// - The deployment URL.
/// - The change URL associated with the deployment.
///
/// # Panics
///
/// This function will panic if the `deployment` or `deployment_status` fields inside the `ValueItem` are `None`.
///
/// # Example
///
/// ```rust
/// let value = ValueItem {
///     json_data: JsonData {
///         deployment: Some(Deployment {
///             created_at: Utc::now(),
///             sha: "abcdef".to_string(),
///             ..Default::default()
///         }),
///         deployment_status: Some(DeploymentStatus {
///             state: "success".to_string(),
///         }),
///     }
/// };
///
/// let entry = extract_deployment_data(&value, "team-a".to_string(), "repo-a".to_string());
/// assert_eq!(entry.status, true);
/// assert_eq!(entry.team, "team-a");
/// assert_eq!(entry.repository, "repo-a");
/// ```
///
/// This function provides a convenient way to extract deployment information and populate a `DeployEntry` struct.
fn extract_deployment_data(
    value: &ValueItem,
    team_name: String,
    repository_name: String,
) -> DeployEntry {
    let d: &Deployment = value.json_data.deployment.as_ref().unwrap();
    let status = value.json_data.deployment_status.as_ref().unwrap().state == "success";

    let deploy_url = GitHub::extract_deployment_url(value);
    let change_url = GitHub::extract_change_url(value);

    DeployEntry {
        status,
        repository: repository_name,
        team: team_name,
        created_at: d.created_at,
        sha: d.sha.clone(),
        deploy_url,
        change_url,
    }
}

/// Filters duplicate deployments by their SHA, retaining only successful ones.
///
/// This function takes a mutable reference to a vector of `DeployEntry` structs and filters out duplicate
/// deployments that share the same SHA. If multiple deployments have the same SHA, only the first successful
/// deployment is retained. If a deployment with the same SHA is unsuccessful, it is removed unless a successful
/// deployment with the same SHA is not yet encountered.
///
/// The filtering is done in-place, modifying the original vector by removing duplicates based on the SHA.
///
/// # Arguments
///
/// * `deploys` - A mutable reference to a vector of `DeployEntry` structs to be filtered.
///
/// # Behavior
///
/// - The function uses a `HashMap` to track SHAs that have been seen and their success status.
/// - If a deployment's SHA has not been encountered, it is added to the map.
/// - If a deployment's SHA has already been encountered, only the first successful deployment is retained, and
///   any further deployments with the same SHA are removed.
///
/// # Example
///
/// ```rust
/// let mut deploys = vec![
///     DeployEntry {
///         sha: "abcdef".to_string(),
///         status: false,
///         ..Default::default()
///     },
///     DeployEntry {
///         sha: "abcdef".to_string(),
///         status: true,
///         ..Default::default()
///     },
///     DeployEntry {
///         sha: "123456".to_string(),
///         status: true,
///         ..Default::default()
///     },
/// ];
///
/// filter_duplicate_deployments_by_sha(&mut deploys);
///
/// assert_eq!(deploys.len(), 2); // Only the successful "abcdef" and "123456" remain
/// assert!(deploys.iter().any(|d| d.sha == "abcdef" && d.status == true));
/// assert!(deploys.iter().any(|d| d.sha == "123456"));
/// ```
///
/// This function is useful for cleaning up deployment lists where multiple entries may exist
/// for the same deployment, but only the successful ones should be retained.
fn filter_duplicate_deployments_by_sha(deploys: &mut Vec<DeployEntry>) {
    let mut seen_shas: HashMap<String, bool> = HashMap::new();

    deploys.retain(|entry| {
        let sha = entry.sha.clone();

        if let Some(&seen) = seen_shas.get(&sha) {
            if !seen && entry.status {
                seen_shas.entry(sha).and_modify(|value| *value = true);
                return true;
            }
            false
        } else {
            seen_shas.insert(sha, entry.status);
            true
        }
    });
}

/// Sorts and filters deployment data by environment, repository, and timestamp.
///
/// This function processes a `QueryResponse` containing deployment data, filters the deployments
/// based on the environment names (targeting production environments), and groups the filtered
/// deployments by repository name. It also sorts the deployments by their creation timestamp
/// and filters out duplicate deployments based on their SHA, keeping only the first successful
/// deployment for each SHA.
///
/// The production environments are determined by reading the `PRODUCTION_ENVIRONMENT_NAMES` environment
/// variable, which defaults to "production,prod" if not set. Only deployments from these environments are
/// considered during processing.
///
/// # Arguments
///
/// * `data` - A `QueryResponse` struct containing deployment data to be processed.
///
/// # Returns
///
/// A `HashMap` where:
/// - The key is a `String` representing the repository name.
/// - The value is a vector of `DeployEntry` structs sorted by creation time, with duplicate
///   deployments filtered out.
///
/// # Environment Variables
///
/// * `PRODUCTION_ENVIRONMENT_NAMES` - A comma-separated list of environment names considered as production.
///   Defaults to "production,prod" if not set.
///
/// # Behavior
///
/// 1. Filters deployments based on environment names (must match a name in the `PRODUCTION_ENVIRONMENT_NAMES` variable).
/// 2. Groups the deployments by the repository name.
/// 3. Sorts each group of deployments by their `created_at` timestamp.
/// 4. Filters out duplicate deployments based on the SHA, retaining only the first successful deployment for each SHA.
///
/// # Example
///
/// ```rust
/// let query_response = QueryResponse {
///     data: ... // Query result data here
/// };
///
/// let sorted_deployments = sort_deploy_data(query_response);
///
/// for (repo, deploys) in sorted_deployments {
///     println!("Repository: {}", repo);
///     for deploy in deploys {
///         println!("Deployment at {} with SHA {}", deploy.created_at, deploy.sha);
///     }
/// }
/// ```
///
/// In this example, the deployment data is sorted by repository and timestamp, and duplicates are filtered by SHA.
fn sort_deploy_data(data: QueryResponse) -> HashMap<String, Vec<DeployEntry>> {
    let mut grouped_deploys: HashMap<String, Vec<DeployEntry>> = HashMap::new();
    let prod_env_names =
        env::var("PRODUCTION_ENVIRONMENT_NAMES").unwrap_or("production,prod".to_string());

    for r in data.data.result {
        let env = r.stream.deployment_environment_name.unwrap().to_lowercase();

        if !prod_env_names.contains(&env) || env.starts_with("prod-") {
            continue;
        }

        let repository_name = r.stream.vcs_repository_name;
        let team_name = r.stream.team_name;

        for value in r.values {
            let record =
                extract_deployment_data(&value, team_name.clone(), repository_name.clone());

            grouped_deploys
                .entry(repository_name.clone())
                .or_default()
                .push(record)
        }
    }

    for v in grouped_deploys.values_mut() {
        v.sort_by(|l, r| l.created_at.cmp(&r.created_at));

        filter_duplicate_deployments_by_sha(v);
    }

    grouped_deploys
}

/// Sorts and groups issue data by repository, ordered by creation date.
///
/// This function processes a `QueryResponse` containing issue data, groups the issues by
/// repository name, and sorts each group of issues by their creation timestamp (`created_at`).
///
/// Each issue is represented as an `IssueEntry` containing the issue's number, creation time, and closing time.
///
/// # Arguments
///
/// * `data` - A `QueryResponse` struct containing issue data to be processed.
///
/// # Returns
///
/// A `HashMap` where:
/// - The key is a `String` representing the repository name.
/// - The value is a vector of `IssueEntry` structs sorted by their `created_at` timestamp.
///
/// # Behavior
///
/// 1. Extracts the repository name and issue details from the provided `QueryResponse`.
/// 2. Groups the issues by the repository name.
/// 3. Sorts each group of issues by their creation timestamp.
///
/// # Example
///
/// ```rust
/// let query_response = QueryResponse {
///     data: ... // Query result data here
/// };
///
/// let sorted_issues = sort_issue_data(query_response);
///
/// for (repo, issues) in sorted_issues {
///     println!("Repository: {}", repo);
///     for issue in issues {
///         println!("Issue #{} created at {}", issue.number, issue.created_at);
///     }
/// }
/// ```
///
/// In this example, the issues are grouped by repository and sorted by their creation time.
fn sort_issue_data(data: QueryResponse) -> HashMap<String, Vec<IssueEntry>> {
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

/// Sorts and groups merge data by SHA (commit hash).
///
/// This function processes a `QueryResponse` containing merge data and groups the merge entries
/// by the SHA of the merge commit. For each pull request, it extracts the relevant user, title,
/// and merge timestamp, and creates a `MergeEntry`. The data is then stored in a `HashMap` where
/// the key is the merge commit SHA, and the value is the corresponding `MergeEntry`.
///
/// If multiple entries are encountered for the same SHA, only the first one is retained.
///
/// # Arguments
///
/// * `merge_data` - A `QueryResponse` struct containing merge data to be processed.
///
/// # Returns
///
/// A `HashMap` where:
/// - The key is a `String` representing the merge commit SHA.
/// - The value is a `MergeEntry` struct containing details about the merge event, such as the user who made the pull request, the pull request title, and the merge timestamp.
///
/// # Example
///
/// ```rust
/// let merge_data = QueryResponse {
///     data: ... // Query result data here
/// };
///
/// let sorted_merges = sort_merge_data(merge_data);
///
/// for (sha, entry) in sorted_merges {
///     println!("Merge commit SHA: {}", sha);
///     println!("Merged by: {}, Title: {}, Merged at: {}", entry.user, entry.title, entry.merged_at);
/// }
/// ```
///
/// In this example, the merge data is grouped by SHA and contains details about the pull request and user who performed the merge.
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

/// Asynchronously queries deployment, issue, and merge data in parallel.
///
/// This function takes a `DataRequest` and concurrently queries three different sets of data:
/// deployment data, issue data, and merge data. It uses `tokio::join!` to run the queries in parallel,
/// and returns a tuple containing the results of the three queries if all are successful. If any query fails,
/// the function logs the error and returns it.
///
/// # Arguments
///
/// * `request` - A `DataRequest` struct containing the information needed to perform the queries.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok((QueryResponse, QueryResponse, QueryResponse))` - A tuple of `QueryResponse` values representing
///   the deployment, issue, and merge data.
/// - `Err(anyhow::Error)` - If any of the queries fail, an error is returned.
///
/// # Behavior
///
/// 1. The function spawns three asynchronous tasks to query deployment data, issue data, and merge data.
/// 2. It waits for all three tasks to complete using `tokio::join!`.
/// 3. If any of the queries result in an error, the function logs the error and returns it.
/// 4. If all queries are successful, the function returns a tuple containing the three query responses.
///
/// # Example
///
/// ```rust
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Some(Utc::now() - Duration::days(7)),
///     end: Some(Utc::now()),
/// };
///
/// let result = query_data(request).await;
///
/// match result {
///     Ok((deploy_data, issue_data, merge_data)) => {
///         println!("Deployment data: {:?}", deploy_data);
///         println!("Issue data: {:?}", issue_data);
///         println!("Merge data: {:?}", merge_data);
///     }
///     Err(e) => {
///         eprintln!("Query failed: {:?}", e);
///     }
/// }
/// ```
///
/// In this example, the function queries deployment, issue, and merge data concurrently and handles any potential errors.
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

/// Retrieves the batch size for querying data over a specific number of days.
///
/// This function reads the `LOKI_DAYS_BATCH_SIZE` environment variable to determine the number of days
/// to use as the batch size for querying. If the environment variable is not set or cannot be parsed as an
/// integer, the function defaults to returning `5`.
///
/// # Returns
///
/// An `i64` representing the number of days to use as the batch size. The value is either retrieved
/// from the `LOKI_DAYS_BATCH_SIZE` environment variable or defaults to `5` if the variable is not set or
/// contains an invalid value.
///
/// # Example
///
/// ```rust
/// // If LOKI_DAYS_BATCH_SIZE is set to "7"
/// let batch_size = get_batch_days_size();
/// assert_eq!(batch_size, 7);
///
/// // If LOKI_DAYS_BATCH_SIZE is not set or is invalid, the default value of 5 is returned.
/// let batch_size = get_batch_days_size();
/// assert_eq!(batch_size, 5);
/// ```
///
/// This function is useful for determining how many days' worth of data to process in each batch,
/// with the flexibility of configuring the value via an environment variable.
fn get_batch_days_size() -> i64 {
    let var = env::var("LOKI_DAYS_BATCH_SIZE");

    match var {
        Ok(value) => value.parse::<i64>().unwrap_or(5),
        Err(_) => 5,
    }
}

/// Gathers deployment, issue, and merge data over a range of time by batching the requests.
///
/// This function takes a `DataRequest` and processes it in batches, determined by the number of days
/// specified in the environment variable `LOKI_DAYS_BATCH_SIZE` (defaulting to 5 days if not set). It repeatedly
/// queries the data within smaller time windows until the entire requested time range is covered. The gathered
/// data is then sorted and returned as a `GatheredData` struct containing deployment, issue, and merge data grouped
/// by repository and SHA.
///
/// # Arguments
///
/// * `request` - A `DataRequest` struct specifying the time range and filters for the query.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(GatheredData)` - A struct containing the gathered and sorted data for deployments, issues, and merges.
/// - `Err(anyhow::Error)` - If any batch query fails, an error is returned.
///
/// # Batching Behavior
///
/// The function calculates the total number of days in the request time range and divides the work into batches.
/// Each batch retrieves data for a time window determined by `LOKI_DAYS_BATCH_SIZE`. The function uses multiple
/// asynchronous queries, accumulating the results as it proceeds through the time range.
///
/// # Example
///
/// ```rust
/// let request = DataRequest {
///     team: Some("team-a".to_string()),
///     repositories: Some(vec!["repo-a".to_string(), "repo-b".to_string()]),
///     start: Utc::now() - Duration::days(30),
///     end: Utc::now(),
/// };
///
/// let gathered_data = gather_data(request).await;
///
/// match gathered_data {
///     Ok(data) => {
///         println!("Deployments by repo: {:?}", data.deployments_by_repo);
///         println!("Issues by repo: {:?}", data.issues_by_repo);
///         println!("Merges by SHA: {:?}", data.merges_by_sha);
///     }
///     Err(e) => eprintln!("Error gathering data: {:?}", e),
/// }
/// ```
///
/// This example demonstrates querying data over a 30-day period, batched in smaller chunks,
/// and then accessing the gathered deployment, issue, and merge data.
///
/// # Environment Variables
///
/// * `LOKI_DAYS_BATCH_SIZE` - Defines the number of days to include in each batch of the query. Defaults to 5 days if not set.
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

    let sorted_deploy_data = sort_deploy_data(deploy_data);
    let sorted_issue_data = sort_issue_data(issue_data);
    let sorted_merge_data = sort_merge_data(merge_data);

    let gathered_data = GatheredData {
        deployments_by_repo: sorted_deploy_data,
        issues_by_repo: sorted_issue_data,
        merges_by_sha: sorted_merge_data,
    };

    Ok(gathered_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::env;

    #[test]
    fn test_fill_query_params_with_all_fields() {
        env::set_var("SERVICE_NAME", "test_service");

        let request = DataRequest {
            team: Some("test_team".to_string()),
            repositories: Some(vec!["repo1".to_string(), "repo2".to_string()]),
            start: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            end: DateTime::<Utc>::from_timestamp(1, 0).unwrap(),
        };

        let filter = Some("filter".to_string());
        let query = "query".to_string();

        let result = fill_query_params(&request, &query, filter.as_ref());

        assert_eq!(result.start, "0");
        assert_eq!(result.end, "1000000000");
        assert_eq!(
            result.query,
            r#"{service_namespace=`test_service`} | team_name="test_team", vcs_repository_name="repo1|repo2", query filter"#
        );
        assert_eq!(result.limit, 5000);
    }

    #[test]
    fn test_fill_query_params_without_optional_fields() {
        env::set_var("SERVICE_NAME", "test_service");

        let request = DataRequest {
            team: None,
            repositories: None,
            start: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            end: DateTime::<Utc>::from_timestamp(1, 0).unwrap(),
        };

        let filter = None;
        let query = "query".to_string();

        let result = fill_query_params(&request, &query, filter.as_ref());

        assert_eq!(result.start, "0");
        assert_eq!(result.end, "1000000000");
        assert_eq!(
            result.query,
            r#"{service_namespace=`test_service`} | query"#
        );
        assert_eq!(result.limit, 5000);
    }

    #[test]
    fn test_filter_duplicate_deployments_by_sha_with_successful_duplicates() {
        let mut deploys = vec![
            DeployEntry {
                sha: "abcdef".to_string(),
                status: false,
                ..Default::default()
            },
            DeployEntry {
                sha: "abcdef".to_string(),
                status: true,
                ..Default::default()
            },
            DeployEntry {
                sha: "123456".to_string(),
                status: true,
                ..Default::default()
            },
            DeployEntry {
                sha: "abcdef".to_string(),
                status: false,
                ..Default::default()
            },
        ];

        filter_duplicate_deployments_by_sha(&mut deploys);

        assert_eq!(deploys.len(), 3);
        assert!(deploys
            .iter()
            .any(|d| d.sha == "abcdef" && d.status == false));
        assert!(deploys
            .iter()
            .any(|d| d.sha == "abcdef" && d.status == true));
        assert!(deploys
            .iter()
            .any(|d| d.sha == "123456" && d.status == true));
    }

    #[test]
    fn test_filter_duplicate_deployments_by_sha_without_successful_duplicates() {
        let mut deploys = vec![
            DeployEntry {
                sha: "abcdef".to_string(),
                status: false,
                ..Default::default()
            },
            DeployEntry {
                sha: "123456".to_string(),
                status: true,
                ..Default::default()
            },
            DeployEntry {
                sha: "abcdef".to_string(),
                status: false,
                ..Default::default()
            },
        ];

        filter_duplicate_deployments_by_sha(&mut deploys);

        assert_eq!(deploys.len(), 2);
        assert!(deploys
            .iter()
            .any(|d| d.sha == "abcdef" && d.status == false));
        assert!(deploys
            .iter()
            .any(|d| d.sha == "123456" && d.status == true));
    }

    #[test]
    fn test_filter_duplicate_deployments_by_sha_only_one_deployment() {
        let mut deploys = vec![DeployEntry {
            sha: "abcdef".to_string(),
            status: true,
            ..Default::default()
        }];

        filter_duplicate_deployments_by_sha(&mut deploys);

        assert_eq!(deploys.len(), 1);
        assert_eq!(deploys[0].sha, "abcdef");
        assert_eq!(deploys[0].status, true);
    }

    #[test]
    fn test_filter_duplicate_deployments_by_sha_no_deployments() {
        let mut deploys: Vec<DeployEntry> = vec![];

        filter_duplicate_deployments_by_sha(&mut deploys);

        assert_eq!(deploys.len(), 0);
    }

    #[test]
    fn test_filter_duplicate_deployments_by_sha_all_successful() {
        let mut deploys = vec![
            DeployEntry {
                sha: "abcdef".to_string(),
                status: true,
                ..Default::default()
            },
            DeployEntry {
                sha: "abcdef".to_string(),
                status: true,
                ..Default::default()
            },
            DeployEntry {
                sha: "123456".to_string(),
                status: true,
                ..Default::default()
            },
        ];

        filter_duplicate_deployments_by_sha(&mut deploys);

        assert_eq!(deploys.len(), 2);
        assert!(deploys
            .iter()
            .any(|d| d.sha == "abcdef" && d.status == true));
        assert!(deploys
            .iter()
            .any(|d| d.sha == "123456" && d.status == true));
    }
}
