# 🌐 Federation Implementation Specification

**Version**: 1.0.0  
**Date**: October 30, 2025  
**Status**: ✅ **APPROVED FOR IMPLEMENTATION**  
**Authority**: ecoPrimals Architecture Council  
**Based On**: FRACTAL_FEDERATION_SPECIFICATION.md, SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md, HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md

---

## 📋 SPECIFICATION OVERVIEW

### Purpose
This specification defines the **complete implementation plan** for Songbird's federation system, combining:
1. **Immediate REST API Federation** (Track 1) - Production-ready HTTP coordination
2. **Complete Specification Compliance** (Track 2) - mDNS, Fractal Federation, Quorum Sensing

### Scope
- **Track 1 (REST)**: HTTP/REST API federation for immediate tower coordination
- **Track 2A (mDNS)**: Auto-discovery via mDNS/DNS-SD
- **Track 2B (Fractal)**: Hierarchical fractal federation system
- **Track 2C (Sovereign)**: Quorum-based sovereign federation
- **Track 2D (Hybrid)**: Multi-protocol transport layer (HTTP + tarpc + WebSocket)

### Stakeholders
- **Primary**: Songbird development team
- **Secondary**: Metal Matrix operators, ecoPrimals ecosystem
- **End Users**: Tower operators, friend networks, community grids, IoT deployments

---

## 🎯 ARCHITECTURAL PRINCIPLES (REQUIRED)

### 1. Dual-Track Architecture
**REQUIREMENT**: System MUST support both simple REST and complex protocols simultaneously

```
┌─────────────────────────────────────────────────────────────┐
│                    Federation Layer                          │
├─────────────────────────────────────────────────────────────┤
│  Track 1: REST API        │  Track 2: Complete Protocols    │
│  ├─ HTTP/REST             │  ├─ mDNS/DNS-SD                 │
│  ├─ Simple join/status    │  ├─ Fractal tiers              │
│  ├─ Service discovery     │  ├─ Quorum sensing             │
│  └─ IoT-friendly          │  └─ tarpc + WebSocket          │
├─────────────────────────────────────────────────────────────┤
│           Universal Capability Routing                       │
│           (Already Implemented ✅)                           │
└─────────────────────────────────────────────────────────────┘
```

### 2. Progressive Enhancement
**REQUIREMENT**: Each track MUST work independently and enhance each other

- **Track 1 alone**: Production-ready federation
- **Track 2 alone**: Advanced federation
- **Both together**: Optimal federation (REST as fallback, advanced protocols preferred)

### 3. Zero Breaking Changes
**REQUIREMENT**: Adding Track 2 MUST NOT break Track 1

- REST API endpoints remain stable
- Track 2 protocols add capabilities, don't replace
- Automatic protocol negotiation with fallback

---

## 🚀 TRACK 1: REST API FEDERATION SPECIFICATION

### Overview
**Timeline**: 1-3 days  
**Status**: 🚧 IMPLEMENTING  
**Purpose**: Immediate tower coordination via HTTP/REST

### Phase 1A: Basic HTTP Federation (Day 1)

#### Required API Endpoints

**POST /api/federation/join**
```json
// Request
{
  "node_id": "string (UUID)",
  "node_name": "string",
  "node_address": "string (IP:PORT)",
  "cpu_cores": "integer",
  "memory_gb": "integer",
  "gpu_model": "string (optional)",
  "storage_gb": "integer (optional)",
  "capabilities": ["string"],
  "status": "string (active|inactive)"
}

// Response
{
  "federation_id": "string (UUID)",
  "active_nodes": "integer",
  "nodes": [/* NodeRegistration array */],
  "joined_at": "ISO 8601 timestamp"
}
```

**GET /api/federation/status**
```json
// Response
{
  "federation_id": "string (UUID)",
  "active_nodes": "integer",
  "nodes": [/* NodeRegistration array */],
  "total_cpu_cores": "integer",
  "total_memory_gb": "integer",
  "total_storage_gb": "integer",
  "uptime_seconds": "integer"
}
```

