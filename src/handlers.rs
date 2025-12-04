use crate::{config::AppConfig, ops, security};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use simd_json::prelude::*;
use std::sync::Arc;
use tracing::{info, warn};

/// The shared state structure
pub struct AppState {
    pub config: AppConfig,
}

pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // 1. Extract Signature
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // 2. Security Check (HMAC)
    if !security::verify_signature(&state.config.secret, &body, signature) {
        warn!("Unauthorized access attempt detected.");
        return StatusCode::UNAUTHORIZED;
    }

    // 3. High-Perf Parsing (simd-json)
    // We clone bytes to Vec because simd-json modifies input in-place for performance
    let mut body_vec = body.to_vec();
    
    // simd_json::to_borrowed_value is the fastest way to read JSON in Rust
    let payload = match simd_json::to_borrowed_value(&mut body_vec) {
        Ok(json) => json,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // 4. Extract Data (Zero allocations for strings)
    let repo_name = payload["repository"]["full_name"].as_str().unwrap_or("");
    let push_ref = payload["ref"].as_str().unwrap_or("");

    // 5. Routing Logic
    if let Some(repo_config) = state.config.repos.get(repo_name) {
        if push_ref == repo_config.branch {
            info!("Trigger detected for {} on {}", repo_name, push_ref);
            
            // Fire and Forget (Async Task)
            let config_clone = repo_config.clone();
            let name_clone = repo_name.to_string();
            
            tokio::spawn(async move {
                ops::perform_update(name_clone, config_clone).await;
            });
            
            return StatusCode::OK;
        }
    }

    StatusCode::OK
}