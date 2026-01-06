# 🌐 Multi-Primal Interaction Architecture

**Date**: January 7, 2026  
**Status**: 🚀 **READY FOR NESTGATE INTEGRATION**  
**Priority**: **HIGH** - Enable network effects, interaction testing

---

## 🎯 Vision: Network Effect of Sovereign Primals

**Philosophy**: Each primal is **sovereign and standalone**, but gains power through **capability-based interaction** with other primals.

**Example**:
- **BearDog**: Security, encryption, genetic lineage
- **NestGate**: Database, persistent storage
- **Songbird**: Discovery, federation, coordination
- **ToadStool**: ML orchestration (future)
- **Squirrel**: Event streaming (future)

**Network Effect**: More primals = exponentially more value, but **zero n² coupling**!

---

## ✅ **Songbird v3.13.0 Already Supports This!**

### **Universal Primal Adapter** ✅ **COMPLETE**

```rust
// crates/songbird-orchestrator/src/universal_adapter.rs

pub struct UniversalAdapter {
    capability_registry: Arc<CapabilityRegistry>,
    // Discovers ANY primal by capability, not by name!
}

impl UniversalAdapter {
    /// Discover providers for a capability (NO hardcoded primal names!)
    pub async fn discover(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        // Discovery methods (in order):
        // 1. Local capability registry (O(1) lookup)
        // 2. mDNS (local network discovery)
        // 3. DHT (distributed discovery) - future
        // 4. Registry service (if available) - future
    }
    
    /// Call a primal method (protocol-agnostic!)
    pub async fn call(&self, capability: &str, method: &str, params: Value) -> Result<Value> {
        // 1. Discover providers for capability
        // 2. Select best provider (QoS, trust, proximity)
        // 3. Auto-negotiate protocol (tarpc → JSON-RPC → HTTP)
        // 4. Make the call
        // 5. Return result
    }
}
```

**Key Features**:
- ✅ **Zero hardcoding** - discovers primals by capability
- ✅ **Protocol-agnostic** - tarpc/JSON-RPC/HTTP auto-detected
- ✅ **QoS-based selection** - picks best available provider
- ✅ **Fault tolerant** - tries next provider on failure

---

## 📦 **NestGate Integration - Database Primal**

### **NestGate Capabilities**

```yaml
# What NestGate provides
capabilities:
  - "storage/database"
  - "storage/query"
  - "storage/backup"
  - "storage/migration"
  - "data/relational"
  - "data/document"  # If supports JSON documents
  - "data/timeseries" # If supports time-series

# NestGate endpoints
endpoints:
  - protocol: "tarpc"      # PRIMARY
    address: "tarpc://127.0.0.1:7200"
    
  - protocol: "json-rpc"   # SECONDARY
    address: "unix:///tmp/nestgate-{family}-{node}.sock"
    
  - protocol: "http"       # FALLBACK
    address: "http://127.0.0.1:7201"

# QoS metrics
qos:
  latency_ms: 5
  success_rate: 0.999
  availability: 0.99
```

---

## 🎯 **Interaction Patterns**

### **Pattern 1: Songbird ↔ BearDog** ✅ **WORKING**

**Use Case**: Trust evaluation, genetic lineage

```rust
// Songbird discovers BearDog's security capability
let trust_decision = universal_adapter
    .call("security/trust", "evaluate_peer", json!({
        "peer_id": "tower2",
        "peer_tags": ["beardog:family:nat0"]
    }))
    .await?;

// Songbird doesn't know it's BearDog!
// It just knows "security/trust" capability exists
```

**Result**: ✅ Working in v3.13.0

---

### **Pattern 2: Songbird ↔ NestGate** 🚀 **READY TO IMPLEMENT**

**Use Case**: Persist discovered peers, federation state

```rust
// Songbird discovers NestGate's database capability
let result = universal_adapter
    .call("storage/database", "insert", json!({
        "table": "discovered_peers",
        "data": {
            "peer_id": "tower2",
            "node_name": "Tower 2",
            "family_id": "nat0",
            "discovered_at": "2026-01-07T12:00:00Z",
            "trust_level": 1,
            "capabilities": ["birdsong/*", "coordination/*"]
        }
    }))
    .await?;

// Query peers from database
let peers = universal_adapter
    .call("storage/database", "query", json!({
        "table": "discovered_peers",
        "filter": {"family_id": "nat0"},
        "order_by": "discovered_at DESC"
    }))
    .await?;
```

