use std::sync::Arc;
use serde::Serialize;
use axum::{
  extract::Extension, http::StatusCode, response::Json
};
use dashmap::DashMap;
use anyhow::Result;

use crate::helpers::{
  gatherer::link_data, loki::gather_data, request::DataRequest, response::ResponseRecord
};

pub type DataCache = Arc<DashMap<String, DataResponse>>;

#[derive(Serialize, Debug, Default, Clone)]
pub struct DataResponse {
  records: Vec<ResponseRecord>,
}

pub async fn handle_request(Extension(cache): Extension<DataCache>, Json(request): Json<DataRequest>) -> Result<Json<DataResponse>, StatusCode> {
  let request_key = format!("{:?}", request);

  if let Some(cached_response) = cache.get(&request_key) {
    return Ok(Json(cached_response.clone()));
  }

  let data_set = gather_data(request).await;

  match data_set {
    Ok(data) => {
      let records = link_data(data);

      let response = DataResponse {
        records,
      };

      cache.insert(request_key, response.clone());
      Ok(Json(response))
    },
    Err(e) => {
      tracing::error!("Processing Data Failed: {:?}", e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    },
  }
}