# 🍄 ToadStool Integration Guide

## 🎯 Quick Solution for Distributed ML

**TL;DR**: Use HTTP mode for development, TLS is already configured!

```bash
# On each tower, run:
export SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator

# Or use the helper script:
./scripts/toadstool-quick-start.sh eastgate    # Tower A
./scripts/toadstool-quick-start.sh strandgate  # Tower B
```

---

## ✅ TLS Status: FIXED!

**Good news**: Songbird already has `rustls` with `ring` crypto provider configured!

The TLS blocker mentioned in your V2 README is **resolved**. You have two options:

### Option A: HTTP Mode (Fastest - Recommended for Dev)
Disable TLS for easy cross-tower communication:

```bash
export SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator
```

### Option B: HTTPS Mode (Production-Ready)
TLS works out of the box! Songbird auto-generates self-signed certificates:

```bash
# Just run normally - TLS enabled by default
cargo run --release --bin songbird-orchestrator
# → Auto-generates certs in certs/
# → HTTPS on port 8081
```

---

## 🚀 Quick Start for Distributed ML

### Step 1: Start Songbird on Each Tower

**Tower A (Eastgate - RTX 2070)**:
```bash
cd ~/Development/ecoPrimals/songbird
./scripts/toadstool-quick-start.sh eastgate
```

**Tower B (Strandgate - RTX 3070)**:
```bash
cd ~/Development/ecoPrimals/songbird
./scripts/toadstool-quick-start.sh strandgate
```

### Step 2: Verify Connectivity

```bash
# From ToadStool coordinator
curl http://192.168.1.134:8081/health  # Tower A
curl http://192.168.1.135:8081/health  # Tower B (adjust IP)
```

### Step 3: Submit Distributed Task

```rust
// From ToadStool coordinator
use reqwest::Client;
use serde_json::json;

let client = Client::new();

// Submit to Tower A
let response_a = client
    .post("http://192.168.1.134:8081/api/compute/task")
    .json(&json!({
        "task": {
            "name": "ml_training_partition_a",
            "gpu": true,
            "cpu_cores": 8.0,
            "memory_mb": 16384
        },
        "priority": 8,
        "timeout_secs": 600
    }))
    .send()
    .await?;

let job_id_a = response_a.json::<ComputeTaskResponse>().await?.job_id;

// Submit to Tower B
let response_b = client
    .post("http://192.168.1.135:8081/api/compute/task")
    .json(&json!({
        "task": {
            "name": "ml_training_partition_b",
            "gpu": true,
            "cpu_cores": 8.0,
            "memory_mb": 16384
        },
        "priority": 8,
        "timeout_secs": 600
    }))
    .send()
    .await?;

let job_id_b = response_b.json::<ComputeTaskResponse>().await?.job_id;
```

---

## 🔒 Security: Fail-Safe by Default

### Philosophy

Songbird follows **fail-secure by default**:
- ✅ TLS **enabled** by default (production-safe)
- ✅ Easy opt-out for development (`SONGBIRD_TLS_ENABLED=false`)
- ✅ No accidental insecure deployments

### Development vs Production

| Environment | TLS Setting | Command |
|-------------|-------------|---------|
| **Local Dev** | Disabled | `export SONGBIRD_TLS_ENABLED=false` |
| **Cross-Tower Dev** | Disabled | Use helper scripts |
| **Production** | Enabled (default) | No env var needed |

---

## 📊 V2 Architecture with Songbird

```
ToadStool Coordinator
    ↓
    ├─→ HTTP POST http://192.168.1.134:8081/api/compute/task
    │   (Tower A - Eastgate - RTX 2070)
    │   └─→ Train on 30k samples
    │
    └─→ HTTP POST http://192.168.1.135:8081/api/compute/task
        (Tower B - Strandgate - RTX 3070)
        └─→ Train on 30k samples
            ↓
    Poll for results:
    GET /api/compute/task/{job_id}
            ↓
    Aggregate results in coordinator
```

---

## 🎯 API Reference for ToadStool

### Submit Task

```bash
POST /api/compute/task
Content-Type: application/json

{
  "task": {
    "name": "ml_training",
    "gpu": true,
    "cpu_cores": 8.0,
    "memory_mb": 16384
  },
  "priority": 8,
  "timeout_secs": 600
}
```

**Response:**
```json
{
  "job_id": "uuid-here",
  "status": "queued",
  "assigned_tower": "tower-a"
}
```

### Check Task Status

```bash
GET /api/compute/task/{job_id}
```

**Response:**
```json
{
  "job_id": "uuid-here",
  "status": "completed",
  "result": { ... },
  "execution_time_ms": 45000
}
```

### Real-Time Updates (WebSocket)