**Benefits**:
- Persistent peer list (survives restarts)
- Historical federation data
- Query capabilities (filter, sort, aggregate)

---

### **Pattern 3: BearDog ↔ NestGate** 🚀 **FUTURE**

**Use Case**: Persist trust relationships, genetic lineage history

```rust
// BearDog stores trust decisions in NestGate
// (via Songbird's capability registry)
let result = universal_adapter
    .call("storage/database", "insert", json!({
        "table": "trust_relationships",
        "data": {
            "evaluator_id": "tower1",
            "peer_id": "tower2",
            "trust_level": 1,
            "reason": "same_genetic_family",
            "evaluated_at": "2026-01-07T12:00:00Z"
        }
    }))
    .await?;
```

---

### **Pattern 4: neuralAPI ↔ NestGate** 🚀 **FUTURE**

**Use Case**: Persist learning data, pathway optimization history

```rust
// neuralAPI stores graph execution results
let result = universal_adapter
    .call("storage/database", "insert", json!({
        "table": "graph_executions",
        "data": {
            "graph_id": "optimization_v1",
            "execution_time_ms": 150,
            "pathway_used": ["songbird", "beardog", "nestgate"],
            "result": "success",
            "timestamp": "2026-01-07T12:00:00Z"
        }
    }))
    .await?;
```

---

## 🔑 **Sovereign Standalone Capabilities**

### **Key Storage (Sovereign Design)**

**Philosophy**: Each primal has its own key storage, **not centralized**.

#### **Option A: Per-Primal Key Storage** ⭐ **RECOMMENDED**

```
~/.config/beardog/keys/
  ├── family_seed.key      # Genetic seed (encrypted with HSM/password)
  ├── node_private.key     # Node's private key
  ├── node_public.key      # Node's public key
  └── trust_policies/
      └── nat0.yaml        # Signed trust policy

~/.config/songbird/keys/
  ├── node_id              # Persistent node ID
  ├── discovery_key.key    # For BirdSong encryption (if used)
  └── contact_keys/
      └── tower2.key       # Shared secret with tower2 (future Phase 3)

~/.config/nestgate/keys/
  ├── database_encryption.key  # Database encryption key
  ├── backup_key.key          # Backup encryption key
  └── access_tokens/
      └── songbird.token      # Token for Songbird to access NestGate
```

**Benefits**:
- ✅ Each primal is **truly sovereign**
- ✅ No single point of failure
- ✅ Keys never leave primal's control
- ✅ Can backup/restore per-primal

**Key Exchange**: Via Songbird's capability registry + trust system

---

#### **Option B: BearDog as Key Custodian** (Not Recommended)

```
~/.config/beardog/keys/
  ├── all_keys/
  │   ├── beardog/...
  │   ├── songbird/...
  │   └── nestgate/...
```

**Problems**:
- ❌ Centralized (defeats sovereignty)
- ❌ BearDog becomes single point of failure
- ❌ Primals can't run without BearDog

**Verdict**: Don't do this. Each primal sovereign!

---

## 🧪 **NestGate Interaction Testing Plan**

### **Phase 1: Discovery & Registration** ⏰ **1-2 days**

**Goal**: Songbird discovers NestGate, registers it in capability registry

**Steps**:
1. NestGate broadcasts its capabilities (mDNS or manual registration)
2. Songbird receives broadcast
3. Songbird registers NestGate in capability registry
4. Query capability registry to verify

**Testing**:
```bash
# Start NestGate
./nestgate-server &

# Start Songbird
./primalBins/songbird-orchestrator &

# Query capabilities
echo '{
  "jsonrpc":"2.0",
  "method":"capabilities.list",
  "params":{"capability":"storage/database"},
  "id":1
}' | nc -U /tmp/songbird-nat0-tower1.sock | jq

# Expected result:
# {
#   "result": {
#     "providers": [
#       {
#         "provider_id": "nestgate-tower1",
#         "capabilities": ["storage/database", "storage/query"],
#         "endpoints": [
#           {"protocol": "tarpc", "address": "tarpc://127.0.0.1:7200"}
#         ]
#       }
#     ]
#   }
# }
```

---

### **Phase 2: Basic Interaction** ⏰ **2-3 days**

**Goal**: Songbird stores peer data in NestGate

