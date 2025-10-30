# 🌐 Eastgate ↔ Strandgate Federation Setup Guide

> **⚠️ IMPORTANT**: This guide contains errors. See **[FEDERATION_SETUP_CORRECTED.md](FEDERATION_SETUP_CORRECTED.md)** for the working version.

**Objective**: Get your two Linux towers talking via Songbird federation  
**Date**: October 30, 2025  
**Status**: ⚠️ SUPERSEDED - Contains aspirational features not yet implemented  

---

## 🚨 Critical Issues Found

1. **CLI Arguments Don't Exist**: `--mode`, `--node-name`, `--listen`, `--bootstrap` are not implemented
2. **Build Failures**: Some systems experience `module core not found` errors
3. **Federation Not Ready**: Auto-discovery and mesh formation are still in development

**Use the corrected guide instead**: [FEDERATION_SETUP_CORRECTED.md](FEDERATION_SETUP_CORRECTED.md)

---

# Original Guide (FOR REFERENCE ONLY)

---

## 🎯 Tower Configuration

### **Eastgate** (Dev/Orchestrator)
- **CPU**: 20 cores (i9-12900K)
- **GPU**: RTX 4070
- **Role**: Main orchestrator + development
- **IP**: *Check with `ip addr` or `hostname -I`*

### **Strandgate** (Compute Beast)
- **CPU**: 64 cores (Dual EPYC) 🔥
- **GPU**: RTX 3070 FE
- **RAM**: 256GB ECC
- **Storage**: 56TB Mixed
- **Role**: Heavy parallel compute + storage

---

## 📋 Prerequisites

Both towers should:
- ✅ Be on the same LAN/network
- ✅ Have Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- ✅ Have Git installed
- ✅ Be able to ping each other

**Quick Network Check:**
```bash
# On Eastgate, find your IP
hostname -I

# On Strandgate, ping Eastgate
ping eastgate.local  # or use IP address
```

---

## 🚀 Step-by-Step Setup

### Step 1: Push from Eastgate to GitHub

```bash
# On Eastgate (where you are now)
cd ~/Development/ecoPrimals/songbird

# Commit recent changes
git add .
git commit -m "feat: add federation setup and capability showcase"

# Push to GitHub
git push origin main  # or your branch name
```

---

### Step 2: Clone on Strandgate

```bash
# SSH into Strandgate or work directly on it
ssh strandgate.local  # or use IP

# Create workspace
mkdir -p ~/Development/ecoPrimals
cd ~/Development/ecoPrimals

# Clone the repo
git clone https://github.com/ecoPrimals/songbird.git
cd songbird

# Build (this will take a few minutes first time)
cargo build --release --workspace
```

---

### Step 3: Configure Eastgate (Orchestrator)

```bash
# On Eastgate
cd ~/Development/ecoPrimals/songbird

# Create federation config
cat > eastgate-federation.env << 'EOF'
# Eastgate - Main Orchestrator Node
SONGBIRD_NODE_ID="eastgate-orchestrator"
SONGBIRD_NODE_NAME="Eastgate"
SONGBIRD_NODE_TYPE="orchestrator"
SONGBIRD_ROLE="coordination"

# Network configuration
SONGBIRD_LISTEN_ADDR="0.0.0.0:8000"
SONGBIRD_ADVERTISE_ADDR="eastgate.local:8000"
SONGBIRD_DISCOVERY_PORT="8100"
SONGBIRD_FEDERATION_PORT="8200"

# Capabilities
SONGBIRD_CAPABILITIES="orchestration,coordination,discovery,ai,development"
SONGBIRD_CPU_CORES="20"
SONGBIRD_MEMORY_GB="192"
SONGBIRD_GPU_MODEL="RTX_4070"

# Discovery settings
SONGBIRD_DISCOVERY_ENABLED="true"
SONGBIRD_DISCOVERY_PROTOCOLS="mdns,upnp"
SONGBIRD_DISCOVERY_INTERVAL="5s"

# Federation
SONGBIRD_FEDERATION_ENABLED="true"
SONGBIRD_AUTO_DISCOVERY="true"

# Logging
RUST_LOG="info,songbird=debug"
EOF

# Load configuration and start Songbird
source eastgate-federation.env

cargo run --release --bin songbird-orchestrator -- \
  --mode federation \
  --node-name "Eastgate" \
  --listen 0.0.0.0:8000
```

---

### Step 4: Configure Strandgate (Compute Beast)

