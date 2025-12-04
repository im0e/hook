#!/bin/bash

# Generate self-signed TLS certificates for development/testing
# Usage: ./scripts/generate-certs.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CERT_DIR="$PROJECT_ROOT/resources/certificates"

# Create certificates directory
mkdir -p "$CERT_DIR"

echo "Generating self-signed certificates in $CERT_DIR..."

openssl req -x509 \
    -newkey rsa:4096 \
    -keyout "$CERT_DIR/key.pem" \
    -out "$CERT_DIR/cert.pem" \
    -days 365 \
    -nodes \
    -subj "/C=US/ST=State/L=City/O=Development/CN=localhost"

echo ""
echo "✓ Certificates generated successfully!"
echo "  - Certificate: $CERT_DIR/cert.pem"
echo "  - Private Key: $CERT_DIR/key.pem"
echo ""
echo "Add this to your config.toml to enable HTTPS:"
echo ""
echo "[tls]"
echo "cert_path = \"resources/certificates/cert.pem\""
echo "key_path = \"resources/certificates/key.pem\""