**Implementation**:
```rust
// crates/songbird-orchestrator/src/app/discovery_bridge.rs

async fn store_discovered_peer_in_database(
    &self,
    peer: &DiscoveredPeer,
) -> Result<()> {
    // Discover database capability
    let adapter = UniversalAdapter::new(self.capability_registry.clone());
    
    // Store peer
    adapter.call("storage/database", "insert", json!({
        "table": "discovered_peers",
        "data": {
            "peer_id": peer.node_id.clone(),
            "node_name": peer.node_name.clone(),
            "family_id": peer.tags.as_ref()
                .and_then(|tags| tags.iter()
                    .find(|t| t.contains("family:"))
                    .map(|t| t.split(':').nth(2).unwrap_or("unknown"))),
            "discovered_at": chrono::Utc::now().to_rfc3339(),
            "trust_level": 1,
        }
    })).await?;
    
    Ok(())
}
```

**Testing**:
```bash
# Discover peer (triggers database store)
# ... peer discovery happens ...

# Query NestGate directly
echo '{
  "jsonrpc":"2.0",
  "method":"query",
  "params":{
    "table":"discovered_peers",
    "filter":{"family_id":"nat0"}
  },
  "id":1
}' | nc -U /tmp/nestgate-nat0-tower1.sock | jq

# Verify peer data is persisted
```

---

### **Phase 3: Trust-Based Access Control** ⏰ **3-4 days**

**Goal**: Only trusted primals can access NestGate

**Implementation**:
```rust
// NestGate checks trust before allowing operations

impl NestGateServer {
    async fn handle_insert(&self, request: InsertRequest, caller_id: &str) -> Result<()> {
        // Query trust level via Songbird
        let trust = self.query_trust_level(caller_id).await?;
        
        match trust.level {
            TrustLevel::None => {
                return Err(Error::Unauthorized("No trust relationship"));
            }
            TrustLevel::Limited => {
                // Can read, cannot write
                if request.table != "public_data" {
                    return Err(Error::Unauthorized("Limited trust - read only"));
                }
            }
            TrustLevel::Elevated | TrustLevel::Highest => {
                // Full access
            }
        }
        
        // Perform operation
        self.database.insert(&request.table, &request.data).await
    }
}
```

---

### **Phase 4: Multi-Primal Workflows** ⏰ **1 week**

**Goal**: Complex workflows involving Songbird, BearDog, and NestGate

**Example Workflow**: Federated peer discovery with persistence

```rust
async fn discover_and_trust_peer(
    peer_id: &str,
) -> Result<FederatedPeer> {
    // 1. Songbird discovers peer (UDP multicast)
    let peer = songbird.discover_peer(peer_id).await?;
    
    // 2. Songbird asks BearDog to evaluate trust
    let trust = beardog.evaluate_trust(&peer).await?;
    
    // 3. If trusted, Songbird stores in NestGate
    if trust.level >= TrustLevel::Limited {
        nestgate.store_peer(&peer, &trust).await?;
    }
    
    // 4. Songbird establishes federation connection
    songbird.add_to_federation(&peer, trust.level).await?;
    
    Ok(FederatedPeer { peer, trust })
}
```

**Testing**:
- Deploy 2 towers with Songbird + BearDog + NestGate
- Trigger discovery
- Verify end-to-end workflow
- Check persistence across restarts

---

## 📋 **NestGate Integration Checklist**

### **For NestGate Team**:

#### **1. Capability Registration** ⏰ **1 day**
- [ ] Define capabilities (`storage/database`, `storage/query`, etc.)
- [ ] Broadcast capabilities via mDNS (or manual registration)
- [ ] Implement JSON-RPC 2.0 endpoint (Unix socket)
- [ ] Implement tarpc endpoint (optional, for performance)

#### **2. API Endpoints** ⏰ **2 days**
- [ ] `insert` - Insert data into table
- [ ] `query` - Query data with filters
- [ ] `update` - Update existing data
- [ ] `delete` - Delete data
- [ ] `create_table` - Schema management
- [ ] `health` - Health check

#### **3. Trust Integration** ⏰ **2 days**
- [ ] Accept caller_id in requests
- [ ] Query Songbird for trust level
- [ ] Enforce access control based on trust
- [ ] Audit logging for sensitive operations

#### **4. Testing** ⏰ **2 days**
- [ ] Unit tests for each endpoint
- [ ] Integration tests with Songbird
- [ ] Load testing (concurrent requests)
- [ ] Failure recovery testing

---

### **For Songbird Team** (Me!):

