# 🎵 Phase 2: Songbird Federation

**Goal**: Multiple Songbirds forming a coordinated mesh  
**Prerequisites**: Phase 1 completed, 2+ machines OR multiple ports  
**Time**: 30-60 minutes  
**Complexity**: Intermediate-Advanced

---

## 🎯 What You'll Learn

- Mesh formation between Songbirds
- Cross-tower service discovery
- Load balancing across nodes
- Automatic failover
- Multi-node metrics aggregation
- Dynamic node joining/leaving

---

## 📁 Demos

### 1. Mesh Formation (5 minutes)
**File**: `demos/01-mesh-formation.sh`

Create a mesh of 2-3 Songbird instances that discover each other.

```bash
./demos/01-mesh-formation.sh
```

**What it shows**:
- Automatic peer discovery (mDNS/DNS-SD)
- Mesh topology formation
- Node health propagation
- Federation coordinator election

**Architecture**:
```
Tower A (Port 8000) ←→ Tower B (Port 8001) ←→ Tower C (Port 8002)
         ↖                                          ↙
                    Fully connected mesh
```

---

### 2. Cross-Tower Discovery (5 minutes)
**File**: `demos/02-cross-tower-discovery.sh`

Register service on Tower A, discover it from Tower B.

```bash
./demos/02-cross-tower-discovery.sh
```

**What it shows**:
- Service registration on one node
- Discovery propagation across mesh
- Capability queries across towers
- Transparent routing

**Key Insight**: Services can be on ANY tower, discovery works everywhere!

---

### 3. Load Balancing (10 minutes)
**File**: `demos/03-load-balancing.sh`

Submit tasks and watch them distribute across the mesh.

```bash
./demos/03-load-balancing.sh
```

**What it shows**:
- Round-robin distribution
- Health-aware routing
- Load metrics collection
- Automatic rebalancing

**Demo Flow**:
1. Register 3 compute services (one per tower)
2. Submit 100 tasks
3. Watch distribution: ~33 tasks per tower
4. Observe latency and throughput

---

### 4. Failover (10 minutes)
**File**: `demos/04-failover.sh`

Kill a node and watch the mesh adapt.

```bash
./demos/04-failover.sh
```

**What it shows**:
- Node failure detection (<10 seconds)
- Automatic service rebalancing
- Request rerouting
- Graceful degradation
- Recovery when node returns

**Steps**:
1. Start 3-node mesh
2. Submit continuous tasks
3. Kill Tower B
4. Watch tasks redistribute to A & C
5. Restart Tower B
6. Watch it rejoin automatically

---

### 5. Multi-Tower Metrics (5 minutes)
**File**: `demos/05-multi-tower-metrics.sh`

Aggregated observability across the mesh.

```bash
./demos/05-multi-tower-metrics.sh
```

**What it shows**:
- Mesh-wide metrics dashboard
- Per-tower breakdowns
- Latency percentiles across towers
- Health status for entire mesh

---

### 6. Dynamic Joining (10 minutes)
**File**: `demos/06-dynamic-joining.sh`

Add and remove nodes at runtime.

```bash
./demos/06-dynamic-joining.sh
```

**What it shows**:
- Zero-configuration node joining
- Automatic capability propagation
- Service redistribution
- Graceful node removal
- Split-brain handling

---

## 🚀 Quick Start

### Local Simulation (Single Machine)
```bash
# Simulate 3 towers on different ports
./setup-local-federation.sh

# Run all demos
./run-all-demos.sh
```

### Multi-Machine Setup
```bash
# Tower A (192.168.1.144)
SONGBIRD_PORT=8000 \
SONGBIRD_FEDERATION=true \
./scripts/start-tower.sh

# Tower B (192.168.1.134)
SONGBIRD_PORT=8000 \
SONGBIRD_FEDERATION=true \
SONGBIRD_PEERS=192.168.1.144:8000 \
./scripts/start-tower.sh

# Tower C (192.168.1.207)
SONGBIRD_PORT=8000 \
SONGBIRD_FEDERATION=true \
SONGBIRD_PEERS=192.168.1.144:8000 \
./scripts/start-tower.sh
```

---

## 📊 Expected Output

### Mesh Formation
```
🎵 Starting Federation Demo...

Starting Tower A (Port 8000)...
✅ Tower A started

Starting Tower B (Port 8001)...
✅ Tower B started
🔗 Discovered peer: Tower A (8000)

Starting Tower C (Port 8002)...
✅ Tower C started
🔗 Discovered peer: Tower A (8000)
🔗 Discovered peer: Tower B (8001)

📊 Mesh Status:
   Nodes: 3
   Topology: Fully connected
   Health: All green
   Coordinator: Tower A
```

