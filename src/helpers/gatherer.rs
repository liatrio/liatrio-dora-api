use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use tracing::instrument;

use super::response::ResponseRecord;

#[derive(Debug, Clone, Default)]
pub struct IssueEntry {
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub number: u32,
}

#[derive(Debug, Clone, Default)]
pub struct MergeEntry {
    pub merged_at: DateTime<Utc>,
    pub user: String,
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct DeployEntry {
    pub status: bool,
    pub repository: String,
    pub team: String,
    pub created_at: DateTime<Utc>,
    pub sha: String,
    pub deploy_url: String,
    pub change_url: String,
}

#[derive(Debug, Clone, Default)]
pub struct GatheredData {
    pub deployments_by_repo: HashMap<String, Vec<DeployEntry>>,
    pub issues_by_repo: HashMap<String, Vec<IssueEntry>>,
    pub merges_by_sha: HashMap<String, MergeEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Failure {
    failed_at: Option<DateTime<Utc>>,
    fixed_at: Option<DateTime<Utc>>,
    issue_url: Option<String>,
    fixed_url: Option<String>,
}

/// Extracts failure details for a given deployment based on related issues and the deployment's SHA.
///
/// This function analyzes a deployment to determine if it failed and identifies any related issues that occurred
/// between the deployment's creation time and the next deployment's time. If the deployment failed, or if relevant
/// issues are found, it constructs a `Failure` struct with the failure details, including the time of failure, time of resolution,
/// and an optional issue URL.
///
/// # Arguments
///
/// * `deployment` - A reference to a `DeployEntry` struct representing the deployment being analyzed.
/// * `next_deployment_at` - A `DateTime<Utc>` representing the time of the next deployment, used to limit the issue search window.
/// * `data` - A reference to a `GatheredData` struct containing issues and other data relevant to the deployment.
///
/// # Returns
///
/// A tuple containing:
/// - `String` - The SHA of the deployment (empty if no failure or related issues are found).
/// - `Failure` - A struct containing details of the failure, including failure and fix times and an optional issue URL.
///
/// # Behavior
///
/// 1. If the deployment succeeded, the function searches for issues created between the deployment's creation time
///    and the time of the next deployment. If relevant issues are found, it identifies the earliest issue and the latest
///    issue closure time to determine the failure and fix times.
/// 2. If the deployment failed, the function captures the failure time directly from the deployment's creation time.
/// 3. If an issue is associated with the failure, the function modifies the deployment URL to point to the related issue.
///
/// # Example
///
/// ```rust
/// let deployment = DeployEntry {
///     status: false,
///     created_at: Utc::now() - Duration::hours(2),
///     sha: "abcdef".to_string(),
///     deploy_url: "https://github.com/owner/repo/actions/runs/123456".to_string(),
///     repository: "repo-a".to_string(),
///     ..Default::default()
/// };
///
/// let gathered_data = GatheredData {
///     deployments_by_repo: HashMap::new(),
///     issues_by_repo: HashMap::new(),
///     merges_by_sha: HashMap::new(),
/// };
///
/// let next_deployment_at = Utc::now();
///
/// let (sha, failure) = extract_failure_by_sha(&deployment, next_deployment_at, &gathered_data);
///
/// println!("SHA: {}, Failure: {:?}", sha, failure);
/// ```
///
/// This example demonstrates how to extract failure details for a given deployment, analyzing related issues if the deployment failed.
#[instrument]
fn extract_failure_by_sha(
    deployment: &DeployEntry,
    next_deployment_at: DateTime<Utc>,
    data: &GatheredData,
) -> (String, Failure) {
    let mut deploy_issues: Vec<&IssueEntry> = [].to_vec();
    let mut failure: Failure = Default::default();
    let mut sha: String = String::default();

    if deployment.status {
        if let Some(issues) = data.issues_by_repo.get(&deployment.repository) {
            deploy_issues = issues
                .iter()
                .filter(|issue| {
                    issue.created_at >= deployment.created_at
                        && issue.created_at < next_deployment_at
                })
                .collect()
        }
    } else {
        sha.clone_from(&deployment.sha);
        failure.failed_at = Some(deployment.created_at);
    }

    if !deploy_issues.is_empty() {
        let opened = deploy_issues
            .iter()
            .min_by_key(|record| record.created_at)
            .unwrap();

        let closing = deploy_issues
            .iter()
            .filter_map(|record| record.closed_at.map(|time| (record, time)))
            .max_by_key(|&(_, time)| time)
            .map(|(record, _)| record);

        failure.failed_at = Some(opened.created_at);

        sha.clone_from(&deployment.sha);

        let re = Regex::new(r"actions/runs/\d+").unwrap();

        let url = re.replace(
            deployment.deploy_url.as_str(),
            &format!("issues/{}", opened.number),
        );

        failure.issue_url = Some(url.to_string());

        if let Some(issue) = closing {
            if issue.closed_at > failure.failed_at {
                failure.fixed_at = issue.closed_at;
            }
        }
    }

    (sha, failure)
}

/// Identifies and maps failures for each deployment in the gathered data.
///
/// This function processes the deployment data from the `GatheredData` struct, finding failures and associating
/// them with their respective SHA values. It determines if a deployment failed by calling `extract_failure_by_sha`,
/// and tracks both failures and their fixes across multiple deployments.
///
/// If a failure is found but no fix is yet available (i.e., a succeeding deployment hasn’t fixed the failure),
/// the function holds onto the failure until a fix is found or until the last deployment is processed.
/// The failures are returned as a `HashMap` where the key is the deployment's SHA and the value is a `Failure` struct.
///
/// # Arguments
///
/// * `data` - A reference to the `GatheredData` struct that contains the deployment, issue, and merge data.
///
/// # Returns
///
/// A `HashMap<String, Failure>` where:
/// - The key is the SHA of the deployment.
/// - The value is a `Failure` struct containing the failure details (failure time, fix time, issue URL, fixed URL).
///
/// # Behavior
///
/// 1. Iterates through the deployments in the `GatheredData`, checking each deployment for failures.
/// 2. If a failure is found, it tracks the failure until a fix (from a later deployment) is identified.
/// 3. Once a failure is fixed, or if no fix is found by the end of the deployments, the failure is added to the result map.
/// 4. If multiple failures occur, they are stored individually in the result map based on their SHA.
///
/// # Example
///
/// ```rust
/// let gathered_data = GatheredData {
///     deployments_by_repo: ... // Deployment data
///     issues_by_repo: ...      // Issue data
///     merges_by_sha: ...       // Merge data
/// };
///
/// let failures = find_failures_per_deployment(&gathered_data);
///
/// for (sha, failure) in failures {
///     println!("SHA: {}, Failed at: {:?}, Fixed at: {:?}", sha, failure.failed_at, failure.fixed_at);
///     if let Some(issue_url) = failure.issue_url {
///         println!("Related issue: {}", issue_url);
///     }
/// }
/// ```
///
/// This example demonstrates how failures per deployment are identified and linked to their corresponding SHA.
///
/// # Notes
///
/// - The function handles the case where a failure is identified but has not yet been fixed by holding it in a temporary
///   variable (`previous_failure`) until a fix is found.
/// - If no fix is found by the end of the deployments, the failure is recorded without a fix time.
#[instrument]
fn find_failures_per_deployment(data: &GatheredData) -> HashMap<String, Failure> {
    let mut previous_failure: Option<(String, Failure)> = None;
    let mut failures: HashMap<String, Failure> = HashMap::new();

    for (_, deployments) in data.deployments_by_repo.iter() {
        let len: usize = deployments.len();

        for (index, deployment) in deployments.iter().enumerate() {
            let is_last = index + 1 >= len;

            let next_deployment_at = if !is_last {
                deployments[index + 1].created_at
            } else {
                DateTime::<Utc>::MAX_UTC
            };

            let (sha, failure) = extract_failure_by_sha(deployment, next_deployment_at, data);

            match failure.failed_at {
                Some(_) => {
                    if is_last {
                        failures.insert(sha, failure);

                        if previous_failure.is_some() {
                            let (sha, failure_data) = previous_failure.unwrap();

                            failures.insert(sha, failure_data);
                            previous_failure = None;
                        }
                    } else if previous_failure.is_none() {
                        previous_failure = Some((sha, failure));
                    }
                }
                None => {
                    if previous_failure.is_some() {
                        let (sha, mut failure_data) = previous_failure.unwrap();

                        if failure_data.fixed_at.is_none() {
                            failure_data.fixed_at = Some(deployment.created_at);
                            failure_data.fixed_url = Some(deployment.deploy_url.clone());
                        }

                        failures.insert(sha, failure_data);

                        previous_failure = None;
                    }
                }
            }
        }
    }

    failures
}

/// Links deployment, failure, and merge data into a list of response records.
///
/// This function processes the gathered deployment, issue, and merge data, and creates a list of
/// `ResponseRecord` objects that combine the information from each deployment with any associated failures
/// and merges. It links the failures and merges to the corresponding deployments by their SHA values.
///
/// For each deployment:
/// - If a failure is found (based on the SHA), the failure details (failure time, fix time, and issue URL) are added to the response.
/// - If a merge is found (based on the SHA), the merge details (merged time, title, and user) are added to the response.
///
/// # Arguments
///
/// * `data` - A `GatheredData` struct containing grouped deployment, issue, and merge data.
///
/// # Returns
///
/// A `Vec<ResponseRecord>` containing the combined data for each deployment. Each `ResponseRecord` contains:
/// - Deployment details: repository, team, SHA, status, creation time, deployment URL, and change URL.
/// - Failure details (if applicable): failure time, fix time, issue URL, and fixed URL.
/// - Merge details (if applicable): merged time, pull request title, and user.
///
/// # Example
///
/// ```rust
/// let gathered_data = GatheredData {
///     deployments_by_repo: ... // Deployment data here
///     issues_by_repo: ...      // Issue data here
///     merges_by_sha: ...       // Merge data here
/// };
///
/// let response_records = link_data(gathered_data);
///
/// for record in response_records {
///     println!("Repository: {}", record.repository);
///     println!("SHA: {}, Status: {}", record.sha, record.status);
///     if let Some(failed_at) = record.failed_at {
///         println!("Failed at: {:?}", failed_at);
///     }
///     if let Some(merged_at) = record.merged_at {
///         println!("Merged at: {:?}", merged_at);
///     }
/// }
/// ```
///
/// This example demonstrates how the function links deployment data with associated failures and merges, producing response records.
///
/// # Behavior
///
/// 1. The function first finds failures related to each deployment by SHA using `find_failures_per_deployment`.
/// 2. It then iterates over each deployment, adding deployment information to the `ResponseRecord`.
/// 3. If a failure is found, it adds failure details to the `ResponseRecord`.
/// 4. If a merge is found, it adds merge details to the `ResponseRecord`.
/// 5. The resulting list of response records is returned.
#[instrument]
pub fn link_data(data: GatheredData) -> Vec<ResponseRecord> {
    let mut records: Vec<ResponseRecord> = [].to_vec();

    let failures = find_failures_per_deployment(&data);

    data.deployments_by_repo.iter().for_each(|(_, value)| {
        value.iter().for_each(|deployment| {
            let mut record: ResponseRecord = ResponseRecord {
                repository: deployment.repository.clone(),
                team: deployment.team.clone(),
                sha: deployment.sha.clone(),
                status: deployment.status,
                created_at: deployment.created_at,
                deploy_url: deployment.deploy_url.clone(),
                change_url: deployment.change_url.clone(),
                ..Default::default()
            };

            let failure = failures.get(&deployment.sha);

            if failure.is_some() {
                let failure_data = failure.unwrap();

                record.failed_at = failure_data.failed_at;
                record.fixed_at = failure_data.fixed_at;
                record.issue_url.clone_from(&failure_data.issue_url);
                record.fixed_url.clone_from(&failure_data.fixed_url);
            }

            let merge = data.merges_by_sha.get(&deployment.sha);

            if merge.is_some() {
                let merge_data = merge.unwrap();

                record.merged_at = Some(merge_data.merged_at);
                record.title = Some(merge_data.title.clone());
                record.user = Some(merge_data.user.clone());
            }

            records.push(record);
        })
    });

    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_extract_failure_by_sha_with_failure_no_issues() {
        let deployment = DeployEntry {
            status: false,
            created_at: Utc::now() - Duration::hours(3),
            sha: "abcdef".to_string(),
            deploy_url: "https://github.com/owner/repo/actions/runs/123456".to_string(),
            repository: "repo-a".to_string(),
            ..Default::default()
        };

        let gathered_data = GatheredData {
            issues_by_repo: HashMap::new(),
            ..Default::default()
        };

        let next_deployment_at = Utc::now();
        let (sha, failure) =
            extract_failure_by_sha(&deployment, next_deployment_at, &gathered_data);

        assert_eq!(sha, "abcdef");
        assert_eq!(
            failure,
            Failure {
                failed_at: Some(deployment.created_at),
                fixed_at: None,
                issue_url: None,
                fixed_url: None,
            }
        );
    }

    #[test]
    fn test_extract_failure_by_sha_with_issues_and_fix() {
        let deployment = DeployEntry {
            status: true,
            created_at: Utc::now() - Duration::hours(3),
            sha: "abcdef".to_string(),
            deploy_url: "https://github.com/owner/repo/actions/runs/123456".to_string(),
            repository: "repo-a".to_string(),
            ..Default::default()
        };

        let issue1 = IssueEntry {
            created_at: Utc::now() - Duration::hours(2),
            closed_at: Some(Utc::now() - Duration::hours(1)),
            number: 42,
        };

        let gathered_data = GatheredData {
            issues_by_repo: vec![("repo-a".to_string(), vec![issue1.clone()])]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let next_deployment_at = Utc::now();
        let (sha, failure) =
            extract_failure_by_sha(&deployment, next_deployment_at, &gathered_data);

        assert_eq!(sha, "abcdef");
        assert_eq!(
            failure,
            Failure {
                failed_at: Some(issue1.created_at),
                fixed_at: Some(issue1.closed_at.unwrap()),
                issue_url: Some("https://github.com/owner/repo/issues/42".to_string()),
                fixed_url: None,
            }
        );
    }

    #[test]
    fn test_extract_failure_by_sha_with_issues_but_no_fix() {
        let deployment = DeployEntry {
            status: true,
            created_at: Utc::now() - Duration::hours(3),
            sha: "abcdef".to_string(),
            deploy_url: "https://github.com/owner/repo/actions/runs/123456".to_string(),
            repository: "repo-a".to_string(),
            ..Default::default()
        };

        let issue1 = IssueEntry {
            created_at: Utc::now() - Duration::hours(2),
            closed_at: None,
            number: 42,
        };

        let gathered_data = GatheredData {
            issues_by_repo: vec![("repo-a".to_string(), vec![issue1.clone()])]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let next_deployment_at = Utc::now();
        let (sha, failure) =
            extract_failure_by_sha(&deployment, next_deployment_at, &gathered_data);

        assert_eq!(sha, "abcdef");
        assert_eq!(
            failure,
            Failure {
                failed_at: Some(issue1.created_at),
                fixed_at: None,
                issue_url: Some("https://github.com/owner/repo/issues/42".to_string()),
                fixed_url: None,
            }
        );
    }

    #[test]
    fn test_extract_failure_by_sha_no_issues_no_failure() {
        let deployment = DeployEntry {
            status: true,
            created_at: Utc::now() - Duration::hours(3),
            sha: "abcdef".to_string(),
            deploy_url: "https://github.com/owner/repo/actions/runs/123456".to_string(),
            repository: "repo-a".to_string(),
            ..Default::default()
        };

        let gathered_data = GatheredData {
            issues_by_repo: HashMap::new(),
            ..Default::default()
        };

        let next_deployment_at = Utc::now();
        let (sha, failure) =
            extract_failure_by_sha(&deployment, next_deployment_at, &gathered_data);

        assert_eq!(sha, "");
        assert_eq!(failure, Failure::default());
    }
}
