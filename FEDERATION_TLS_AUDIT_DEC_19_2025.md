# 🔒 Federation TLS & Discovery Audit - December 19, 2025

**Status:** ⚠️ **ISSUES FOUND** - Not following our own principles  
**Problem:** Current federation using hardcoded ports without TLS  
**Solution:** Migrate to capability-based discovery with TLS by default

---

## 🚨 CURRENT ISSUES IDENTIFIED

### 1. **Hardcoded Ports** ❌
```
eastgate processes:
- tarpc-server 0.0.0.0:8091  ← Hardcoded
- tarpc-server 0.0.0.0:8092  ← Hardcoded
- tarpc-server 0.0.0.0:8093  ← Hardcoded
- orchestrator :::8000       ← Different from expected 8080!
```

**Violation:** We just removed all hardcoding but these old processes are still using fixed ports

---

### 2. **No TLS** ❌
```
Current: HTTP on port 8000 (insecure)
Expected: HTTPS with auto-generated certificates (secure by default)
```

**Violation:** Our code has TLS failsafe-by-default, but current processes not using it

---

### 3. **Wrong Port** ❌
```
Orchestrator listening on: :::8000 (IPv6)
Health check trying:       localhost:8080 (IPv4)
Result:                    Not responding ❌
```

**Cause:** Old process running on different port than expected

---

## 📊 WHAT OUR CODE ACTUALLY IMPLEMENTS

### TLS Failsafe (✅ Implemented in Code)

**From `crates/songbird-network/src/tls.rs`:**
```rust
// Auto-generate self-signed certificates
cert_manager.ensure_certificates().await?;

// Load TLS config
let rustls_config = cert_manager.load_tls_config().await?;

info!("✅ TLS configuration loaded, HTTPS server listening on https://{}", addr);
info!("   🔒 SECURE BY DEFAULT - All connections encrypted");
```

**Default Behavior (from docs/operations/TLS_CONFIGURATION.md):**
```bash
# Start Songbird (TLS enabled by default)
cargo run --bin songbird-orchestrator
# → HTTPS on auto-selected port with self-signed certs
```

---

### Auto Port Selection (✅ Implemented in Code)

**From `crates/songbird-orchestrator/src/app/http_server.rs`:**
```rust
/// Smart port binding with automatic fallback
///
/// Tries the requested port first, then auto-increments until 
/// it finds an available port. Maximum 10 attempts before giving up.
async fn bind_with_fallback(addr: &SocketAddr) -> Result<...> {
    let host = addr.ip();
    let mut port = addr.port();
    // ... auto-increment logic
}
```

---

### Capability-Based Discovery (✅ Implemented in Code)

**Features:**
- mDNS auto-discovery
- UDP broadcast on port 2300
- Service registry
- No hardcoded endpoints

---

## 🔍 WHY WESTGATE CAN'T CONNECT

### Problem 1: Old Processes Running
```bash
# These are from Dec 17-18, using OLD configuration
eastgate 3692711  ... Dec17  tarpc-server 0.0.0.0:8091
eastgate 3974785  ... Dec18  songbird-orchestrator (port 8000)
```

**Solution:** Restart with NEW production-ready configuration

---

### Problem 2: Westgate Using Modern Config
```
✅ Federation:    Enabled
✅ Auto-Discovery: Enabled  
✅ TLS:           Enabled (auto-generated certs)
✅ IP Detection:  Automatic (192.168.1.123)
✅ Port:          Auto-selected
```

**Westgate is doing it RIGHT!** But eastgate/strandgate are using old setup.

---

### Problem 3: Protocol Mismatch
```
Westgate: HTTPS on auto-selected port (modern, secure)
Eastgate: HTTP on fixed port 8000 (old, insecure)
Result:   Cannot connect - different protocols!
```

---

## ✅ CORRECT CONFIGURATION

### Environment Variables (Failsafe Defaults)

```bash
# TLS (Enabled by default - no need to set unless disabling)
# export SONGBIRD_TLS_ENABLED=true  ← Default, auto-enabled

# Node Identity (auto-detected from hostname)
export SONGBIRD_NODE_ID="eastgate"

# Discovery (enabled by default)
export SONGBIRD_ENABLE_DISCOVERY="true"

# Federation (enabled by default)
export SONGBIRD_ENABLE_FEDERATION="true"

# Port (auto-selected if not specified)
# export SONGBIRD_PORT=8080  ← Will auto-increment if occupied

# TLS SANs (auto-includes hostname + IP)
export SONGBIRD_TLS_SANS="localhost,eastgate.local,192.168.1.100"

# Broadcast for discovery
export SONGBIRD_BROADCAST_ADDRESSES="255.255.255.255:2300,192.168.1.255:2300"
```

