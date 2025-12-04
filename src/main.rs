mod config;
mod handlers;
mod ops;
mod security;

use axum::{routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::{info, debug, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Logging with enhanced formatting
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into())
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Load Config
    let config = config::load_config().await?;
    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str.parse()?;

    info!("Starting hOOk on {}", addr);
    debug!("Loaded {} repository configurations", config.repos.len());
    for (name, repo) in &config.repos {
        debug!("  - {} -> {} (branch: {})", name, repo.path, repo.branch);
    }

    // Shared State
    let shared_state = Arc::new(handlers::AppState { config });

    // Router Setup
    let app = Router::new()
        .route("/webhook", post(handlers::github_webhook))
        .with_state(shared_state.clone());

    // Start Server
    if let Some(tls) = &shared_state.config.tls {
        let rustls_config = RustlsConfig::from_pem_file(
            PathBuf::from(&tls.cert_path),
            PathBuf::from(&tls.key_path),
        )
        .await?;

        info!("HTTPS enabled");
        axum_server::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        info!("Running in HTTP mode (no TLS configured)");
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}