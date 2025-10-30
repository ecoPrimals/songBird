# 🧪 Phase 1A Federation Testing Guide

**Status**: ✅ Code Complete - Ready to Test!  
**Date**: October 30, 2025  
**What's Being Tested**: Basic REST API federation between Eastgate and Strandgate

---

## ✅ Prerequisites

- [x] Phase 1A code complete
- [x] Builds successfully on both towers
- [x] Both towers on same network (192.168.1.x)
- [x] Code synced to GitHub

---

## 🚀 Test 1: Basic Federation Join

### Step 1: Start Eastgate (Bootstrap Node)

```bash
# On Eastgate
cd /home/eastgate/Development/ecoPrimals/songbird

# Set environment variables
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_NODE_NAME=Eastgate
export SONGBIRD_PORT=8080

# Start orchestrator
cargo run --release --bin songbird-orchestrator
```

**Expected Output**:
```
🚀 Starting Songbird Orchestrator
🏠 Running in standalone mode (federation disabled)  # ← Wrong! Should be federation mode
OR
🌐 Federation mode enabled
🔗 Will join federation via bootstrap: <none>  # ← Correct for bootstrap node
🌐 Starting HTTP server on 0.0.0.0:8080
✅ HTTP server listening on 0.0.0.0:8080
✅ Songbird Orchestrator started successfully
```

### Step 2: Verify Eastgate API

```bash
# From any machine on the network
curl http://192.168.1.144:8080/health
# Expected: "OK"

curl http://192.168.1.144:8080/api/federation/status
# Expected: JSON with Eastgate node
```

### Step 3: Start Strandgate (Joining Node)

```bash
# On Strandgate (after pulling from GitHub)
cd ~/Development/ecoPrimals/songbird
git pull origin type-unification-capability

# Set environment variables
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_NODE_NAME=Strandgate
export SONGBIRD_PORT=8080  # Same port, different machine
export SONGBIRD_BOOTSTRAP_ADDRESS=192.168.1.144:8080

# Start orchestrator
cargo run --release --bin songbird-orchestrator
```

**Expected Output**:
```
🚀 Starting Songbird Orchestrator
🌐 Federation mode enabled
🔗 Will join federation via bootstrap: 192.168.1.144:8080
🌐 Starting federation coordinator...
🤝 Joining federation via bootstrap: 192.168.1.144:8080
📡 Sending join request to: http://192.168.1.144:8080/api/federation/join
✅ Joined federation successfully
✅ Federation coordinator started successfully
🌐 Starting HTTP server on 0.0.0.0:8080
✅ HTTP server listening on 0.0.0.0:8080
✅ Songbird Orchestrator started successfully
```

### Step 4: Verify Federation Status

```bash
# Check Eastgate
curl http://192.168.1.144:8080/api/federation/status | jq

# Expected:
{
  "federation_id": "...",
  "active_nodes": 2,
  "nodes": [
    {
      "node_id": "...",
      "node_name": "Eastgate",
      "node_address": "0.0.0.0:8080",
      "capabilities": ["orchestrator"],
      "cpu_cores": 128,
      "memory_gb": 256,
      "status": "active",
      ...
    },
    {
      "node_id": "...",
      "node_name": "Strandgate",
      "node_address": "0.0.0.0:8080",
      "capabilities": ["orchestrator"],
      "cpu_cores": 128,
      "memory_gb": 256,
      "status": "active",
      ...
    }
  ],
  ...
}

# Check Strandgate
curl http://192.168.1.174:8080/api/federation/status | jq
# Should show same 2 nodes
```

---

## 🧪 Test 2: Heartbeat and Health Monitoring

**Time**: Wait 60 seconds after both nodes joined

### Check Heartbeats

```bash
# Watch logs on Eastgate
# Expected: Heartbeat messages every 30 seconds

# Watch logs on Strandgate
# Expected: Heartbeat messages every 30 seconds
```

### Test Node Timeout

```bash
# Kill Strandgate
# Wait 60 seconds
# Check Eastgate federation status
curl http://192.168.1.144:8080/api/federation/status | jq

# Expected: Strandgate shows status "inactive"
```

---

## 🧪 Test 3: Node List Endpoint

```bash
# List all nodes
curl http://192.168.1.144:8080/api/federation/nodes | jq

# Expected: Array of all registered nodes
[
  {
    "node_id": "...",
    "node_name": "Eastgate",
    ...
  },
  {
    "node_id": "...",
    "node_name": "Strandgate",
    ...
  }
]
```

---

## ❌ Common Issues & Fixes

### Issue 1: "Running in standalone mode"

**Problem**: `SONGBIRD_FEDERATION_ENABLED=true` not being read

**Fix**:
```bash
# Make sure you export, not just set
export SONGBIRD_FEDERATION_ENABLED=true

# Verify it's set
echo $SONGBIRD_FEDERATION_ENABLED  # Should print "true"

# Then run cargo
cargo run --release --bin songbird-orchestrator
```

### Issue 2: "Failed to connect to bootstrap node"

**Problem**: Network connectivity or wrong address

**Fix**:
```bash
# Verify Eastgate is reachable
ping 192.168.1.144

# Verify port is open
nc -zv 192.168.1.144 8080

# Check Eastgate is actually listening
ss -tlnp | grep 8080
```

### Issue 3: "Address already in use"

**Problem**: Port 8080 already taken

**Fix**:
```bash
# Use different port
export SONGBIRD_PORT=8081
export SONGBIRD_BOOTSTRAP_ADDRESS=192.168.1.144:8080  # Still connect to Eastgate on 8080

cargo run --release --bin songbird-orchestrator
```

### Issue 4: Build fails on Strandgate

**Problem**: Stale build artifacts

**Fix**:
```bash
cd ~/Development/ecoPrimals/songbird
git pull origin type-unification-capability
cargo clean
cargo build --release --bin songbird-orchestrator
```

---

## ✅ Success Criteria

- [x] Both nodes start without errors
- [x] Strandgate successfully joins Eastgate's federation
- [x] `/api/federation/status` shows both nodes as "active"
- [x] Both nodes send heartbeats every 30 seconds
- [x] Node timeout detection works (inactive after 60s)
- [x] API endpoints respond correctly on both towers

---

## 📝 Test Results Template

```
### Test 1: Basic Federation Join
- [x] Eastgate started: ✅/❌
- [x] Eastgate API responsive: ✅/❌
- [x] Strandgate started: ✅/❌
- [x] Strandgate joined federation: ✅/❌
- [x] Both nodes visible in status: ✅/❌

### Test 2: Heartbeats
- [x] Heartbeats sent every 30s: ✅/❌
- [x] Node timeout detection: ✅/❌

### Test 3: API Endpoints
- [x] /health: ✅/❌
- [x] /api/federation/status: ✅/❌
- [x] /api/federation/nodes: ✅/❌

### Issues Found
(List any issues encountered)

### Overall Result
Phase 1A: ✅ PASS / ❌ FAIL
```

---

## 🎯 Next Steps After Testing

### If Tests Pass ✅
Continue to Phase 1B: Service Federation

### If Tests Fail ❌
Debug and fix issues, then retest