**GET /api/federation/nodes**
```json
// Response
[
  {
    "node_id": "string",
    "node_name": "string",
    "node_address": "string",
    "cpu_cores": "integer",
    "memory_gb": "integer",
    "capabilities": ["string"],
    "status": "string",
    "last_heartbeat": "ISO 8601 timestamp"
  }
]
```

**POST /api/federation/heartbeat**
```json
// Request
{
  "node_id": "string",
  "timestamp": "ISO 8601",
  "status": "string",
  "metrics": {
    "cpu_usage_percent": "float",
    "memory_usage_percent": "float",
    "active_services": "integer"
  }
}

// Response
{
  "acknowledged": "boolean",
  "federation_status": "string"
}
```

#### Required Components

**1. FederationState**
```rust
// File: crates/songbird-network-federation/src/state.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct FederationState {
    pub federation_id: Uuid,
    pub nodes: Arc<RwLock<HashMap<String, NodeRegistration>>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    pub node_id: String,
    pub node_name: String,
    pub node_address: String,
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub gpu_model: Option<String>,
    pub storage_gb: Option<usize>,
    pub capabilities: Vec<String>,
    pub status: NodeStatus,
    pub joined_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Inactive,
    Unhealthy,
}
```

**2. FederationCoordinator Implementation**
```rust
// File: crates/songbird-network-federation/src/federation.rs

impl FederationCoordinator {
    pub async fn coordinate(&self, config: &FederationConfig) -> SongbirdResult<()> {
        if !config.enabled {
            return Ok(());
        }
        
        // Initialize federation state
        self.initialize_state(config).await?;
        
        // Join federation if bootstrap provided
        if let Some(bootstrap) = &config.bootstrap_address {
            self.join_federation(bootstrap, config).await?;
        }
        
        // Start background tasks
        self.start_heartbeat_loop(config).await?;
        self.start_health_monitor().await?;
        
        Ok(())
    }
}
```

**3. API Route Integration**
```rust
// File: crates/songbird-orchestrator/src/server/mod.rs

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        // Existing routes
        .route("/api/health", get(health_check))
        .route("/api/metrics", get(metrics))
        
        // NEW: Federation routes
        .nest("/api/federation", federation_routes())
        
        .with_state(state)
}
```

---

### Phase 1B: Service Discovery Across Towers (Day 2)

#### Required API Endpoints

**GET /api/federation/services**
```json
// Response
[
  {
    "service_id": "string (UUID)",
    "service_name": "string",
    "capabilities": ["string"],
    "node_id": "string",
    "node_address": "string",
    "endpoint": "string (URL)",
    "health": "string (healthy|unhealthy|unknown)",
    "registered_at": "ISO 8601"
  }
]
```

**POST /api/federation/services/register**
```json
// Request
{
  "service_name": "string",
  "capabilities": ["string"],
  "endpoint": "string",
  "metadata": {/* optional metadata */}
}

// Response
{
  "service_id": "string (UUID)",
  "registered": "boolean",
  "visible_to_nodes": ["string (node_ids)"]
}
```

#### Federated Service Registry
```rust
// File: crates/songbird-network-federation/src/service_registry.rs

pub struct FederatedServiceRegistry {
    local_services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
    remote_services_cache: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    federation_state: Arc<FederationState>,
}

impl FederatedServiceRegistry {
    pub async fn discover_services(&self, capability: &str) -> Vec<ServiceInfo> {
        let mut services = Vec::new();
        
        // 1. Local services
        services.extend(self.find_local_services(capability).await);
        
        // 2. Query each federated node
        for node in self.federation_state.nodes.read().await.values() {
            if let Ok(remote) = self.query_remote_services(&node.node_address, capability).await {
                services.extend(remote);
            }
        }
        
        // 3. Cache results
        self.cache_services(capability, &services).await;
        
        services
    }
}
```

---

### Phase 1C: Testing & Validation (Day 3)

#### Required Test Scenarios

**Test 1: Basic Federation Join**
```bash
# Start Eastgate
songbird tower start --name eastgate

# Start Strandgate (joins Eastgate)
songbird tower start --name strandgate --bootstrap 192.168.1.144:8080 --federation

# Verify
curl http://192.168.1.144:8080/api/federation/status | jq
# Expected: 2 nodes active
```

