use std::{sync::Arc, env};
use serde::{Deserialize, Serialize};
use axum::{
  extract::Extension, http::StatusCode, response::Json
};
use dashmap::DashMap;
use anyhow::{Result, anyhow};
use reqwest::Error;


#[derive(Serialize, Debug, Clone, Default)]
pub struct Team {
  pub name: String,
  pub repositories: Vec<String>
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct TeamsResponse {
  pub teams: Vec<Team>
}

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubTeam {
  id: u64,
  name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubRepository {
  name: String,
}

pub type TeamsCache = Arc<DashMap<String, TeamsResponse>>;

async fn get_teams(gh_org: &String, gh_token: &String) -> Result<Vec<GitHubTeam>> {
  let client = reqwest::Client::new();
  let url = format!("https://api.github.com/orgs/{}/teams", gh_org);
  
  let response_result = client.get(url)
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
        return Err(anyhow!(format!("GitHUb responsed with status: {:?}", status)));
      }

      let parse_result: Result<Vec<GitHubTeam>, Error> = response.json().await;

      match parse_result {
        Ok(value) => return Ok(value),
        Err(e) => {
          tracing::error!("GitHub Teams Response Parsing Failed: {:?}", e);
          return Err(e.into());
        }
      }
    },
    Err(e) => {
      tracing::error!("GitHub Teams Request Failed: {:?}", e);
      return Err(e.into());
    }
  }
}

async fn get_repositories(team: GitHubTeam, gh_org: &String, gh_token: &String) -> Result<Vec<GitHubRepository>> {
  let client = reqwest::Client::new();
  let url = format!("https://api.github.com/teams/{}/repos", team.id);
  
  let response_result = client.get(url)
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
        tracing::error!("GitHub Repositories Request Responded with status: {:?}", status);
        return Err(anyhow!(format!("GitHUb responsed with status: {:?}", status)));
      }

      let parse_result: Result<Vec<GitHubRepository>, Error> = response.json().await;

      match parse_result {
        Ok(value) => return Ok(value),
        Err(e) => {
          tracing::error!("GitHub Repositories Response Parsing Failed: {:?}", e);
          return Err(e.into());
        }
      }
    },
    Err(e) => {
      tracing::error!("GitHub Repositories Request Failed: {:?}", e);
      return Err(e.into());
    }
  }
}


pub async fn handle_request(Extension(cache): Extension<TeamsCache>) -> Result<Json<TeamsResponse>, StatusCode> {
  let request_key = format!("teams");

  if let Some(cached_response) = cache.get(&request_key) {
    return Ok(Json(cached_response.clone()));
  }

  let mut response : TeamsResponse = Default::default();
  
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

  let teams_result = get_teams(&gh_org, &gh_token).await;

  match teams_result {
    Ok(teams) => {
      for team in teams {
        let repositories_result = get_repositories(team.clone(), &gh_org, &gh_token).await;

        match repositories_result {
          Ok(repositories) => {
            let repository_names = repositories.into_iter().map(|e| e.name).collect();

            let new_team: Team = Team {
              name: team.name.clone(),
              repositories: repository_names
            };

            response.teams.push(new_team);
          },
          Err(e) => {
            tracing::error!("Failed to get repositories for team: {}", team.name);
          }
        }
      }
    },
    Err(e) => {
      tracing::error!("GitHub Request Failed");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  }

  
  cache.insert(request_key, response.clone());
  Ok(Json(response))
}
