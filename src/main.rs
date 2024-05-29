use axum::{
    routing::{post, get},
    Router,
};
use helpers::error::AppError;
use std::env;
use dotenv::dotenv;

mod helpers;
mod routes;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    let app = Router::new()
        .route("/deployment_frequency", post(routes::deployment_frequency::handle_request))
        .route("/change_lead_time", post(routes::change_lead_time::handle_request))
        .route("/change_failure_rate", post(routes::change_failure_rate::handle_request))
        .route("/recover_time", post(routes::recover_time::handle_request))
        .route("/health", get(routes::health::handle_request));

    let port = env::var("PORT")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await?;

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
