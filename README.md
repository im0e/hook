
# hOOk

```
 _       ___    ___   _    
| |__   / _ \  / _ \ | | __
| '_ \ | | | || | | || |/ /
| | | || |_| || |_| ||   < 
|_| |_| \___/  \___/ |_|\_\
```

A lightweight, high-performance GitHub webhook handler written in Rust.

## Features

- 🚀 **High Performance** - Uses `simd-json` for blazing-fast JSON parsing
- 🔒 **Secure** - HMAC-SHA256 signature verification for webhook payloads
- 🔐 **HTTPS Support** - Optional TLS with `rustls`
- ⚡ **Async** - Built on `tokio` and `axum` for efficient concurrent handling
- 🎯 **Auto-Deploy** - Automatically pulls changes and runs deploy commands

## Quick Start

### 1. Install

```bash
cargo build --release
```

### 2. Configure

Create a `config.toml`:

```toml
host = "0.0.0.0"
port = 3000
secret = "your-github-webhook-secret"

# Optional: Enable HTTPS
[tls]
cert_path = "resources/certificates/cert.pem"
key_path = "resources/certificates/key.pem"

[repos."owner/repo-name"]
path = "/var/www/my-app"
branch = "refs/heads/main"
deploy_command = "bash ./deploy.sh"
```

### 3. Generate TLS Certificates (Optional)

```bash
./scripts/generate-certs.sh
```

### 4. Run

```bash
./target/release/hook
```

## GitHub Webhook Setup

1. Go to your repo → **Settings** → **Webhooks** → **Add webhook**
2. Set **Payload URL** to `https://your-server:3000/webhook`
3. Set **Content type** to `application/json`
4. Set **Secret** to match your `config.toml` secret
5. Select **Just the push event**

## Configuration Options

| Field | Description |
|-------|-------------|
| `host` | Bind address (e.g., `0.0.0.0`) |
| `port` | Server port |
| `secret` | GitHub webhook secret for HMAC verification |
| `tls.cert_path` | Path to TLS certificate (optional) |
| `tls.key_path` | Path to TLS private key (optional) |

### Repository Config

| Field | Description |
|-------|-------------|
| `path` | Local path to clone/pull the repository |
| `branch` | Branch ref to watch (e.g., `refs/heads/main`) |
| `deploy_command` | Command to run after pulling (optional) |

## Architecture

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│  GitHub Push    │────▶│   hOOk       │────▶│  Git Pull   │
│  Webhook        │     │  (verify +   │     │  + Deploy   │
└─────────────────┘     │   route)     │     └─────────────┘
                        └──────────────┘
```

## License

MIT