# Copilot Instructions for the `hook` Project

## Project Overview
The `hook` project is a Rust-based application designed to handle GitHub webhook events. It processes incoming webhook payloads, verifies their authenticity, and triggers specific operations based on the repository and branch configuration.

### Key Components
- **`src/handlers.rs`**: Contains the main webhook handler function (`github_webhook`). This function:
  - Verifies the HMAC signature of incoming requests.
  - Parses JSON payloads using `simd-json` for high performance.
  - Routes requests based on repository and branch configurations.
  - Triggers asynchronous operations via `tokio::spawn`.
- **`src/config.rs`**: Defines the `AppConfig` structure, which holds application-wide configurations, including repository-specific settings.
- **`src/security.rs`**: Implements security-related utilities, such as HMAC signature verification.
- **`src/ops.rs`**: Contains the logic for performing repository updates or other operations triggered by webhook events.

### Data Flow
1. **Incoming Webhook**: The `github_webhook` function in `handlers.rs` receives the webhook payload.
2. **Verification**: The HMAC signature is verified using the secret key from the configuration.
3. **Parsing**: The JSON payload is parsed into a `simd-json` value.
4. **Routing**: The repository and branch are matched against the configuration.
5. **Operation Execution**: If a match is found, an asynchronous task is spawned to perform the operation.

## Developer Workflows

### Building the Project
Use the standard Rust build command:
```bash
cargo build
```

### Running the Project
Run the application locally:
```bash
cargo run
```

### Testing
Currently, no explicit test suite is defined. Add tests in the `tests/` directory and run them using:
```bash
cargo test
```

### Debugging
- Use `tracing` for logging. Ensure the `tracing` and `tracing-subscriber` crates are added to `Cargo.toml`.
- Logs are emitted at various levels (e.g., `info`, `warn`) to help trace execution flow.

## Project-Specific Conventions
- **JSON Parsing**: Use `simd-json` for high-performance JSON parsing. Ensure payloads are cloned into a `Vec<u8>` before parsing, as `simd-json` modifies inputs in-place.
- **Asynchronous Tasks**: Use `tokio::spawn` for fire-and-forget tasks, such as repository updates.
- **Configuration**: Store repository-specific settings in `AppConfig` and access them via the shared `AppState`.
- **Error Handling**: Return appropriate HTTP status codes (e.g., `401 Unauthorized`, `400 Bad Request`) for error scenarios.

## External Dependencies
- **`simd-json`**: For efficient JSON parsing.
- **`tokio`**: For asynchronous runtime.
- **`tracing`**: For structured logging.
- **`axum`**: For building the HTTP server and handling requests.

## Examples
### Webhook Handler
The `github_webhook` function in `src/handlers.rs` demonstrates the core patterns:
```rust
pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !security::verify_signature(&state.config.secret, &body, signature) {
        warn!("Unauthorized access attempt detected.");
        return StatusCode::UNAUTHORIZED;
    }

    let mut body_vec = body.to_vec();
    let payload = match simd_json::to_borrowed_value(&mut body_vec) {
        Ok(json) => json,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let repo_name = payload["repository"]["full_name"].as_str().unwrap_or("");
    let push_ref = payload["ref"].as_str().unwrap_or("");

    if let Some(repo_config) = state.config.repos.get(repo_name) {
        if push_ref == repo_config.branch {
            info!("Trigger detected for {} on {}", repo_name, push_ref);
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
```

## Future Improvements
- Add unit and integration tests for critical components.
- Enhance error handling and logging for better observability.
- Document additional workflows as the project evolves.

---

Feel free to update this document as the project grows and new patterns emerge.