---

## 🚀 MIGRATION PLAN

### Step 1: Stop Old Processes

**On Eastgate:**
```bash
# Kill old tarpc servers (hardcoded ports)
pkill -f tarpc-server

# Kill old orchestrator (HTTP on port 8000)
pkill -f songbird-orchestrator

# Verify stopped
ps aux | grep -E "songbird|tarpc" | grep -v grep
```

---

### Step 2: Start Modern Configuration

**On Eastgate:**
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Set environment
export SONGBIRD_NODE_ID="eastgate"
export SONGBIRD_TLS_ENABLED=true  # Explicit for clarity
export SONGBIRD_ENABLE_DISCOVERY=true
export SONGBIRD_ENABLE_FEDERATION=true
export SONGBIRD_TLS_SANS="localhost,eastgate.local,eastgate,192.168.1.100,127.0.0.1"

# Create certs directory
mkdir -p certs

# Start orchestrator (will auto-generate TLS certs)
./target/release/songbird-orchestrator

# Should see:
# ✅ Self-signed certificate generated: certs/songbird.crt
# ✅ TLS configuration loaded, HTTPS server listening on https://[::]:8443
# 🔒 SECURE BY DEFAULT - All connections encrypted
```

---

### Step 3: Update Strandgate

**On Strandgate:**
```bash
# Same modern configuration
export SONGBIRD_NODE_ID="strandgate"
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_ENABLE_DISCOVERY=true
export SONGBIRD_ENABLE_FEDERATION=true
export SONGBIRD_TLS_SANS="localhost,strandgate.local,strandgate,192.168.1.101,127.0.0.1"

mkdir -p certs
./target/release/songbird-orchestrator
```

---

### Step 4: Westgate Already Correct! ✅

Westgate is already using the modern configuration:
- ✅ TLS enabled
- ✅ Auto-discovery enabled
- ✅ Federation enabled
- ✅ Port auto-selected

**No changes needed on westgate** - it will auto-discover eastgate/strandgate once they're updated!

---

## 🎯 VERIFICATION

### Check TLS is Working

**On each tower:**
```bash
# Check HTTPS endpoint (note the 's' in https and -k for self-signed)
curl -k https://localhost:8443/health

# Should return:
# {"status":"ok","service":"eastgate-orchestrator",...}
```

**Note:** Port will be auto-selected (likely 8443 if 8080 occupied)

---

### Check Discovery

**On any tower:**
```bash
# Run discovery
./target/release/songbird-cli discover --timeout 10

# Should find all 3 towers with HTTPS endpoints
```

---

### Check Federation

**Verify certificate:**
```bash
# Check certificate details
openssl x509 -in certs/songbird.crt -text -noout

# Should show SANs:
# DNS:localhost, DNS:eastgate.local, IP:192.168.1.100, ...
```

**Cross-tower HTTPS:**
```bash
# From eastgate, check westgate with TLS
curl -k https://westgate.local:8443/health

