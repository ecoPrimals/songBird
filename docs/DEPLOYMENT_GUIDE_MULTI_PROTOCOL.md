# 🚀 Multi-Protocol Songbird Deployment Guide

**Date:** December 17, 2025  
**Version:** 1.0  
**Status:** ✅ PRODUCTION READY

---

## 📊 Overview

This guide covers deploying Songbird with the new multi-protocol capabilities:
- **JSON-RPC 2.0** - Universal API access
- **BTSP Interface** - BearDog-ready encryption
- **Protocol Capability** - Intelligent protocol selection
- **TLS/HTTPS** - Secure connections

---

## 🎯 Quick Start

### Minimum Configuration (HTTP + JSON-RPC)

```bash
# Start Songbird with JSON-RPC enabled
cargo run --release --bin songbird-orchestrator

# JSON-RPC available at:
# http://localhost:8080/jsonrpc
```

### Recommended Configuration (HTTPS + JSON-RPC)

```bash
# Enable TLS for production
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=certs/songbird.crt
export SONGBIRD_TLS_KEY=certs/songbird.key

cargo run --release --bin songbird-orchestrator

# JSON-RPC available at:
# https://localhost:8443/jsonrpc
```

### Advanced Configuration (All Protocols)

```bash
# Full multi-protocol setup
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_JSONRPC_ENABLED=true
export SONGBIRD_BTSP_ENABLED=true
export SONGBIRD_BTSP_LOCAL_FALLBACK=true

cargo run --release --bin songbird-orchestrator
```

---

## 🔧 Configuration Options

### Core Settings

```bash
# Network
SONGBIRD_BIND_ADDRESS=[::]:8080    # IPv6 dual-stack
SONGBIRD_PORT=8080                 # HTTP port
SONGBIRD_HTTPS_PORT=8443           # HTTPS port (if TLS enabled)

# TLS Configuration
SONGBIRD_TLS_ENABLED=true          # Enable HTTPS
SONGBIRD_TLS_CERT=path/to/cert.pem
SONGBIRD_TLS_KEY=path/to/key.pem
SONGBIRD_TLS_SANS=localhost,127.0.0.1,tower.example.com
```

### Protocol Configuration

```bash
# JSON-RPC
SONGBIRD_JSONRPC_ENABLED=true      # Enable JSON-RPC endpoint
SONGBIRD_JSONRPC_PATH=/jsonrpc     # Endpoint path

# BTSP (BearDog Secure Tunnel Protocol)
SONGBIRD_BTSP_ENABLED=true         # Enable BTSP
SONGBIRD_BTSP_DISCOVERY=capability # Discovery method
SONGBIRD_BTSP_LOCAL_FALLBACK=true  # Fallback to local if BearDog unavailable
SONGBIRD_BTSP_GENETIC_AUTH=false   # Requires BearDog
```

---

## 📋 Deployment Scenarios

### Scenario 1: Development (Local Testing)

**Purpose:** Local development and testing

```bash
# Minimal setup
cargo run --bin songbird-orchestrator

# Access:
# - HTTP API: http://localhost:8080
# - JSON-RPC: http://localhost:8080/jsonrpc
```

**Features:**
- HTTP only (no TLS)
- JSON-RPC enabled
- Local BTSP for testing
- Fast iteration

---

### Scenario 2: Production (Single Tower)

**Purpose:** Production deployment on single server

```bash
# Generate self-signed certificate (or use Let's Encrypt)
mkdir -p certs
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout certs/songbird.key \
  -out certs/songbird.crt \
  -days 365 \
  -subj "/CN=songbird.example.com"

# Configure
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=certs/songbird.crt
export SONGBIRD_TLS_KEY=certs/songbird.key
export SONGBIRD_JSONRPC_ENABLED=true

# Deploy
cargo build --release
./target/release/songbird-orchestrator

# Access:
# - HTTPS API: https://songbird.example.com:8443
# - JSON-RPC: https://songbird.example.com:8443/jsonrpc
```

**Features:**
- TLS/HTTPS encryption
- JSON-RPC over HTTPS
- Production-grade security
- Internet-ready

---

### Scenario 3: Federation (Multi-Tower)

