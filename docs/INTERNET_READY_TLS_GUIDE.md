# Internet-Ready TLS Configuration Guide

**Status:** ✅ IMPLEMENTED (December 17, 2025)  
**Crate:** `songbird-orchestrator`, `songbird-network-federation`  
**Dependencies:** `rustls`, `rcgen`, `axum-server`

## Overview

Songbird now supports TLS/HTTPS for secure federation over untrusted networks including the internet. This feature enables:

- **Encrypted communication** between federation peers
- **Self-signed certificates** for LAN/development
- **CA-signed certificates** for production internet deployments
- **Automatic certificate generation** when certificates are missing
- **Graceful fallback** to HTTP for local development

## Quick Start

### Enable TLS

```bash
# Enable TLS for Songbird orchestrator
export SONGBIRD_TLS_ENABLED=true

# Start Songbird
cargo run --bin songbird-orchestrator
```

Songbird will:
1. Check for existing certificates at `certs/songbird.crt` and `certs/songbird.key`
2. Generate self-signed certificates if they don't exist
3. Start HTTPS server with TLS encryption

### Custom Certificate Paths

```bash
# Use custom certificate locations
export SONGBIRD_TLS_CERT=/path/to/your/cert.pem
export SONGBIRD_TLS_KEY=/path/to/your/key.pem
export SONGBIRD_TLS_ENABLED=true
```

### Subject Alternative Names (SANs)

```bash
# Add multiple hostnames/IPs to certificate
export SONGBIRD_TLS_SANS="localhost,127.0.0.1,songbird.local,192.168.1.100"
```

### Custom Node Identity

```bash
# Set common name for certificate
export SONGBIRD_NODE_ID=songbird-tower-01
```

## Architecture

### TLS Components

```
┌─────────────────────────────────────────────────────────┐
│ Songbird Orchestrator                                   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ HTTP Server (start_http_server)                  │  │
│  │                                                  │  │
│  │  ┌─────────────────┐    ┌────────────────────┐ │  │
│  │  │ TLS Enabled?    │───▶│ start_https_server │ │  │
│  │  └─────────────────┘    └────────────────────┘ │  │
│  │           │                      │              │  │
│  │           │ No                   │ Yes          │  │
│  │           ▼                      ▼              │  │
│  │  ┌──────────────────┐  ┌────────────────────┐  │  │
│  │  │ Plain HTTP       │  │ HTTPS with rustls  │  │  │
│  │  │ (axum::serve)    │  │ (axum-server)      │  │  │
│  │  └──────────────────┘  └────────────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  TLS Certificate Manager (songbird-network-federation) │
│  ┌──────────────────────────────────────────────────┐  │
│  │ - TlsConfig                                      │  │
│  │ - TlsCertificateManager                          │  │
│  │ - Self-signed generation (rcgen)                 │  │
│  │ - Certificate loading (rustls-pemfile)           │  │
│  │ - rustls ServerConfig builder                    │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Certificate Generation Flow

```
Start
  │
  ├──▶ TLS Enabled?
  │     │
  │     ├── No ──▶ Start HTTP Server
  │     │
  │     └── Yes ──▶ Load TlsConfig from env
  │                 │
  │                 ├──▶ Check if certificates exist
  │                 │     │
  │                 │     ├── Yes ──▶ Load existing certs
  │                 │     │
  │                 │     └── No ──▶ Generate self-signed
  │                 │                │
  │                 │                ├──▶ Create cert params
  │                 │                ├──▶ Add SANs (DNS + IP)
  │                 │                ├──▶ Generate key pair
  │                 │                ├──▶ Self-sign certificate
  │                 │                └──▶ Save to files
  │                 │
  │                 ├──▶ Build rustls ServerConfig
  │                 │
  │                 └──▶ Start HTTPS Server (axum-server)
  │
  └──▶ Running
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_TLS_ENABLED` | `false` | Enable TLS/HTTPS support |
| `SONGBIRD_TLS_CERT` | `certs/songbird.crt` | Path to certificate file (PEM) |
| `SONGBIRD_TLS_KEY` | `certs/songbird.key` | Path to private key file (PEM) |
| `SONGBIRD_TLS_SANS` | `localhost,127.0.0.1` | Comma-separated SANs (DNS names and IPs) |
| `SONGBIRD_NODE_ID` | `songbird` | Common name for certificate |

## Production Deployment

### Using Let's Encrypt

```bash
# 1. Obtain certificates from Let's Encrypt
certbot certonly --standalone -d songbird.yourdomain.com

# 2. Configure Songbird to use Let's Encrypt certificates
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=/etc/letsencrypt/live/songbird.yourdomain.com/fullchain.pem
export SONGBIRD_TLS_KEY=/etc/letsencrypt/live/songbird.yourdomain.com/privkey.pem
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_PORT=443

# 3. Run Songbird (as root or with CAP_NET_BIND_SERVICE)
sudo -E cargo run --release --bin songbird-orchestrator
```

### Using Custom CA

```bash
# 1. Generate your own CA and certificates (example with openssl)
# Create CA
openssl req -x509 -new -nodes -keyout ca-key.pem -out ca-cert.pem -days 3650

# Create certificate signing request
openssl req -new -nodes -keyout songbird-key.pem -out songbird-csr.pem

# Sign certificate with CA
openssl x509 -req -in songbird-csr.pem -CA ca-cert.pem -CAkey ca-key.pem \
  -CAcreateserial -out songbird-cert.pem -days 365