```bash
# On Strandgate
cd ~/Development/ecoPrimals/songbird

# Create federation config
cat > strandgate-federation.env << 'EOF'
# Strandgate - Parallel Compute Node
SONGBIRD_NODE_ID="strandgate-compute"
SONGBIRD_NODE_NAME="Strandgate"
SONGBIRD_NODE_TYPE="compute"
SONGBIRD_ROLE="parallel_compute"

# Network configuration
SONGBIRD_LISTEN_ADDR="0.0.0.0:8000"
SONGBIRD_ADVERTISE_ADDR="strandgate.local:8000"
SONGBIRD_DISCOVERY_PORT="8100"
SONGBIRD_FEDERATION_PORT="8200"

# Capabilities (BEAST MODE!)
SONGBIRD_CAPABILITIES="parallel_compute,storage,gpu_compute,cpu_heavy"
SONGBIRD_CPU_CORES="64"
SONGBIRD_MEMORY_GB="256"
SONGBIRD_GPU_MODEL="RTX_3070_FE"
SONGBIRD_STORAGE_TB="56"

# Discovery settings
SONGBIRD_DISCOVERY_ENABLED="true"
SONGBIRD_DISCOVERY_PROTOCOLS="mdns,upnp"
SONGBIRD_DISCOVERY_INTERVAL="5s"

# Federation - Bootstrap to Eastgate
SONGBIRD_FEDERATION_ENABLED="true"
SONGBIRD_AUTO_DISCOVERY="true"
SONGBIRD_BOOTSTRAP_NODES="eastgate.local:8000"

# Logging
RUST_LOG="info,songbird=debug"
EOF

# Load configuration and start Songbird
source strandgate-federation.env

cargo run --release --bin songbird-orchestrator -- \
  --mode federation \
  --node-name "Strandgate" \
  --listen 0.0.0.0:8000 \
  --bootstrap eastgate.local:8000
```

---

## ✅ Verification Steps

### 1. Check Federation Status

**On Eastgate:**
```bash
# Check if Strandgate is discovered
curl http://localhost:8000/api/federation/status | jq

# Expected output:
# {
#   "federation_status": "healthy",
#   "active_nodes": 2,
#   "nodes": [
#     {"name": "Eastgate", "role": "orchestrator", "cores": 20},
#     {"name": "Strandgate", "role": "compute", "cores": 64}
#   ]
# }
```

### 2. Check Network Discovery

**Watch the logs on both machines** - you should see:

**Eastgate logs:**
```
[INFO] Songbird orchestrator starting...
[INFO] Federation mode enabled
[INFO] Listening on 0.0.0.0:8000
[INFO] mDNS discovery active
[DEBUG] Discovered peer: strandgate.local:8000
[INFO] Federation mesh established with Strandgate
[INFO] Total federation capacity: 84 cores, 448GB RAM
```

**Strandgate logs:**
```
[INFO] Songbird orchestrator starting...
[INFO] Federation mode enabled
[INFO] Bootstrap nodes: eastgate.local:8000
[DEBUG] Connecting to bootstrap: eastgate.local:8000
[INFO] Federation mesh established with Eastgate
[INFO] Ready for compute workloads
```

### 3. Check Capabilities

```bash
# On Eastgate - Query federation capabilities
curl http://localhost:8000/api/federation/capabilities | jq

# Expected: Should list capabilities from both towers
# - Eastgate: orchestration, ai, development
# - Strandgate: parallel_compute, storage, gpu_compute
```

---

## 🎮 Test Federation with a Deployment

Once connected, test deploying a service across both towers:

```bash
# On Eastgate
curl -X POST http://localhost:8000/api/byob/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "team": "test-federation",
    "deployment_type": "federated",
    "requirements": {
      "nodes": 2,
      "min_cores": 30,
      "capabilities": ["coordination", "parallel_compute"]
    },
    "services": {
      "orchestrator": {
        "resources": {"cpu": 8, "memory": "16GB"},
        "placement": "orchestrator"
      },
      "heavy_compute": {
        "resources": {"cpu": 32, "memory": "64GB"},
        "placement": "compute_optimized"
      }
    }
  }'
```

**What should happen:**
1. ✅ Songbird analyzes both towers
2. ✅ Places `orchestrator` service on Eastgate (orchestrator role)
3. ✅ Places `heavy_compute` on Strandgate (64 cores available!)
4. ✅ Establishes routing between them
5. ✅ Reports deployment success

---

## 🔍 Troubleshooting

### Issue: Towers can't discover each other

**Check 1: Network connectivity**
```bash
# On Eastgate
ping strandgate.local

# On Strandgate
ping eastgate.local
```

**Check 2: Firewall rules**
```bash
# Open ports on both machines
sudo ufw allow 8000/tcp
sudo ufw allow 8100/tcp
sudo ufw allow 8200/tcp
sudo ufw allow 5353/udp  # mDNS
```