**Purpose:** Multiple towers communicating over internet

```bash
# Tower 1 Configuration
export SONGBIRD_TOWER_ID=tower-1
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=certs/tower1.crt
export SONGBIRD_TLS_KEY=certs/tower1.key
export SONGBIRD_BTSP_ENABLED=true

# Tower 2 Configuration
export SONGBIRD_TOWER_ID=tower-2
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=certs/tower2.crt
export SONGBIRD_TLS_KEY=certs/tower2.key
export SONGBIRD_BTSP_ENABLED=true

# Deploy both towers
./target/release/songbird-orchestrator
```

**Features:**
- Tower-to-tower encryption (TLS)
- BTSP for additional security layer
- Protocol capability negotiation
- Internet-safe federation

---

### Scenario 4: With BearDog Integration

**Purpose:** Full genetic cryptography with BearDog

```bash
# Ensure BearDog is running
# BearDog should be discoverable via capability system

# Configure Songbird
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_BTSP_ENABLED=true
export SONGBIRD_BTSP_DISCOVERY=capability
export SONGBIRD_BTSP_SECURITY_CAPABILITY=enterprise-security
export SONGBIRD_BTSP_GENETIC_AUTH=true  # Requires BearDog
export SONGBIRD_BTSP_KEY_LINEAGE=true   # Requires BearDog

# Deploy
./target/release/songbird-orchestrator

# Songbird will automatically discover and use BearDog
```

**Features:**
- BearDog genetic cryptography
- Key lineage tracking
- Multi-party consent
- Threshold key schemes
- Sovereign security with network effects

---

## 🧪 Testing Your Deployment

### Test JSON-RPC

```bash
# Test version endpoint
curl -X POST https://localhost:8443/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.version",
    "params": [],
    "id": 1
  }'

# Expected response:
{
  "jsonrpc": "2.0",
  "result": {
    "version": "0.1.0",
    "protocol": "JSON-RPC 2.0",
    "capabilities": ["discovery", "registry", "health", "protocol_negotiation"]
  },
  "id": 1
}
```

### Test Health

```bash
curl -X POST https://localhost:8443/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.health",
    "params": [],
    "id": 1
  }'
```

### Test Protocol Capability

```bash
curl -X POST https://localhost:8443/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.protocols",
    "params": [],
    "id": 1
  }'

# Should show available protocols:
# - HTTP, HTTPS, JSON-RPC, tarpc, BTSP
```

### Run Test Script

```bash
# Use provided test client
./examples/jsonrpc_client.sh

# Should run all tests and show results
```

---

## 🔐 Security Considerations

### TLS/HTTPS

**Recommended:**
- Use CA-signed certificates for production
- Rotate certificates regularly
- Use TLS 1.3
- Configure proper SANs

**Self-Signed Certificates:**
```bash
# Generate with SANs
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout songbird.key -out songbird.crt -days 365 \
  -subj "/CN=songbird.example.com" \
  -addext "subjectAltName=DNS:songbird.example.com,DNS:localhost,IP:127.0.0.1"
```

### BTSP

**Local Mode (Testing):**
- Uses AES-256-GCM
- Suitable for development
- NOT for production without BearDog

**BearDog Mode (Production):**
- Genetic cryptography
- Key lineage tracking
- Multi-party consent
- Production-grade security

### Firewall Rules

```bash
# Allow HTTPS
sudo ufw allow 8443/tcp

# Allow JSON-RPC (if needed separately)
sudo ufw allow 8080/tcp

# Block everything else
sudo ufw default deny incoming
```

---

## 📊 Monitoring

### Health Checks

```bash
# Simple health check
curl -f https://localhost:8443/health || exit 1

# JSON-RPC health check
curl -X POST https://localhost:8443/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"songbird.health","params":[],"id":1}' \
  | jq '.result.status' | grep -q "healthy"
```

### Logs

```bash
# View logs
journalctl -u songbird -f

# Or if running directly
./target/release/songbird-orchestrator 2>&1 | tee songbird.log
```

### Metrics

Monitor:
- Request rates (JSON-RPC methods)
- Response times
- Error rates
- Active connections
- Protocol usage

---

