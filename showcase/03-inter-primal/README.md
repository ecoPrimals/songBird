# 🎵🍄 Phase 3: Inter-Primal Mesh (Songbird + Toadstool)

**Goal**: Complete distributed compute mesh with Songbird orchestrating Toadstool  
**Prerequisites**: Phase 1 & 2 completed, Toadstool built  
**Time**: 45-90 minutes  
**Complexity**: Advanced-Expert

---

## 🎯 What You'll Learn

- Songbird orchestrating Toadstool compute
- Distributed ML training across towers
- Friend joining LAN mesh (zero-config)
- Compute bridge for cross-tower deployment
- GPU-aware task routing
- Production mesh deployment

---

## 🌟 Featured Demo: Friend Joins LAN

**Scenario**: Your friend brings their laptop to your LAN

**What Happens** (Automatic):
1. Friend runs `./join-mesh.sh`
2. Songbird auto-discovers your mesh (mDNS)
3. Capabilities register automatically
4. Work starts routing to their machine
5. If they run Toadstool, GPU tasks route there

**Demo**: `./demos/03-lan-join-demo.sh`  
**Time**: 5 minutes  
**Result**: Zero-configuration mesh expansion!

---

## 📁 Demos

### 1. Simple Compute Task (5 minutes)
**File**: `demos/01-simple-compute.sh`

Songbird orchestrates Toadstool for basic compute.

```bash
./demos/01-simple-compute.sh
```

**What it shows**:
- Songbird discovering Toadstool instance
- Submitting compute task via Songbird API
- Toadstool executing workload
- Results returning through Songbird
- Transparent orchestration

---

### 2. Distributed ML Training (30 minutes)
**File**: `demos/02-distributed-ml.sh`

**Based on**: `experiments/imagenet_training/`

3-tower distributed PyTorch training orchestrated by Songbird.

```bash
./demos/02-distributed-ml.sh
```

**What it shows**:
- Songbird coordinating 3 Toadstool instances
- GPU detection and assignment
- Distributed PyTorch DDP setup
- Training across towers
- Real-time monitoring
- Results aggregation

**Architecture**:
```
    Songbird (Tower A)
          ↓
   Coordinates:
          ├→ Toadstool (Tower A) - GPU: RTX 4070 - Rank 0
          ├→ Toadstool (Tower B) - GPU: RTX 3070 - Rank 1
          └→ Toadstool (Tower C) - GPU: RTX 3090 - Rank 2
```

---

### 3. Friend Joins LAN (10 minutes) 🌟
**File**: `demos/03-lan-join-demo.sh`

**THE SHOWCASE DEMO** - Zero-configuration mesh joining.

```bash
# On your machine (Tower A)
./setup-mesh.sh

# On friend's laptop (Tower D)
./join-mesh.sh

# Watch them join automatically!
./watch-mesh-grow.sh
```

**What it shows**:
- Zero manual configuration
- Automatic mDNS discovery
- Capability announcement
- Immediate work distribution
- Dynamic load rebalancing
- Friend can leave/rejoin anytime

**Real-World Value**: *"My friend shows up with their gaming rig, plugs into LAN, runs one script, and we're training ML models across 4 GPUs"*

---

### 4. Compute Bridge (15 minutes)
**File**: `demos/04-compute-bridge.sh`

**From experiments**: Songbird deploying across Songbird instances.

```bash
./demos/04-compute-bridge.sh
```

**What it shows**:
- Songbird updating remote Songbird
- Deploying Toadstool to new tower
- Cross-tower primal coordination
- SSH-free deployment
- Rolling updates across mesh

**Use Case**: Deploy or update primals across entire mesh from one command.

---

### 5. GPU Orchestration (15 minutes)
**File**: `demos/05-gpu-orchestration.sh`

GPU-aware intelligent routing.

```bash
./demos/05-gpu-orchestration.sh
```

**What it shows**:
- GPU capability advertisement
- Task complexity analysis
- Automatic GPU routing
- CPU fallback if no GPU
- Mixed workload optimization

**Task Routing**:
```
Light Task → Any Toadstool (CPU)
Heavy Task → Toadstool with GPU
ML Training → Multi-GPU Toadstool cluster
```

---

### 6. Production Mesh (45 minutes)
**File**: `demos/06-production-mesh.sh`