#### **1. Capability Registry Enhancements** ⏰ **1 day**
- [ ] Ensure `storage/database` capability is recognized
- [ ] Add database-specific QoS metrics
- [ ] Test multi-provider selection (if multiple databases)

#### **2. Discovery Bridge Updates** ⏰ **1 day**
- [ ] Optional: Store discovered peers in NestGate
- [ ] Query NestGate for historical peer data
- [ ] Fallback if NestGate unavailable (in-memory only)

#### **3. Documentation** ⏰ **1 day**
- [ ] Multi-primal interaction guide
- [ ] NestGate integration examples
- [ ] Testing procedures

---

## 🎯 **Timeline**

| Phase | Duration | Team | Deliverable |
|-------|----------|------|-------------|
| **Capability Definition** | 1 day | NestGate | Capability YAML |
| **JSON-RPC Server** | 2 days | NestGate | Basic API working |
| **Discovery Integration** | 1 day | Songbird | Auto-discovery working |
| **Basic Interaction** | 2 days | Both | Store/query working |
| **Trust Integration** | 2 days | Both | Access control working |
| **E2E Testing** | 2 days | Both | Full workflow verified |

**Total**: ~10 days (2 weeks) for full integration

---

## 🚀 **Quick Start for NestGate Team**

### **Step 1: Define Capabilities**

```yaml
# nestgate-capabilities.yaml
service_name: "nestgate"
service_id: "nestgate-tower1"
version: "1.0.0"

capabilities:
  - name: "storage/database"
    description: "Relational database storage"
    methods:
      - "insert"
      - "query"
      - "update"
      - "delete"
  
  - name: "storage/query"
    description: "Advanced query with filters"
    methods:
      - "query"
      - "aggregate"

endpoints:
  - protocol: "json-rpc"
    address: "unix:///tmp/nestgate-{family}-{node}.sock"
    preferred: true
  
  - protocol: "tarpc"
    address: "tarpc://127.0.0.1:7200"
    preferred: false

qos:
  latency_p50_ms: 5
  latency_p99_ms: 20
  success_rate: 0.999
  availability: 0.99
```

---

### **Step 2: Implement JSON-RPC Server**

```rust
// Example NestGate JSON-RPC server

use serde_json::{json, Value};
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<()> {
    let socket_path = format!("/tmp/nestgate-{}-{}.sock", 
        env::var("FAMILY_ID")?, 
        env::var("NODE_ID")?
    );
    
    let listener = UnixListener::bind(&socket_path)?;
    
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: UnixStream) -> Result<()> {
    // Read JSON-RPC request
    let request: JsonRpcRequest = read_request(&stream).await?;
    
    // Route to handler
    let result = match request.method.as_str() {
        "insert" => handle_insert(request.params).await,
        "query" => handle_query(request.params).await,
        "health" => Ok(json!({"status": "healthy"})),
        _ => Err(Error::MethodNotFound),
    };
    
    // Send JSON-RPC response
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result,
        id: request.id,
    };
    write_response(&stream, &response).await?;
    
    Ok(())
}
```

---

### **Step 3: Register with Songbird**

```bash
# Manual registration (for testing)
echo '{
  "jsonrpc":"2.0",
  "method":"capabilities.register",
  "params":{
    "provider_id": "nestgate-tower1",
    "capabilities": ["storage/database", "storage/query"],
    "endpoints": [
      {
        "protocol": "json-rpc",
        "address": "unix:///tmp/nestgate-nat0-tower1.sock"
      }
    ],
    "qos": {
      "latency_ms": 5,
      "success_rate": 0.999
    }
  },
  "id":1
}' | nc -U /tmp/songbird-nat0-tower1.sock | jq
```

---

## 🎊 **Summary**

### **Ready for NestGate**:
✅ **Songbird v3.13.0** has all infrastructure needed  
✅ **Universal Adapter** discovers any primal by capability  
✅ **Protocol-agnostic** communication working  
✅ **Capability registry** O(1) lookup  

### **NestGate Integration**:
📅 **2 weeks** for full integration  
🎯 **Network effect** starts with 3 primals (Songbird, BearDog, NestGate)  
🚀 **Sovereign design** - each primal has own key storage  

### **Philosophy Validated**:
> "Each primal is sovereign and standalone, but gains exponential power through capability-based interaction with other primals."

---

🎉 **READY FOR MULTI-PRIMAL NETWORK EFFECTS!** 🌐

**NestGate team**: See quick start guide above. Songbird v3.13.0 is ready to discover and interact with you! 🚀

