use anyhow::{anyhow, Result};
use axum::{extract::Extension, http::StatusCode, response::Json};
use dashmap::DashMap;
use reqwest::Error;
use serde::Deserialize;
use std::{env, sync::Arc};
use tracing::instrument;

use crate::helpers::response::TeamsResponse;

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubTeam {
    name: String,
}

pub type TeamsCache = Arc<DashMap<String, TeamsResponse>>;

async fn get_teams(gh_org: &String, gh_token: &String, page: usize) -> Result<Vec<GitHubTeam>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/orgs/{}/teams", gh_org);

    let response_result = client
        .get(url)
        .query(&[("page", page), ("per_page", 100)])
        .header("User-Agent", "request")
        .header("Authorization", format!("token {}", gh_token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await;

    match response_result {
        Ok(response) => {
            let status = response.status();

            if !status.is_success() {
                tracing::error!("GitHub Teams Request Responded with status: {:?}", status);
                return Err(anyhow!(format!(
                    "GitHub responded with status: {:?}",
                    status
                )));
            }

            let parse_result: Result<Vec<GitHubTeam>, Error> = response.json().await;

            match parse_result {
                Ok(value) => Ok(value),
                Err(e) => {
                    tracing::error!("GitHub Teams Response Parsing Failed: {:?}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            tracing::error!("GitHub Teams Request Failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[instrument]
pub async fn handle_request(
    Extension(cache): Extension<TeamsCache>,
) -> Result<Json<TeamsResponse>, StatusCode> {
    let request_key = "teams".to_string();

    if let Some(cached_response) = cache.get(&request_key) {
        return Ok(Json(cached_response.clone()));
    }

    let mut response: TeamsResponse = Default::default();

    let gh_org_var = env::var("GITHUB_ORG");
    let gh_token_var = env::var("GITHUB_TOKEN");

    let gh_org = match gh_org_var {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("{}: GITHUB_ORG", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let gh_token = match gh_token_var {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("{}: GITHUB_TOKEN", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut page = 1;
    let mut all_teams: Vec<GitHubTeam> = [].to_vec();

    loop {
        let team_result = get_teams(&gh_org, &gh_token, page).await;

        match team_result {
            Ok(mut teams) => {
                if !teams.is_empty() {
                    all_teams.append(&mut teams);
                    page += 1;
                } else {
                    break;
                }
            }
            Err(_) => {
                tracing::error!("GitHub Request Failed");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    response.teams = all_teams.iter().map(|team| team.name.clone()).collect();

    cache.insert(request_key, response.clone());
    Ok(Json(response))
}
