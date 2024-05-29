use serde::{Deserialize, Serialize};
use axum::{
  http::StatusCode,
  response::Json
};

// use crate::helpers::loki::query;

#[derive(Deserialize, Debug)]
pub struct ChangeFailureRateRequest {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeFailureRateResponse {

}

pub async fn handle_request(Json(_data): Json<ChangeFailureRateRequest>) -> Result<Json<ChangeFailureRateResponse>, StatusCode> {
  let response : ChangeFailureRateResponse = ChangeFailureRateResponse{};

  Ok(Json(response))
}