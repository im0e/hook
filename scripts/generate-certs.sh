#!/bin/bash

# Generate self-signed TLS certificates for development/testing
# Usage: ./scripts/generate-certs.sh [IP_ADDRESS]
# Example: ./scripts/generate-certs.sh 91.109.114.110

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CERT_DIR="$PROJECT_ROOT/resources/certificates"
IP_ADDRESS="${1:-127.0.0.1}"

# Create certificates directory
mkdir -p "$CERT_DIR"

echo "Generating self-signed certificates in $CERT_DIR..."
echo "IP/Hostname: $IP_ADDRESS"

# Build alt_names section based on input
if [[ "$IP_ADDRESS" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    # Input is an IP address
    ALT_NAMES="DNS.1 = localhost
IP.1 = 127.0.0.1
IP.2 = $IP_ADDRESS"
else
    # Input is a hostname
    ALT_NAMES="DNS.1 = localhost
DNS.2 = $IP_ADDRESS
IP.1 = 127.0.0.1"
fi

# Create OpenSSL config with SAN
cat > "$CERT_DIR/openssl.cnf" <<EOF
[req]
default_bits = 4096
prompt = no
default_md = sha256
distinguished_name = dn
x509_extensions = v3_ext

[dn]
C = US
ST = State
L = City
O = Development
CN = $IP_ADDRESS

[v3_ext]
subjectAltName = @alt_names
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyEncipherment

[alt_names]
$ALT_NAMES
EOF

openssl req -x509 \
    -newkey rsa:4096 \
    -keyout "$CERT_DIR/key.pem" \
    -out "$CERT_DIR/cert.pem" \
    -days 365 \
    -nodes \
    -config "$CERT_DIR/openssl.cnf"

# Clean up config
rm "$CERT_DIR/openssl.cnf"

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