**Test 2: Service Discovery**
```bash
# Register service on Eastgate
curl -X POST http://192.168.1.144:8080/api/services \
  -d '{"name":"test-service","capabilities":["http"]}'

# Query from Strandgate
curl http://strandgate:8080/api/federation/services | jq
# Expected: Service from Eastgate visible
```

**Test 3: Node Failure Detection**
```bash
# Stop Eastgate
pkill -f songbird-orchestrator

# Wait 2 heartbeat intervals (60s)
sleep 60

# Check Strandgate status
curl http://strandgate:8080/api/federation/status | jq
# Expected: Eastgate marked as inactive
```

---

## 🏗️ TRACK 2: COMPLETE SPECIFICATION COMPLIANCE

### Track 2A: mDNS/DNS-SD Discovery (Weeks 1-2)

#### Required Components

**1. mDNS Service Registration**
```rust
// Dependency: mdns-sd = "0.10"

use mdns_sd::{ServiceDaemon, ServiceInfo};

pub struct MdnsDiscovery {
    daemon: ServiceDaemon,
    service_type: &'static str,
}

// REQUIRED: Register service
impl MdnsDiscovery {
    pub async fn register(&self, config: &NodeConfig) -> SongbirdResult<()> {
        let service_info = ServiceInfo::new(
            "_songbird-tower._tcp.local.",
            &config.node_name,
            &config.advertise_address,
            config.port,
            &[
                ("node_id", &config.node_id),
                ("capabilities", &config.capabilities.join(",")),
                ("cpu_cores", &config.cpu_cores.to_string()),
                ("memory_gb", &config.memory_gb.to_string()),
            ],
        )?;
        
        self.daemon.register(service_info)?;
        Ok(())
    }
    
    // REQUIRED: Discover other towers
    pub async fn discover(&self, timeout: Duration) -> SongbirdResult<Vec<DiscoveredTower>> {
        // Implementation per spec
    }
}
```

**2. Auto-Join Logic**
```rust
// REQUIRED: Automatic federation joining
pub async fn auto_join_best_tower(&self, discovered: Vec<DiscoveredTower>) -> SongbirdResult<()> {
    // Select best tower based on:
    // 1. Lowest latency (< 10ms preferred)
    // 2. Highest available capacity
    // 3. Matching capabilities
    
    if let Some(best) = self.select_optimal_tower(&discovered).await {
        self.join_federation(&best.address).await?;
    }
    
    Ok(())
}
```

---

### Track 2B: Fractal Federation System (Weeks 3-6)

**SPECIFICATION**: Implement `FRACTAL_FEDERATION_SPECIFICATION.md`

#### Required Tiers

**Edge Tier** (Required):
```rust
pub type EdgeFederation<Security, Storage> = ZeroCostFederationSystem<
    Security, Storage,
    50,   // MAX_NODES
    25,   // MAX_PROPOSALS  
    60,   // HEARTBEAT_INTERVAL
    60    // CONSENSUS_THRESHOLD
>;
```

**Regional Tier** (Required):
```rust
pub type RegionalFederation<Security, Storage> = ZeroCostFederationSystem<
    Security, Storage,
    500,  // MAX_NODES
    50,   // MAX_PROPOSALS
    45,   // HEARTBEAT_INTERVAL
    67    // CONSENSUS_THRESHOLD
>;
```

**Global Tier** (Optional):
```rust
pub type GlobalFederation<Security, Storage> = ZeroCostFederationSystem<
    Security, Storage,
    5000,  // MAX_NODES
    100,   // MAX_PROPOSALS
    30,    // HEARTBEAT_INTERVAL
    67     // CONSENSUS_THRESHOLD
>;
```

---

### Track 2C: Sovereign Quorum Sensing (Weeks 7-10)

**SPECIFICATION**: Implement `SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md`

#### Required Components

**1. Sovereign Node Identity** (Required)
**2. Quorum Signaling** (Required)
**3. Consensus Emergence** (Required)
**4. Anti-Centralization Measures** (Required)

