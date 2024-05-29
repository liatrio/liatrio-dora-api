use serde::{Deserialize, Serialize};
use axum::{
  http::StatusCode,
  response::Json
};

// use crate::helpers::loki::query;

#[derive(Deserialize, Debug)]
pub struct RecoverTimeRequest {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecoverTimeResponse {

}

pub async fn handle_request(Json(_data): Json<RecoverTimeRequest>) -> Result<Json<RecoverTimeResponse>, StatusCode> {
  let response : RecoverTimeResponse = RecoverTimeResponse{};

  Ok(Json(response))
}