# 2. Configure Songbird
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=songbird-cert.pem
export SONGBIRD_TLS_KEY=songbird-key.pem
```

### Multi-Tower Federation with TLS

```bash
# Tower 1
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_NODE_ID=tower-01
export SONGBIRD_PORT=8080
export SONGBIRD_TLS_SANS="localhost,127.0.0.1,tower-01.local,10.0.0.1"
cargo run --bin songbird-orchestrator &

# Tower 2
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_NODE_ID=tower-02
export SONGBIRD_PORT=8081
export SONGBIRD_TLS_SANS="localhost,127.0.0.1,tower-02.local,10.0.0.2"
cargo run --bin songbird-orchestrator &

# Test federation over TLS
curl -k https://localhost:8080/health
curl -k https://localhost:8081/health
```

**Note:** The `-k` flag bypasses certificate verification (for self-signed certs). For production, distribute your CA certificate and verify properly.

## Testing TLS

### 1. Start with TLS Enabled

```bash
SONGBIRD_TLS_ENABLED=true cargo run --bin songbird-orchestrator
```

### 2. Verify Certificate Generation

```bash
ls -lh certs/
# Should show: songbird.crt and songbird.key
```

### 3. Test HTTPS Endpoint

```bash
# With self-signed cert (insecure)
curl -k https://localhost:8080/health

# Verify TLS connection
openssl s_client -connect localhost:8080 -showcerts
```

### 4. Check Logs

Look for these log messages:
```
🔐 TLS enabled - configuring HTTPS server
🔐 Generating self-signed TLS certificate
✅ Self-signed certificate generated: certs/songbird.crt / certs/songbird.key
🔐 Loading TLS configuration
✅ TLS configuration loaded successfully
✅ TLS configuration loaded, HTTPS server listening on https://[::]:8080
   Certificate: certs/songbird.crt, Key: certs/songbird.key
```

## Security Considerations

### Self-Signed Certificates

✅ **Appropriate for:**
- LAN deployments
- Development and testing
- Private networks (VPN)
- When combined with other security layers

⚠️ **Not recommended for:**
- Public internet without additional security
- Production with untrusted networks
- When you need third-party verification

### CA-Signed Certificates

✅ **Recommended for:**
- Public internet deployments
- Production environments
- When federation peers need to verify identity
- Compliance requirements

### mTLS (Mutual TLS)

**Status:** 🚧 Roadmap (4-6 weeks)

Future enhancement will add mutual TLS authentication where both client and server present certificates:
- Per-tower certificate generation
- Trust management and certificate distribution
- Automatic certificate rotation
- Revocation handling

See: `showcase/02-federation/SOVEREIGN_SECURITY_READY.md` for mTLS roadmap

## Implementation Details

### Key Files

1. **`crates/songbird-network-federation/src/tls.rs`**
   - `TlsConfig`: Configuration struct
   - `TlsCertificateManager`: Certificate lifecycle management
   - `generate_self_signed_certificate()`: rcgen-based generation
   - `load_tls_config()`: rustls ServerConfig builder
   - `ensure_certificates()`: Automatic cert provisioning

2. **`crates/songbird-orchestrator/src/app/mod.rs`**
   - `start_http_server()`: TLS decision point
   - `start_http_server_plain()`: Plain HTTP (axum)
   - `start_https_server()`: HTTPS with TLS (axum-server)

### Dependencies

```toml
# In songbird-network-federation
rustls = "0.23"
rustls-pemfile = "2.0"
rcgen = "0.12"
tokio-rustls = "0.26"

# In songbird-orchestrator
axum-server = { version = "0.7", features = ["tls-rustls"] }
```

### Zero-Cost When Disabled

When `SONGBIRD_TLS_ENABLED=false` (default):
- No TLS overhead
- No certificate generation
- Plain HTTP only
- Zero performance impact

## Troubleshooting

### Certificate Generation Fails

```
Error: Certificate generation failed: Invalid DNS name
```

**Solution:** Check `SONGBIRD_TLS_SANS` for invalid characters or formats.

### Port Already in Use

```
Error: Address already in use (os error 98)
```

**Solution:** Songbird automatically finds an available port. Check logs for actual port used.

### Permission Denied (Port 443)

```
Error: Permission denied (os error 13)
```

**Solution:** 
- Run with sudo: `sudo -E cargo run`
- Or use capabilities: `sudo setcap CAP_NET_BIND_SERVICE=+eip target/release/songbird-orchestrator`
- Or use port >= 1024

### Self-Signed Certificate Warnings

Browsers and clients will warn about self-signed certificates. This is expected. Options:
1. Add exception in browser
2. Use `-k` flag with curl
3. Add CA to trusted store
4. Use proper CA-signed certificates

## Next Steps

1. ✅ **TLS Support** (DONE)
2. 🚧 **mTLS** - Mutual authentication (4-6 weeks)
3. 🚧 **WireGuard Integration** - VPN backend (6-8 weeks)
4. 🚧 **Certificate Rotation** - Automatic renewal (8-10 weeks)

## Related Documentation

- `showcase/02-federation/SOVEREIGN_SECURITY_READY.md` - Security roadmap
- `showcase/02-federation/SECURITY_STATUS.md` - Current security status
- `specs/FEDERATION_IMPLEMENTATION_SPECIFICATION.md` - Federation architecture
- `docs/root-essential/PRODUCTION_DEPLOYMENT_GUIDE.md` - Production setup

---

**Implementation Date:** December 17, 2025  
**Semantic Version Impact:** Minor (new feature, backward compatible)  
**Breaking Changes:** None (TLS disabled by default)

