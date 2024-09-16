use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

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

#[derive(Debug, Clone, Default)]
struct Failure {
    failed_at: Option<DateTime<Utc>>,
    fixed_at: Option<DateTime<Utc>>,
    issue_url: Option<String>,
    fixed_url: Option<String>,
}

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
        let opened_at = deploy_issues.iter().map(|issue| issue.created_at).min();
        let closing = deploy_issues
            .iter()
            .filter_map(|record| record.closed_at.map(|time| (record, time)))
            .max_by_key(|&(_, time)| time)
            .map(|(record, _)| record);

        failure.failed_at = opened_at;
        sha.clone_from(&deployment.sha);

        if let Some(issue) = closing {
            if issue.closed_at > opened_at {
                failure.fixed_at = issue.closed_at;

                let re = Regex::new(r"actions/runs/\d+").unwrap();
                let url = re.replace(
                    deployment.deploy_url.as_str(),
                    &format!("issues/{}", issue.number),
                );

                failure.issue_url = Some(url.to_string());
            }
        }
    }

    (sha, failure)
}

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