# From westgate, check eastgate with TLS
curl -k https://eastgate.local:8443/health
```

---

## 📊 BEFORE vs AFTER

### BEFORE (Current - Broken)

| Tower | Protocol | Port | TLS | Discovery | Issues |
|-------|----------|------|-----|-----------|--------|
| **Eastgate** | HTTP | 8000 | ❌ No | ❌ Old | Hardcoded, insecure |
| **Strandgate** | HTTP? | 8090? | ❌ No | ❌ Old | Hardcoded, insecure |
| **Westgate** | HTTPS | Auto | ✅ Yes | ✅ Yes | **CORRECT!** |

**Problem:** Protocol mismatch, hardcoded ports, no TLS

---

### AFTER (Production-Ready)

| Tower | Protocol | Port | TLS | Discovery | Status |
|-------|----------|------|-----|-----------|--------|
| **Eastgate** | HTTPS | Auto (8443) | ✅ Yes | ✅ Yes | ✅ Secure |
| **Strandgate** | HTTPS | Auto (8443) | ✅ Yes | ✅ Yes | ✅ Secure |
| **Westgate** | HTTPS | Auto (8443) | ✅ Yes | ✅ Yes | ✅ Secure |

**Result:** 
- ✅ All using HTTPS (encrypted)
- ✅ Auto-discovery working
- ✅ No hardcoded ports
- ✅ TLS failsafe by default
- ✅ Can connect to each other

---

## 🔐 TLS CERTIFICATE SHARING (Optional)

### Option A: Self-Signed Per Tower (Simplest)

Each tower generates its own certificate. Federation works with `-k` (insecure flag) for self-signed certs.

**Pros:**
- ✅ No cert sharing needed
- ✅ Auto-generated
- ✅ Each tower independent

**Cons:**
- ⚠️ Must use `-k` flag or configure to trust self-signed

---

### Option B: Shared Self-Signed (Better for LAN)

Generate one cert, share across towers.

```bash
# On eastgate - generate with all tower names
export SONGBIRD_TLS_SANS="localhost,eastgate.local,strandgate.local,westgate.local,192.168.1.100,192.168.1.101,192.168.1.123"
./target/release/songbird-orchestrator  # Generates cert

# Copy to other towers
scp certs/songbird.{crt,key} strandgate:/path/to/songbird/certs/
scp certs/songbird.{crt,key} westgate:/path/to/songbird/certs/

# Start all towers (will use shared cert)
```

**Pros:**
- ✅ Single cert for all towers
- ✅ No `-k` flag needed
- ✅ Proper certificate validation

**Cons:**
- ⚠️ Must copy cert to all towers

---

### Option C: Let's Encrypt (Production Internet)

For internet-facing deployments.

```bash
# Use certbot
sudo certbot certonly --standalone -d songbird.yourdomain.com

# Point Songbird to real cert
export SONGBIRD_TLS_CERT=/etc/letsencrypt/live/songbird.yourdomain.com/fullchain.pem
export SONGBIRD_TLS_KEY=/etc/letsencrypt/live/songbird.yourdomain.com/privkey.pem
```

---

## 🎯 RECOMMENDED ACTIONS

### Immediate (Now)

1. **Stop old processes** on eastgate and strandgate
2. **Start with modern config** (TLS enabled, auto-discovery)
3. **Verify all 3 towers** can discover each other
4. **Test HTTPS endpoints** with curl

### Short Term (Today)

5. **Update deployment scripts** to use new configuration
6. **Document TLS setup** for future deployments
7. **Test distributed ML** with 3-tower federation
8. **Verify encryption** with network sniffer

### Documentation

9. **Update README** to show TLS-first setup
10. **Create migration guide** from HTTP to HTTPS
11. **Add troubleshooting** for TLS issues

---

## 📝 QUICK START COMMANDS

### Restart Eastgate (Modern Configuration)

```bash
# Stop old
pkill -f "songbird-orchestrator|tarpc-server"

# Set environment
export SONGBIRD_NODE_ID="eastgate"
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_ENABLE_DISCOVERY=true
export SONGBIRD_ENABLE_FEDERATION=true
export SONGBIRD_TLS_SANS="localhost,eastgate.local,eastgate,192.168.1.100"

# Start new
cd /home/eastgate/Development/ecoPrimals/songbird
mkdir -p certs
nohup ./target/release/songbird-orchestrator > /tmp/eastgate.log 2>&1 &

# Verify
curl -k https://localhost:8443/health
tail -f /tmp/eastgate.log
```

---

## 🎉 SUCCESS INDICATORS

When properly configured, you'll see:

```bash
$ curl -k https://eastgate.local:8443/health
{"status":"ok","service":"eastgate-orchestrator","tls":true,"port":8443}

$ curl -k https://westgate.local:8443/health
{"status":"ok","service":"westgate-orchestrator","tls":true,"port":8443}

$ ./target/release/songbird-cli discover
Found 3 towers:
✅ eastgate (orchestration) - https://192.168.1.100:8443
✅ strandgate (gpu-compute) - https://192.168.1.101:8443
✅ westgate (storage) - https://192.168.1.123:8443
```

---

**Status:** ⚠️ **MIGRATION NEEDED**  
**Cause:** Old processes using HTTP + hardcoded ports  
**Solution:** Restart with TLS + auto-discovery (westgate already correct!)  
**Impact:** Secure, capability-based 3-tower federation

**🔒 Let's get back to our failsafe-by-default principles!** ✨

