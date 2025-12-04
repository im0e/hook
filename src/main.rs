mod config;
mod handlers;
mod ops;
mod security;

use axum::{routing::post, Router};
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Logging
    tracing_subscriber::fmt::init();

    // Load Config
    let config = config::load_config().await?;
    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str.parse()?;

    info!("Starting Rusty-Deploy on {}", addr);

    // Shared State
    let shared_state = Arc::new(handlers::AppState { config });

    // Router Setup
    let app = Router::new()
        .route("/webhook", post(handlers::github_webhook))
        .with_state(shared_state);

    // Start Server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}