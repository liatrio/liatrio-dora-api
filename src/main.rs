use axum::{
    routing::{post, get},
    Router,
};
use anyhow::Result;
use std::env;
use dotenv::dotenv;

mod helpers;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let app = Router::new()
        .route("/data", post(routes::data::handle_request))
        .route("/health", get(routes::health::handle_request));

    let port = env::var("PORT")?;
    let addr = format!("[::]:{port}").parse::<std::net::SocketAddr>().unwrap();

    let listener = tokio::net::TcpListener::bind(addr)
        .await?;

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