### Load Balancing
```
📤 Submitting 100 tasks...

📊 Task Distribution:
   Tower A: 34 tasks (34%)
   Tower B: 33 tasks (33%)
   Tower C: 33 tasks (33%)

✅ Perfect load distribution!
```

### Failover
```
⚡ Killing Tower B...

⚠️  Node failure detected (Tower B)
🔄 Rebalancing services...
✅ Tasks redistributed to Tower A & C

📊 New Distribution:
   Tower A: 50 tasks (50%)
   Tower C: 50 tasks (50%)

🔄 Restarting Tower B...
✅ Tower B rejoined mesh
📊 Load rebalanced automatically
```

---

## 🛠️ Configuration

### Federation Configuration
**File**: `configs/federation.toml`

```toml
[federation]
enabled = true
discovery_method = "mdns"
heartbeat_interval = "5s"
failure_threshold = 3

[discovery]
# mDNS for LAN
mdns_enabled = true
mdns_domain = "_songbird._tcp.local"

# DNS-SD for cloud
dns_sd_enabled = false

[coordination]
# Leader election
election_enabled = true
election_timeout = "10s"
```

### Per-Tower Configuration
```toml
# Tower A (Leader)
[node]
id = "tower-a"
role = "coordinator"
port = 8000

# Tower B (Worker)
[node]
id = "tower-b"
role = "worker"
port = 8001
peers = ["tower-a:8000"]
```

---

## 🏗️ Architecture

### Mesh Topology
```
┌──────────────────────────────────────────────────┐
│  FULL MESH (Recommended for <10 nodes)          │
│                                                   │
│     Tower A ←──→ Tower B                        │
│        ↑  ↖    ↗    ↑                          │
│        │     ↗     │                           │
│        ↓  ↗      ↓                           │
│     Tower C ←──→ Tower D                        │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  HUB-SPOKE (Alternative for >10 nodes)          │
│                                                   │
│           Tower A (Coordinator)                  │
│             ↙  ↓  ↘                           │
│      Tower B Tower C Tower D                    │
│        ↙↓↘    ↙↓↘    ↙↓↘                 │
│     Workers... Workers... Workers...            │
└──────────────────────────────────────────────────┘
```

### Discovery Flow
```
1. Tower B starts
2. Tower B broadcasts presence (mDNS)
3. Tower A receives broadcast
4. Tower A responds with mesh info
5. Tower B connects to all known peers
6. Tower B announces services
7. Mesh converges (<30 seconds)
```

---

## 💡 Key Concepts

### Federation vs Clustering
**Federation**: Loosely coupled, autonomous nodes  
**Clustering**: Tightly coupled, shared state

**Songbird uses Federation**:
- Nodes are independent
- No single point of failure
- Services can move between nodes
- Partial mesh failures OK

### Zero-Configuration Discovery
**mDNS** (LAN): Automatic peer discovery  
**DNS-SD** (Cloud): Service-based discovery  
**Environment** (Manual): Explicit peer list

**Priority**: mDNS > DNS-SD > Environment

### Load Balancing Strategies
1. **Round Robin**: Equal distribution
2. **Least Loaded**: Send to least busy
3. **Latency-Based**: Prefer low latency
4. **Capability-Weighted**: Prefer capable nodes

---

## 🔧 Troubleshooting

### Nodes Can't Discover Each Other
```bash
# Check firewall (mDNS needs UDP 5353)
sudo ufw allow 5353/udp

# Check mDNS service
systemctl status avahi-daemon

# Test manual discovery
curl http://192.168.1.144:8000/api/v1/federation/peers
```

### Split Brain Scenario
```bash
# Check coordinator election
curl http://localhost:8000/api/v1/federation/coordinator

# Force re-election
curl -X POST http://localhost:8000/api/v1/federation/elect
```

### High Latency Between Towers
```bash
# Check network latency
ping 192.168.1.134

# View tower-to-tower metrics
curl http://localhost:8000/api/v1/metrics/federation
```

---

## 📚 Next Steps

After mastering Phase 2:

1. **Proceed to Phase 3**: `../03-inter-primal/README.md`
   - Songbird + Toadstool integration
   - Distributed compute
   - Friend joining LAN mesh

2. **Experiment with Failure Modes**:
   - Network partitions
   - Asymmetric failures
   - Recovery scenarios

3. **Performance Testing**:
   - Measure mesh overhead
   - Profile cross-tower latency
   - Test at scale (10+ nodes)

---

## 🎯 Success Criteria

Phase 2 is complete when you can:

- [ ] Form a mesh of 2+ Songbirds
- [ ] Register service on one tower, discover from another
- [ ] Submit tasks that balance across towers
- [ ] Kill a node and see automatic failover
- [ ] View aggregated metrics across mesh
- [ ] Add/remove nodes dynamically

---

**Ready for the mesh?** Start with `./setup-local-federation.sh`!

🎵 **Experience the power of distributed orchestration!** 🎵

