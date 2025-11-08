# 🌉 Quick Start: Agnostic Compute Bridge

**Start a compute node in < 1 minute**

---

## 📦 Binary Location

```bash
/home/eastgate/Development/ecoPrimals/songbird/target/release/songbird-compute-bridge
```

---

## 🚀 Start Tower A Compute

```bash
export COMPUTE_SERVICE_NAME="Tower A Compute"
export COMPUTE_HOST=192.168.1.144
export COMPUTE_PORT=9000
export SONGBIRD_FEDERATION_ENDPOINT=http://192.168.1.144:8080
export COMPUTE_TOWER_ID=tower-a-orchestrator

./target/release/songbird-compute-bridge
```

**Expected:**
```
🌉 Starting Songbird Compute Bridge (Agnostic)
================================================
📊 Detected Resources:
   CPU Cores: 24
   Memory: 31GB
   GPUs: 1
   GPU Model: NVIDIA GeForce RTX 2070 SUPER
✅ Registered with Songbird federation
🚀 Compute Bridge listening on 192.168.1.144:9000
```

---

## 🏗️ Start Tower B Compute

```bash
# On Tower B (strandgate):
export COMPUTE_SERVICE_NAME="Tower B Massive CPU"
export COMPUTE_HOST=192.168.1.134
export COMPUTE_PORT=9000
export SONGBIRD_FEDERATION_ENDPOINT=http://192.168.1.144:8080
export COMPUTE_TOWER_ID=tower-b-strandgate

./target/release/songbird-compute-bridge
```

**Auto-detects:**
- 128 CPU cores ✅
- 251GB RAM ✅
- No GPU ✅
- Capabilities: compute, cpu, batch-processing, parallel-computing ✅

---

## ✅ Verify

```bash
# Check health
curl http://192.168.1.144:9000/health
# → OK

# Check federation
curl http://192.168.1.144:8080/api/federation/services/type/compute | jq '.'

# Should show both Tower A and Tower B compute services
```

---

## 🎯 That's It!

- No config files
- No hardcoded endpoints
- No manual capability lists
- No service-specific code

**Just environment variables and automatic detection.** 🚀
