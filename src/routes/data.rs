use anyhow::Result;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Json,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::helpers::{
    gatherer::link_data, loki::gather_data, request::DataRequest, response::ResponseRecord,
};

pub type DataCache = Arc<DashMap<String, DataResponse>>;

#[derive(Serialize, Debug, Default, Clone)]
pub struct DataResponse {
    records: Vec<ResponseRecord>,
}

#[derive(Deserialize, Debug)]
pub struct RequestParams {
    pub no_cache: Option<bool>,
}

pub async fn handle_request(
    Extension(cache): Extension<DataCache>,
    Query(params): Query<RequestParams>,
    Json(request): Json<DataRequest>,
) -> Result<Json<DataResponse>, StatusCode> {
    let request_key = format!("{:?}", request);

    if !params.no_cache.unwrap_or_default() {
        if let Some(cached_response) = cache.get(&request_key) {
            return Ok(Json(cached_response.clone()));
        }
    }

    let data_set = gather_data(request).await;

    match data_set {
        Ok(data) => {
            let records = link_data(data);

            let response = DataResponse { records };

            if cache.contains_key(&request_key) {
                cache.alter(&request_key, |_, _| response.clone());
            } else {
                cache.insert(request_key, response.clone());
            }

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Processing Data Failed: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
