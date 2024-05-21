use std::env;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::helpers::error::AppError;

pub async fn query<D, R>(data: D) -> Result<R, AppError>
where
  R: DeserializeOwned,
  D: Serialize
{
  let client = reqwest::Client::new();
  let url = env::var("LOKI_URL")?;
  let user = env::var("LOKI_USER")?;
  let password = env::var("LOKI_TOKEN")?;

  let response = client
    .post(url)
    .json(&data)
    .basic_auth(user, Some(password))
    .send()
    .await?;

  let data: R = response.json().await?;
  Ok(data)
}