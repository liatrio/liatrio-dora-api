use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};

use crate::helpers::common::DataRequest;

//use crate::helpers::loki::QueryResponse;
//use crate::helpers::queries::{};

#[derive(Serialize, Debug, Clone)]
pub struct ChangeFailureRateRecord {
}


#[derive(Serialize, Debug)]
pub struct ChangeFailureRateResponse {
  records: Vec<ChangeFailureRateRecord>
}

pub async fn handle_request(Json(_request): Json<DataRequest>) -> Result<Json<ChangeFailureRateResponse>, StatusCode> {
  let response : ChangeFailureRateResponse = ChangeFailureRateResponse {
    records: [].to_vec()
  };

  Ok(Json(response))
}