Complete production deployment.

```bash
./demos/06-production-mesh.sh
```

**What it shows**:
- 3+ towers full deployment
- Monitoring and observability
- Resilience testing
- Performance optimization
- Real workload simulation

---

### 7. Zero-Config Demo (10 minutes)
**File**: `demos/07-zero-config-everything.sh`

**The Magic Demo** - Everything automatic.

```bash
./demos/07-zero-config-everything.sh
```

**What it shows**:
- Start mesh with one command
- All discovery automatic
- All routing automatic
- All load balancing automatic
- All failover automatic

**Philosophy**: *"It just works"*

---

## 🚀 Quick Start

### Prerequisites
```bash
# Build Songbird
cd /home/eastgate/Development/ecoPrimals/songbird
cargo build --release

# Build Toadstool
cd /home/eastgate/Development/ecoPrimals/toadstool
cargo build --release
```

### Setup Mesh (Your Machine)
```bash
cd showcase/03-inter-primal
./setup-mesh.sh
```

### Friend Joins (Their Machine)
```bash
# They just need this one script!
./join-mesh.sh
```

### Run Featured Demo
```bash
./demos/03-lan-join-demo.sh
```

---

## 📊 Expected Output

### Simple Compute
```
🎵 Songbird discovering Toadstool...
✅ Found Toadstool at http://localhost:9000

📤 Submitting compute task...
  Task: process_data
  Input: 1000 records

🍄 Toadstool executing...
✅ Task complete (2.3s)

📊 Results:
  Records processed: 1000
  Compute time: 2.1s
  Overhead: 0.2s (orchestration)
```

### Friend Joins LAN
```
👋 Friend's laptop joining mesh...

🔍 Discovering existing mesh...
✅ Found 2 towers:
   - Tower A (Eastgate): http://192.168.1.144:8000
   - Tower B (Strandgate): http://192.168.1.134:8000

📝 Registering as Tower D...
✅ Registered with mesh

📢 Announcing capabilities:
   - compute_light
   - compute_heavy (GPU: RTX 3080)
   - storage_fast (NVMe SSD)

🎵 Mesh updated:
   Nodes: 3 → 4
   GPUs: 2 → 3
   Capacity: +35%

✅ Ready to receive work!

💬 "Welcome to the mesh, friend!"
```

### Distributed ML
```
🤖 Starting Distributed Training...

📊 Configuration:
  Framework: PyTorch DDP
  Dataset: ImageNet-100
  Towers: 3
  GPUs: 3 (RTX 4070, RTX 3070, RTX 3090)
  Batch size: 64 per tower

🎵 Songbird coordinating...
✅ Master (Rank 0) on Tower A
✅ Worker (Rank 1) on Tower B
✅ Worker (Rank 2) on Tower C

🍄 Toadstool executing training...

Epoch 1/20:
  Tower A: Batch 10/156 (6.4 it/s)
  Tower B: Batch 10/156 (5.8 it/s)
  Tower C: Batch 10/156 (7.1 it/s)
  
  Average: 6.4 it/s
  ETA: 4m 30s

✅ Training complete!
  Accuracy: 87.3%
  Time: 1h 12m
  Speedup: 2.8x (vs single GPU)
```

---

## 🏗️ Architecture

### Complete Mesh
```
┌────────────────────────────────────────────────────────┐
│                  SONGBIRD MESH                         │
│                                                         │
│   Tower A ←→ Tower B ←→ Tower C ←→ Tower D (Friend)  │
│      ↓          ↓          ↓          ↓               │
│  Toadstool  Toadstool  Toadstool  Toadstool          │
│  (RTX4070)  (RTX3070)  (RTX3090)  (RTX3080)          │
│                                                         │
│  Fully connected, auto-discovered, zero-config        │
└────────────────────────────────────────────────────────┘
```

### Task Routing
```
User → Songbird (Analyzes task)
                │
        ┌───────┴────────┬──────────┬──────────┐
        ↓                ↓           ↓           ↓
    Toadstool A     Toadstool B  Toadstool C  Toadstool D
    (If light)      (If heavy)   (If GPU)     (If available)
```

