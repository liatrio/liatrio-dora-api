use axum::{
    extract::Extension,
    routing::{post, get},
    Router,
};
use anyhow::Result;
use std::{env, sync::Arc};
use dotenv::dotenv;
use dashmap::DashMap;

mod helpers;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cache: routes::data::DataCache = Arc::new(DashMap::new());

    let app = Router::new()
        .route("/data", post(routes::data::handle_request))
        .layer(Extension(cache))
        .route("/health", get(routes::health::handle_request));

    let port = env::var("PORT")?;
    let addr = format!("[::]:{port}").parse::<std::net::SocketAddr>().unwrap();

    let listener = tokio::net::TcpListener::bind(addr)
        .await?;

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service()).await.unwrap();

    Ok(())
}
