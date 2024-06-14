use anyhow::Result;
use super::loki::{query, QueryResponse};
use super::common::{DataRequest, fill_query_params};

pub async fn gather_merge_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"merged_at=~".+""#), None::<&str>);
  
  let query_result = query(query_params).await?;

  return Ok(query_result);
}

pub async fn gather_opened_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"action="opened""#), Some("|= `pull_request`"));
  
  let query_result = query(query_params).await?;

  return Ok(query_result);
}

pub async fn gather_deploy_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"deployment_state=~"success|failure""#), None::<&str>);
  
  let query_result = query(query_params).await?;

  return Ok(query_result);
}

pub async fn gather_issue_data(request: &DataRequest) -> Result<QueryResponse> {
  let query_params = fill_query_params(request, Some(r#"action="labeled""#), Some("|= `incident`"));
  
  let query_result = query(query_params).await?;

  return Ok(query_result);
}