use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{Response, Error};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Debug, Clone)]
pub struct QueryParams {
  pub query: String,
  pub start: String,
  pub end: String,
}

#[derive(Deserialize, Debug)]
pub struct QueryResponse {
  pub status: String,
  pub data: Data,
}

#[derive(Deserialize, Debug)]
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
}

#[derive(Deserialize, Debug)]
pub struct Issue {
  pub created_at: DateTime<Utc>,
  pub closed_at: Option<DateTime<Utc>>,
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
  pub created_at: DateTime<Utc>,
  pub sha: String,
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
      return client.get(url)
        .query(&data)
        .send()
        .await
    }
    _ => {
      return client.get(url)
        .query(&data)
        .basic_auth(user, Some(password))
        .send()
        .await
    }
  }
}

pub async fn query(data: QueryParams) -> Result<QueryResponse> {
    let url_var = env::var("LOKI_URL");
    let user_var = env::var("LOKI_USER");
    let password_var = env::var("LOKI_TOKEN");

    let url = match url_var {
      Ok(value) => value,
      Err(e) => return Err(anyhow!(format!("{}: LOKI_URL", e.to_string())))
    };

    let user = user_var.unwrap_or("".to_string());
    let password = password_var.unwrap_or("".to_string());

    let response_result = get_response(url, user, password, data).await;

    match response_result {
      Ok(response) => {
        let parse_result: Result<QueryResponse, Error> = response.json().await;

        match parse_result {
          Ok(value) => return Ok(value),
          Err(e) => {
            return Err(e.into());
          }
        }
      },
      Err(e) => {
        return Err(e.into());
      }
    }
}
