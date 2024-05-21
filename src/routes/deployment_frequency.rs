use serde::{Deserialize, Serialize};
use axum::{
  http::StatusCode,
  response::Json
};

use crate::helpers::loki::query;

#[derive(Deserialize, Debug)]
pub struct DeploymentFrequencyRequest {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeploymentFrequencyResponse {

}

pub async fn handle_request(Json(data): Json<DeploymentFrequencyRequest>) -> Result<Json<DeploymentFrequencyResponse>, StatusCode> {
  let response : DeploymentFrequencyResponse = DeploymentFrequencyResponse{};

  Ok(Json(response))
}