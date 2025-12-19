# 🔒 TLS Configuration Guide

## 🎯 Overview

Songbird uses **fail-secure by default** - TLS is **enabled** unless explicitly opted out.

This ensures production security while allowing easy local development.

---

## 🔐 Default Behavior

```bash
# Start Songbird (TLS enabled by default)
cargo run --bin songbird-orchestrator
# → HTTPS on port 8081 with self-signed certs
```

**What happens:**
1. ✅ Generates self-signed certificate automatically
2. ✅ Binds to HTTPS
3. ✅ All connections encrypted
4. ✅ Fail-secure by design

---

## 🛠️ Local Development (Disable TLS)

### Option 1: Environment Variable (Recommended)
```bash
# Disable TLS for local dev
export SONGBIRD_TLS_ENABLED=false
cargo run --bin songbird-orchestrator
# → HTTP on port 8081 (insecure, local only)
```

### Option 2: Config File
```toml
# config/local.toml
[server]
tls_enabled = false
```

### Option 3: Helper Script
```bash
# Quick local development
./scripts/start-local-http.sh
```

---

## 🌐 Cross-Tower Development

### For Toadstool Team (Distributed ML)

#### Problem
ToadStool needs to coordinate across towers, but TLS certificates don't match.

#### Solution Options

**Option A: Disable TLS for Local Testing** (Fastest)
```bash
# On each tower
export SONGBIRD_TLS_ENABLED=false
cargo run --bin songbird-orchestrator
```

✅ **Pros**: Works immediately, no cert issues  
⚠️ **Cons**: Insecure (local network only)

**Option B: Use Shared Self-Signed Cert** (Recommended)
```bash
# Generate shared cert once
./scripts/generate-dev-cert.sh

# Copy to all towers
scp certs/* user@tower-b:~/songbird/certs/
scp certs/* user@tower-c:~/songbird/certs/

# Start normally (TLS enabled with shared cert)
cargo run --bin songbird-orchestrator
```

✅ **Pros**: Encrypted, works across towers  
✅ **Cons**: One-time setup

**Option C: Production Certs** (Production Only)
```bash
# Use real certificates (Let's Encrypt, etc.)
export SONGBIRD_TLS_CERT=/path/to/cert.pem
export SONGBIRD_TLS_KEY=/path/to/key.pem
cargo run --bin songbird-orchestrator
```

---

## 🎯 Recommended Workflow for ToadStool

### Step 1: Quick Test (HTTP)
```bash
# Tower A (Eastgate)
export SONGBIRD_TLS_ENABLED=false
export SONGBIRD_PORT=8081
cargo run --bin songbird-orchestrator

# Tower B (Strandgate)
export SONGBIRD_TLS_ENABLED=false
export SONGBIRD_PORT=8081
cargo run --bin songbird-orchestrator

# Submit tasks
curl http://192.168.1.134:8081/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{"task": {"name": "ml_training", "gpu": true}}'
```

### Step 2: Add TLS Later (Production)
Once basic functionality works, enable TLS:

```bash
# Generate shared cert
./scripts/generate-dev-cert.sh --sans "192.168.1.134,192.168.1.135"

# Copy to towers
./scripts/deploy-certs.sh tower-b tower-c

# Restart with TLS (default)
cargo run --bin songbird-orchestrator
```

---

## 🔧 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_TLS_ENABLED` | `true` | Enable/disable TLS (fail-secure) |
| `SONGBIRD_TLS_CERT` | `certs/songbird.crt` | Certificate path |
| `SONGBIRD_TLS_KEY` | `certs/songbird.key` | Private key path |
| `SONGBIRD_TLS_SANS` | (auto) | Subject Alternative Names |
| `SONGBIRD_PORT` | `8081` | HTTP(S) port |

---

## 📜 Certificate Management

### Auto-Generated Self-Signed (Default)
```bash
# Songbird generates cert automatically if missing
cargo run --bin songbird-orchestrator
# → Creates certs/songbird.{crt,key}
```

