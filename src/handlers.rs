use crate::{config::AppConfig, ops, security};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use simd_json::prelude::*;
use std::sync::Arc;
use tracing::{info, warn, debug, error, instrument};

/// The shared state structure
pub struct AppState {
    pub config: AppConfig,
}

#[instrument(skip(state, headers, body), fields(payload_size = body.len()))]
pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    debug!("Received webhook request ({} bytes)", body.len());
    
    // Log relevant headers
    if let Some(event) = headers.get("x-github-event") {
        debug!("GitHub event: {:?}", event);
    }
    if let Some(delivery) = headers.get("x-github-delivery") {
        debug!("Delivery ID: {:?}", delivery);
    }

    // 1. Extract Signature
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // 2. Security Check (HMAC)
    if !security::verify_signature(&state.config.secret, &body, signature) {
        warn!(signature_present = !signature.is_empty(), "Unauthorized access attempt detected");
        return StatusCode::UNAUTHORIZED;
    }
    debug!("Signature verified successfully");

    // 3. High-Perf Parsing (simd-json)
    // We clone bytes to Vec because simd-json modifies input in-place for performance
    let mut body_vec = body.to_vec();
    
    // simd_json::to_borrowed_value is the fastest way to read JSON in Rust
    let payload = match simd_json::to_borrowed_value(&mut body_vec) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to parse JSON payload: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // 4. Extract Data (Zero allocations for strings)
    let repo_name = payload["repository"]["full_name"].as_str().unwrap_or("");
    let push_ref = payload["ref"].as_str().unwrap_or("");
    let sender = payload["sender"]["login"].as_str().unwrap_or("unknown");
    let commit_count = payload["commits"].as_array().map(|c| c.len()).unwrap_or(0);

    debug!(
        repo = repo_name,
        branch = push_ref,
        sender = sender,
        commits = commit_count,
        "Parsed webhook payload"
    );

    if repo_name.is_empty() || push_ref.is_empty() {
        error!(
            repo = repo_name,
            branch = push_ref,
            "Missing critical webhook fields"
        );
        return StatusCode::BAD_REQUEST;
    }

    // 5. Routing Logic
    match state.config.repos.get(repo_name) {
        Some(repo_config) => {
            debug!(
                repo = repo_name,
                received_ref = push_ref,
                expected_ref = repo_config.branch.as_str(),
                "Checking branch match"
            );

            if push_ref == repo_config.branch {
                info!(
                    repo = repo_name,
                    branch = push_ref,
                    sender = sender,
                    commits = commit_count,
                    "Trigger matched - starting deployment"
                );
                
                // Fire and Forget (Async Task)
                let config_clone = repo_config.clone();
                let name_clone = repo_name.to_string();
                let token = state.config.git_token.clone();
                
                tokio::spawn(async move {
                    ops::perform_update(name_clone, config_clone, token).await;
                });
                
                return StatusCode::OK;
            } else {
                debug!(
                    repo = repo_name,
                    received_ref = push_ref,
                    expected_ref = repo_config.branch.as_str(),
                    "Branch mismatch - ignoring"
                );
            }
        }
        None => {
            debug!(
                repo = repo_name,
                configured_repos = ?state.config.repos.keys().collect::<Vec<_>>(),
                "Repository not configured - ignoring"
            );
        }
    }

    StatusCode::OK
}