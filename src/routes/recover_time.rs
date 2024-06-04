use serde::Serialize;
use axum::{
  http::StatusCode,
  response::Json
};

use crate::helpers::common::DataRequest;

//use crate::helpers::loki::QueryResponse;
//use crate::helpers::queries::{};

#[derive(Serialize, Debug, Clone)]
pub struct RecoverTimeRecord {
}


#[derive(Serialize, Debug)]
pub struct RecoverTimeResponse {
  records: Vec<RecoverTimeRecord>
}

pub async fn handle_request(Json(_request): Json<DataRequest>) -> Result<Json<RecoverTimeResponse>, StatusCode> {
  let response : RecoverTimeResponse = RecoverTimeResponse {
    records: [].to_vec()
  };

  Ok(Json(response))
}