**SANs included automatically:**
- `localhost`
- `127.0.0.1`
- Local network IP (auto-detected)
- Custom SANs from `SONGBIRD_TLS_SANS`

### Custom SANs
```bash
# Add specific IPs/hostnames
export SONGBIRD_TLS_SANS="192.168.1.134,tower-a.local,eastgate"
cargo run --bin songbird-orchestrator
```

### Manual Certificate
```bash
# Use your own certificate
export SONGBIRD_TLS_CERT=/path/to/cert.pem
export SONGBIRD_TLS_KEY=/path/to/key.pem
cargo run --bin songbird-orchestrator
```

---

## 🚀 Production Deployment

### Best Practices

1. **Always use TLS in production**
   ```bash
   # Don't set SONGBIRD_TLS_ENABLED=false in production!
   ```

2. **Use real certificates**
   ```bash
   # Let's Encrypt, corporate CA, etc.
   export SONGBIRD_TLS_CERT=/etc/letsencrypt/live/songbird/fullchain.pem
   export SONGBIRD_TLS_KEY=/etc/letsencrypt/live/songbird/privkey.pem
   ```

3. **Certificate rotation**
   ```bash
   # Restart Songbird to reload certificates
   systemctl restart songbird
   ```

---

## 🐛 Troubleshooting

### Error: "Could not automatically determine CryptoProvider"

**Cause**: Missing rustls crypto provider feature

**Fix**: Update Cargo.toml (should be included in v0.2.1+)
```toml
rustls = { version = "0.23", features = ["aws-lc-rs"] }
```

### Error: "Certificate verification failed"

**For local dev:**
```bash
# Option 1: Disable TLS
export SONGBIRD_TLS_ENABLED=false

# Option 2: Use shared cert
./scripts/generate-dev-cert.sh --shared
```

**For production:**
- Use valid certificates from trusted CA
- Ensure SANs match hostnames/IPs

### Error: "Port 8081 already in use"

```bash
# Use different port
export SONGBIRD_PORT=8082
cargo run --bin songbird-orchestrator
```

---

## 📊 Security Comparison

| Mode | Encryption | Cert Validation | Use Case |
|------|------------|-----------------|----------|
| **TLS (default)** | ✅ Yes | Self-signed | Development |
| **TLS (prod certs)** | ✅ Yes | ✅ Trusted CA | Production |
| **HTTP (opt-out)** | ❌ No | N/A | Local testing |

---

## 🎓 Philosophy: Fail-Secure by Default

### Why TLS is enabled by default:

1. **Security First**: Production deployments are secure by default
2. **No Surprises**: Can't accidentally deploy without TLS
3. **Easy Opt-Out**: Simple to disable for local dev
4. **Best Practice**: Industry standard for distributed systems

### When to disable TLS:

✅ **Good reasons:**
- Local development on trusted network
- Debugging TLS issues
- Testing across towers (temporary)

❌ **Bad reasons:**
- Production deployment
- Internet-facing services
- Cross-datacenter communication

---

## 📝 Quick Reference

```bash
# Development (HTTP)
export SONGBIRD_TLS_ENABLED=false
cargo run --bin songbird-orchestrator

# Development (HTTPS with self-signed)
cargo run --bin songbird-orchestrator

# Production (HTTPS with real certs)
export SONGBIRD_TLS_CERT=/path/to/cert.pem
export SONGBIRD_TLS_KEY=/path/to/key.pem
cargo run --bin songbird-orchestrator
```

---

## 🔗 Related Documentation

- [Deployment Guide](../../DEPLOYMENT_GUIDE.md)
- [Configuration Guide](../../CONFIGURATION_GUIDE.md)
- [Security Model](../architecture/SECURITY.md)

---

**Remember**: TLS is **enabled by default** for your safety! 🔒

Disable only for local development on trusted networks.

