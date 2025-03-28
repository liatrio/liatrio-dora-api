use anyhow::Result;
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use dotenv::dotenv;
use std::{env, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use opentelemetry::sdk::trace as sdktrace;
use tracing_opentelemetry::OpenTelemetryLayer;

mod helpers;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Set up the OpenTelemetry tracer (OTLP exporter to collector)
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "liatrio-dora-api"),
            ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Create a tracing layer with the OTEL tracer
    let telemetry = OpenTelemetryLayer::new(tracer);

    // Build a tracing subscriber with the OTEL layer + log formatting
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    let data_cache: routes::data::DataCache = Arc::new(DashMap::new());
    let teams_cache: routes::teams::TeamsCache = Arc::new(DashMap::new());

    let app = Router::new()
        .route("/data", post(routes::data::handle_request))
        .layer(Extension(data_cache))
        .route("/teams", get(routes::teams::handle_request))
        .layer(Extension(teams_cache))
        .route("/health", get(routes::health::handle_request));

    let port = env::var("PORT")?;
    let addr = format!("[::]:{port}")
        .parse::<std::net::SocketAddr>()
        .unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::warn!("listening on {:?}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider(); // graceful shutdown

    Ok(())
}

async fn shutdown_signal() {
    use std::sync::mpsc;
    use std::{thread, time::Duration};

    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
    let (sender, receiver) = mpsc::channel();
    let _ = thread::spawn(move || {
        opentelemetry::global::shutdown_tracer_provider();
        sender.send(()).ok()
    });
    let shutdown_res = receiver.recv_timeout(Duration::from_millis(2_000));
    if shutdown_res.is_err() {
        tracing::error!("failed to shutdown OpenTelemetry");
    }
}