### Friend Join Flow
```
1. Friend's laptop boots
2. Runs join-mesh.sh
3. mDNS discovers Songbird
4. Songbird shares mesh topology
5. Laptop connects to all peers
6. Starts Toadstool (optional)
7. Announces capabilities
8. Songbird starts routing work
9. Mesh rebalances automatically

Total time: <30 seconds
User intervention: One command
```

---

## 💡 Key Insights

### Why This Is Revolutionary

**Traditional HPC**:
- Complex cluster configuration
- Manual node provisioning
- SSH key management
- Job scheduler setup
- Hours of setup time

**Songbird + Toadstool**:
- Zero configuration
- Automatic discovery
- No SSH needed
- Built-in orchestration
- <5 minutes setup time

### The "Friend Joins LAN" Value Prop

*"Your friend shows up for a LAN party. You're training an AI model. They run ONE script. Now you have their GPU too. No config. No setup. Just works."*

This is the **killer feature** that makes distributed compute accessible.

---

## 🛠️ Configuration

### Minimal Setup (Friend's Machine)
```bash
# friend-config.toml
[node]
auto_discover = true
auto_join = true
announce_capabilities = true

[discovery]
method = "mdns"
search_domain = "_songbird._tcp.local"
```

### Your Mesh (Host)
```toml
# mesh-config.toml
[federation]
enabled = true
accept_new_nodes = true
require_auth = false  # Or use simple token

[compute]
toadstool_enabled = true
auto_route_heavy = true
gpu_aware = true
```

---

## 🔧 Troubleshooting

### Friend Can't Discover Mesh
```bash
# Check mDNS
avahi-browse -a | grep songbird

# Check firewall
sudo ufw allow 5353/udp
sudo ufw allow 8000/tcp

# Manual discovery
SONGBIRD_PEERS=192.168.1.144:8000 ./join-mesh.sh
```

### Toadstool Won't Start
```bash
# Check Toadstool binary
which toadstool-server

# Test standalone
./target/release/toadstool-server --port 9000

# Check logs
tail -f /tmp/toadstool.log
```

### Tasks Not Routing
```bash
# Check Songbird sees Toadstool
curl http://localhost:8000/api/v1/capabilities/query?capability=compute_heavy

# Check Toadstool registered
curl http://localhost:9000/health
```

---

## 📚 References

### From Experiments
- **ML Training**: `../../../experiments/imagenet_training/`
- **Compute Bridge**: `../../../experiments/` (compute bridge evolution)
- **Test Plans**: `../../../experiments/local_tower_test_plan.md`

### From Toadstool
- **Showcase**: `../../../../toadstool/showcase/`
- **AI Orchestration**: `../../../../toadstool/showcase/real-world/06-ai-orchestration/`
- **Distributed Compute**: `../../../../toadstool/showcase/scripts/demo-distributed-compute.sh`

### Documentation
- **Integration**: `../../../docs/planning/TOADSTOOL_SONGBIRD_INTEGRATION_PLAN.md`
- **ML Integration**: `../../../docs/planning/TOADSTOOL_SONGBIRD_ML_INTEGRATION.md`
- **Compute Layer**: `../../../docs/reference/COMPUTE_LAYER_DECISION_GUIDE.md`

---

## 🎯 Success Criteria

Phase 3 is complete when you can:

- [ ] Songbird orchestrates Toadstool compute task
- [ ] Distributed ML training works across 3+ towers
- [ ] Friend can join mesh with one command (<5 min)
- [ ] GPU tasks automatically route to GPU-capable nodes
- [ ] Compute bridge deploys across towers
- [ ] Production mesh handles real workloads
- [ ] Everything works zero-configuration

---

## 🎉 The Grand Finale

### Demo for Visitors/Investors

```bash
# 1. Show your mesh (takes 30s)
./show-existing-mesh.sh

# 2. Friend joins (takes 30s)
# On their machine:
./join-mesh.sh

# 3. Show mesh grew automatically
./show-mesh-growth.sh

# 4. Submit distributed task
./submit-demo-task.sh

# 5. Watch it execute across all nodes
./watch-execution.sh

# Total time: 3 minutes
# Configuration needed: ZERO
# Wow factor: Maximum
```

---

**Ready for the complete mesh?** Start with `./setup-mesh.sh`!

🎵🍄 **This is the future of distributed computing** 🍄🎵

*Zero configuration. Maximum capability. True mesh computing.*

