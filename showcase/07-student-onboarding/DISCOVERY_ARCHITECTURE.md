# Discovery Architecture - How Songbird Finds Compute

**Core Principle:** Zero hardcoded IPs. Capability-based discovery at runtime.

---

## The Problem We Solved Today

**Before (Technical Debt):**
```rust
// Hardcoded mock responses - BAD!
let nodes = vec![
    "192.168.1.144:8000",  // Eastgate
    "192.168.1.134:8081",  // Strandgate
];
```

**After (Today's Fix):**
```rust
// Real capability-based discovery - GOOD!
let nodes = self.service_registry
    .find_by_capability(&["gpu-compute"])
    .await?;
```

---

## How Auto-Discovery Works

### 1. Node Registration (ToadStool)

When a compute node starts:

```rust
// ToadStool registers itself with federation
service_registry.register(ServiceRegistration {
    service_id: "toadstool-eastgate",
    node_name: "Eastgate",
    capabilities: vec![
        "gpu-compute".to_string(),
        "ml-training".to_string(),
        "distributed-training".to_string(),
    ],
    endpoints: vec![
        ServiceEndpoint {
            protocol: "http",
            address: self.local_ip,  // Auto-detected
            port: 8000,
            tls: false,
        }
    ],
    metadata: json!({
        "gpu": "RTX 3090",
        "vram_gb": 24,
        "cuda_version": "12.1",
    }),
}).await?;
```

**Key points:**
- Node knows **what it can do** (capabilities)
- Node advertises **where it is** (endpoints)
- Node provides **metadata** (GPU specs)
- **No coordinator decides this** - node self-describes

### 2. Songbird Discovery

When a student submits a task:

```rust
// Student task needs GPU compute
let task = StudentTask {
    script: "train.py",
    required_capabilities: vec!["gpu-compute"],
    gpu_memory_gb: 4,
};

// Songbird finds nodes that can handle this
let available_nodes = service_registry
    .find_by_capability(&["gpu-compute"])
    .await?;

// Filter by current availability
let nodes_with_capacity: Vec<_> = available_nodes
    .into_iter()
    .filter(|node| {
        node.metadata["available_vram_gb"].as_u64() >= 4
    })
    .collect();

// Route to least-loaded node
let selected = select_least_loaded(&nodes_with_capacity)?;

// Submit task
toadstool_client.submit_task(&selected.endpoint, task).await?;
```

**Key points:**
- Songbird **asks** registry what's available
- Filters by **capability + resources**
- Routes based on **current load**
- **Zero hardcoded knowledge**

### 3. Federation Registry

The registry maintains the live service map:

```rust
pub struct FederatedServiceRegistry {
    // In-memory registry of all services
    services: RwLock<HashMap<ServiceId, ServiceRegistration>>,
    
    // Last heartbeat from each service
    last_seen: RwLock<HashMap<ServiceId, Instant>>,
    
    // Capability index for fast lookups
    capabilities_index: RwLock<HashMap<String, Vec<ServiceId>>>,
}

impl FederatedServiceRegistry {
    pub async fn find_by_capability(&self, caps: &[&str]) -> Vec<ServiceRegistration> {
        let index = self.capabilities_index.read().await;
        let services = self.services.read().await;
        
        // Find all services with requested capabilities
        let mut matches = HashSet::new();
        for cap in caps {
            if let Some(service_ids) = index.get(*cap) {
                matches.extend(service_ids);
            }
        }
        
        // Return live services only (recent heartbeat)
        let now = Instant::now();
        matches
            .into_iter()
            .filter_map(|id| {
                let last_seen = self.last_seen.read().await.get(id)?;
                if now.duration_since(*last_seen) < Duration::from_secs(60) {
                    services.get(id).cloned()
                } else {
                    None  // Service timed out
                }
            })
            .collect()
    }
}
```

**Key points:**
- Registry is **the source of truth**
- Services that don't heartbeat are **removed**
- Lookups are **fast** (capability index)
- **Distributed** (registry can be federated too!)

---

## Deployment Flow

### Step 1: Start Federation Registry

```bash
# On any node (or distributed)
cargo run --bin songbird-registry -- --port 8000
```

Registry is now listening for service registrations.

### Step 2: Start Compute Nodes

```bash
# On Eastgate
cargo run --bin toadstool -- --registry http://192.168.1.144:8000

# On Strandgate  
cargo run --bin toadstool -- --registry http://192.168.1.144:8000
```

Each ToadStool node:
1. Detects local GPUs
2. Auto-discovers local IP
3. Registers with registry
4. Starts heartbeat

**Registry now knows:**
- Eastgate: RTX 3090, 24GB, available
- Strandgate: RTX 3070, 8GB, available

### Step 3: Start Songbird Coordinator

```bash
# On Windows laptop (or anywhere)
cargo run --bin songbird -- --registry http://192.168.1.144:8000
```

Songbird:
1. Connects to registry
2. Discovers available compute nodes
3. Starts listening for student connections
4. Routes tasks to discovered nodes

**No IPs configured!** Just registry URL.

### Step 4: Students Connect

```bash
# Student laptop
export SONGBIRD_URL="ws://192.168.1.50:8080"  # Your Windows laptop
python submit.py
```

Student submits task → Songbird queries registry → Routes to available GPU → Results return

---

## Configuration Comparison

### ❌ Wrong (Hardcoded)

```toml
[compute]
nodes = [
    "http://192.168.1.144:8000",  # Eastgate
    "http://192.168.1.134:8081",  # Strandgate
]
```

**Problems:**
- IP changes? Config breaks
- Add node? Update config
- Node offline? Still tries to route
- **This is the technical debt we eliminated!**

### ✅ Right (Discovery)

```toml
[federation]
registry_url = "http://localhost:8000/api/federation/registry"
discovery_enabled = true

[compute]
required_capabilities = ["gpu-compute"]
scheduling = "least-loaded"
```

**Benefits:**
- IP changes? Registry updates automatically
- Add node? It self-registers
- Node offline? Registry timeout removes it
- **Zero hardcoded knowledge!**

---

## Registry Location Strategies

### Option 1: Single Registry (Simple)

```
Registry on Eastgate (or any tower)
    ↓
All nodes register here
All coordinators discover here
```

**Config:**
```toml
registry_url = "http://192.168.1.144:8000/api/federation/registry"
```

**Pros:** Simple, works immediately  
**Cons:** Single point of failure (acceptable for v1)

### Option 2: Multi-Registry (Resilient)

```
Registry on Eastgate
Registry on Strandgate
    ↓ (gossip protocol)
Registries sync with each other
Nodes register with any
Coordinators query any
```

**Config:**
```toml
registry_urls = [
    "http://192.168.1.144:8000/api/federation/registry",
    "http://192.168.1.134:8081/api/federation/registry",
]
```

**Pros:** No single point of failure  
**Cons:** More complex (v2 feature)

### Option 3: Distributed Registry (Ultimate)

```
Every node runs registry
    ↓ (CRDT or Raft consensus)
Fully decentralized
Byzantine fault tolerant
```

**Pros:** Maximum resilience  
**Cons:** Complex, overkill for current scale (future)

---

## How This Differs from Traditional Systems

### Traditional HPC (SLURM, PBS)

```
Administrator configures:
  - Head node IP
  - Compute node IPs
  - Partition definitions
  - Resource allocations

Users submit to fixed queues.
```

**Rigid, centralized, requires admin.**

### EcoPrimals Songbird

```
Nodes self-register:
  - Auto-detect capabilities
  - Advertise endpoints
  - Update availability

Coordinators discover nodes.
Students submit to any coordinator.
```

**Flexible, decentralized, self-organizing.**

---

## Testing Discovery

### Verify Registry is Working

```bash
# Query registry directly
curl http://192.168.1.144:8000/api/federation/services

# Should return:
{
  "services": [
    {
      "service_id": "toadstool-eastgate",
      "capabilities": ["gpu-compute", "ml-training"],
      "endpoints": [{"address": "192.168.1.144", "port": 8000}]
    },
    {
      "service_id": "toadstool-strandgate",
      "capabilities": ["gpu-compute", "ml-training"],
      "endpoints": [{"address": "192.168.1.134", "port": 8081}]
    }
  ]
}
```

### Test Node Registration

```bash
# On ToadStool node
curl -X POST http://192.168.1.144:8000/api/federation/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "test-node",
    "capabilities": ["gpu-compute"],
    "endpoints": [{"address": "192.168.1.100", "port": 8000}]
  }'
```

### Test Discovery from Songbird

```bash
# Query for GPU compute capabilities
curl http://192.168.1.144:8000/api/federation/discover?capability=gpu-compute

# Should return available nodes
```

---

## Evolution Path

### V1 (Current - Today)
- ✅ Service registry implemented
- ✅ Capability-based discovery working
- ✅ Zero production mocks
- ⏳ Single registry deployment

### V2 (Next - January)
- Multi-registry with sync
- Heartbeat monitoring
- Auto-pruning dead nodes
- Load-based routing

### V3 (Future - Q1-Q2 2025)
- Distributed registry (CRDT)
- Cross-internet discovery
- BearDog-secured registration
- Capability negotiation

---

## Configuration for Students

**Students don't care about any of this!**

They just:
```bash
export SONGBIRD_URL="ws://YOUR.IP:8080"
python submit.py
```

All the discovery, routing, and coordination happens **transparently**.

That's the whole point of sovereign infrastructure: **complexity hidden, sovereignty preserved.**

---

**Key Takeaway:** NO hardcoded IPs anywhere. Registry provides discovery. Nodes self-register. Songbird discovers. Students benefit.

🎵🍄🔍

