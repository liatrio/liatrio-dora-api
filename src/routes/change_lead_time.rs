use serde::{Deserialize, Serialize};
use axum::{
  http::StatusCode,
  response::Json
};

// use crate::helpers::loki::query;

#[derive(Deserialize, Debug)]
pub struct ChangeLeadTimeRequest {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeLeadTimeResponse {

}

pub async fn handle_request(Json(_data): Json<ChangeLeadTimeRequest>) -> Result<Json<ChangeLeadTimeResponse>, StatusCode> {
  let response : ChangeLeadTimeResponse = ChangeLeadTimeResponse{};

  Ok(Json(response))
}