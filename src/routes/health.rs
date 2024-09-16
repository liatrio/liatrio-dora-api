use axum::{http::StatusCode, response::Json};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HealthResponse {}

pub async fn handle_request() -> Result<Json<HealthResponse>, StatusCode> {
    let response: HealthResponse = HealthResponse {};

    Ok(Json(response))
}