---

### Track 2D: Hybrid Protocol (Weeks 11-12)

**SPECIFICATION**: Implement `HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md`

#### Required Protocol Support

**HTTP/REST** (Required): ✅ Already implemented
**tarpc** (Required): High-performance internal
**WebSocket** (Required): Real-time streaming
**Custom JSON RPC** (Optional): Universal adapter layer

---

## 📊 IMPLEMENTATION TIMELINE

| Track | Phase | Duration | Status | Dependencies |
|-------|-------|----------|--------|--------------|
| **Track 1** | 1A: HTTP Federation | Day 1 (4-6h) | 🚧 In Progress | None |
| **Track 1** | 1B: Service Discovery | Day 2 (4-6h) | 📋 Planned | 1A complete |
| **Track 1** | 1C: Testing | Day 3 (2-4h) | 📋 Planned | 1B complete |
| **Track 2** | 2A: mDNS Discovery | Weeks 1-2 | 📋 Planned | Track 1 complete |
| **Track 2** | 2B: Fractal Federation | Weeks 3-6 | 📋 Planned | 2A complete |
| **Track 2** | 2C: Sovereign Quorum | Weeks 7-10 | 📋 Planned | 2B complete |
| **Track 2** | 2D: Hybrid Protocol | Weeks 11-12 | 📋 Planned | 2C complete |

---

## ✅ ACCEPTANCE CRITERIA

### Track 1 Complete When:
- [ ] Two towers can join federation via HTTP
- [ ] Federation status API shows all nodes
- [ ] Services registered on one tower visible on others
- [ ] Heartbeat detection works (inactive nodes detected)
- [ ] Tests pass for all scenarios

### Track 2A Complete When:
- [ ] Towers auto-discover via mDNS on LAN
- [ ] No manual bootstrap needed
- [ ] Discovery works within 5 seconds
- [ ] Service metadata propagated via DNS-SD

### Track 2B Complete When:
- [ ] All tiers implemented (Edge, Regional, Global)
- [ ] Zero-cost abstractions verified (benchmarks)
- [ ] BearDog integration working
- [ ] ToadStool integration working

### Track 2C Complete When:
- [ ] Quorum sensing operational
- [ ] No centralization detected
- [ ] Sovereign node identity working
- [ ] Emergent consensus tested

### Track 2D Complete When:
- [ ] tarpc connections established
- [ ] Automatic protocol negotiation
- [ ] Performance metrics per protocol
- [ ] Fallback to HTTP working

---

## 🔒 SECURITY REQUIREMENTS

### Authentication (Required)
- [ ] Node identity verification (BearDog integration)
- [ ] Message signing
- [ ] Trust level calculation

### Authorization (Required)
- [ ] Capability-based access control
- [ ] Service permissions
- [ ] Data sovereignty enforcement

### Encryption (Required)
- [ ] TLS for HTTP endpoints
- [ ] Encrypted tarpc channels
- [ ] BearDog tunnel integration

---

## 📝 DOCUMENTATION REQUIREMENTS

### User Documentation (Required)
- [ ] Tower operator guide
- [ ] Federation setup guide
- [ ] Troubleshooting guide

### API Documentation (Required)
- [ ] REST API reference
- [ ] Protocol specifications
- [ ] Integration examples

### Developer Documentation (Required)
- [ ] Architecture overview
- [ ] Implementation details
- [ ] Testing strategies

---

## 🎯 SUCCESS METRICS

### Track 1 Metrics
- **Join Time**: < 1 second
- **Service Discovery**: < 500ms
- **Heartbeat Interval**: 30 seconds
- **Failure Detection**: < 60 seconds

### Track 2 Metrics
- **mDNS Discovery**: < 5 seconds
- **Fractal Scalability**: O(1) performance
- **Quorum Consensus**: < 10 seconds
- **Protocol Negotiation**: < 100ms

---

**Status**: Specification approved, implementation starting  
**Next**: Create FEDERATION_IMPLEMENTATION_PROGRESS.md at root  
**Authority**: ecoPrimals Development Team