```javascript
// Connect to task events
const ws = new WebSocket('ws://192.168.1.134:8081/api/ws/tasks');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Task event:', data);
  // { type: "task_event", task_id: "...", event_type: "Started", ... }
};
```

---

## 🔧 Configuration Options

### Environment Variables

```bash
# TLS Configuration
export SONGBIRD_TLS_ENABLED=false        # Disable TLS (dev only)
export SONGBIRD_TLS_CERT=path/to/cert    # Custom certificate
export SONGBIRD_TLS_KEY=path/to/key      # Custom key

# Server Configuration
export SONGBIRD_PORT=8081                # HTTP(S) port
export SONGBIRD_BIND_ADDRESS="::"        # Bind to all interfaces
export SONGBIRD_NODE_ID=tower-a          # Node identifier

# Logging
export RUST_LOG=info,songbird=debug      # Verbose logging
```

### Helper Scripts

```bash
# Quick start for ToadStool
./scripts/toadstool-quick-start.sh eastgate

# Generic HTTP mode
./scripts/start-local-http.sh

# Production mode (TLS enabled)
cargo run --release --bin songbird-orchestrator
```

---

## 🐛 Troubleshooting

### Issue: "Connection refused"

**Check**:
1. Is Songbird running? `ps aux | grep songbird`
2. Correct port? Default is 8081
3. Firewall? `sudo ufw allow 8081`

**Fix**:
```bash
# Restart Songbird
./scripts/toadstool-quick-start.sh eastgate
```

### Issue: "TLS certificate error"

**For development**:
```bash
# Disable TLS
export SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator
```

**For production**:
```bash
# Use shared certificate
./scripts/generate-dev-cert.sh --sans "192.168.1.134,192.168.1.135"
```

### Issue: "Task not executing"

**Check**:
1. GPU available? `nvidia-smi`
2. Worker running? Check logs
3. Task queued? `curl http://localhost:8081/api/compute/task/{job_id}`

---

## 📈 Expected Performance

| Metric | V1 (Simulated) | V2 (Real) |
|--------|----------------|-----------|
| **Accuracy** | 94.81% | 94-96% |
| **Time** | 75s | 60-90s |
| **GPUs** | 0 (CPU sim) | 2 (RTX 2070 + 3070) |
| **Network** | None | ~10-50ms latency |
| **Throughput** | N/A | ~2000 samples/sec |

---

## 🎓 Migration Path

### Phase 1: HTTP Mode (Now)
```bash
# Easy cross-tower communication
export SONGBIRD_TLS_ENABLED=false
```

✅ **Pros**: Works immediately, no cert setup  
⚠️ **Cons**: Insecure (local network only)

### Phase 2: Shared Self-Signed Cert (Later)
```bash
# Generate once, share across towers
./scripts/generate-dev-cert.sh --shared
```

✅ **Pros**: Encrypted, still easy  
✅ **Cons**: One-time setup

### Phase 3: Production Certs (Production)
```bash
# Use real certificates
export SONGBIRD_TLS_CERT=/path/to/prod/cert.pem
export SONGBIRD_TLS_KEY=/path/to/prod/key.pem
```

✅ **Pros**: Production-ready, trusted  
✅ **Cons**: Requires CA/Let's Encrypt

---

## 📝 Example: Complete Workflow

```bash
# 1. Start Songbird on both towers (HTTP mode)
# Tower A:
export SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator

# Tower B:
export SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator

# 2. From ToadStool coordinator, submit tasks
curl -X POST http://192.168.1.134:8081/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {"name": "ml_train_a", "gpu": true},
    "priority": 8
  }'

curl -X POST http://192.168.1.135:8081/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {"name": "ml_train_b", "gpu": true},
    "priority": 8
  }'

# 3. Monitor progress
watch -n 1 'curl -s http://192.168.1.134:8081/api/compute/task/{job_id}'

# 4. Collect results and aggregate
```

---

## 🎉 Summary

**TLS Blocker**: ✅ **RESOLVED** - `rustls` with `ring` already configured!

**Recommended Approach**:
1. Use HTTP mode for development (`SONGBIRD_TLS_ENABLED=false`)
2. Test distributed ML across towers
3. Enable TLS later for production

**Quick Start**:
```bash
./scripts/toadstool-quick-start.sh eastgate
```

**You're unblocked and ready to go!** 🚀

---

## 🔗 Related Documentation

- [TLS Configuration Guide](../operations/TLS_CONFIGURATION.md)
- [API Reference](../api/REST_API.md)
- [Deployment Guide](../../DEPLOYMENT_GUIDE.md)

---

**Questions?** Check the [Troubleshooting](#-troubleshooting) section or open an issue!