**Check 3: mDNS resolution**
```bash
# Install avahi if needed
sudo apt install avahi-daemon avahi-utils

# Check mDNS
avahi-browse -a
```

---

### Issue: Bootstrap connection fails

**If Strandgate can't reach Eastgate:**

```bash
# On Strandgate, use IP instead of .local
# Find Eastgate's IP first
# Then update bootstrap:
export SONGBIRD_BOOTSTRAP_NODES="192.168.1.10:8000"  # Use actual IP
```

---

### Issue: Build fails on Strandgate

```bash
# On Strandgate - Update Rust and rebuild
rustup update stable
cargo clean
cargo build --release --workspace
```

---

## 📊 Expected Performance

### Federation Metrics
- **Discovery time**: < 5 seconds (mDNS on LAN)
- **Latency between towers**: 0.5-2ms (LAN)
- **Total capacity**: 84 CPU cores, 448GB RAM, 2 GPUs
- **Bandwidth**: 1-10Gbps (depending on your LAN switch)

### Resource Distribution
- **Eastgate**: Light coordination, dev work, AI tasks
- **Strandgate**: Heavy parallel compute, bulk storage, large models
- **Combined**: Way more capable than either alone!

---

## 🎯 What You Can Do Once Federated

### 1. Distributed AI Training
```bash
# Split training across both GPUs
# Eastgate: RTX 4070 (lighter model)
# Strandgate: RTX 3070 (data prep)
# Combined: Faster training pipeline
```

### 2. Parallel Compute Jobs
```bash
# Use Strandgate's 64 cores for parallel tasks
# Coordinate from Eastgate
# Distribute results efficiently
```

### 3. Storage Federation
```bash
# Strandgate's 56TB becomes available to Eastgate
# File distribution across towers
# Redundancy and backup
```

### 4. Development + Production
```bash
# Develop on Eastgate
# Deploy to Strandgate for testing
# Federated staging environment
```

---

## 🚀 Next Steps After Success

### Add More Towers!

Once Eastgate ↔ Strandgate is working:

**Add Northgate** (AI/ML flagship):
```bash
# On Northgate
git clone https://github.com/ecoPrimals/songbird.git
# Use similar config with bootstrap: eastgate.local:8000
# Now you have 3 towers federated!
```

**Add Southgate, Swiftgate, Westgate** - same process!

### Full Metal Matrix Federation

When all 6 towers are federated:
- **Total**: 148 CPU cores
- **Total**: 6 GPUs (5090, 4070, 3x 3070, 2070S)
- **Total**: ~700GB RAM
- **Total**: 140+ TB storage
- **Network effects**: Exponential capability improvement!

---

## 📝 Quick Reference Commands

### Start Federation

**Eastgate:**
```bash
cd ~/Development/ecoPrimals/songbird
source eastgate-federation.env
cargo run --release --bin songbird-orchestrator -- --mode federation --node-name Eastgate
```

**Strandgate:**
```bash
cd ~/Development/ecoPrimals/songbird
source strandgate-federation.env
cargo run --release --bin songbird-orchestrator -- --mode federation --node-name Strandgate --bootstrap eastgate.local:8000
```

### Check Status
```bash
curl http://localhost:8000/api/federation/status | jq
curl http://localhost:8000/api/federation/capabilities | jq
curl http://localhost:8000/api/metrics | jq
```

### Stop Federation
```bash
# Ctrl+C on both terminals
# Or send SIGTERM
pkill -f songbird-orchestrator
```

---

## 🎉 Success Criteria

You know it's working when:
- ✅ Both towers show "Federation mesh established" in logs
- ✅ `curl .../federation/status` shows 2 active nodes
- ✅ Total capacity = 84 cores (20 + 64)
- ✅ Both towers can route requests to each other
- ✅ Deployments can span both towers

---

## 💡 Pro Tips

1. **Use tmux/screen** - Keep Songbird running in background:
   ```bash
   tmux new -s songbird
   # Run Songbird
   # Detach with Ctrl+B then D
   ```

2. **Systemd service** - Auto-start on boot:
   ```bash
   # Create systemd service for Songbird
   sudo systemctl enable songbird
   ```

3. **Monitoring** - Watch federation in real-time:
   ```bash
   watch -n 1 'curl -s http://localhost:8000/api/federation/status | jq'
   ```

4. **Log aggregation** - Collect logs from both towers:
   ```bash
   # Consider setting up centralized logging
   ```

---

**Status**: Ready to federate! 🚀  
**Difficulty**: Easy (10 minutes)  
**Payoff**: MASSIVE (84 cores working together!)

Let's get your towers talking! 🌐

