use chrono::{DateTime, Utc};
use serde::Deserialize;


use crate::helpers::loki::QueryParams;

#[derive(Deserialize, Debug, Clone)]
pub struct DataRequest {
  pub repositories: Option<Vec<String>>,
  pub team: Option<String>,
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>
}

pub fn fill_query_params<Q: AsRef<str>, F: AsRef<str>>(request: &DataRequest, query: Option<Q>, filter: Option<F>) -> QueryParams {
  let team_query = match &request.team {
    Some(t) => format!(r#",team_name="{}""#, t),
    None => "".to_string()
  };

  let repo_query = match &request.repositories {
    Some(r) => format!(r#",repository_name="{}""#, r.join("|")),
    None => "".to_string()
  };

  let unwrapped_query = query
    .as_ref()
    .map_or("".to_string(), |q| q.as_ref().to_string());

  let unwrapped_filter = filter
    .as_ref()
    .map_or("".to_string(), |f| f.as_ref().to_string());

  let query = match unwrapped_filter.as_str() {
    "" => format!(r#"{{{}{}{}}}"#, unwrapped_query, team_query, repo_query),
    _ => format!(r#"{{{}{}{}}} {}"#, unwrapped_query, team_query, repo_query, unwrapped_filter)
  };

  let params = QueryParams {
    start: request.start.timestamp_nanos_opt().unwrap().to_string(),
    end: request.end.timestamp_nanos_opt().unwrap().to_string(),
    query: query
  };

  return params;
}