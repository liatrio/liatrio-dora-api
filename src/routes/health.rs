use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};

#[derive(Serialize, Debug)]
pub struct HealthResponse {

}

pub async fn handle_request() -> Result<Json<HealthResponse>, StatusCode> {
  let response : HealthResponse = HealthResponse{};

  Ok(Json(response))
}