## 🔄 Upgrade Path

### From HTTP-Only to HTTPS

```bash
# 1. Generate certificates
./scripts/generate_certs.sh

# 2. Enable TLS
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TLS_CERT=certs/songbird.crt
export SONGBIRD_TLS_KEY=certs/songbird.key

# 3. Restart
systemctl restart songbird
```

### From Local BTSP to BearDog

```bash
# 1. Deploy BearDog (ensure running)

# 2. Enable BearDog features
export SONGBIRD_BTSP_GENETIC_AUTH=true
export SONGBIRD_BTSP_KEY_LINEAGE=true

# 3. Restart Songbird
# Will automatically discover and use BearDog
systemctl restart songbird
```

---

## 🆘 Troubleshooting

### JSON-RPC Not Responding

```bash
# Check if port is listening
lsof -i :8080

# Check logs for errors
tail -f songbird.log | grep jsonrpc

# Test with curl
curl -v http://localhost:8080/jsonrpc
```

### TLS Certificate Errors

```bash
# Verify certificate
openssl x509 -in certs/songbird.crt -text -noout

# Check key matches cert
openssl x509 -in certs/songbird.crt -noout -modulus | openssl md5
openssl rsa -in certs/songbird.key -noout -modulus | openssl md5
# Should match

# Test TLS connection
openssl s_client -connect localhost:8443
```

### BTSP Not Working

```bash
# Check if BearDog is running
curl http://localhost:8443/health

# Check BTSP configuration
env | grep BTSP

# Check logs
tail -f songbird.log | grep btsp
```

---

## 📋 Systemd Service

Create `/etc/systemd/system/songbird.service`:

```ini
[Unit]
Description=Songbird Universal Orchestrator
After=network.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
Environment="SONGBIRD_TLS_ENABLED=true"
Environment="SONGBIRD_TLS_CERT=/opt/songbird/certs/songbird.crt"
Environment="SONGBIRD_TLS_KEY=/opt/songbird/certs/songbird.key"
Environment="SONGBIRD_JSONRPC_ENABLED=true"
Environment="SONGBIRD_BTSP_ENABLED=true"
ExecStart=/opt/songbird/target/release/songbird-orchestrator
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable songbird
sudo systemctl start songbird
sudo systemctl status songbird
```

---

## 🎯 Best Practices

### Production Checklist

- ✅ Use TLS/HTTPS (never HTTP in production)
- ✅ Use CA-signed certificates
- ✅ Enable JSON-RPC over HTTPS only
- ✅ Configure firewall rules
- ✅ Set up monitoring and alerts
- ✅ Regular certificate rotation
- ✅ Regular security updates
- ✅ Test failover scenarios
- ✅ Document your configuration
- ✅ Implement backup strategy

### Performance Tuning

```bash
# Increase connection limits
export SONGBIRD_MAX_CONNECTIONS=5000

# Adjust worker threads
export TOKIO_WORKER_THREADS=8

# Enable performance monitoring
export SONGBIRD_METRICS_ENABLED=true
```

---

## 📚 Additional Resources

- **JSON-RPC Guide:** `docs/JSONRPC_GUIDE.md`
- **BTSP Interface:** `docs/BTSP_INTERFACE_GUIDE.md`
- **TLS Deployment:** `docs/INTERNET_READY_TLS_GUIDE.md`
- **Architecture:** `docs/MULTI_PROTOCOL_FEDERATION_PLAN.md`

---

## ✅ Deployment Verification

After deployment, verify:

```bash
# 1. Service is running
systemctl status songbird

# 2. Ports are listening
ss -tlnp | grep -E '8080|8443'

# 3. TLS is working
curl -k https://localhost:8443/health

# 4. JSON-RPC is responding
./examples/jsonrpc_client.sh

# 5. Logs show no errors
journalctl -u songbird --since "5 minutes ago" | grep -i error
```

**Expected:** All checks pass, no errors in logs

---

**Status:** ✅ PRODUCTION READY  
**Updated:** December 17, 2025  
**Support:** See documentation in `docs/` directory

---

*"Deploy with confidence. Multi-protocol Songbird is ready to soar!"* 🚀🔐✨

